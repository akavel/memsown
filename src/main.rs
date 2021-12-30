use std::sync::{Arc, Mutex};
use std::thread;

use anyhow::Result;
use iced::Application;
use rusqlite::Connection as DbConnection;

use backer::interlude::*;
use rayon::prelude::*;

use backer::config::{self, Config};
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

    let scanner = {
        // TODO[LATER]: consider not cloning config maybe (?)
        // TODO[LATER]: somehow pass args prettier to the thread
        let (db, config) = (db.clone(), config.clone());
        thread::spawn(move || scan(db, config).unwrap())
    };

    Gallery::run(iced::Settings::with_flags(db))?;

    // TODO: somehow be checking status of the thread before GUI finishes; and/or run the thread in loop?
    scanner.join().map_err(|err| anyhow!(ifmt!("error scanning: " err;?)))?;

    Ok(())
}

// TODO[LATER]: use Arc<RwLock<T>> instead of Arc<Mutex<T>>
type SyncedDb = Arc<Mutex<DbConnection>>;

fn scan(db: SyncedDb, config: Config) -> Result<()> {
    for err in config.markers.disk
        .into_par_iter()
        .enumerate()
        .filter_map(|(i, marker)| process_tree(i, marker, db.clone()).err())
        .collect::<Vec<_>>()
    {
        ieprintln!("Error: " err);
    }

    // FIXME: Stage 2: check if all files from DB are present on disk, delete entries for any missing

    // FIXME: Stage 3: scan all files once more and refresh them in DB

    Ok(())
}

// FIXME: duplicated between here and src/bin/view.rs !!!
struct Gallery {
    db: SyncedDb,

    // States of sub-widgets
    scrollable: iced::widget::scrollable::State,
}

impl iced::Application for Gallery {
    type Message = ();
    type Flags = SyncedDb;
    type Executor = iced::executor::Default;

    fn new(flags: SyncedDb) -> (Gallery, iced::Command<Self::Message>) {
        (Gallery {
            db: flags,
            scrollable: iced::widget::scrollable::State::new(),
        }, iced::Command::none())
    }

    fn title(&self) -> String {
        String::from("Backer") // TODO[LATER]: description and/or status info and/or version
    }

    fn update(&mut self, _message: Self::Message) -> iced::Command<Self::Message> {
        // FIXME
        iced::Command::none()
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        // FIXME: Milestone: detect click
        // FIXME: Milestone: add preview window on click
        // FIXME: Milestone: show some info about where img is present

        iced::widget::scrollable::Scrollable::new(&mut self.scrollable)
            .push(backer::widgets::gallery::Gallery::new(Arc::clone(&self.db)))
            .into()
    }
}
