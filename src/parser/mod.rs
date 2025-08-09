pub mod streaming;

use crate::cli::ParseArgs;
use anyhow::{Context, Result};
use encoding_rs::Encoding;
use std::io::{BufReader, BufWriter, Read, Write};
pub use streaming::ParsedCsvReader;

pub fn parse_command(args: ParseArgs) -> Result<()> {
    let input: Box<dyn Read> = match &args.input {
        Some(path) => Box::new(std::fs::File::open(path)?),
        None => Box::new(std::io::stdin()),
    };

    let output: Box<dyn Write> = match &args.output {
        Some(path) => Box::new(std::fs::File::create(path)?),
        None => Box::new(std::io::stdout()),
    };

    // Handle encoding
    let encoding = Encoding::for_label(args.encoding.as_bytes())
        .with_context(|| format!("Unsupported encoding: {}", args.encoding))?;

    let reader = if encoding == encoding_rs::UTF_8 {
        BufReader::with_capacity(8192, input)
    } else {
        // For non-UTF8 encodings, we need to decode first
        let decoded_reader = EncodingReader::new(input, encoding);
        BufReader::with_capacity(8192, Box::new(decoded_reader) as Box<dyn Read>)
    };

    let writer = BufWriter::with_capacity(8192, output);

    streaming::process_csv(reader, writer, &args)?;

    Ok(())
}

// Custom reader that handles encoding conversion
pub struct EncodingReader {
    inner: Box<dyn Read>,
    encoding: &'static Encoding,
    buffer: Vec<u8>,
    decoded: String,
    position: usize,
}

impl EncodingReader {
    pub fn new(reader: Box<dyn Read>, encoding: &'static Encoding) -> Self {
        Self {
            inner: reader,
            encoding,
            buffer: vec![0; 8192],
            decoded: String::new(),
            position: 0,
        }
    }
}

impl Read for EncodingReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // If we have decoded data, return it
        if self.position < self.decoded.len() {
            let bytes = self.decoded.as_bytes();
            let available = &bytes[self.position..];
            let to_copy = std::cmp::min(available.len(), buf.len());
            buf[..to_copy].copy_from_slice(&available[..to_copy]);
            self.position += to_copy;
            return Ok(to_copy);
        }

        // Read more data and decode
        self.decoded.clear();
        self.position = 0;

        let bytes_read = self.inner.read(&mut self.buffer)?;
        if bytes_read == 0 {
            return Ok(0);
        }

        let (cow, _, _) = self.encoding.decode(&self.buffer[..bytes_read]);
        self.decoded = cow.into_owned();

        // Now return data from the decoded string
        let bytes = self.decoded.as_bytes();
        let to_copy = std::cmp::min(bytes.len(), buf.len());
        buf[..to_copy].copy_from_slice(&bytes[..to_copy]);
        self.position = to_copy;

        Ok(to_copy)
    }
}
