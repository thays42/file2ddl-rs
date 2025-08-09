# Code Review Guide for file2ddl-rs

## Project Overview

**file2ddl** is a high-performance CSV parser and DDL generator written in Rust. It transforms raw CSV files into database-ready formats and generates CREATE TABLE statements for PostgreSQL, MySQL, and Netezza. The project emphasizes streaming architecture, type inference, and memory efficiency.

## File Organization & Roles

### Configuration Files
- **`Cargo.toml`** - Package manifest with dependencies and benchmark configuration
- **`Cargo.lock`** - Dependency lock file for reproducible builds
- **`Makefile`** - Build automation (if present)

### Documentation
- **`README.md`** - User-facing documentation
- **`CLAUDE.md`** - AI assistant context and project instructions
- **`design.md`** - Requirements specification and technical design
- **`docs/plan.md`** - Implementation plan and phases
- **`docs/PERFORMANCE.md`** - Performance analysis and benchmarking results
- **`database_config_schema.md`** - Database configuration format specification
- **`example_db_config.json`** - Sample database configuration

### Core Application
- **`src/main.rs`** - Application entry point with logging setup
- **`src/lib.rs`** - Library root module exposing public API
- **`src/cli/mod.rs`** - Command-line interface definition using clap

### Parser Module
- **`src/parser/mod.rs`** - Parse command implementation with encoding support
- **`src/parser/streaming.rs`** - Core streaming CSV parser with RFC 4180 compliance

### Analyzer Module
- **`src/analyzer/mod.rs`** - Describe/diagnose command implementations
- **`src/analyzer/inference.rs`** - Streaming type inference engine
- **`src/analyzer/patterns.rs`** - Type pattern matching (regex-based)
- **`src/analyzer/column.rs`** - Per-column statistics and analysis
- **`src/analyzer/optimized.rs`** - Performance-optimized analyzer
- **`src/analyzer/diagnose.rs`** - CSV file structural diagnosis

### Type System & Database Support
- **`src/types/mod.rs`** - SQL type system and column statistics
- **`src/database.rs`** - Database dialect abstraction and DDL generation

### Supporting Modules
- **`src/perf/mod.rs`** - Performance monitoring utilities
- **`src/utils/mod.rs`** - Shared utilities and helper functions

### Test Suite
- **`tests/describe_integration_tests.rs`** - Integration tests for describe command
- **`tests/diagnose_integration_tests.rs`** - Integration tests for diagnose command
- **`tests/performance_regression.rs`** - Performance regression testing
- **`tests/integration/`** - Parse command integration tests
- **`tests/data/`** - Test data files (CSV samples)

### Benchmarks
- **`benches/csv_parsing.rs`** - CSV parsing performance benchmarks
- **`benches/type_inference.rs`** - Type inference performance benchmarks  
- **`benches/memory_profile.rs`** - Memory usage profiling

## Recommended Review Order

### Phase 1: Foundation & Architecture (Start Here)
1. **`design.md`** - Understand requirements and technical constraints
2. **`CLAUDE.md`** - Review implementation phases and current status
3. **`Cargo.toml`** - Examine dependencies and build configuration
4. **`src/main.rs` & `src/lib.rs`** - Application entry points
5. **`src/cli/mod.rs`** - CLI interface and command structure

### Phase 2: Core Parsing Logic
6. **`src/parser/mod.rs`** - Parse command and encoding handling
7. **`src/parser/streaming.rs`** - Core CSV parsing implementation
8. **`tests/integration/parse_command.rs`** - Parse command test coverage

### Phase 3: Type System & Analysis
9. **`src/types/mod.rs`** - SQL type system and promotion rules
10. **`src/analyzer/patterns.rs`** - Type detection patterns
11. **`src/analyzer/inference.rs`** - Streaming inference engine
12. **`src/analyzer/column.rs`** - Column-level analysis

### Phase 4: Database Integration
13. **`src/database.rs`** - Database dialect system
14. **`database_config_schema.md`** - Custom database configuration
15. **`example_db_config.json`** - Configuration examples

### Phase 5: Commands & Features
16. **`src/analyzer/mod.rs`** - Describe command implementation
17. **`src/analyzer/diagnose.rs`** - Diagnose command functionality
18. **`tests/describe_integration_tests.rs`** - Integration test coverage

### Phase 6: Performance & Optimization
19. **`src/analyzer/optimized.rs`** - Performance optimizations
20. **`src/perf/mod.rs`** - Performance monitoring
21. **`benches/`** - All benchmark files
22. **`tests/performance_regression.rs`** - Performance validation
23. **`docs/PERFORMANCE.md`** - Performance analysis results

### Phase 7: Supporting Infrastructure
24. **`src/utils/mod.rs`** - Utilities and helpers
25. **`tests/data/`** - Test data examination
26. **`docs/plan.md`** - Implementation plan review

## Detailed File Analysis

### Core Architecture Files

#### `src/main.rs` (Lines: 7)
**Role**: Application entry point  
**Key Features**: Initializes env_logger, delegates to lib::run()  
**External References**: 
- [env_logger documentation](https://docs.rs/env_logger/)
- [anyhow error handling](https://docs.rs/anyhow/)

#### `src/lib.rs` (Lines: 23)
**Role**: Library root and command dispatcher  
**Key Features**: Module exports, command routing via clap  
**Dependencies**: clap Parser trait  
**External References**:
- [clap derive API](https://docs.rs/clap/latest/clap/_derive/index.html)

#### `src/cli/mod.rs` (Lines: 220)
**Role**: Command-line interface definition  
**Key Features**: 
- Three subcommands: Parse, Describe, Diagnose
- Comprehensive argument validation
- Database type enumeration
- Quote style handling
**External References**:
- [clap ValueEnum](https://docs.rs/clap/latest/clap/derive.ValueEnum.html)
- [RFC 4180 CSV specification](https://tools.ietf.org/html/rfc4180)

### Parser Implementation

#### `src/parser/mod.rs` (Lines: 93)
**Role**: Parse command implementation with encoding support  
**Key Features**:
- Multi-encoding support via encoding_rs
- Buffered I/O with 8KB buffers
- Custom EncodingReader for non-UTF8 files
**Complex Logic**: EncodingReader implements Read trait for on-the-fly encoding conversion  
**External References**:
- [encoding_rs documentation](https://docs.rs/encoding_rs/)
- [Rust Read trait](https://doc.rust-lang.org/std/io/trait.Read.html)

#### `src/parser/streaming.rs` (~500+ lines)
**Role**: Core streaming CSV parser  
**Key Features**:
- RFC 4180 compliance
- Streaming architecture (no full file loading)
- Quote handling and escaping
- Null value transformation
- Bad row collection and limits
**Performance Considerations**: 
- Line-by-line processing
- Configurable buffer sizes
- Memory-bounded operation
**External References**:
- [csv crate documentation](https://docs.rs/csv/)
- [RFC 4180](https://tools.ietf.org/html/rfc4180)

### Type System

#### `src/types/mod.rs` (Lines: 165)
**Role**: SQL type system and column statistics  
**Key Features**:
- SqlType enum with promotion hierarchy
- Type promotion rules and validation
- ColumnStats for statistical analysis
- Database-agnostic type representation
**Complex Logic**: Type promotion algorithm with precedence rules  
**External References**:
- [PostgreSQL data types](https://www.postgresql.org/docs/current/datatype.html)
- [MySQL data types](https://dev.mysql.com/doc/refman/8.0/en/data-types.html)

#### `src/database.rs` (Lines: 399)
**Role**: Database dialect abstraction  
**Key Features**:
- DatabaseDialect trait for extensibility
- Built-in support for PostgreSQL, MySQL, Netezza
- Configurable dialects via JSON
- Type mapping and feature detection
**Design Pattern**: Strategy pattern for database-specific behavior  
**External References**:
- [serde JSON processing](https://docs.rs/serde_json/)
- [Database-specific SQL documentation]

### Analysis Engine

#### `src/analyzer/inference.rs` (~400+ lines estimated)
**Role**: Streaming type inference engine  
**Key Features**:
- Streaming analysis (constant memory usage)
- Type promotion tracking
- Statistical collection
- Error handling and recovery
**Performance Critical**: Core bottleneck for large files  

#### `src/analyzer/patterns.rs` (~200+ lines estimated)
**Role**: Type pattern matching  
**Key Features**:
- Regex-based type detection
- Date/time format parsing
- Numeric pattern validation
- Boolean value recognition
**External References**:
- [regex crate documentation](https://docs.rs/regex/)
- [chrono date/time parsing](https://docs.rs/chrono/)

### Testing Infrastructure

#### `tests/describe_integration_tests.rs` (Lines: 50+ shown)
**Role**: End-to-end testing for describe command  
**Key Features**:
- Process-level testing via cargo run
- DDL generation validation
- Database-specific output testing
**Testing Strategy**: Black-box integration testing  

#### `benches/csv_parsing.rs` (Lines: 30+ shown)
**Role**: Performance benchmarking  
**Key Features**:
- Criterion-based benchmarking
- Throughput measurement
- Synthetic data generation
**External References**:
- [criterion benchmarking](https://docs.rs/criterion/)

## Key Implementation Patterns

### Streaming Architecture
- **Memory Efficiency**: Fixed memory usage regardless of file size
- **Iterator-based**: Lazy evaluation and processing
- **Buffer Management**: 8KB default buffers throughout

### Error Handling Strategy
- **anyhow**: Unified error handling with context
- **Graceful Degradation**: Continue processing with warnings
- **User Control**: Configurable error limits and bad row collection

### Performance Optimizations
- **Zero-Copy**: Minimize string allocations where possible
- **Regex Compilation**: Pre-compiled patterns for type detection
- **Buffered I/O**: Consistent buffer sizes for optimal throughput

### Extensibility Design
- **Trait-based**: DatabaseDialect, parsers use trait abstractions
- **Configuration-driven**: JSON-based database configurations
- **Modular**: Clear separation between parsing, analysis, and output

## Critical Review Areas

1. **Memory Safety**: Verify buffer management and streaming guarantees
2. **Performance**: Examine hot paths in type inference and parsing
3. **Error Handling**: Validate error propagation and user feedback
4. **Unicode Handling**: Review encoding support and character processing
5. **SQL Injection**: Verify DDL generation safely handles column names
6. **Configuration Validation**: Check database config parsing and validation

This guide provides a systematic approach to understanding the codebase architecture, implementation patterns, and critical functionality.