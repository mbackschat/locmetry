//! Language corrections (docs/proposals/archive/language-corrections.md): project
//! entries that replace the counter's language for files it recognized
//! wrongly, with a counter language or `"unrecognized"`.
use crate::cli::*;
use crate::common::locmetry;
use serde_json::{json, Value};
use std::{fs, process::Command};

fn file<'a>(document: &'a Value, path: &str) -> Option<&'a Value> {
    document["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"] == path)
}

const LATEX_CLASS: &str =
    "% A report class.\n\\NeedsTeXFormat{LaTeX2e}\n\\ProvidesClass{report}\n\\LoadClass{article}\n";

/// LaTeX classes misread as Visual Basic 6 count their `%` comments as code;
/// TeX shares their comment syntax.
#[test]
fn a_correction_counts_the_file_with_the_named_language() {
    let document = measured(
        "version = 1\n[[languages.corrections]]\nid = \"latex\"\nmatch = [\"tex/**/*.cls\"]\nlanguage = \"TeX\"\nreason = \"LaTeX document classes\"\n",
        &[("tex/classes/report.cls", LATEX_CLASS), ("other/x.cls", LATEX_CLASS)],
    );
    let corrected = file(&document, "tex/classes/report.cls").unwrap();
    assert_eq!(corrected["language"], "TeX");
    // The corrected language's content kind applies, as for a mapping.
    assert_eq!(corrected["kind"], "documentation");
    assert_eq!(corrected["language_correction"], "latex");
    assert!(corrected.get("language_mapping").is_none());
    assert_eq!(corrected["counts"]["comments"], 1);
    assert_eq!(corrected["counts"]["code"], 3);
    let untouched = file(&document, "other/x.cls").unwrap();
    assert_eq!(untouched["language"], "VB6/VBA");
    assert!(untouched.get("language_correction").is_none());
    assert_eq!(
        document["metadata"]["config"]["languages"]["corrections"][0]["id"],
        "latex"
    );
}

#[test]
fn a_correction_to_unrecognized_lists_the_file_as_unrecognized() {
    let document = measured(
        "version = 1\n[[languages.corrections]]\nid = \"tree-sitter-queries\"\nmatch = [\"app/queries/*.scm\"]\nlanguage = \"unrecognized\"\n",
        &[
            ("app/queries/highlights.scm", "; Keywords\n(identifier) @variable\n"),
            ("app/Main.kt", "fun main() {}\n"),
        ],
    );
    assert!(file(&document, "app/queries/highlights.scm").is_none());
    let coverage = &document["metadata"]["coverage"];
    assert_eq!(
        coverage["unrecognized"],
        json!(["app/queries/highlights.scm"])
    );
    assert_eq!(coverage["unrecognized_files"], 1);
    assert_eq!(coverage["selected_files"], 2);
    assert_eq!(coverage["counted_files"], 1);
    assert!(document["summary"]["source_languages"]
        .get("Scheme")
        .is_none());
}

/// JSON keeps the corrected counter name, while tables fold `C++ Header` into
/// C++ as they fold other variants.
#[test]
fn corrected_cpp_headers_join_the_cpp_row() {
    let config = "version = 1\n[[languages.corrections]]\nid = \"cpp-headers\"\nmatch = [\"include/**/*.h\"]\nlanguage = \"C++ Header\"\n";
    let temp = project(&[
        (".locmetryrc", config),
        ("include/a.h", "class A {};\n"),
        ("src/a.cpp", "int a() { return 1; }\n"),
    ]);
    let root = temp.path().join("project");
    let (success, document, stderr) = report(&root, &[]);
    assert!(success, "{stderr}");
    assert_eq!(
        file(&document, "include/a.h").unwrap()["language"],
        "C++ Header"
    );
    let table = String::from_utf8(run(&root, &["--format", "table"]).stdout).unwrap();
    assert!(!table.contains("C Header"), "{table}");
    let cpp = table
        .lines()
        .find(|line| line.trim_start().starts_with("C++ "))
        .expect(&table);
    assert_eq!(cpp.split_whitespace().nth(1), Some("2"), "{cpp}");
}

/// Corrections apply before recognition: a mapping never applies to a
/// corrected file, the first matching correction decides, and a correction
/// that decides no file is reported like an unused entry.
#[test]
fn corrections_precede_mappings_and_unused_ones_are_reported() {
    let config = r#"version = 1
[[languages.mappings]]
id = "zzz-ini"
match = ["**/*.zzz"]
language = "INI"

[[languages.corrections]]
id = "first"
match = ["conf/a.zzz"]
language = "unrecognized"

[[languages.corrections]]
id = "second"
match = ["conf/*.zzz"]
language = "TOML"

[[languages.corrections]]
id = "stale"
match = ["gone/**"]
language = "unrecognized"
"#;
    let document = measured(
        config,
        &[
            ("conf/a.zzz", "x = 1\n"),
            ("conf/b.zzz", "x = 1\n"),
            ("other/c.zzz", "x = 1\n"),
        ],
    );
    assert!(file(&document, "conf/a.zzz").is_none());
    let b = file(&document, "conf/b.zzz").unwrap();
    assert_eq!(
        (&b["language"], &b["language_correction"]),
        (&json!("TOML"), &json!("second"))
    );
    assert!(b.get("language_mapping").is_none());
    assert_eq!(
        file(&document, "other/c.zzz").unwrap()["language_mapping"],
        "zzz-ini"
    );
    assert_eq!(document["metadata"]["coverage"]["mapped_files"], 1);
    assert_eq!(document["summary"]["unused_entries"], json!(["stale"]));
}

#[test]
fn personal_defaults_cannot_correct_languages() {
    let temp = project(&[("a.scm", "(x) @y\n")]);
    let home = temp.path().join("home");
    fs::create_dir_all(home.join("locmetry")).unwrap();
    fs::write(
        home.join("locmetry/config.toml"),
        "version = 1\n[[languages.corrections]]\nid = \"scm\"\nmatch = [\"**/*.scm\"]\nlanguage = \"unrecognized\"\n",
    )
    .unwrap();
    let output = locmetry()
        .current_dir(temp.path().join("project"))
        .env("XDG_CONFIG_HOME", &home)
        .args(["--output", "-"])
        .output()
        .unwrap();
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("languages.mappings"), "{stderr}");
}

#[test]
fn explain_shows_the_correction_and_its_reason() {
    let config = "version = 1\n[[languages.corrections]]\nid = \"latex\"\nmatch = [\"tex/*.cls\"]\nlanguage = \"TeX\"\nreason = \"LaTeX document classes\"\n\n[[languages.corrections]]\nid = \"queries\"\nmatch = [\"queries/*.scm\"]\nlanguage = \"unrecognized\"\n";
    let temp = project(&[
        (".locmetryrc", config),
        ("tex/report.cls", LATEX_CLASS),
        ("queries/highlights.scm", "(identifier) @variable\n"),
    ]);
    let root = temp.path().join("project");
    let explain = |path: &str, format: &str| {
        let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
            .args(["explain", path, "--no-user-config", "--format", format])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout).unwrap()
    };
    let json: Value = serde_json::from_str(&explain("tex/report.cls", "json")).unwrap();
    assert_eq!(json["result"]["file"]["language_correction"], "latex");
    assert_eq!(
        json["result"]["correction"],
        json!({"id": "latex", "reason": "LaTeX document classes"})
    );
    let text = explain("tex/report.cls", "table");
    assert!(
        text.contains("Language correction: latex (LaTeX document classes)\n"),
        "{text}"
    );
    let json: Value = serde_json::from_str(&explain("queries/highlights.scm", "json")).unwrap();
    assert_eq!(json["result"]["state"], "unrecognized");
    assert_eq!(json["result"]["correction"], json!({"id": "queries"}));
    let text = explain("queries/highlights.scm", "table");
    assert!(text.contains("Language correction: queries\n"), "{text}");
}
