# file2ddl Project Overview

## Purpose
A high-performance CSV parser and DDL generator written in Rust that helps users prepare raw data files for loading into database tables.

## Current Status
**Phase 1 Complete** - Basic streaming CSV parser with configurable delimiters

## Key Commands

### Build & Test
```bash
# Build the project
cargo build

# Run tests
cargo test

# Run with verbose output for debugging
RUST_LOG=debug cargo run -- parse -i input.csv -v

# Run benchmarks
cargo bench

# Check code formatting
cargo fmt -- --check

# Run linter
cargo clippy
```

### Usage Examples
```bash
# Parse CSV from stdin
cat data.csv | cargo run -- parse

# Parse CSV with custom delimiter
cargo run -- parse -i tests/data/pipe_delimited.txt -o output.csv -d '|'

# Parse with verbose output
cargo run -- parse -i tests/data/simple.csv -v
```

## Project Structure
```
.                        # Project root
├── Cargo.toml          # Package manifest
├── Cargo.lock          # Dependency lock file
├── README.md           # Project documentation
├── CLAUDE.md           # AI assistant context
├── design.md           # Design requirements
├── plan.md             # Implementation plan
├── src/
│   ├── main.rs          # Entry point
│   ├── lib.rs           # Library root
│   ├── cli/             # CLI argument parsing (clap)
│   ├── parser/          # CSV parsing logic
│   │   ├── mod.rs       # Main parser module
│   │   └── streaming.rs # Streaming CSV implementation
│   ├── analyzer/        # Type inference (Phase 3-4)
│   ├── ddl/            # DDL generation (Phase 4)
│   ├── types/          # Type definitions (Phase 3)
│   └── utils/          # Utilities
└── tests/
    ├── integration/     # Integration tests
    └── data/           # Test data files
```

## Key Design Principles

### Performance & Memory
- **Streaming architecture**: Process files line-by-line, never load entire file into memory
- **Buffer size**: 8KB default for I/O operations
- **Max line length**: 1MB default (configurable with --max-line-length)
- **Iterator patterns**: Use Rust's lazy evaluation for efficiency

### Error Handling
- Graceful degradation with warnings
- Bad row collection with --badfile option
- Continue processing with --badmax option
- Non-zero exit on errors

## Development Phases

### ✅ Phase 1: Foundation (Complete)
- Project structure with Cargo dependencies
- CLI argument parsing using clap
- Basic streaming CSV reader
- Parse command with delimiter handling
- Test infrastructure

### 🚧 Phase 2: Full Parse Command (Next)
- Quote character handling (single/double)
- Quote escaping support
- NULL value transformation (--fnull, --tnull)
- Error handling with bad row collection
- Multiple encoding support
- RFC 4180 compliant output

### 📋 Phase 3: Type Inference Engine
- Type detection for SQL types (BOOLEAN, INTEGER, DATE, etc.)
- Configurable type precedence
- Type promotion logic
- Date/time format parsing

### 📋 Phase 4: Describe Command & DDL
- DDL generation for PostgreSQL, MySQL, Netezza
- Column statistics collection
- Verbose logging for type promotions

### 📋 Phase 5: Optimization
- Performance benchmarks
- Comprehensive test suite
- Documentation

## Test Data
- `tests/data/simple.csv` - Basic CSV file
- `tests/data/pipe_delimited.txt` - Pipe-delimited file

## Important Implementation Notes

### Type Inference Hierarchy
```
BOOLEAN -> SMALLINT -> INTEGER -> BIGINT -> DOUBLE PRECISION -> VARCHAR
DATE -> VARCHAR
TIME -> VARCHAR
DATETIME -> VARCHAR
```

### Supported SQL Types
- BOOLEAN (configurable with --ftrue/--ffalse)
- SMALLINT (-32,768 to 32,767)
- INTEGER (-2,147,483,648 to 2,147,483,647)
- BIGINT (-9,223,372,036,854,775,808 to 9,223,372,036,854,775,807)
- DOUBLE PRECISION
- DATE (%Y-%m-%d default)
- TIME (%H:%M:%S default)
- DATETIME (%Y-%m-%d %H:%M:%S default)
- VARCHAR(n)

## Dependencies
- **clap**: CLI argument parsing
- **csv**: RFC 4180 compliant CSV handling
- **serde**: Serialization
- **chrono**: Date/time parsing
- **encoding_rs**: Character encoding support
- **anyhow/thiserror**: Error handling
- **log/env_logger**: Logging framework

## Testing Strategy
- Unit tests for individual functions
- Integration tests for command flows
- Property-based testing with proptest
- Benchmarks with criterion
- Test various delimiters, quotes, encodings, and error conditions