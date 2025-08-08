# file2ddl Makefile
# Convenient targets for building, testing, and running the CLI

.PHONY: help build test clean fmt lint check run-simple run-pipe run-quoted run-nulls run-bad run-comprehensive run-describe run-describe-ddl run-describe-mysql run-describe-netezza run-describe-types run-describe-verbose run-describe-nulls run-describe-pipe test-new-features test-noheader test-escquote test-boolean-detection test-max-line-length test-badmax-all test-combined-features

# Default target - show help
help:
	@echo "file2ddl - CSV Parser and DDL Generator"
	@echo ""
	@echo "Build & Development:"
	@echo "  make build          - Build the project in debug mode"
	@echo "  make release        - Build the project in release mode"
	@echo "  make test           - Run all tests"
	@echo "  make bench          - Run benchmarks"
	@echo "  make check          - Run all checks (test, fmt, clippy)"
	@echo "  make fmt            - Format code"
	@echo "  make lint           - Run clippy linter"
	@echo "  make clean          - Clean build artifacts"
	@echo ""
	@echo "Parse Command Examples:"
	@echo "  make run-simple     - Parse simple CSV file"
	@echo "  make run-pipe       - Parse pipe-delimited file"
	@echo "  make run-quoted     - Parse CSV with quoted fields"
	@echo "  make run-nulls      - Parse CSV with NULL transformation"
	@echo "  make run-bad        - Parse CSV with bad row handling"
	@echo "  make run-single     - Parse CSV with single quotes"
	@echo "  make run-escape     - Parse CSV with escape quotes"
	@echo "  make run-encoding   - Parse CSV with different encoding"
	@echo "  make run-comprehensive - Parse comprehensive test file"
	@echo ""
	@echo "Describe Command Examples:"
	@echo "  make run-describe       - Basic type analysis of CSV"
	@echo "  make run-describe-ddl   - Generate PostgreSQL DDL"
	@echo "  make run-describe-mysql - Generate MySQL DDL"
	@echo "  make run-describe-netezza - Generate Netezza DDL"
	@echo "  make run-describe-types - Test type inference patterns"
	@echo "  make run-describe-verbose - Type analysis with debug logging"
	@echo "  make run-describe-nulls - Describe with custom NULL handling"
	@echo ""
	@echo "New Feature Tests (Design.md Compliance):"
	@echo "  make test-new-features  - Test all new CLI flags"
	@echo "  make test-noheader      - Test --noheader flag"
	@echo "  make test-escquote      - Test --escquote flag"
	@echo "  make test-boolean-detection - Test --ftrue/--ffalse flags"
	@echo "  make test-max-line-length - Test --max-line-length flag"
	@echo "  make test-badmax-all    - Test --badmax 'all' functionality"
	@echo "  make test-combined-features - Test multiple features together"
	@echo ""
	@echo "Interactive Examples:"
	@echo "  make demo-stdin     - Demo reading from stdin"
	@echo "  make demo-transform - Demo NULL transformation pipeline"
	@echo ""
	@echo "Debugging:"
	@echo "  make debug-simple   - Run with debug logging enabled"
	@echo ""
	@echo "Enhanced Examples (using new features):"
	@echo "  make run-describe-enhanced - Describe with all new flags"
	@echo "  make run-parse-enhanced    - Parse with all new flags"

# Build targets
build:
	cargo build

release:
	cargo build --release

# Testing targets
test:
	cargo test

bench:
	cargo bench

# Code quality targets
fmt:
	cargo fmt

lint:
	cargo clippy

check: test fmt lint
	@echo "✅ All checks passed!"

clean:
	cargo clean
	rm -f tests/data/bad_output.csv
	rm -f tests/data/output.csv

# Basic parse examples
run-simple:
	@echo "=== Parsing simple CSV ==="
	@echo "📁 Input file (tests/data/simple.csv):"
	@cat tests/data/simple.csv
	@echo ""
	@echo "🔄 Processing..."
	cargo run -- parse -i tests/data/simple.csv -v

run-pipe:
	@echo "=== Parsing pipe-delimited file ==="
	@echo "📁 Input file (tests/data/pipe_delimited.txt):"
	@cat tests/data/pipe_delimited.txt
	@echo ""
	@echo "🔄 Processing with delimiter '|'..."
	cargo run -- parse -i tests/data/pipe_delimited.txt -d '|' -v

run-quoted:
	@echo "=== Parsing CSV with quoted fields ==="
	@echo "📁 Input file (tests/data/quoted.csv):"
	@cat tests/data/quoted.csv
	@echo ""
	@echo "🔄 Processing..."
	cargo run -- parse -i tests/data/quoted.csv -v

# NULL transformation examples
run-nulls:
	@echo "=== Parsing with NULL transformation ==="
	@echo "📁 Input file (tests/data/nulls.csv):"
	@cat tests/data/nulls.csv
	@echo ""
	@echo "🔄 Transforming 'NULL' and empty strings to '\\N'..."
	cargo run -- parse -i tests/data/nulls.csv --fnull NULL --fnull "" --tnull "\\N" -v

# Error handling examples
run-bad:
	@echo "=== Parsing with bad row handling ==="
	@echo "📁 Input file (tests/data/bad_rows.csv):"
	@cat tests/data/bad_rows.csv
	@echo ""
	@echo "🔄 Processing with bad row handling..."
	cargo run -- parse -i tests/data/bad_rows.csv --badfile tests/data/bad_output.csv --badmax 10 -v || true
	@echo ""
	@echo "📝 Bad rows written to tests/data/bad_output.csv:"
	@cat tests/data/bad_output.csv 2>/dev/null || echo "No bad rows file generated"

# Quote style examples
run-single:
	@echo "=== Parsing with single quotes ==="
	@echo "📝 Creating test file with single quotes..."
	@echo "'name','age','city'" > /tmp/single_quoted.csv
	@echo "'Alice','30','New York'" >> /tmp/single_quoted.csv
	@echo "'Bob','25','Los Angeles'" >> /tmp/single_quoted.csv
	@echo "📁 Input file (/tmp/single_quoted.csv):"
	@cat /tmp/single_quoted.csv
	@echo ""
	@echo "🔄 Processing with single quote style..."
	cargo run -- parse -i /tmp/single_quoted.csv --quote single -v

run-escape:
	@echo "=== Parsing with escape quotes ==="
	@echo "📝 Creating test file with escaped quotes..."
	@echo 'name,description' > /tmp/escaped.csv
	@echo '"Alice","She said \"Hello\""' >> /tmp/escaped.csv
	@echo '"Bob","Path is C:\\Users\\Bob"' >> /tmp/escaped.csv
	@echo "📁 Input file (/tmp/escaped.csv):"
	@cat /tmp/escaped.csv
	@echo ""
	@echo "🔄 Processing with escape quotes..."
	cargo run -- parse -i /tmp/escaped.csv --escquote "\\" -v

# Encoding examples
run-encoding:
	@echo "=== Parsing with UTF-8 encoding (testing non-ASCII characters) ==="
	@echo "📝 Creating test file with UTF-8 characters..."
	@echo "name,city,note" > /tmp/utf8_test.csv
	@echo "José,São Paulo,Café" >> /tmp/utf8_test.csv
	@echo "François,Paris,Château" >> /tmp/utf8_test.csv
	@echo "李明,北京,中文" >> /tmp/utf8_test.csv
	@echo "📁 Input file (/tmp/utf8_test.csv):"
	@cat /tmp/utf8_test.csv
	@echo ""
	@echo "🔄 Processing with UTF-8 encoding..."
	cargo run -- parse -i /tmp/utf8_test.csv --encoding utf-8 -v

# Comprehensive example
run-comprehensive:
	@echo "=== Comprehensive parse example ==="
	@echo "📁 Input file (tests/data/comprehensive.csv):"
	@cat tests/data/comprehensive.csv
	@echo ""
	@echo "🔄 Processing with: quotes, NULL transformation, bad row handling..."
	cargo run -- parse -i tests/data/comprehensive.csv \
		--fnull NULL --fnull "" \
		--tnull "\\N" \
		--badfile tests/data/comprehensive_bad.csv \
		--badmax 100 \
		-v

# Interactive demos
demo-stdin:
	@echo "=== Demo: Reading from stdin ==="
	@echo "📝 Input data (piped through stdin):"
	@echo "name,age,city"
	@echo "Alice,30,NYC"
	@echo "Bob,25,LA"
	@echo ""
	@echo "🔄 Processing through stdin..."
	@echo -e "name,age,city\nAlice,30,NYC\nBob,25,LA" | cargo run -- parse -v

demo-transform:
	@echo "=== Demo: Transformation pipeline ==="
	@echo "📝 Input data (with NULLs and empty values):"
	@echo "id,value,status"
	@echo "1,100,active"
	@echo "2,NULL,inactive"
	@echo "3,,pending"
	@echo ""
	@echo "🔄 Processing with NULL transformation..."
	@echo -e "id,value,status\n1,100,active\n2,NULL,inactive\n3,,pending" | \
		cargo run -- parse --fnull NULL --fnull "" --tnull "\\N" -v

demo-pipeline:
	@echo "=== Demo: Multi-stage pipeline ==="
	@echo "📝 Stage 1: Generate CSV with pipe delimiters"
	@echo -e "name|age|status\nAlice|30|NULL\nBob|25|active" > /tmp/pipeline.txt
	@echo "📁 Generated file:"
	@cat /tmp/pipeline.txt
	@echo ""
	@echo "🔄 Stage 2: Parse and transform (pipe delimiter, NULL transformation)"
	@cargo run -- parse -i /tmp/pipeline.txt -d '|' --fnull NULL --tnull "MISSING" -o /tmp/pipeline_out.csv
	@echo ""
	@echo "📋 Stage 3: Final result:"
	@cat /tmp/pipeline_out.csv

# Describe command examples
run-describe:
	@echo "=== Basic type analysis of CSV ==="
	@echo "📁 Input file (tests/data/simple.csv):"
	@cat tests/data/simple.csv
	@echo ""
	@echo "🔍 Analyzing column types..."
	cargo run -- describe -i tests/data/simple.csv

run-describe-ddl:
	@echo "=== Generate PostgreSQL DDL ==="
	@echo "📁 Input file (tests/data/simple.csv):"
	@cat tests/data/simple.csv
	@echo ""
	@echo "🏗️ Generating PostgreSQL CREATE TABLE statement..."
	cargo run -- describe -i tests/data/simple.csv --ddl

run-describe-mysql:
	@echo "=== Generate MySQL DDL ==="
	@echo "📁 Input file (tests/data/simple.csv):"
	@cat tests/data/simple.csv
	@echo ""
	@echo "🏗️ Generating MySQL CREATE TABLE statement..."
	cargo run -- describe -i tests/data/simple.csv --ddl --database mysql

run-describe-netezza:
	@echo "=== Generate Netezza DDL ==="
	@echo "📁 Input file (tests/data/simple.csv):"
	@cat tests/data/simple.csv
	@echo ""
	@echo "🏗️ Generating Netezza CREATE TABLE statement..."
	cargo run -- describe -i tests/data/simple.csv --ddl --database netezza

run-describe-types:
	@echo "=== Test type inference patterns ==="
	@echo "📁 Input file (tests/data/type_inference.csv):"
	@cat tests/data/type_inference.csv
	@echo ""
	@echo "🧠 Analyzing complex type patterns..."
	cargo run -- describe -i tests/data/type_inference.csv --ddl -v

run-describe-verbose:
	@echo "=== Type analysis with debug logging ==="
	@echo "📁 Input file (tests/data/promotions.csv):"
	@cat tests/data/promotions.csv
	@echo ""
	@echo "🐛 Running with verbose debug output..."
	RUST_LOG=debug cargo run -- describe -i tests/data/promotions.csv -v

run-describe-nulls:
	@echo "=== Describe with custom NULL handling ==="
	@echo "📁 Input file (tests/data/nulls.csv):"
	@cat tests/data/nulls.csv
	@echo ""
	@echo "🔍 Analyzing with custom NULL patterns..."
	cargo run -- describe -i tests/data/nulls.csv --fnull NULL --fnull "" --ddl

run-describe-pipe:
	@echo "=== Describe pipe-delimited file ==="
	@echo "📁 Input file (tests/data/pipe_delimited.txt):"
	@cat tests/data/pipe_delimited.txt
	@echo ""
	@echo "🔍 Analyzing pipe-delimited data..."
	cargo run -- describe -i tests/data/pipe_delimited.txt -d '|' --ddl

# Debugging targets
debug-simple:
	@echo "=== Running with debug logging ==="
	RUST_LOG=debug cargo run -- parse -i tests/data/simple.csv -v

debug-parser:
	@echo "=== Running with parser debug logging ==="
	RUST_LOG=file2ddl::parser=debug cargo run -- parse -i tests/data/comprehensive.csv -v

# Performance testing
perf-large:
	@echo "=== Performance test with large file ==="
	@echo "Generating large test file (1M rows)..."
	@python3 -c "import csv,random; w=csv.writer(open('/tmp/large.csv','w')); w.writerow(['id','name','value']); [w.writerow([i,f'User{i}',random.randint(1,1000)]) for i in range(1000000)]" 2>/dev/null || \
		(echo "id,name,value" > /tmp/large.csv && for i in {1..1000000}; do echo "$$i,User$$i,$$((RANDOM % 1000))" >> /tmp/large.csv; done)
	@echo "Parsing large file..."
	@time cargo run --release -- parse -i /tmp/large.csv -o /dev/null

# Installation targets
install:
	cargo install --path .

uninstall:
	cargo uninstall file2ddl

# Development shortcuts
dev: fmt build test
	@echo "✅ Ready for development!"

ci: check
	@echo "✅ CI checks passed!"

# Show current version
version:
	@grep "^version" Cargo.toml | head -1 | cut -d'"' -f2

# Create test data
create-test-data:
	@echo "Creating additional test data files..."
	@mkdir -p tests/data
	@echo "id,name,value" > tests/data/normal.csv
	@echo "1,Alice,100" >> tests/data/normal.csv
	@echo "2,Bob,200" >> tests/data/normal.csv
	@echo "Creating malformed.csv..."
	@echo "a,b,c" > tests/data/malformed.csv
	@echo "1,2" >> tests/data/malformed.csv
	@echo "3,4,5,6" >> tests/data/malformed.csv
	@echo "✅ Test data created"

# New feature testing targets (design.md compliance)
test-new-features: test-noheader test-escquote test-boolean-detection test-max-line-length test-badmax-all
	@echo "✅ All new features tested successfully!"

test-noheader:
	@echo "=== Testing --noheader flag ==="
	@echo "📝 Creating headerless CSV file..."
	@echo "Alice,30,true" > /tmp/no_headers.csv
	@echo "Bob,25,false" >> /tmp/no_headers.csv
	@echo "Charlie,35,true" >> /tmp/no_headers.csv
	@echo "📁 Input file (/tmp/no_headers.csv):"
	@cat /tmp/no_headers.csv
	@echo ""
	@echo "🔄 Parsing with --noheader flag..."
	cargo run -- parse -i /tmp/no_headers.csv --noheader -v
	@echo ""
	@echo "🔍 Analyzing with --noheader flag..."
	cargo run -- describe -i /tmp/no_headers.csv --noheader --ddl
	@echo ""

test-escquote:
	@echo "=== Testing --escquote flag ==="
	@echo "📝 Creating CSV with escaped quotes..."
	@echo 'name,quote,path' > /tmp/escaped_test.csv
	@echo '"Alice","She said \"Hello\"","C:\path\to\file"' >> /tmp/escaped_test.csv
	@echo '"Bob","\"Quoted text\"","D:\another\path"' >> /tmp/escaped_test.csv
	@echo "📁 Input file (/tmp/escaped_test.csv):"
	@cat /tmp/escaped_test.csv
	@echo ""
	@echo "🔄 Parsing with --escquote..."
	cargo run -- parse -i /tmp/escaped_test.csv --escquote '"' -v
	@echo ""
	@echo "🔍 Analyzing with --escquote..."
	cargo run -- describe -i /tmp/escaped_test.csv --escquote '"' --ddl
	@echo ""

test-boolean-detection:
	@echo "=== Testing --ftrue/--ffalse boolean detection ==="
	@echo "📝 Creating CSV with custom boolean values..."
	@echo 'name,active,verified' > /tmp/custom_booleans.csv
	@echo 'Alice,yes,Y' >> /tmp/custom_booleans.csv
	@echo 'Bob,no,N' >> /tmp/custom_booleans.csv
	@echo 'Charlie,yes,Y' >> /tmp/custom_booleans.csv
	@echo "📁 Input file (/tmp/custom_booleans.csv):"
	@cat /tmp/custom_booleans.csv
	@echo ""
	@echo "🧠 Analyzing with custom boolean values (--ftrue=yes --ffalse=no)..."
	cargo run -- describe -i /tmp/custom_booleans.csv --ftrue "yes" --ffalse "no" --ddl -v
	@echo ""
	@echo "🧠 Analyzing second column with Y/N values..."
	@echo 'name,status' > /tmp/yn_booleans.csv
	@echo 'Alice,Y' >> /tmp/yn_booleans.csv
	@echo 'Bob,N' >> /tmp/yn_booleans.csv
	cargo run -- describe -i /tmp/yn_booleans.csv --ftrue "Y" --ffalse "N" --ddl -v
	@echo ""

test-max-line-length:
	@echo "=== Testing --max-line-length flag ==="
	@echo "📝 Creating CSV with varying line lengths..."
	@echo 'id,short,medium,long' > /tmp/line_lengths.csv
	@echo '1,hi,hello world,this is a somewhat longer line of text' >> /tmp/line_lengths.csv
	@echo '2,ok,good morning,this is an even longer line of text that should still be processed normally' >> /tmp/line_lengths.csv
	@echo "📁 Input file (/tmp/line_lengths.csv):"
	@cat /tmp/line_lengths.csv
	@echo ""
	@echo "🔄 Parsing with default max-line-length..."
	cargo run -- parse -i /tmp/line_lengths.csv --max-line-length 1048576 -v
	@echo ""
	@echo "🔍 Analyzing with custom max-line-length..."
	cargo run -- describe -i /tmp/line_lengths.csv --max-line-length 1048576 --ddl -v
	@echo ""

test-badmax-all:
	@echo "=== Testing --badmax 'all' functionality ==="
	@echo "📝 Creating CSV with intentional errors..."
	@echo 'id,name,age' > /tmp/errors_test.csv
	@echo '1,Alice,30' >> /tmp/errors_test.csv
	@echo '2,Bob' >> /tmp/errors_test.csv  # Missing field
	@echo '3,Charlie,35,extra' >> /tmp/errors_test.csv  # Extra field
	@echo '4,David,invalid_age' >> /tmp/errors_test.csv  # Not an error in CSV parsing
	@echo '5,Eve,25' >> /tmp/errors_test.csv
	@echo "📁 Input file (/tmp/errors_test.csv):"
	@cat /tmp/errors_test.csv
	@echo ""
	@echo "🔄 Testing --badmax 'all' (unlimited bad rows)..."
	cargo run -- parse -i /tmp/errors_test.csv --badmax all --badfile /tmp/all_bad_rows.csv -v || true
	@echo ""
	@echo "📝 Bad rows file content:"
	@cat /tmp/all_bad_rows.csv 2>/dev/null || echo "No bad rows detected (CSV parser is flexible)"
	@echo ""
	@echo "🔄 Testing --badmax '1' (limited bad rows)..."
	cargo run -- parse -i /tmp/errors_test.csv --badmax 1 --badfile /tmp/limited_bad_rows.csv -v || true
	@echo ""

test-combined-features:
	@echo "=== Testing combined new features ==="
	@echo "📝 Creating complex test scenario..."
	@echo 'Alice,30,yes,null,"She said \"Hi\""' > /tmp/combined_test.csv
	@echo 'Bob,25,no,,"Path: C:\test"' >> /tmp/combined_test.csv
	@echo 'Charlie,invalid,yes,NULL,"Quote: \"test\""' >> /tmp/combined_test.csv
	@echo "📁 Input file (/tmp/combined_test.csv):"
	@cat /tmp/combined_test.csv
	@echo ""
	@echo "🔄 Parse with: --noheader, --escquote, --fnull, --badmax all..."
	cargo run -- parse -i /tmp/combined_test.csv \
		--noheader \
		--escquote '"' \
		--fnull "null" --fnull "NULL" --fnull "" \
		--tnull "\\N" \
		--badmax all \
		--badfile /tmp/combined_bad.csv \
		-v || true
	@echo ""
	@echo "🔍 Analyze with combined features..."
	cargo run -- describe -i /tmp/combined_test.csv \
		--noheader \
		--escquote '"' \
		--fnull "null" --fnull "NULL" --fnull "" \
		--ftrue "yes" --ffalse "no" \
		--max-line-length 1048576 \
		--ddl -v
	@echo ""

# Enhanced existing targets to test new flags where relevant
run-describe-enhanced:
	@echo "=== Enhanced describe with all new flags ==="
	@echo "📁 Input file (tests/data/simple.csv):"
	@cat tests/data/simple.csv
	@echo ""
	@echo "🔍 Analyzing with enhanced flags..."
	cargo run -- describe -i tests/data/simple.csv \
		--fnull "" \
		--ftrue "true" --ffalse "false" \
		--encoding utf-8 \
		--max-line-length 1048576 \
		--ddl --database postgres -v

run-parse-enhanced:
	@echo "=== Enhanced parse with all new flags ==="
	@echo "📁 Input file (tests/data/simple.csv):"
	@cat tests/data/simple.csv
	@echo ""
	@echo "🔄 Parsing with enhanced flags..."
	cargo run -- parse -i tests/data/simple.csv \
		--fnull "" \
		--tnull "NULL" \
		--encoding utf-8 \
		--max-line-length 1048576 \
		--badmax all \
		-v

.DEFAULT_GOAL := help