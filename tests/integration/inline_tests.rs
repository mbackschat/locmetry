use crate::common;
use common::{stdout, write_all};
use locmetry::contract::MeasurementReport;
use serde_json::{json, Value};
use std::{path::Path, process::Output};

fn run(root: &Path, args: &[&str]) -> Output {
    common::locmetry()
        .current_dir(root)
        .args(args)
        .output()
        .unwrap()
}

/// The `#[cfg(test)]` module of `src/lib.rs` spans 11 lines: 8 code lines
/// (the attribute, `mod tests {`, `use`, `#[test]`, the function's three
/// lines and the module's closing brace), 2 comment lines (a line comment and
/// a doc comment, which Tokei parses as a Markdown blob) and 1 blank. Strings and comments
/// hold braces and attribute text that must neither end nor start an item.
const LIB: &str = r##"pub fn add(a: u32, b: u32) -> u32 {
    let _ = "#[cfg(test)] mod fake { }";
    a + b
}

// #[cfg(test)] fn commented_out() {}
#[cfg(test)]
mod tests {
    use super::*;
    // A comment with a brace: }

    /// Adds.
    #[test]
    fn adds() {
        assert_eq!(add(1, 1), 2); // r#"}"#
    }
}
"##;

fn project(root: &Path) {
    write_all(
        root,
        &[
            ("Cargo.toml", "[package]\nname = \"demo\"\n"),
            ("src/lib.rs", LIB),
            (
                "src/out.rs",
                "pub fn out() {}\n\n#[cfg(test)]\nmod tests;\n",
            ),
            ("src/out/tests.rs", "#[test]\nfn out() {}\n"),
        ],
    );
}

fn report(root: &Path) -> Value {
    serde_json::from_str(&stdout(run(root, &["--files", "--output", "-"]))).unwrap()
}

fn file<'a>(report: &'a Value, path: &str) -> &'a Value {
    report["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"] == path)
        .unwrap()
}

#[test]
fn inline_rust_tests_are_reported_without_changing_purposes() {
    let temp = tempfile::tempdir().unwrap();
    project(temp.path());
    let report = report(temp.path());
    let lib = file(&report, "src/lib.rs");
    assert_eq!(
        lib["inline_tests"],
        json!({"code": 8, "comments": 2, "blanks": 1, "lines": 11})
    );
    assert_eq!(lib["purpose"], "production");
    assert_eq!(
        report["summary"]["inline_tests"],
        json!({"production": {"files": 1, "code": 8, "comments": 2, "blanks": 1, "lines": 11}})
    );
    // Purposes stay whole-file: the inline lines remain production code.
    let code = |path| file(&report, path)["counts"]["code"].as_u64().unwrap();
    assert_eq!(
        report["summary"]["source_purpose"]["production"]["code"],
        code("src/lib.rs") + code("src/out.rs")
    );
}

/// `mod tests;` is only a declaration: its code is counted in its own file,
/// which the `rust-test-module` convention already classifies as test.
#[test]
fn out_of_line_test_modules_add_no_inline_tests() {
    let temp = tempfile::tempdir().unwrap();
    project(temp.path());
    let report = report(temp.path());
    assert!(file(&report, "src/out.rs").get("inline_tests").is_none());
    assert!(file(&report, "src/out/tests.rs")
        .get("inline_tests")
        .is_none());
}

/// The legend qualifies the Prod. and Test/prod figures, so default tables
/// show it with the rows it qualifies; Markdown keeps its note.
#[test]
fn tables_and_markdown_name_inline_test_code() {
    let temp = tempfile::tempdir().unwrap();
    project(temp.path());
    let legend = " Production in 'Rust' includes 8 code lines of inline #[cfg(test)] tests in 1 file. 'locmetry --files --output -' names them.\n";
    for args in [
        &["--format", "table", "--no-color"][..],
        &["--format", "table", "--no-color", "--verbose"],
    ] {
        let table = stdout(run(temp.path(), args));
        assert_eq!(table.matches(legend).count(), 1, "{args:?}\n{table}");
        assert!(!table.contains("Inline tests:"), "{table}");
    }
    for args in [
        &["--format", "table", "--no-color", "--tables-only"][..],
        &["--format", "table", "--no-color", "--table-purpose", "test"],
    ] {
        let table = stdout(run(temp.path(), args));
        assert!(!table.contains("inline #[cfg(test)]"), "{args:?}\n{table}");
    }
    let markdown = stdout(run(temp.path(), &["--format", "markdown"]));
    assert!(
        markdown.contains(
            "Inline tests: production includes 8 code lines in Rust `#[cfg(test)]` items (1 file)."
        ),
        "{markdown}"
    );
}

/// A project whose only `#[cfg(test)]` item is an out-of-line declaration
/// has zero inline test code, so no report mentions inline tests.
#[test]
fn zero_inline_test_code_produces_no_note_or_field() {
    let temp = tempfile::tempdir().unwrap();
    write_all(
        temp.path(),
        &[
            ("Cargo.toml", "[package]\nname = \"demo\"\n"),
            (
                "src/lib.rs",
                "pub fn lib() {}\n\n#[cfg(test)]\nmod tests;\n",
            ),
            ("src/tests.rs", "#[test]\nfn lib() {}\n"),
        ],
    );
    let report = report(temp.path());
    assert!(report["summary"].get("inline_tests").is_none());
    let table = stdout(run(
        temp.path(),
        &["--format", "table", "--no-color", "--verbose"],
    ));
    assert!(!table.contains("inline #[cfg(test)]"), "{table}");
    let markdown = stdout(run(temp.path(), &["--format", "markdown"]));
    assert!(!markdown.contains("Inline tests:"), "{markdown}");
}

/// Saved reports must keep inline test counts inside non-test source: per
/// file, per purpose, and as the sum of the file records.
#[test]
fn saved_reports_with_inconsistent_inline_tests_are_rejected() {
    let temp = tempfile::tempdir().unwrap();
    project(temp.path());
    let saved = report(temp.path());
    let rejected = |change: &dyn Fn(&mut Value), message: &str| {
        let mut value = saved.clone();
        change(&mut value);
        let typed: MeasurementReport = serde_json::from_value(value).unwrap();
        let error = typed.validate().unwrap_err().to_string();
        assert!(error.contains(message), "{error}");
    };
    let index = |value: &Value, path: &str| {
        value["files"]
            .as_array()
            .unwrap()
            .iter()
            .position(|file| file["path"] == path)
            .unwrap()
    };
    rejected(
        &|value| {
            value["summary"]["inline_tests"]["production"] =
                json!({"files": 1, "code": 999, "comments": 2, "blanks": 1, "lines": 1002})
        },
        "inline_tests must lie within non-test source",
    );
    rejected(
        &|value| {
            value["summary"]["inline_tests"]["test"] =
                json!({"files": 1, "code": 1, "comments": 0, "blanks": 0, "lines": 1})
        },
        "inline_tests must lie within non-test source",
    );
    rejected(
        &|value| {
            let test = index(value, "src/out/tests.rs");
            value["files"][test]["inline_tests"] =
                json!({"code": 1, "comments": 0, "blanks": 0, "lines": 1});
        },
        "file inline_tests must lie within non-test source counts",
    );
    rejected(
        &|value| {
            let lib = index(value, "src/lib.rs");
            value["files"][lib]
                .as_object_mut()
                .unwrap()
                .remove("inline_tests");
        },
        "files conflict with summary inline_tests",
    );
}
