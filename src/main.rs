use std::ffi::OsStr;
use std::fs::read;
use std::io::Cursor;

use exif::{Exif, Reader};
use walkdir::WalkDir;

fn main() {
    println!("Hello, world!");

    // mdb = openDb("backer.db")
    // let markers = @[
    //   r"d:\backer-id.json",
    //   r"c:\fotki\backer-id.json",
    // ]

    // FIXME: Stage 1: add not-yet-known files into DB
    for entry in WalkDir::new(r"c:\fotki") {
        // TODO[LATER]: use `?` instead of .unwrap() and ret. some err from main() or print error info
        let f = entry.unwrap();

        // We're interested only in files, and only with .jpg/.jpeg extension
        // TODO[LATER]: simplify this, through some iterator expression maybe + 'globset' crate [or 'wax' crate]
        if !f.file_type().is_file() { continue; }
        let ext = f.path().extension().map(OsStr::to_str).flatten().map(str::to_ascii_lowercase);
        match ext.as_deref() {
            Some("jpg") | Some("jpeg") => (),
            _ => continue
        }
        let buf = read(f.path()).unwrap();

    // FIXME:    - calc sha1 hash

        // Extract some info from JPEG's Exif metadata
        let exif = Reader::new().read_from_container(&mut Cursor::new(buf)).unwrap();
        let date = exif_date(&exif);
        let orient = exif_orientation(&exif);
        // TODO: test exif deorienting with cases from: https://github.com/recurser/exif-orientation-examples
        // (see also: https://www.daveperrett.com/articles/2012/07/28/exif-orientation-handling-is-a-ghetto)

    // FIXME:    - create 200x200 thumbnail
    // FIXME:       - lanczos resizing
    // FIXME:       - deoriented

        println!("{} {:?} {:?}", f.path().display(), date.map(|d| d.to_string()), orient);
    }

    // FIXME: Stage 2: scan all files once more and refresh them in DB
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
