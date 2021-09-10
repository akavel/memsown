use std::fs::{read, File};
use std::io::{Cursor, Write};
use std::path::Path;

use chrono::naive::{NaiveDate, NaiveDateTime};
use exif::{Exif, Reader};
use globwalk::GlobWalkerBuilder;
use image::io::Reader as ImageReader;
use image::imageops::FilterType;
use path_slash::PathExt;
use rusqlite::{params, Connection as DbConnection};
use sha1::{Sha1, Digest};

fn main() {
    // TODO[LATER]: run rustfmt on this repo
    // TODO[LATER]: run clippy on this repo
    println!("Hello, world!");

    let raw_stdout = std::io::stdout();
    let mut stdout = raw_stdout.lock();

    let db = DbConnection::open("backer.db").unwrap();
    db_init(&db);

    // mdb = openDb("backer.db")
    // let markers = @[
    //   r"d:\backer-id.json",
    //   r"c:\fotki\backer-id.json",
    // ]

    let root = r"c:\fotki";

    let marker = marker_read(root);
    println!("marker {}", &marker);

    // FIXME: Stage 1: add not-yet-known files into DB
    let images = GlobWalkerBuilder::new(&root, "*.{jpg,jpeg}")
        .case_insensitive(true)
        .file_type(globwalk::FileType::FILE)
        .build();
    for entry in images.unwrap() {
        // TODO[LATER]: use `?` instead of .unwrap() and ret. some err from main() or print error info
        let path = entry.unwrap().path().to_owned();
        let buf = read(&path).unwrap();

        let relative = path.strip_prefix(root).unwrap().to_slash().unwrap();
        if db_exists(&db, &marker, &relative) {
            stdout.write_all(b".").unwrap();
            stdout.flush().unwrap();
            continue;
        }


        // Calculate sha1 hash of the file contents.
        // TODO[LATER]: maybe switch to a secure hash (sha2 or other, see: https://github.com/RustCrypto/hashes)
        let hash = format!("{:x}", Sha1::digest(&buf));

        // Extract some info from JPEG's Exif metadata
        let exif = Reader::new().read_from_container(&mut Cursor::new(&buf)).unwrap();
        // TODO[LATER]: extract date from other Exif fields or filename
        let date = exif_date(&exif).unwrap_or(::exif::DateTime{
            year: 0, month: 0, day: 0, hour: 0, minute: 0, second: 0,
            nanosecond: None, offset: None,
        });
        let orient = exif_orientation(&exif);
        // TODO: test exif deorienting with cases from: https://github.com/recurser/exif-orientation-examples
        // (see also: https://www.daveperrett.com/articles/2012/07/28/exif-orientation-handling-is-a-ghetto)

    // FIXME:    - create 200x200 thumbnail
    // FIXME:       - lanczos resizing
    // FIXME:       - deoriented
        let img = ImageReader::new(Cursor::new(&buf)).with_guessed_format().unwrap().decode().unwrap();
        // let thumb = img.resize(200, 200, FilterType::Lanczos3);
        let thumb = img.resize(200, 200, FilterType::CatmullRom);
        thumb.save("tmp.jpg").unwrap();
        let mut thumb_jpeg = Vec::<u8>::new();
        thumb.write_to(&mut thumb_jpeg, image::ImageOutputFormat::Jpeg(25)).unwrap();

        let info = FileInfo{
            hash: hash.clone(),
            date: NaiveDate::from_ymd(date.year.into(), date.month.into(), date.day.into())
                .and_hms(date.hour.into(), date.minute.into(), date.second.into()),
            thumb: thumb_jpeg,
        };
        db_upsert(&db, &marker, &relative, &info);

        println!("{} {} {:?} {:?}", &hash, path.display(), date.to_string(), orient);
    }

    // FIXME: Stage 2: scan all files once more and refresh them in DB
}

fn marker_read(dir: &str) -> String {
    // FIXME[LATER]: return some Result instead of unwrapping
    use serde::Deserialize;
    #[derive(Deserialize)]
    struct Marker {
        id: String,
    }

    let file = File::open(Path::new(dir).join("backer-id.json")).unwrap();
    let reader = std::io::BufReader::new(file);
    let m: Marker = serde_json::from_reader(reader).unwrap();
    m.id
}

fn db_init(db: &DbConnection) {
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
      ").unwrap();
}

fn db_exists(db: &DbConnection, marker: &str, relative: &str) -> bool {
    db.query_row("
        SELECT COUNT(*) FROM location
          WHERE backend_tag = ?
          AND path = ?",
        params![marker, relative],
        |row| row.get(0),
    ).unwrap()
}

struct FileInfo {
    hash: String,
    date: NaiveDateTime,
    thumb: Vec<u8>,
}

fn db_upsert(db: &DbConnection, marker: &str, relative: &str, info: &FileInfo) {
    db.execute("
        INSERT INTO file(hash,date,thumbnail) VALUES(?,?,?)
          ON CONFLICT(hash) DO UPDATE SET
            date = ifnull(date, excluded.date)",
        params![&info.hash, &info.date, &info.thumb],
    ).unwrap();
    db.execute("
        INSERT INTO location(file_id,backend_tag,path)
          SELECT rowid, ?, ? FROM file
            WHERE hash = ? LIMIT 1;",
        params![&marker, &relative, &info.hash],
    ).unwrap();
}

fn exif_date(exif: &Exif) -> Option<::exif::DateTime> {
    use exif::{DateTime, Field, In, Tag, Value};

    match exif.get_field(Tag::DateTime, In::PRIMARY) {
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
