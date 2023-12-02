use iced::pure::Application;
use tracing_chrome::ChromeLayerBuilder as ChromiumTracingBuilder;
use tracing_subscriber::prelude::*;

use backer::db;
use backer::gui::Gui;

fn main() -> iced::Result {
    println!("Hello view");

    let (chromium_tracing, _guard) = ChromiumTracingBuilder::new().build();
    tracing_subscriber::registry().with(chromium_tracing).init();

    let db = db::open("backer.db").unwrap();

    Gui::run(iced::Settings::with_flags(db))
}
