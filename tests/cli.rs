use serde_json::{json, Value};
use std::{fs, process::Command};

fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join("source")).unwrap();
    fs::write(temp.path().join("source/main.rs"), "fn main() {}\n").unwrap();
    fs::write(
        temp.path().join("rules.json"),
        json!({
            "project":"synthetic", "root":temp.path().join("source"),
            "rules":[{"id":"app","pattern":"\\.rs$","purpose":"production"}]
        })
        .to_string(),
    )
    .unwrap();
    temp
}

#[test]
fn binary_measures_without_any_executables_on_path() {
    let temp = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .env("PATH", "")
        .args(["--config"])
        .arg(temp.path().join("rules.json"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let document: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        document["summary"]["source_purpose"]["production"]["code"],
        1
    );
    assert_eq!(document["metadata"]["backend"], "tokei 15.0.0 (embedded)");
    assert_eq!(document["metadata"]["backend_inaccurate"], false);
}

#[test]
fn report_files_are_generated_outside_the_scan_root() {
    let temp = fixture();
    let prefix = temp.path().join("output/report");
    let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .arg("--config")
        .arg(temp.path().join("rules.json"))
        .arg("--output")
        .arg(&prefix)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value =
        serde_json::from_slice(&fs::read(prefix.with_extension("json")).unwrap()).unwrap();
    assert_eq!(report["summary"]["source"]["code"], 1);
    let markdown = fs::read_to_string(prefix.with_extension("md")).unwrap();
    assert!(markdown.contains("| production | 1 | 1 | 0 | 0 | 1 |"));
    assert!(prefix.with_extension("tokei.json").exists());
}

#[test]
fn output_in_scan_root_is_rejected_before_writing() {
    let temp = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .arg("--config")
        .arg(temp.path().join("rules.json"))
        .arg("--output")
        .arg(temp.path().join("source/reports/report"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!temp.path().join("source/reports").exists());
}

#[test]
fn missing_root_and_invalid_config_fail_without_success_json() {
    let temp = fixture();
    for args in [
        vec!["--config", "absent.json"],
        vec!["--config", "rules.json", "--root", "absent"],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_locmetry"))
            .current_dir(temp.path())
            .args(args)
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(result.stdout.is_empty());
    }
}

fn run_with_output(temp: &tempfile::TempDir, prefix: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .arg("--config")
        .arg(temp.path().join("rules.json"))
        .arg("--output")
        .arg(prefix)
        .output()
        .unwrap()
}

#[test]
fn dotted_output_prefixes_keep_distinct_report_files() {
    let temp = fixture();
    for version in ["run.v1", "run.v2"] {
        let output = run_with_output(&temp, &temp.path().join("output").join(version));
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    for name in ["run.v1", "run.v2"] {
        for suffix in ["json", "md", "tokei.json"] {
            let path = temp.path().join(format!("output/{name}.{suffix}"));
            assert!(path.exists(), "missing {}", path.display());
        }
    }
    assert!(!temp.path().join("output/run.json").exists());
}

#[test]
fn output_prefix_that_would_overwrite_the_configuration_is_rejected() {
    let temp = fixture();
    let config = fs::read(temp.path().join("rules.json")).unwrap();
    let output = run_with_output(&temp, &temp.path().join("rules"));
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("configuration"));
    assert_eq!(fs::read(temp.path().join("rules.json")).unwrap(), config);
    assert!(!temp.path().join("rules.md").exists());
}

#[test]
fn markdown_format_writes_a_readable_report_to_stdout() {
    let temp = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .args(["--format", "markdown", "--config"])
        .arg(temp.path().join("rules.json"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("# synthetic measured classification"));
    assert!(stdout.contains("| production | 1 | 1 | 0 | 0 | 1 |"));
}

#[cfg(unix)]
#[test]
fn unreadable_files_produce_partial_output_naming_them_and_fail() {
    use std::os::unix::fs::PermissionsExt;
    let temp = fixture();
    let locked = temp.path().join("source/locked.py");
    fs::write(&locked, "x = 1\n").unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read(&locked).is_ok() {
        return; // Permission bits do not restrict this user (for example root).
    }
    let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .arg("--config")
        .arg(temp.path().join("rules.json"))
        .output()
        .unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o644)).unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("locked.py"));
    let document: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(document["metadata"]["backend_inaccurate"], true);
    assert_eq!(document["metadata"]["coverage"]["unread"][0], "locked.py");
}
