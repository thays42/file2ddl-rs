pub mod streaming;

use crate::cli::ParseArgs;
use anyhow::Result;
use std::io::{BufReader, BufWriter, Read, Write};

pub fn parse_command(args: ParseArgs) -> Result<()> {
    let input: Box<dyn Read> = match &args.input {
        Some(path) => Box::new(std::fs::File::open(path)?),
        None => Box::new(std::io::stdin()),
    };
    
    let output: Box<dyn Write> = match &args.output {
        Some(path) => Box::new(std::fs::File::create(path)?),
        None => Box::new(std::io::stdout()),
    };
    
    let reader = BufReader::with_capacity(8192, input);
    let writer = BufWriter::with_capacity(8192, output);
    
    streaming::process_csv(reader, writer, &args)?;
    
    Ok(())
}