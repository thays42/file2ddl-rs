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
        Some(args.badmax.parse::<usize>().unwrap_or(0))
    };
    let mut reader_builder = ReaderBuilder::new();
    reader_builder
        .delimiter(args.delimiter as u8)
        .has_headers(true)
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
    let mut expected_field_count = None;

    // Write headers if present and track expected field count
    if let Ok(headers) = reader.headers() {
        expected_field_count = Some(headers.len());
        writer.write_record(headers)?;
        if let Some(ref mut bw) = bad_writer {
            // Write a different header for bad file to avoid field count mismatch
            let bad_headers = StringRecord::from(vec!["Row", "Error"]);
            bw.write_record(&bad_headers)?;
        }
    }

    // Process records one at a time
    for result in reader.records() {
        total_rows += 1;

        match result {
            Ok(record) => {
                // Check field count consistency
                if let Some(expected) = expected_field_count {
                    if record.len() != expected {
                        bad_row_count += 1;
                        
                        // Create user-friendly error message
                        let error_msg = format!(
                            "Line {} has {} fields, but expected {} fields",
                            total_rows + 1, // +1 because we count header as row 1
                            record.len(),
                            expected
                        );
                        
                        eprintln!("{}", error_msg);
                        
                        if args.verbose {
                            eprintln!("Row content: {:?}", record.iter().collect::<Vec<_>>());
                        }

                        // Write to bad file if configured
                        if let Some(ref mut bw) = bad_writer {
                            if max_bad_rows.is_none() || bad_row_count <= max_bad_rows.unwrap() {
                                let error_record = StringRecord::from(vec![
                                    format!("Row {}", total_rows + 1),
                                    error_msg,
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
                        continue; // Skip processing this record
                    }
                }
                
                let null_transformed = transform_nulls(&record, args);
                let processed_record = substitute_newlines(&null_transformed, args);
                writer.write_record(&processed_record)?;
            }
            Err(e) => {
                bad_row_count += 1;

                if args.verbose {
                    eprintln!("Error reading row {}: {}", total_rows + 1, e);
                }

                // Write to bad file if configured
                if let Some(ref mut bw) = bad_writer {
                    if max_bad_rows.is_none() || bad_row_count <= max_bad_rows.unwrap() {
                        // Write error info as a CSV record
                        let error_record = StringRecord::from(vec![
                            format!("Row {}", total_rows + 1),
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

    // Return error if we had bad rows - parsing should fail with non-zero exit code
    if bad_row_count > 0 {
        anyhow::bail!("Parsing failed with {} error(s)", bad_row_count);
    }

    Ok(())
}

fn create_bad_row_writer(path: &PathBuf, args: &ParseArgs) -> Result<csv::Writer<File>> {
    let file = File::create(path)?;
    let mut writer_builder = WriterBuilder::new();
    writer_builder
        .delimiter(args.delimiter as u8)
        .flexible(true); // Allow variable field counts for error records
    
    if let Some(quote_byte) = args.quote.as_byte() {
        writer_builder.quote(quote_byte);
    }
    
    let writer = writer_builder.from_writer(file);
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

fn substitute_newlines(record: &StringRecord, args: &ParseArgs) -> StringRecord {
    let mut new_record = StringRecord::new();

    for field in record.iter() {
        let field_with_subs = field.replace('\n', &args.sub_newline).replace('\r', "");
        new_record.push_field(&field_with_subs);
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
            badmax: "0".to_string(),
            noheader: false,
            max_line_length: 1048576,
            encoding: "utf-8".to_string(),
            verbose: false,
            sub_newline: " ".to_string(),
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

    #[test]
    fn test_intrafield_newlines_default() {
        let input = "name,description,age\n\"Alice\",\"Has, comma\",30\n\"Bob\",\"Uses \"\"quotes\"\"\",25\n\"Charlie\",\"New \nline\",35";
        let mut output = Vec::new();

        let result = process_csv(Cursor::new(input), &mut output, &default_args());

        assert!(result.is_ok());
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("New  line")); // Two spaces because original has "New \nline"
        assert!(!output_str.contains("New \n"));
    }

    #[test]
    fn test_intrafield_newlines_custom_substitute() {
        let input = "name,description\n\"Alice\",\"Line1\nLine2\nLine3\"";
        let mut output = Vec::new();

        let mut args = default_args();
        args.sub_newline = " | ".to_string();

        let result = process_csv(Cursor::new(input), &mut output, &args);

        assert!(result.is_ok());
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Line1 | Line2 | Line3"));
    }

    #[test]
    fn test_carriage_return_removal() {
        let input = "name,description\n\"Alice\",\"Line1\r\nLine2\"";
        let mut output = Vec::new();

        let result = process_csv(Cursor::new(input), &mut output, &default_args());

        assert!(result.is_ok());
        let output_str = String::from_utf8(output).unwrap();
        assert!(output_str.contains("Line1 Line2"));
        assert!(!output_str.contains('\r'));
    }
}
