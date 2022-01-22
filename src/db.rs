use std::path::Path;
use std::sync::{Arc, Mutex};

use anyhow::Result;
use rusqlite::{params, Connection, Error::QueryReturnedNoRows};

use crate::interlude::*;

// TODO[LATER]: use Arc<RwLock<T>> instead of Arc<Mutex<T>>
pub type SyncedDb = Arc<Mutex<Connection>>;

pub fn open(path: impl AsRef<Path>) -> Result<SyncedDb> {
    let db = Connection::open(path.as_ref())?;
    init(&db)?;
    Ok(Arc::new(Mutex::new(db)))
}

pub fn init(db: &Connection) -> rusqlite::Result<()> {
    db.execute_batch(
        "
          CREATE TABLE IF NOT EXISTS file (
            hash TEXT UNIQUE NOT NULL
              CHECK(length(hash) > 0),
            date TEXT,
            thumbnail BLOB
          );
          CREATE INDEX IF NOT EXISTS file_date ON file(date);

          CREATE TABLE IF NOT EXISTS location (
            file_id INTEGER NOT NULL,
            backend_tag STRING NOT NULL,
            path STRING NOT NULL
          );
          CREATE INDEX IF NOT EXISTS
            location_fileID ON location (file_id);
          CREATE UNIQUE INDEX IF NOT EXISTS
            location_perBackend ON location (backend_tag, path);
        ",
    )
}

pub fn exists(db: &Connection, marker: &str, relative: &str) -> ::rusqlite::Result<bool> {
    db.query_row(
        "SELECT COUNT(*) FROM location
            WHERE backend_tag = ?
            AND path = ?",
        params![marker, relative],
        |row| row.get(0),
    )
}

pub fn upsert(
    db: &Connection,
    marker: &str,
    relative: &str,
    info: &crate::model::FileInfo,
) -> Result<()> {
    db.execute(
        "INSERT INTO file(hash,date,thumbnail) VALUES(?,?,?)
            ON CONFLICT(hash) DO UPDATE SET
                date = ifnull(date, excluded.date)",
        params![&info.hash, &info.date, &info.thumb],
    )?;
    db.execute(
        "INSERT INTO location(file_id,backend_tag,path)
            SELECT rowid, ?, ? FROM file
              WHERE hash = ? LIMIT 1;",
        params![&marker, &relative, &info.hash],
    )?;
    Ok(())
}

pub fn remove(db: &Connection, marker: &str, relative: &str) -> Result<()> {
    db.execute(
        "DELETE FROM location
            WHERE backend_tag = ?
            AND path = ?",
        params![&marker, &relative],
    )?;
    Ok(())
}

pub fn hashes(db: SyncedDb, marker: &str) -> impl Iterator<Item = Result<(String, String)>> {
    LooseIterator {
        db,
        marker: marker.to_string(),
        offset: 0,
    }
}

struct LooseIterator {
    // TODO[LATER]: verify if usize is the type matching sqlite expectation
    db: SyncedDb,
    marker: String,
    offset: usize,
}

impl Iterator for LooseIterator {
    type Item = Result<(String, String)>;
    fn next(&mut self) -> Option<Self::Item> {
        // TODO[LATER]: avoid unwrap?
        let db = self.db.lock().unwrap();
        let row = db.query_row(
            "SELECT path, hash FROM location
            LEFT JOIN file
                ON location.file_id = file.rowid
                WHERE backend_tag = ?
                LIMIT 1 OFFSET ?",
            params![&self.marker, &self.offset],
            |row| Ok((row.get(0)?, row.get(1)?)),
        );
        self.offset += 1;
        match row {
            Err(QueryReturnedNoRows) => None,
            Err(err) => Some(Err(anyhow!(err))),
            Ok(row) => Some(Ok(row)),
        }
    }
}
