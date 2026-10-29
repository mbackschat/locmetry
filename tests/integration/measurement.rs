use crate::common;
use common::write_all;
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
    let rows = file_records(&payload, &policy("")).unwrap();
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
        &json!({
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
        &json!({"TypeScript":{"reports":[
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
        let error = file_records(&payload, &policy("")).unwrap_err();
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
    let rows = file_records(&result.payload, &config).unwrap();
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

#[cfg(unix)]
#[test]
fn legal_unix_names_with_colons_and_backslashes_keep_their_identity() {
    let temp = tempfile::tempdir().unwrap();
    write_all(
        temp.path(),
        &[
            ("a:file.rs", "fn a() {}\n"),
            ("a\\b.rs", "fn b() {}\n"),
            ("a/b.rs", "fn c() {}\n"),
        ],
    );
    let config = policy("");
    let rows = file_records(&scan(temp.path(), &config).unwrap().payload, &config).unwrap();
    let paths: Vec<_> = rows.iter().map(|row| row.path.as_str()).collect();
    assert_eq!(paths, ["a/b.rs", "a:file.rs", "a\\b.rs"]);
}

#[test]
fn raw_backend_payload_contains_only_root_relative_paths() {
    let temp = tempfile::tempdir().unwrap();
    write_all(
        temp.path(),
        &[(
            "web/index.html",
            "<p>x</p>\n<script>\nlet x = 1;\n</script>\n",
        )],
    );
    let payload = scan(temp.path(), &policy("")).unwrap().payload;
    let root = dunce::canonicalize(temp.path()).unwrap();
    let raw = payload.to_string();
    assert!(raw.contains("web/index.html"));
    assert!(!raw.contains(root.to_str().unwrap()), "{raw}");
}

fn scan_rows(files: &[(&str, &str)]) -> Vec<locmetry::FileRecord> {
    let temp = tempfile::tempdir().unwrap();
    write_all(temp.path(), files);
    let config = policy("");
    file_records(&scan(temp.path(), &config).unwrap().payload, &config).unwrap()
}

#[test]
fn handlebars_distinguishes_template_text_from_quoted_syntax() {
    let cases = [
        ("Owner's notice\n\n{{name}}\n", (2, 0, 1)),
        ("Don't hide comments\n{{! explanation }}\n\n", (1, 1, 1)),
        ("An opening \" in prose\n\n{{name}}\n", (2, 0, 1)),
        ("It's prose\n\nThat's all\n", (2, 0, 1)),
        ("{{helper 'first\n\nlast'}}\n\n", (3, 0, 1)),
        ("{{helper \"first\n\nlast\"}}\n\n", (3, 0, 1)),
        (
            "{{{helper 'first\n\nlast'}}}\n\n{{! comment }}\n",
            (3, 1, 1),
        ),
        ("{{~helper 'first'~}}\n\n{{! comment }}\n", (1, 1, 1)),
        (
            "{{helper '}} {{! literal }} <!--'}}\n{{! comment }}\n",
            (1, 1, 0),
        ),
        (
            r#"{{helper "escaped \" }} {{! literal }}"}}
{{! comment }}
"#,
            (1, 1, 0),
        ),
        ("{{lookup [owner's}}key]}}\n\n{{! comment }}\n", (1, 1, 1)),
        ("{{!-- comment with {{name}}\nmore --}}\n\n", (0, 2, 1)),
        ("{{~!-- comment with {{name}}\nmore --~}}\n", (0, 2, 0)),
        ("{{~! comment ~}}\ntext {{! comment }}\n", (1, 1, 0)),
        ("<!-- owner's comment\n\nend -->\n{{name}}\n", (1, 3, 0)),
        (
            "{{{{raw}}}}\nOwner's {{! literal }}\n\n{{{{/raw}}}}\n{{! comment }}\n",
            (3, 1, 1),
        ),
        ("\\{{helper 'literal}}\n\n{{! comment }}\n", (1, 1, 1)),
        ("\\\\{{helper 'first\n\nlast'}}\n", (3, 0, 0)),
        (
            "<a title=\"<!-- literal -->\">Owner's</a>\n\n<!-- comment -->\n",
            (1, 1, 1),
        ),
        ("Owner's notice\r\n\r\n{{name}}", (2, 0, 1)),
        ("Ümlaut's notice\n\n{{name}}\n", (2, 0, 1)),
        ("", (0, 0, 0)),
        ("{{~{helper 'value'}~}}\nOwner's prose\n\n", (2, 0, 1)),
        ("{{lookup [key\\]'s]}}\n\n{{! comment }}\n", (1, 1, 1)),
        ("{{{{raw}}}}\n{{{{nested}}}}\n{{{{/nested}}}}\n{{! literal }}\n{{{{/raw}}}}\n{{! comment }}\n", (5, 1, 0)),
        ("{{{{raw\n'value'}}}}\n{{! literal }}\n{{{{/raw}}}}\n{{! comment }}\n", (4, 1, 0)),
    ];
    for (text, expected) in cases {
        for extension in ["hbs", "handlebars"] {
            let rows = scan_rows(&[(&format!("template.{extension}"), text)]);
            let row = &rows[0];
            assert_eq!(row.language, "Handlebars");
            assert_eq!(row.kind, Kind::Source);
            assert_eq!(
                (row.counts.code, row.counts.comments, row.counts.blanks),
                expected,
                "{extension}: {text:?}"
            );
            assert_eq!(row.counts.lines, text.lines().count() as u64);
        }
    }
}

/// Lean 4.31.0 accepted every fixture except the unterminated comment on 2026-10-28.
#[test]
fn lean_comment_markers_inside_literals_stay_code() {
    let cases = [
        (
            "def opener : String := \"/-\"\n\ndef a : Nat := 1\ndef b : Nat := 2\n",
            (3, 0, 1),
        ),
        (
            r##"def path : String := "packet/-receipt.json"
def interp : String := s!"x {"/-"} y"
def quoteChar : Char := '"'
def raw : String := r#"a "/-" b"#
def escaped : String := "a \" /- b"
/- nested /- comment -/ still comment -/
/-- doc -/
def a : Nat := 1
#eval [path, interp, toString quoteChar, raw, escaped]
"##,
            (7, 2, 0),
        ),
        ("def a : Nat := 1\n/- open\n\nstill\n", (1, 3, 0)),
        (
            "-- line\ndef a : Nat := 1 -- trailing\n\n/-- doc\n  more -/\ndef b : Nat := 2\n/-! module\ndoc -/\n",
            (2, 5, 1),
        ),
        (
            "def k (f : Char → Char → Nat) (x' : Char) : Nat := f x' '\"'\n-- note\ndef a : Nat := 1\n",
            (2, 1, 0),
        ),
        (
            "def brace : String := \"{\"\ndef p : String := panic! \"{\"\ndef c : String := \"-/ /-\"\ndef a : Nat := 1\n",
            (4, 0, 0),
        ),
        (
            r###"def raw : String := r##"a "# /- "##
-- note
def a : Nat := 1
"###,
            (2, 1, 0),
        ),
        (
            "def «x\"y» : Nat := 1\ndef z : String := \"/-\"\ndef a : Nat := 1\n",
            (3, 0, 0),
        ),
        (
            "def s : String := \"first\n\nlast /-\"\ndef a : Nat := 1\n",
            (4, 0, 0),
        ),
        (
            "def q : Char := '\\''\ndef b : Char := '\\\\'\ndef s : String := \"/-\"\ndef a : Nat := 1\n",
            (4, 0, 0),
        ),
        (
            r#"def m : String := s! "a {"/-"} b {s!"c {"--"}"} d"
-- note
def a : Nat := 1
"#,
            (2, 1, 0),
        ),
        (
            "def g : String := \"a\\\n  /- b\"\ndef a : Nat := 1\n",
            (3, 0, 0),
        ),
        (
            "def opener : String := \"/-\"\r\n\r\n-- note\r\n",
            (1, 1, 1),
        ),
        // A line is a comment only when it holds nothing but comments, as the
        // owner decided on 2026-10-29.
        (
            "/- a\nb -/ def x : Nat := 1\n/- c -/ def y : Nat := 2\ndef z : Nat := 3 /- d\n-/\n/- e -/ /- f\n-/\n/- g -/ def w : Nat := 4 /- h -/\n",
            (4, 4, 0),
        ),
    ];
    for (text, expected) in cases {
        for extension in ["lean", "hlean"] {
            let rows = scan_rows(&[(&format!("Main.{extension}"), text)]);
            let row = &rows[0];
            assert_eq!(row.language, "Lean");
            assert_eq!(
                (row.counts.code, row.counts.comments, row.counts.blanks),
                expected,
                "{extension}: {text:?}"
            );
            assert_eq!(row.counts.lines, text.lines().count() as u64);
        }
    }
}

#[test]
fn handlebars_preserves_bom_decoding() {
    let text = "Owner's notice\n\n{{! comment }}\n";
    let mut utf8 = vec![0xef, 0xbb, 0xbf];
    utf8.extend(text.as_bytes());
    let mut utf16le = vec![0xff, 0xfe];
    utf16le.extend(text.encode_utf16().flat_map(u16::to_le_bytes));
    let mut utf16be = vec![0xfe, 0xff];
    utf16be.extend(text.encode_utf16().flat_map(u16::to_be_bytes));
    for bytes in [utf8, utf16le, utf16be] {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join("notice.hbs"), bytes).unwrap();
        let config = policy("");
        let result = scan(temp.path(), &config).unwrap();
        let rows = result.records(&config).unwrap();
        assert!(!result.inaccurate);
        assert_eq!(result.coverage.counted_files, 1);
        assert_eq!(
            (
                rows[0].counts.code,
                rows[0].counts.comments,
                rows[0].counts.blanks
            ),
            (1, 1, 1)
        );
    }
}

#[cfg(unix)]
#[test]
fn unreadable_handlebars_is_reported_as_missing_and_inaccurate() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    let file = temp.path().join("notice.hbs");
    fs::write(&file, "Owner's notice\n").unwrap();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read(&file).is_ok() {
        return;
    }
    let result = scan(temp.path(), &policy(""));
    fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
    let result = result.unwrap();
    assert!(result.inaccurate);
    assert_eq!(result.coverage.unread, ["notice.hbs"]);
    assert_eq!(result.coverage.counted_files, 0);
}

#[test]
fn mapped_handlebars_uses_the_same_counter_and_keeps_provenance() {
    let temp = tempfile::tempdir().unwrap();
    write_all(
        temp.path(),
        &[("notice.template", "Owner's notice\n\n{{! comment }}\n")],
    );
    let config = policy(
        r#"
[[languages.mappings]]
id = "notice-template"
match = ["*.template"]
language = "Handlebars"
"#,
    );
    let result = scan(temp.path(), &config).unwrap();
    let rows = result.records(&config).unwrap();
    assert_eq!(result.coverage.counted_files, 1);
    assert_eq!(result.coverage.mapped_files, 1);
    assert!(!result.inaccurate);
    assert_eq!(rows[0].language_mapping.as_deref(), Some("notice-template"));
    assert_eq!(
        (
            rows[0].counts.code,
            rows[0].counts.comments,
            rows[0].counts.blanks
        ),
        (1, 1, 1)
    );
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
    write_all(
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
    write_all(temp.path(), &[("locked.py", "x = 1\n")]);
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
        (3, 10, 1, 14)
    );
    assert_eq!(rows[0].languages.keys().collect::<Vec<_>>(), ["Rust"]);
}

#[test]
fn documentation_embedded_in_source_counts_as_host_comments() {
    let rows = scan_rows(&[(
        "src/Page.vue",
        "<template lang=\"markdown\">\n# Title\n\nSome text.\n</template>\n",
    )]);
    let counts = rows[0].counts;
    assert_eq!(
        (counts.code, counts.comments, counts.blanks, counts.lines),
        (2, 2, 1, 5)
    );
    assert_eq!(rows[0].languages.keys().collect::<Vec<_>>(), ["Vue"]);
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

#[test]
fn handlebars_decode_failures_are_content_failures_for_detected_and_mapped_files() {
    for name in ["notice.hbs", "notice.handlebars", "notice.template"] {
        let temp = tempfile::tempdir().unwrap();
        fs::write(temp.path().join(name), b"caf\xe9\n").unwrap();
        let config = policy(
            r#"
[[languages.mappings]]
id = "templates"
match = ["*.template"]
language = "Handlebars"
"#,
        );
        let result = scan(temp.path(), &config).unwrap();
        assert!(result.inaccurate);
        assert!(result.coverage.unread.is_empty(), "{name}");
        assert_eq!(result.coverage.parse_failures.len(), 1, "{name}");
        let failure = &result.coverage.parse_failures[0];
        assert_eq!(failure.path, name);
        assert_eq!(failure.reason.to_string(), "invalid_encoding");
        assert!(failure
            .message
            .contains("Cannot decode Handlebars encoding"));
        assert!(result.records(&config).unwrap().is_empty());
    }
}

#[test]
fn lean_decode_failures_are_content_failures() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("Main.lean"), b"-- caf\xe9\n").unwrap();
    let config = policy("");
    let result = scan(temp.path(), &config).unwrap();
    assert!(result.inaccurate);
    assert!(result.coverage.unread.is_empty());
    let failure = &result.coverage.parse_failures[0];
    assert_eq!(failure.path, "Main.lean");
    assert_eq!(failure.reason.to_string(), "invalid_encoding");
    assert!(failure.message.contains("Cannot decode Lean encoding"));
    assert!(result.records(&config).unwrap().is_empty());
}
