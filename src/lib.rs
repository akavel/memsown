//! Module containing a generic iterator returning typed results from cached SQLite queries.

use rusqlite::{Params, Connection, Row, Result};

struct Typed<F> {
    f: F,
}

impl<T, F> Typed<F> 
where 
    F: FnMut(&Row) -> Result<T>
    // F: FnMut(&Row<'_>) -> Result<T>
    // F: FnMut(&Row<'stmt>) -> Result<T>
{
    fn new<P>(conn: &Connection, sql: &str, params: P, f: F) -> Self 
    where 
        P: Params,
    {
        Self { f }
    }
}

impl<T, F> Iterator for Typed<F>
where F: FnMut(&Row) -> Result<T> {
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        // FIXME
        None
    }
}


#[cfg(test)]
mod test {
    use super::*;
    use rusqlite::params;

    #[test]
    fn simple_use() {
        let conn = new_db();
        let iter = simple_iter(&conn);
        let maybe_all = iter.collect::<Result<Vec<_>>>();
        let all = maybe_all.unwrap();
        assert_eq!(all, &[
            ("hello".to_string(), 1i64),
            ("world".to_string(), 2i64),
        ]);
    }

    fn simple_iter(conn: &Connection) -> impl Iterator<Item = Result<(String, i64)>> {
        Typed::new(
            conn,
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

