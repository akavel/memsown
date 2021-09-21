use anyhow::Result;
use iced::Sandbox;
use rusqlite::{params, Connection as DbConnection, OptionalExtension};

fn main() -> iced::Result {
    println!("Hello view");
    let _ = backer::widgets::gallery::Gallery::new();

    // TODO[LATER]: see if IPFS can be reused from: https://github.com/FuzzrNet/Fuzzr

    Gallery::run(iced::Settings::default())
}

struct Gallery {
    db: DbConnection,
}

impl iced::Sandbox for Gallery {
    type Message = ();

    fn new() -> Gallery {
        Gallery{
            db: DbConnection::open("backer.db").unwrap(),
        }
    }

    fn title(&self) -> String {
        String::from("Backer") // TODO[LATER]: description and/or status info and/or version
    }

    fn update(&mut self, _message: Self::Message) {
        // FIXME
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        // FIXME: Milestone: show n=15 images in grid
        // FIXME: Milestone: wrap in scrollable
        // FIXME: Milestone: add date headers
        // FIXME: Milestone: detect click
        // FIXME: Milestone: add preview window on click
        // let thumb = self.db.query_row(
        //     "SELECT thumbnail FROM file LIMIT 1",
        //     [],
        //     |row| row.get(0),
        // ).optional().unwrap();

        // backer::widgets::gallery::Gallery::new()
        backer::widgets::gallery::Gallery::new().into()
        // use iced::{Text, Image, image::Handle};
        // match thumb {
        //     None => Text::new("No thumbnails found in DB").into(),
        //     Some(img) => Image::new(Handle::from_memory(img)).into(),
        // }
    }
}
