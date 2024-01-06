//! Module containing a generic iterator returning typed results from cached SQLite queries.

use rusqlite::{Connection, MappedRows, Result};
use std::marker::PhantomData;

pub type RowParser<T> = fn(&rusqlite::Row) -> Result<T>;

pub struct TypedQuery<'conn, Params, Row> {
    stmt: rusqlite::CachedStatement<'conn>,
    params_type: PhantomData<Params>,
    row_parser: RowParser<Row>,
}

impl<'conn, Params, Row> TypedQuery<'conn, Params, Row> {
    pub fn new(
        conn: &'conn Connection,
        sql: &str,
        row_parser: RowParser<Row>,
    ) -> TypedQuery<'conn, Params, Row> {
        // FIXME[LATER]: change unwrap() to expect() or smth
        // FIXME[LATER]: pass unwrap to 1st next()
        let stmt = conn.prepare_cached(sql).unwrap();
        Self {
            stmt,
            params_type: PhantomData,
            row_parser,
        }
    }
}

impl<'conn, Params, Row> TypedQuery<'conn, Params, Row>
where
    Params: rusqlite::Params,
{
    // TODO: fn ... -> impl Iterator<Item = Result<Row>> {
    pub fn run(&mut self, params: Params) -> MappedRows<'_, RowParser<Row>> {
        // FIXME[LATER]: change unwrap() to expect() or smth
        // FIXME[LATER]: pass unwrap to 1st next()
        self.stmt.query_map(params, self.row_parser).unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rusqlite::Row;

    #[test]
    fn simple_use() {
        let conn = new_db();
        let mut query = simple_query(&conn);
        let maybe_all = query.run(("bleh-dummy", 100)).collect::<Result<Vec<_>>>();
        let all = maybe_all.unwrap();
        assert_eq!(
            all,
            &[("hello".to_string(), 1i64), ("world".to_string(), 2i64),]
        );
    }

    fn simple_query<'conn>(
        conn: &'conn Connection,
    ) -> TypedQuery<'conn, (&str, i64), (String, i64)> {
        TypedQuery::new(
            conn,
            "SELECT foo, bar FROM foobar
                WHERE foo != ?
                ORDER BY bar
                LIMIT ?",
            |row: &Row| {
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
            INSERT INTO foobar(foo, bar) VALUES('world', 2);",
        )
        .unwrap();
        db
    }
}
