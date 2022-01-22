use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use path_slash::PathBufExt;
use rusqlite::Connection as DbConnection;
use tempfile::tempdir;

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

fn stage2(tree: &Tree, db: &Arc<Mutex<DbConnection>>) -> Result<()> {
    for item in db::hashes(db.clone(), &tree.marker) {
        let (relative_path, db_hash) = item?;

        let path = tree.root.join(PathBuf::from_slash(&relative_path));

        // Try reading file contents to memory.
        let contents = match fs::read(&path) {
            Ok(data) => Some(data),
            Err(err) if err.kind() == io::ErrorKind::NotFound => None,
            Err(err) => return Err(anyhow!(err)),
        };
        if let Some(data) = contents {
            let disk_hash = hash(&data);

            if disk_hash == db_hash {
                print!(",");
                io::stdout().flush()?;
            } else {
                iprintln!("\nBAD HASH: " disk_hash " != " db_hash " @ " path;?);
                // iprintln!("* " db_hash " @ " path;?);
            }
        } else {
            let db = db.lock().unwrap();
            // TODO[LATER]: add error context info
            db::remove(&db, &tree.marker, &relative_path)?;
        }
    }

    Ok(())
}

#[test]
fn stage2_file_not_found() {
    // arrange

    let marker: &str = "foo-marker";
    let relative_path: &str = "foo-dir/foo-file.jpeg";
    let root = tempdir().unwrap();
    let mut marker_file = fs::File::create(root.path().join("marker.json")).unwrap();
    marker_file
        .write_all(r#"{"id": "foo-marker"}"#.as_bytes())
        .unwrap();
    // writeln!(marker_file, r#"{"id": "foo-marker"}"#).unwrap();
    drop(marker_file);
    let tree = Tree::open(
        root.path().join("marker.json"),
        &config::DatePathsPerMarker::new(),
    )
    .unwrap();

    let conn = rusqlite::Connection::open_in_memory().unwrap();
    let db = Arc::new(Mutex::new(conn));
    let conn = db.lock().unwrap();
    db::init(&conn).unwrap();
    db::upsert(
        &conn,
        &marker,
        &relative_path,
        &backer::model::FileInfo {
            hash: hash(&Vec::new()),
            date: None,
            thumb: Vec::new(),
        },
    )
    .unwrap();
    assert_eq!(db::exists(&conn, &marker, &relative_path), Ok(true));
    drop(conn);

    // act

    let res = stage2(&tree, &db);

    // assert

    assert!(res.is_ok(), "stage2 == {:?}", &res);

    let conn = db.lock().unwrap();
    assert_eq!(db::exists(&conn, &marker, &relative_path), Ok(false));
    drop(conn);
}
