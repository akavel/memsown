use anyhow::Result;

use backer::config;
use backer::db;
use backer::interlude::*;
use backer::scanning::scan;


fn main() {
    if let Err(err) = run() {
        ieprintln!("error: " error_chain(&err));
    }
}

fn run() -> Result<()> {
    let db = db::open("backer.db")?;
    let config = config::read("backer.toml")?;
    scan(db, config)
}
