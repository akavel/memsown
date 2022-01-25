use iced::{alignment, Font, Length, Text};

pub const ICONS: Font = Font::External {
    name: "Icons",
    bytes: include_bytes!("../res/fontello/font/fontello.ttf"),
};

fn icon(unicode: char) -> Text {
    Text::new(&unicode.to_string())
        .font(ICONS)
        .width(Length::Units(20))
        .horizontal_alignment(alignment::Horizontal::Center)
        .size(20)
}

pub fn icon_eye() -> Text {
    icon('\u{E800}')
}

pub fn icon_eye_off() -> Text {
    icon('\u{E801}')
}

pub fn icon_check_empty() -> Text {
    icon('\u{F096}')
}

pub fn icon_minus_squared_alt() -> Text {
    icon('\u{F147}')
}

pub fn icon_ok_squared() -> Text {
    icon('\u{F14A}')
}
