use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::Result;
use rusqlite::Connection as DbConnection;

use backer::interlude::*;

use backer::config;
use backer::db;
use backer::scanning::*;


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

fn main() {
    if let Err(err) = run() {
        ieprintln!("Error: " error_chain(&err) ".");
    }
}

fn run() -> Result<()> {
    // TODO[LATER]: run rustfmt on this repo
    // TODO[LATER]: run clippy on this repo
    println!("Hello, world!");

    let db = DbConnection::open("backer.db")?;
    db::init(&db)?;
    let db = Arc::new(Mutex::new(db));

    // Read and parse config.
    let config = config::read("backer.toml")?;

    // TODO[LATER]: consider using 'rayon' lib for prettier parallelism
    let mut threads = vec![];
    for (i, marker) in config.markers.disk.iter().enumerate() {
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

