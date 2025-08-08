use crate::analyzer::patterns::TypeInferencer;
use crate::types::{ColumnStats, SqlType};
use std::collections::HashSet;

const MAX_SAMPLE_VALUES: usize = 10;
const MAX_UNIQUE_VALUES: usize = 1000;

#[derive(Debug)]
pub struct ColumnAnalyzer {
    stats: ColumnStats,
    inferencer: TypeInferencer,
    null_values: HashSet<String>,
    unique_values: HashSet<String>,
    first_non_null_type: Option<SqlType>,
}

impl ColumnAnalyzer {
    pub fn new(name: String, inferencer: TypeInferencer, null_values: Vec<String>) -> Self {
        let mut null_set = HashSet::new();
        null_set.insert("".to_string());
        null_set.insert("NULL".to_string());
        null_set.insert("null".to_string());

        for val in null_values {
            null_set.insert(val);
        }

        ColumnAnalyzer {
            stats: ColumnStats::new(name),
            inferencer,
            null_values: null_set,
            unique_values: HashSet::new(),
            first_non_null_type: None,
        }
    }

    pub fn analyze_value(&mut self, value: &str) {
        self.stats.total_count += 1;

        let trimmed = value.trim();

        // Check for null values
        if self.is_null_value(trimmed) {
            self.stats.null_count += 1;
            return;
        }

        // Track unique values (with limit to prevent memory explosion)
        if self.unique_values.len() < MAX_UNIQUE_VALUES {
            self.unique_values.insert(trimmed.to_string());
        }

        // Update max length
        self.stats.max_length = self.stats.max_length.max(trimmed.len());

        // Update min/max values for ordering
        self.update_min_max(trimmed);

        // Add to sample values
        if self.stats.sample_values.len() < MAX_SAMPLE_VALUES
            && !self.stats.sample_values.contains(&trimmed.to_string())
        {
            self.stats.sample_values.push(trimmed.to_string());
        }

        // Infer type and potentially promote
        let inferred_type = self.inferencer.infer_type(trimmed);
        self.update_type(inferred_type, trimmed);
    }

    fn is_null_value(&self, value: &str) -> bool {
        self.null_values.contains(value)
    }

    fn update_min_max(&mut self, value: &str) {
        match (&self.stats.min_value, &self.stats.max_value) {
            (None, None) => {
                self.stats.min_value = Some(value.to_string());
                self.stats.max_value = Some(value.to_string());
            }
            (Some(min), Some(max)) => {
                if value < min {
                    self.stats.min_value = Some(value.to_string());
                }
                if value > max {
                    self.stats.max_value = Some(value.to_string());
                }
            }
            _ => {} // Should not happen
        }
    }

    fn update_type(&mut self, new_type: SqlType, value: &str) {
        // If this is our first non-null value, set the initial type
        if self.first_non_null_type.is_none() {
            self.first_non_null_type = Some(new_type.clone());
            self.stats.sql_type = new_type;
            return;
        }

        // Check if we need to promote the type
        if self.stats.sql_type != new_type {
            let promoted_type = self.stats.sql_type.promote(&new_type);

            if promoted_type != self.stats.sql_type {
                // Log the promotion
                let promotion_msg = format!(
                    "Promoted from {} to {} due to value: '{}'",
                    self.stats.sql_type, promoted_type, value
                );
                self.stats.type_promotions.push(promotion_msg);
                self.stats.sql_type = promoted_type;
            }
        }

        // Handle VARCHAR sizing
        if let SqlType::Varchar(current_size) = &self.stats.sql_type {
            if let SqlType::Varchar(new_size) = &new_type {
                match (current_size, new_size) {
                    (Some(current), Some(new)) => {
                        if *new > *current {
                            self.stats.sql_type = SqlType::Varchar(Some(*new));
                        }
                    }
                    (Some(_), None) => {
                        self.stats.sql_type = SqlType::Varchar(None);
                    }
                    _ => {} // Keep current
                }
            }
        }
    }

    pub fn finalize(&mut self) {
        // Final adjustments to type based on statistics

        // If we have a VARCHAR with a size, consider if we should make it unlimited
        if let SqlType::Varchar(Some(size)) = &self.stats.sql_type {
            if *size > 4000 {
                // Arbitrary threshold for "large" text
                self.stats.sql_type = SqlType::Varchar(None);
            }
        }

        // Ensure VARCHAR has a minimum size of 1 even if all values were null/empty
        if let SqlType::Varchar(Some(0)) = &self.stats.sql_type {
            self.stats.sql_type = SqlType::Varchar(Some(1));
        }
    }

    pub fn get_stats(&self) -> &ColumnStats {
        &self.stats
    }

    pub fn into_stats(self) -> ColumnStats {
        self.stats
    }

    pub fn unique_value_count(&self) -> usize {
        self.unique_values.len()
    }

    pub fn cardinality_ratio(&self) -> f64 {
        if self.stats.total_count == 0 {
            0.0
        } else {
            self.unique_values.len() as f64
                / (self.stats.total_count - self.stats.null_count) as f64
        }
    }

    pub fn is_likely_categorical(&self) -> bool {
        let non_null_count = self.stats.total_count - self.stats.null_count;

        if non_null_count == 0 {
            return false;
        }

        // Consider it categorical if:
        // 1. Low cardinality ratio (< 0.1) and reasonable number of values
        // 2. Very few unique values (< 20) regardless of ratio
        let cardinality = self.cardinality_ratio();
        let unique_count = self.unique_values.len();

        (cardinality < 0.1 && non_null_count > 10) || unique_count < 20
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyzer::patterns::TypeInferencer;

    #[test]
    fn test_basic_analysis() {
        let inferencer = TypeInferencer::new();
        let mut analyzer = ColumnAnalyzer::new("test_col".to_string(), inferencer, vec![]);

        analyzer.analyze_value("123");
        analyzer.analyze_value("456");
        analyzer.analyze_value("789");

        let stats = analyzer.get_stats();
        assert_eq!(stats.sql_type, SqlType::SmallInt);
        assert_eq!(stats.total_count, 3);
        assert_eq!(stats.null_count, 0);
        assert_eq!(stats.max_length, 3);
    }

    #[test]
    fn test_type_promotion() {
        let inferencer = TypeInferencer::new();
        let mut analyzer = ColumnAnalyzer::new("test_col".to_string(), inferencer, vec![]);

        analyzer.analyze_value("123"); // SmallInt
        analyzer.analyze_value("true"); // Boolean -> promotes to SmallInt
        analyzer.analyze_value("2147483648"); // BigInt -> promotes to BigInt

        let stats = analyzer.get_stats();
        assert_eq!(stats.sql_type, SqlType::BigInt);
        assert!(stats.type_promotions.len() > 0);
    }

    #[test]
    fn test_null_handling() {
        let inferencer = TypeInferencer::new();
        let mut analyzer =
            ColumnAnalyzer::new("test_col".to_string(), inferencer, vec!["N/A".to_string()]);

        analyzer.analyze_value("123");
        analyzer.analyze_value("");
        analyzer.analyze_value("NULL");
        analyzer.analyze_value("N/A");
        analyzer.analyze_value("456");

        let stats = analyzer.get_stats();
        assert_eq!(stats.total_count, 5);
        assert_eq!(stats.null_count, 3);
        assert_eq!(stats.sql_type, SqlType::SmallInt);
        assert_eq!(stats.null_percentage(), 60.0);
    }

    #[test]
    fn test_varchar_sizing() {
        let inferencer = TypeInferencer::new();
        let mut analyzer = ColumnAnalyzer::new("test_col".to_string(), inferencer, vec![]);

        analyzer.analyze_value("short");
        analyzer.analyze_value("a much longer string value");

        let stats = analyzer.get_stats();
        assert_eq!(stats.sql_type, SqlType::Varchar(Some(26)));
        assert_eq!(stats.max_length, 26);
    }

    #[test]
    fn test_categorical_detection() {
        let inferencer = TypeInferencer::new();
        let mut analyzer = ColumnAnalyzer::new("status".to_string(), inferencer, vec![]);

        // Add many values but only a few unique ones
        for _ in 0..100 {
            analyzer.analyze_value("active");
        }
        for _ in 0..50 {
            analyzer.analyze_value("inactive");
        }
        for _ in 0..25 {
            analyzer.analyze_value("pending");
        }

        assert!(analyzer.is_likely_categorical());
        assert_eq!(analyzer.unique_value_count(), 3);
        assert!(analyzer.cardinality_ratio() < 0.1);
    }
}
