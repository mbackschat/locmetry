//! `--purpose` narrows table views to the selected purposes' source, and
//! `--bars=comments` charts Code against Comments (docs/PRODUCT.md, Output Routing).
use crate::common;
use std::path::Path;

fn project() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    for (path, content) in [
        (
            ".locmetryrc",
            "version = 1\n[[rules]]\nid = \"app\"\nmatch = [\"src/**\"]\npurpose = \"production\"\n",
        ),
        ("src/lib.rs", "// doc\nfn a() {}\nfn b() {}\n"),
        ("tests/it.rs", "// t\nfn t() {}\n"),
        ("proto/x.py", "x = 1\n"),
    ] {
        common::write(&temp.path().join("project"), path, content);
    }
    temp
}

fn run(cwd: &Path, args: &[&str], columns: &str) -> std::process::Output {
    common::locmetry()
        .env("COLUMNS", columns)
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap()
}

fn table(args: &[&str]) -> String {
    let temp = project();
    let output = run(&temp.path().join("project"), args, "80");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

/// Cells of the first line starting with `label`, without separators or commas.
fn cells(text: &str, label: &str) -> Vec<String> {
    let line = text
        .lines()
        .find(|line| line.starts_with(&format!(" {label} ")))
        .unwrap_or_else(|| panic!("no {label} row in:\n{text}"));
    line.replace('│', " ")
        .split_whitespace()
        .map(String::from)
        .collect()
}

/// Selecting production counts only production source: Comments become its
/// comments, other purposes leave no rows, and Comment % takes Test/prod's place.
#[test]
fn production_only_tables_count_production_source_and_show_comment_share() {
    let text = table(&["--format", "table", "--folders", "--purpose", "production"]);
    let header = |label| cells(&text, label).join(" ");
    assert_eq!(
        header("Language"),
        "Language Files Lines Code Comments Blanks Comment %"
    );
    assert_eq!(
        cells(&text, "Rust"),
        ["Rust", "1", "3", "2", "1", "0", "33.3%"]
    );
    assert_eq!(
        cells(&text, "Total"),
        ["Total", "1", "3", "2", "1", "0", "33.3%"]
    );
    assert!(!text.contains(" Python "), "{text}");
    assert_eq!(header("Folder"), "Folder Lines Code Comment %");
    assert_eq!(cells(&text, "src"), ["src", "3", "2", "33.3%"]);
    assert!(
        !text.contains("\n tests ") && !text.contains("\n proto "),
        "{text}"
    );
    assert!(text.contains(
        " Lines = Code + Comments + Blanks; Comment % = Comments / (Code + Comments).\n"
    ));
    assert!(text.contains(" Purposes shown: Production; Test and Other excluded.\n"));
    assert!(!text.contains("Test/prod"), "{text}");
    assert!(
        text.contains("; purposes: production\nMeasured at: "),
        "{text}"
    );
}

/// The ratio follows the selection: Test/prod needs both of its purposes.
#[test]
fn selected_purposes_keep_their_columns_and_choose_the_ratio() {
    let text = table(&["--format", "table", "--purpose", "production,test"]);
    assert_eq!(
        cells(&text, "Language").join(" "),
        "Language Files Lines Code Comments Blanks Prod. Test Test/prod"
    );
    assert_eq!(
        cells(&text, "Rust"),
        ["Rust", "2", "5", "3", "2", "0", "2", "1", "0.50"]
    );
    assert!(text.contains("Production + Test = Code; Test/prod = Test / Production."));
    assert!(text.contains(" Purposes shown: Production and Test; Other excluded.\n"));

    let text = table(&["--format", "table", "--purpose", "other,test"]);
    assert_eq!(
        cells(&text, "Language").join(" "),
        "Language Files Lines Code Comments Blanks Test Other Comment %"
    );
    assert_eq!(
        cells(&text, "Python"),
        ["Python", "1", "1", "1", "0", "0", "0", "1", "0.0%"]
    );
}

/// Naming every purpose selects everything: neither the table nor the footer
/// may present the complete totals as a selection.
#[test]
fn selecting_every_purpose_is_no_selection() {
    let temp = project();
    let root = temp.path().join("project");
    let body = |args: &[&str]| {
        let text = String::from_utf8(run(&root, args, "80").stdout).unwrap();
        text.split("\nMeasured at:").next().unwrap().to_string()
    };
    assert_eq!(
        body(&["--format", "table", "--purpose", "production,test,other"]),
        body(&["--format", "table"])
    );
}

#[test]
fn purpose_filters_only_tables() {
    let temp = project();
    let output = run(
        &temp.path().join("project"),
        &["--format", "json", "--purpose", "production"],
        "80",
    );
    assert!(output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("--purpose has no effect here; it applies only to table output"));
}

/// Comment bars compare densities: the densest row fills the bar, under a
/// titled column; a bare --bars keeps the purpose split and never swallows a directory.
#[test]
fn comment_bars_compare_densities_under_a_titled_column() {
    let temp = project();
    let root = temp.path().join("project");
    let output = run(&root, &["--format", "table", "--bars=comments"], "300");
    let text = String::from_utf8(output.stdout).unwrap();
    let rust = text
        .lines()
        .find(|line| line.starts_with(" Rust "))
        .unwrap();
    assert!(
        rust.ends_with(&format!("│     40.0%  {}", "▓".repeat(40))),
        "{rust}"
    );
    let header = text
        .lines()
        .find(|line| line.starts_with(" Language "))
        .unwrap();
    assert!(header.ends_with("│ Comment %  Comment density"), "{header}");
    assert!(text.contains(
        "Comment % = Comments / (Code + Comments). Bars: ▓ Comment %; the densest row fills the bar."
    ));

    let bare = run(
        temp.path(),
        &["--format", "table", "--bars", "project"],
        "300",
    );
    assert!(
        bare.status.success(),
        "{}",
        String::from_utf8_lossy(&bare.stderr)
    );
    let text = String::from_utf8(bare.stdout).unwrap();
    assert!(
        text.contains("Bars: █ Production  ▓ Test  ░ Other"),
        "{text}"
    );
}

/// Inline comment bars need room for their 15-character column title, more
/// than the 10-cell bar minimum, so the header line never outgrows the rows.
#[test]
fn inline_comment_bars_need_room_for_their_title() {
    let temp = project();
    let root = temp.path().join("project");
    let table = |columns: usize| {
        let output = run(
            &root,
            &["--format", "table", "--bars=comments"],
            &columns.to_string(),
        );
        String::from_utf8(output.stdout).unwrap()
    };
    // The Total row has no bar, so its length is the language table's width.
    let width = table(300)
        .lines()
        .find(|line| line.starts_with(" Total"))
        .unwrap()
        .chars()
        .count();
    let at = table(width + 2 + 15);
    let rust = at.lines().find(|line| line.starts_with(" Rust ")).unwrap();
    assert!(rust.ends_with(&format!("  {}", "▓".repeat(15))), "{rust}");
    assert!(!at.contains("\n Comment density"), "{at}");
    let below = table(width + 2 + 14);
    assert!(below.contains("\n Comment density"), "{below}");
}

/// One purpose leaves purpose bars nothing to split, so bars follow the
/// table's Comment % column and chart comment density.
#[test]
fn single_purpose_bars_chart_comments() {
    let temp = project();
    let output = run(
        &temp.path().join("project"),
        &["--format", "table", "--purpose", "production", "--bars"],
        "300",
    );
    let text = String::from_utf8(output.stdout).unwrap();
    let rust = text
        .lines()
        .find(|line| line.starts_with(" Rust "))
        .unwrap();
    assert!(
        rust.ends_with(&format!("│     33.3%  {}", "▓".repeat(40))),
        "{rust}"
    );
    assert!(
        text.contains(" Bars: ▓ Comment %; the densest row fills the bar.\n"),
        "{text}"
    );
}

/// Locks the 0.5 vocabulary across every output format: the neutral purpose
/// is `other`, undecided code is `unclaimed`, and the retired word is gone.
#[test]
fn every_format_uses_other_and_unclaimed_instead_of_unclassified() {
    let temp = tempfile::tempdir().unwrap();
    common::write(temp.path(), "loose/a.py", "a = 1\n");
    let root = temp.path().to_str().unwrap();
    for format in ["table", "markdown", "json", "headline"] {
        let output = common::locmetry()
            .args([root, "--no-user-config", "--format", format])
            .output()
            .unwrap();
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(
            !text.to_lowercase().contains("unclassified"),
            "{format}: {text}"
        );
        let expected = match format {
            "table" | "markdown" => "Other",
            _ => "\"other\"",
        };
        assert!(text.contains(expected), "{format}: {text}");
    }
    let json = common::json(
        &common::locmetry()
            .args([root, "--no-user-config", "--format", "json", "--files"])
            .output()
            .unwrap(),
        0,
    );
    assert_eq!(json["schema_version"], "2");
    assert_eq!(json["summary"]["unclaimed_folders"][0]["folder"], "loose");
    assert_eq!(json["files"][0]["category"], "unclaimed");
}
