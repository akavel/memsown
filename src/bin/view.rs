use std::sync::{Arc, Mutex};

use iced::Application;
use rusqlite::Connection as DbConnection;

use backer::gui::Gui;


fn main() -> iced::Result {
    println!("Hello view");

    let db = Arc::new(Mutex::new(DbConnection::open("backer.db").unwrap()));

    Gui::run(iced::Settings::with_flags(db))
}
