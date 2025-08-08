# file2ddl Project Overview

## Purpose
A high-performance CSV parser and DDL generator written in Rust that helps users prepare raw data files for loading into database tables.

## Current Status
**Phase 5 Complete** - Full optimization with performance benchmarking and memory profiling

## Key Commands

### Build & Test
```bash
# Build the project
cargo build

# Run tests
cargo test

# Run with verbose output for debugging
RUST_LOG=debug cargo run -- describe -i input.csv -v

# Run benchmarks
cargo bench

# Run performance regression tests
cargo test performance_regression

# Check code formatting
cargo fmt -- --check

# Run linter
cargo clippy
```

### Usage Examples
```bash
# Analyze CSV structure and types
cargo run -- describe -i data.csv

# Generate PostgreSQL DDL
cargo run -- describe -i data.csv --ddl

# Generate MySQL DDL with verbose output
cargo run -- describe -i data.csv --ddl --database mysql -v

# Parse CSV from stdin (Phase 1 command)
cat data.csv | cargo run -- parse

# Parse CSV with custom delimiter
cargo run -- parse -i tests/data/pipe_delimited.txt -o output.csv -d '|'

# Analyze pipe-delimited file
cargo run -- describe -i tests/data/pipe_delimited.txt -d '|' --ddl
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
│   ├── analyzer/        # Type inference engine with optimization
│   │   ├── mod.rs       # Main analyzer with describe command
│   │   ├── patterns.rs  # Type pattern matching
│   │   ├── column.rs    # Per-column analysis
│   │   ├── inference.rs # Streaming inference engine
│   │   └── optimized.rs # Performance-optimized analyzer
│   ├── ddl/            # DDL generation utilities
│   ├── types/          # SQL type system and column statistics
│   │   └── mod.rs       # SqlType enum and ColumnStats
│   ├── perf/           # Performance monitoring utilities
│   └── utils/          # Utilities
├── benches/            # Performance benchmarks
│   ├── csv_parsing.rs   # CSV parsing benchmarks
│   ├── type_inference.rs# Type inference benchmarks
│   └── memory_profile.rs# Memory profiling benchmarks
├── docs/               # Documentation
│   ├── PERFORMANCE.md   # Performance analysis report
│   └── plan.md         # Implementation plan
└── tests/
    ├── describe_integration_tests.rs # Integration tests for describe
    ├── integration/     # Parse command integration tests
    ├── performance_regression.rs # Performance regression tests
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

### ✅ Phase 2: Full Parse Command (Complete)
- Quote character handling (single/double)
- Quote escaping support
- NULL value transformation (--fnull, --tnull)
- Error handling with bad row collection
- Multiple encoding support
- RFC 4180 compliant output

### ✅ Phase 3: Type Inference Engine (Complete)
- Type detection for SQL types (BOOLEAN, SMALLINT, INTEGER, BIGINT, DOUBLE PRECISION, DATE, TIME, DATETIME, VARCHAR)
- Intelligent type promotion hierarchy
- Pattern-based recognition with regex
- Date/time format parsing with configurable formats
- Streaming analysis without loading entire files into memory
- Column statistics collection (null counts, sample values, min/max)
- Describe command with table output and DDL generation

### ✅ Phase 4: Describe Command & DDL (Complete)
- DDL generation for PostgreSQL, MySQL, Netezza
- Column statistics collection with null percentage
- Verbose logging for type promotions
- Configurable null value detection
- Column name sanitization for SQL compliance

### ✅ Phase 5: Performance Optimization (Complete)
- Comprehensive benchmarking suite with criterion
- Memory optimization and profiling infrastructure
- Performance regression testing framework
- Optimized analyzer with adaptive buffer sizing
- Memory usage monitoring and estimation
- Large file processing optimization (500+ MiB/s throughput)

## Test Data
- `tests/data/simple.csv` - Basic CSV file with id, name, age, active columns
- `tests/data/pipe_delimited.txt` - Pipe-delimited file
- `tests/data/type_inference.csv` - Complex types for testing inference
- `tests/data/promotions.csv` - Type promotion testing
- `tests/data/nulls.csv` - NULL value handling
- `tests/data/quoted.csv` - Quote handling testing

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
- **clap**: CLI argument parsing with derive macros
- **csv**: RFC 4180 compliant CSV handling
- **serde**: Serialization framework
- **chrono**: Date/time parsing and formatting
- **regex**: Pattern matching for type inference
- **encoding_rs**: Character encoding support
- **anyhow/thiserror**: Comprehensive error handling
- **log/env_logger**: Structured logging framework
- **tempfile**: Temporary file management for testing
- **criterion**: Performance benchmarking framework
- **proptest**: Property-based testing

## Testing Strategy
- **Unit tests**: 24+ tests for individual functions and modules
- **Integration tests**: 15+ end-to-end command tests
- **Type inference tests**: Comprehensive coverage of all SQL types
- **Pattern matching tests**: Boolean, numeric, date/time validation
- **Error handling tests**: NULL values, type promotions, malformed data
- **Multi-database tests**: PostgreSQL, MySQL, Netezza DDL generation
- **Performance benchmarks**: CSV parsing, type inference, memory profiling
- **Regression tests**: Automated performance threshold validation

## Current Capabilities
- ✅ **Parse Command**: Stream CSV with configurable delimiters, quotes, null handling
- ✅ **Describe Command**: Full type inference with statistical analysis
- ✅ **DDL Generation**: CREATE TABLE statements for major databases
- ✅ **Type System**: Complete SQL type hierarchy with intelligent promotions
- ✅ **Streaming Architecture**: Memory-efficient processing of large files
- ✅ **Error Resilience**: Graceful handling of malformed data with limits
- ✅ **Performance Optimization**: 500+ MiB/s throughput with adaptive optimization
- ✅ **Memory Efficiency**: Constant memory usage regardless of file size
- ✅ **Benchmarking Suite**: Comprehensive performance validation and monitoring
- ✅ **Regression Testing**: Automated performance threshold enforcement