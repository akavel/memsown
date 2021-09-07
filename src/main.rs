use walkdir::WalkDir;

fn main() {
    println!("Hello, world!");

    // mdb = openDb("backer.db")
    // let markers = @[
    //   r"d:\backer-id.json",
    //   r"c:\fotki\backer-id.json",
    // ]

    // FIXME: Stage 1: add not-yet-known files into DB
    // FIXME:  foreach *.{jpg,jpeg} in c:\fotki\...:
    for entry in WalkDir::new(r"c:\fotki") {
        // FIXME: if path not "*.{jpg,jpeg}" then continue
        // TODO[LATER]: use `?` instead of .unwrap() and ret. some err from main() or print error info
        println!("{}", entry.unwrap().path().display());
    // FIXME:    - calc sha1 hash
    // FIXME:    - extract exif date
    // FIXME:    - extract exif orientation
    // FIXME:    - create 200x200 thumbnail
    // FIXME:       - lanczos resizing
    // FIXME:       - deoriented
    }
    // FIXME: Stage 2: scan all files once more and refresh them in DB
}

