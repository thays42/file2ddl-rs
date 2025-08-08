# file2ddl

A high-performance CSV parser and DDL generator written in Rust that helps users prepare raw data files for loading into database tables.

## Phase 3 Complete ✅

### What's Implemented
- ✅ **Streaming CSV Parser**: Memory-efficient processing with configurable delimiters, quotes, and encodings
- ✅ **Type Inference Engine**: Intelligent detection of SQL types (BOOLEAN, SMALLINT, INTEGER, BIGINT, DOUBLE PRECISION, DATE, TIME, DATETIME, VARCHAR)
- ✅ **DDL Generation**: CREATE TABLE statements for PostgreSQL, MySQL, and Netezza
- ✅ **Statistical Analysis**: Column statistics, null detection, type promotion tracking
- ✅ **Error Handling**: Graceful processing with configurable error limits
- ✅ **Comprehensive Testing**: 33+ unit and integration tests

### Primary Commands

```bash
# Analyze CSV structure and generate table schema
cargo run -- describe -i data.csv

# Generate PostgreSQL DDL
cargo run -- describe -i data.csv --ddl

# Generate MySQL DDL with detailed analysis  
cargo run -- describe -i data.csv --ddl --database mysql -v

# Process pipe-delimited files
cargo run -- describe -i data.txt -d '|' --ddl

# Parse/clean CSV (Phase 1-2 functionality)
cargo run -- parse -i data.csv -o clean.csv
```

### Build & Test

```bash
# Build the project
cargo build

# Run all tests (33+ passing)
cargo test

# Run with debug logging
RUST_LOG=debug cargo run -- describe -i input.csv -v

# Check code quality
cargo clippy
cargo fmt -- --check
```

### Example Output

**Table Analysis:**
```
Column               Type            Nulls    Total    Null%    Max Len   
--------------------------------------------------------------------------------
id                   SMALLINT        0        5        0.0%     1         
name                 VARCHAR(7)      1        5        20.0%    7         
salary               DOUBLE PRECISION 1       5        20.0%    8         
active               BOOLEAN         0        5        0.0%     5         
created_date         DATE            1        5        20.0%    10        
```

**Generated DDL:**
```sql
CREATE TABLE data (
    id SMALLINT NOT NULL,
    name VARCHAR(7),
    salary DOUBLE PRECISION,
    active BOOLEAN NOT NULL,
    created_date DATE
);
```

### Project Structure
```
file2ddl/
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs            # Library root
│   ├── cli/              # CLI argument definitions
│   ├── parser/           # CSV parsing logic
│   ├── analyzer/         # Type inference engine
│   │   ├── mod.rs        # Describe command implementation
│   │   ├── patterns.rs   # Type pattern matching
│   │   ├── column.rs     # Per-column analysis
│   │   └── inference.rs  # Streaming inference engine
│   ├── types/            # SQL type system
│   │   └── mod.rs        # SqlType enum and ColumnStats
│   ├── ddl/              # DDL generation utilities
│   └── utils/            # Utilities
└── tests/
    ├── describe_integration_tests.rs # Integration tests
    ├── integration/      # Parse command tests
    └── data/            # Test data files
```

### Key Features

- **Streaming Architecture**: Process files of any size without loading into memory
- **Type Inference**: Smart detection of SQL data types with promotion hierarchy
- **Multi-Database Support**: Generate DDL for PostgreSQL, MySQL, Netezza
- **Statistical Analysis**: Null detection, cardinality analysis, sample values
- **Error Resilience**: Continue processing with configurable error limits
- **Configurable Formats**: Custom date/time formats, delimiters, null values

### Next Steps (Phase 5+)
- Performance benchmarking and optimization
- Additional database targets (SQL Server, Oracle)
- Advanced type detection (JSON, XML, geographic data)
- Web interface and API