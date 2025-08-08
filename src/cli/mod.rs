use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Parse(ParseArgs),
    Describe(DescribeArgs),
}

#[derive(Parser)]
pub struct ParseArgs {
    #[arg(short, long, help = "Input file path (default: stdin)")]
    pub input: Option<PathBuf>,

    #[arg(short, long, help = "Output file path (default: stdout)")]
    pub output: Option<PathBuf>,

    #[arg(short, long, default_value = ",", help = "Field delimiter")]
    pub delimiter: char,

    #[arg(
        short,
        long,
        value_enum,
        default_value = "double",
        help = "Quote character"
    )]
    pub quote: QuoteStyle,

    #[arg(long, help = "Quote escape character")]
    pub escquote: Option<char>,

    #[arg(long, help = "Values to transform FROM null")]
    pub fnull: Vec<String>,

    #[arg(long, default_value = "", help = "Value to transform TO null")]
    pub tnull: String,

    #[arg(long, help = "File to write bad rows to")]
    pub badfile: Option<PathBuf>,

    #[arg(
        long,
        default_value = "100",
        help = "Maximum bad rows to output (use 'all' for unlimited)"
    )]
    pub badmax: String,

    #[arg(long, default_value = "utf-8", help = "Input file encoding")]
    pub encoding: String,

    #[arg(short = 'H', long, help = "File does not start with column headers")]
    pub noheader: bool,

    #[arg(long, default_value = "1048576", help = "Maximum line length in bytes")]
    pub max_line_length: usize,

    #[arg(short, long, help = "Verbose output")]
    pub verbose: bool,
}

#[derive(Parser, Debug)]
pub struct DescribeArgs {
    #[arg(short, long, help = "Input file path (default: stdin)")]
    pub input: Option<PathBuf>,

    #[arg(short, long, default_value = ",", help = "Field delimiter")]
    pub delimiter: char,

    #[arg(
        short,
        long,
        value_enum,
        default_value = "double",
        help = "Quote character"
    )]
    pub quote: QuoteStyle,

    #[arg(long, help = "Quote escape character")]
    pub escquote: Option<char>,

    #[arg(long, help = "Generate DDL statement")]
    pub ddl: bool,

    #[arg(long, value_enum, default_value = "postgres", help = "Target database")]
    pub database: DatabaseType,

    #[arg(long, help = "Date format string")]
    pub fdate: Option<String>,

    #[arg(long, help = "Time format string")]
    pub ftime: Option<String>,

    #[arg(long, help = "DateTime format string")]
    pub fdatetime: Option<String>,

    #[arg(long, help = "Values to treat as NULL")]
    pub fnull: Vec<String>,

    #[arg(long, default_value = "1", help = "TRUE value for boolean detection")]
    pub ftrue: String,

    #[arg(long, default_value = "0", help = "FALSE value for boolean detection")]
    pub ffalse: String,

    #[arg(long, default_value = "utf-8", help = "Input file encoding")]
    pub encoding: String,

    #[arg(short = 'H', long, help = "File does not start with column headers")]
    pub noheader: bool,

    #[arg(long, default_value = "1048576", help = "Maximum line length in bytes")]
    pub max_line_length: usize,

    #[arg(short, long, help = "Verbose output")]
    pub verbose: bool,
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum QuoteStyle {
    Double,
    Single,
    None,
}

impl QuoteStyle {
    pub fn as_byte(&self) -> Option<u8> {
        match self {
            QuoteStyle::Double => Some(b'"'),
            QuoteStyle::Single => Some(b'\''),
            QuoteStyle::None => None,
        }
    }
}

#[derive(Clone, Copy, Debug, ValueEnum)]
pub enum DatabaseType {
    Postgres,
    Mysql,
    Netezza,
}
