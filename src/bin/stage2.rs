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

    stage2(&tree, &db)?;

    Ok(())
}
