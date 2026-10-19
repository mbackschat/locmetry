use crate::common;
use serde_json::Value;
use std::{fs, path::Path, process::Output};

fn run(cwd: &Path, args: &[&str]) -> Output {
    common::locmetry()
        .current_dir(cwd)
        .env("XDG_CONFIG_HOME", cwd.join("no-user-config"))
        .args(args)
        .output()
        .unwrap()
}

fn fixture() -> tempfile::TempDir {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("service/src/deep")).unwrap();
    fs::write(dir.path().join("service/src/deep/main.py"), "print(1)\n").unwrap();
    fs::write(
        dir.path().join("workspace.toml"),
        "version = 1\nroots = [\"service\"]\n",
    )
    .unwrap();
    dir
}

fn success(out: &Output) {
    assert_eq!(
        out.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).unwrap()
}

fn without_time(mut value: Value) -> Value {
    value["metadata"]
        .as_object_mut()
        .unwrap()
        .remove("measured_at_unix_seconds");
    value
}

fn baseline(dir: &Path) {
    success(&run(dir, &["service", "--output", "before.json"]));
}

#[test]
fn a_single_measurement_table_omits_its_title_even_with_empty_optional_tables() {
    let dir = fixture();
    for extra in [
        vec![],
        vec!["--verbose"],
        vec!["--purpose", "production", "--folders", "--categories"],
        vec![
            "--purpose",
            "production",
            "--folder-rows",
            "5",
            "--categories",
        ],
    ] {
        let mut args = vec!["service", "--format", "table", "--no-bars"];
        args.extend(extra);
        let out = run(dir.path(), &args);
        success(&out);
        let text = String::from_utf8(out.stdout).unwrap();
        let lines: Vec<_> = text.lines().collect();
        assert!(lines[1].starts_with(" Language "), "{text}");
        assert_eq!(lines[0], lines[2].replace('┼', "─"), "{text}");
        assert!(!text.contains('▸'), "{text}");
    }
}

#[test]
fn a_single_table_top_rule_spans_inline_bars_but_not_a_separate_chart() {
    let dir = fixture();
    for width in [40, 160] {
        let out = common::locmetry()
            .current_dir(dir.path())
            .args(["service", "--format", "table", "--bars"])
            .env("COLUMNS", width.to_string())
            .output()
            .unwrap();
        success(&out);
        let text = String::from_utf8(out.stdout).unwrap();
        let lines: Vec<_> = text.lines().collect();
        assert_eq!(lines[0], lines[2].replace('┼', "─"), "{text}");
        assert_eq!(lines[1].contains("Code share"), width == 160, "{text}");
        assert!(!text.contains('▸'), "{text}");
    }
}

#[test]
fn measurement_tables_use_attached_top_rules_instead_of_redundant_titles() {
    let dir = fixture();
    for (args, labels) in [
        (vec!["service", "--folders"], vec!["Folder"]),
        (vec!["service", "--categories"], vec!["Category"]),
        (vec!["service", "--folder-rows", "5"], vec!["Folder"]),
        (
            vec![
                "service",
                "--subfolder",
                "src",
                "--folder-rows",
                "5",
                "--categories",
            ],
            vec!["Folder", "Category"],
        ),
        (
            vec!["service", "--purpose", "other", "--folders"],
            vec!["Folder"],
        ),
        (
            vec!["service", "--folders", "--categories"],
            vec!["Folder", "Category"],
        ),
        (
            vec!["workspace", "--file", "workspace.toml"],
            vec!["Repository"],
        ),
        (
            vec!["workspace", "--file", "workspace.toml", "--categories"],
            vec!["Repository", "Category"],
        ),
    ] {
        for (bars, width) in [
            ("--no-bars", "80"),
            ("--bars", "40"),
            ("--bars", "200"),
            ("--bars=comments", "200"),
        ] {
            let out = common::locmetry()
                .current_dir(dir.path())
                .env("XDG_CONFIG_HOME", dir.path().join("no-user-config"))
                .env("COLUMNS", width)
                .args(&args)
                .args(["--format", "table", bars])
                .output()
                .unwrap();
            success(&out);
            let table = String::from_utf8(out.stdout).unwrap();
            assert!(!table.contains('▸'), "{table}");
            let lines: Vec<_> = table.lines().collect();
            for label in std::iter::once("Language").chain(labels.iter().copied()) {
                let header = lines
                    .iter()
                    .position(|line| line.starts_with(&format!(" {label} ")))
                    .unwrap();
                assert_eq!(
                    lines[header - 1],
                    lines[header + 1].replace('┼', "─"),
                    "{table}"
                );
                if label != "Language" {
                    assert!(lines[header - 2].is_empty(), "{table}");
                }
            }
            assert!(!table.contains('\x1b'));
        }
    }
}

#[test]
fn file_output_is_silent_and_defaults_to_a_comparable_full_report_in_both_modes() {
    let dir = fixture();
    for args in [
        vec!["service"],
        vec!["workspace", "--file", "workspace.toml"],
    ] {
        let mut save = args.clone();
        save.extend(["--output", "saved.json"]);
        let out = run(dir.path(), &save);
        success(&out);
        assert!(out.stdout.is_empty());
        assert!(out.stderr.is_empty());
        let saved = json(&fs::read(dir.path().join("saved.json")).unwrap());
        assert_eq!(saved["summary"]["source"]["code"], 1);
        assert!(saved.get("review").is_some());
        assert!(saved.get("breakdown").is_some());
        assert!(saved.get("report").is_none());
        let mut stdout = args;
        stdout.extend(["--output", "-"]);
        let out = run(dir.path(), &stdout);
        success(&out);
        assert_eq!(without_time(saved), without_time(json(&out.stdout)));
        success(&run(dir.path(), &["compare", "saved.json", "saved.json"]));
    }
}

#[test]
fn explicit_json_is_full_and_headline_is_compact_in_measurement_and_compare() {
    let dir = fixture();
    baseline(dir.path());
    for args in [
        vec!["service"],
        vec!["workspace", "--file", "workspace.toml"],
        vec!["compare", "before.json", "before.json"],
    ] {
        let mut full_args = args.clone();
        full_args.extend(["--format", "json"]);
        let full = run(dir.path(), &full_args);
        success(&full);
        assert!(json(&full.stdout).get("breakdown").is_some());
        let mut compact_args = args.clone();
        compact_args.extend(["--format", "headline"]);
        let compact = run(dir.path(), &compact_args);
        success(&compact);
        assert!(json(&compact.stdout).get("breakdown").is_none());
        assert!(json(&compact.stdout).get("report").is_none());
        let implicit = run(dir.path(), &args);
        success(&implicit);
        assert!(json(&implicit.stdout).get("breakdown").is_none());
    }
}

#[test]
fn explicit_formats_control_files_and_stdout_for_all_exporting_commands() {
    let dir = fixture();
    baseline(dir.path());
    for args in [
        vec!["service"],
        vec!["workspace", "--file", "workspace.toml"],
        vec!["compare", "before.json", "before.json"],
    ] {
        for format in ["table", "markdown", "json", "headline"] {
            let mut save = args.clone();
            save.extend(["--format", format, "--output", "result.out"]);
            let out = run(dir.path(), &save);
            success(&out);
            assert!(out.stdout.is_empty());
            assert!(out.stderr.is_empty());
            let saved = fs::read(dir.path().join("result.out")).unwrap();
            assert!(!saved.contains(&27));
            let mut stdout = args.clone();
            stdout.extend(["--format", format, "--output", "-"]);
            let out = run(dir.path(), &stdout);
            success(&out);
            match format {
                "json" | "headline" => {
                    let mut saved = json(&saved);
                    let mut printed = json(&out.stdout);
                    if args[0] != "compare" {
                        saved = without_time(saved);
                        printed = without_time(printed);
                    }
                    assert_eq!(saved, printed);
                    assert_eq!(saved.get("breakdown").is_some(), format == "json");
                }
                _ => {
                    if args[0] == "compare" {
                        assert_eq!(saved, out.stdout);
                    } else {
                        let saved_text = String::from_utf8_lossy(&saved);
                        let printed_text = String::from_utf8_lossy(&out.stdout);
                        let (saved_body, saved_date) =
                            saved_text.rsplit_once("\nMeasured at: ").unwrap();
                        let (printed_body, printed_date) =
                            printed_text.rsplit_once("\nMeasured at: ").unwrap();
                        assert!(saved_date.trim_end().ends_with(" UTC"));
                        assert!(printed_date.trim_end().ends_with(" UTC"));
                        assert_eq!(saved_body, printed_body);
                    }
                    if format == "markdown" {
                        assert!(saved.starts_with(b"# "));
                    } else {
                        let label = if args[0] == "compare" {
                            "All counted"
                        } else {
                            "Language"
                        };
                        assert!(String::from_utf8_lossy(&saved).contains(label));
                    }
                }
            }
        }
    }
}

#[test]
fn file_detail_requires_full_json_before_scanning_or_creating_a_file() {
    let dir = fixture();
    for format in [None, Some("headline"), Some("table"), Some("markdown")] {
        let mut args = vec!["missing-root", "--files"];
        if let Some(format) = format {
            args.extend(["--format", format, "--output", "invalid.out"]);
        }
        let out = run(dir.path(), &args);
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stdout.is_empty());
        assert!(String::from_utf8_lossy(&out.stderr).contains("--format json"));
        assert!(!dir.path().join("invalid.out").exists());
    }
    let out = run(
        dir.path(),
        &["service", "--files", "--output", "files.json"],
    );
    success(&out);
    let saved = json(&fs::read(dir.path().join("files.json")).unwrap());
    assert_eq!(saved["files"][0]["path"], "src/deep/main.py");
}

#[test]
fn schema_rejects_output_format_and_file_detail_and_explain_rejects_headline() {
    let dir = fixture();
    for option in [
        vec!["--output", "schema.json"],
        vec!["--format", "json"],
        vec!["--files"],
    ] {
        let mut args = vec!["--schema", "report"];
        args.extend(option);
        let out = run(dir.path(), &args);
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stdout.is_empty());
        assert!(!dir.path().join("schema.json").exists());
    }
    let out = run(
        dir.path(),
        &[
            "explain",
            "src/deep/main.py",
            "--root",
            "service",
            "--format",
            "headline",
        ],
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("headline"));
    let out = run(
        dir.path(),
        &["explain", "src/deep/main.py", "--root", "service"],
    );
    success(&out);
    assert_eq!(json(&out.stdout)["result"]["state"], "counted");
}

#[test]
fn partial_measurement_and_comparison_write_only_the_file_and_preserve_exit_two() {
    let dir = fixture();
    fs::write(dir.path().join("service/bad.ipynb"), "invalid").unwrap();
    for args in [
        vec!["service", "--output", "partial.json"],
        vec![
            "workspace",
            "--file",
            "workspace.toml",
            "--output",
            "workspace-partial.json",
        ],
        vec![
            "compare",
            "partial.json",
            "partial.json",
            "--output",
            "change.json",
        ],
    ] {
        let out = run(dir.path(), &args);
        assert_eq!(out.status.code(), Some(2));
        assert!(out.stdout.is_empty());
        assert!(!out.stderr.is_empty());
        let saved = json(&fs::read(dir.path().join(args.last().unwrap())).unwrap());
        let metadata = if args[0] == "compare" {
            &saved["before"]["metadata"]
        } else {
            &saved["metadata"]
        };
        assert_eq!(metadata["backend_inaccurate"], true);
    }
}

#[test]
fn ineffective_presentation_options_warn_only_when_explicit_in_both_modes() {
    let dir = fixture();
    for args in [
        vec!["service"],
        vec!["workspace", "--file", "workspace.toml"],
    ] {
        let mut base = args.clone();
        base.extend(["--format", "json"]);
        let original = run(dir.path(), &base);
        success(&original);
        assert!(original.stderr.is_empty());
        for option in [
            vec!["--depth", "2"],
            vec!["--categories"],
            vec!["--verbose"],
        ] {
            let mut changed = base.clone();
            changed.extend(&option);
            let out = run(dir.path(), &changed);
            success(&out);
            assert!(String::from_utf8_lossy(&out.stderr).contains(option[0]));
            assert_eq!(
                without_time(json(&original.stdout)),
                without_time(json(&out.stdout))
            );
        }
        for (format, option) in [
            ("table", "--categories"),
            ("table", "--verbose"),
            ("markdown", "--depth"),
        ] {
            let mut effective = args.clone();
            effective.extend(["--format", format, option]);
            if option == "--depth" {
                effective.push("2");
            }
            let out = run(dir.path(), &effective);
            success(&out);
            assert!(out.stderr.is_empty());
        }
    }
}

#[test]
fn adaptive_tables_can_be_saved_but_require_an_explicit_table_format() {
    let dir = fixture();
    let out = run(
        dir.path(),
        &["service", "--folder-rows", "6", "--output", "folders.txt"],
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("--format table"));
    assert!(!dir.path().join("folders.txt").exists());
    let out = run(
        dir.path(),
        &[
            "service",
            "--folder-rows",
            "6",
            "--output",
            "folders.txt",
            "--format",
            "table",
        ],
    );
    success(&out);
    assert!(out.stdout.is_empty());
    assert!(fs::read_to_string(dir.path().join("folders.txt"))
        .unwrap()
        .contains("src/deep"));
}

#[test]
fn saved_headline_comparison_error_explains_how_to_capture_a_full_baseline() {
    let dir = fixture();
    success(&run(
        dir.path(),
        &[
            "service",
            "--format",
            "headline",
            "--output",
            "headline.json",
        ],
    ));
    let out = run(dir.path(), &["compare", "headline.json", "headline.json"]);
    assert_eq!(out.status.code(), Some(1));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("breakdown") && err.contains("--output FILE") && err.contains("--format json"),
        "{err}"
    );
    assert!(err.contains("cannot reconstruct"), "{err}");
}

#[test]
fn human_file_output_preserves_destination_protection_and_write_errors() {
    let dir = fixture();
    baseline(dir.path());
    for format in ["table", "markdown", "json", "headline"] {
        let out = run(
            dir.path(),
            &[
                "service",
                "--format",
                format,
                "--output",
                "service/result.out",
            ],
        );
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stdout.is_empty());
        assert!(!dir.path().join("service/result.out").exists());
        let original = fs::read(dir.path().join("before.json")).unwrap();
        let out = run(
            dir.path(),
            &[
                "compare",
                "before.json",
                "before.json",
                "--format",
                format,
                "--output",
                "before.json",
            ],
        );
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stdout.is_empty());
        assert_eq!(fs::read(dir.path().join("before.json")).unwrap(), original);
    }
    let out = run(dir.path(), &["service", "--output", "before.json/child"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
}

#[test]
fn historical_headline_report_pointers_round_trip_in_both_contracts() {
    let dir = fixture();
    baseline(dir.path());
    let mut measurement = json(&run(dir.path(), &["service"]).stdout);
    measurement["report"] = "old-report.json".into();
    measurement["metadata"]["locmetry_version"] = "0.3.0".into();
    let decoded: locmetry::contract::MeasurementReport =
        serde_json::from_value(measurement.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), measurement);
    assert!(jsonschema::is_valid(
        &locmetry::report_schema(),
        &measurement
    ));
    let mut comparison = json(&run(dir.path(), &["compare", "before.json", "before.json"]).stdout);
    comparison["report"] = "old-comparison.json".into();
    let decoded: locmetry::compare::Comparison =
        serde_json::from_value(comparison.clone()).unwrap();
    assert_eq!(serde_json::to_value(decoded).unwrap(), comparison);
    assert!(jsonschema::is_valid(
        &locmetry::compare::schema(),
        &comparison
    ));
}

#[cfg(unix)]
#[test]
fn human_exports_protect_configuration_workspace_and_aliased_comparison_inputs() {
    let dir = fixture();
    baseline(dir.path());
    fs::write(dir.path().join("config.toml"), "version = 1\n").unwrap();
    std::os::unix::fs::symlink(
        dir.path().join("before.json"),
        dir.path().join("symlink.json"),
    )
    .unwrap();
    fs::hard_link(
        dir.path().join("before.json"),
        dir.path().join("hardlink.json"),
    )
    .unwrap();
    for format in ["table", "markdown"] {
        for (mut args, path) in [
            (vec!["service", "--config", "config.toml"], "config.toml"),
            (
                vec!["workspace", "--file", "workspace.toml"],
                "workspace.toml",
            ),
            (
                vec!["workspace", "--file", "workspace.toml"],
                "service/src/deep/main.py",
            ),
            (
                vec!["compare", "before.json", "before.json"],
                "symlink.json",
            ),
            (
                vec!["compare", "before.json", "before.json"],
                "hardlink.json",
            ),
        ] {
            let original = fs::read(dir.path().join(path)).unwrap();
            args.extend(["--format", format, "--output", path]);
            let out = run(dir.path(), &args);
            assert_eq!(out.status.code(), Some(1));
            assert!(out.stdout.is_empty());
            assert_eq!(fs::read(dir.path().join(path)).unwrap(), original);
        }
    }
}
