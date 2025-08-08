use std::io::Write;
use std::process::Command;
use tempfile::NamedTempFile;

#[test]
fn test_basic_describe() {
    let csv_data = "id,name,age\n1,Alice,25\n2,Bob,30\n";

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(csv_data.as_bytes()).unwrap();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "describe",
            "-i",
            temp_file.path().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("Column"));
    assert!(stdout.contains("Type"));
    assert!(stdout.contains("id"));
    assert!(stdout.contains("SMALLINT"));
    assert!(stdout.contains("name"));
    assert!(stdout.contains("VARCHAR"));
    assert!(stdout.contains("age"));
}

#[test]
fn test_ddl_generation() {
    let csv_data = "id,active\n1,true\n2,false\n";

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(csv_data.as_bytes()).unwrap();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "describe",
            "-i",
            temp_file.path().to_str().unwrap(),
            "--ddl",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("CREATE TABLE"));
    assert!(stdout.contains("id SMALLINT NOT NULL"));
    assert!(stdout.contains("active BOOLEAN NOT NULL"));
}

#[test]
fn test_mysql_ddl() {
    let csv_data = "test_col\n3.14\n2.71\n";

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(csv_data.as_bytes()).unwrap();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "describe",
            "-i",
            temp_file.path().to_str().unwrap(),
            "--ddl",
            "--database",
            "mysql",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("CREATE TABLE"));
    assert!(stdout.contains("test_col DOUBLE NOT NULL"));
}

#[test]
fn test_null_handling() {
    let csv_data = "nullable_col\n123\n\nNULL\n456\n";

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(csv_data.as_bytes()).unwrap();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "describe",
            "-i",
            temp_file.path().to_str().unwrap(),
            "-v",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("33.3%")); // 1 null out of 3 non-header values
    assert!(stdout.contains("nullable_col"));
    assert!(stdout.contains("SMALLINT"));
}

#[test]
fn test_date_inference() {
    let csv_data = "date_col\n2023-01-15\n2023-02-20\n2023-03-10\n";

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(csv_data.as_bytes()).unwrap();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "describe",
            "-i",
            temp_file.path().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("date_col"));
    assert!(stdout.contains("DATE"));
}

#[test]
fn test_time_inference() {
    let csv_data = "time_col\n09:30:00\n14:15:30\n23:45:59\n";

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(csv_data.as_bytes()).unwrap();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "describe",
            "-i",
            temp_file.path().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("time_col"));
    assert!(stdout.contains("TIME"));
}

#[test]
fn test_datetime_inference() {
    let csv_data = "datetime_col\n2023-01-15 09:30:00\n2023-02-20 14:15:30\n";

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(csv_data.as_bytes()).unwrap();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "describe",
            "-i",
            temp_file.path().to_str().unwrap(),
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("datetime_col"));
    assert!(stdout.contains("DATETIME") || stdout.contains("TIMESTAMP"));
}

#[test]
fn test_pipe_delimiter() {
    let csv_data = "id|name|value\n1|Alice|100\n2|Bob|200\n";

    let mut temp_file = NamedTempFile::new().unwrap();
    temp_file.write_all(csv_data.as_bytes()).unwrap();

    let output = Command::new("cargo")
        .args(&[
            "run",
            "--",
            "describe",
            "-i",
            temp_file.path().to_str().unwrap(),
            "-d",
            "|",
        ])
        .output()
        .expect("Failed to execute command");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("id"));
    assert!(stdout.contains("name"));
    assert!(stdout.contains("value"));
    assert!(stdout.contains("SMALLINT"));
    assert!(stdout.contains("VARCHAR"));
}

#[test]
fn test_stdin_input() {
    let csv_data = "test\n42\n";

    let mut child = Command::new("cargo")
        .args(&["run", "--", "describe"])
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .expect("Failed to spawn command");

    if let Some(stdin) = child.stdin.as_mut() {
        stdin.write_all(csv_data.as_bytes()).unwrap();
        stdin.flush().unwrap();
    }

    let result = child.wait_with_output().unwrap();
    let stdout = String::from_utf8(result.stdout).unwrap();
    assert!(stdout.contains("test"));
    assert!(stdout.contains("SMALLINT"));
}
