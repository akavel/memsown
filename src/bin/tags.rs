use iced::{Application, Column};

fn main() -> iced::Result {
    println!("Hello tagpanel");

    ShowTags::run(iced::Settings::default())
}

struct ShowTags {
    // FIXME: move to `State` struct
    tags: Vec<tag::Tag>,
}

#[derive(Debug, Clone)]
enum Message {
    TagMessage(usize, tag::Message),
}

impl iced::Sandbox for ShowTags {
    type Message = Message;

    fn new() -> Self {
        Self {
            tags: vec![
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
            ],
        }
    }

    fn title(&self) -> String {
        String::from("Backer tags panel") // FIXME[LATER]
    }

    fn update(&mut self, message: Self::Message) {
        match message {
            Message::TagMessage(i, tag_message) => {
                if let Some(tag) = self.tags.get_mut(i) {
                    tag.update(tag_message);
                }
            }
        }
    }

    fn view(&mut self) -> iced::Element<Self::Message> {
        let tags: iced::Element<_> = self
            .tags
            .iter_mut()
            .enumerate()
            .fold(Column::new().spacing(20), |col, (i, tag)| {
                col.push(tag.view().map(move |msg| Message::TagMessage(i, msg)))
            })
            .into();
        // TODO: wrap in Scrollable
        tags
    }
}

mod tag {
    use derivative::Derivative;
    use iced::alignment::Alignment;
    use iced::button::{self, Button};
    use iced::{Length, Row, Text};

    use backer::res;

    #[derive(Debug, Clone)]
    pub enum Message {
        SetSelected(bool),
        SetHidden(bool),
    }

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
        pub fn update(&mut self, message: Message) {
            match message {
                Message::SetSelected(selected) => {
                    self.selected = Some(selected);
                }
                Message::SetHidden(hidden) => {
                    self.hidden = hidden;
                }
            }
        }

        pub fn view(&mut self) -> iced::Element<Message> {
            // TODO[LATER]: handle `name` editing
            Row::new()
                .spacing(20)
                .align_items(Alignment::Center)
                .push(
                    Button::new(
                        &mut self.state.selected_button,
                        match self.selected {
                            None => res::icon_minus_squared_alt(),
                            Some(true) => res::icon_ok_squared(),
                            Some(false) => res::icon_check_empty(),
                        },
                    )
                    .on_press(Message::SetSelected(match self.selected {
                        None | Some(false) => true,
                        Some(true) => false,
                    }))
                    // TODO: .style(style::Button::Icon) - see: iced/examples/todos/
                    .padding(10),
                )
                .push(Text::new(&self.name).width(Length::Fill))
                .push(
                    Button::new(
                        &mut self.state.hidden_button,
                        match self.hidden {
                            false => res::icon_eye(),
                            true => res::icon_eye_off(),
                        },
                    )
                    .on_press(Message::SetHidden(!self.hidden))
                    // TODO: .style(style::Button::Icon) - see: iced/examples/todos/
                    .padding(10),
                )
                .into()
        }
    }
}
