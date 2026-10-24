use crate::common;
use common::locmetry;
use serde_json::{json, Value};
use std::{fs, path::Path};

pub(crate) fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join("source")).unwrap();
    fs::write(temp.path().join("source/main.rs"), "fn main() {}\n").unwrap();
    fs::write(temp.path().join("rules.toml"), APP_RS_RULE).unwrap();
    temp
}

pub(crate) const APP_RS_RULE: &str =
    "version = 1\n[[rules]]\nid = \"app\"\nmatch = [\"**/*.rs\"]\npurpose = \"production\"\n";

/// Scan root and configuration of [`fixture`]; the configuration lies outside the root.
pub(crate) fn config_args(temp: &tempfile::TempDir) -> [std::path::PathBuf; 4] {
    [
        "--root".into(),
        temp.path().join("source"),
        "--config".into(),
        temp.path().join("rules.toml"),
    ]
}

pub(crate) fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
    locmetry().current_dir(cwd).args(args).output().unwrap()
}

pub(crate) fn project(files: &[(&str, &str)]) -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("project")).unwrap();
    common::write_all(&temp.path().join("project"), files);
    temp
}

/// Stdout headline of a run.
pub(crate) fn headline(cwd: &Path, args: &[&str]) -> (bool, Value, String) {
    let output = run(cwd, args);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let document = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    (output.status.success(), document, stderr)
}

/// Full report with per-file records, written to stdout through `--output -`.
pub(crate) fn report(cwd: &Path, args: &[&str]) -> (bool, Value, String) {
    let args = [args, &["--output", "-", "--files"]].concat();
    headline(cwd, &args)
}

/// The full report of a project with this `.locmetryrc` and these files.
pub(crate) fn measured(config: &str, files: &[(&str, &str)]) -> Value {
    let files = [&[(".locmetryrc", config)], files].concat();
    let temp = project(&files);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    document
}

/// The record of `path` in a full report with per-file records.
pub(crate) fn file<'a>(document: &'a Value, path: &str) -> &'a Value {
    document["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"] == path)
        .unwrap_or_else(|| panic!("{path} is not in the report"))
}

pub(crate) fn rules_by_path(document: &Value) -> Vec<(String, String)> {
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

pub(crate) const APP_RULE: &str =
    "version = 1\n[[rules]]\nid = \"app\"\nmatch = [\"src/**\"]\npurpose = \"production\"\n";

/// Row positions of `labels` in `text`, to compare orders without pinning layout.
pub(crate) fn positions(text: &str, labels: &[&str]) -> Vec<usize> {
    labels
        .iter()
        .map(|label| text.find(label).unwrap())
        .collect()
}

pub(crate) fn is_ascending(positions: &[usize]) -> bool {
    positions.windows(2).all(|pair| pair[0] < pair[1])
}

#[test]
fn binary_measures_without_any_executables_on_path() {
    let temp = fixture();
    let output = locmetry()
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
    assert_eq!(document["metadata"]["backend"], locmetry::BACKEND);
    assert_eq!(document["metadata"]["backend_inaccurate"], false);
}

#[test]
fn report_file_is_written_verbatim_outside_the_scan_root() {
    let temp = fixture();
    let path = temp.path().join("out/run.v1");
    let output = locmetry()
        .args(config_args(&temp))
        .arg("--output")
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(report["summary"]["source"]["code"], 1);
    let written: Vec<_> = fs::read_dir(temp.path().join("out"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(written, ["run.v1"], "a run writes exactly one file");
}

#[test]
fn output_in_scan_root_is_rejected_before_writing() {
    let temp = fixture();
    let output = locmetry()
        .args(config_args(&temp))
        .arg("--output")
        .arg(temp.path().join("source/reports/report.json"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!temp.path().join("source/reports").exists());
}

#[test]
fn output_that_would_overwrite_the_configuration_is_rejected() {
    let temp = fixture();
    let output = locmetry()
        .args(config_args(&temp))
        .arg("--output")
        .arg(temp.path().join("rules.toml"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("configuration"));
    assert_eq!(
        fs::read_to_string(temp.path().join("rules.toml")).unwrap(),
        APP_RS_RULE
    );
}

#[test]
fn missing_root_and_invalid_config_fail_without_success_json() {
    let temp = fixture();
    for args in [
        vec!["--config", "absent.toml"],
        vec!["--config", "rules.toml", "--root", "absent"],
    ] {
        let result = run(temp.path(), &args);
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
    }
}

#[test]
fn unusable_roots_and_workspace_files_are_named_as_typed() {
    let temp = fixture();
    for (args, named) in [
        (vec!["--root", "absent"], "'absent'"),
        (vec!["--root", "rules.toml"], "'rules.toml'"),
        (vec!["explain", "x", "--root", "absent"], "'absent'"),
        (
            vec!["workspace", "--workspace-file", "absent.toml"],
            "'absent.toml'",
        ),
    ] {
        let result = run(temp.path(), &args);
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert_eq!(result.status.code(), Some(1), "{args:?}");
        assert!(stderr.contains(named), "{args:?}: {stderr}");
    }
}

/// Severity must survive `NO_COLOR` and agents reading stderr as text, so each
/// message names it as clap's usage errors already do.
#[test]
fn stderr_messages_name_their_severity_like_usage_errors() {
    let temp = fixture();
    for (args, prefix) in [
        (vec!["--bogus"], "error: "),
        (vec!["--root", "absent"], "error: "),
        (
            vec!["--root", "source", "--format", "json", "--show-bars"],
            "warning: ",
        ),
    ] {
        let result = run(temp.path(), &args);
        let stderr = String::from_utf8_lossy(&result.stderr);
        assert!(stderr.starts_with(prefix), "{args:?}: {stderr}");
    }
}

/// Exit code 2 is reserved for partial reports, so usage errors must not use
/// clap's default of 2.
#[test]
fn usage_errors_exit_1_while_help_and_version_succeed() {
    let temp = fixture();
    for args in [vec!["--bogus"], vec!["--hidden", "maybe"]] {
        let result = run(temp.path(), &args);
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
        assert!(!result.stderr.is_empty());
    }
    for args in [vec!["--help"], vec!["--version"]] {
        let result = run(temp.path(), &args);
        assert_eq!(result.status.code(), Some(0));
        assert!(!result.stdout.is_empty());
    }
}

#[test]
fn markdown_format_writes_a_readable_report_to_stdout() {
    let temp = fixture();
    let output = locmetry()
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
    assert!(stdout.starts_with(
        "# source: lines of code by purpose\n\n*Production means maintained non-test source, including separately categorized tooling. Data, configuration, and documentation are counted apart from source. Lines = Code + Comments + Blanks in every table with these columns, and the purpose rows add up to all source.*\n\n## Source purpose"
    ));
    assert!(stdout.contains("| production | 1 | 1 | 1 | 0 | 0 |"));
    // Measurement caveats belong with coverage, not in the opening.
    let coverage = &stdout[stdout.find("## Coverage").unwrap()..];
    assert!(coverage
        .contains("Counts describe the working copy, including untracked files, not a commit."));
    assert!(!stdout.contains("Tokei"));
}

#[test]
fn explicit_table_format_overrides_the_stdout_export_default() {
    let temp = fixture();
    let result = run(
        temp.path(),
        &["--root", "source", "--format", "table", "--output", "-"],
    );
    assert_eq!(result.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&result.stdout).contains("Language"));
}

/// Labelled counts read correctly for any number, including one.
#[test]
fn markdown_coverage_reads_correctly_for_single_files() {
    let temp = project(&[("diagram.png", "not an image\n")]);
    let output = run(&temp.path().join("project"), &["--format", "markdown"]);
    assert!(output.status.success());
    let markdown = String::from_utf8(output.stdout).unwrap();
    assert!(
        markdown.contains("Files selected: 1; counted: 0, including 0 through language mappings; not recognized: 1; recognized but unreadable: 0. Their lines are missing, not zero."),
        "{markdown}"
    );
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
    let output = locmetry().args(config_args(&temp)).output().unwrap();
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

#[test]
fn configuration_is_discovered_in_the_scan_root_from_another_directory() {
    let temp = project(&[(".locmetryrc", APP_RULE), ("src/a.ts", "let a = 1;\n")]);
    let (success, document, stderr) = report(temp.path(), &["--root", "project"]);
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

/// The configuration is measurement policy, not project content, so the loaded
/// file must not appear as a selected file or a coverage gap.
#[test]
fn loaded_configuration_inside_the_scan_root_is_not_measured() {
    let temp = project(&[(".locmetryrc", APP_RULE), ("src/a.ts", "let a = 1;\n")]);
    let (success, document, stderr) = report(temp.path(), &["--root", "project"]);
    assert!(success, "{stderr}");
    let metadata = &document["metadata"];
    assert_eq!(metadata["coverage"]["selected_files"], 1);
    assert_eq!(metadata["coverage"]["unrecognized"], json!([]));
    assert!(metadata["scope"]["exclude"]
        .as_array()
        .unwrap()
        .contains(&json!("**/.locmetryrc")));
}

/// Every `.locmetryrc` is policy, loaded or not, even without default
/// exclusions: one beside an explicit `--config` or in a nested project would
/// otherwise be an unrecognized file, so trying a draft policy would change
/// coverage compared with a discovery run of the same tree.
#[test]
fn unloaded_configurations_are_not_measured() {
    let temp = project(&[
        (".locmetryrc", APP_RULE),
        (
            "draft.toml",
            "version = 1\n[scope]\ndefault_excludes = false\n",
        ),
        ("nested/.locmetryrc", APP_RULE),
        ("src/a.ts", "let a = 1;\n"),
    ]);
    let (success, document, stderr) =
        report(&temp.path().join("project"), &["--config", "draft.toml"]);
    assert!(success, "{stderr}");
    let metadata = &document["metadata"];
    assert_eq!(metadata["coverage"]["selected_files"], 1);
    assert_eq!(metadata["coverage"]["unrecognized"], json!([]));
    assert!(metadata["scope"]["exclude"]
        .as_array()
        .unwrap()
        .contains(&json!("**/.locmetryrc")));
}

/// Glob metacharacters in the configuration's name match literally, so its
/// exclusion cannot hide a sibling that the unescaped pattern would match.
#[test]
fn explicit_configuration_inside_the_scan_root_is_excluded_literally() {
    let temp = project(&[
        ("cfg/lo[c].toml", APP_RULE),
        ("cfg/loc.toml", "answer = 42\n"),
        ("src/a.ts", "let a = 1;\n"),
    ]);
    let (success, document, stderr) = report(
        &temp.path().join("project"),
        &["--config", "cfg/lo[c].toml"],
    );
    assert!(success, "{stderr}");
    let paths: Vec<_> = rules_by_path(&document)
        .into_iter()
        .map(|(path, _)| path)
        .collect();
    assert_eq!(paths, ["cfg/loc.toml", "src/a.ts"]);
}

#[test]
fn explicit_configuration_does_not_change_the_scan_root() {
    let temp = project(&[("src/a.ts", "let a = 1;\n")]);
    fs::write(temp.path().join("shared.toml"), APP_RULE).unwrap();
    fs::write(temp.path().join("outside.ts"), "let b = 1;\n").unwrap();
    let (success, document, stderr) = report(
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
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
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
    let (success, document, stderr) = report(
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
            "**/.lake/**",
            "gen/**",
            "vendor/**",
            "**/.locmetryrc"
        ])
    );
}

#[test]
fn invalid_discovered_configuration_fails_before_scanning() {
    let temp = project(&[
        (".locmetryrc", "version = 2\n"),
        ("src/a.ts", "let a = 1;\n"),
    ]);
    let (success, document, stderr) = headline(&temp.path().join("project"), &[]);
    assert!(!success);
    assert_eq!(document, Value::Null);
    assert!(
        stderr.contains(".locmetryrc") && stderr.contains("Unsupported configuration version 2"),
        "{stderr}"
    );
}

#[test]
fn stdout_headline_keeps_compact_evidence_without_a_report_pointer() {
    let temp = project(&[
        ("src/a.rs", "fn a() {}\n"),
        ("tools/gen.sh", "echo hi\n"),
        ("assets/logo.xyz", "x\n"),
        ("assets/icon.xyz", "x\n"),
    ]);
    let root = temp.path().join("project");
    let path = temp.path().join("report.json");
    let saved = run(&root, &["--output", path.to_str().unwrap()]);
    assert!(saved.status.success());
    assert!(saved.stdout.is_empty());
    let (success, headline, stderr) = headline(&root, &[]);
    assert!(success, "{stderr}");
    assert_eq!(headline["schema_version"], "3");
    assert!(headline.get("report").is_none());
    for absent in ["breakdown", "review", "files"] {
        assert!(headline.get(absent).is_none(), "{absent}");
    }
    assert!(headline["metadata"].get("config").is_none());
    let coverage = &headline["metadata"]["coverage"];
    assert!(coverage.get("unrecognized").is_none());
    assert_eq!(coverage["unrecognized_files"], 2);
    assert_eq!(coverage["unrecognized_extensions"], json!({"xyz": 2}));
    let top = &headline["summary"]["top_folders"];
    assert_eq!(top["src"]["other"], 1);
    assert_eq!(top["tools"]["other"], 1);
    assert!(headline["metadata"].get("kinds").is_none());
    let full: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(full["summary"], headline["summary"]);
    assert_eq!(
        full["metadata"]["coverage"]["unrecognized"],
        json!(["assets/icon.xyz", "assets/logo.xyz"])
    );
}

#[test]
fn report_lists_one_breakdown_row_per_line_and_records_only_on_request() {
    let temp = project(&[
        ("src/a.rs", "fn a() {}\n"),
        (
            "web/index.html",
            "<p>x</p>\n<script>\nlet x = 1;\n</script>\n",
        ),
    ]);
    let root = temp.path().join("project");
    let output = run(&root, &["--output", "-"]);
    let text = String::from_utf8(output.stdout).unwrap();
    let report: Value = serde_json::from_str(&text).unwrap();
    assert!(report.get("files").is_none(), "records only with --files");
    let breakdown = report["breakdown"].as_array().unwrap();
    for row in breakdown {
        let line = serde_json::to_string(row).unwrap();
        assert!(
            text.lines()
                .any(|text_line| text_line.trim_end_matches(',') == line),
            "row not on its own line: {line}"
        );
    }
    let row = |language: &str| {
        breakdown
            .iter()
            .find(|row| row["folder"] == "web" && row["language"] == language)
            .unwrap()
    };
    assert_eq!(row("HTML")["files"], 1);
    assert_eq!(
        (
            row("JavaScript")["files"].clone(),
            row("JavaScript")["code"].clone()
        ),
        (json!(0), json!(1)),
        "embedded languages add lines, not files"
    );
    assert_eq!(
        report["review"],
        json!([
            {"path":"web/index.html","language":"HTML","category":"unclaimed","rule":"fallback","code":4,"lines":4},
            {"path":"src/a.rs","language":"Rust","category":"unclaimed","rule":"fallback","code":1,"lines":1}
        ])
    );
    let (_, detailed, _) = report_with_files(&root);
    let record = |path: &str| {
        detailed["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|file| file["path"] == path)
            .unwrap()
            .clone()
    };
    let rust = record("src/a.rs");
    assert!(rust.get("languages").is_none());
    assert!(rust["counts"].get("files").is_none());
    assert_eq!(
        record("web/index.html")["languages"]["JavaScript"]["code"],
        1
    );
}

fn report_with_files(root: &Path) -> (bool, Value, String) {
    report(root, &[])
}

#[test]
fn markdown_folder_depth_is_configurable_while_json_keeps_every_folder() {
    let temp = project(&[("a/b/c/deep.rs", "fn deep() {}\n")]);
    let root = temp.path().join("project");
    // Only the folder breakdown has a depth; the review lists folders at any depth.
    let folder_table = |extra: &[&str]| {
        let args = [&["--format", "markdown"], extra].concat();
        let markdown = String::from_utf8(run(&root, &args).stdout).unwrap();
        let start = markdown.find("## Folder breakdown").unwrap();
        let end = markdown.find("## Source requiring review").unwrap();
        markdown[start..end].to_string()
    };
    assert!(!folder_table(&[]).contains("| a/b/c |"));
    assert!(folder_table(&["--markdown-depth", "3"]).contains("| a/b/c |"));
    let (_, document, _) = report(&root, &[]);
    assert_eq!(document["breakdown"][0]["folder"], "a/b/c");
}

#[test]
fn empty_roots_report_zero_counts_successfully() {
    let temp = project(&[]);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    assert_eq!(document["metadata"]["coverage"]["selected_files"], 0);
    assert_eq!(document["summary"]["all_counted"]["files"], 0);
    assert_eq!(document["breakdown"], json!([]));
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
    let (_, document, _) = headline(&root, &[]);
    let markdown = String::from_utf8(run(&root, &["--format", "markdown"]).stdout).unwrap();
    for purpose in ["production", "test", "other"] {
        // Absent purposes are zero in JSON; Markdown lists every purpose.
        let counts = &document["summary"]["source_purpose"][purpose];
        let count = |metric: &str| counts[metric].as_u64().unwrap_or(0);
        let row = format!(
            "| {purpose} | {} | {} | {} | {} | {} |",
            count("files"),
            count("lines"),
            count("code"),
            count("comments"),
            count("blanks")
        );
        assert!(markdown.contains(&row), "missing {row}");
    }
}

#[test]
fn breakdown_rows_add_up_to_the_summary() {
    let temp = project(&[
        ("src/a.ts", "let a = 1;\n// note\n\nlet b = 2;\n"),
        ("src/deep/b.test.ts", "let c = 1;\n"),
        ("web/index.html", "<script>\nlet x = 1;\n</script>\n"),
        ("README.md", "# Title\n"),
        ("data/x.json", "{}\n"),
    ]);
    let (_, document, _) = report(&temp.path().join("project"), &[]);
    let rows = document["breakdown"].as_array().unwrap();
    for metric in ["files", "code", "comments", "blanks", "lines"] {
        let total: u64 = rows.iter().map(|row| row[metric].as_u64().unwrap()).sum();
        assert_eq!(
            total, document["summary"]["all_counted"][metric],
            "{metric}"
        );
        let source: u64 = rows
            .iter()
            .filter(|row| row["kind"] == "source")
            .map(|row| row[metric].as_u64().unwrap())
            .sum();
        assert_eq!(source, document["summary"]["source"][metric], "{metric}");
    }
    let src: u64 = rows
        .iter()
        .filter(|row| {
            let folder = row["folder"].as_str().unwrap();
            folder == "src" || folder.starts_with("src/")
        })
        .map(|row| row["code"].as_u64().unwrap())
        .sum();
    let top = &document["summary"]["top_folders"]["src"];
    let top_code: u64 = ["production", "test", "other"]
        .iter()
        .map(|purpose| top[purpose].as_u64().unwrap_or(0))
        .sum();
    assert_eq!(src, top_code, "recursive folder totals are prefix sums");
}

#[test]
fn report_metadata_records_the_effective_policy() {
    let temp = project(&[(".locmetryrc", APP_RULE), ("src/a.ts", "let a = 1;\n")]);
    let (_, document, _) = report(&temp.path().join("project"), &["--hidden", "false"]);
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
    assert_eq!(metadata["backend"], locmetry::BACKEND);
}

#[test]
fn measurement_help_explains_json_headline_and_full_report() {
    for args in [vec!["--help"], vec!["workspace", "--help"]] {
        let output = locmetry().args(args).output().unwrap();
        assert!(output.status.success());
        let help = String::from_utf8(output.stdout).unwrap();
        let help = help.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(help.contains("a JSON headline in pipes"), "{help}");
        assert!(
            help.contains("--output FILE saves only to that file, defaulting to full JSON"),
            "{help}"
        );
    }
}

#[test]
fn positional_directory_matches_explicit_root_and_discovers_its_config() {
    let temp = fixture();
    fs::write(temp.path().join("source/.locmetryrc"), APP_RS_RULE).unwrap();
    let report = |args: &[&str]| {
        let output = run(
            temp.path(),
            &[&["--files", "--output", "-"][..], args].concat(),
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut value: Value = serde_json::from_slice(&output.stdout).unwrap();
        value["metadata"]
            .as_object_mut()
            .unwrap()
            .remove("measured_at_unix_seconds");
        value
    };
    let expected = report(&["--root", "source"]);
    assert_eq!(expected["files"][0]["purpose"], "production");
    let absolute = temp.path().join("source");
    for args in [
        vec!["source"],
        vec!["source", "--format", "json"],
        vec!["--format", "json", "source"],
        vec!["--", "source"],
    ] {
        assert_eq!(report(&args), expected, "{args:?}");
    }
    assert_eq!(
        report(&[absolute.to_str().unwrap()]),
        report(&["--root", absolute.to_str().unwrap()])
    );
    fs::rename(temp.path().join("source"), temp.path().join("source tree")).unwrap();
    assert_eq!(report(&["source tree"]), report(&["--root", "source tree"]));
}

#[test]
fn positional_directory_disambiguates_command_names_and_rejects_multiple_roots() {
    let temp = fixture();
    for name in ["compare", "explain", "workspace", "help"] {
        fs::create_dir(temp.path().join(name)).unwrap();
        fs::write(temp.path().join(name).join("main.py"), "print(1)\n").unwrap();
        for args in [vec![format!("./{name}")], vec!["--".into(), name.into()]] {
            let args: Vec<_> = args.iter().map(String::as_str).collect();
            let (ok, report, err) = headline(temp.path(), &args);
            assert!(ok, "{err}");
            assert_eq!(report["summary"]["source"]["code"], 1);
        }
        let args = match name {
            "help" => vec!["help"],
            _ => vec![name, "--help"],
        };
        let output = run(temp.path(), &args);
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("Usage:"));
    }
    for args in [
        vec!["source", "--root", "source"],
        vec!["--root", "source", "source"],
        vec!["source", "other"],
        vec!["source", "workspace", "--root", "other"],
    ] {
        let output = run(temp.path(), &args);
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn root_options_do_not_turn_subcommand_names_into_directories() {
    let temp = fixture();
    for name in ["compare", "workspace", "explain"] {
        fs::create_dir(temp.path().join(name)).unwrap();
        fs::write(temp.path().join(name).join("main.py"), "print(1)\n").unwrap();
        let output = run(temp.path(), &["--files", name]);
        assert_eq!(output.status.code(), Some(1), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
        assert!(String::from_utf8_lossy(&output.stderr).contains(name));
        let (ok, value, err) = headline(temp.path(), &["--root", name]);
        assert!(ok, "{err}");
        assert_eq!(value["summary"]["source"]["code"], 1);
    }
}
