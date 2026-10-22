//! `--table-purpose` and `--table-language` narrow table views to the selected
//! source and name what they leave out, and
//! `--show-bars=comments` charts comment density relative to the densest row (docs/PRODUCT.md, Output Routing).
use crate::common::{self, cells};
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

/// The notes under the language table name what a filter leaves out; the
/// footer keeps only location and time, so the selection is not repeated.
fn unfiltered_footer(text: &str) -> bool {
    text.lines()
        .any(|line| line.starts_with("Root: ") && !line.contains(';'))
}

/// Selecting production counts only production source: Comments become its
/// comments, other purposes leave no rows, and Comment % takes Test/prod's place.
#[test]
fn production_only_tables_count_production_source_and_show_comment_share() {
    let text = table(&[
        "--format",
        "table",
        "--show-folders",
        "--table-purpose",
        "production",
    ]);
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
        " Lines = Code + Comments + Blanks · Comment % = Comments / (Code + Comments)\n"
    ));
    assert!(text.contains(" Not shown (purposes): Test and Other (2 code lines).\n"));
    assert!(!text.contains("Test/prod"), "{text}");
    assert!(unfiltered_footer(&text), "{text}");
}

/// The ratio follows the selection: Test/prod needs both of its purposes.
#[test]
fn selected_purposes_keep_their_columns_and_choose_the_ratio() {
    let text = table(&["--format", "table", "--table-purpose", "production,test"]);
    assert_eq!(
        cells(&text, "Language").join(" "),
        "Language Files Lines Code Comments Blanks Prod. Test Test/prod"
    );
    assert_eq!(
        cells(&text, "Rust"),
        ["Rust", "2", "5", "3", "2", "0", "2", "1", "0.50"]
    );
    assert!(text.contains("Production + Test = Code · Test/prod = Test / Production (Code)"));
    assert!(text.contains(" Not shown (purpose): Other (1 code line).\n"));

    let text = table(&["--format", "table", "--table-purpose", "other,test"]);
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
        body(&[
            "--format",
            "table",
            "--table-purpose",
            "production,test,other"
        ]),
        body(&["--format", "table"])
    );
}

/// A language selection counts only those languages' source in every table,
/// matches names in any case, and names each excluded language with the code
/// it held, so the shown totals reconcile with the full measurement.
#[test]
fn language_selection_counts_only_those_languages_and_names_the_rest() {
    let text = table(&[
        "--format",
        "table",
        "--show-folders",
        "--table-language",
        "rust",
    ]);
    let all = ["Rust", "2", "5", "3", "2", "0", "2", "1", "0", "0.50"];
    assert_eq!(cells(&text, "Rust"), all);
    assert_eq!(cells(&text, "Total")[1..], all[1..]);
    assert!(
        !text.contains("\n Python ") && !text.contains("\n proto "),
        "{text}"
    );
    assert_eq!(cells(&text, "src")[1..3], ["3", "2"]);
    assert!(text.contains(" Not shown (language): Python (1 code line).\n"));
    assert!(unfiltered_footer(&text), "{text}");
}

/// Purposes filter first and languages then, so the two notes' excluded code
/// and the shown total add up to the full measurement: 1 + 1 + 2 = 4.
#[test]
fn purpose_and_language_notes_reconcile_with_the_full_total() {
    let text = table(&[
        "--format",
        "table",
        "--table-purpose",
        "production,other",
        "--table-language",
        "Python",
    ]);
    assert_eq!(cells(&text, "Total")[3], "1");
    assert!(text.contains(" Not shown (purpose): Test (1 code line).\n"));
    assert!(text.contains(" Not shown (language): Rust (2 code lines).\n"));
    assert!(unfiltered_footer(&text), "{text}");
}

/// An embedded language keeps only its own lines; the host file it lives in
/// counts no file for it, as in the unfiltered language table. Excluded
/// languages are named largest first, ties by name: HTML and Rust hold 3.
#[test]
fn embedded_language_selection_keeps_only_the_embedded_lines() {
    let temp = project();
    let root = temp.path().join("project");
    common::write(
        &root,
        "web/page.html",
        "<p>Hello</p>\n<script>\nconst x = 1;\n</script>\n",
    );
    let full = String::from_utf8(run(&root, &["--format", "table"], "80").stdout).unwrap();
    let text = String::from_utf8(
        run(
            &root,
            &["--format", "table", "--table-language", "JavaScript"],
            "80",
        )
        .stdout,
    )
    .unwrap();
    assert_eq!(cells(&text, "JavaScript")[1..4], ["0", "1", "1"]);
    assert_eq!(cells(&text, "Total")[1..4], ["0", "1", "1"]);
    let excluded: u64 = cells(&full, "Total")[3]
        .replace(',', "")
        .parse::<u64>()
        .unwrap()
        - 1;
    assert!(
        text.contains(&format!(
            " Not shown (languages): HTML, Rust and Python ({excluded} code lines).\n"
        )),
        "{text}"
    );
}

/// Naming every measured language selects everything, as naming every purpose does.
#[test]
fn selecting_every_measured_language_is_no_selection() {
    let temp = project();
    let root = temp.path().join("project");
    let body = |args: &[&str]| {
        let text = String::from_utf8(run(&root, args, "80").stdout).unwrap();
        text.split("\nMeasured at:").next().unwrap().to_string()
    };
    assert_eq!(
        body(&["--format", "table", "--table-language", "Python,Rust"]),
        body(&["--format", "table"])
    );
}

/// A typo or a data language would silently empty every table, so both are
/// usage errors that name the value.
#[test]
fn unknown_and_non_source_languages_are_rejected() {
    let temp = project();
    let root = temp.path().join("project");
    for (value, message) in [
        ("Rusty", "unknown language \"Rusty\""),
        ("json", "JSON is data, not source"),
    ] {
        let output = run(
            &root,
            &["--format", "table", "--table-language", value],
            "80",
        );
        assert_eq!(output.status.code(), Some(1), "{value}");
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(error.contains(message), "{error}");
    }
}

#[test]
fn purpose_filters_only_tables() {
    let temp = project();
    let output = run(
        &temp.path().join("project"),
        &["--format", "json", "--table-purpose", "production"],
        "80",
    );
    assert!(output.status.success());
    assert!(String::from_utf8(output.stderr)
        .unwrap()
        .contains("--table-purpose has no effect here; it applies only to table output"));
}

/// Comment bars compare densities: the densest row fills the bar, under a
/// titled column; a bare --show-bars keeps the purpose split and never swallows a directory.
#[test]
fn comment_bars_compare_densities_under_a_titled_column() {
    let temp = project();
    let root = temp.path().join("project");
    let output = run(&root, &["--format", "table", "--show-bars=comments"], "300");
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
        "Comment % = Comments / (Code + Comments). Bars: ▓ Comment %. The densest row fills the bar."
    ));

    let bare = run(
        temp.path(),
        &["--format", "table", "--show-bars", "project"],
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
            &["--format", "table", "--show-bars=comments"],
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
        &[
            "--format",
            "table",
            "--table-purpose",
            "production",
            "--show-bars",
        ],
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
        text.contains(" Bars: ▓ Comment %. The densest row fills the bar.\n"),
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
    assert_eq!(json["schema_version"], "3");
    assert_eq!(json["summary"]["unclaimed_folders"][0]["folder"], "loose");
    assert_eq!(json["files"][0]["category"], "unclaimed");
}
