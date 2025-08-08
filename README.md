# file2ddl

A high-performance CSV parser and DDL generator written in Rust.

## Phase 1 Complete ✓

### What's Implemented
- ✅ Project structure with Cargo dependencies
- ✅ CLI argument parsing using clap
- ✅ Basic streaming CSV reader with configurable delimiters
- ✅ Parse command with delimiter handling
- ✅ Unit and integration test infrastructure

### Usage

```bash
# Parse CSV from stdin
cat data.csv | file2ddl parse

# Parse CSV with custom delimiter
file2ddl parse -i data.txt -o output.csv -d '|'

# Parse with verbose output
file2ddl parse -i data.csv -v
```

### Build & Test

```bash
# Build the project
cargo build

# Run tests
cargo test

# Run the tool
cargo run -- parse -i input.csv
```

### Project Structure
```
file2ddl/
├── src/
│   ├── main.rs           # Entry point
│   ├── lib.rs           # Library root
│   ├── cli/             # CLI argument definitions
│   ├── parser/          # CSV parsing logic
│   ├── analyzer/        # Data analysis (Phase 3-4)
│   ├── ddl/            # DDL generation (Phase 4)
│   ├── types/          # Type definitions (Phase 3)
│   └── utils/          # Utilities
└── tests/
    ├── integration/     # Integration tests
    └── data/           # Test data files
```

### Next Steps (Phase 2-6)
- Phase 2: Full parse command with quote handling, NULL transformation, error handling
- Phase 3: Type inference engine
- Phase 4: Describe command & DDL generation
- Phase 5: Performance optimization
- Phase 6: Documentation & polish