use iced::Application;

use backer::db;
use backer::gui::Gui;


fn main() -> iced::Result {
    println!("Hello view");

    let db = db::open("backer.db").unwrap();

    Gui::run(iced::Settings::with_flags(db))
}
