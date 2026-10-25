use crate::common;
use serde_json::Value;
use std::{fs, path::Path, process::Output};

fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    common::write_all(
        temp.path(),
        &[
            (
                "project/.locmetryrc",
                "version = 1\n[[rules]]\nid = \"app\"\nmatch = [\"src/**\"]\npurpose = \"production\"\n",
            ),
            ("project/src/lib.rs", "// doc\nfn a() {}\nfn b() {}\n"),
            ("project/tests/test.rs", "fn test() {}\n"),
            ("project/proto/deep/x.py", "x = 1\n"),
            ("repos.toml", "version = 1\nroots = [\"project\"]\n"),
        ],
    );
    temp
}

fn run(root: &Path, args: &[&str]) -> Output {
    common::locmetry()
        .current_dir(root)
        .env("COLUMNS", "180")
        .args(["--no-color"])
        .args(args)
        .output()
        .unwrap()
}

fn success(output: &Output) -> String {
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout.clone()).unwrap()
}

#[test]
fn presentation_names_do_not_change_the_saved_measurement() {
    let temp = fixture();
    let output = run(
        temp.path(),
        &[
            "--root",
            "project",
            "--output",
            "full.json",
            "--print",
            "table",
            "--show-folders",
            "--show-categories",
            "--show-bars=purpose",
            "--table-purpose",
            "production",
        ],
    );
    let text = success(&output);
    assert!(output.stderr.is_empty());
    assert!(text.contains(" Folder "));
    assert!(text.contains(" Category "));
    assert!(text.contains("Comment density"));
    assert!(text.contains("Not shown (purposes): Test and Other"));
    let full: Value =
        serde_json::from_str(&fs::read_to_string(temp.path().join("full.json")).unwrap()).unwrap();
    assert_eq!(full["summary"]["source"]["code"], 4);
    assert_eq!(full["summary"]["source_purpose"]["production"]["code"], 2);
    assert!(text.lines().any(|line| {
        line.replace('│', " ")
            .split_whitespace()
            .collect::<Vec<_>>()
            .starts_with(&["Total", "1", "3", "2", "1", "0", "33.3%"])
    }));
}

#[test]
fn retired_spellings_are_rejected_in_every_command_that_used_them() {
    let temp = fixture();
    for prefix in [
        vec!["--root", "project"],
        vec!["workspace", "--root", "project"],
    ] {
        for (old, tip) in [
            (
                vec!["--categories"],
                "'--categories' was renamed to '--show-categories'",
            ),
            (
                vec!["--bars=table"],
                "'--bars' was renamed to '--show-bars'",
            ),
            (
                vec!["--purpose", "production"],
                "'--purpose' was renamed to '--table-purpose'",
            ),
            (
                vec!["--depth", "2"],
                "'--depth' was renamed to '--markdown-depth'",
            ),
        ] {
            let args = [prefix.clone(), old].concat();
            let output = run(temp.path(), &args);
            assert_eq!(output.status.code(), Some(1), "{args:?}");
            assert!(output.stdout.is_empty());
            assert_retired_spelling_names_its_successor(&output.stderr, tip);
        }
    }
    for (args, tip) in [
        (
            vec!["--root", "project", "--folders"],
            "'--folders' was renamed to '--show-folders'",
        ),
        (
            vec!["workspace", "--file", "repos.toml"],
            "'workspace --file' was renamed to '--workspace-file'",
        ),
        (
            vec!["explain", "project/src/lib.rs", "--workspace", "repos.toml"],
            "'explain --workspace' was renamed to '--workspace-file'",
        ),
    ] {
        let output = run(temp.path(), &args);
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        assert_retired_spelling_names_its_successor(&output.stderr, tip);
    }
}

/// Clap's similarity tips sent agents with pre-0.7.0 knowledge to unrelated
/// options, such as `--purpose` to `--verbose` and `workspace --file` to
/// `--files` (agent regression desk review, 2026-10-21, finding F11).
fn assert_retired_spelling_names_its_successor(stderr: &[u8], tip: &str) {
    let stderr = String::from_utf8_lossy(stderr);
    assert!(stderr.contains("unexpected argument"), "{stderr}");
    assert_eq!(stderr.matches("tip:").count(), 1, "{stderr}");
    assert!(stderr.contains(tip), "{stderr}");
    assert!(!stderr.contains("similar argument"), "{stderr}");
    assert!(!stderr.contains("as a value"), "{stderr}");
}

#[test]
fn workspace_file_selects_the_same_definition_for_measurement_and_explanation() {
    let temp = fixture();
    let report = common::json(
        &run(
            temp.path(),
            &[
                "workspace",
                "--workspace-file",
                "repos.toml",
                "--format",
                "json",
            ],
        ),
        0,
    );
    assert_eq!(report["summary"]["source"]["code"], 4);
    let explanation = common::json(
        &run(
            temp.path(),
            &[
                "explain",
                "project/src/lib.rs",
                "--workspace-file",
                "repos.toml",
                "--format",
                "json",
            ],
        ),
        0,
    );
    assert_eq!(explanation["result"]["state"], "counted");
    for args in [
        vec![
            "workspace",
            "--workspace-file",
            "repos.toml",
            "--root",
            "project",
        ],
        vec![
            "explain",
            "project/src/lib.rs",
            "--workspace-file",
            "repos.toml",
            "--root",
            "project",
        ],
    ] {
        assert_eq!(run(temp.path(), &args).status.code(), Some(1));
    }
}

#[test]
fn full_json_accepts_file_detail_in_either_output_position() {
    let temp = fixture();
    for prefix in [
        vec!["--root", "project"],
        vec!["workspace", "--workspace-file", "repos.toml"],
    ] {
        for other in ["table", "markdown", "headline", "json"] {
            for json_saved in [false, true] {
                let (saved_format, printed_format) = if json_saved {
                    ("json", other)
                } else {
                    (other, "json")
                };
                let args = [
                    prefix.clone(),
                    vec![
                        "--output",
                        "saved",
                        "--format",
                        saved_format,
                        "--print",
                        printed_format,
                        "--files",
                    ],
                ]
                .concat();
                let output = run(temp.path(), &args);
                success(&output);
                assert!(output.stderr.is_empty(), "{args:?}");
                let saved = fs::read_to_string(temp.path().join("saved")).unwrap();
                let full: Value = serde_json::from_slice(if json_saved {
                    saved.as_bytes()
                } else {
                    &output.stdout
                })
                .unwrap();
                assert_eq!(full["summary"]["source"]["code"], 4);
                assert_eq!(full["files"].as_array().unwrap().len(), 3);
                let other_text = if json_saved {
                    output.stdout.as_slice()
                } else {
                    saved.as_bytes()
                };
                if other == "headline" {
                    let headline: Value = serde_json::from_slice(other_text).unwrap();
                    assert_eq!(headline["detail"], "headline");
                    assert!(headline.get("files").is_none());
                }
            }
        }
    }
}

#[test]
fn presentation_applies_to_both_tables_but_does_not_filter_a_markdown_view() {
    let temp = fixture();
    for prefix in [
        vec!["--root", "project"],
        vec!["workspace", "--workspace-file", "repos.toml"],
    ] {
        for (saved_format, printed_format) in [
            ("table", "table"),
            ("markdown", "table"),
            ("table", "markdown"),
        ] {
            let args = [
                prefix.clone(),
                vec![
                    "--output",
                    "saved",
                    "--format",
                    saved_format,
                    "--print",
                    printed_format,
                    "--table-purpose",
                    "production",
                    "--show-categories",
                    "--verbose",
                    "--no-bars",
                ],
            ]
            .concat();
            let output = run(temp.path(), &args);
            let printed = success(&output);
            assert!(output.stderr.is_empty());
            let saved = fs::read_to_string(temp.path().join("saved")).unwrap();
            let table = if saved_format == "table" {
                &saved
            } else {
                &printed
            };
            assert!(table.contains("Not shown (purposes): Test and Other"));
            assert!(table.lines().any(|line| {
                line.replace('│', " ")
                    .split_whitespace()
                    .collect::<Vec<_>>()
                    .starts_with(&["Total", "1", "3", "2", "1", "0", "33.3%"])
            }));
            if saved_format == printed_format {
                assert_eq!(saved, printed);
            } else {
                let markdown = if saved_format == "markdown" {
                    &saved
                } else {
                    &printed
                };
                assert!(markdown.contains("| Rust | 2 | 4 | 3 |"), "{markdown}");
                assert!(markdown.contains("proto/deep/x.py"));
            }
        }
    }
}

#[test]
fn purpose_selection_preserves_repeatability_and_chart_kind_depends_on_selection() {
    let temp = fixture();
    let output = run(
        temp.path(),
        &[
            "project",
            "--format",
            "table",
            "--table-purpose",
            "production",
            "--table-purpose",
            "test",
            "--show-bars=purpose",
        ],
    );
    let text = success(&output);
    assert!(text.contains("Code share"));
    assert!(text.contains("Not shown (purpose): Other"));
    let only_production_data = run(
        temp.path(),
        &[
            "--root",
            "project",
            "--subfolder",
            "src",
            "--format",
            "table",
            "--show-bars=purpose",
        ],
    );
    assert!(success(&only_production_data).contains("Code share"));
    for args in [
        vec!["project", "--show-folders", "--show-folders"],
        vec!["project", "--show-bars=purpose", "--show-bars=comments"],
        vec!["project", "--show-bars", "--no-bars"],
        vec!["--schema", "report", "--table-purpose", "production"],
        vec!["--guide", "reports", "--markdown-depth", "2"],
        vec!["compare", "before.json", "after.json", "--show-folders"],
    ] {
        let output = run(temp.path(), &args);
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn invalid_view_combinations_preserve_files_and_fail_before_root_preparation() {
    let temp = tempfile::tempdir().unwrap();
    for prefix in [
        vec!["--root", "missing"],
        vec!["workspace", "--root", "missing"],
    ] {
        for (extra, expected) in [
            (vec!["--format", "table", "--files"], "'--files' adds"),
            (
                vec!["--format", "headline", "--print", "table", "--files"],
                "'--files' adds",
            ),
        ] {
            for exists in [false, true] {
                let dest = temp.path().join("saved");
                if exists {
                    fs::write(&dest, "keep me").unwrap();
                }
                let args = [prefix.clone(), vec!["--output", "saved"], extra.clone()].concat();
                let output = run(temp.path(), &args);
                assert_eq!(output.status.code(), Some(1));
                assert!(output.stdout.is_empty());
                let error = String::from_utf8_lossy(&output.stderr);
                assert!(error.contains(expected), "{error}");
                // The missing root cannot be prepared or scanned; the format error must win.
                assert!(!error.contains("does not exist"), "{error}");
                if exists {
                    assert_eq!(fs::read_to_string(&dest).unwrap(), "keep me");
                    fs::remove_file(dest).unwrap();
                } else {
                    assert!(!dest.exists());
                }
            }
        }
    }
    for args in [
        vec!["--root", "missing", "--print", "json", "--files"],
        vec![
            "--root", "missing", "--output", "-", "--print", "json", "--files",
        ],
    ] {
        let output = run(temp.path(), &args);
        assert!(
            String::from_utf8_lossy(&output.stderr).contains("'--print' requires '--output FILE'")
        );
        assert!(output.stdout.is_empty());
    }
    fs::write(temp.path().join("saved"), "keep me").unwrap();
    let output = run(
        temp.path(),
        &[
            "--root",
            "missing",
            "--output",
            "saved",
            "--folder-rows",
            "10",
        ],
    );
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("'--folder-rows' requires table output")
    );
    assert_eq!(
        fs::read_to_string(temp.path().join("saved")).unwrap(),
        "keep me"
    );
}

/// Agents run without a terminal, where an option of one format used to leave
/// a JSON headline and a warning their own `2>&1 | sed` filters hid (feedback
/// of three projects, 2026-10-25). Such an option now selects its format
/// unless `--format` or `--output` decides; a saved report keeps full JSON and
/// the warning names `--print`.
#[test]
fn an_option_of_one_format_selects_it_unless_format_or_output_decides() {
    let temp = fixture();
    for (option, format) in [
        ("--show-folders", "table"),
        ("--show-categories", "table"),
        ("--show-bars", "table"),
        ("--no-bars", "table"),
        ("--verbose", "table"),
        ("--tables-only", "table"),
        ("--table-purpose", "table"),
        ("--table-category", "table"),
        ("--table-language", "table"),
        ("--folder-rows", "table"),
        ("--markdown-depth", "markdown"),
        ("--files", "json"),
    ] {
        let mut args = vec!["--root", "project", option];
        match option {
            "--table-purpose" | "--table-category" => args.push("production"),
            "--table-language" => args.push("Rust"),
            "--folder-rows" => args.push("10"),
            "--markdown-depth" => args.push("2"),
            _ => {}
        }
        let output = run(temp.path(), &args);
        let text = success(&output);
        assert!(
            output.stderr.is_empty(),
            "{option}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        match format {
            "table" => assert!(text.contains(" Language "), "{option}: {text}"),
            "markdown" => assert!(text.starts_with("# "), "{option}: {text}"),
            _ => assert_eq!(common::json(&output, 0)["detail"], "full"),
        }
        if option == "--files" {
            continue;
        }
        args.extend(["--output", "full.json"]);
        let output = run(temp.path(), &args);
        if option == "--folder-rows" {
            // Folder detail cannot shape a saved JSON report, so it still fails.
            assert_eq!(output.status.code(), Some(1));
            assert!(String::from_utf8_lossy(&output.stderr).contains("--print table"));
            continue;
        }
        success(&output);
        let saved: Value =
            serde_json::from_str(&fs::read_to_string(temp.path().join("full.json")).unwrap())
                .unwrap();
        assert_eq!(saved["detail"], "full", "{option}");
        assert!(String::from_utf8_lossy(&output.stderr).contains(&format!("--print {format}")));
    }
    for args in [
        vec!["--root", "project", "--sort", "code"],
        vec!["--root", "project", "--output", "-", "--verbose"],
    ] {
        let output = run(temp.path(), &args);
        assert_ne!(output.stdout.first(), Some(&b' '), "{args:?}");
        common::json(&output, 0);
    }
    // Options of different formats imply none of them, so both still warn.
    let output = run(
        temp.path(),
        &["--root", "project", "--verbose", "--markdown-depth", "2"],
    );
    assert_eq!(common::json(&output, 0)["detail"], "headline");
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(
        error.contains("'--verbose' has no effect here")
            && error.contains("'--markdown-depth' has no effect here"),
        "{error}"
    );
    // Table and Markdown both apply '--split-languages', so it implies neither.
    let output = run(temp.path(), &["--root", "project", "--split-languages"]);
    assert_eq!(common::json(&output, 0)["detail"], "headline");
    stdout_of_compare_tables_only(temp.path());
}

fn stdout_of_compare_tables_only(root: &Path) {
    success(&run(root, &["--root", "project", "--output", "saved.json"]));
    let output = run(
        root,
        &["compare", "saved.json", "saved.json", "--tables-only"],
    );
    let text = success(&output);
    assert!(
        text.contains(" Kind ") && output.stderr.is_empty(),
        "{text}"
    );
}

#[test]
fn markdown_depth_limits_only_folder_breakdown_with_selection_relative_levels() {
    let temp = fixture();
    for (prefix, depth, included, excluded) in [
        (vec!["--root", "project"], "0", "", "| proto/deep |"),
        (
            vec!["--root", "project", "--subfolder", "proto"],
            "1",
            "| proto/deep |",
            "",
        ),
        (
            vec!["workspace", "--workspace-file", "repos.toml"],
            "1",
            "| project |",
            "| project/proto |",
        ),
        (
            vec!["workspace", "--workspace-file", "repos.toml"],
            "2",
            "| project/proto |",
            "| project/proto/deep |",
        ),
    ] {
        let args = [
            prefix,
            vec!["--format", "markdown", "--markdown-depth", depth],
        ]
        .concat();
        let text = success(&run(temp.path(), &args));
        let folders = text
            .split("## Folder breakdown")
            .nth(1)
            .unwrap()
            .split("\n## ")
            .next()
            .unwrap();
        if !included.is_empty() {
            assert!(folders.contains(included), "{folders}");
        }
        if !excluded.is_empty() {
            assert!(!folders.contains(excluded), "{folders}");
        }
        if depth == "0" {
            assert_eq!(
                folders
                    .lines()
                    .filter(|line| line.starts_with("| "))
                    .count(),
                2
            );
        }
        assert!(text.contains("proto/deep/x.py"), "{text}");
    }
}

#[test]
fn help_groups_scope_output_and_presentation_without_advertising_retired_names() {
    let temp = tempfile::tempdir().unwrap();
    for help in ["-h", "--help"] {
        let text = success(&run(temp.path(), &[help]));
        let selection = text.find("Selection and policy:").unwrap();
        let output = text.find("Output and content:").unwrap();
        let presentation = text.find("Presentation:").unwrap();
        assert_eq!(text.matches("\nOptions:\n").count(), 1);
        assert!(text[presentation..].contains("\nReference:\n"));
        assert!(selection < output && output < presentation, "{text}");
        for option in ["--subfolder", "--root", "--config"] {
            assert!(text[selection..output].contains(option), "{option}");
        }
        for option in ["--files", "--output", "--print"] {
            assert!(text[output..presentation].contains(option), "{option}");
        }
        for option in [
            "--show-folders",
            "--show-bars",
            "--show-categories",
            "--table-purpose",
            "--markdown-depth",
        ] {
            assert!(text[presentation..].contains(option), "{option}");
        }
        for old in [
            "--folders",
            "--bars",
            "--categories",
            "--purpose",
            "--depth",
        ] {
            assert!(!text.contains(old), "{old}");
        }
        assert!(text.contains("--root <DIR>"));
        assert!(text.contains("--config <FILE>"));
    }
}
