use std::fs::{read, File};
use std::io::{Cursor, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};

use anyhow::{anyhow, Context, Result};
use chrono::naive::{NaiveDate, NaiveDateTime};
use exif::{Exif, Reader as ExifReader};
use globwalk::GlobWalkerBuilder;
use image::imageops::FilterType;
use image::io::Reader as ImageReader;
use path_slash::PathExt;
use rusqlite::{params, Connection as DbConnection};
use sha1::{Digest, Sha1};

fn main() -> Result<()> {
    // TODO[LATER]: run rustfmt on this repo
    // TODO[LATER]: run clippy on this repo
    println!("Hello, world!");

    let raw_stdout = std::io::stdout();
    let mut stdout = raw_stdout.lock();

    // TODO[LATER]: use Arc<RwLock<T>> instead of Arc<Mutex<T>>
    let db = DbConnection::open("backer.db")?;
    db_init(&db)?;
    let db = Arc::new(RwLock::new(db));

    // TODO[LATER]: load from JSON more or less: {"disk":["d:\\backer-id.json","c:\\fotki\\backer-id.json"],"ipfs":[...]}
    let marker_paths = vec![
        r"d:\backer-id.json",
        r"c:\fotki\backer-id.json",
    ];

    // FIXME: Milestone: read from multiple marker roots in parallel
    for (i, marker) in marker_paths.iter().enumerate() {
        // TODO[LATER]: extract loop contents into a fn for readability

        let (root, marker) = match marker_read(marker) {
            Ok(m) => Ok(m),
            Err(err) => {
                if let Some(cause) = err.downcast_ref::<std::io::Error>() {
                    if cause.kind() == std::io::ErrorKind::NotFound {
                        println!("\nSkipping tree at '{}': {}", marker, error_chain(&err));
                        continue;
                    }
                }
                Err(err)
            },
        }?;
        println!("marker {} at: {}", &marker, root.display());

        // Stage 1: add not-yet-known files into DB
        // TODO[LATER]: in parallel thread, count all matching files, then when done start showing progress bar/percentage
        let images = GlobWalkerBuilder::new(&root, "*.{jpg,jpeg}")
            .case_insensitive(true)
            .file_type(globwalk::FileType::FILE)
            .build();
        for entry in images? {
            let path = entry?.path().to_owned();
            let buf = read(&path)?;

            let os_relative = path.strip_prefix(&root)?;
            let relative = os_relative
                .to_slash()
                .with_context(|| format!("Failed to convert path {:?} to slash-based", os_relative))?;
            let db_read = db.read().unwrap();
            if db_exists(&db_read, &marker, &relative)? {
                stdout.write_all(b".")?;
                stdout.flush()?;
                continue;
            }
            drop(db_read);

            // Calculate sha1 hash of the file contents.
            // TODO[LATER]: maybe switch to a secure hash (sha2 or other, see: https://github.com/RustCrypto/hashes)
            let hash = format!("{:x}", Sha1::digest(&buf));

            // Does the JPEG have Exif block? We assume it'd be the most reliable source of metadata.
            let exif = ExifReader::new()
                .read_from_container(&mut Cursor::new(&buf))
                .ok();
            let date = try_deduce_date(exif.as_ref(), &relative);
            // // TODO[LATER]: use some orientation enum / stricter type instead of raw u16
            // // TODO[LATER]: test exif deorienting with cases from: https://github.com/recurser/exif-orientation-examples
            // // (see also: https://www.daveperrett.com/articles/2012/07/28/exif-orientation-handling-is-a-ghetto)
            // let orientation = exif.as_ref().and_then(exif_orientation).unwrap_or(1);

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
            let db_write = db.write().unwrap();
            db_upsert(&db_write, &marker, &relative, &info)?;
            drop(db_write);

            print!("{}", i);
            stdout.flush()?;
            // println!("{} {} {:?} {:?}", &hash, path.display(), date.map(|d| d.to_string()), orientation);
        }
    }


    // FIXME: Stage 2: check if all files from DB are present on disk, delete entries for any missing

    // FIXME: Stage 3: scan all files once more and refresh them in DB

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
    let m: Marker = serde_json::from_reader(std::io::BufReader::new(file))?;

    Ok((parent.to_owned(), m.id))
}

fn error_chain(err: &anyhow::Error) -> String {
    err.chain().into_iter().map(|e| e.to_string()).collect::<Vec<String>>().join(": ")
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
        if let Some(d) = exif_date_from(exif, Tag::DateTime) {
            return Some(exif_date_to_naive(&d));
        }
        // TODO[LATER]: does this field make sense?
        if let Some(d) = exif_date_from(exif, Tag::DateTimeOriginal) {
            return Some(exif_date_to_naive(&d));
        }
        // TODO[LATER]: are ther other fields we could try?
    }
    // TODO[LATER]: try extracting date from relative_path
    // TODO[LATER]: try extracting date from file's creation and modification date (NOTE: latter can be earlier than former on Windows!)
    None
}

fn exif_date_to_naive(d: &::exif::DateTime) -> NaiveDateTime {
    NaiveDate::from_ymd(d.year.into(), d.month.into(), d.day.into()).and_hms(
        d.hour.into(),
        d.minute.into(),
        d.second.into(),
    )
}

fn exif_date_from(exif: &Exif, tag: ::exif::Tag) -> Option<::exif::DateTime> {
    use exif::{DateTime, Field, In, Value};

    match exif.get_field(tag, In::PRIMARY) {
        Some(Field {
            value: Value::Ascii(ref vec),
            ..
        }) if !vec.is_empty() => DateTime::from_ascii(&vec[0]).ok(),
        _ => None,
    }
}

// // TODO: for meaning, see: https://magnushoff.com/articles/jpeg-orientation/
// fn exif_orientation(exif: &Exif) -> Option<u16> {
//     use exif::{Field, In, Tag, Value};

//     match exif.get_field(Tag::Orientation, In::PRIMARY) {
//         Some(Field{value: Value::Short(ref vec), ..})
//             if !vec.is_empty() => Some(vec[0]),
//         _ => None
//     }
// }
