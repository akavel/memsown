use iced::pure::{button, column, row, text, Element};

pub struct Panel {
    tags: Vec<tag::Tag>,
}

#[derive(Debug, Clone)]
pub enum Event {
    OfNthTag(usize, tag::Event),
}

impl Panel {
    pub fn new(tags: &[tag::Tag]) -> Self {
        Self { tags: tags.into() }
    }

    pub fn get(&self, i: usize) -> &tag::Tag {
        self.tags.get(i).unwrap()
    }

    pub fn update(&mut self, event: Event) {
        match event {
            Event::OfNthTag(i, tag_event) => {
                if let Some(tag) = self.tags.get_mut(i) {
                    tag.update(tag_event)
                }
            }
        }
    }

    pub fn view(&self) -> Element<Event> {
        // TODO: wrap in Scrollable
        let tags: Element<_> = self
            .tags
            .iter()
            .enumerate()
            .fold(column().spacing(20), |col, (i, tag)| {
                col.push(tag.view().map(move |msg| Event::OfNthTag(i, msg)))
            })
            .into();
        tags
    }
}

impl FromIterator<tag::Tag> for Panel {
    fn from_iter<T>(iter: T) -> Self
    where
        T: IntoIterator<Item = tag::Tag>,
    {
        Self {
            tags: iter.into_iter().collect(),
        }
    }
}

pub mod tag {
    use super::*;

    use iced::alignment::Alignment;

    use crate::res;

    #[derive(Debug, Clone)]
    pub enum Event {
        SetSelected(bool),
        SetHidden(bool),
    }

    #[derive(Clone)]
    pub struct Tag {
        pub name: String,
        // TODO: are there three-state checkboxes in iced?
        pub selected: Option<bool>,
        pub hidden: bool,
    }

    impl Tag {
        pub fn new(name: String, selected: Option<bool>, hidden: bool) -> Self {
            Self {
                name,
                selected,
                hidden,
            }
        }

        pub fn update(&mut self, event: Event) {
            match event {
                Event::SetSelected(selected) => {
                    self.selected = Some(selected);
                }
                Event::SetHidden(hidden) => {
                    self.hidden = hidden;
                }
            }
        }

        pub fn view(&self) -> Element<Event> {
            // TODO[LATER]: handle `name` editing
            // TODO[LATER]: make buttons align vertically among others
            let selected_icon = match self.selected {
                None => res::icon_minus_squared_alt(),
                Some(true) => res::icon_ok_squared(),
                Some(false) => res::icon_check_empty(),
            };
            let hidden_icon = match self.hidden {
                false => res::icon_eye(),
                true => res::icon_eye_off(),
            };
            row()
                .spacing(20)
                .align_items(Alignment::Center)
                .push(
                    button(selected_icon)
                        .on_press(Event::SetSelected(!self.selected.unwrap_or(false)))
                        // TODO: .style(style::Button::Icon) - see: iced/examples/todos/
                        .padding(10),
                )
                .push(text(&self.name)) //.width(Length::Fill))
                .push(
                    button(hidden_icon)
                        .on_press(Event::SetHidden(!self.hidden))
                        // TODO: .style(style::Button::Icon) - see: iced/examples/todos/
                        .padding(10),
                )
                .into()
        }
    }
}
