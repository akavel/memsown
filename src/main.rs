use std::thread;

use anyhow::Result;
use iced::Application;

use backer::config;
use backer::db;
use backer::gui::Gui;
use backer::interlude::*;
use backer::scanning::*;

// TODO: migrate to iced v0.4.0 with its new features & architecture
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

    let db = db::open("backer.db")?;

    // Read and parse config.
    let config = config::read("backer.toml")?;

    let scanner = {
        // TODO[LATER]: consider not cloning config maybe (?)
        // TODO[LATER]: somehow pass args prettier to the thread
        let (db, config) = (db.clone(), config);
        thread::spawn(move || scan(db, config).unwrap())
    };

    // TODO[LATER]: see if IPFS can be reused from: https://github.com/FuzzrNet/Fuzzr

    Gui::run(iced::Settings::with_flags(db))?;

    // TODO: somehow be checking status of the thread before GUI finishes; and/or run the thread in loop?
    scanner
        .join()
        .map_err(|err| anyhow!(ifmt!("error scanning: " err;?)))?;

    Ok(())
}
