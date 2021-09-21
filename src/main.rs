use std::fs::{read, File};
use std::io::{Cursor, Write};
use std::path::Path;

use anyhow::{Context, Result};
use chrono::naive::{NaiveDate, NaiveDateTime};
use exif::{Exif, Reader as ExifReader};
use globwalk::GlobWalkerBuilder;
use image::io::Reader as ImageReader;
use image::imageops::FilterType;
use path_slash::PathExt;
use rusqlite::{params, Connection as DbConnection};
use sha1::{Sha1, Digest};

fn main() -> Result<()> {
    // TODO[LATER]: run rustfmt on this repo
    // TODO[LATER]: run clippy on this repo
    println!("Hello, world!");

    let raw_stdout = std::io::stdout();
    let mut stdout = raw_stdout.lock();

    let db = DbConnection::open("backer.db")?;
    db_init(&db)?;

    // mdb = openDb("backer.db")
    // let markers = @[
    //   r"d:\backer-id.json",
    //   r"c:\fotki\backer-id.json",
    // ]

    let root = r"c:\fotki";

    let marker = marker_read(root)?;
    println!("marker {}", &marker);

    // Stage 1: add not-yet-known files into DB
    let images = GlobWalkerBuilder::new(&root, "*.{jpg,jpeg}")
        .case_insensitive(true)
        .file_type(globwalk::FileType::FILE)
        .build();
    for entry in images? {
        let path = entry?.path().to_owned();
        let buf = read(&path)?;

        let os_relative = path.strip_prefix(root)?;
        let relative = os_relative.to_slash()
            .with_context(|| format!("Failed to convert path {:?} to slash-based", os_relative))?;
        if db_exists(&db, &marker, &relative)? {
            stdout.write_all(b".")?;
            stdout.flush()?;
            continue;
        }


        // Calculate sha1 hash of the file contents.
        // TODO[LATER]: maybe switch to a secure hash (sha2 or other, see: https://github.com/RustCrypto/hashes)
        let hash = format!("{:x}", Sha1::digest(&buf));

        // Does the JPEG have Exif block? We assume it'd be the most reliable source of metadata.
        let exif = ExifReader::new().read_from_container(&mut Cursor::new(&buf)).ok();
        // TODO[LATER]: use some orientation enum / stricter type instead of raw u16
        // TODO[LATER]: test exif deorienting with cases from: https://github.com/recurser/exif-orientation-examples
        // (see also: https://www.daveperrett.com/articles/2012/07/28/exif-orientation-handling-is-a-ghetto)
        let orientation = exif.as_ref().and_then(exif_orientation).unwrap_or(1);
        let date = try_deduce_date(exif.as_ref(), &relative);

    // FIXME:    - create 200x200 thumbnail
    // FIXME:       - lanczos resizing
    // FIXME:       - deoriented
        let img = match ImageReader::new(Cursor::new(&buf)).with_guessed_format()?.decode() {
            Ok(img) => img,
            Err(err) => {
                // TODO[LATER]: use termcolor crate to print errors in red
                eprintln!("\nFailed to decode JPEG {:?}, skipping: {}", &path, err);
                continue;
            }
        };
        // let img = ImageReader::new(Cursor::new(&buf)).with_guessed_format()?.decode()
        //     .with_context(|| format!("Failed to decode JPEG {:?}", &path))?;
        // let thumb = img.resize(200, 200, FilterType::Lanczos3);
        let thumb = img.resize(200, 200, FilterType::CatmullRom);
        let mut thumb_jpeg = Vec::<u8>::new();
        thumb.write_to(&mut thumb_jpeg, image::ImageOutputFormat::Jpeg(25))?;

        let info = backer::model::FileInfo{
            hash: hash.clone(),
            date: date,
            thumb: thumb_jpeg,
        };
        db_upsert(&db, &marker, &relative, &info)?;

        stdout.write_all(b"+")?;
        stdout.flush()?;
        // println!("{} {} {:?} {:?}", &hash, path.display(), date.map(|d| d.to_string()), orientation);
    }

    // FIXME: Stage 2: scan all files once more and refresh them in DB

    Ok(())
}

fn marker_read(dir: &str) -> Result<String> {
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct Marker {
        id: String,
    }

    let file = File::open(Path::new(dir).join("backer-id.json"))?;
    let reader = std::io::BufReader::new(file);
    let m: Marker = serde_json::from_reader(reader)?;
    Ok(m.id)
}

fn db_init(db: &DbConnection) -> ::rusqlite::Result<()> {
    db.execute_batch("
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
      ")
}

fn db_exists(db: &DbConnection, marker: &str, relative: &str) -> ::rusqlite::Result<bool> {
    db.query_row("
        SELECT COUNT(*) FROM location
          WHERE backend_tag = ?
          AND path = ?",
        params![marker, relative],
        |row| row.get(0),
    )
}

fn db_upsert(db: &DbConnection, marker: &str, relative: &str, info: &backer::model::FileInfo) -> Result<()> {
    db.execute("
        INSERT INTO file(hash,date,thumbnail) VALUES(?,?,?)
          ON CONFLICT(hash) DO UPDATE SET
            date = ifnull(date, excluded.date)",
        params![&info.hash, &info.date, &info.thumb],
    )?;
    db.execute("
        INSERT INTO location(file_id,backend_tag,path)
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
        if let Some(d) = exif_date_from(&exif, Tag::DateTime) {
            return Some(exif_date_to_naive(&d));
        }
        // TODO[LATER]: does this field make sense?
        if let Some(d) = exif_date_from(&exif, Tag::DateTimeOriginal) {
            return Some(exif_date_to_naive(&d));
        }
        // TODO[LATER]: are ther other fields we could try?
    }
    // TODO[LATER]: try extracting date from relative_path
    // TODO[LATER]: try extracting date from file's creation and modification date (NOTE: latter can be earlier than former on Windows!)
    None
}

fn exif_date_to_naive(d: &::exif::DateTime) -> NaiveDateTime {
    NaiveDate::from_ymd(d.year.into(), d.month.into(), d.day.into())
        .and_hms(d.hour.into(), d.minute.into(), d.second.into())
}

fn exif_date_from(exif: &Exif, tag: ::exif::Tag) -> Option<::exif::DateTime> {
    use exif::{DateTime, Field, In, Value};

    match exif.get_field(tag, In::PRIMARY) {
        Some(Field{value: Value::Ascii(ref vec), ..})
            if !vec.is_empty() => {
                DateTime::from_ascii(&vec[0]).ok()
            }
        _ => None
    }
}

// TODO: for meaning, see: https://magnushoff.com/articles/jpeg-orientation/
fn exif_orientation(exif: &Exif) -> Option<u16> {
    use exif::{Field, In, Tag, Value};

    match exif.get_field(Tag::Orientation, In::PRIMARY) {
        Some(Field{value: Value::Short(ref vec), ..})
            if !vec.is_empty() => Some(vec[0]),
        _ => None
    }
}
