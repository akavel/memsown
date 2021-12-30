use anyhow::Result;
use rusqlite::{params, Connection};

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
