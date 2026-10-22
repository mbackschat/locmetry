//! `--table-category` narrows table views to purpose:category keys and names
//! the categories it leaves out (docs/proposals/archive/table-category-filter.md).
use crate::common::{self, cells};
use std::path::Path;

/// Code per category: production:implementation 2 (Rust), production:tooling
/// 2 and production:verification-tooling 1 (Python), test:test 1 (Rust) and
/// other:unclaimed 1 (Python); 7 in all.
fn project(root: &Path) {
    for (path, content) in [
        (
            ".locmetryrc",
            concat!(
                "version = 1\n",
                "[[rules]]\nid = \"app\"\nmatch = [\"src/**\"]\npurpose = \"production\"\ncategory = \"implementation\"\n",
                "[[rules]]\nid = \"tools\"\nmatch = [\"tools/**\"]\npurpose = \"production\"\ncategory = \"tooling\"\n",
                "[[rules]]\nid = \"gates\"\nmatch = [\"gates/**\"]\npurpose = \"production\"\ncategory = \"verification-tooling\"\n",
            ),
        ),
        ("src/lib.rs", "// doc\nfn a() {}\nfn b() {}\n"),
        ("tools/gen.py", "x = 1\ny = 2\n"),
        ("gates/size.py", "z = 3\n"),
        ("tests/it.rs", "// t\nfn t() {}\n"),
        ("proto/x.py", "w = 4\n"),
    ] {
        common::write(root, path, content);
    }
}

fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
    common::locmetry()
        .env("COLUMNS", "100")
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap()
}

fn table(args: &[&str]) -> String {
    let temp = tempfile::tempdir().unwrap();
    project(temp.path());
    let mut all = vec!["--format", "table", "--show-categories"];
    all.extend(args);
    let output = run(temp.path(), &all);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success() && stderr.is_empty(), "{stderr}");
    String::from_utf8(output.stdout).unwrap()
}

/// A bare purpose selects all its categories and a key selects one. Keeping
/// only part of Production's categories would make Test/prod compare
/// unrelated slices, so Comment % takes its place.
#[test]
fn purposes_and_exact_keys_select_categories_in_every_table() {
    let text = table(&["--table-category", "test,production:tooling"]);
    assert_eq!(
        cells(&text, "Language").join(" "),
        "Language Files Lines Code Comments Blanks Prod. Test Comment %"
    );
    assert_eq!(
        cells(&text, "Rust"),
        ["Rust", "1", "2", "1", "1", "0", "0", "1", "50.0%"]
    );
    assert_eq!(
        cells(&text, "Python"),
        ["Python", "1", "2", "2", "0", "0", "2", "0", "0.0%"]
    );
    assert_eq!(
        cells(&text, "Total"),
        ["Total", "2", "4", "3", "1", "0", "2", "1", "25.0%"]
    );
    assert_eq!(cells(&text, "test:test")[1..4], ["1", "2", "1"]);
    assert_eq!(cells(&text, "production:tooling")[1..4], ["1", "2", "2"]);
    assert!(!text.contains("production:implementation  "), "{text}");
    assert!(!text.contains("Test/prod"), "{text}");
    assert!(
        text.contains(
            " Not shown (categories): production:implementation, other:unclaimed and production:verification-tooling (4 code lines).\n"
        ),
        "{text}"
    );
}

/// `*` matches any run of characters; a selection within one purpose shows
/// that purpose alone, as `--table-purpose` does.
#[test]
fn wildcards_match_within_either_part() {
    let text = table(&["--table-category", "production:*tooling*"]);
    assert_eq!(
        cells(&text, "Language").join(" "),
        "Language Files Lines Code Comments Blanks Comment %"
    );
    assert_eq!(cells(&text, "Total")[1..4], ["2", "3", "3"]);
    assert_eq!(cells(&text, "production:verification-tooling")[3], "1");
    assert!(text.contains(
        " Not shown (categories): production:implementation, other:unclaimed and test:test (4 code lines).\n"
    ));

    let text = table(&["--table-category", "*:unclaimed"]);
    assert_eq!(cells(&text, "Total")[1..4], ["1", "1", "1"]);
    assert_eq!(cells(&text, "other:unclaimed")[3], "1");
}

/// Whole purposes leave Test/prod comparing all tests with all production.
#[test]
fn whole_purposes_keep_test_per_production() {
    let text = table(&["--table-category", "test,production"]);
    assert_eq!(
        cells(&text, "Language").join(" "),
        "Language Files Lines Code Comments Blanks Prod. Test Test/prod"
    );
    assert_eq!(
        cells(&text, "Rust"),
        ["Rust", "2", "5", "3", "2", "0", "2", "1", "0.50"]
    );
    assert!(text.contains(" Not shown (category): other:unclaimed (1 code line).\n"));
}

/// Purposes filter before categories, so both notes and the shown total add
/// up to the full measurement: 2 + 2 + 3 = 7.
#[test]
fn category_and_purpose_selections_combine_and_reconcile() {
    let text = table(&[
        "--table-purpose",
        "production",
        "--table-category",
        "*:*tooling*",
    ]);
    assert_eq!(cells(&text, "Total")[3], "3");
    assert!(text.contains(" Not shown (purposes): Test and Other (2 code lines).\n"));
    assert!(text.contains(" Not shown (category): production:implementation (2 code lines).\n"));
}

#[test]
fn selecting_every_category_is_no_selection() {
    let temp = tempfile::tempdir().unwrap();
    project(temp.path());
    let body = |args: &[&str]| {
        let text = String::from_utf8(run(temp.path(), args).stdout).unwrap();
        text.split("\nMeasured at:").next().unwrap().to_string()
    };
    assert_eq!(
        body(&["--format", "table", "--table-category", "*"]),
        body(&["--format", "table"])
    );
}

/// A purpose typo would silently empty every table, so it is a usage error;
/// a category is the project's own name, so an unmatched one is not.
#[test]
fn unknown_purposes_are_rejected_and_unmatched_categories_are_not() {
    let temp = tempfile::tempdir().unwrap();
    project(temp.path());
    for value in ["prod:tooling", "tooling", "production:"] {
        let output = run(
            temp.path(),
            &["--format", "table", "--table-category", value],
        );
        assert_eq!(output.status.code(), Some(1), "{value}");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains(&format!("'{value}'")), "{error}");
    }
    let text = table(&["--table-category", "production:nope"]);
    assert!(text.contains("(7 code lines)."), "{text}");
}

#[test]
fn category_selection_filters_only_tables() {
    let temp = tempfile::tempdir().unwrap();
    project(temp.path());
    let output = run(
        temp.path(),
        &["--format", "json", "--table-category", "test"],
    );
    assert!(output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("'--table-category' has no effect here. It applies only to table output"));
}

#[test]
fn workspace_member_rows_count_only_selected_categories() {
    let temp = tempfile::tempdir().unwrap();
    for member in ["web", "api"] {
        project(&temp.path().join(member));
    }
    let output = run(
        temp.path(),
        &[
            "workspace",
            "--root",
            "web",
            "--root",
            "api",
            "--format",
            "table",
            "--table-category",
            "production:*tooling*",
        ],
    );
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert_eq!(cells(&text, "web")[1..3], ["3", "3"]);
    assert_eq!(cells(&text, "api")[1..3], ["3", "3"]);
}
