use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use file2ddl::analyzer::inference::StreamingInferenceEngine;
use tempfile;

/// Memory profiling benchmark to test memory efficiency of different approaches
fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_efficiency");
    
    // Test memory scaling with file size
    let file_sizes = vec![
        (1_000, "1K_rows"),
        (10_000, "10K_rows"),
        (100_000, "100K_rows"),
    ];
    
    for (rows, name) in file_sizes {
        let csv_content = create_large_csv(rows, 20); // 20 columns
        let size_bytes = csv_content.len() as u64;
        
        group.throughput(Throughput::Bytes(size_bytes));
        group.bench_with_input(BenchmarkId::new("streaming_analysis", name), &csv_content, |b, content| {
            b.iter(|| {
                // Test our streaming approach - should use constant memory
                let mut engine = StreamingInferenceEngine::new(
                    vec!["NULL".to_string(), "".to_string()],
                    None,
                    None,
                    None,
                    1000,
                    false
                );
                
                let temp_file = tempfile::NamedTempFile::new().unwrap();
                std::fs::write(temp_file.path(), content).unwrap();
                
                let result = engine.analyze_csv_file(
                    temp_file.path().to_str().unwrap(),
                    b',',
                    Some(b'"')
                ).unwrap();
                
                black_box(result);
            });
        });
    }
    
    group.finish();
}

/// Test buffer size impact on performance
fn bench_buffer_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("buffer_optimization");
    
    // Create a moderately sized test file
    let csv_content = create_large_csv(10_000, 15);
    let size_bytes = csv_content.len() as u64;
    
    // We can't directly control buffer size in our current implementation,
    // but we can test different approaches that might affect buffering
    group.throughput(Throughput::Bytes(size_bytes));
    
    group.bench_function("standard_analysis", |b| {
        b.iter(|| {
            let mut engine = StreamingInferenceEngine::new(
                vec!["NULL".to_string(), "".to_string()],
                None,
                None,
                None,
                1000,
                false
            );
            
            let temp_file = tempfile::NamedTempFile::new().unwrap();
            std::fs::write(temp_file.path(), &csv_content).unwrap();
            
            let result = engine.analyze_csv_file(
                temp_file.path().to_str().unwrap(),
                b',',
                Some(b'"')
            ).unwrap();
            
            black_box(result);
        });
    });
    
    group.finish();
}

/// Test column count scaling
fn bench_column_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("column_scaling");
    
    let column_counts = vec![
        (5, "5_cols"),
        (20, "20_cols"),
        (50, "50_cols"),
        (100, "100_cols"),
    ];
    
    for (cols, name) in column_counts {
        let csv_content = create_large_csv(5_000, cols);
        let size_bytes = csv_content.len() as u64;
        
        group.throughput(Throughput::Bytes(size_bytes));
        group.bench_with_input(BenchmarkId::new("analyze", name), &csv_content, |b, content| {
            b.iter(|| {
                let mut engine = StreamingInferenceEngine::new(
                    vec!["NULL".to_string(), "".to_string()],
                    None,
                    None,
                    None,
                    1000,
                    false
                );
                
                let temp_file = tempfile::NamedTempFile::new().unwrap();
                std::fs::write(temp_file.path(), content).unwrap();
                
                let result = engine.analyze_csv_file(
                    temp_file.path().to_str().unwrap(),
                    b',',
                    Some(b'"')
                ).unwrap();
                
                black_box(result);
            });
        });
    }
    
    group.finish();
}

fn create_large_csv(rows: usize, cols: usize) -> String {
    let mut csv = String::new();
    
    // Create header
    for i in 0..cols {
        if i > 0 { csv.push(','); }
        csv.push_str(&format!("column_{}", i));
    }
    csv.push('\n');
    
    // Create data - mix different types to test type inference
    for row in 0..rows {
        for col in 0..cols {
            if col > 0 { csv.push(','); }
            
            let value = match col % 8 {
                0 => format!("{}", row), // Integer
                1 => format!("{}.{}", row, col % 100), // Float  
                2 => if row % 2 == 0 { "true".to_string() } else { "false".to_string() }, // Boolean
                3 => "2024-01-15".to_string(), // Date
                4 => "14:30:25".to_string(), // Time
                5 => "2024-01-15 14:30:25".to_string(), // DateTime
                6 => if row % 10 == 0 { "NULL".to_string() } else { format!("text_{}", row) }, // Varchar with nulls
                _ => format!("value_{}_{}", row, col), // Mixed
            };
            
            csv.push_str(&value);
        }
        csv.push('\n');
    }
    
    csv
}

criterion_group!(
    benches, 
    bench_memory_usage,
    bench_buffer_sizes,
    bench_column_scaling
);
criterion_main!(benches);