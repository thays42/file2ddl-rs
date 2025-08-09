pub mod analyzer;
pub mod cli;
pub mod database;
pub mod parser;
pub mod perf;
pub mod types;
pub mod utils;

use anyhow::Result;

pub fn run() -> Result<()> {
    use clap::Parser;
    use cli::{Cli, Commands};

    let cli = Cli::parse();

    match cli.command {
        Commands::Parse(args) => parser::parse_command(args),
        Commands::Describe(args) => analyzer::describe_command(args),
        Commands::Diagnose(args) => analyzer::diagnose_command(args),
    }
}
