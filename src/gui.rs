use std::sync::Arc;

use iced::widget::scrollable as iced_scrollable;

use crate::db::SyncedDb;
use crate::widgets::gallery::Gallery;

// FIXME: duplicated between here and src/bin/view.rs !!!
pub struct Gui {
    db: SyncedDb,

    // States of sub-widgets
    scrollable: iced_scrollable::State,
}

impl iced::Application for Gui {
    type Message = ();
    type Flags = SyncedDb;
    type Executor = iced::executor::Default;

    fn new(flags: SyncedDb) -> (Gui, iced::Command<Self::Message>) {
        (
            Gui {
                db: flags,
                scrollable: iced_scrollable::State::new(),
            },
            iced::Command::none(),
        )
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

        iced_scrollable::Scrollable::new(&mut self.scrollable)
            .push(Gallery::new(Arc::clone(&self.db)))
            .into()
    }
}
