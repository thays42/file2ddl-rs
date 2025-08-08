use std::process::Command;
use std::fs;
use tempfile::NamedTempFile;

#[test]
fn test_parse_simple_csv() {
    let output = Command::new("cargo")
        .args(&["run", "--", "parse", "-i", "tests/data/simple.csv"])
        .output()
        .expect("Failed to execute command");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("id,name,age,active"));
    assert!(stdout.contains("1,Alice,30,true"));
}

#[test]
fn test_parse_pipe_delimiter() {
    let output_file = NamedTempFile::new().unwrap();
    let output_path = output_file.path().to_str().unwrap();
    
    let status = Command::new("cargo")
        .args(&[
            "run", "--", "parse",
            "-i", "tests/data/pipe_delimited.txt",
            "-o", output_path,
            "-d", "|"
        ])
        .status()
        .expect("Failed to execute command");
    
    assert!(status.success());
    
    let content = fs::read_to_string(output_path).unwrap();
    assert!(content.contains("id|name|age|active"));
    assert!(content.contains("1|Alice|30|true"));
}

#[test]
fn test_parse_custom_delimiter_conversion() {
    let output = Command::new("cargo")
        .args(&[
            "run", "--", "parse",
            "-i", "tests/data/pipe_delimited.txt",
            "-d", "|"
        ])
        .output()
        .expect("Failed to execute command");
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("id|name|age|active"));
    assert!(stdout.contains("1|Alice|30|true"));
}