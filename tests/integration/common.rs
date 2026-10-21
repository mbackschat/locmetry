//! Helpers shared by the integration tests.

use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

/// The binary isolated from the developer's own user defaults; a test that
/// needs user defaults points `XDG_CONFIG_HOME` at its own directory.
pub fn locmetry() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_locmetry"));
    command.env("XDG_CONFIG_HOME", "/nonexistent/locmetry-tests");
    command
}

/// Writes `text` to `path` below `root`, creating its folders.
pub fn write(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// Writes each `(path, text)` below `root`.
pub fn write_all(root: &Path, files: &[(&str, &str)]) {
    for (path, text) in files {
        write(root, path, text);
    }
}

/// Stdout as JSON, after the run exited with `status`; stderr explains a mismatch.
pub fn json(output: &Output, status: i32) -> Value {
    assert_eq!(
        output.status.code(),
        Some(status),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

/// Stdout of a successful run; stderr explains a failure.
pub fn stdout(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

/// Cells of the first line starting with `label`, without separators or commas.
pub fn cells(text: &str, label: &str) -> Vec<String> {
    let line = text
        .lines()
        .find(|line| line.starts_with(&format!(" {label} ")))
        .unwrap_or_else(|| panic!("no {label} row in:\n{text}"));
    line.replace('│', " ")
        .split_whitespace()
        .map(String::from)
        .collect()
}

/// `text` without SGR sequences, whatever the palette emits.
pub fn unstyled(text: &str) -> String {
    let mut plain = String::new();
    let mut rest = text;
    while let Some((before, after)) = rest.split_once("\x1b[") {
        plain += before;
        rest = after.split_once('m').unwrap().1;
    }
    plain + rest
}

/// Recover displayed commands across Clap's terminal-width wrapping.
pub fn help_examples(help: &str) -> Vec<(String, String)> {
    let block = help
        .split_once("\nExamples:\n")
        .unwrap()
        .1
        .split("\n\n")
        .next()
        .unwrap();
    let mut examples: Vec<(String, String)> = Vec::new();
    for line in block.lines() {
        if let Some(command) = line.strip_prefix("    ") {
            let last = examples.last_mut().unwrap();
            if !last.1.is_empty() {
                last.1.push(' ');
            }
            last.1.push_str(command);
        } else {
            examples.push((line.trim().trim_end_matches(':').to_owned(), String::new()));
        }
    }
    examples
}
