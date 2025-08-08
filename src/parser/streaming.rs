use crate::cli::ParseArgs;
use anyhow::Result;
use csv::{ReaderBuilder, WriterBuilder};
use std::io::{Read, Write};

pub fn process_csv<R: Read, W: Write>(
    input: R,
    output: W,
    args: &ParseArgs,
) -> Result<()> {
    let mut reader = ReaderBuilder::new()
        .delimiter(args.delimiter as u8)
        .quote(args.quote.as_byte().unwrap_or(b'"'))
        .has_headers(true)
        .from_reader(input);
    
    let mut writer = WriterBuilder::new()
        .delimiter(args.delimiter as u8)
        .quote(args.quote.as_byte().unwrap_or(b'"'))
        .from_writer(output);
    
    // Write headers if present
    if let Ok(headers) = reader.headers() {
        writer.write_record(headers)?;
    }
    
    // Process records one at a time
    for result in reader.records() {
        match result {
            Ok(record) => {
                // For Phase 1, just pass through with delimiter handling
                writer.write_record(&record)?;
            }
            Err(e) => {
                if args.verbose {
                    eprintln!("Error reading record: {}", e);
                }
                // In Phase 2, we'll implement bad row handling
            }
        }
    }
    
    writer.flush()?;
    Ok(())
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
            badmax: 100,
            verbose: false,
        }
    }
    
    #[test]
    fn test_simple_csv_passthrough() {
        let input = "name,age\nAlice,30\nBob,25";
        let mut output = Vec::new();
        
        let result = process_csv(
            Cursor::new(input),
            &mut output,
            &default_args()
        );
        
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
        
        let result = process_csv(
            Cursor::new(input),
            &mut output,
            &args
        );
        
        assert!(result.is_ok());
        let output_str = String::from_utf8(output).unwrap();
        assert_eq!(output_str, "name|age\nAlice|30\nBob|25\n");
    }
    
    #[test]
    fn test_empty_file() {
        let input = "";
        let mut output = Vec::new();
        
        let result = process_csv(
            Cursor::new(input),
            &mut output,
            &default_args()
        );
        
        assert!(result.is_ok());
        let output_str = String::from_utf8(output).unwrap();
        // Empty file produces empty quoted header from csv library
        assert!(output_str == "" || output_str == "\"\"\n");
    }
}