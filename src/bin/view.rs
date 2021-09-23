use std::sync::{Arc, Mutex};

use anyhow::Result;
use iced::Sandbox;
use rusqlite::Connection as DbConnection;

use backer::widgets::gallery::Gallery;


fn main() -> iced::Result {
    println!("Hello view");

    // TODO[LATER]: see if IPFS can be reused from: https://github.com/FuzzrNet/Fuzzr

    Gallery::run(iced::Settings::default())
}

struct Gallery {
    db: Arc<Mutex<DbConnection>>,
}

impl iced::Sandbox for Gallery {
    type Message = ();

    fn new() -> Gallery {
        Gallery{
            db: Arc::new(Mutex::new(DbConnection::open("backer.db").unwrap())),
        }
    }

    fn title(&self) -> String {
        String::from("Backer") // TODO[LATER]: description and/or status info and/or version
    }

    fn update(&mut self, _message: Self::Message) {
        // FIXME
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        // FIXME: Milestone: wrap in scrollable
        // FIXME: Milestone: add date headers
        // FIXME: Milestone: detect click
        // FIXME: Milestone: add preview window on click

        // FIXME: wrap in scrollable
        Gallery::new(Arc::clone(&self.db)).into()
    }
}
