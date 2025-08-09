use file2ddl::analyzer::optimized::{AnalysisConfig, OptimizedAnalyzer, PerformanceTester};
use std::time::Duration;
use tempfile::NamedTempFile;

#[test]
fn test_small_file_performance() {
    let test_csv = create_test_csv(100, 5);
    let temp_file = NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), test_csv).unwrap();

    let mut analyzer = OptimizedAnalyzer::new(false);
    let start = std::time::Instant::now();

    let config = AnalysisConfig {
        delimiter: b',',
        quote: Some(b'"'),
        null_values: vec!["NULL".to_string(), "".to_string()],
        date_format: None,
        time_format: None,
        datetime_format: None,
        max_errors: 1000,
        sub_newline: " ".to_string(),
    };
    let _results = analyzer
        .analyze_file(temp_file.path().to_str().unwrap(), config)
        .unwrap();

    let duration = start.elapsed();

    // Small files should complete quickly
    assert!(
        duration < Duration::from_millis(200),
        "Small file analysis took too long: {:?}",
        duration
    );
}

#[test]
fn test_medium_file_performance() {
    let test_csv = create_test_csv(5000, 15);
    let temp_file = NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), test_csv).unwrap();

    let mut analyzer = OptimizedAnalyzer::new(false);
    let start = std::time::Instant::now();

    let config = AnalysisConfig {
        delimiter: b',',
        quote: Some(b'"'),
        null_values: vec!["NULL".to_string(), "".to_string()],
        date_format: None,
        time_format: None,
        datetime_format: None,
        max_errors: 1000,
        sub_newline: " ".to_string(),
    };
    let _results = analyzer
        .analyze_file(temp_file.path().to_str().unwrap(), config)
        .unwrap();

    let duration = start.elapsed();

    // Medium files should complete in reasonable time
    assert!(
        duration < Duration::from_secs(2),
        "Medium file analysis took too long: {:?}",
        duration
    );
}

#[test]
fn test_memory_scaling() {
    // Test that memory usage doesn't explode with larger files
    let sizes = vec![(1000, 5), (5000, 10), (10000, 15)];

    for (rows, cols) in sizes {
        let test_csv = create_test_csv(rows, cols);
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), test_csv).unwrap();

        let mut analyzer = OptimizedAnalyzer::new(false);

        // This should not panic or run out of memory
        let config = AnalysisConfig {
            delimiter: b',',
            quote: Some(b'"'),
            null_values: vec!["NULL".to_string(), "".to_string()],
            date_format: None,
            time_format: None,
            datetime_format: None,
            max_errors: 1000,
            sub_newline: " ".to_string(),
        };
        let results = analyzer
            .analyze_file(temp_file.path().to_str().unwrap(), config)
            .unwrap();

        assert_eq!(results.len(), cols);
        assert!(results.iter().all(|r| r.total_count == rows));
    }
}

#[test]
fn test_column_scaling_performance() {
    // Test performance with increasing column counts
    let column_counts = vec![5, 20, 50];

    for cols in column_counts {
        let test_csv = create_test_csv(1000, cols);
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), test_csv).unwrap();

        let mut analyzer = OptimizedAnalyzer::new(false);
        let start = std::time::Instant::now();

        let config = AnalysisConfig {
            delimiter: b',',
            quote: Some(b'"'),
            null_values: vec!["NULL".to_string(), "".to_string()],
            date_format: None,
            time_format: None,
            datetime_format: None,
            max_errors: 1000,
            sub_newline: " ".to_string(),
        };
        let results = analyzer
            .analyze_file(temp_file.path().to_str().unwrap(), config)
            .unwrap();

        let duration = start.elapsed();

        assert_eq!(results.len(), cols);

        // Performance should scale reasonably with column count
        let max_duration = Duration::from_millis(100 * cols as u64);
        assert!(
            duration < max_duration,
            "Analysis with {} columns took too long: {:?}",
            cols,
            duration
        );
    }
}

#[test]
fn test_type_inference_performance() {
    // Test performance with different type complexities
    let test_cases = vec![
        ("simple_integers", create_integer_csv(2000)),
        ("mixed_types", create_mixed_type_csv(2000)),
        ("string_heavy", create_string_heavy_csv(2000)),
    ];

    for (name, test_csv) in test_cases {
        let temp_file = NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), test_csv).unwrap();

        let mut analyzer = OptimizedAnalyzer::new(false);
        let start = std::time::Instant::now();

        let config = AnalysisConfig {
            delimiter: b',',
            quote: Some(b'"'),
            null_values: vec!["NULL".to_string(), "".to_string()],
            date_format: None,
            time_format: None,
            datetime_format: None,
            max_errors: 1000,
            sub_newline: " ".to_string(),
        };
        let _results = analyzer
            .analyze_file(temp_file.path().to_str().unwrap(), config)
            .unwrap();

        let duration = start.elapsed();

        // All type inference scenarios should complete reasonably quickly
        assert!(
            duration < Duration::from_secs(1),
            "Type inference test '{}' took too long: {:?}",
            name,
            duration
        );
    }
}

#[test]
fn test_performance_regression_suite() {
    // Run the full regression test suite
    let result = PerformanceTester::run_regression_tests();
    assert!(
        result.is_ok(),
        "Performance regression tests failed: {:?}",
        result
    );
}

// Test data generation helpers

fn create_test_csv(rows: usize, cols: usize) -> String {
    let mut csv = String::new();

    // Header
    for i in 0..cols {
        if i > 0 {
            csv.push(',');
        }
        csv.push_str(&format!("col_{}", i));
    }
    csv.push('\n');

    // Data rows
    for row in 0..rows {
        for col in 0..cols {
            if col > 0 {
                csv.push(',');
            }
            let value = match col % 6 {
                0 => row.to_string(),
                1 => format!("{}.{}", row, col % 100),
                2 => if row % 2 == 0 { "true" } else { "false" }.to_string(),
                3 => "2024-01-15".to_string(),
                4 => "14:30:25".to_string(),
                _ => format!("text_value_{}", row),
            };
            csv.push_str(&value);
        }
        csv.push('\n');
    }

    csv
}

fn create_integer_csv(rows: usize) -> String {
    let mut csv = String::from("id,value1,value2\n");

    for i in 0..rows {
        csv.push_str(&format!("{},{},{}\n", i, i * 2, i * 3));
    }

    csv
}

fn create_mixed_type_csv(rows: usize) -> String {
    let mut csv = String::from("id,amount,active,created,name\n");

    for i in 0..rows {
        csv.push_str(&format!(
            "{},{:.2},{},{},user_{}\n",
            i,
            i as f64 * 1.5,
            i % 2 == 0,
            "2024-01-15 12:30:45",
            i
        ));
    }

    csv
}

fn create_string_heavy_csv(rows: usize) -> String {
    let mut csv = String::from("id,description,category,tags\n");

    for i in 0..rows {
        csv.push_str(&format!(
            "{},\"This is a longer description for item number {} with various details\",category_{},\"tag1,tag2,tag3\"\n",
            i, i, i % 10
        ));
    }

    csv
}
