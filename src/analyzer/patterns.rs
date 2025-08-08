use crate::types::SqlType;
use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use regex::Regex;
use std::sync::OnceLock;

pub struct TypePatterns {
    boolean_true: Regex,
    boolean_false: Regex,
    integer: Regex,
    double: Regex,
    date: Regex,
    time: Regex,
    datetime: Regex,
}

static PATTERNS: OnceLock<TypePatterns> = OnceLock::new();

impl TypePatterns {
    pub fn new() -> Self {
        TypePatterns {
            boolean_true: Regex::new(r"^(?i)(true|t|yes|y|1)$").unwrap(),
            boolean_false: Regex::new(r"^(?i)(false|f|no|n|0)$").unwrap(),
            integer: Regex::new(r"^[+-]?\d+$").unwrap(),
            double: Regex::new(r"^[+-]?(\d+\.?\d*|\.\d+)([eE][+-]?\d+)?$").unwrap(),
            date: Regex::new(r"^\d{4}-\d{2}-\d{2}$").unwrap(),
            time: Regex::new(r"^\d{1,2}:\d{2}:\d{2}$").unwrap(),
            datetime: Regex::new(r"^\d{4}-\d{2}-\d{2} \d{1,2}:\d{2}:\d{2}$").unwrap(),
        }
    }

    pub fn get() -> &'static TypePatterns {
        PATTERNS.get_or_init(TypePatterns::new)
    }
}

#[derive(Debug, Clone)]
pub struct TypeInferencer {
    date_format: String,
    time_format: String,
    datetime_format: String,
    true_values: Vec<String>,
    false_values: Vec<String>,
}

impl TypeInferencer {
    pub fn new() -> Self {
        TypeInferencer {
            date_format: "%Y-%m-%d".to_string(),
            time_format: "%H:%M:%S".to_string(),
            datetime_format: "%Y-%m-%d %H:%M:%S".to_string(),
            true_values: vec![
                "true".to_string(),
                "t".to_string(),
                "yes".to_string(),
                "y".to_string(),
                "1".to_string(),
            ],
            false_values: vec![
                "false".to_string(),
                "f".to_string(),
                "no".to_string(),
                "n".to_string(),
                "0".to_string(),
            ],
        }
    }

    pub fn with_formats(
        date_fmt: Option<String>,
        time_fmt: Option<String>,
        datetime_fmt: Option<String>,
    ) -> Self {
        let mut inferencer = Self::new();
        if let Some(fmt) = date_fmt {
            inferencer.date_format = fmt;
        }
        if let Some(fmt) = time_fmt {
            inferencer.time_format = fmt;
        }
        if let Some(fmt) = datetime_fmt {
            inferencer.datetime_format = fmt;
        }
        inferencer
    }

    pub fn with_boolean_values(mut self, true_vals: Vec<String>, false_vals: Vec<String>) -> Self {
        self.true_values = true_vals;
        self.false_values = false_vals;
        self
    }

    pub fn infer_type(&self, value: &str) -> SqlType {
        let trimmed = value.trim();

        if trimmed.is_empty() {
            return SqlType::Varchar(Some(1));
        }

        let patterns = TypePatterns::get();

        // Check boolean
        if self.is_boolean_true(trimmed) || self.is_boolean_false(trimmed) {
            return SqlType::Boolean;
        }

        // Check integer
        if patterns.integer.is_match(trimmed) {
            if let Ok(num) = trimmed.parse::<i64>() {
                return match num {
                    -32768..=32767 => SqlType::SmallInt,
                    -2147483648..=2147483647 => SqlType::Integer,
                    _ => SqlType::BigInt,
                };
            }
        }

        // Check double
        if patterns.double.is_match(trimmed) {
            if trimmed.parse::<f64>().is_ok() {
                return SqlType::DoublePrecision;
            }
        }

        // Check date
        if self.is_date(trimmed) {
            return SqlType::Date;
        }

        // Check time
        if self.is_time(trimmed) {
            return SqlType::Time;
        }

        // Check datetime
        if self.is_datetime(trimmed) {
            return SqlType::DateTime;
        }

        // Default to VARCHAR with length
        SqlType::Varchar(Some(trimmed.len()))
    }

    fn is_boolean_true(&self, value: &str) -> bool {
        self.true_values
            .iter()
            .any(|v| v.eq_ignore_ascii_case(value))
    }

    fn is_boolean_false(&self, value: &str) -> bool {
        self.false_values
            .iter()
            .any(|v| v.eq_ignore_ascii_case(value))
    }

    fn is_date(&self, value: &str) -> bool {
        // First try the default pattern
        let patterns = TypePatterns::get();
        if patterns.date.is_match(value) {
            if let Ok(_) = NaiveDate::parse_from_str(value, "%Y-%m-%d") {
                return true;
            }
        }

        // Try custom format if different
        if self.date_format != "%Y-%m-%d" {
            if let Ok(_) = NaiveDate::parse_from_str(value, &self.date_format) {
                return true;
            }
        }

        false
    }

    fn is_time(&self, value: &str) -> bool {
        // First try the default pattern
        let patterns = TypePatterns::get();
        if patterns.time.is_match(value) {
            if let Ok(_) = NaiveTime::parse_from_str(value, "%H:%M:%S") {
                return true;
            }
        }

        // Try custom format if different
        if self.time_format != "%H:%M:%S" {
            if let Ok(_) = NaiveTime::parse_from_str(value, &self.time_format) {
                return true;
            }
        }

        false
    }

    fn is_datetime(&self, value: &str) -> bool {
        // First try the default pattern
        let patterns = TypePatterns::get();
        if patterns.datetime.is_match(value) {
            if let Ok(_) = NaiveDateTime::parse_from_str(value, "%Y-%m-%d %H:%M:%S") {
                return true;
            }
        }

        // Try custom format if different
        if self.datetime_format != "%Y-%m-%d %H:%M:%S" {
            if let Ok(_) = NaiveDateTime::parse_from_str(value, &self.datetime_format) {
                return true;
            }
        }

        false
    }
}

impl Default for TypeInferencer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_boolean_inference() {
        let inferencer = TypeInferencer::new();

        assert_eq!(inferencer.infer_type("true"), SqlType::Boolean);
        assert_eq!(inferencer.infer_type("TRUE"), SqlType::Boolean);
        assert_eq!(inferencer.infer_type("false"), SqlType::Boolean);
        assert_eq!(inferencer.infer_type("FALSE"), SqlType::Boolean);
        assert_eq!(inferencer.infer_type("1"), SqlType::Boolean);
        assert_eq!(inferencer.infer_type("0"), SqlType::Boolean);
        assert_eq!(inferencer.infer_type("yes"), SqlType::Boolean);
        assert_eq!(inferencer.infer_type("no"), SqlType::Boolean);
    }

    #[test]
    fn test_integer_inference() {
        let inferencer = TypeInferencer::new();

        assert_eq!(inferencer.infer_type("100"), SqlType::SmallInt);
        assert_eq!(inferencer.infer_type("32767"), SqlType::SmallInt);
        assert_eq!(inferencer.infer_type("32768"), SqlType::Integer);
        assert_eq!(inferencer.infer_type("2147483647"), SqlType::Integer);
        assert_eq!(inferencer.infer_type("2147483648"), SqlType::BigInt);
        assert_eq!(inferencer.infer_type("-32768"), SqlType::SmallInt);
        assert_eq!(inferencer.infer_type("-32769"), SqlType::Integer);
    }

    #[test]
    fn test_double_inference() {
        let inferencer = TypeInferencer::new();

        assert_eq!(inferencer.infer_type("3.14"), SqlType::DoublePrecision);
        assert_eq!(inferencer.infer_type("1.23e-4"), SqlType::DoublePrecision);
        assert_eq!(inferencer.infer_type("-2.5E+10"), SqlType::DoublePrecision);
        assert_eq!(inferencer.infer_type(".5"), SqlType::DoublePrecision);
    }

    #[test]
    fn test_date_inference() {
        let inferencer = TypeInferencer::new();

        assert_eq!(inferencer.infer_type("2023-12-25"), SqlType::Date);
        assert_eq!(inferencer.infer_type("2023-01-01"), SqlType::Date);
    }

    #[test]
    fn test_time_inference() {
        let inferencer = TypeInferencer::new();

        assert_eq!(inferencer.infer_type("14:30:00"), SqlType::Time);
        assert_eq!(inferencer.infer_type("9:15:30"), SqlType::Time);
    }

    #[test]
    fn test_datetime_inference() {
        let inferencer = TypeInferencer::new();

        assert_eq!(
            inferencer.infer_type("2023-12-25 14:30:00"),
            SqlType::DateTime
        );
        assert_eq!(
            inferencer.infer_type("2023-01-01 9:15:30"),
            SqlType::DateTime
        );
    }

    #[test]
    fn test_varchar_inference() {
        let inferencer = TypeInferencer::new();

        assert_eq!(inferencer.infer_type("hello"), SqlType::Varchar(Some(5)));
        assert_eq!(
            inferencer.infer_type("test string"),
            SqlType::Varchar(Some(11))
        );
        assert_eq!(inferencer.infer_type(""), SqlType::Varchar(Some(1)));
    }

    #[test]
    fn test_custom_boolean_values() {
        let inferencer = TypeInferencer::new().with_boolean_values(
            vec!["ON".to_string(), "ENABLED".to_string()],
            vec!["OFF".to_string(), "DISABLED".to_string()],
        );

        assert_eq!(inferencer.infer_type("ON"), SqlType::Boolean);
        assert_eq!(inferencer.infer_type("ENABLED"), SqlType::Boolean);
        assert_eq!(inferencer.infer_type("OFF"), SqlType::Boolean);
        assert_eq!(inferencer.infer_type("DISABLED"), SqlType::Boolean);

        // Default boolean values should no longer work
        assert_eq!(inferencer.infer_type("true"), SqlType::Varchar(Some(4)));
    }
}
