use crate::analyzer::{column::ColumnAnalyzer, patterns::TypeInferencer};
use crate::types::ColumnStats;
use anyhow::{Context, Result};
use csv::ReaderBuilder;
use log;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

pub struct StreamingInferenceEngine {
    analyzers: HashMap<usize, ColumnAnalyzer>,
    headers: Vec<String>,
    row_count: usize,
    error_count: usize,
    max_errors: usize,
    inferencer: TypeInferencer,
    null_values: Vec<String>,
    verbose: bool,
}

impl StreamingInferenceEngine {
    pub fn new(
        null_values: Vec<String>,
        date_format: Option<String>,
        time_format: Option<String>,
        datetime_format: Option<String>,
        max_errors: usize,
        verbose: bool,
    ) -> Self {
        let inferencer = TypeInferencer::with_formats(date_format, time_format, datetime_format);

        StreamingInferenceEngine {
            analyzers: HashMap::new(),
            headers: Vec::new(),
            row_count: 0,
            error_count: 0,
            max_errors,
            inferencer,
            null_values,
            verbose,
        }
    }

    pub fn analyze_csv_file(
        &mut self,
        file_path: &str,
        delimiter: u8,
        quote: Option<u8>,
    ) -> Result<Vec<ColumnStats>> {
        let file =
            File::open(file_path).with_context(|| format!("Failed to open file: {}", file_path))?;

        let buf_reader = BufReader::new(file);
        self.analyze_csv_reader(buf_reader, delimiter, quote)
    }

    pub fn analyze_csv_stdin(
        &mut self,
        delimiter: u8,
        quote: Option<u8>,
    ) -> Result<Vec<ColumnStats>> {
        let stdin = std::io::stdin();
        let buf_reader = BufReader::new(stdin.lock());
        self.analyze_csv_reader(buf_reader, delimiter, quote)
    }

    fn analyze_csv_reader<R: BufRead>(
        &mut self,
        reader: R,
        delimiter: u8,
        quote: Option<u8>,
    ) -> Result<Vec<ColumnStats>> {
        let mut csv_reader = ReaderBuilder::new()
            .delimiter(delimiter)
            .quote(quote.unwrap_or(b'"'))
            .has_headers(true)
            .flexible(true)
            .from_reader(reader);

        // Read headers
        self.headers = csv_reader
            .headers()?
            .iter()
            .map(|h| h.to_string())
            .collect();

        if self.verbose {
            eprintln!("Found {} columns: {:?}", self.headers.len(), self.headers);
        }
        
        // Also log for RUST_LOG debug mode
        log::debug!("Found {} columns: {:?}", self.headers.len(), self.headers);

        // Initialize analyzers for each column
        for (i, header) in self.headers.iter().enumerate() {
            let analyzer = ColumnAnalyzer::new(
                header.clone(),
                self.inferencer.clone(),
                self.null_values.clone(),
                self.verbose,
            );
            self.analyzers.insert(i, analyzer);
        }

        // Process each record
        for result in csv_reader.records() {
            match result {
                Ok(record) => {
                    self.process_record(&record)?;
                }
                Err(e) => {
                    self.error_count += 1;
                    log::warn!("Error processing row {}: {}", self.row_count + 1, e);

                    if self.error_count >= self.max_errors {
                        return Err(anyhow::anyhow!(
                            "Too many errors ({} >= {}). Stopping processing.",
                            self.error_count,
                            self.max_errors
                        ));
                    }
                }
            }
        }

        // Finalize all analyzers
        for analyzer in self.analyzers.values_mut() {
            analyzer.finalize();
        }

        if self.verbose {
            eprintln!(
                "Analysis complete. Processed {} rows with {} errors.",
                self.row_count, self.error_count
            );
        }
        
        // Also log for RUST_LOG debug mode
        log::debug!(
            "Analysis complete. Processed {} rows with {} errors.",
            self.row_count, self.error_count
        );

        // Return column statistics in header order
        let mut stats = Vec::new();
        for i in 0..self.headers.len() {
            if let Some(analyzer) = self.analyzers.remove(&i) {
                stats.push(analyzer.into_stats());
            }
        }

        Ok(stats)
    }

    fn process_record(&mut self, record: &csv::StringRecord) -> Result<()> {
        self.row_count += 1;

        if self.verbose && self.row_count % 10000 == 0 {
            eprintln!("Processed {} rows", self.row_count);
        }
        
        // Also log for RUST_LOG debug mode (but with lower frequency to avoid spam)
        if self.row_count % 10000 == 0 {
            log::debug!("Processed {} rows", self.row_count);
        }

        // Process each field in the record
        for (i, field) in record.iter().enumerate() {
            if i >= self.headers.len() {
                if self.verbose {
                    eprintln!(
                        "Row {} has more columns than headers. Ignoring extra columns.",
                        self.row_count
                    );
                }
                
                // Also log for RUST_LOG debug mode
                log::warn!(
                    "Row {} has more columns than headers. Ignoring extra columns.",
                    self.row_count
                );
                break;
            }

            if let Some(analyzer) = self.analyzers.get_mut(&i) {
                analyzer.analyze_value(field, self.row_count);
            }
        }

        // Handle missing fields (fewer columns than headers)
        if record.len() < self.headers.len() {
            if self.verbose {
                eprintln!(
                    "Row {} has fewer columns than headers. Treating missing as null.",
                    self.row_count
                );
            }
            
            // Also log for RUST_LOG debug mode
            log::debug!(
                "Row {} has fewer columns than headers. Treating missing as null.",
                self.row_count
            );

            for i in record.len()..self.headers.len() {
                if let Some(analyzer) = self.analyzers.get_mut(&i) {
                    analyzer.analyze_value("", self.row_count); // Empty string is treated as null
                }
            }
        }

        Ok(())
    }

    pub fn get_summary(&self) -> InferenceSummary {
        InferenceSummary {
            total_rows: self.row_count,
            total_columns: self.headers.len(),
            error_count: self.error_count,
            headers: self.headers.clone(),
        }
    }

    pub fn print_type_promotions(&self) {
        if !self.verbose {
            return;
        }

        for (_i, analyzer) in &self.analyzers {
            let stats = analyzer.get_stats();
            if !stats.type_promotions.is_empty() {
                println!("\nType promotions for column '{}':", stats.name);
                for promotion in &stats.type_promotions {
                    println!("  {}", promotion);
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct InferenceSummary {
    pub total_rows: usize,
    pub total_columns: usize,
    pub error_count: usize,
    pub headers: Vec<String>,
}

impl InferenceSummary {
    pub fn success_rate(&self) -> f64 {
        if self.total_rows == 0 {
            100.0
        } else {
            ((self.total_rows - self.error_count) as f64 / self.total_rows as f64) * 100.0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_basic_inference() {
        let csv_data = "id,name,age,active\n1,Alice,25,true\n2,Bob,30,false\n3,Charlie,35,true";
        let cursor = Cursor::new(csv_data);

        let mut engine = StreamingInferenceEngine::new(vec![], None, None, None, 100, false);

        let stats = engine.analyze_csv_reader(cursor, b',', Some(b'"')).unwrap();

        assert_eq!(stats.len(), 4);
        assert_eq!(stats[0].name, "id");
        assert_eq!(stats[0].sql_type, crate::types::SqlType::SmallInt);
        assert_eq!(stats[1].name, "name");
        assert_eq!(stats[2].name, "age");
        assert_eq!(stats[2].sql_type, crate::types::SqlType::SmallInt);
        assert_eq!(stats[3].name, "active");
        assert_eq!(stats[3].sql_type, crate::types::SqlType::Boolean);
    }

    #[test]
    fn test_null_handling() {
        let csv_data = "id,value\n1,100\n2,\n3,NULL\n4,200";
        let cursor = Cursor::new(csv_data);

        let mut engine =
            StreamingInferenceEngine::new(vec!["NULL".to_string()], None, None, None, 100, false);

        let stats = engine.analyze_csv_reader(cursor, b',', Some(b'"')).unwrap();

        assert_eq!(stats.len(), 2);
        assert_eq!(stats[1].null_count, 2); // Empty string and "NULL"
        assert_eq!(stats[1].total_count, 4);
        assert_eq!(stats[1].null_percentage(), 50.0);
    }

    #[test]
    fn test_type_promotion() {
        let csv_data = "mixed\n123\ntrue\n456.78\nhello";
        let cursor = Cursor::new(csv_data);

        let mut engine = StreamingInferenceEngine::new(
            vec![],
            None,
            None,
            None,
            100,
            true, // verbose to capture promotions
        );

        let stats = engine.analyze_csv_reader(cursor, b',', Some(b'"')).unwrap();

        assert_eq!(stats.len(), 1);
        assert_eq!(stats[0].sql_type, crate::types::SqlType::Varchar(Some(5))); // "hello" is 5 chars
        assert!(!stats[0].type_promotions.is_empty());
    }

    #[test]
    fn test_missing_columns() {
        let csv_data = "a,b,c\n1,2,3\n4,5\n6"; // Second row missing c, third row missing b and c
        let cursor = Cursor::new(csv_data);

        let mut engine = StreamingInferenceEngine::new(vec![], None, None, None, 100, false);

        let stats = engine.analyze_csv_reader(cursor, b',', Some(b'"')).unwrap();

        assert_eq!(stats.len(), 3);
        assert_eq!(stats[0].null_count, 0); // Column 'a' has no nulls
        assert_eq!(stats[1].null_count, 1); // Column 'b' missing in last row
        assert_eq!(stats[2].null_count, 2); // Column 'c' missing in last 2 rows
    }
}
