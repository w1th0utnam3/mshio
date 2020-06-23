use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::fs;
use std::path::Path;

/// Relative path to the directory containing the test mesh data
static TEST_DATA_DIR: &'static str = "tests/data";

/// Reads a whole test mesh file from the data directory as a vector of bytes
fn read_to_bytes<P: AsRef<Path>>(filename: P) -> Vec<u8> {
    let filepath = Path::join(TEST_DATA_DIR.as_ref(), filename.as_ref());
    fs::read(filepath).unwrap()
}

fn parse_msh_from_file(mshfile: &[u8]) -> mshio::MshFile<u64, i32, f64> {
    mshio::parse_msh_bytes(mshfile).unwrap()
}

fn criterion_benchmark(c: &mut Criterion) {
    let cylinder_msh = read_to_bytes("cylinder_3d.msh");
    let curved_bike_msh = read_to_bytes("bike_original.obj_curved.msh");

    let mut group = c.benchmark_group("import meshes");
    group.sample_size(20);
    group.bench_function("cylinder_3d", |b| {
        b.iter(|| parse_msh_from_file(black_box(&cylinder_msh)))
    });
    group.sample_size(100);
    group.bench_function("fine_bike_curved", |b| {
        b.iter(|| parse_msh_from_file(black_box(&curved_bike_msh)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
