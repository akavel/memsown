use iced::Column;

pub struct Panel {
    tags: Vec<tag::Tag>,
}

#[derive(Debug, Clone)]
pub enum Message {
    TagMessage(usize, tag::Message),
}

impl Panel {
    pub fn new(tags: &[tag::Tag]) -> Self {
        Self { tags: tags.into() }
    }

    pub fn update(&mut self, message: Message) {
        match message {
            Message::TagMessage(i, tag_message) => {
                if let Some(tag) = self.tags.get_mut(i) {
                    tag.update(tag_message);
                }
            }
        }
    }

    pub fn view(&mut self) -> iced::Element<Message> {
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

pub mod tag {
    use derivative::Derivative;
    use iced::alignment::Alignment;
    use iced::button::{self, Button};
    use iced::{Length, Row, Text};

    use crate::res;

    #[derive(Debug, Clone)]
    pub enum Message {
        SetSelected(bool),
        SetHidden(bool),
    }

    #[derive(Clone)]
    pub struct Tag {
        pub name: String,
        // TODO: are there three-state checkboxes in iced?
        pub selected: Option<bool>,
        pub hidden: bool,

        pub state: State,
    }

    #[derive(Clone, Derivative)]
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
