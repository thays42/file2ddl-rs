use crate::cli::DiagnoseArgs;
use anyhow::Result;
use csv::ReaderBuilder;
use std::collections::HashMap;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct DiagnosticError {
    pub line_number: usize,
    pub content: String,
    pub error_type: ErrorType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorType {
    FieldCountMismatch { expected: usize, actual: usize },
    QuoteError(String),
    EncodingError(String),
    LineLengthExceeded { max: usize, actual: usize },
    ParseError(String),
}

impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorType::FieldCountMismatch { expected, actual } => {
                write!(
                    f,
                    "Field count mismatch: expected {}, got {}",
                    expected, actual
                )
            }
            ErrorType::QuoteError(msg) => write!(f, "Quote error: {}", msg),
            ErrorType::EncodingError(msg) => write!(f, "Encoding error: {}", msg),
            ErrorType::LineLengthExceeded { max, actual } => {
                write!(f, "Line length exceeded: {} bytes (max {})", actual, max)
            }
            ErrorType::ParseError(msg) => write!(f, "Parse error: {}", msg),
        }
    }
}

pub struct DiagnosticSummary {
    pub total_lines: usize,
    pub problematic_lines: usize,
    pub errors_by_type: HashMap<ErrorType, Vec<DiagnosticError>>,
    pub stopped_at_limit: bool,
}

pub fn diagnose_csv<R: Read>(reader: R, args: &DiagnoseArgs) -> Result<DiagnosticSummary> {
    // Set up CSV reader with same configuration as parse command
    let mut reader_builder = ReaderBuilder::new();
    reader_builder
        .delimiter(args.delimiter as u8)
        .has_headers(!args.noheader)
        .flexible(true); // Allow variable number of fields - we'll validate manually

    // Set quote character
    if let Some(quote_byte) = args.quote.as_byte() {
        reader_builder.quote(quote_byte);
    } else {
        reader_builder.quoting(false);
    }

    // Set escape character if provided
    if let Some(esc) = args.escquote {
        reader_builder.escape(Some(esc as u8));
    }

    let mut csv_reader = reader_builder.from_reader(reader);

    let mut line_number = 0;
    let mut expected_fields: Option<usize> = args.fields;
    let mut errors_by_type: HashMap<ErrorType, Vec<DiagnosticError>> = HashMap::new();
    let mut problematic_lines = 0;
    let mut stopped_at_limit = false;

    // Get headers and determine expected field count if not specified
    if !args.noheader {
        if let Ok(headers) = csv_reader.headers() {
            line_number = 1; // Header is line 1
            if expected_fields.is_none() {
                expected_fields = Some(headers.len());
            }
        }
    }

    // Process each record
    for result in csv_reader.records() {
        line_number += 1;

        let record = match result {
            Ok(record) => record,
            Err(e) => {
                // Handle parse errors
                let error = DiagnosticError {
                    line_number,
                    content: format!("Parse error on line {}", line_number),
                    error_type: ErrorType::ParseError(e.to_string()),
                };

                errors_by_type
                    .entry(error.error_type.clone())
                    .or_default()
                    .push(error);

                problematic_lines += 1;
                if problematic_lines >= args.badmax {
                    stopped_at_limit = true;
                    break;
                }
                continue;
            }
        };

        // Auto-detect expected field count from first data record if no headers
        if expected_fields.is_none() && line_number == 1 {
            expected_fields = Some(record.len());
        }

        // Check field count if we have an expectation
        if let Some(expected) = expected_fields {
            let actual_fields = record.len();
            if actual_fields != expected {
                let raw_line = record
                    .iter()
                    .collect::<Vec<_>>()
                    .join(&args.delimiter.to_string());

                let error = DiagnosticError {
                    line_number,
                    content: raw_line,
                    error_type: ErrorType::FieldCountMismatch {
                        expected,
                        actual: actual_fields,
                    },
                };

                errors_by_type
                    .entry(error.error_type.clone())
                    .or_default()
                    .push(error);

                problematic_lines += 1;
                if problematic_lines >= args.badmax {
                    stopped_at_limit = true;
                    break;
                }
            }
        }

        // Check line length
        let raw_line = record
            .iter()
            .collect::<Vec<_>>()
            .join(&args.delimiter.to_string());
        if raw_line.len() > args.max_line_length {
            let error = DiagnosticError {
                line_number,
                content: raw_line.clone(),
                error_type: ErrorType::LineLengthExceeded {
                    max: args.max_line_length,
                    actual: raw_line.len(),
                },
            };

            errors_by_type
                .entry(error.error_type.clone())
                .or_default()
                .push(error);

            problematic_lines += 1;
            if problematic_lines >= args.badmax {
                stopped_at_limit = true;
                break;
            }
        }
    }

    Ok(DiagnosticSummary {
        total_lines: line_number,
        problematic_lines,
        errors_by_type,
        stopped_at_limit,
    })
}

pub fn print_diagnostic_summary(summary: &DiagnosticSummary) {
    println!("File Diagnosis Summary");
    println!("======================");
    println!("Total lines processed: {}", summary.total_lines);

    if summary.stopped_at_limit {
        println!(
            "Problematic lines found: {} (stopped at --badmax limit)",
            summary.problematic_lines
        );
    } else {
        println!("Problematic lines found: {}", summary.problematic_lines);
    }

    if summary.problematic_lines == 0 {
        println!("\n✓ No issues found in the CSV file.");
        return;
    }

    println!();

    // Group errors by general type first
    let mut field_count_errors: Vec<&DiagnosticError> = Vec::new();
    let mut quote_errors: Vec<&DiagnosticError> = Vec::new();
    let mut encoding_errors: Vec<&DiagnosticError> = Vec::new();
    let mut line_length_errors: Vec<&DiagnosticError> = Vec::new();
    let mut parse_errors: Vec<&DiagnosticError> = Vec::new();

    for (error_type, errors) in &summary.errors_by_type {
        match error_type {
            ErrorType::FieldCountMismatch { .. } => field_count_errors.extend(errors),
            ErrorType::QuoteError(_) => quote_errors.extend(errors),
            ErrorType::EncodingError(_) => encoding_errors.extend(errors),
            ErrorType::LineLengthExceeded { .. } => line_length_errors.extend(errors),
            ErrorType::ParseError(_) => parse_errors.extend(errors),
        }
    }

    // Display grouped errors
    if !field_count_errors.is_empty() {
        println!("Field Count Issues:");
        // Group by expected/actual field counts
        let mut count_groups: HashMap<(usize, usize), Vec<&DiagnosticError>> = HashMap::new();
        for error in field_count_errors {
            if let ErrorType::FieldCountMismatch { expected, actual } = &error.error_type {
                count_groups
                    .entry((*expected, *actual))
                    .or_default()
                    .push(error);
            }
        }

        for ((expected, actual), errors) in count_groups {
            println!(
                "- Lines with {} fields (expected {}): {} lines",
                actual,
                expected,
                errors.len()
            );
            for error in errors {
                println!(
                    "  [L{}]: {}",
                    error.line_number,
                    truncate_content(&error.content, 100)
                );
            }
        }
        println!();
    }

    if !quote_errors.is_empty() {
        println!("Quote Issues:");
        println!("- Quote violations: {} lines", quote_errors.len());
        for error in quote_errors {
            println!(
                "  [L{}]: {}",
                error.line_number,
                truncate_content(&error.content, 100)
            );
        }
        println!();
    }

    if !encoding_errors.is_empty() {
        println!("Encoding Issues:");
        println!(
            "- Invalid encoding sequences: {} lines",
            encoding_errors.len()
        );
        for error in encoding_errors {
            println!(
                "  [L{}]: {}",
                error.line_number,
                truncate_content(&error.content, 100)
            );
        }
        println!();
    }

    if !line_length_errors.is_empty() {
        println!("Line Length Issues:");
        if let Some(error) = line_length_errors.first() {
            if let ErrorType::LineLengthExceeded { max, .. } = &error.error_type {
                println!(
                    "- Lines exceeding {} bytes: {} lines",
                    max,
                    line_length_errors.len()
                );
            }
        }
        for error in line_length_errors {
            println!(
                "  [L{}]: {}",
                error.line_number,
                truncate_content(&error.content, 100)
            );
        }
        println!();
    }

    if !parse_errors.is_empty() {
        println!("Parse Errors:");
        println!("- CSV parsing errors: {} lines", parse_errors.len());
        for error in parse_errors {
            println!("  [L{}]: {}", error.line_number, error.error_type);
        }
        println!();
    }
}

fn truncate_content(content: &str, max_len: usize) -> String {
    if content.len() <= max_len {
        content.to_string()
    } else {
        format!("{}...", &content[..max_len.saturating_sub(3)])
    }
}
