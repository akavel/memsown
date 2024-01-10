use iced::{Element, Sandbox};

use backer::widgets::tags::{self, tag};

fn main() -> iced::Result {
    println!("Hello tagpanel");

    ShowTags::run(iced::Settings::default())
}

struct ShowTags {
    panel: tags::Panel,
}

impl Sandbox for ShowTags {
    type Message = tags::Event;

    fn new() -> Self {
        let panel = tags::Panel::new(&[
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
            tag::Tag {
                name: "tag 3".to_string(),
                selected: Some(false),
                hidden: false,
            },
        ]);
        Self { panel }
    }

    fn title(&self) -> String {
        String::from("Backer tags panel") // FIXME[LATER]
    }

    fn update(&mut self, message: Self::Message) {
        self.panel.update(message);
    }

    fn view(&self) -> Element<Self::Message> {
        self.panel.view()
    }
}
