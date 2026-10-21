use crate::common;
use common::write_all;
use std::{fs, path::Path, process::Output};

fn run(root: &Path, args: &[&str]) -> Output {
    common::locmetry()
        .current_dir(root)
        .env("XDG_CONFIG_HOME", root.join("absent-user-config"))
        .env("COLUMNS", "200")
        .args(args)
        .output()
        .unwrap()
}

fn stdout(output: Output) -> String {
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn project(root: &Path) {
    write_all(
        root,
        &[
            (
                "src/lib.rs",
                "// lib\npub fn lib() {}\n\npub fn more() {}\n",
            ),
            ("tests/lib.rs", "#[test]\nfn lib() {}\n"),
            ("scripts/tool.py", "print(1)\n"),
            ("README.md", "# Example\n"),
        ],
    );
}

/// Locks the contract by its oracle, the full report: the bare view keeps
/// only lines of that report, in order, and drops every line outside a
/// table, so it must contain each table's rows but no legend, note or footer.
fn assert_tables_only(full: &str, bare: &str, removed: &[&str]) {
    let mut remaining = full.lines();
    for line in bare.lines() {
        assert!(
            remaining.any(|candidate| candidate == line),
            "{line:?} is not an ordered line of the full report:\n{full}\n---\n{bare}"
        );
    }
    for text in removed {
        assert!(full.contains(text), "{text:?} missing from full report");
        assert!(!bare.contains(text), "{text:?} kept:\n{bare}");
    }
    assert!(!bare.starts_with('\n') && bare.ends_with('\n') && !bare.ends_with("\n\n"));
}

#[test]
fn root_tables_keep_every_table_without_legends_notes_or_footer() {
    let temp = tempfile::tempdir().unwrap();
    project(temp.path());
    let args = [
        "--format",
        "table",
        "--no-color",
        "--show-bars",
        "--show-folders",
        "--show-categories",
        "--table-language",
        "rust",
    ];
    let full = stdout(run(temp.path(), &args));
    let bare = stdout(run(temp.path(), &[&args[..], &["--tables-only"]].concat()));
    assert_tables_only(
        &full,
        &bare,
        &[
            "Lines = Code + Comments + Blanks",
            "Not shown (language): Python",
            "Bars:",
            "Rows of one purpose",
            "Root:",
            "Measured at:",
        ],
    );
    for row in [" Language", " Folder", " Category", " Total", " Rust", "▓"] {
        assert!(bare.contains(row), "{row:?} missing:\n{bare}");
    }
}

#[test]
fn workspace_tables_drop_the_share_legend_and_roots_footer() {
    let temp = tempfile::tempdir().unwrap();
    project(&temp.path().join("api"));
    project(&temp.path().join("web"));
    let args = [
        "workspace",
        "--root",
        "api",
        "--root",
        "web",
        "--format",
        "table",
        "--no-color",
        "--no-bars",
    ];
    let full = stdout(run(temp.path(), &args));
    let bare = stdout(run(temp.path(), &[&args[..], &["--tables-only"]].concat()));
    assert_tables_only(
        &full,
        &bare,
        &["Share = Code / workspace Code", "Roots:", "Measured at:"],
    );
    for row in [" Repository", " api", " web"] {
        assert!(bare.contains(row), "{row:?} missing:\n{bare}");
    }
}

#[test]
fn a_printed_overview_is_bare_while_the_saved_report_stays_complete() {
    let temp = tempfile::tempdir().unwrap();
    project(&temp.path().join("project"));
    let report = temp.path().join("report.json");
    let bare = stdout(run(
        temp.path(),
        &[
            "project",
            "--output",
            report.to_str().unwrap(),
            "--print",
            "table",
            "--no-color",
            "--tables-only",
        ],
    ));
    assert!(bare.contains(" Language") && !bare.contains("Measured at:"));
    assert!(fs::read_to_string(report)
        .unwrap()
        .contains("measured_at_unix_seconds"));
}

#[test]
fn comparison_tables_drop_title_ratio_notes_findings_and_dates() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    project(&root);
    let save = |name: &str| {
        let path = temp.path().join(name);
        let path_arg = path.to_str().unwrap();
        stdout(run(&root, &["--output", path_arg]));
        path
    };
    let before = save("before.json");
    write_all(&root, &[("src/new.rs", "pub fn new() {}\n")]);
    let after = save("after.json");
    let args = [
        "compare",
        before.to_str().unwrap(),
        after.to_str().unwrap(),
        "--format",
        "table",
        "--no-color",
    ];
    let full = stdout(run(temp.path(), &args));
    let bare = stdout(run(temp.path(), &[&args[..], &["--tables-only"]].concat()));
    assert_tables_only(
        &full,
        &bare,
        &[
            "Net change:",
            "Test/prod (Code):",
            "Measurement notes",
            "Before measured at:",
            "After measured at:",
        ],
    );
    for row in [" Kind", " Purpose", " Language", " Rust"] {
        assert!(bare.contains(row), "{row:?} missing:\n{bare}");
    }
}

#[test]
fn verbose_conflicts_and_other_formats_warn() {
    let temp = tempfile::tempdir().unwrap();
    project(&temp.path().join("project"));
    let conflict = run(
        temp.path(),
        &["project", "--format", "table", "--verbose", "--tables-only"],
    );
    assert_eq!(conflict.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&conflict.stderr).contains("cannot be used with"));
    let json = run(
        temp.path(),
        &["project", "--format", "json", "--tables-only"],
    );
    assert!(json.status.success());
    assert!(String::from_utf8_lossy(&json.stderr)
        .contains("--tables-only has no effect here; it applies only to table output"));
    let report = temp.path().join("report.json");
    stdout(run(
        temp.path(),
        &["project", "--output", report.to_str().unwrap()],
    ));
    let markdown = run(
        temp.path(),
        &[
            "compare",
            report.to_str().unwrap(),
            report.to_str().unwrap(),
            "--format",
            "markdown",
            "--tables-only",
        ],
    );
    assert!(markdown.status.success());
    assert!(String::from_utf8_lossy(&markdown.stderr)
        .contains("--tables-only has no effect here; it applies only to table output"));
}
