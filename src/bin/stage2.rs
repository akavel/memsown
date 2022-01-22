use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

use anyhow::Result;
use path_slash::PathBufExt;

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
        let (relative_path, db_hash) = item?;

        let path = tree.root.join(PathBuf::from_slash(relative_path));

        // Read file contents to memory.
        let buf = fs::read(&path)?;
        let disk_hash = hash(&buf);

        if disk_hash == db_hash {
            print!(".");
            io::stdout().flush()?;
        } else {
            iprintln!("\nBAD HASH: " disk_hash " != " db_hash " @ " path;?);
            // iprintln!("* " db_hash " @ " path;?);
        }
    }

    Ok(())
}
