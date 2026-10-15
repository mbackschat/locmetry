use serde_json::{json, Value};
use std::{fs, process::Command};

fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join("source")).unwrap();
    fs::write(temp.path().join("source/main.rs"), "fn main() {}\n").unwrap();
    fs::write(temp.path().join("rules.toml"), APP_RS_RULE).unwrap();
    temp
}

const APP_RS_RULE: &str =
    "version = 1\n[[rules]]\nid = \"app\"\nmatch = [\"**/*.rs\"]\npurpose = \"production\"\n";

/// Scan root and configuration of [`fixture`]; the configuration lies outside the root.
fn config_args(temp: &tempfile::TempDir) -> [std::path::PathBuf; 4] {
    [
        "--root".into(),
        temp.path().join("source"),
        "--config".into(),
        temp.path().join("rules.toml"),
    ]
}

#[test]
fn binary_measures_without_any_executables_on_path() {
    let temp = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .env("PATH", "")
        .args(config_args(&temp))
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
        .args(config_args(&temp))
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
        .args(config_args(&temp))
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
        vec!["--config", "absent.toml"],
        vec!["--config", "rules.toml", "--root", "absent"],
    ] {
        let result = Command::new(env!("CARGO_BIN_EXE_locmetry"))
            .current_dir(temp.path())
            .args(args)
            .output()
            .unwrap();
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
    }
}

fn run_with_output(temp: &tempfile::TempDir, prefix: &std::path::Path) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .args(config_args(temp))
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
    // Any file name can hold a TOML configuration, including a report suffix.
    let config = temp.path().join("rules.md");
    fs::write(&config, APP_RS_RULE).unwrap();
    let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .arg("--root")
        .arg(temp.path().join("source"))
        .arg("--config")
        .arg(&config)
        .arg("--output")
        .arg(temp.path().join("rules"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("configuration"));
    assert_eq!(fs::read_to_string(&config).unwrap(), APP_RS_RULE);
    assert!(!temp.path().join("rules.json").exists());
}

#[test]
fn markdown_format_writes_a_readable_report_to_stdout() {
    let temp = fixture();
    let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .args(["--format", "markdown"])
        .args(config_args(&temp))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with("# source measured classification"));
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
        .args(config_args(&temp))
        .output()
        .unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(
        output.status.code(),
        Some(2),
        "partial results use exit code 2"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("locked.py"));
    let document: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(document["metadata"]["backend_inaccurate"], true);
    assert_eq!(document["metadata"]["coverage"]["unread"][0], "locked.py");
}

fn project(files: &[(&str, &str)]) -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    for (path, content) in files {
        let path = temp.path().join("project").join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
    temp
}

fn measure(cwd: &std::path::Path, args: &[&str]) -> (bool, Value, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let document = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    (output.status.success(), document, stderr)
}

fn rules_by_path(document: &Value) -> Vec<(String, String)> {
    document["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            (
                file["path"].as_str().unwrap().into(),
                file["rule"].as_str().unwrap().into(),
            )
        })
        .collect()
}

const APP_RULE: &str =
    "version = 1\n[[rules]]\nid = \"app\"\nmatch = [\"src/**\"]\npurpose = \"production\"\n";

#[test]
fn configuration_is_discovered_in_the_scan_root_from_another_directory() {
    let temp = project(&[(".locmetryrc", APP_RULE), ("src/a.ts", "let a = 1;\n")]);
    let (success, document, stderr) = measure(temp.path(), &["--root", "project"]);
    assert!(success, "{stderr}");
    assert_eq!(
        rules_by_path(&document),
        [("src/a.ts".into(), "app".into())]
    );
    assert_eq!(document["project"], "project");
    assert!(document["metadata"]["config_path"]
        .as_str()
        .unwrap()
        .ends_with(".locmetryrc"));
}

#[test]
fn explicit_configuration_does_not_change_the_scan_root() {
    let temp = project(&[("src/a.ts", "let a = 1;\n")]);
    fs::write(temp.path().join("shared.toml"), APP_RULE).unwrap();
    fs::write(temp.path().join("outside.ts"), "let b = 1;\n").unwrap();
    let (success, document, stderr) = measure(
        &temp.path().join("project"),
        &["--config", "../shared.toml"],
    );
    assert!(success, "{stderr}");
    assert_eq!(
        rules_by_path(&document),
        [("src/a.ts".into(), "app".into())]
    );
}

#[test]
fn without_configuration_built_in_defaults_apply() {
    let temp = project(&[
        ("src/a.ts", "let a = 1;\n"),
        ("src/a.test.ts", "let b = 1;\n"),
        ("node_modules/x/index.js", "let c = 1;\n"),
    ]);
    let (success, document, stderr) = measure(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    assert_eq!(
        rules_by_path(&document),
        [
            ("src/a.test.ts".into(), "builtin:js-test-file".into()),
            ("src/a.ts".into(), "fallback".into()),
        ]
    );
    assert_eq!(document["metadata"]["config_path"], Value::Null);
}

#[test]
fn cli_scope_settings_override_the_project_configuration() {
    let config = format!("{APP_RULE}[scope]\nhidden = true\nexclude = [\"gen/**\"]\n");
    let temp = project(&[
        (".locmetryrc", config.as_str()),
        ("src/a.ts", "let a = 1;\n"),
        ("gen/a.ts", "let b = 1;\n"),
        ("vendor/a.ts", "let c = 1;\n"),
        (".tool/a.ts", "let d = 1;\n"),
    ]);
    let (success, document, stderr) = measure(
        &temp.path().join("project"),
        &["--hidden", "false", "--exclude", "vendor/**"],
    );
    assert!(success, "{stderr}");
    assert_eq!(
        rules_by_path(&document),
        [("src/a.ts".into(), "app".into())]
    );
    assert_eq!(
        document["metadata"]["scope"]["exclude"],
        json!([
            "**/.git/**",
            "**/node_modules/**",
            "**/.build/**",
            "gen/**",
            "vendor/**"
        ])
    );
}

#[test]
fn invalid_discovered_configuration_fails_before_scanning() {
    let temp = project(&[
        (".locmetryrc", "version = 2\n"),
        ("src/a.ts", "let a = 1;\n"),
    ]);
    let (success, document, stderr) = measure(&temp.path().join("project"), &[]);
    assert!(!success);
    assert_eq!(document, Value::Null);
    assert!(
        stderr.contains(".locmetryrc") && stderr.contains("Unsupported configuration version 2"),
        "{stderr}"
    );
}

#[test]
fn stdout_with_report_files_is_the_report_without_records_plus_their_paths() {
    let temp = fixture();
    let prefix = temp.path().join("out/report");
    let output = run_with_output(&temp, &prefix);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let headline: Value = serde_json::from_slice(&output.stdout).unwrap();
    let full: Value =
        serde_json::from_slice(&fs::read(temp.path().join("out/report.json")).unwrap()).unwrap();
    assert_eq!(headline["schema_version"], "1");
    assert_eq!(headline["metadata"], full["metadata"]);
    assert_eq!(headline["summary"]["source"], full["summary"]["source"]);
    assert!(headline.get("files").is_none());
    assert!(headline["summary"].get("folders").is_none());
    for (key, suffix) in [
        ("json", "json"),
        ("markdown", "md"),
        ("tokei", "tokei.json"),
    ] {
        let path = headline["reports"][key].as_str().unwrap();
        assert!(
            path.ends_with(&format!("report.{suffix}")) && std::path::Path::new(path).exists(),
            "{path}"
        );
    }
}

#[test]
fn markdown_folder_depth_is_configurable_while_json_keeps_every_folder() {
    let temp = project(&[("a/b/c/deep.rs", "fn deep() {}\n")]);
    let root = temp.path().join("project");
    let markdown = |extra: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
            .current_dir(&root)
            .args(["--format", "markdown"])
            .args(extra)
            .output()
            .unwrap();
        String::from_utf8(output.stdout).unwrap()
    };
    assert!(!markdown(&[]).contains("| a/b/c |"));
    assert!(markdown(&["--depth", "3"]).contains("| a/b/c |"));
    let (_, document, _) = measure(&root, &[]);
    assert!(document["summary"]["folders"].get("a/b/c").is_some());
}

#[test]
fn empty_roots_report_zero_counts_successfully() {
    let temp = project(&[]);
    fs::create_dir_all(temp.path().join("project")).unwrap();
    let (success, document, stderr) = measure(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    assert_eq!(document["metadata"]["coverage"]["selected_files"], 0);
    assert_eq!(document["summary"]["all_counted"]["files"], 0);
    assert_eq!(document["files"], json!([]));
}

#[test]
fn markdown_and_json_report_the_same_totals() {
    let temp = project(&[
        ("src/a.ts", "let a = 1;\n// note\n\nlet b = 2;\n"),
        ("src/a.test.ts", "let c = 1;\n"),
        ("gradlew", "#!/bin/sh\necho hi\n"),
    ]);
    let root = temp.path().join("project");
    let (_, document, _) = measure(&root, &[]);
    let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .current_dir(&root)
        .args(["--format", "markdown"])
        .output()
        .unwrap();
    let markdown = String::from_utf8(output.stdout).unwrap();
    for purpose in ["production", "test", "unclassified"] {
        // Absent purposes are zero in JSON; Markdown lists every purpose.
        let counts = &document["summary"]["source_purpose"][purpose];
        let count = |metric: &str| counts[metric].as_u64().unwrap_or(0);
        let row = format!(
            "| {purpose} | {} | {} | {} | {} | {} |",
            count("files"),
            count("code"),
            count("comments"),
            count("blanks"),
            count("lines")
        );
        assert!(markdown.contains(&row), "missing {row}");
    }
}

#[test]
fn metadata_records_the_effective_policy() {
    let temp = project(&[(".locmetryrc", APP_RULE), ("src/a.ts", "let a = 1;\n")]);
    let (_, document, _) = measure(&temp.path().join("project"), &["--hidden", "false"]);
    let metadata = &document["metadata"];
    assert_eq!(metadata["scope"]["hidden"], false);
    assert_eq!(metadata["config"]["rules"][0]["id"], "app");
    let conventions = metadata["config"]["builtin_conventions"]
        .as_array()
        .unwrap();
    assert!(conventions
        .iter()
        .any(|c| c["id"] == "builtin:build-wrapper"));
    assert!(!metadata["config"]["builtin_language_mappings"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(metadata["backend"], "tokei 15.0.0 (embedded)");
}

#[test]
fn reports_are_compact_and_omit_redundant_or_zero_counts() {
    let temp = project(&[
        ("src/a.rs", "fn a() {}\n"),
        (
            "web/index.html",
            "<p>x</p>\n<script>\nlet x = 1;\n</script>\n",
        ),
    ]);
    let root = temp.path().join("project");
    let prefix = temp.path().join("out/report");
    let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .current_dir(&root)
        .arg("--output")
        .arg(&prefix)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = fs::read_to_string(temp.path().join("out/report.json")).unwrap();
    assert_eq!(text.lines().count(), 1, "report JSON is not pretty-printed");
    assert_eq!(String::from_utf8(output.stdout).unwrap().lines().count(), 1);
    let report: Value = serde_json::from_str(&text).unwrap();
    let record = |path: &str| {
        report["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|file| file["path"] == path)
            .unwrap()
            .clone()
    };
    let rust = record("src/a.rs");
    assert!(
        rust.get("languages").is_none(),
        "single-language records repeat no counts"
    );
    assert!(rust["counts"].get("files").is_none());
    assert_eq!(rust["counts"]["code"], 1);
    let html = record("web/index.html");
    assert_eq!(html["languages"]["JavaScript"]["code"], 1);
    assert!(html["languages"]["JavaScript"].get("files").is_none());
    let folder = &report["summary"]["folders"]["src"]["direct"];
    assert!(
        folder["source_purpose"].get("test").is_none(),
        "zero entries are omitted"
    );
    assert_eq!(folder["source_purpose"]["unclassified"]["code"], 1);
    let web = &report["summary"]["folders"]["web"]["direct"];
    assert_eq!(web["source_languages"]["JavaScript"]["code"], 1);
}
