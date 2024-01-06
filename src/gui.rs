use iced::pure::{row, scrollable, Application, Element};
use tracing::{Level, span};

use crate::db::{SyncedDb, SqlValue};
use crate::interlude::*;
use crate::widgets::{
    gallery::{self, Gallery},
    tags::{self, tag},
};

pub struct Gui {
    db: SyncedDb,
    gallery_selection: gallery::Selection,
    tags: tags::Panel,
}

#[derive(Debug, Clone)]
pub enum Message {
    OfTags(tags::Event),
    GallerySelection(gallery::Selection),
}

impl Application for Gui {
    type Message = Message;
    type Flags = SyncedDb;
    type Executor = iced::executor::Default;

    fn new(db: SyncedDb) -> (Gui, iced::Command<Self::Message>) {
        let gui = Gui {
            db: Arc::clone(&db),
            gallery_selection: Default::default(),
            tags: tags::Panel::new(&[
                tag::Tag {
                    name: "hidden".to_string(),
                    selected: None,
                    hidden: true,
                },
                tag::Tag {
                    name: "tag 2".to_string(),
                    selected: Some(true),
                    hidden: false,
                },
            ]),
        };
        (gui, iced::Command::none())
    }

    fn title(&self) -> String {
        String::from("Backer") // TODO[LATER]: description and/or status info and/or version
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        let prof_span = span!(Level::TRACE, "gui::update");
        let _enter = prof_span.enter();

        // TODO: when GallerySelect received, save file IDs, so that their tags can be easily
        // updated in DB when a tag is toggled
        match message {
            Message::OfTags(m) => {
                match m {
                    tags::Event::OfNthTag(ref n, tags::tag::Event::SetHidden(ref hidden)) => {
                        println!("SET HDN [{}] = {}", n, hidden);
//TODO: update DB based on rowid stored in self.tags
                    }
                    _ => {}
                }
                self.tags.update(m);
                self.load_tags_for_selection();
            }
            Message::GallerySelection(selection) => {
                self.gallery_selection = selection;
                self.load_tags_for_selection();
            }
        }
        iced::Command::none()
    }

    fn view(&self) -> Element<Self::Message> {
        let prof_span = span!(Level::TRACE, "gui::view");
        let _enter = prof_span.enter();

        // FIXME: Milestone: detect click
        // FIXME: Milestone: add preview window on click
        // FIXME: Milestone: show some info about where img is present

        let gallery = Gallery::new(Arc::clone(&self.db))
            .with_selection(self.gallery_selection)
            .on_select(Message::GallerySelection);
        let tags = self.tags.view().map(Message::OfTags);
        row()
            .push(
                scrollable(gallery), // // .height(iced::Length::Fill)
                                     // .width(iced::Length::Fill),
            )
            .push(tags)
            .into()
    }
}

impl Gui {
    fn load_tags_for_selection(&mut self) {
        let prof_span = span!(Level::TRACE, "gui::load_tags_for_selection");
        let _enter = prof_span.enter();

        let db = self.db.lock().unwrap();

        let query = crate::db::tags_for_file_ids(&db);
        let file_rowids = self.gallery_selection.rowids.iter().copied().map(SqlValue::from).collect::<Vec<_>>();
        let limit = file_rowids.len() as u32;
        let file_rowids = std::rc::Rc::new(file_rowids);
        self.tags = query.run((file_rowids,))
            .map(|v| v.unwrap())
            .map(|(name, hidden, count)| {
                let selected = if count == 0 {
                    Some(false)
                } else if count == limit {
                    Some(true)
                } else {
                    None
                };
                tag::Tag { name, hidden, selected }
            })
            .collect();

        /*
// FIXME: use a list of rowids as selection, instead of limit+offset
        let sql = r"
SELECT tag.name, tag.hidden, count(ttt)
FROM tag LEFT JOIN (
    SELECT tag_id AS ttt
    FROM file_tag
    WHERE file_id IN (
        SELECT file.rowid
        FROM file
            LEFT JOIN file_tag ON file.rowid = file_tag.file_id
            LEFT JOIN tag ON tag.rowid = file_tag.tag_id
            GROUP BY file.rowid
            HAVING count(hidden)=0
        ORDER BY date
        LIMIT ? OFFSET ?
    )
) ON tag.rowid = ttt
GROUP BY tag.rowid";
        // FIXME: make it work when there are 0 images total in DB
        let selection = self.gallery_selection.range();
        let limit = selection.end() - selection.start() + 1;
        println!("NEW TAGS for: {} .. {}", selection.start(), limit);
        self.tags = db
            .prepare_cached(sql)
            .unwrap()
            .query_map([limit, *selection.start()], |row| {
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
                Ok(tag::Tag {
                    name,
                    hidden,
                    selected,
                })
            })
            .unwrap()
            .map(|x| x.unwrap())
            .collect();
        */
    }
}
