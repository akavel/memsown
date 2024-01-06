//! Module containing a generic iterator returning typed results from cached SQLite queries.

use rusqlite::{Connection, MappedRows, Params, Result, Row};
use std::marker::PhantomData;

struct TypedQuery<'conn, P, T> {
    stmt: rusqlite::CachedStatement<'conn>,
    params_type: PhantomData<P>,
    row_mapper: fn(&Row) -> Result<T>,
}

impl<'conn, T, P> TypedQuery<'conn, P, T>
{
    fn new(conn: &'conn Connection, sql: &str, f: fn(&Row)->Result<T>) -> TypedQuery<'conn, P, T> {
        // FIXME[LATER]: change unwrap() to expect() or smth
        // FIXME[LATER]: pass unwrap to 1st next()
        let stmt = conn.prepare_cached(sql).unwrap();
        Self {
            stmt,
            params_type: PhantomData,
            row_mapper: f,
        }
    }
}

impl<'conn, T, P> TypedQuery<'conn, P, T>
where
    P: Params,
{
    // TODO[LATER]: can we ensure Self cannot be ever used after?
    // TODO: fn ... -> impl Iterator<Item = Result<T>> {
    fn run(&mut self, params: P) -> MappedRows<'_, fn(&Row)->Result<T>> {
        // FIXME[LATER]: change unwrap() to expect() or smth
        // FIXME[LATER]: pass unwrap to 1st next()
        self.stmt
            .query_map(params, self.row_mapper)
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use rusqlite::params;

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

    // fn simple_query(conn: &Connection) -> impl Iterable<Item = Result<(String, i64)>> {
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
