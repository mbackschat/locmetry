//! Helpers shared by the integration tests. Each test crate compiles its own
//! copy and uses a subset, so unused helpers are expected.
#![allow(dead_code)]

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
