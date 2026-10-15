use locmetry::{build_report, file_records, scan, Kind, Policy};
use serde_json::{json, Value};
use std::fs;

/// Totals with per-folder summaries, the shape saved measurements used.
fn summary_json(rows: &[locmetry::FileRecord]) -> Value {
    let report = build_report(rows);
    let mut value = serde_json::to_value(&report.totals).unwrap();
    value["folders"] = serde_json::to_value(&report.folders).unwrap();
    value
}

fn policy(toml: &str) -> Policy {
    Policy::from_toml(&format!("version = 1\n{toml}")).unwrap()
}

fn stats(code: u64) -> Value {
    json!({"code":code,"comments":0,"blanks":0,"blobs":{}})
}

fn report(path: &str, code: u64) -> Value {
    json!({"name":path,"stats":stats(code)})
}

#[test]
fn ordered_rules_preserve_exceptions_and_unknowns() {
    let config = policy(
        r#"[[rules]]
id = "demo"
match = ["test-support/demo/**"]
purpose = "production"
category = "example"
[[rules]]
id = "support"
match = ["test-support/**"]
purpose = "test"
category = "test-support"
"#,
    );
    assert_eq!(
        config
            .classify("test-support/demo/main.ts", Kind::Source)
            .rule,
        "demo"
    );
    assert_eq!(
        config
            .classify("test-support/helpers.ts", Kind::Source)
            .purpose
            .to_string(),
        "test"
    );
    assert_eq!(config.classify("unknown.ts", Kind::Source).rule, "fallback");
}

#[test]
fn embedded_languages_count_once_and_docs_stay_out_of_source() {
    let payload = json!({
        "HTML":{"reports":[{"name":"./src/index.html","stats":{"code":3,"comments":0,"blanks":0,"blobs":{"JavaScript":stats(7)}}}]},
        "Markdown":{"reports":[{"name":"./README.md","stats":{"code":0,"comments":2,"blanks":0,"blobs":{"Python":stats(5)}}}]},
        "JavaScript":{"reports":[],"children":{"HTML":[report("src/index.html",7)]}},
        "Total":{"reports":[report("duplicate",999)]}
    });
    let rows = file_records(payload, &policy("")).unwrap();
    let summary = summary_json(&rows);
    assert_eq!(rows.len(), 2);
    assert_eq!(summary["source"]["code"], 10);
    assert_eq!(summary["all_counted"]["code"], 15);
    assert_eq!(summary["source_languages"]["JavaScript"]["code"], 7);
    assert_eq!(summary["source"]["files"], 1);
}

#[test]
fn fixtures_remain_data_and_shell_and_rust_are_source() {
    let config = policy("[[rules]]\nid = \"tests\"\nmatch = [\"**/tests/**\"]\npurpose = \"test\"");
    let rows = file_records(
        json!({
            "JSON":{"reports":[report("tests/data.json",100)]},
            "BASH":{"reports":[report("scripts/build.sh",9)]},
            "Rust":{"reports":[report("src/main.rs",2)]}
        }),
        &config,
    )
    .unwrap();
    let summary = summary_json(&rows);
    assert_eq!(summary["source"]["code"], 11);
    assert_eq!(summary["by_kind_purpose"]["data:test"]["code"], 100);
}

#[test]
fn direct_and_recursive_folder_totals_reconcile() {
    let rows = file_records(
        json!({"TypeScript":{"reports":[
            report("root.ts",2),report("packages/a/main.ts",3),report("packages/a/tests/main.ts",5)
        ]}}),
        &policy(""),
    )
    .unwrap();
    let summary = summary_json(&rows);
    assert_eq!(summary["folders"]["."]["recursive"]["source"]["code"], 10);
    assert_eq!(summary["folders"]["."]["direct"]["source"]["code"], 2);
    assert_eq!(
        summary["folders"]["packages/a"]["direct"]["source"]["code"],
        3
    );
    assert_eq!(
        summary["folders"]["packages/a"]["recursive"]["source"]["code"],
        8
    );
    let empty = summary_json(&[]);
    // An empty root keeps its zero `all_counted`; absent `source` means zero.
    assert_eq!(
        empty["folders"]["."]["recursive"]["all_counted"]["files"],
        0
    );
    assert!(empty["folders"]["."]["recursive"].get("source").is_none());
}

#[test]
fn invalid_paths_duplicates_and_counts_fail() {
    for (payload, message) in [
        (
            json!({"TypeScript":{"reports":[report("main.ts",1),report("./main.ts",1)]}}),
            "Duplicate physical file",
        ),
        (
            json!({"TypeScript":{"reports":[report("../main.ts",1)]}}),
            "outside scan root",
        ),
        (
            json!({"TypeScript":{"reports":[report("/main.ts",1)]}}),
            "outside scan root",
        ),
        (
            json!({"TypeScript":{"reports":[{"name":"main.ts","stats":{"code":-1,"comments":0,"blanks":0}}]}}),
            "Invalid backend statistics",
        ),
    ] {
        let error = file_records(payload, &policy("")).unwrap_err();
        assert!(format!("{error:#}").contains(message), "{error:#}");
    }
}

#[test]
fn embedded_scan_respects_root_exclusions_and_ignores_but_not_ambient_config() {
    let temp = tempfile::tempdir().unwrap();
    for path in [
        "src/main.ts",
        "materials/example.ts",
        "materials-old/main.ts",
        "packages/materials/main.ts",
        ".hidden/main.ts",
        ".git/secret.ts",
        "ignored/main.ts",
    ] {
        let path = temp.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "const x = 1;\n").unwrap();
    }
    fs::write(temp.path().join(".ignore"), "ignored/\n").unwrap();
    fs::write(temp.path().join("tokei.toml"), "types = [\"Python\"]\n").unwrap();
    let config = policy("[scope]\nexclude = [\"materials/**\"]");
    let result = scan(temp.path(), &config).unwrap();
    let rows = file_records(result.payload, &config).unwrap();
    let paths: Vec<_> = rows.iter().map(|row| row.path.as_str()).collect();
    for present in [
        "src/main.ts",
        "materials-old/main.ts",
        "packages/materials/main.ts",
        ".hidden/main.ts",
    ] {
        assert!(paths.contains(&present), "missing {present}");
    }
    for absent in ["materials/example.ts", ".git/secret.ts", "ignored/main.ts"] {
        assert!(!paths.contains(&absent), "unexpected {absent}");
    }
}

fn write_tree(root: &std::path::Path, files: &[(&str, &str)]) {
    for (path, content) in files {
        let path = root.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, content).unwrap();
    }
}

#[cfg(unix)]
#[test]
fn legal_unix_names_with_colons_and_backslashes_keep_their_identity() {
    let temp = tempfile::tempdir().unwrap();
    write_tree(
        temp.path(),
        &[
            ("a:file.rs", "fn a() {}\n"),
            ("a\\b.rs", "fn b() {}\n"),
            ("a/b.rs", "fn c() {}\n"),
        ],
    );
    let config = policy("");
    let rows = file_records(scan(temp.path(), &config).unwrap().payload, &config).unwrap();
    let paths: Vec<_> = rows.iter().map(|row| row.path.as_str()).collect();
    assert_eq!(paths, ["a/b.rs", "a:file.rs", "a\\b.rs"]);
}

#[test]
fn raw_backend_payload_contains_only_root_relative_paths() {
    let temp = tempfile::tempdir().unwrap();
    write_tree(
        temp.path(),
        &[(
            "web/index.html",
            "<p>x</p>\n<script>\nlet x = 1;\n</script>\n",
        )],
    );
    let payload = scan(temp.path(), &policy("")).unwrap().payload;
    let root = temp.path().canonicalize().unwrap();
    let raw = payload.to_string();
    assert!(raw.contains("web/index.html"));
    assert!(!raw.contains(root.to_str().unwrap()), "{raw}");
}

fn scan_rows(files: &[(&str, &str)]) -> Vec<locmetry::FileRecord> {
    let temp = tempfile::tempdir().unwrap();
    write_tree(temp.path(), files);
    let config = policy("");
    file_records(scan(temp.path(), &config).unwrap().payload, &config).unwrap()
}

/// Locks Tokei's default, which docs/measurement.md documents: docstrings
/// count as code, not comments, because locmetry never sets
/// `treat_doc_strings_as_comments`. Changing it would change Python totals,
/// so it must be a deliberate, versioned decision.
#[test]
fn python_docstrings_count_as_code() {
    let rows = scan_rows(&[(
        "lib.py",
        "def f():\n    \"\"\"Return one.\n\n    More text.\n    \"\"\"\n    # comment\n    return 1\n",
    )]);
    let counts = rows[0].counts;
    assert_eq!(
        (counts.code, counts.comments, counts.blanks),
        (6, 1, 0),
        "all four docstring lines, even the blank one, count as code with def and return"
    );
}

#[test]
fn programming_languages_without_a_kind_list_entry_count_as_source() {
    let rows = scan_rows(&[
        ("main.go", "package main\n"),
        ("lib.c", "int x;\n"),
        ("query.sql", "select 1;\n"),
        ("data.json", "{}\n"),
        ("README.md", "# Title\n"),
        ("notes.txt", "text\n"),
    ]);
    let kinds: Vec<_> = rows
        .iter()
        .map(|row| (row.path.as_str(), row.kind))
        .collect();
    assert_eq!(
        kinds,
        [
            ("README.md", Kind::Documentation),
            ("data.json", Kind::Data),
            ("lib.c", Kind::Source),
            ("main.go", Kind::Source),
            ("notes.txt", Kind::Documentation),
            ("query.sql", Kind::Source),
        ]
    );
}

#[test]
fn coverage_lists_selected_files_the_backend_did_not_count() {
    let temp = tempfile::tempdir().unwrap();
    write_tree(
        temp.path(),
        &[
            ("src/main.rs", "fn main() {}\n"),
            ("assets/logo.unknownext", "x\n"),
        ],
    );
    let coverage = scan(temp.path(), &policy("")).unwrap().coverage;
    assert_eq!(coverage.selected_files, 2);
    assert_eq!(coverage.counted_files, 1);
    assert_eq!(coverage.unrecognized, ["assets/logo.unknownext"]);
    assert!(coverage.unread.is_empty());
}

#[cfg(unix)]
#[test]
fn coverage_separates_unreadable_recognized_files() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    write_tree(temp.path(), &[("locked.py", "x = 1\n")]);
    let locked = temp.path().join("locked.py");
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read(&locked).is_ok() {
        return; // Permission bits do not restrict this user (for example root).
    }
    let coverage = scan(temp.path(), &policy("")).unwrap().coverage;
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(coverage.unread, ["locked.py"]);
    assert!(coverage.unrecognized.is_empty());
}

#[test]
fn rust_doc_comments_and_doctests_count_as_host_comments() {
    let rows = scan_rows(&[(
        "src/lib.rs",
        "//! Crate.\n//!\n//! ```rust\n//! let x = 1;\n//! let y = 2;\n//! ```\n\n/// Adds.\n/// ```rust\n/// let z = 3;\n/// ```\nfn a() {\n    let b = 1;\n}\n",
    )]);
    let counts = rows[0].counts;
    assert_eq!(
        (counts.code, counts.comments, counts.blanks, counts.lines),
        (3, 9, 2, 14)
    );
    assert_eq!(rows[0].languages.keys().collect::<Vec<_>>(), ["Rust"]);
}

#[test]
fn script_embedded_in_html_remains_source_code() {
    let rows = scan_rows(&[(
        "web/index.html",
        "<p>x</p>\n<script>\nlet x = 1;\nlet y = 2;\n</script>\n",
    )]);
    let summary = summary_json(&rows);
    assert_eq!(summary["source_languages"]["JavaScript"]["code"], 2);
    assert_eq!(summary["source"]["lines"], 5);
}

#[test]
fn folder_totals_reconcile_at_every_level() {
    let rows = scan_rows(&[
        ("root.ts", "let a = 1;\n"),
        ("packages/a/main.ts", "let b = 1;\nlet c = 2;\n"),
        ("packages/a/tests/main.ts", "let d = 1;\n"),
        ("packages/b/deep/er/x.py", "x = 1\n\n# c\n"),
        ("docs/guide.md", "# Guide\n"),
    ]);
    let report = summary_json(&rows);
    let folders = report["folders"].as_object().unwrap();
    let field = |value: &Value, metric: &str| value["all_counted"][metric].as_u64().unwrap();
    for metric in ["files", "code", "comments", "blanks", "lines"] {
        let direct_sum: u64 = folders.values().map(|f| field(&f["direct"], metric)).sum();
        assert_eq!(direct_sum, field(&report, metric), "{metric}");
        for (name, folder) in folders {
            let children: u64 = folders
                .iter()
                .filter(|(child, _)| {
                    let parent = child.rsplit_once('/').map_or(".", |(parent, _)| parent);
                    *child != "." && parent == name
                })
                .map(|(_, child)| field(&child["recursive"], metric))
                .sum();
            assert_eq!(
                field(&folder["recursive"], metric),
                field(&folder["direct"], metric) + children,
                "{name} {metric}"
            );
        }
    }
    assert!(
        folders.contains_key("packages/b/deep"),
        "intermediate folder without direct files"
    );
}
