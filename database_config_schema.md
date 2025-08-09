# Database Configuration Schema

The `--database-config` option allows you to define custom database dialects using a JSON configuration file.

## Schema

```json
{
  "name": "string",                    // Human-readable database name
  "type_mappings": {                   // Required type mappings
    "Boolean": "string",               // SQL Boolean type
    "SmallInt": "string",              // SQL SmallInt type  
    "Integer": "string",               // SQL Integer type
    "BigInt": "string",                // SQL BigInt type
    "DoublePrecision": "string",       // SQL Double/Float type
    "Date": "string",                  // SQL Date type
    "Time": "string",                  // SQL Time type
    "DateTime": "string",              // SQL DateTime/Timestamp type
    "Varchar": "string",               // SQL VARCHAR(n) - use {} for length placeholder
    "VarcharUnlimited": "string"       // SQL unlimited text type
  },
  "features": {                        // Optional feature flags
    "unlimited_varchar": boolean,      // Supports unlimited VARCHAR
    "boolean_type": boolean,           // Has native boolean type
    "double_precision": boolean,       // Supports double precision
    "timestamp_type": boolean          // Has timestamp type
  },
  "default_varchar_length": number,    // Default VARCHAR length (optional)
  "unlimited_varchar_type": "string"   // Type to use for unlimited VARCHAR
}
```

## Example Configurations

### Oracle
```json
{
  "name": "Oracle",
  "type_mappings": {
    "Boolean": "CHAR(1)",
    "SmallInt": "NUMBER(5)",
    "Integer": "NUMBER(10)",
    "BigInt": "NUMBER(19)",
    "DoublePrecision": "BINARY_DOUBLE",
    "Date": "DATE",
    "Time": "TIMESTAMP",
    "DateTime": "TIMESTAMP",
    "Varchar": "VARCHAR2({})",
    "VarcharUnlimited": "CLOB"
  },
  "features": {
    "unlimited_varchar": true,
    "boolean_type": false
  },
  "default_varchar_length": 4000,
  "unlimited_varchar_type": "CLOB"
}
```

### SQLite
```json
{
  "name": "SQLite",
  "type_mappings": {
    "Boolean": "INTEGER",
    "SmallInt": "INTEGER",
    "Integer": "INTEGER", 
    "BigInt": "INTEGER",
    "DoublePrecision": "REAL",
    "Date": "TEXT",
    "Time": "TEXT",
    "DateTime": "TEXT",
    "Varchar": "TEXT",
    "VarcharUnlimited": "TEXT"
  },
  "features": {
    "unlimited_varchar": true,
    "boolean_type": false
  },
  "unlimited_varchar_type": "TEXT"
}
```

## Usage

```bash
# Use custom database configuration
cargo run -- describe -i data.csv --ddl --database-config oracle.json

# Still works with built-in databases
cargo run -- describe -i data.csv --ddl --database postgres
```

## Validation

The configuration file is validated on load:
- All required type mappings must be present
- JSON must be well-formed
- File must be readable

## Type Mapping Templates

The `Varchar` mapping supports a `{}` placeholder that gets replaced with the actual length:
- `"VARCHAR({})"` → `VARCHAR(50)` for a 50-character field
- `"VARCHAR2({})"` → `VARCHAR2(100)` for a 100-character field

This allows proper sizing of VARCHAR columns based on the actual data analyzed.