# Implementation Plan for file2ddl

## Language Choice: **Rust**

### Rationale:
- **Performance**: Critical requirement per design doc - Rust provides C-level performance
- **Memory efficiency**: Zero-cost abstractions and no garbage collector align with streaming requirements
- **Safety**: Memory safety guarantees prevent buffer overflows and data races
- **Iterator patterns**: First-class support for streaming/lazy evaluation
- **CSV ecosystem**: Mature libraries like `csv` crate for RFC 4180 compliance
- **CLI tooling**: Excellent CLI framework support with `clap`

## Core Libraries

```toml
# Cargo.toml dependencies
[dependencies]
clap = { version = "4.5", features = ["derive"] }      # CLI argument parsing
csv = "1.3"                                             # RFC 4180 compliant CSV handling
serde = { version = "1.0", features = ["derive"] }     # Serialization
chrono = "0.4"                                          # Date/time parsing
encoding_rs = "0.8"                                     # Character encoding
anyhow = "1.0"                                          # Error handling
thiserror = "1.0"                                       # Custom error types
log = "0.4"                                              # Logging framework
env_logger = "0.11"                                     # Logger implementation
rayon = "1.10"                                          # Parallel processing (optional)

[dev-dependencies]
tempfile = "3.0"                                        # Test file generation
criterion = "0.5"                                       # Benchmarking
proptest = "1.0"                                        # Property-based testing
```

## Project Structure

```
file2ddl/
├── src/
│   ├── main.rs                 # Entry point, CLI setup
│   ├── lib.rs                  # Library root
│   ├── cli/
│   │   ├── mod.rs             # CLI argument definitions
│   │   ├── parse.rs           # Parse subcommand args
│   │   └── describe.rs        # Describe subcommand args
│   ├── parser/
│   │   ├── mod.rs             # Parser module
│   │   ├── streaming.rs       # Streaming CSV parser
│   │   ├── transformer.rs     # Field transformation logic
│   │   └── error_handler.rs   # Bad row handling
│   ├── analyzer/
│   │   ├── mod.rs             # Data analysis module
│   │   ├── type_inference.rs  # Type detection engine
│   │   ├── type_promoter.rs   # Type promotion logic
│   │   └── stats.rs          # Column statistics
│   ├── ddl/
│   │   ├── mod.rs             # DDL generation
│   │   ├── postgres.rs        # PostgreSQL dialect
│   │   ├── mysql.rs          # MySQL dialect
│   │   └── netezza.rs        # Netezza dialect
│   ├── types/
│   │   ├── mod.rs             # Type definitions
│   │   ├── sql_types.rs       # SQL type enum
│   │   └── config.rs          # Configuration structs
│   └── utils/
│       ├── mod.rs             # Utilities
│       └── io.rs              # I/O helpers
├── tests/
│   ├── integration/           # Integration tests
│   ├── data/                  # Test data files
│   └── benchmarks/            # Performance benchmarks
├── Cargo.toml
└── README.md
```

## Development Phases

### Phase 1: Foundation (Week 1)
- Set up project structure and dependencies
- Implement CLI argument parsing with `clap`
- Create basic streaming CSV reader
- Implement simple parse command (delimiter handling only)
- Set up test infrastructure

### Phase 2: Parse Command (Week 2-3)
- Implement quote character handling
- Add quote escaping support
- Implement NULL value transformation
- Add error handling with bad row collection
- Support multiple encodings
- Create RFC 4180 compliant output writer

### Phase 3: Type Inference Engine (Week 3-4)
- Implement type detection for each SQL type
- Create configurable type precedence system
- Implement type promotion logic
- Add date/time format parsing
- Support NULL value handling in inference

### Phase 4: Describe Command & DDL (Week 4-5)
- Implement DDL generation for PostgreSQL
- Add support for MySQL and Netezza dialects
- Create verbose logging system
- Implement column statistics collection

### Phase 5: Optimization & Testing (Week 5-6)
- Performance optimization with benchmarks
- Comprehensive test suite development
- Documentation and examples
- Error message refinement

## Key Implementation Details

### Streaming Architecture:
```rust
// Use iterators for lazy evaluation
let reader = csv::ReaderBuilder::new()
    .delimiter(config.delimiter)
    .has_headers(config.has_headers)
    .from_reader(BufReader::with_capacity(8192, input));

// Process records one at a time
for result in reader.records() {
    let record = result?;
    // Process without holding entire file
}
```

### Type Inference State Machine:
```rust
enum InferredType {
    Unknown,
    Boolean,
    SmallInt,
    Integer,
    BigInt,
    Double,
    Date,
    Time,
    DateTime,
    Varchar(usize),
}

// Track type per column, promote as needed
struct ColumnType {
    current: InferredType,
    max_length: usize,
    null_count: usize,
}
```

### Error Handling Strategy:
- Use `Result<T, E>` throughout
- Custom error types with `thiserror`
- Graceful degradation with warnings
- Detailed error context with line numbers

## Performance Considerations

### Memory Management:
- Stream processing with buffered I/O (8KB default buffer)
- Maximum line length enforcement (1MB default)
- Lazy evaluation using iterators
- No full file loading into memory

### Optimization Techniques:
- Use `BufReader` and `BufWriter` for I/O
- Pre-allocate vectors where size is known
- Use `SmallVec` for small, stack-allocated collections
- Consider `rayon` for parallel type inference on large files

## Testing Strategy

### Unit Tests:
- Test each type detector function
- Test type promotion logic
- Test delimiter and quote parsing
- Test NULL value transformation

### Integration Tests:
- Test full parse command flow
- Test full describe command flow
- Test error handling paths
- Test verbose logging output

### End-to-End Tests:
- Valid CSV to DDL generation
- Malformed data handling
- Large file processing
- Various delimiter and quote combinations

### Test Data Categories:
- Simple valid CSVs
- Complex nested quotes
- Mixed type columns requiring promotion
- Files with bad rows
- Empty files and single-line files
- Large files (generated programmatically)
- Various encodings (UTF-8, Latin-1, etc.)

## CLI Interface Examples

### Parse Command:
```bash
# Basic usage with pipe
cat input.csv | file2ddl parse > output.csv

# With file paths
file2ddl parse -i input.txt -o output.csv -d '|' -q single

# With error handling
file2ddl parse -i data.csv --badfile errors.txt --badmax 100

# NULL handling
file2ddl parse -i data.csv --fnull "NA" --fnull "N/A" --tnull "NULL"
```

### Describe Command:
```bash
# Generate DDL
file2ddl describe -i data.csv --ddl --database postgres

# With custom date formats
file2ddl describe -i data.csv --ddl --fdate "%m/%d/%Y" --ftime "%I:%M %p"

# Verbose mode for debugging
file2ddl describe -i data.csv --ddl -v
```

## Next Steps

1. Initialize Rust project with `cargo new file2ddl`
2. Set up basic CLI structure with clap
3. Implement streaming CSV reader
4. Begin with parse command implementation
5. Add comprehensive tests alongside implementation