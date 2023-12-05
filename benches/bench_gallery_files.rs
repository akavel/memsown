use chrono::naive::{NaiveDate, NaiveDateTime};
use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn bench_gallery_files(c: &mut Criterion) {
    // c.bench_function("1) with tags", |b| b.iter(|| 
}

criterion_group!(benches, bench_gallery_files);
criterion_main!(benches);

fn setup_db_with_tags() -> db::Connection {
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
