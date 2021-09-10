use anyhow::Result;
use iced::Sandbox;
use rusqlite::{params, Connection as DbConnection, OptionalExtension};

fn main() -> iced::Result {
    println!("Hello view");


    // FIXME: Milestone: just show single thumbnail from DB, using iced crate

    // FIXME: scrollable window with Gallery widget inside
    // FIXME: Gallery widget showing thumbnails & date separators, based on DB contents

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
        let thumb = self.db.query_row(
            "SELECT thumbnail FROM file LIMIT 1",
            [],
            |row| row.get(0),
        ).optional().unwrap();

        use iced::{Text, Image, image::Handle};
        match thumb {
            None => Text::new("No thumbnails found in DB").into(),
            Some(img) => Image::new(Handle::from_memory(img)).into(),
        }
    }
}
