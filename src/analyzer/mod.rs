pub mod column;
pub mod inference;
pub mod patterns;

use crate::cli::{DatabaseType, DescribeArgs};
use crate::types::ColumnStats;
use anyhow::Result;
use inference::StreamingInferenceEngine;
use log::{debug, info};
use std::path::Path;

pub fn describe_command(args: DescribeArgs) -> Result<()> {
    if args.verbose {
        info!("Starting describe command analysis");
        debug!("Arguments: {:?}", args);
    }

    // Prepare null values list
    let null_values = vec!["".to_string(), "NULL".to_string(), "null".to_string()];

    // Create inference engine
    let mut engine = StreamingInferenceEngine::new(
        null_values,
        args.fdate,
        args.ftime,
        args.fdatetime,
        100, // max errors
        args.verbose,
    );

    // Get delimiter and quote character
    let delimiter = args.delimiter as u8;
    let quote = args.quote.as_byte();

    // Analyze the CSV data
    let stats = if let Some(input_path) = &args.input {
        engine.analyze_csv_file(input_path.to_string_lossy().as_ref(), delimiter, quote)?
    } else {
        engine.analyze_csv_stdin(delimiter, quote)?
    };

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

fn print_analysis_output(stats: &[ColumnStats], verbose: bool) -> Result<()> {
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

        // Show sample values if verbose
        if verbose && !stat.sample_values.is_empty() {
            println!(
                "    Sample values: {}",
                stat.sample_values
                    .iter()
                    .take(5)
                    .map(|v| format!("'{}'", v))
                    .collect::<Vec<_>>()
                    .join(", ")
            );
        }

        // Show min/max if available
        if verbose {
            if let (Some(min), Some(max)) = (&stat.min_value, &stat.max_value) {
                if min != max {
                    println!("    Range: '{}' to '{}'", min, max);
                }
            }
        }
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
    if sanitized
        .chars()
        .next()
        .map_or(false, |c| c.is_ascii_digit())
    {
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
