use criterion::{black_box, criterion_group, criterion_main, Criterion};

pub fn bench_gallery_files(c: &mut Criterion) {
    // c.bench_function("1) with tags", |b| b.iter(|| 
}

criterion_group!(benches, bench_gallery_files);
criterion_main!(benches);

fn setup_db_with_tags() -> db::Connection {
    // FIXME: for benchmarks, should we use on-disk database for consistency with real-life results?
    let conn = db::open_in_memory();
    // FIXME: insert some sample data
    let sql = "INSERT INTO tag(name, hidden) VALUES(?,?)";
    conn.execute(sql, params!["foo", true]).unwrap();
    conn.execute(sql, params!["bar", false]).unwrap();

    // FIXME: insert sample files
    // let sql = 

    // FIXME: connect tags with files - table file_tag

    conn
}
