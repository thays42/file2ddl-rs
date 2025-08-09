pub mod column;
pub mod inference;
pub mod optimized;
pub mod patterns;

use crate::cli::{DatabaseType, DescribeArgs, ParseArgs};
use crate::parser::ParsedCsvReader;
use crate::types::ColumnStats;
use anyhow::{Context, Result};
use encoding_rs::Encoding;
use inference::StreamingInferenceEngine;
use log::{debug, info};
use std::fs::File;
use std::io::Read;
use std::path::Path;

pub fn describe_command(args: DescribeArgs) -> Result<()> {
    if args.verbose {
        info!("Starting describe command analysis");
        debug!("Arguments: {:?}", args);
    }

    // Convert DescribeArgs to ParseArgs to leverage parse command logic
    let parse_args = convert_describe_to_parse_args(&args);

    // Prepare null values list - use provided fnull or defaults
    let null_values = if args.fnull.is_empty() {
        vec!["".to_string(), "NULL".to_string(), "null".to_string()]
    } else {
        args.fnull.clone()
    };

    // Create inference engine
    let mut engine = StreamingInferenceEngine::new(
        null_values,
        args.fdate,
        args.ftime,
        args.fdatetime,
        0, // max errors - fail on first error like parse command
        args.verbose,
        args.sub_newline.clone(),
    );

    // Create input reader with encoding support (like parse command)
    let input: Box<dyn Read> = match &args.input {
        Some(path) => Box::new(File::open(path)?),
        None => Box::new(std::io::stdin()),
    };

    // Handle encoding (same as parse command)
    let encoding = Encoding::for_label(parse_args.encoding.as_bytes())
        .with_context(|| format!("Unsupported encoding: {}", parse_args.encoding))?;

    let reader: Box<dyn Read> = if encoding == encoding_rs::UTF_8 {
        input
    } else {
        // For non-UTF8 encodings, we need to decode first
        let decoded_reader = crate::parser::EncodingReader::new(input, encoding);
        Box::new(decoded_reader)
    };

    // Create ParsedCsvReader that will apply all parse command transformations
    let parsed_reader = ParsedCsvReader::new(reader, parse_args)?;

    // Analyze using the parsed reader
    let stats = engine.analyze_with_parsed_reader(parsed_reader)?;

    // Print type promotions if verbose
    if args.verbose {
        engine.print_type_promotions();
    }

    // Display results
    if args.ddl {
        print_ddl_output(&stats, &args.database, args.input.as_deref())?;
    } else {
        print_analysis_output(&stats, args.verbose)?;
    }

    let summary = engine.get_summary();
    if args.verbose {
        info!(
            "Analysis summary: {} rows, {} columns, {:.1}% success rate",
            summary.total_rows,
            summary.total_columns,
            summary.success_rate()
        );
    }

    Ok(())
}

/// Convert DescribeArgs to ParseArgs to reuse parse command logic
fn convert_describe_to_parse_args(args: &DescribeArgs) -> ParseArgs {
    ParseArgs {
        input: args.input.clone(),
        output: None, // describe doesn't write output files
        delimiter: args.delimiter,
        quote: args.quote.clone(),
        escquote: args.escquote,
        fnull: args.fnull.clone(),
        tnull: String::new(), // describe analyzes original null values
        badfile: None, // describe doesn't write bad files
        badmax: "0".to_string(), // describe fails on first error like original
        noheader: false, // describe always expects headers
        max_line_length: 1048576, // default from parse command
        encoding: "utf-8".to_string(), // default encoding
        verbose: args.verbose,
        sub_newline: args.sub_newline.clone(),
    }
}

fn print_analysis_output(stats: &[ColumnStats], _verbose: bool) -> Result<()> {
    // Print table header
    println!(
        "{:<20} {:<15} {:<8} {:<8} {:<8} {:<10}",
        "Column", "Type", "Nulls", "Total", "Null%", "Max Len"
    );
    println!("{}", "-".repeat(80));

    // Print each column
    for stat in stats {
        let null_pct = if stat.total_count > 0 {
            format!("{:.1}%", stat.null_percentage())
        } else {
            "0.0%".to_string()
        };

        println!(
            "{:<20} {:<15} {:<8} {:<8} {:<8} {:<10}",
            truncate_string(&stat.name, 20),
            truncate_string(&stat.sql_type.to_string(), 15),
            stat.null_count,
            stat.total_count,
            null_pct,
            stat.max_length
        );
    }

    Ok(())
}

fn print_ddl_output(
    stats: &[ColumnStats],
    database: &DatabaseType,
    input_path: Option<&Path>,
) -> Result<()> {
    // Generate table name from input file or use default
    let table_name = if let Some(path) = input_path {
        path.file_stem()
            .and_then(|name| name.to_str())
            .unwrap_or("imported_table")
            .replace(" ", "_")
            .replace("-", "_")
    } else {
        "imported_table".to_string()
    };

    // Print CREATE TABLE statement
    match database {
        DatabaseType::Postgres => print_postgres_ddl(&table_name, stats)?,
        DatabaseType::Mysql => print_mysql_ddl(&table_name, stats)?,
        DatabaseType::Netezza => print_netezza_ddl(&table_name, stats)?,
    }

    Ok(())
}

fn print_postgres_ddl(table_name: &str, stats: &[ColumnStats]) -> Result<()> {
    println!("CREATE TABLE {} (", table_name);

    for (i, stat) in stats.iter().enumerate() {
        let column_name = sanitize_column_name(&stat.name);
        let data_type = stat.sql_type.to_postgres_ddl();
        let nullable = if stat.is_nullable() { "" } else { " NOT NULL" };
        let comma = if i == stats.len() - 1 { "" } else { "," };

        println!("    {} {}{}{}", column_name, data_type, nullable, comma);
    }

    println!(");");
    Ok(())
}

fn print_mysql_ddl(table_name: &str, stats: &[ColumnStats]) -> Result<()> {
    println!("CREATE TABLE {} (", table_name);

    for (i, stat) in stats.iter().enumerate() {
        let column_name = sanitize_column_name(&stat.name);
        let data_type = stat.sql_type.to_mysql_ddl();
        let nullable = if stat.is_nullable() { "" } else { " NOT NULL" };
        let comma = if i == stats.len() - 1 { "" } else { "," };

        println!("    {} {}{}{}", column_name, data_type, nullable, comma);
    }

    println!(");");
    Ok(())
}

fn print_netezza_ddl(table_name: &str, stats: &[ColumnStats]) -> Result<()> {
    println!("CREATE TABLE {} (", table_name);

    for (i, stat) in stats.iter().enumerate() {
        let column_name = sanitize_column_name(&stat.name);
        let data_type = stat.sql_type.to_netezza_ddl();
        let nullable = if stat.is_nullable() { "" } else { " NOT NULL" };
        let comma = if i == stats.len() - 1 { "" } else { "," };

        println!("    {} {}{}{}", column_name, data_type, nullable, comma);
    }

    println!(");");
    Ok(())
}

fn sanitize_column_name(name: &str) -> String {
    // Replace spaces and special characters with underscores
    let sanitized = name
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();

    // Ensure it starts with a letter or underscore
    if sanitized.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        format!("_{}", sanitized)
    } else {
        sanitized
    }
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len.saturating_sub(3)])
    }
}
