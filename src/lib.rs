//! Module containing a generic iterator returning typed results from cached SQLite queries.

use rusqlite::{Connection, MappedRows, Params, Result, Row};

// TODO[LATER]: some other way or trait more canonical?
trait Iterable {
    type Item;
    type Iter: Iterator<Item = Self::Item>;

    fn iter(&mut self) -> Self::Iter;
}

// struct Typed<'stmt, F> {
struct TypedQuery<'conn, P, F> {
    stmt: rusqlite::CachedStatement<'conn>,
    params: Option<P>,
    row_mapper: Option<F>,
    // rows: rusqlite::MappedRows<'conn, F>,
}

impl<'conn, T, P, F> TypedQuery<'conn, P, F>
where
    // F: FnMut(&Row<'_>) -> Result<T>
    // F: FnMut(&Row<'stmt>) -> Result<T>
    P: Params,
    F: FnMut(&Row) -> Result<T>,
{
    fn new(conn: &'conn Connection, sql: &str, params: P, f: F) -> TypedQuery<'conn, P, F> {
        // FIXME[LATER]: change unwrap() to expect() or smth
        // FIXME[LATER]: pass unwrap to 1st next()
        let stmt = conn.prepare_cached(sql).unwrap();
        Self {
            stmt,
            params: Some(params),
            row_mapper: Some(f),
        }
    }

    // TODO[LATER]: can we ensure Self cannot be ever used after?
    // fn iter(&mut self) -> impl Iterator<Item = Result<T>> {
    fn iter(&mut self) -> MappedRows<'_, F> {
        // FIXME[LATER]: change unwrap() to expect() or smth
        // FIXME[LATER]: pass unwrap to 1st next()
        self.stmt
            .query_map(self.params.take().unwrap(), self.row_mapper.take().unwrap())
            .unwrap()
    }
}

/*
impl<'conn, P, F> Iterable for TypedQuery
{
}
*/

#[cfg(test)]
mod test {
    use super::*;
    use rusqlite::params;

    #[test]
    fn simple_use() {
        let conn = new_db();
        let mut query = simple_query(&conn);
        let maybe_all = query.iter().collect::<Result<Vec<_>>>();
        let all = maybe_all.unwrap();
        assert_eq!(
            all,
            &[("hello".to_string(), 1i64), ("world".to_string(), 2i64),]
        );
    }

    // fn simple_query(conn: &Connection) -> impl Iterable<Item = Result<(String, i64)>> {
    fn simple_query<'conn>(
        conn: &'conn Connection,
    ) -> TypedQuery<'conn, impl Params, impl FnMut(&Row<'_>) -> Result<(String, i64)>> {
        TypedQuery::new(
            conn,
            "SELECT foo, bar FROM foobar
                WHERE foo != ?
                ORDER BY bar
                LIMIT ?",
            params!["bleh-dummy", 100],
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
