use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Helper to get the pandorust binary path.
fn pandorust_cmd() -> Command {
    Command::new(env!("CARGO_BIN_EXE_pandorust"))
}

#[test]
fn test_md_to_html_cli() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("input.md");
    let output = tmp.path().join("output.html");

    fs::write(
        &input,
        r#"---
title: CLI Test
---

# Hello World

This is a **bold** paragraph.
"#,
    )
    .unwrap();

    let status = pandorust_cmd()
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .status()
        .expect("failed to execute pandorust");

    assert!(status.success(), "pandorust CLI should exit successfully");
    assert!(output.exists(), "output HTML file should exist");

    let html = fs::read_to_string(&output).unwrap();
    assert!(
        html.contains("<title>CLI Test</title>"),
        "HTML should contain the title from front matter"
    );
    assert!(
        html.contains("<h1"),
        "HTML should contain an h1 heading"
    );
    assert!(
        html.contains("<strong>bold</strong>"),
        "HTML should contain bold text"
    );
}

#[test]
fn test_md_to_docx_cli() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("input.md");
    let output = tmp.path().join("output.docx");

    fs::write(
        &input,
        r#"# DOCX Test

A paragraph with **bold** and *italic*.
"#,
    )
    .unwrap();

    let status = pandorust_cmd()
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .status()
        .expect("failed to execute pandorust");

    assert!(status.success(), "pandorust CLI should exit successfully");
    assert!(output.exists(), "output DOCX file should exist");

    let bytes = fs::read(&output).unwrap();
    assert!(
        bytes.len() > 100,
        "DOCX file should be non-trivial size, got {} bytes",
        bytes.len()
    );
    assert_eq!(
        &bytes[0..2],
        b"PK",
        "DOCX file should start with PK (zip header)"
    );
}

#[test]
fn test_format_detection_with_explicit_flags() {
    let tmp = TempDir::new().unwrap();
    let input = tmp.path().join("source.txt"); // non-standard extension
    let output = tmp.path().join("result.txt"); // non-standard extension

    fs::write(
        &input,
        "# Explicit Format Test\n\nA paragraph.\n",
    )
    .unwrap();

    // Use explicit -f and -t flags to override extension detection
    let status = pandorust_cmd()
        .arg(input.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .arg("-f")
        .arg("md")
        .arg("-t")
        .arg("html")
        .status()
        .expect("failed to execute pandorust");

    assert!(status.success(), "pandorust CLI should succeed with explicit format flags");
    assert!(output.exists(), "output file should exist");

    let html = fs::read_to_string(&output).unwrap();
    assert!(
        html.contains("<h1"),
        "Output should be valid HTML with h1"
    );
}

#[test]
fn test_missing_input_file() {
    let tmp = TempDir::new().unwrap();
    let nonexistent = tmp.path().join("does_not_exist.md");
    let output = tmp.path().join("output.html");

    let result = pandorust_cmd()
        .arg(nonexistent.to_str().unwrap())
        .arg("-o")
        .arg(output.to_str().unwrap())
        .output()
        .expect("failed to execute pandorust");

    assert!(
        !result.status.success(),
        "pandorust CLI should fail when input file does not exist"
    );

    let stderr = String::from_utf8_lossy(&result.stderr);
    assert!(
        stderr.contains("Error"),
        "stderr should contain an error message, got: {}",
        stderr
    );
}
