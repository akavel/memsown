//! Module containing a generic iterator returning typed results from cached SQLite queries.

use rusqlite::{params, Connection};

struct Typed {
}

impl Iterator for Typed {
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn simple_use() {
        let conn = new_db();
        let iter = simple_iter(&conn);
        let maybe_all = iter.collect::<Result<Vec<_>>>();
        let all = maybe_all.unwrap();
        assert_eq!(all, &[
            ("hello", 1),
            ("world", 2),
        ]);
    }

    fn simple_iter(conn: &Connection) -> impl Iterator<Result<(String, i64)>> {
        Typed::new(
            "SELECT foo, bar FROM foobar
                WHERE foo != ?
                ORDER BY bar
                LIMIT ?",
            params!["bleh-dummy", 100],
            |row| {
                let foo: String = row.get(0)?;
                let bar: i64 = row.get(1)?;
                Ok((foo, bar))
            },
        )
    }

    fn new_db() -> rusqlite::Connection {
        let db = rusqlite::Connection::open_in_memory().unwrap();
        db.execute_batch(
            "CREATE TABLE IF NOT EXISTS foobar(foo TEXT, bar INTEGER);
            INSERT INTO foobar(foo, bar) VALUES('hello', 1);
            INSERT INTO foobar(foo, bar) VALUES('world', 2);"
        ).unwrap();
        db
    }
}

