use crate::common;
use serde_json::Value;
use std::{fs, path::Path, process::Output};

fn run(root: &Path, args: &[&str]) -> Output {
    common::locmetry()
        .current_dir(root)
        .env("XDG_CONFIG_HOME", root.join("absent-config"))
        .args(args)
        .output()
        .unwrap()
}

fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("project/src/nested")).unwrap();
    fs::write(temp.path().join("project/src/nested/main.py"), "print(1)\n").unwrap();
    temp
}

fn has_table(text: &str, name: &str) -> bool {
    text.lines()
        .any(|line| line.trim_start().starts_with(&format!("{name} ")))
}

#[test]
fn folders_are_opt_in_for_root_subfolder_and_saved_tables_independently_of_notes() {
    let temp = fixture();
    for subfolder in [false, true] {
        for verbose in [false, true] {
            for extra in [
                vec![],
                vec!["--show-folders"],
                vec!["--folder-rows", "8"],
                vec!["--show-folders", "--folder-rows", "8"],
            ] {
                let expected = !extra.is_empty();
                let mut args = vec!["project", "--format", "table"];
                if subfolder {
                    args.extend(["--subfolder", "src"]);
                }
                if verbose {
                    args.push("--verbose");
                }
                args.extend(extra);
                let output = run(temp.path(), &args);
                assert!(
                    output.status.success(),
                    "{}",
                    String::from_utf8_lossy(&output.stderr)
                );
                assert!(output.stderr.is_empty());
                let text = String::from_utf8(output.stdout).unwrap();
                assert!(has_table(&text, "Language"));
                assert_eq!(has_table(&text, "Folder"), expected, "{args:?}: {text}");
                assert_eq!(text.contains("Unclaimed source:"), verbose);
                assert!(text.lines().last().unwrap().starts_with("Measured at:"));
                args.extend(["--output", "saved.txt"]);
                let output = run(temp.path(), &args);
                assert!(output.status.success());
                assert!(output.stdout.is_empty());
                assert!(output.stderr.is_empty());
                let saved = fs::read_to_string(temp.path().join("saved.txt")).unwrap();
                assert_eq!(
                    text.rsplit_once("\nMeasured at: ").unwrap().0,
                    saved.rsplit_once("\nMeasured at: ").unwrap().0
                );
            }
        }
    }
}

#[test]
fn folders_warn_without_changing_non_table_measurements() {
    let temp = fixture();
    for format in ["markdown", "json", "headline"] {
        let original = run(temp.path(), &["project", "--format", format]);
        let requested = run(
            temp.path(),
            &["project", "--format", format, "--show-folders"],
        );
        assert!(original.status.success());
        assert!(requested.status.success());
        assert!(String::from_utf8_lossy(&requested.stderr)
            .contains("'--show-folders' has no effect here. It applies only to table output"));
        if format == "markdown" {
            let original = String::from_utf8(original.stdout).unwrap();
            let requested = String::from_utf8(requested.stdout).unwrap();
            assert!(requested.contains("## Folder breakdown"));
            assert_eq!(
                original.rsplit_once("\nMeasured at: ").unwrap().0,
                requested.rsplit_once("\nMeasured at: ").unwrap().0
            );
        } else {
            let mut original: Value = serde_json::from_slice(&original.stdout).unwrap();
            let mut requested: Value = serde_json::from_slice(&requested.stdout).unwrap();
            original["metadata"]["measured_at_unix_seconds"] = Value::Null;
            requested["metadata"]["measured_at_unix_seconds"] = Value::Null;
            assert_eq!(original, requested);
        }
    }
    let output = run(temp.path(), &["project", "--show-folders"]);
    assert!(output.status.success());
    assert!(serde_json::from_slice::<Value>(&output.stdout).is_ok());
    assert!(String::from_utf8_lossy(&output.stderr).contains("'--show-folders' has no effect here"));
}

#[test]
fn workspace_keeps_member_rows_and_folders_is_a_root_measurement_option() {
    let temp = fixture();
    let workspace = run(
        temp.path(),
        &["workspace", "--root", "project", "--format", "table"],
    );
    assert!(workspace.status.success());
    let text = String::from_utf8(workspace.stdout).unwrap();
    assert!(has_table(&text, "Member"));
    assert!(!has_table(&text, "Folder"));
    for args in [
        vec!["workspace", "--root", "project", "--show-folders"],
        vec!["--show-folders", "workspace", "--root", "project"],
        vec!["--schema", "report", "--show-folders"],
        vec!["--guide", "reports", "--show-folders"],
    ] {
        let output = run(temp.path(), &args);
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        assert!(output.stdout.is_empty());
    }
    let help = run(temp.path(), &["--help"]);
    assert!(String::from_utf8_lossy(&help.stdout).contains("--show-folders"));
}

#[test]
fn failures_stay_visible_with_or_without_folder_rows() {
    let temp = fixture();
    fs::write(temp.path().join("project/bad.ipynb"), "invalid").unwrap();
    for folders in [false, true] {
        let mut args = vec!["project", "--format", "table"];
        if folders {
            args.push("--show-folders");
        }
        let output = run(temp.path(), &args);
        assert_eq!(output.status.code(), Some(2));
        let text = String::from_utf8(output.stdout).unwrap();
        assert_eq!(has_table(&text, "Folder"), folders);
        assert!(text.contains("Not counted: 1 file with a parsing failure"));
        assert!(text.contains("Backend reported incomplete counts"));
        assert!(String::from_utf8_lossy(&output.stderr).contains("bad.ipynb"));
        assert!(text.lines().last().unwrap().starts_with("Measured at:"));
    }
}
