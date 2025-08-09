use crate::analyzer::inference::StreamingInferenceEngine;
use crate::perf::{BufferOptimizer, PerfMetrics, StreamingOptimizer};
use crate::types::ColumnStats;
use anyhow::Result;
use log::{info, warn};
use std::fs::File;

/// Configuration structure for analysis parameters
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub delimiter: u8,
    pub quote: Option<u8>,
    pub null_values: Vec<String>,
    pub date_format: Option<String>,
    pub time_format: Option<String>,
    pub datetime_format: Option<String>,
    pub max_errors: usize,
    pub sub_newline: String,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            delimiter: b',',
            quote: Some(b'"'),
            null_values: vec!["".to_string(), "NULL".to_string()],
            date_format: None,
            time_format: None,
            datetime_format: None,
            max_errors: 100,
            sub_newline: " ".to_string(),
        }
    }
}

/// Optimized analyzer that automatically selects best performance settings
pub struct OptimizedAnalyzer {
    perf_metrics: PerfMetrics,
    verbose: bool,
}

impl OptimizedAnalyzer {
    pub fn new(verbose: bool) -> Self {
        Self {
            perf_metrics: PerfMetrics::new(),
            verbose,
        }
    }

    /// Analyze a CSV file with performance optimizations
    pub fn analyze_file(
        &mut self,
        file_path: &str,
        config: AnalysisConfig,
    ) -> Result<Vec<ColumnStats>> {
        self.perf_metrics.checkpoint("start_analysis");

        // Pre-analyze the file to optimize processing
        let (file_size, estimated_rows, estimated_columns) =
            self.analyze_file_structure(file_path, config.delimiter)?;

        if self.verbose {
            info!(
                "File analysis: size={} bytes, ~{} rows, ~{} columns",
                file_size, estimated_rows, estimated_columns
            );
        }

        self.perf_metrics.checkpoint("file_structure_analyzed");

        // Calculate optimal settings
        let available_memory = BufferOptimizer::get_available_memory();
        let optimal_buffer = BufferOptimizer::calculate_buffer_size(file_size, available_memory);
        let chunk_size =
            StreamingOptimizer::calculate_chunk_size(estimated_rows, estimated_columns);

        if self.verbose {
            info!(
                "Optimization settings: buffer={} bytes, chunk_size={} rows",
                optimal_buffer, chunk_size
            );
        }

        // Estimate memory requirements
        let estimated_memory =
            StreamingOptimizer::estimate_memory_for_analysis(estimated_rows, estimated_columns);
        if estimated_memory > available_memory / 2 {
            // Use half available memory as warning threshold
            warn!(
                "Analysis may require significant memory: {} bytes estimated",
                estimated_memory
            );
        }

        self.perf_metrics.checkpoint("optimization_calculated");
        self.perf_metrics.record_memory("pre_analysis");

        // Create optimized inference engine
        let mut engine = StreamingInferenceEngine::new(
            config.null_values,
            config.date_format,
            config.time_format,
            config.datetime_format,
            config.max_errors,
            self.verbose,
            config.sub_newline,
        );

        self.perf_metrics.checkpoint("engine_created");

        // Run the analysis
        let result = engine.analyze_csv_file(file_path, config.delimiter, config.quote)?;

        self.perf_metrics.checkpoint("analysis_complete");
        self.perf_metrics.record_memory("post_analysis");

        if self.verbose {
            self.perf_metrics.print_summary();
            self.print_analysis_summary(&result);
        }

        Ok(result)
    }

    /// Quick analysis of file structure for optimization
    fn analyze_file_structure(
        &self,
        file_path: &str,
        delimiter: u8,
    ) -> Result<(u64, usize, usize)> {
        let file = File::open(file_path)?;
        let file_size = file.metadata()?.len();

        // Sample first few lines to estimate structure
        use std::io::{BufRead, BufReader};
        let mut reader = BufReader::with_capacity(8192, file);
        let mut line = String::new();

        // Read header to count columns
        let columns = if reader.read_line(&mut line)? > 0 {
            line.trim().split(delimiter as char).count()
        } else {
            1 // Default if file is empty
        };

        // Estimate rows based on file size and sample line length
        let estimated_rows = if !line.is_empty() {
            (file_size as usize / line.len()).max(1)
        } else {
            1000 // Default estimate
        };

        Ok((file_size, estimated_rows, columns))
    }

    fn print_analysis_summary(&self, results: &[ColumnStats]) {
        info!("Analysis Summary:");
        info!("  - Analyzed {} columns", results.len());

        let total_rows = results.first().map(|r| r.total_count).unwrap_or(0);
        info!("  - Total rows processed: {}", total_rows);

        let null_columns = results.iter().filter(|r| r.null_count > 0).count();
        info!("  - Columns with nulls: {}", null_columns);

        // Type distribution
        use std::collections::HashMap;
        let mut type_counts = HashMap::new();
        for result in results {
            *type_counts.entry(&result.sql_type).or_insert(0) += 1;
        }

        info!("  - Type distribution:");
        for (sql_type, count) in type_counts {
            info!("    {:?}: {}", sql_type, count);
        }
    }
}

/// Performance testing utilities for regression testing
pub struct PerformanceTester;

impl PerformanceTester {
    /// Run performance regression tests
    pub fn run_regression_tests() -> Result<()> {
        info!("Running performance regression tests...");

        // Test with different file sizes
        let test_cases = vec![
            (1000, 5, "small_file"),
            (10000, 20, "medium_file"),
            (50000, 10, "large_file"),
        ];

        for (rows, cols, name) in test_cases {
            let test_data = Self::create_test_csv(rows, cols);
            use tempfile::NamedTempFile;
            let temp_file = NamedTempFile::new()?;
            std::fs::write(temp_file.path(), test_data)?;

            let mut analyzer = OptimizedAnalyzer::new(true);
            let start_time = std::time::Instant::now();

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
            let _results = analyzer.analyze_file(temp_file.path().to_str().unwrap(), config)?;

            let duration = start_time.elapsed();
            info!("Test '{}' completed in {:?}", name, duration);

            // Performance thresholds (these could be adjusted based on requirements)
            let max_duration = match name {
                "small_file" => std::time::Duration::from_millis(100),
                "medium_file" => std::time::Duration::from_millis(500),
                "large_file" => std::time::Duration::from_secs(5),
                _ => std::time::Duration::from_secs(10),
            };

            if duration > max_duration {
                warn!(
                    "Performance regression detected for '{}': {:?} > {:?}",
                    name, duration, max_duration
                );
            }
        }

        info!("Performance regression tests completed");
        Ok(())
    }

    fn create_test_csv(rows: usize, cols: usize) -> String {
        let mut csv = String::with_capacity(rows * cols * 10); // Pre-allocate

        // Header
        for i in 0..cols {
            if i > 0 {
                csv.push(',');
            }
            csv.push_str(&format!("col_{}", i));
        }
        csv.push('\n');

        // Data
        for row in 0..rows {
            for col in 0..cols {
                if col > 0 {
                    csv.push(',');
                }
                let value = match col % 5 {
                    0 => row.to_string(),
                    1 => format!("{}.{}", row, col),
                    2 => if row % 2 == 0 { "true" } else { "false" }.to_string(),
                    3 => "2024-01-15".to_string(),
                    _ => format!("value_{}", row),
                };
                csv.push_str(&value);
            }
            csv.push('\n');
        }

        csv
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_optimized_analyzer() -> Result<()> {
        let test_csv = "id,name,value\n1,test,123.45\n2,sample,678.90\n";
        let temp_file = NamedTempFile::new()?;
        std::fs::write(temp_file.path(), test_csv)?;

        let mut analyzer = OptimizedAnalyzer::new(false);
        let config = AnalysisConfig {
            delimiter: b',',
            quote: Some(b'"'),
            null_values: vec!["NULL".to_string()],
            date_format: None,
            time_format: None,
            datetime_format: None,
            max_errors: 100,
            sub_newline: " ".to_string(),
        };
        let results = analyzer.analyze_file(temp_file.path().to_str().unwrap(), config)?;

        assert_eq!(results.len(), 3); // 3 columns
        Ok(())
    }

    #[test]
    fn test_file_structure_analysis() -> Result<()> {
        let test_csv = "a,b,c,d\n1,2,3,4\n5,6,7,8\n";
        let temp_file = NamedTempFile::new()?;
        std::fs::write(temp_file.path(), test_csv)?;

        let analyzer = OptimizedAnalyzer::new(false);
        let (size, rows, cols) =
            analyzer.analyze_file_structure(temp_file.path().to_str().unwrap(), b',')?;

        assert!(size > 0);
        assert_eq!(cols, 4);
        assert!(rows > 0);
        Ok(())
    }
}
