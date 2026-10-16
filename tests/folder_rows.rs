use serde_json::Value;
use std::{
    collections::BTreeMap,
    fs,
    path::Path,
    process::{Command, Output},
};

fn write(root: &Path, path: &str, content: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}
fn source(root: &Path, path: &str, lines: usize) {
    write(root, path, &"fn item() {}\n".repeat(lines));
}
fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    write(temp.path(), ".locmetryrc", "version=1\n[[rules]]\nid='prod'\nmatch=['**/p.rs']\npurpose='production'\n[[rules]]\nid='test'\nmatch=['**/t.rs']\npurpose='test'\n");
    temp
}
fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .args(["--root", root.to_str().unwrap(), "--no-user-config"])
        .args(args)
        .env("NO_COLOR", "1")
        .output()
        .unwrap()
}
fn table(root: &Path, budget: usize, args: &[&str]) -> String {
    let out = run(
        root,
        &[
            &["--format", "table", "--folder-rows", &budget.to_string()][..],
            args,
        ]
        .concat(),
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(!text.contains('\x1b'));
    text
}
fn folder_rows(text: &str) -> Vec<(String, Vec<String>)> {
    let mut result = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        if line.trim_start().starts_with("Folder ") {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        let plain = line.replace('│', " ").replace(',', "");
        let cells: Vec<_> = plain.split_whitespace().collect();
        if cells.len() < 6 {
            continue;
        }
        let (label, values) = cells.split_at(cells.len() - 5);
        if values[0].parse::<u64>().is_err() {
            continue;
        }
        let label = label.join(" ");
        result.push((
            label.clone(),
            values.iter().map(|s| s.to_string()).collect(),
        ));
        if label == "Total" {
            break;
        }
    }
    result
}
fn counts(text: &str) -> BTreeMap<String, Vec<String>> {
    folder_rows(text).into_iter().collect()
}
fn values(code: u64, prod: u64, test: u64, unclassified: u64, ratio: &str) -> Vec<String> {
    vec![
        code.to_string(),
        prod.to_string(),
        test.to_string(),
        unclassified.to_string(),
        ratio.into(),
    ]
}

#[test]
fn adaptive_rows_partition_purposes_direct_files_and_weighted_ratios() {
    let temp = fixture();
    let root = temp.path();
    source(root, "packages/core/p.rs", 4);
    source(root, "packages/core/t.rs", 2);
    source(root, "packages/web/p.rs", 2);
    source(root, "packages/web/t.rs", 4);
    source(root, "packages/tools/unknown.rs", 1);
    source(root, "packages/p.rs", 1);
    source(root, "examples/p.rs", 3);
    source(root, "p.rs", 1);
    write(root, "packages/core/readme.md", "# Document\n");
    write(root, "packages/core/data.json", "{}\n");
    let text = table(root, 7, &[]);
    assert_eq!(
        counts(&text),
        BTreeMap::from([
            ("packages/core".into(), values(6, 4, 2, 0, "0.50")),
            ("packages/web".into(), values(6, 2, 4, 0, "2.00")),
            ("packages/tools".into(), values(1, 0, 0, 1, "-")),
            ("packages (direct files)".into(), values(1, 1, 0, 0, "0.00")),
            ("examples".into(), values(3, 3, 0, 0, "0.00")),
            ("(root files)".into(), values(1, 1, 0, 0, "0.00")),
            ("Total".into(), values(18, 11, 6, 1, "0.55")),
        ])
    );
    assert!(text.contains("Adaptive folder detail; rows cover measured source once."));
    let named = table(root, 7, &["--sort", "name"]);
    assert_eq!(counts(&text), counts(&named));
    assert_ne!(folder_rows(&text), folder_rows(&named));
    assert_eq!(text, table(root, 7, &[]));
}

#[test]
fn wide_folders_use_scoped_remainders_but_never_hide_one_child() {
    let temp = fixture();
    let root = temp.path();
    for (child, n) in [("a", 8), ("b", 4), ("c", 2), ("d", 1)] {
        source(root, &format!("packages/{child}/p.rs"), n);
    }
    let rows = counts(&table(root, 3, &[]));
    assert_eq!(
        rows,
        BTreeMap::from([
            ("packages/a".into(), values(8, 8, 0, 0, "0.00")),
            ("packages/b".into(), values(4, 4, 0, 0, "0.00")),
            (
                "packages (other 2 child folders)".into(),
                values(3, 3, 0, 0, "0.00")
            ),
            ("Total".into(), values(15, 15, 0, 0, "0.00")),
        ])
    );
    assert!(counts(&table(root, 2, &[])).contains_key("packages"));
    assert!(!table(root, 4, &[]).contains("other "));
    fs::remove_dir_all(root.join("packages/d")).unwrap();
    assert!(!table(root, 3, &[]).contains("other "));
    source(root, "packages/p.rs", 1);
    assert!(counts(&table(root, 3, &[])).contains_key("packages"));
    assert!(counts(&table(root, 4, &[])).contains_key("packages (direct files)"));
    source(root, "packages/d/p.rs", 1);
    let rows = counts(&table(root, 4, &[]));
    assert_eq!(
        rows["packages (other 2 child folders)"],
        values(3, 3, 0, 0, "0.00")
    );
    assert_eq!(rows["packages (direct files)"], values(1, 1, 0, 0, "0.00"));
    assert_eq!(rows["Total"], values(16, 16, 0, 0, "0.00"));
}

#[test]
fn collapse_applies_to_initial_and_revealed_chains_and_keeps_wide_initial_rows() {
    let temp = fixture();
    let root = temp.path();
    source(root, "src/main/java/com/acme/p.rs", 5);
    source(root, "src/test/java/com/acme/t.rs", 2);
    source(root, "extra/deep/p.rs", 1);
    let small = counts(&table(root, 1, &[]));
    assert!(small.contains_key("src"));
    assert!(small.contains_key("extra/deep"));
    assert_eq!(small.len(), 3);
    let expanded = counts(&table(root, 3, &[]));
    assert!(expanded.contains_key("src/main/java/com/acme"));
    assert!(expanded.contains_key("src/test/java/com/acme"));
    assert_eq!(expanded["src/test/java/com/acme"], values(2, 0, 2, 0, "-"));
    assert_eq!(expanded["Total"], values(8, 6, 2, 0, "0.33"));
    assert!(table(root, 1, &[]).contains("--folder-rows"));
}

#[test]
fn largest_fitting_folder_expands_with_stable_name_ties() {
    let temp = fixture();
    let root = temp.path();
    for parent in ["a", "b"] {
        for child in ["x", "y"] {
            source(root, &format!("{parent}/{child}/p.rs"), 2);
        }
    }
    let rows = counts(&table(root, 3, &[]));
    assert!(rows.contains_key("a/x") && rows.contains_key("a/y") && rows.contains_key("b"));
    assert_eq!(rows, counts(&table(root, 3, &["--sort", "name"])));
    source(root, "huge/one/p.rs", 10);
    source(root, "huge/two/p.rs", 10);
    source(root, "huge/three/p.rs", 10);
    let rows = counts(&table(root, 4, &[]));
    assert!(rows.contains_key("huge"));
    assert!(rows.contains_key("a/x"));
}

#[test]
fn subfolder_detail_keeps_scope_zero_code_and_coverage() {
    let temp = fixture();
    let root = temp.path();
    source(root, "web/src/p.rs", 1);
    source(root, "web/src/ui/button/p.rs", 2);
    write(root, "web/src/zero/deeper/p.rs", "// comment\n\n");
    source(root, "web/src-old/p.rs", 50);
    write(root, "outside.ipynb", "broken");
    let rows = counts(&table(root, 10, &["--subfolder", "web/src"]));
    assert_eq!(
        rows,
        BTreeMap::from([
            ("web/src (direct files)".into(), values(1, 1, 0, 0, "0.00")),
            ("web/src/ui/button".into(), values(2, 2, 0, 0, "0.00")),
            ("web/src/zero/deeper".into(), values(0, 0, 0, 0, "-")),
            ("Total".into(), values(3, 3, 0, 0, "0.00")),
        ])
    );
    write(root, "web/src/bad.ipynb", "broken");
    let plain = run(root, &["--format", "table", "--subfolder", "web/src"]);
    let adaptive = run(
        root,
        &[
            "--format",
            "table",
            "--subfolder",
            "web/src",
            "--folder-rows",
            "10",
        ],
    );
    assert_eq!(adaptive.status.code(), Some(2));
    assert_eq!(plain.status.code(), Some(2));
    assert_eq!(plain.stderr, adaptive.stderr);
    assert!(String::from_utf8_lossy(&adaptive.stdout).contains("web/src/bad.ipynb"));
}

#[test]
fn adaptive_view_preserves_saved_reports_and_unclaimed_notes() {
    let temp = fixture();
    let root = temp.path();
    source(root, "packages/a/unknown.rs", 5);
    source(root, "packages/b/unknown.rs", 2);
    source(root, "packages/unknown.rs", 1);
    let dest = tempfile::tempdir().unwrap();
    let path = dest.path().join("report.json");
    let mut reports = Vec::new();
    let mut notes = Vec::new();
    for args in [vec![], vec!["--folder-rows", "10"]] {
        let out = run(
            root,
            &[
                &["--format", "table", "--output", path.to_str().unwrap()][..],
                &args,
            ]
            .concat(),
        );
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        let text = String::from_utf8(out.stdout).unwrap();
        notes.push(text[text.find("Unclaimed source:").unwrap()..].to_string());
        let mut value: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
        value["metadata"]
            .as_object_mut()
            .unwrap()
            .remove("measured_at_unix_seconds");
        reports.push(value);
    }
    assert_eq!(reports[0], reports[1]);
    assert_eq!(notes[0], notes[1]);
}

#[test]
fn invalid_options_fail_before_writing_with_format_remedy() {
    let temp = fixture();
    source(temp.path(), "p.rs", 1);
    let dest = tempfile::tempdir().unwrap();
    let path = dest.path().join("report.json");
    for args in [
        vec![],
        vec!["--format", "json"],
        vec!["--format", "markdown"],
    ] {
        let out = run(
            temp.path(),
            &[
                &["--folder-rows", "10", "--output", path.to_str().unwrap()][..],
                &args,
            ]
            .concat(),
        );
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stdout.is_empty());
        assert!(String::from_utf8_lossy(&out.stderr).contains("--format table"));
        assert!(!path.exists());
    }
    for args in [
        vec!["--folder-rows"],
        vec!["--folder-rows", "0"],
        vec!["--folder-rows", "acme"],
        vec!["--folder-rows", "-1"],
        vec!["--folder-rows", "10", "--schema", "report"],
        vec!["--folder-rows", "10", "--output", "-"],
    ] {
        let out = run(temp.path(), &args);
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stdout.is_empty());
        assert!(String::from_utf8_lossy(&out.stderr).contains("--folder-rows"));
    }
    for args in [
        vec![
            "workspace",
            "--root",
            temp.path().to_str().unwrap(),
            "--folder-rows",
            "10",
        ],
        vec![
            "--folder-rows",
            "10",
            "workspace",
            "--root",
            temp.path().to_str().unwrap(),
        ],
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_locmetry"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stdout.is_empty());
        assert!(String::from_utf8_lossy(&out.stderr).contains("--folder-rows"));
    }
}

#[test]
fn empty_source_and_control_characters_do_not_corrupt_tables() {
    let temp = fixture();
    write(temp.path(), "notes/readme.md", "# Hello\n");
    let text = table(temp.path(), 5, &[]);
    assert!(folder_rows(&text).is_empty());
    source(temp.path(), "a\tb/deep/p.rs", 1);
    let text = table(temp.path(), 1, &[]);
    assert!(!text.contains('\t'));
    assert!(counts(&text).contains_key("a b/deep"));
}

#[test]
fn adaptive_styling_preserves_plain_alignment_and_mutes_the_legend() {
    let temp = fixture();
    source(temp.path(), "packages/alpha/p.rs", 3);
    source(temp.path(), "packages/beta/t.rs", 1);
    let mut policy = locmetry::Policy::load(&temp.path().join(".locmetryrc")).unwrap();
    policy
        .exclude_configuration(temp.path(), &temp.path().join(".locmetryrc"))
        .unwrap();
    let scan = locmetry::scan(temp.path(), &policy).unwrap();
    let rows = scan.records(&policy).unwrap();
    let summary = locmetry::build_report(&rows);
    let findings = locmetry::Findings {
        subfolder: None,
        coverage: &scan.coverage,
        modules: &[],
        submodules: &[],
        unused_entries: &[],
        unused_exclusions: &[],
        members: &[],
    };
    let render = |theme| {
        locmetry::render_table_with_options(
            "example",
            &summary,
            &rows,
            &findings,
            locmetry::TableOptions {
                categories: false,
                order: locmetry::RowOrder::Code,
                theme,
                folder_rows: Some(3),
            },
        )
    };
    let plain = render(locmetry::style::Theme::PLAIN);
    let styled = render(locmetry::style::Theme::terminal(true));
    assert!(
        styled.contains("\x1b[90mAdaptive folder detail; rows cover measured source once.\x1b[0m")
    );
    let mut unstyled = styled;
    for escape in [
        "\x1b[1m",
        "\x1b[90m",
        "\x1b[1;36m",
        "\x1b[36m",
        "\x1b[2m",
        "\x1b[33m",
        "\x1b[31m",
        "\x1b[35m",
        "\x1b[0m",
    ] {
        unstyled = unstyled.replace(escape, "");
    }
    assert_eq!(plain, unstyled);
}
