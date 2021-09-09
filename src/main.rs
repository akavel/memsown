use std::fs::read;
use std::io::Cursor;

use exif::{Exif, Reader};
use globwalk::GlobWalkerBuilder;

fn main() {
    println!("Hello, world!");

    // mdb = openDb("backer.db")
    // let markers = @[
    //   r"d:\backer-id.json",
    //   r"c:\fotki\backer-id.json",
    // ]

    // FIXME: Stage 1: add not-yet-known files into DB
    let images = GlobWalkerBuilder::new(r"c:\fotki", "*.{jpg,jpeg}")
        .case_insensitive(true)
        .file_type(globwalk::FileType::FILE)
        .build();
    for entry in images.unwrap() {
        // TODO[LATER]: use `?` instead of .unwrap() and ret. some err from main() or print error info
        let path = entry.unwrap().path().to_owned();
        let buf = read(&path).unwrap();

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

        println!("{} {:?} {:?}", path.display(), date.map(|d| d.to_string()), orient);
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
