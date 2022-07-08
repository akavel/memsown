use std::sync::Arc;

use iced::widget::scrollable as iced_scrollable;
use iced::Row;

use crate::db::SyncedDb;
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
        match message {
            Message::TagsMessage(msg) => self.tags.update(msg),
        };

        iced::Command::none()
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        // FIXME: Milestone: detect click
        // FIXME: Milestone: add preview window on click
        // FIXME: Milestone: show some info about where img is present

        iced::Row::new()
            .push(
                iced_scrollable::Scrollable::new(&mut self.scrollable)
                    .push(Gallery::new(&mut self.gallery))
                    // .height(iced::Length::Fill)
                    .width(iced::Length::Fill),
            )
            .push(self.tags.view().map(move |msg| Message::TagsMessage(msg)))
            .into()
    }
}
