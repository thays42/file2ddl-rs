This document outlines the requirements for `file2ddl`, a command-line interface application that assists users in preparing raw data files for loading into database tables. The application will offer subcommands to parse and describe data, adhering to strict performance and memory constraints.

### Subcommands
- `parse`: Transforms the input plain text delimited file into a standardized format based on user specifications.
    - Standardized Output Format: The output conforms to [RFC 4180](https://www.rfc-editor.org/rfc/rfc4180) for quoting and field separation.
    - Delimiter Parameter: User specifies the expected delimiter with `-d` or `--delimiter`.
    - Column headers: User specifies if the file does not start with column headers with `-H` or `--noheader`. By default, the application assumes the file starts with a column header line.
    - Quote Character Parameter: User specifies the expected quote character `-q double` or `--quotes double`. Options are `single` or `double`, corresponding to `'` and `"`.
    - Quote Escaping Parameter: User may specify the expected quote escaping parameter with `--escquote`. Default, this is `"`, per RFC 4180.
    - Null Parameters: User may specify a set of values that should be treated as NULL with `--fnull` and a desired NULL representation with `--tnull`. Multiple values are specified with multiple `--fnull` flags. Any value in `--fnull` is transformed into the value specified by `--tnull`. E.g., if `--fnull "NA"` and `--tnull "NULL"`, then any `"NA"` field value is transformed into `"NULL"`. If neither of these flags are specified, the application does not consider any values NULL.
    - Error Handling During Parsing: For malformed rows or type conversion errors, the application will log a warning with relevant details (line number, error type) and stop processing by default. 
        - User may request bad rows be output to a file with `--badfile`.
        - User may request processing continue up to a number of errors with `--badmax`. `--badmax all` will output all bad rows.
        - If any bad rows, a summary is output to stderr ("X bad records encountered") and the process exits with non-zero status.
    - Encoding: Default, `UTF-8`. User may specify with `--encoding` flag.

- `describe`: Outputs descriptive information about the input file, inferring an ANSI SQL table definition. What descriptors are output are based on the flags in this command call. This command takes the same flags as the `parse` command and uses the output of the `parse` command when collecting this descriptive information.
    - SQL DDL (`--ddl`): A suggested `CREATE TABLE` statement that fits the data.
        - Data Type Inference: Types will be inferred based on all lines. The application will assign the smallest type that fits the data seen so far and promote the assigned type when it observes a value that would not fit in the assigned type.
            - Matching Values to Types: Once the application parses the value in a field, it will apply a sequence of tests against that value to determine the type. 
            - Type precedence is based on Postresql data types by default, but the application will support multiple databases via a `--database` parameter. Thus, the type inference engine should be configurable so that it is easy to describe how it works for Postgres vs. Netezza vs. MySQL, etc.
            - Supported Types: BOOLEAN, INTEGER, SMALLINT, BIGINT, DOUBLE PRECISION, DATE, TIME, DATETIME, and VARCHAR.
            - BOOLEAN detection: User specifies `--ftrue <TRUE_VALUE>` and `--ffalse <FALSE_VALUE>`. Application checks field values against these two values. These default to 1 and 0.
            - Integer detection: Application checks if field value only contains digits and optionally starts with a `-` character. If so, it converts the value to an integer and determines if the integer value fits within the range corresponding to the integer type.
                - SMALLINT: –32,768 to 32,767
                - INTEGER: –2,147,483,648 to 2,147,483,647
                - BIGINT: -9,223,372,036,854,775,808 to 9,223,372,036,854,775,807
            - DATE: %Y-%m-%d default. User can override with `--fdate`.
            - TIME: %H:%M:%S default. User can override with `--ftime`.
            - DATETIME: %Y-%m-%d %H:%M:%S default. User can override with `--fdatetime`.
            - Double precision detection: Application checks if field value only contains digits, optionally starts with a `-` character, and optionally contains a `.` character. Scientific notation is specifically not supported.
            - Character detection: VARCHAR(n) is the fallback data type if no other type fits the data. `n` grows based on the number of bytes in the field.
        - Type promotion: 
            - BOOLEAN -> SMALLINT -> INTEGER -> BIGINT -> DOUBLE PRECISION -> VARCHAR
            - DATE -> VARCHAR
            - TIME -> VARCHAR
            - DATETIME -> VARCHAR 
        - Ambiguity Resolution: When data can fit multiple types, the precedence listed above will be applied. For instance, "123456" will be inferred as `INTEGER` before `VARCHAR(6)`.
        - Date parsing: The application uses a reasonable default format for date, time, and datetime fields. The user can specify custom formats using `--fdate`, `--ftime`, and `--fdatetime`, using standard syntax in a `strptime` function.
        - NULL values: The user can specify a set of values that should be treated as NULL with `--fnull`. If a field ever takes one of these values, that value is ignored in the data type inference process.
        - Verbose logging:
            - Type promotions. On the first line, verbose logging presents inferred data types. When a type is promoted, verbose logging presents that the type was promoted from TYPE1 to TYPE2 on line N. If there are column headers, fields are identified based on column headers. If not, column headers are identified by `F1`, `F2`, etc.

### Development Requirements

- Performance: The application must process data quickly, though this goal is directional and has no concrete target metrics. This is a critical consideration for the choice of programming language and implementation strategies. 
- Input/Output (I/O): The application must support:
    - Reading input from `stdin` (allowing text streams to be piped into it).
    - Writing output to `stdout` (allowing output redirection).
    - Alternatively, users can specify an input file path (`-i <path>`) and an output file path (`-o <path>`) instead of using pipes/redirection.
- Verbose Mode: The application must support a verbose mode (`-v` or `--verbose`) to log detailed information about the parsing and description process, including progress, warnings, and errors. This goes to stderr.
- Memory Usage: The application must use limited memory, ideally holding no more than a few logical lines from the file in memory at any given time. This requires a streaming approach to data processing, utilizing Rust's iterator patterns and buffered I/O to process data line by line or in small chunks, avoiding loading entire files into memory.
    - Maximum Line Length: A default maximum logical line length of 1MB will be enforced. Users can override this (`--max-line-length <bytes>`). Lines exceeding this limit will result in an error indicating the line that exceeds the max line length.
- Test Suite: The application must have a rich test suite covering:
    - Unit Tests: For individual functions and components.
    - Integration Tests: For interactions between components and subcommand functionality.
    - End-to-End Tests: For common user flows.
    - Input File Variety: The test suite will include a wide array of input files demonstrating robustness across different delimiters, quoting styles, malformed data, empty files, and very large files.
    - Test Data Management: A dedicated `tests/data` directory will house representative input files, including valid and invalid cases. A test script or utility may be used to generate large test files programmatically.

