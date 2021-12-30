use std::fs::{read, File};
use std::io::{self, Cursor, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::{anyhow, Context, Result};
use chrono::naive::NaiveDateTime;
use exif::{Exif, Reader as ExifReader};
use globwalk::GlobWalkerBuilder;
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use path_slash::PathExt;
use rusqlite::{params, Connection as DbConnection};
use sha1::{Digest, Sha1};

use backer::imaging::*;

// TODO[LATER]: load marker_paths from JSON
// TODO: load date-from-path regexps from JSON:
//   {"paths-to-dates": {"sf7-c-fotki": {".*(20\d\d)-(\d\d)-(\d\d)": "\1-\2-\3"}}}
// TODO: introduce "dry-run" mode for testing date-from-path regexps
// - [LATER] show which rule index detected date in which file (also Exif)
// - [LATER] also show files that were still "missed" by detection
// TODO: use date-from-path regexps
// TODO: skip small images (threshold size configurable in JSON - dimensions or bytes?)
// - or maybe just skip them in viewer for now?
// TODO: re-scan disks and re-generate data in DB
// - for above step, we should probably have an iterator easily generating file paths from marker
//   path (similar as already exists for initial scan, but extracted for reuse)
// TODO: delete files not found on disk from DB (only locations - keep "files")
// - for above step, we need to be able to easily check if specific path exists at location in
//   specific marker's tree
//   - it should use same filters as the main files iterator (incl. extension, jpeg size)
// TODO: merge 'view' and 'main' binaries
// TODO: deorient thumbnails stored into DB

fn main() -> Result<()> {
    // TODO[LATER]: run rustfmt on this repo
    // TODO[LATER]: run clippy on this repo
    println!("Hello, world!");

    let db = DbConnection::open("backer.db")?;
    db_init(&db)?;
    let db = Arc::new(Mutex::new(db));

    // TODO[LATER]: load from JSON more or less: {"disk":["d:\\backer-id.json","c:\\fotki\\backer-id.json"],"ipfs":[...]}
    #[rustfmt::skip]
    let marker_paths = vec![
        r"d:\backer-id.json",
        r"c:\fotki\backer-id.json",
    ];

    // TODO[LATER]: consider using 'rayon' lib for prettier parallelism
    let mut threads = vec![];
    for (i, marker) in marker_paths.iter().enumerate() {
        let db = db.clone();
        let marker = marker.to_owned();
        threads.push(thread::spawn(move || process_tree(i, marker, db).unwrap()));
    }
    for t in threads {
        t.join().unwrap();
    }

    // FIXME: Stage 2: check if all files from DB are present on disk, delete entries for any missing

    // FIXME: Stage 3: scan all files once more and refresh them in DB

    Ok(())
}

// TODO[LATER]: accept Path (or Into<Path>/From<Path>)
fn process_tree(i: usize, marker_path: &str, db: Arc<Mutex<DbConnection>>) -> Result<()> {
    let m = marker_read(marker_path);
    if check_io_error(&m) == Some(io::ErrorKind::NotFound) {
        println!(
            "\nSkipping tree at '{}': {}",
            marker_path,
            error_chain(&m.unwrap_err())
        );
        return Ok(());
    }
    let (root, marker) = m?;
    println!("marker {} at: {}", &marker, root.display());

    // Stage 1: add not-yet-known files into DB
    // TODO[LATER]: in parallel thread, count all matching files, then when done start showing progress bar/percentage
    let images = GlobWalkerBuilder::new(&root, "*.{jpg,jpeg}")
        .case_insensitive(true)
        .file_type(globwalk::FileType::FILE)
        .build();
    for entry in images? {
        let path = match entry {
            Ok(entry) => entry.path().to_owned(),
            Err(err) => {
                eprintln!("\nFailed to access file, skipping: {}", err);
                continue;
            }
        };
        let buf = read(&path)?;

        let os_relative = path.strip_prefix(&root)?;
        let relative = os_relative
            .to_slash()
            .with_context(|| format!("Failed to convert path {:?} to slash-based", os_relative))?;
        let db_read = db.lock().unwrap();
        if db_exists(&db_read, &marker, &relative)? {
            print!(".");
            io::stdout().flush()?;
            continue;
        }
        drop(db_read);

        // Calculate sha1 hash of the file contents.
        // TODO[LATER]: maybe switch to a secure hash (sha2 or other, see: https://github.com/RustCrypto/hashes)
        let hash = format!("{:x}", Sha1::digest(&buf));

        // FIXME: if image is very small, it's probably a thumbnail already and we don't want to archive it

        // Does the JPEG have Exif block? We assume it'd be the most reliable source of metadata.
        let exif = ExifReader::new()
            .read_from_container(&mut Cursor::new(&buf))
            .ok();
        let date = try_deduce_date(exif.as_ref(), &relative);
        // // TODO[LATER]: use some orientation enum / stricter type instead of raw u16
        // let orientation = exif.as_ref().and_then(|v| v.orientation()).unwrap_or(1);

        let img = match ImageReader::new(Cursor::new(&buf))
            .with_guessed_format()?
            .decode()
        {
            Ok(img) => img,
            Err(err) => {
                // TODO[LATER]: use termcolor crate to print errors in red
                // FIXME[LATER]: resolve JPEG decoding error: "spectral selection is not allowed in non-progressive scan"
                eprintln!("\nFailed to decode JPEG {:?}, skipping: {}", &path, err);
                continue;
            }
        };
        // let thumb = img.resize(200, 200, FilterType::Lanczos3);
        let thumb = img.resize(200, 200, FilterType::CatmullRom);
        // FIXME[LATER]: fix the thumbnail's orientation
        let mut thumb_jpeg = Vec::<u8>::new();
        thumb.write_to(&mut thumb_jpeg, image::ImageOutputFormat::Jpeg(90))?;

        let info = backer::model::FileInfo {
            hash: hash.clone(),
            date,
            thumb: thumb_jpeg,
        };
        let db_write = db.lock().unwrap();
        db_upsert(&db_write, &marker, &relative, &info)?;
        drop(db_write);

        print!("{}", i);
        io::stdout().flush()?;
        // println!("{} {} {:?} {:?}", &hash, path.display(), date.map(|d| d.to_string()), orientation);
    }

    Ok(())
}

// TODO[LATER]: accept Path and return Result<(Path,...)> with proper lifetime
fn marker_read(file_path: &str) -> Result<(PathBuf, String)> {
    let file_path = Path::new(file_path);
    let parent = file_path.parent().ok_or_else(|| {
        anyhow!(
            "Could not split parent directory of '{}'",
            file_path.display()
        )
    })?;

    use serde::Deserialize;
    #[derive(Deserialize)]
    struct Marker {
        id: String,
    }
    let file = File::open(file_path)
        .with_context(|| format!("Failed to open '{}'", file_path.display()))?;
    let m: Marker = serde_json::from_reader(io::BufReader::new(file))?;

    Ok((parent.to_owned(), m.id))
}

fn check_io_error<T>(result: &Result<T>) -> Option<io::ErrorKind> {
    result
        .as_ref()
        .err()
        .and_then(|err| err.downcast_ref::<io::Error>())
        .map(|cause| cause.kind())
}

fn error_chain(err: &anyhow::Error) -> String {
    err.chain()
        .into_iter()
        .map(|e| e.to_string())
        .collect::<Vec<String>>()
        .join(": ")
}

fn db_init(db: &DbConnection) -> ::rusqlite::Result<()> {
    db.execute_batch(
        "
          CREATE TABLE IF NOT EXISTS file (
            hash TEXT UNIQUE NOT NULL
              CHECK(length(hash) > 0),
            date TEXT,
            thumbnail BLOB
          );
          CREATE INDEX IF NOT EXISTS file_date ON file(date);

          CREATE TABLE IF NOT EXISTS location (
            file_id INTEGER NOT NULL,
            backend_tag STRING NOT NULL,
            path STRING NOT NULL
          );
          CREATE INDEX IF NOT EXISTS
            location_fileID ON location (file_id);
          CREATE UNIQUE INDEX IF NOT EXISTS
            location_perBackend ON location (backend_tag, path);
        ",
    )
}

fn db_exists(db: &DbConnection, marker: &str, relative: &str) -> ::rusqlite::Result<bool> {
    db.query_row(
        "SELECT COUNT(*) FROM location
            WHERE backend_tag = ?
            AND path = ?",
        params![marker, relative],
        |row| row.get(0),
    )
}

fn db_upsert(
    db: &DbConnection,
    marker: &str,
    relative: &str,
    info: &backer::model::FileInfo,
) -> Result<()> {
    db.execute(
        "INSERT INTO file(hash,date,thumbnail) VALUES(?,?,?)
            ON CONFLICT(hash) DO UPDATE SET
                date = ifnull(date, excluded.date)",
        params![&info.hash, &info.date, &info.thumb],
    )?;
    db.execute(
        "INSERT INTO location(file_id,backend_tag,path)
            SELECT rowid, ?, ? FROM file
              WHERE hash = ? LIMIT 1;",
        params![&marker, &relative, &info.hash],
    )?;
    Ok(())
}

/// Try hard to find out some datetime info from either `exif` data, or `relative_path` of the file.
fn try_deduce_date(exif: Option<&Exif>, relative_path: &str) -> Option<NaiveDateTime> {
    if let Some(exif) = exif {
        use exif::Tag;
        // TODO[LATER]: are ther other fields we could try?
        if let Some(d) = vec![Tag::DateTime, Tag::DateTimeOriginal]
            .into_iter()
            .filter_map(|tag| exif.datetime(tag))
            .filter_map(|dt| dt.to_naive_opt())
            .next()
        {
            return Some(d);
        }
    }
    // TODO[LATER]: try extracting date from relative_path
    // TODO[LATER]: try extracting date from file's creation and modification date (NOTE: latter can be earlier than former on Windows!)
    None
}
