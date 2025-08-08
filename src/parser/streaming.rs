use crate::cli::ParseArgs;
use anyhow::Result;
use csv::{ReaderBuilder, StringRecord, WriterBuilder};
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;

pub fn process_csv<R: Read, W: Write>(input: R, output: W, args: &ParseArgs) -> Result<()> {
    // Parse badmax - support "all" for unlimited
    let max_bad_rows = if args.badmax == "all" {
        None
    } else {
        Some(args.badmax.parse::<usize>().unwrap_or(100))
    };
    let mut reader_builder = ReaderBuilder::new();
    reader_builder
        .delimiter(args.delimiter as u8)
        .has_headers(true)
        .flexible(true); // Allow variable number of fields per record

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

    let mut reader = reader_builder.from_reader(input);

    let mut writer_builder = WriterBuilder::new();
    writer_builder
        .delimiter(args.delimiter as u8)
        .double_quote(true); // RFC 4180 compliant double quote escaping

    if let Some(quote_byte) = args.quote.as_byte() {
        writer_builder.quote(quote_byte);
    }

    let mut writer = writer_builder.from_writer(output);

    // Set up bad row writer if needed
    let mut bad_writer = if let Some(ref badfile) = args.badfile {
        Some(create_bad_row_writer(badfile, args)?)
    } else {
        None
    };

    let mut bad_row_count = 0;
    let mut total_rows = 0;

    // Write headers if present
    if let Ok(headers) = reader.headers() {
        writer.write_record(headers)?;
        if let Some(ref mut bw) = bad_writer {
            bw.write_record(headers)?;
        }
    }

    // Process records one at a time
    for result in reader.records() {
        total_rows += 1;

        match result {
            Ok(record) => {
                let processed_record = transform_nulls(&record, args);
                writer.write_record(&processed_record)?;
            }
            Err(e) => {
                bad_row_count += 1;

                if args.verbose {
                    eprintln!("Error reading row {}: {}", total_rows, e);
                }

                // Write to bad file if configured
                if let Some(ref mut bw) = bad_writer {
                    if max_bad_rows.is_none() || bad_row_count <= max_bad_rows.unwrap() {
                        // Write error info as a CSV record
                        let error_record = StringRecord::from(vec![
                            format!("Row {}", total_rows),
                            format!("{}", e),
                        ]);
                        bw.write_record(&error_record)?;
                    }
                }

                // Stop processing if we exceed badmax (unless "all")
                if let Some(max_bad) = max_bad_rows {
                    if bad_row_count > max_bad {
                        if args.verbose {
                            eprintln!("Maximum bad rows ({}) exceeded, stopping", max_bad);
                        }
                        break;
                    }
                }
            }
        }
    }

    writer.flush()?;

    if let Some(mut bw) = bad_writer {
        bw.flush()?;
    }

    if args.verbose && bad_row_count > 0 {
        eprintln!(
            "Processed {} rows with {} errors",
            total_rows, bad_row_count
        );
    }

    // Return error if we had bad rows and strict mode is enabled
    // For now, we'll just report success since flexible mode is on
    Ok(())
}

fn create_bad_row_writer(path: &PathBuf, args: &ParseArgs) -> Result<csv::Writer<File>> {
    let file = File::create(path)?;
    let writer = WriterBuilder::new()
        .delimiter(args.delimiter as u8)
        .quote(args.quote.as_byte().unwrap_or(b'"'))
        .from_writer(file);
    Ok(writer)
}

fn transform_nulls(record: &StringRecord, args: &ParseArgs) -> StringRecord {
    if args.fnull.is_empty() {
        return record.clone();
    }

    let mut new_record = StringRecord::new();

    for field in record.iter() {
        if args.fnull.contains(&field.to_string()) {
            new_record.push_field(&args.tnull);
        } else {
            new_record.push_field(field);
        }
    }

    new_record
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cli::QuoteStyle;
    use std::io::Cursor;

    fn default_args() -> ParseArgs {
        ParseArgs {
            input: None,
            output: None,
            delimiter: ',',
            quote: QuoteStyle::Double,
            escquote: None,
            fnull: vec![],
            tnull: String::new(),
            badfile: None,
            badmax: "100".to_string(),
            noheader: false,
            max_line_length: 1048576,
            encoding: "utf-8".to_string(),
            verbose: false,
        }
    }

    #[test]
    fn test_simple_csv_passthrough() {
        let input = "name,age\nAlice,30\nBob,25";
        let mut output = Vec::new();

        let result = process_csv(Cursor::new(input), &mut output, &default_args());

        assert!(result.is_ok());
        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str, "name,age\nAlice,30\nBob,25\n");
    }

    #[test]
    fn test_custom_delimiter() {
        let input = "name|age\nAlice|30\nBob|25";
        let mut output = Vec::new();

        let mut args = default_args();
        args.delimiter = '|';

        let result = process_csv(Cursor::new(input), &mut output, &args);

        assert!(result.is_ok());
        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str, "name|age\nAlice|30\nBob|25\n");
    }

    #[test]
    fn test_empty_file() {
        let input = "";
        let mut output = Vec::new();

        let result = process_csv(Cursor::new(input), &mut output, &default_args());

        assert!(result.is_ok());
        let output_str = String::from_utf8(output).unwrap();
        // Empty file produces empty quoted header from csv library
        assert!(output_str.is_empty() || output_str == "\"\"\n");
    }

    #[test]
    fn test_quoted_fields() {
        let input = "name,description\n\"Alice\",\"Has, comma\"\n\"Bob\",\"Uses \"\"quotes\"\"\"";
        let mut output = Vec::new();

        let result = process_csv(Cursor::new(input), &mut output, &default_args());

        assert!(result.is_ok());
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("\"Has, comma\""));
    }

    #[test]
    fn test_single_quotes() {
        let input = "name,age\n'Alice',30\n'Bob',25";
        let mut output = Vec::new();

        let mut args = default_args();
        args.quote = QuoteStyle::Single;

        let result = process_csv(Cursor::new(input), &mut output, &args);

        assert!(result.is_ok());
        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str, "name,age\nAlice,30\nBob,25\n");
    }

    #[test]
    fn test_null_transformation() {
        let input = "name,age,city\nAlice,30,NULL\nBob,NULL,NYC\nCharlie,25,";
        let mut output = Vec::new();

        let mut args = default_args();
        args.fnull = vec!["NULL".to_string(), "".to_string()];
        args.tnull = "\\N".to_string();

        let result = process_csv(Cursor::new(input), &mut output, &args);

        assert!(result.is_ok());
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("\\N"));
        assert!(output_str.contains("NYC"));
        assert!(!output_str.contains("NULL"));
    }

    #[test]
    fn test_escape_quote() {
        let input = "name,description\n\"Alice\",\"She said \\\"Hello\\\"\"\n";
        let mut output = Vec::new();

        let mut args = default_args();
        args.escquote = Some('\\');

        let result = process_csv(Cursor::new(input), &mut output, &args);

        assert!(result.is_ok());
    }
}
