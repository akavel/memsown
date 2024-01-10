use iced::{alignment, Font, Length};
use iced::widget::{text, Text};

// FIXME: font loading
/*
pub const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../res/fontello/font/fontello.ttf"),
};
*/

// fn icon(unicode: char) -> Text<'static> {
fn icon(unicode: &'static str) -> Text<'static> {
    Text::new(std::borrow::Cow::from(unicode))
    // Text::new(unicode.into())
    // Text::new(&unicode.to_string())
        // .font(ICONS)
        .width(Length::Fixed(20.0))
        .horizontal_alignment(alignment::Horizontal::Center)
        .size(20)
        .shaping(text::Shaping::Advanced)
}

pub fn icon_eye() -> Text<'static> {
    // icon('\u{E800}')
    // icon('o')
    icon("👁")
}

pub fn icon_eye_off() -> Text<'static> {
    // icon('\u{E801}')
    icon("-")
}

pub fn icon_check_empty() -> Text<'static> {
    // icon('\u{F096}')
    icon("☐")
}

pub fn icon_minus_squared_alt() -> Text<'static> {
    // icon('\u{F147}')
    icon("⊟")
}

pub fn icon_ok_squared() -> Text<'static> {
    // icon('\u{F14A}')
    icon("☑")
}
