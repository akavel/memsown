use std::sync::{Arc, Mutex};

use anyhow::Result;
use iced::Sandbox;
use rusqlite::Connection as DbConnection;

fn main() -> iced::Result {
    println!("Hello view");

    // TODO[LATER]: see if IPFS can be reused from: https://github.com/FuzzrNet/Fuzzr

    Gallery::run(iced::Settings::default())
}

struct Gallery {
    // TODO[LATER]: use Arc<RwLock<T>> instead of Arc<Mutex<T>>
    db: Arc<Mutex<DbConnection>>,

    // States of sub-widgets
    scrollable: iced::widget::scrollable::State,
}

impl iced::Sandbox for Gallery {
    type Message = ();

    fn new() -> Gallery {
        Gallery {
            db: Arc::new(Mutex::new(DbConnection::open("backer.db").unwrap())),

            scrollable: iced::widget::scrollable::State::new(),
        }
    }

    fn title(&self) -> String {
        String::from("Backer") // TODO[LATER]: description and/or status info and/or version
    }

    fn update(&mut self, _message: Self::Message) {
        // FIXME
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
