use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::io::Cursor;
use csv::{ReaderBuilder};

fn create_test_csv(rows: usize, cols: usize) -> String {
    let mut csv = String::new();
    
    // Header
    for i in 0..cols {
        if i > 0 { csv.push(','); }
        csv.push_str(&format!("col_{}", i));
    }
    csv.push('\n');
    
    // Data rows
    for row in 0..rows {
        for col in 0..cols {
            if col > 0 { csv.push(','); }
            csv.push_str(&format!("value_{}_{}", row, col));
        }
        csv.push('\n');
    }
    
    csv
}

fn bench_streaming_parser(c: &mut Criterion) {
    let mut group = c.benchmark_group("streaming_parser");
    
    // Test different file sizes
    let sizes = vec![
        (100, 5, "small"),
        (1000, 10, "medium"), 
        (10000, 20, "large"),
    ];
    
    for (rows, cols, name) in sizes {
        let csv_content = create_test_csv(rows, cols);
        let size_bytes = csv_content.len() as u64;
        
        group.throughput(Throughput::Bytes(size_bytes));
        group.bench_with_input(BenchmarkId::new("parse", name), &csv_content, |b, content| {
            b.iter(|| {
                let cursor = Cursor::new(content);
                let mut reader = ReaderBuilder::new()
                    .delimiter(b',')
                    .has_headers(true)
                    .from_reader(cursor);
                let mut count = 0;
                
                for result in reader.records() {
                    if result.is_ok() {
                        count += 1;
                    }
                }
                
                black_box(count);
            });
        });
    }
    
    group.finish();
}

fn bench_different_delimiters(c: &mut Criterion) {
    let mut group = c.benchmark_group("delimiter_performance");
    
    let csv_content = create_test_csv(1000, 10);
    let pipe_content = csv_content.replace(',', "|");
    let tab_content = csv_content.replace(',', "\t");
    
    let size_bytes = csv_content.len() as u64;
    group.throughput(Throughput::Bytes(size_bytes));
    
    group.bench_function("comma", |b| {
        b.iter(|| {
            let cursor = Cursor::new(&csv_content);
            let mut reader = ReaderBuilder::new()
                .delimiter(b',')
                .has_headers(true)
                .from_reader(cursor);
            let mut count = 0;
            
            for result in reader.records() {
                if result.is_ok() {
                    count += 1;
                }
            }
            
            black_box(count);
        });
    });
    
    group.bench_function("pipe", |b| {
        b.iter(|| {
            let cursor = Cursor::new(&pipe_content);
            let mut reader = ReaderBuilder::new()
                .delimiter(b'|')
                .has_headers(true)
                .from_reader(cursor);
            let mut count = 0;
            
            for result in reader.records() {
                if result.is_ok() {
                    count += 1;
                }
            }
            
            black_box(count);
        });
    });
    
    group.bench_function("tab", |b| {
        b.iter(|| {
            let cursor = Cursor::new(&tab_content);
            let mut reader = ReaderBuilder::new()
                .delimiter(b'\t')
                .has_headers(true)
                .from_reader(cursor);
            let mut count = 0;
            
            for result in reader.records() {
                if result.is_ok() {
                    count += 1;
                }
            }
            
            black_box(count);
        });
    });
    
    group.finish();
}

criterion_group!(benches, bench_streaming_parser, bench_different_delimiters);
criterion_main!(benches);