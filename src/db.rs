use std::path::Path;

use anyhow::Result;
use rusqlite::{params, Connection, Error::QueryReturnedNoRows};

use crate::interlude::*;


mod typed_query;
pub use typed_query::*;


// TODO[LATER]: use Arc<RwLock<T>> instead of Arc<Mutex<T>>
pub type SyncedDb = Arc<Mutex<Connection>>;

pub type SqlValue = rusqlite::types::Value;

/// "...SQLite tables have a 64-bit signed integer key (...) usually called the 'rowid'..."
/// (https://sqlite.org/lang_createtable.html#rowid)
pub type Rowid = i64;

#[derive(Copy, Clone, Debug)]
pub struct OffsetAndLimit {
    pub offset: i64, // FIXME: make sure right type
    pub limit: i64, // FIXME: make sure right type
}

impl OffsetAndLimit {
    pub fn new(offset: i64, limit: i64) -> Self {
        Self { offset, limit }
    }
}

pub fn open(path: impl AsRef<Path>) -> Result<SyncedDb> {
    let db = Connection::open(path.as_ref())?;
    init(&db)?;
    Ok(Arc::new(Mutex::new(db)))
}

pub fn init(db: &Connection) -> rusqlite::Result<()> {
    rusqlite::vtab::array::load_module(&db)?;
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
            backend_tag STRING NOT NULL, -- FIXME[LATER]: change to TEXT (https://stackoverflow.com/a/42264331/98528)
            path STRING NOT NULL         -- FIXME[LATER]: change to TEXT (https://stackoverflow.com/a/42264331/98528)
          );
          CREATE INDEX IF NOT EXISTS
            location_fileID ON location (file_id);
          CREATE UNIQUE INDEX IF NOT EXISTS
            location_perBackend ON location (backend_tag, path);

          CREATE TABLE IF NOT EXISTS tag (
            name TEXT UNIQUE NOT NULL
              CHECK(length(name) > 0),
            hidden BOOLEAN DEFAULT FALSE NOT NULL
          );
          INSERT INTO tag(name, hidden) VALUES
              ('hidden', TRUE)
            ON CONFLICT(name) DO NOTHING;

          CREATE TABLE IF NOT EXISTS file_tag (
            file_id INTEGER NOT NULL,
            tag_id INTEGER NOT NULL
          );
          CREATE UNIQUE INDEX IF NOT EXISTS
            file_tag_unique ON file_tag (file_id, tag_id);
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

// FIXME[LATER]: somehow resolve if same hash at different locations gets attributed a different date
pub fn upsert(
    db: &Connection,
    marker: &str,
    relative: &str,
    info: &crate::model::FileInfo,
) -> Result<()> {
    db.execute(
        "INSERT INTO file(hash,date,thumbnail) VALUES(?,?,?)
            ON CONFLICT(hash) DO UPDATE SET
                date = ifnull(date, excluded.date),
                thumbnail = excluded.thumbnail",
        params![&info.hash, &info.date, &info.thumb],
    )?;
    db.execute(
        "INSERT INTO location(file_id,backend_tag,path)
            SELECT rowid, ?, ? FROM file
              WHERE hash = ? LIMIT 1
            ON CONFLICT(backend_tag, path) DO UPDATE SET
              file_id = excluded.file_id",
        params![&marker, &relative, &info.hash],
    )?;
    Ok(())
}

pub fn tags_for_file_ids<'cnx>(db: &'cnx Connection) ->
    TypedQuery<'cnx, (std::rc::Rc<Vec<SqlValue>>,), (String, bool, u32)>
{
    let sql = r"
SELECT tag.name, tag.hidden, count(ttt)
FROM tag LEFT JOIN (
    SELECT tag_id AS ttt
    FROM file_tag
    WHERE file_id IN rarray(?)
) ON tag.rowid = ttt
GROUP BY tag.rowid";
    TypedQuery::new(db, sql, |row| {
        let name: String = row.get_unwrap(0);
        let hidden: bool = row.get_unwrap(1);
        // FIXME[LATER]: u32 or maybe i64 ?
        let count: u32 = row.get_unwrap(2);
        Ok((name, hidden, count))
    })
}

// pub fn visible_files(db: &Connection, oal: OffsetAndLimit) -> impl Iterator<Item = anyhow::Result<Rowid>> + '_ {
pub fn visible_files_rowids(db: &Connection, oal: OffsetAndLimit) -> Vec<Rowid> {
    let mut query = visible_files_in_limit_and_offset(&db);
    query.run((oal.limit, oal.offset))
        .map(|v| v.unwrap())
        .map(|(rowid, _)| rowid)
        .collect()
}

pub fn visible_files_in_limit_and_offset<'cnx>(db: &'cnx Connection) ->
    TypedQuery<'cnx, (i64, i64), (i64, crate::model::FileInfo)>
{
    let sql = r"
SELECT rowid, hash, date, thumbnail
FROM file
WHERE rowid NOT IN (
  SELECT file_id AS hidden_file
  FROM file_tag
  WHERE tag_id IN (
    SELECT ROWID
    FROM tag
    WHERE hidden IS TRUE
  )
)
ORDER BY date
LIMIT ? OFFSET ?";
    TypedQuery::new(db, sql, |row| {
        let rowid = row.get_unwrap(0);
        let f = crate::model::FileInfo {
            hash: row.get_unwrap(1),
            date: row.get_unwrap(2),
            thumb: row.get_unwrap(3),
        };
        Ok((rowid, f))
    })
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

#[cfg(test)]
mod test {
    use std::rc::Rc;
    use chrono::NaiveDate;

    use crate::{db, db::SqlValue};
    use crate::model::FileInfo;

    fn all_files(conn: &db::Connection) -> Vec<FileInfo> {
        conn.prepare("SELECT hash, date, thumbnail FROM file")
            .unwrap()
            .query_map([], |row| {
                Ok(FileInfo {
                    hash: row.get_unwrap(0),
                    date: row.get_unwrap(1),
                    thumb: row.get_unwrap(2),
                })
            })
            .unwrap()
            .map(|x| x.unwrap())
            .collect()
    }

    #[test]
    fn rusqlite_feat_array() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        db::init(&conn).unwrap();
        let raw_vals = [1i64, 2, 3, 4];
        // Note: a `Rc<Vec<SqlValue>>` must be used as the parameter.
        let wrapped_vals = Rc::new(raw_vals.iter().copied().map(SqlValue::from).collect::<Vec<_>>());
        let mut stmt = conn.prepare("SELECT value FROM rarray(?);").unwrap();
        let rows = stmt.query_map([wrapped_vals], |row| row.get::<_, i64>(0)).unwrap();
        let got = rows.into_iter().map(|v| v.unwrap()).collect::<Vec<i64>>();
        assert_eq!(raw_vals, &got[..]);
    }

    #[test]
    fn upsert_changed_hash_at_location() {
        // arrange

        let marker: &str = "foo-marker";
        let path: &str = "foo-dir/file.jpeg";
        let hash_a = "fake-hash-a".to_string();
        let hash_b = "fake-hash-b".to_string();
        let date_2 = NaiveDate::from_ymd(2022, 1, 22).and_hms(16, 53, 14);

        let conn = rusqlite::Connection::open_in_memory().unwrap();
        db::init(&conn).unwrap();
        db::upsert(
            &conn,
            marker,
            path,
            &FileInfo {
                hash: hash_a.clone(),
                date: None,
                thumb: vec![b'A'],
            },
        )
        .unwrap();
        assert_eq!(db::exists(&conn, marker, path), Ok(true));
        assert_eq!(
            all_files(&conn),
            vec![FileInfo {
                hash: hash_a.clone(),
                date: None,
                thumb: vec![b'A']
            }]
        );

        // act

        db::upsert(
            &conn,
            marker,
            path,
            &FileInfo {
                hash: hash_b.clone(),
                date: Some(date_2),
                thumb: vec![b'B'],
            },
        )
        .unwrap();

        // assert

        assert_eq!(db::exists(&conn, marker, path), Ok(true));
        assert_eq!(
            all_files(&conn),
            vec![
                FileInfo {
                    hash: hash_a,
                    date: None,
                    thumb: vec![b'A']
                },
                FileInfo {
                    hash: hash_b,
                    date: Some(date_2),
                    thumb: vec![b'B']
                },
            ]
        );
    }

    #[test]
    fn upsert_changed_metadata_at_hash() {
        // arrange

        let marker: &str = "foo-marker";
        let path: &str = "foo-dir/file.jpeg";
        let hash = "fake-hash".to_string();
        let date_2 = NaiveDate::from_ymd(2022, 1, 22).and_hms(16, 53, 14);

        let conn = rusqlite::Connection::open_in_memory().unwrap();
        db::init(&conn).unwrap();
        db::upsert(
            &conn,
            marker,
            path,
            &FileInfo {
                hash: hash.clone(),
                date: None,
                thumb: vec![b'A'],
            },
        )
        .unwrap();
        assert_eq!(db::exists(&conn, marker, path), Ok(true));
        assert_eq!(
            all_files(&conn),
            vec![FileInfo {
                hash: hash.clone(),
                date: None,
                thumb: vec![b'A']
            }]
        );

        // act

        db::upsert(
            &conn,
            marker,
            path,
            &FileInfo {
                hash: hash.clone(),
                date: Some(date_2),
                thumb: vec![b'B'],
            },
        )
        .unwrap();

        // assert

        assert_eq!(db::exists(&conn, marker, path), Ok(true));
        assert_eq!(
            all_files(&conn),
            vec![FileInfo {
                hash,
                date: Some(date_2),
                thumb: vec![b'B']
            }]
        );
    }
}
