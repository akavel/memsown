use iced::Application;

use backer::widgets::tags::{self, tag};

fn main() -> iced::Result {
    println!("Hello tagpanel");

    ShowTags::run(iced::Settings::default())
}

struct ShowTags {
    panel: tags::Panel,
}

impl iced::Sandbox for ShowTags {
    type Message = tags::Message;

    fn new() -> Self {
        let panel = tags::Panel::new(&vec![
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
            tag::Tag {
                name: "tag 3".to_string(),
                selected: Some(false),
                hidden: false,
                state: tag::State::default(),
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

    fn view(&mut self) -> iced::Element<Self::Message> {
        self.panel.view()
    }
}
