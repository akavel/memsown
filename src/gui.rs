use std::sync::Arc;

use iced::widget::scrollable as iced_scrollable;
use iced::Row;

use crate::db::SyncedDb;
use crate::interlude::*;
use crate::widgets::{
    gallery::{self, Gallery},
    tags::{self, tag},
};

// FIXME: duplicated between here and src/bin/view.rs !!!
pub struct Gui {
    db: SyncedDb,

    // States of sub-widgets
    scrollable: iced_scrollable::State,
    gallery: gallery::State,
    tags: tags::Panel,
}

#[derive(Debug, Clone)]
pub enum Message {
    TagsMessage(tags::Message),
    GallerySelect,
}

impl iced::Application for Gui {
    type Message = Message;
    type Flags = SyncedDb;
    type Executor = iced::executor::Default;

    fn new(db: SyncedDb) -> (Gui, iced::Command<Self::Message>) {
        let gui = Gui {
            db: Arc::clone(&db),
            scrollable: iced_scrollable::State::new(),
            gallery: gallery::State::new(db),
            tags: tags::Panel::new(&vec![
                tag::Tag {
                    name: "hidden".to_string(),
                    selected: None,
                    hidden: true,
                    state: tag::State::default(),
                },
                tag::Tag {
                    name: "tag 2".to_string(),
                    selected: Some(true),
                    hidden: false,
                    state: tag::State::default(),
                },
            ]),
        };
        (gui, iced::Command::none())
    }

    fn title(&self) -> String {
        String::from("Backer") // TODO[LATER]: description and/or status info and/or version
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        self.update_tags(message);
        iced::Command::none()
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        // FIXME: Milestone: detect click
        // FIXME: Milestone: add preview window on click
        // FIXME: Milestone: show some info about where img is present

        let selection = self.gallery.selection;
        let gallery = Gallery::new(&mut self.gallery).on_select(|| Message::GallerySelect);
        iced::Row::new()
            .push(
                iced_scrollable::Scrollable::new(&mut self.scrollable)
                    .push(gallery)
                    // .height(iced::Length::Fill)
                    .width(iced::Length::Fill),
            )
            .push(view_tags(&mut self.tags, selection, &self.db))
            .into()
    }
}

fn view_tags<'a, 'b>(
    tags: &'a mut tags::Panel,
    selection: (u32, u32),
    db: &'b Arc<Mutex<rusqlite::Connection>>,
) -> impl Into<iced::Element<'a, Message>> {
    let db = db.lock().unwrap();
    let sql = r"
SELECT tag.name, tag.hidden, count(ttt)
FROM tag LEFT JOIN (
    SELECT tag_id AS ttt
    FROM file_tag
    WHERE file_id IN (
        SELECT rowid
        FROM file
        ORDER BY date
        LIMIT ? OFFSET ?
    )
) ON tag.rowid = ttt
GROUP BY tag.rowid";
    // FIXME: make it work when there are 0 images total in DB
    let limit = selection.1 - selection.0 + 1;
    println!("NEW TAGS for: {} .. {}", selection.0, limit);
    *tags = db
        .prepare_cached(sql)
        .unwrap()
        .query_map([limit, selection.0], |row| {
            let name: String = row.get_unwrap(0);
            let hidden: bool = row.get_unwrap(1);
            let count: u32 = row.get_unwrap(2);
            let selected = if count == 0 {
                Some(false)
            } else if count == limit {
                Some(true)
            } else {
                None
            };
            let state = tag::State::default();
            let tag = tag::Tag {
                name,
                hidden,
                selected,
                state,
            };
            Ok(tag)
        })
        .unwrap()
        .map(|x| x.unwrap())
        .collect();
    tags.view().map(move |msg| Message::TagsMessage(msg))
}

impl Gui {
    fn update_tags(&mut self, message: Message) {
        match message {
            Message::TagsMessage(msg) => self.tags.update(msg),
            _ => (),
        };
    }
}
