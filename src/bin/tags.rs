use iced::{Application, Column};

fn main() -> iced::Result {
    println!("Hello tagpanel");

    ShowTags::run(iced::Settings::default())
}

struct ShowTags {
    // FIXME: move to `State` struct
    // tags: Vec<tag::Tag>,
    t0: tag::Tag,
    t1: tag::Tag,
    t2: tag::Tag,
}

impl iced::Sandbox for ShowTags {
    type Message = ();

    fn new() -> Self {
        Self {
            t0: tag::Tag {
                name: "hidden".to_string(),
                selected: None,
                hidden: true,
                state: tag::State::default(),
            },
            t1: tag::Tag {
                name: "tag 2".to_string(),
                selected: Some(true),
                hidden: false,
                state: tag::State::default(),
            },
            t2: tag::Tag {
                name: "tag 3".to_string(),
                selected: Some(false),
                hidden: false,
                state: tag::State::default(),
            },
        }
    }

    fn title(&self) -> String {
        String::from("Backer tags panel") // FIXME[LATER]
    }

    fn update(&mut self, _message: Self::Message) {
        // FIXME
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        Column::new()
            .spacing(20)
            .push(self.t0.view())
            .push(self.t1.view())
            .push(self.t2.view())
            .into()
    }
}

mod tag {
    use derivative::Derivative;
    use iced::alignment::Alignment;
    use iced::button::{self, Button};
    use iced::{Row, Text};

    type Message = ();

    pub struct Tag {
        pub name: String,
        // TODO: are there three-state checkboxes in iced?
        pub selected: Option<bool>,
        pub hidden: bool,

        pub state: State,
    }

    #[derive(Derivative)]
    #[derivative(Default)]
    pub struct State {
        selected_button: button::State,
        hidden_button: button::State,
    }

    impl Tag {
        pub fn view(&mut self) -> iced::Element<Message> {
            // TODO[LATER]: handle `name` editing
            Row::new()
                .spacing(20)
                .align_items(Alignment::Center)
                .push(
                    Button::new(
                        &mut self.state.selected_button,
                        Text::new(match self.selected {
                            // None => "~",
                            None => "▩", // "⊡"
                            // Some(true) => "✓",
                            Some(true) => "☑", // "✅"
                            // Some(false) => "-",
                            Some(false) => "☐",
                        }),
                    )
                    // TODO: .on_press(TODO)
                    // .style(style::Button::Icon)
                    .padding(10),
                )
                .push(Text::new(&self.name))
                .push(
                    Button::new(
                        &mut self.state.hidden_button,
                        Text::new(match self.hidden {
                            false => "👁",
                            true => "🛇",
                        }),
                    )
                    // TODO: .on_press(TODO)
                    // .style(style::Button::Icon)
                    .padding(10),
                )
                .into()
        }
    }
}
