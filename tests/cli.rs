use serde_json::{json, Value};
use std::{fs, path::Path, process::Command};

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

/// The binary isolated from the developer's own user defaults.
fn locmetry() -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_locmetry"));
    command.env("XDG_CONFIG_HOME", "/nonexistent/locmetry-tests");
    command
}

fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
    locmetry().current_dir(cwd).args(args).output().unwrap()
}

fn project(files: &[(&str, &str)]) -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("project")).unwrap();
    for (path, content) in files {
        let path = temp.path().join("project").join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
    temp
}

/// Stdout headline of a run.
fn headline(cwd: &Path, args: &[&str]) -> (bool, Value, String) {
    let output = run(cwd, args);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let document = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    (output.status.success(), document, stderr)
}

/// Full report with per-file records, written to stdout through `--output -`.
fn report(cwd: &Path, args: &[&str]) -> (bool, Value, String) {
    let args = [args, &["--output", "-", "--files"]].concat();
    headline(cwd, &args)
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
    assert_eq!(document["metadata"]["backend"], "tokei 15.0.0 (embedded)");
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
    assert!(stdout.starts_with("# source measured classification"));
    assert!(stdout.contains("| production | 1 | 1 | 0 | 0 | 1 |"));
}

#[cfg(unix)]
/// Locks the terminal layout: languages and top-level folders sorted by code,
/// then one line each for non-source content, review, and coverage gaps.
#[test]
fn table_format_splits_source_code_by_language_and_purpose() {
    let temp = project(&[
        (".locmetryrc", APP_RULE),
        ("src/lib.rs", "fn a() {}\n"),
        ("src/main.rs", "// entry\nfn main() {}\n"),
        ("tests/it.rs", "#[test]\nfn t() {}\n"),
        ("proto/x.py", "x = 1\n"),
        ("config.json", "{}\n"),
        ("README.md", "# Title\n"),
        ("logo.png", "not an image\n"),
    ]);
    let output = run(&temp.path().join("project"), &["--format", "table"]);
    assert!(output.status.success());
    let expected = "
 project: source code lines by purpose

 Language  Files  Production  Test  Unclassified  Code  Comments  Blanks
 ───────────────────────────────────────────────────────────────────────
 Rust          3           2     2             0     4         1       0
 Python        1           0     0             1     1         0       0
 ───────────────────────────────────────────────────────────────────────
 Total         4           2     2             1     5         1       0

 Folder  Production  Test  Unclassified  Code
 ────────────────────────────────────────────
 src              2     0             0     2
 tests            0     2             0     2
 proto            0     0             1     1

 Not source code: 1 data file (1 line), 1 documentation file (1 line).
 Review: 1 unclassified source file (1 code line); details with --format markdown.
 Not counted: 1 unrecognized file (png: 1); missing lines are not zero.
";
    assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
}

/// `--output -` prints the JSON report, so a human format cannot share stdout.
#[test]
fn table_format_cannot_share_stdout_with_the_json_report() {
    let temp = fixture();
    let result = run(
        temp.path(),
        &["--root", "source", "--format", "table", "--output", "-"],
    );
    assert_eq!(result.status.code(), Some(1));
    assert!(result.stdout.is_empty());
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

/// Runs with user defaults read from `home`, the `XDG_CONFIG_HOME` directory.
fn report_with_user_defaults(cwd: &Path, home: &Path, args: &[&str]) -> (bool, Value, String) {
    let args = [args, &["--output", "-", "--files"]].concat();
    let output = locmetry()
        .current_dir(cwd)
        .env("XDG_CONFIG_HOME", home)
        .args(&args)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let document = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    (output.status.success(), document, stderr)
}

fn user_defaults(temp: &tempfile::TempDir, content: &str) -> std::path::PathBuf {
    let home = temp.path().join("home");
    fs::create_dir_all(home.join("locmetry")).unwrap();
    fs::write(home.join("locmetry/config.toml"), content).unwrap();
    home
}

const USER_DEFAULTS: &str = "version = 1
[scope]
exclude = [\"**/*.generated.ts\"]
[[languages.mappings]]
id = \"tasks\"
match = [\"**/*.runbook\"]
language = \"BASH\"
";

fn languages_by_path(document: &Value) -> Vec<(String, String)> {
    document["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            (
                file["path"].as_str().unwrap().into(),
                file["language"].as_str().unwrap().into(),
            )
        })
        .collect()
}

/// User exclusions sit between the defaults and the project's, and user
/// mappings count files the backend does not recognize.
#[test]
fn user_defaults_add_exclusions_and_language_mappings() {
    let config = format!("{APP_RULE}[scope]\nexclude = [\"vendor/**\"]\n");
    let temp = project(&[
        (".locmetryrc", config.as_str()),
        ("src/a.ts", "let a = 1;\n"),
        ("src/b.generated.ts", "let b = 1;\n"),
        ("vendor/c.ts", "let c = 1;\n"),
        ("scripts/build.runbook", "echo hi\n"),
    ]);
    let home = user_defaults(&temp, USER_DEFAULTS);
    let (success, document, stderr) =
        report_with_user_defaults(&temp.path().join("project"), &home, &[]);
    assert!(success, "{stderr}");
    assert_eq!(
        languages_by_path(&document),
        [
            ("scripts/build.runbook".into(), "BASH".into()),
            ("src/a.ts".into(), "TypeScript".into()),
        ]
    );
    let metadata = &document["metadata"];
    assert_eq!(
        metadata["scope"]["exclude"],
        json!([
            "**/.git/**",
            "**/node_modules/**",
            "**/.build/**",
            "**/*.generated.ts",
            "vendor/**",
            ".locmetryrc"
        ])
    );
    assert!(metadata["user_config_path"]
        .as_str()
        .unwrap()
        .ends_with("locmetry/config.toml"));
}

/// The project knows its files best, so its mappings win over user ones.
#[test]
fn project_language_mappings_take_precedence_over_user_defaults() {
    let config = format!(
        "{APP_RULE}[[languages.mappings]]\nid = \"project-tasks\"\nmatch = [\"**/*.runbook\"]\nlanguage = \"Python\"\n"
    );
    let temp = project(&[
        (".locmetryrc", config.as_str()),
        ("build.runbook", "x = 1\n"),
    ]);
    let home = user_defaults(&temp, USER_DEFAULTS);
    let (success, document, stderr) =
        report_with_user_defaults(&temp.path().join("project"), &home, &[]);
    assert!(success, "{stderr}");
    assert_eq!(
        languages_by_path(&document),
        [("build.runbook".into(), "Python".into())]
    );
}

#[test]
fn no_user_config_ignores_user_defaults() {
    let temp = project(&[("src/b.generated.ts", "let b = 1;\n")]);
    let home = user_defaults(&temp, USER_DEFAULTS);
    let (success, document, stderr) =
        report_with_user_defaults(&temp.path().join("project"), &home, &["--no-user-config"]);
    assert!(success, "{stderr}");
    assert_eq!(document["files"][0]["path"], "src/b.generated.ts");
    assert_eq!(document["metadata"]["user_config_path"], Value::Null);
}

/// User defaults may only narrow scope and teach languages; anything that
/// would classify, widen scope, or collide with project IDs fails the run.
#[test]
fn user_defaults_reject_classification_scope_widening_and_id_clashes() {
    let clash = "version = 1\n[[languages.mappings]]\nid = \"tasks\"\nmatch = [\"*.x\"]\nlanguage = \"BASH\"\n";
    for (user, project_config) in [
        (APP_RULE.to_string(), APP_RULE),
        (
            "version = 1\n[scope]\nhidden = false\n".to_string(),
            APP_RULE,
        ),
        (
            "version = 1\n[classification]\nfallback = \"test\"\n".to_string(),
            APP_RULE,
        ),
        ("version = 2\n".to_string(), APP_RULE),
        (USER_DEFAULTS.to_string(), clash),
    ] {
        let temp = project(&[(".locmetryrc", project_config), ("a.ts", "let a = 1;\n")]);
        let home = user_defaults(&temp, &user);
        let (success, _, stderr) =
            report_with_user_defaults(&temp.path().join("project"), &home, &[]);
        assert!(!success, "accepted user defaults:\n{user}");
        assert!(stderr.contains("config.toml"), "{stderr}");
    }
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
        .contains(&json!(".locmetryrc")));
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
            "gen/**",
            "vendor/**",
            ".locmetryrc"
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
fn stdout_is_a_small_headline_that_names_the_report_file() {
    let temp = project(&[
        ("src/a.rs", "fn a() {}\n"),
        ("tools/gen.sh", "echo hi\n"),
        ("assets/logo.xyz", "x\n"),
        ("assets/icon.xyz", "x\n"),
    ]);
    let root = temp.path().join("project");
    let path = temp.path().join("report.json");
    let (success, headline, stderr) = headline(&root, &["--output", path.to_str().unwrap()]);
    assert!(success, "{stderr}");
    assert_eq!(headline["schema_version"], "1");
    assert_eq!(headline["report"], path.to_str().unwrap());
    for absent in ["breakdown", "review", "files"] {
        assert!(headline.get(absent).is_none(), "{absent}");
    }
    assert!(headline["metadata"].get("config").is_none());
    let coverage = &headline["metadata"]["coverage"];
    assert!(coverage.get("unrecognized").is_none());
    assert_eq!(coverage["unrecognized_files"], 2);
    assert_eq!(coverage["unrecognized_extensions"], json!({"xyz": 2}));
    let top = &headline["summary"]["top_folders"];
    assert_eq!(top["src"]["unclassified"], 1);
    assert_eq!(top["tools"]["unclassified"], 1);
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
            {"path":"web/index.html","language":"HTML","category":"unclassified","rule":"fallback","code":4,"lines":4},
            {"path":"src/a.rs","language":"Rust","category":"unclassified","rule":"fallback","code":1,"lines":1}
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
    let markdown = |extra: &[&str]| {
        let args = [&["--format", "markdown"], extra].concat();
        String::from_utf8(run(&root, &args).stdout).unwrap()
    };
    assert!(!markdown(&[]).contains("| a/b/c |"));
    assert!(markdown(&["--depth", "3"]).contains("| a/b/c |"));
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
    let top_code: u64 = ["production", "test", "unclassified"]
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
    assert_eq!(metadata["backend"], "tokei 15.0.0 (embedded)");
}
