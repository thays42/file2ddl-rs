use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use file2ddl::analyzer::inference::StreamingInferenceEngine;
use file2ddl::types::SqlType;

fn create_test_data_for_type(sql_type: &SqlType, rows: usize) -> String {
    let mut csv = String::new();
    csv.push_str("test_column\n");

    for i in 0..rows {
        let value = match sql_type {
            SqlType::Boolean => {
                if i % 2 == 0 {
                    "true"
                } else {
                    "false"
                }
            }
            SqlType::SmallInt => &(i as i16 % 100).to_string(),
            SqlType::Integer => &(i as i32).to_string(),
            SqlType::BigInt => &(i as i64).to_string(),
            SqlType::DoublePrecision => &format!("{}.{}", i, i % 100),
            SqlType::Date => "2024-01-15",
            SqlType::Time => "14:30:25",
            SqlType::DateTime => "2024-01-15 14:30:25",
            SqlType::Varchar(_) => &format!("string_value_{}", i),
        };
        csv.push_str(value);
        csv.push('\n');
    }

    csv
}

fn bench_type_inference_by_type(c: &mut Criterion) {
    let mut group = c.benchmark_group("type_inference_by_type");

    let row_count = 1000;
    let test_types = vec![
        (SqlType::Boolean, "boolean"),
        (SqlType::Integer, "integer"),
        (SqlType::DoublePrecision, "double"),
        (SqlType::Date, "date"),
        (SqlType::Varchar(Some(50)), "varchar"),
    ];

    for (sql_type, name) in test_types {
        let csv_content = create_test_data_for_type(&sql_type, row_count);
        let size_bytes = csv_content.len() as u64;

        group.throughput(Throughput::Bytes(size_bytes));
        group.bench_with_input(
            BenchmarkId::new("infer", name),
            &csv_content,
            |b, content| {
                b.iter(|| {
                    let mut engine = StreamingInferenceEngine::new(
                        vec!["NULL".to_string(), "".to_string()],
                        None,
                        None,
                        None,
                        1000,
                        false,
                    );

                    // Create temporary file for testing
                    let temp_file = tempfile::NamedTempFile::new().unwrap();
                    std::fs::write(temp_file.path(), content).unwrap();

                    let result = engine
                        .analyze_csv_file(temp_file.path().to_str().unwrap(), b',', Some(b'"'))
                        .unwrap();
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

fn bench_type_promotion_complexity(c: &mut Criterion) {
    let mut group = c.benchmark_group("type_promotion");

    // Create CSV with mixed types that will cause promotions
    let mixed_data = "value\n1\n2.5\ntrue\n100000000000\ntext\n";
    let size_bytes = mixed_data.len() as u64;

    group.throughput(Throughput::Bytes(size_bytes));
    group.bench_function("mixed_types", |b| {
        b.iter(|| {
            let mut engine = StreamingInferenceEngine::new(
                vec!["NULL".to_string(), "".to_string()],
                None,
                None,
                None,
                1000,
                false,
            );

            // Create temporary file for testing
            let temp_file = tempfile::NamedTempFile::new().unwrap();
            std::fs::write(temp_file.path(), mixed_data).unwrap();

            let result = engine
                .analyze_csv_file(temp_file.path().to_str().unwrap(), b',', Some(b'"'))
                .unwrap();
            black_box(result);
        });
    });

    // Create CSV with only numeric promotions (faster path)
    let numeric_data = "value\n1\n2\n3000000000\n4.5\n";

    group.bench_function("numeric_promotion", |b| {
        b.iter(|| {
            let mut engine = StreamingInferenceEngine::new(
                vec!["NULL".to_string(), "".to_string()],
                None,
                None,
                None,
                1000,
                false,
            );

            // Create temporary file for testing
            let temp_file = tempfile::NamedTempFile::new().unwrap();
            std::fs::write(temp_file.path(), numeric_data).unwrap();

            let result = engine
                .analyze_csv_file(temp_file.path().to_str().unwrap(), b',', Some(b'"'))
                .unwrap();
            black_box(result);
        });
    });

    group.finish();
}

fn bench_large_file_inference(c: &mut Criterion) {
    let mut group = c.benchmark_group("large_file_inference");

    // Test scaling with different file sizes
    let sizes = vec![(1000, "small"), (10000, "medium"), (100000, "large")];

    for (rows, name) in sizes {
        let csv_content = create_test_data_for_type(&SqlType::Integer, rows);
        let size_bytes = csv_content.len() as u64;

        group.throughput(Throughput::Bytes(size_bytes));
        group.bench_with_input(
            BenchmarkId::new("analyze", name),
            &csv_content,
            |b, content| {
                b.iter(|| {
                    let mut engine = StreamingInferenceEngine::new(
                        vec!["NULL".to_string(), "".to_string()],
                        None,
                        None,
                        None,
                        1000,
                        false,
                    );

                    // Create temporary file for testing
                    let temp_file = tempfile::NamedTempFile::new().unwrap();
                    std::fs::write(temp_file.path(), content).unwrap();

                    let result = engine
                        .analyze_csv_file(temp_file.path().to_str().unwrap(), b',', Some(b'"'))
                        .unwrap();
                    black_box(result);
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_type_inference_by_type,
    bench_type_promotion_complexity,
    bench_large_file_inference
);
criterion_main!(benches);
