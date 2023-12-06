use chrono::naive::{NaiveDate, NaiveDateTime};
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rusqlite::params;
use backer::{model, db};

pub fn bench_gallery_files(c: &mut Criterion) {
    // c.bench_function("1) with tags", |b| b.iter(|| 
    c.bench_function("troubling_select_with_tags", |b| {
        b.iter_batched(|| setup_db_with_tags(), |mut conn| {
            let mut query = conn
                .prepare_cached(
                    r"
    SELECT hash, date, thumbnail
    FROM file
    LEFT JOIN file_tag ON file.rowid = file_tag.file_id
    LEFT JOIN tag ON tag.rowid = file_tag.tag_id
    GROUP BY file.rowid
    HAVING count(hidden)=0
    ORDER BY date
    LIMIT ? OFFSET ?",
                )
                .unwrap();
            let limit = 100;
            let offset = 0;
            let file_iter = query
                .query_map(params!(limit, offset), |row| {
                    Ok(crate::model::FileInfo {
                        hash: row.get_unwrap(0),
                        date: row.get_unwrap(1),
                        thumb: row.get_unwrap(2),
                    })
                })
                .unwrap();
            for (i, row) in file_iter.enumerate() {
                let file = row.unwrap();
                black_box(file);
            }
            // query
            drop(query);
            conn
        }, criterion::BatchSize::LargeInput);
    });
}

criterion_group!(benches, bench_gallery_files);
criterion_main!(benches);

fn setup_db_with_tags() -> rusqlite::Connection {
    // FIXME: for benchmarks, should we use on-disk database for consistency with real-life results?
    let conn = db::open_in_memory();

    // insert some sample data
    // TODO: write helper macro to reduce repetition & improve readability

    // sample tags
    let sql = "INSERT INTO tag(name, hidden) VALUES(?,?)";
    conn.execute(sql, params!["foo", true]).unwrap();
    let id_tag_foo = conn.last_insert_rowid();
    conn.execute(sql, params!["bar", false]).unwrap();
    let id_tag_bar = conn.last_insert_rowid();

    // sample files
    let sql = "INSERT INTO file(hash, date) VALUES(?,?)";
    let date: Option<NaiveDateTime> = None;
    conn.execute(sql, params!["hash1", date]).unwrap();
    let id_file1 = conn.last_insert_rowid();
    let date: Option<NaiveDateTime> = Some(NaiveDate::from_ymd(
            2023,12,05).and_hms(8,53,12));
    conn.execute(sql, params!["hash2", date]).unwrap();
    let id_file2 = conn.last_insert_rowid();
    let date: Option<NaiveDateTime> = Some(NaiveDate::from_ymd(
            2022,01,02).and_hms(16,00,01));
    conn.execute(sql, params!["hash3", date]).unwrap();
    let id_file3 = conn.last_insert_rowid();

    // FIXME: connect tags with files - table file_tag
    let sql = "INSERT INTO file_tag(file_id, tag_id) VALUES(?,?)";
    conn.execute(sql, params![id_file1, id_tag_foo]).unwrap();
    conn.execute(sql, params![id_file1, id_tag_bar]).unwrap();
    conn.execute(sql, params![id_file2, id_tag_foo]).unwrap();
    conn.execute(sql, params![id_file3, id_tag_bar]).unwrap();

    conn
}
