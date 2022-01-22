use anyhow::Result;

use backer::config;
use backer::db;
use backer::interlude::*;
use backer::scanning::*;

fn main() {
    if let Err(err) = run() {
        ieprintln!("error: " error_chain(&err));
    }
}

fn run() -> Result<()> {
    let db = db::open("backer.db")?;
    // let config = config::read("backer.toml")?;

    let marker_path = r"c:\fotki\backer-id.json";
    let tree = Tree::open(marker_path, &config::DatePathsPerMarker::default())?;

    for item in db::hashes(db, &tree.marker) {
        let (relative_path, hash) = item?;
        iprintln!("* " hash " @ " relative_path);
    }

    Ok(())
}
