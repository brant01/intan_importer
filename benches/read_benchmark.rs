use criterion::{black_box, criterion_group, criterion_main, Criterion};
use intan_importer::load;
use std::path::Path;

pub fn bench_load_header(c: &mut Criterion) {
    // Set up benchmark for loading just the header, not the full file
    // You would need a sample data file in a known location
    let file_path = "path/to/small_test_file.rhs";
    
    if Path::new(file_path).exists() {
        c.bench_function("load_rhs_header", |b| {
            b.iter(|| {
                let result = black_box(load(file_path));
                black_box(result.is_ok())
            });
        });
    } else {
        println!("Skipping benchmark: test file not found at {}", file_path);
    }
}

pub fn bench_file_processing(c: &mut Criterion) {
    // Set up benchmark for processing (reading + parsing all data)
    // You would need a sample data file in a known location
    let file_path = "path/to/test_file.rhs";
    
    if Path::new(file_path).exists() {
        c.bench_function("process_rhs_file", |b| {
            b.iter(|| {
                let result = black_box(load(file_path));
                black_box(result.is_ok())
            });
        });
    } else {
        println!("Skipping benchmark: test file not found at {}", file_path);
    }
}

criterion_group!(benches, bench_load_header, bench_file_processing);
criterion_main!(benches);