use std::io::Write;
use std::process::Command;
use std::str;
use tempfile::NamedTempFile;

#[test]
fn test_diagnose_field_count_issues() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "id,name,age,city").unwrap();
    writeln!(temp_file, "1,John,25,New York").unwrap();
    writeln!(temp_file, "2,Jane,30").unwrap(); // Missing field
    writeln!(temp_file, "3,Bob,35,Chicago,Extra").unwrap(); // Extra field
    temp_file.flush().unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "diagnose",
            "-i",
            temp_file.path().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute diagnose command");

    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(
        output.status.success(),
        "Command failed: {}",
        str::from_utf8(&output.stderr).unwrap()
    );
    assert!(stdout.contains("Total lines processed: 4"));
    assert!(stdout.contains("Problematic lines found: 2"));
    assert!(stdout.contains("Field Count Issues:"));
    assert!(stdout.contains("Lines with 3 fields (expected 4): 1 lines"));
    assert!(stdout.contains("[L3]: 2,Jane,30"));
    assert!(stdout.contains("Lines with 5 fields (expected 4): 1 lines"));
    assert!(stdout.contains("[L4]: 3,Bob,35,Chicago,Extra"));
}

#[test]
fn test_diagnose_custom_fields() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "id,name,age,city").unwrap();
    writeln!(temp_file, "1,John,25,New York").unwrap();
    writeln!(temp_file, "2,Jane,30").unwrap(); // Missing field
    temp_file.flush().unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "diagnose",
            "-i",
            temp_file.path().to_str().unwrap(),
            "--fields",
            "3",
        ])
        .output()
        .expect("Failed to execute diagnose command");

    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(
        output.status.success(),
        "Command failed: {}",
        str::from_utf8(&output.stderr).unwrap()
    );
    assert!(stdout.contains("Total lines processed: 3"));
    assert!(stdout.contains("Problematic lines found: 1"));
    assert!(stdout.contains("Lines with 4 fields (expected 3): 1 lines"));
    assert!(stdout.contains("[L2]: 1,John,25,New York"));
}

#[test]
fn test_diagnose_noheader() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "1,John,25,New York").unwrap();
    writeln!(temp_file, "2,Jane,30").unwrap(); // Missing field
    writeln!(temp_file, "3,Bob,35,Chicago,Extra").unwrap(); // Extra field
    temp_file.flush().unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "diagnose",
            "-i",
            temp_file.path().to_str().unwrap(),
            "--noheader",
        ])
        .output()
        .expect("Failed to execute diagnose command");

    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(
        output.status.success(),
        "Command failed: {}",
        str::from_utf8(&output.stderr).unwrap()
    );
    assert!(stdout.contains("Total lines processed: 3"));
    assert!(stdout.contains("Problematic lines found: 2"));
    assert!(stdout.contains("Field Count Issues:"));
    assert!(stdout.contains("Lines with 3 fields (expected 4): 1 lines"));
    assert!(stdout.contains("[L2]: 2,Jane,30"));
    assert!(stdout.contains("Lines with 5 fields (expected 4): 1 lines"));
    assert!(stdout.contains("[L3]: 3,Bob,35,Chicago,Extra"));
}

#[test]
fn test_diagnose_badmax_limit() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "id,name,age,city").unwrap();
    writeln!(temp_file, "1,John,25,New York").unwrap();
    writeln!(temp_file, "2,Jane,30").unwrap(); // Missing field
    writeln!(temp_file, "3,Bob,35,Chicago,Extra").unwrap(); // Extra field
    temp_file.flush().unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "diagnose",
            "-i",
            temp_file.path().to_str().unwrap(),
            "--badmax",
            "1",
        ])
        .output()
        .expect("Failed to execute diagnose command");

    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(
        output.status.success(),
        "Command failed: {}",
        str::from_utf8(&output.stderr).unwrap()
    );
    assert!(stdout.contains("Total lines processed: 3"));
    assert!(stdout.contains("Problematic lines found: 1 (stopped at --badmax limit)"));
    assert!(stdout.contains("[L3]: 2,Jane,30"));
    assert!(!stdout.contains("[L4]")); // Should not reach line 4
}

#[test]
fn test_diagnose_clean_file() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "id,name,age,city").unwrap();
    writeln!(temp_file, "1,John,25,New York").unwrap();
    writeln!(temp_file, "2,Jane,30,Boston").unwrap();
    writeln!(temp_file, "3,Bob,35,Chicago").unwrap();
    temp_file.flush().unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "diagnose",
            "-i",
            temp_file.path().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute diagnose command");

    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(
        output.status.success(),
        "Command failed: {}",
        str::from_utf8(&output.stderr).unwrap()
    );
    assert!(stdout.contains("Total lines processed: 4"));
    assert!(stdout.contains("Problematic lines found: 0"));
    assert!(stdout.contains("✓ No issues found in the CSV file."));
}

#[test]
fn test_diagnose_pipe_delimiter() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "id|name|age|city").unwrap();
    writeln!(temp_file, "1|John|25|New York").unwrap();
    writeln!(temp_file, "2|Jane|30").unwrap(); // Missing field
    writeln!(temp_file, "3|Bob|35|Chicago|Extra").unwrap(); // Extra field
    temp_file.flush().unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "diagnose",
            "-i",
            temp_file.path().to_str().unwrap(),
            "-d",
            "|",
        ])
        .output()
        .expect("Failed to execute diagnose command");

    let stdout = str::from_utf8(&output.stdout).unwrap();
    assert!(
        output.status.success(),
        "Command failed: {}",
        str::from_utf8(&output.stderr).unwrap()
    );
    assert!(stdout.contains("Total lines processed: 4"));
    assert!(stdout.contains("Problematic lines found: 2"));
    assert!(stdout.contains("[L3]: 2|Jane|30"));
    assert!(stdout.contains("[L4]: 3|Bob|35|Chicago|Extra"));
}

#[test]
fn test_diagnose_verbose_output() {
    let mut temp_file = NamedTempFile::new().unwrap();
    writeln!(temp_file, "id,name,age,city").unwrap();
    writeln!(temp_file, "1,John,25,New York").unwrap();
    writeln!(temp_file, "2,Jane,30").unwrap(); // Missing field
    temp_file.flush().unwrap();

    let output = Command::new("cargo")
        .args([
            "run",
            "--",
            "diagnose",
            "-i",
            temp_file.path().to_str().unwrap(),
            "--verbose",
        ])
        .env("RUST_LOG", "info")
        .output()
        .expect("Failed to execute diagnose command");

    let stderr = str::from_utf8(&output.stderr).unwrap();
    assert!(output.status.success(), "Command failed: {}", stderr);
    // Verbose mode should produce log messages to stderr
    assert!(stderr.contains("Starting diagnose command") || stderr.contains("Diagnosis complete"));
}
