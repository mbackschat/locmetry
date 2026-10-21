use crate::common;
use common::{json, write};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Output,
};

pub(crate) fn run(args: &[&str]) -> Output {
    common::locmetry().args(args).output().unwrap()
}
pub(crate) fn measure(root: &Path) -> Value {
    let output = run(&[
        "--root",
        root.to_str().unwrap(),
        "--no-user-config",
        "--files",
        "--output",
        "-",
    ]);
    assert!(
        matches!(output.status.code(), Some(0 | 2)),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
pub(crate) fn save(root: &Path, name: &str, report: &Value) -> PathBuf {
    let path = root.join(name);
    fs::write(&path, serde_json::to_vec(report).unwrap()).unwrap();
    path
}
pub(crate) fn compare(before: &Path, after: &Path, args: &[&str]) -> Output {
    let mut command = common::locmetry();
    command
        .args(["compare", before.to_str().unwrap(), after.to_str().unwrap()])
        .args(args)
        .output()
        .unwrap()
}
pub(crate) fn change(value: &Value, before: u64, after: u64, delta: i64) {
    assert_eq!(value["before"]["code"], before);
    assert_eq!(value["after"]["code"], after);
    assert_eq!(value["delta"]["code"], delta);
}

#[test]
fn a_single_comparison_grid_omits_its_title_when_detail_grids_are_empty() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "src/main.py", "print(1)\n");
    let saved = save(temp.path(), "input.json", &measure(&root));
    let full = json(&compare(&saved, &saved, &["--format", "json"]), 0);
    let report: locmetry::compare::Comparison = serde_json::from_value(full).unwrap();
    for theme in [
        locmetry::style::Theme::PLAIN,
        locmetry::style::Theme::TERMINAL,
    ] {
        let text = locmetry::compare::render_table_with_theme(
            &report,
            theme,
            locmetry::LanguageVariants::Split,
        );
        assert!(!text.contains("▸ Totals"), "{text}");
        assert!(!text.contains("▸ Purpose"), "{text}");
        assert!(text.contains("All counted"), "{text}");
    }
    let plain = locmetry::compare::render_table_with_theme(
        &report,
        locmetry::style::Theme::PLAIN,
        locmetry::LanguageVariants::Split,
    );
    let lines: Vec<_> = plain.lines().collect();
    let group = lines
        .iter()
        .position(|line| line.contains(" Lines "))
        .unwrap();
    assert_eq!(
        lines[group - 1],
        lines[group + 2].replace('┼', "─"),
        "{plain}"
    );
    assert!(
        locmetry::compare::render_markdown(&report, locmetry::LanguageVariants::Split)
            .contains("## Totals")
    );
}

/// Locks the convention that recorded guides need no trim filter: no
/// comparison table line, styled or plain, ends in blanks.
#[test]
fn comparison_table_lines_end_at_their_last_character() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "src/main.py", "print(1)\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    write(&root, "tests/test_main.py", "assert True\n");
    let after = save(temp.path(), "after.json", &measure(&root));
    let full = json(&compare(&before, &after, &["--format", "json"]), 0);
    let report: locmetry::compare::Comparison = serde_json::from_value(full).unwrap();
    for theme in [
        locmetry::style::Theme::PLAIN,
        locmetry::style::Theme::TERMINAL,
    ] {
        let text = common::unstyled(&locmetry::compare::render_table_with_theme(
            &report,
            theme,
            locmetry::LanguageVariants::Split,
        ));
        assert!(text.contains(" Lines "), "{text}");
        for line in text.lines() {
            assert_eq!(line, line.trim_end(), "{text}");
        }
    }
}

#[test]
fn compare_reports_net_change_without_reading_the_source_checkout() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(
        &root,
        ".locmetryrc",
        "version = 1\n[[rules]]\nid = \"source\"\nmatch = [\"src/**\"]\npurpose = \"production\"\n",
    );
    write(&root, "src/main.py", "a = 1\nb = 2\n# before\n\n");
    write(&root, "tests/check.py", "assert True\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    write(&root, "src/main.py", "a = 1\nb = 2\nc = 3\n# one\n# two\n");
    let after = save(temp.path(), "after.json", &measure(&root));
    fs::remove_dir_all(root).unwrap();
    let full = json(&compare(&before, &after, &["--output", "-"]), 0);
    assert_eq!(full["document_type"], "comparison");
    assert_eq!(full["schema_version"], "3");
    change(&full["summary"]["source"], 3, 4, 1);
    assert_eq!(
        full["summary"]["source"]["delta"],
        json!({"files":0,"code":1,"comments":1,"blanks":-1,"lines":1})
    );
    change(&full["summary"]["source_purpose"]["production"], 2, 3, 1);
    change(&full["folders"]["src"]["direct"]["source"], 2, 3, 1);
    assert!(full["findings"].as_array().unwrap().is_empty());
    let reversed = json(&compare(&after, &before, &["--output", "-"]), 0);
    change(&reversed["summary"]["source"], 4, 3, -1);
    let same = json(&compare(&before, &before, &["--output", "-"]), 0);
    change(&same["summary"]["all_counted"], 3, 3, 0);
    let schema = json(&run(&["--schema", "comparison"]), 0);
    assert!(jsonschema::validator_for(&schema).unwrap().is_valid(&full));
    let headline = json(&compare(&before, &after, &[]), 0);
    for omitted in ["folders", "packages", "breakdown"] {
        assert!(headline.get(omitted).is_none());
    }
    assert_eq!(headline["summary"], full["summary"]);
    assert!(headline["before"]["metadata"].get("config").is_none());
    assert!(jsonschema::validator_for(&schema)
        .unwrap()
        .is_valid(&headline));
    for format in ["table", "markdown"] {
        let output = compare(&before, &after, &["--format", format]);
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.to_lowercase().contains("net change"));
        assert!(text.contains("+1"));
    }
}

#[test]
fn descriptor_removal_changes_data_and_purpose_without_claiming_source_deletion() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "package.json", "{\n  \"name\": \"example\"\n}\n");
    write(&root, "src/index.js", "let a = 1;\nlet b = 2;\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    fs::remove_file(root.join("package.json")).unwrap();
    let after = save(temp.path(), "after.json", &measure(&root));
    let full = json(&compare(&before, &after, &["--output", "-"]), 0);
    change(&full["summary"]["all_counted"], 5, 2, -3);
    change(&full["summary"]["source"], 2, 2, 0);
    change(&full["summary"]["source_purpose"]["production"], 2, 0, -2);
    change(&full["summary"]["source_purpose"]["other"], 0, 2, 2);
    assert_eq!(full["packages"][0]["folder"], ".");
    assert_eq!(full["packages"][0]["detector"], "npm");
    assert_eq!(full["packages"][0]["presence"], "before_only");
    assert_eq!(full["folders"]["src"]["presence"], "both");
    assert_eq!(full["presence_summary"]["packages"]["before_only"], 1);
}

#[test]
fn missing_failure_evidence_is_a_finding_and_observed_failures_are_partial() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "main.py", "print(1)\n");
    let mut historical = measure(&root);
    historical["metadata"]["coverage"]
        .as_object_mut()
        .unwrap()
        .remove("parse_failures");
    let before = save(temp.path(), "before.json", &historical);
    let full = json(&compare(&before, &before, &["--output", "-"]), 0);
    assert!(full["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["code"] == "coverage_gap"));
    write(&root, "broken.ipynb", "{}");
    let after = save(temp.path(), "after.json", &measure(&root));
    let partial = json(&compare(&before, &after, &["--output", "-"]), 2);
    change(&partial["summary"]["source"], 1, 1, 0);
    assert!(!partial["findings"].as_array().unwrap().is_empty());
}

#[test]
fn rejects_headlines_versions_inconsistent_counts_and_overwriting_inputs() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "main.py", "print(1)\n");
    let full = measure(&root);
    let before = save(temp.path(), "before.json", &full);
    for variant in ["headline", "version", "counts", "duplicate"] {
        let mut bad = full.clone();
        match variant {
            "headline" => {
                bad.as_object_mut().unwrap().remove("breakdown");
            }
            "version" => bad["schema_version"] = json!("4"),
            "counts" => bad["summary"]["source"]["lines"] = json!(100),
            "duplicate" => {
                let row = bad["breakdown"][0].clone();
                bad["breakdown"].as_array_mut().unwrap().push(row);
            }
            _ => unreachable!(),
        }
        let path = save(temp.path(), "invalid.json", &bad);
        let output = compare(&before, &path, &[]);
        assert_eq!(output.status.code(), Some(1), "{variant}");
        assert!(output.stdout.is_empty());
    }
    let bytes = fs::read(&before).unwrap();
    let output = compare(&before, &before, &["--output", before.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(fs::read(&before).unwrap(), bytes);
    let alias = temp.path().join("alias.json");
    fs::hard_link(&before, &alias).unwrap();
    assert_eq!(
        compare(&before, &before, &["--output", alias.to_str().unwrap()])
            .status
            .code(),
        Some(1)
    );
    assert_eq!(fs::read(&before).unwrap(), bytes);
}

pub(crate) fn workspace(roots: &[&Path]) -> Value {
    let mut command = common::locmetry();
    command.arg("workspace");
    for root in roots {
        command.arg("--root").arg(root);
    }
    let output = command
        .args(["--no-user-config", "--output", "-"])
        .output()
        .unwrap();
    json(&output, 0)
}

#[test]
fn workspace_arrivals_departures_and_empty_roots_reconcile_separately() {
    let temp = tempfile::tempdir().unwrap();
    let api = temp.path().join("api");
    let legacy = temp.path().join("legacy");
    let worker = temp.path().join("worker");
    let empty = temp.path().join("empty");
    write(&api, "main.py", &"print(1)\n".repeat(100));
    write(&legacy, "main.py", &"print(1)\n".repeat(40));
    write(&worker, "main.py", &"print(1)\n".repeat(30));
    fs::create_dir(&empty).unwrap();
    let before = save(temp.path(), "before.json", &workspace(&[&api, &legacy]));
    write(&api, "main.py", &"print(1)\n".repeat(120));
    let after = save(
        temp.path(),
        "after.json",
        &workspace(&[&worker, &empty, &api]),
    );
    let full = json(&compare(&before, &after, &["--output", "-"]), 0);
    change(&full["summary"]["source"], 140, 150, 10);
    for metric in ["all_counted", "source"] {
        for field in ["files", "code", "comments", "blanks", "lines"] {
            let contribution = &full["summary"]["composition"][metric];
            let sum = contribution["matched"][field].as_i64().unwrap()
                + contribution["entering"][field].as_i64().unwrap()
                - contribution["leaving"][field].as_i64().unwrap();
            assert_eq!(
                sum,
                full["summary"][metric]["delta"][field].as_i64().unwrap()
            );
        }
    }
    let composition = &full["summary"]["composition"]["source"];
    assert_eq!(composition["matched"]["code"], 20);
    assert_eq!(composition["entering"]["code"], 30);
    assert_eq!(composition["leaving"]["code"], 40);
    assert_eq!(full["members"]["empty"]["presence"], "after_only");
    assert_eq!(full["folders"]["empty"]["presence"], "after_only");
    let empty_finding = full["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["entity"]["kind"] == "folder" && f["entity"]["path"] == "empty")
        .unwrap();
    assert_eq!(empty_finding["paths"], json!(["/metadata/members"]));
    assert_eq!(
        full["presence_summary"]["members"],
        json!({"before_only":1,"after_only":2})
    );
    let headline = json(&compare(&before, &after, &[]), 0);
    assert_eq!(headline["presence_summary"], full["presence_summary"]);
    assert!(headline["findings"]
        .as_array()
        .unwrap()
        .iter()
        .all(|f| f["code"] != "presence_changed"));
    let schema = json(&run(&["--schema", "comparison"]), 0);
    assert!(jsonschema::validator_for(&schema).unwrap().is_valid(&full));
    let disjoint_before = save(temp.path(), "disjoint-before.json", &workspace(&[&legacy]));
    let disjoint_after = save(temp.path(), "disjoint-after.json", &workspace(&[&empty]));
    let disjoint = json(
        &compare(&disjoint_before, &disjoint_after, &["--output", "-"]),
        0,
    );
    change(&disjoint["summary"]["source"], 40, 0, -40);
    assert_eq!(
        disjoint["summary"]["composition"]["source"]["matched"]["code"],
        0
    );
    let single = save(temp.path(), "single.json", &measure(&api));
    let invalid = compare(&single, &after, &[]);
    assert_eq!(invalid.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&invalid.stderr).contains("single-root"));
}

#[test]
fn topology_changes_with_zero_net_code_and_coverage_only_folders_remain_visible() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "old/main.py", "print(1)\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    fs::rename(root.join("old"), root.join("new")).unwrap();
    write(&root, "assets/icon.unknown", "not counted\n");
    let after = save(temp.path(), "after.json", &measure(&root));
    let full = json(&compare(&before, &after, &["--output", "-"]), 0);
    change(&full["summary"]["source"], 1, 1, 0);
    assert_eq!(full["folders"]["old"]["presence"], "before_only");
    assert_eq!(full["folders"]["new"]["presence"], "after_only");
    assert_eq!(full["folders"]["assets"]["presence"], "after_only");
    assert!(full["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["code"] == "coverage_gap" && f["folder"] == "assets"));
    assert!(full["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["code"] == "coverage_gap"));
    assert_eq!(
        full["presence_summary"]["folders"],
        json!({"before_only":1,"after_only":2})
    );
}

#[test]
fn comparison_order_is_stable_and_exclusions_are_sets_but_rules_are_ordered() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root,".locmetryrc","version = 1\n[scope]\nexclude = [\"absent-a/**\",\"absent-b/**\"]\n[[rules]]\nid = \"first\"\nmatch = [\"a/**\"]\npurpose = \"production\"\n[[rules]]\nid = \"second\"\nmatch = [\"b/**\"]\npurpose = \"test\"\n");
    write(&root, "a/main.py", "print(1)\n");
    write(&root, "b/check.py", "assert True\n");
    let value = measure(&root);
    let before = save(temp.path(), "before.json", &value);
    let after = save(temp.path(), "after.json", &value);
    let original = json(&compare(&before, &after, &["--output", "-"]), 0);
    let mut reordered = value.clone();
    for field in ["breakdown", "files", "review"] {
        reordered[field].as_array_mut().unwrap().reverse();
    }
    save(temp.path(), "after.json", &reordered);
    assert_eq!(
        json(&compare(&before, &after, &["--output", "-"]), 0),
        original
    );
    for pointer in ["/metadata/scope/exclude", "/metadata/config/scope/exclude"] {
        reordered
            .pointer_mut(pointer)
            .unwrap()
            .as_array_mut()
            .unwrap()
            .reverse();
    }
    save(temp.path(), "after.json", &reordered);
    let changed = json(&compare(&before, &after, &["--output", "-"]), 0);
    assert!(changed["findings"].as_array().unwrap().is_empty());
    reordered["metadata"]["config"]["rules"]
        .as_array_mut()
        .unwrap()
        .reverse();
    save(temp.path(), "after.json", &reordered);
    let changed = json(&compare(&before, &after, &["--output", "-"]), 0);
    assert!(changed["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["code"] == "policy_changed"));
}

#[test]
fn signed_delta_overflow_fails_without_output() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "main.py", "");
    let before_value = measure(&root);
    let before = save(temp.path(), "before.json", &before_value);
    let mut after_value = before_value;
    fn enlarge(v: &mut Value) {
        match v {
            Value::Object(map) => {
                if map.contains_key("code") {
                    map.insert("code".into(), json!(u64::MAX));
                }
                if map.contains_key("lines") {
                    map.insert("lines".into(), json!(u64::MAX));
                }
                for child in map.values_mut() {
                    enlarge(child);
                }
            }
            Value::Array(items) => {
                for child in items {
                    enlarge(child);
                }
            }
            _ => {}
        }
    }
    enlarge(&mut after_value);
    after_value["summary"]["top_folders"]["."]["other"] = json!(u64::MAX);
    let after = save(temp.path(), "after.json", &after_value);
    let output = compare(&before, &after, &[]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8_lossy(&output.stderr).contains("signed 64-bit"));
}

#[test]
fn large_zero_net_restructures_keep_headline_presence_bounded() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    for i in 0..100 {
        write(&root, &format!("old/f{i:03}/empty.py"), "");
    }
    let before = save(temp.path(), "before.json", &measure(&root));
    fs::rename(root.join("old"), root.join("new")).unwrap();
    let after = save(temp.path(), "after.json", &measure(&root));
    let full = json(&compare(&before, &after, &["--output", "-"]), 0);
    let headline = json(&compare(&before, &after, &[]), 0);
    change(&full["summary"]["source"], 0, 0, 0);
    assert_eq!(
        full["presence_summary"]["folders"],
        json!({"before_only":101,"after_only":101})
    );
    assert_eq!(headline["presence_summary"], full["presence_summary"]);
    assert!(headline["findings"].as_array().unwrap().is_empty());
    assert_eq!(
        full["findings"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|f| f["code"] == "presence_changed")
            .count(),
        202
    );
}

#[test]
fn comparison_schema_has_a_valid_example_and_explicit_integer_bounds() {
    let schema = json(&run(&["--schema", "comparison"]), 0);
    let validator = jsonschema::validator_for(&schema).unwrap();
    let examples = schema["examples"]
        .as_array()
        .expect("agent-readable example");
    assert!(!examples.is_empty());
    for example in examples {
        assert!(
            validator.is_valid(example),
            "{:?}",
            validator.iter_errors(example).collect::<Vec<_>>()
        );
    }
    assert_eq!(
        schema["$defs"]["SignedCounts"]["properties"]["code"]["minimum"],
        json!(i64::MIN)
    );
    assert_eq!(
        schema["$defs"]["SignedCounts"]["properties"]["code"]["maximum"],
        json!(i64::MAX)
    );
}

/// The Folders grid ends in the root's source total like every other grid,
/// even when code only moves between top-level folders and the root nets zero.
#[test]
fn folder_grid_keeps_its_total_when_code_only_moves_between_folders() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "a/main.py", "print(1)\nprint(2)\n");
    write(&root, "b/util.py", "print(3)\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    write(&root, "a/main.py", "print(1)\n");
    write(&root, "b/util.py", "print(3)\nprint(2)\n");
    let after = save(temp.path(), "after.json", &measure(&root));
    let output = compare(&before, &after, &["--format", "table"]);
    let text = String::from_utf8(output.stdout).unwrap();
    let folders = text.split("\n Folder ").nth(1).unwrap();
    let total = folders
        .lines()
        .find(|line| line.starts_with(" Source total"))
        .unwrap_or_else(|| panic!("no total in:\n{text}"));
    assert_eq!(
        total
            .replace('│', " ")
            .split_whitespace()
            .collect::<Vec<_>>(),
        ["Source", "total", "3", "3", "0", "3", "3", "0"]
    );
    assert!(!folders.contains(" . ("), "{text}");
}

#[cfg(unix)]
#[test]
fn comparison_output_rejects_symlink_aliases_and_preserves_input_bytes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "main.py", "print(1)\n");
    let input = save(temp.path(), "input.json", &measure(&root));
    let alias = temp.path().join("alias.json");
    std::os::unix::fs::symlink(&input, &alias).unwrap();
    let original = fs::read(&input).unwrap();
    let output = compare(&input, &input, &["--output", alias.to_str().unwrap()]);
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("overwrite"));
    assert_eq!(fs::read(input).unwrap(), original);
}

#[test]
fn full_comparison_output_is_written_and_human_folders_keep_their_hierarchy() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "src/main.py", "print(1)\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    write(&root, "src/inner/a.py", "print(1)\n");
    write(&root, "src-other/a.py", "print(1)\n");
    let after = save(temp.path(), "after.json", &measure(&root));
    let out = temp.path().join("full.json");
    let output = compare(&before, &after, &["--output", out.to_str().unwrap()]);
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    let saved: Value = serde_json::from_slice(&fs::read(&out).unwrap()).unwrap();
    assert!(saved.get("before").is_some());
    assert!(saved.get("folders").is_some());
    let output = compare(&before, &after, &["--format", "markdown"]);
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.find("| src/inner (").unwrap() < text.find("| src-other (").unwrap());
    assert_eq!(
        compare(&before, &after, &["--output", "-", "--format", "table"])
            .status
            .code(),
        Some(0)
    );
}

#[cfg(unix)]
#[test]
fn rendering_escapes_markdown_but_preserves_plain_text_labels() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("R&D");
    write(&root, "main.py", "print(1)\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    write(
        &root,
        "<img src=x onerror=alert(1)>`|/main.py",
        "print(2)\n",
    );
    let after = save(temp.path(), "after.json", &measure(&root));
    let markdown =
        String::from_utf8(compare(&before, &after, &["--format", "markdown"]).stdout).unwrap();
    assert!(!markdown.contains("<img"));
    assert!(markdown.contains("&lt;img src=x onerror=alert(1)&gt;&#96;&#124;"));
    let table = String::from_utf8(compare(&before, &after, &["--format", "table"]).stdout).unwrap();
    assert!(table.contains("R&D"));
    assert!(!table.contains("&amp;"));
    assert!(table.contains(" <img src=x onerror=alert(1)>`| (after only) "));
}

#[test]
fn comparison_totals_fit_their_contents_and_keep_large_signed_values_aligned() {
    use locmetry::{compare::render_table_with_theme, style::Theme, Counts};
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "main.py", "print(1)\n");
    let saved = save(temp.path(), "input.json", &measure(&root));
    let full = json(&compare(&saved, &saved, &["--format", "json"]), 0);
    let mut report: locmetry::compare::Comparison = serde_json::from_value(full).unwrap();
    let totals = |report: &locmetry::compare::Comparison| {
        render_table_with_theme(report, Theme::PLAIN, locmetry::LanguageVariants::Split)
            .lines()
            .skip_while(|line| !line.contains(" Lines "))
            .take_while(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    };
    let width = |line: &str| line.chars().count();
    // The group line ends at its last name; the column headings span the table.
    let fits = |text: &str| {
        let (group, rows) = text.split_once('\n').unwrap();
        let table = width(rows.lines().next().unwrap());
        width(group) <= table && rows.lines().all(|line| width(line) == table)
    };
    let unchanged = totals(&report);
    assert!(fits(&unchanged), "{unchanged}");
    let all_counted = unchanged.lines().last().unwrap();
    assert!(all_counted.starts_with(" All counted "), "{unchanged}");
    assert!(
        all_counted.ends_with(" │        0       0      0"),
        "{unchanged}"
    );

    let change = &mut report.summary.dimensions.all_counted;
    change.before = Counts {
        files: 1,
        code: 1_u64 << 63,
        lines: 1_u64 << 63,
        ..Counts::default()
    };
    change.after = Counts::default();
    change.delta = locmetry::compare::SignedCounts {
        files: -1,
        code: i64::MIN,
        lines: i64::MIN,
        ..Default::default()
    };
    let large = totals(&report);
    let first = large.lines().next().unwrap();
    assert!(fits(&large), "{large}");
    // Separators stay in one column however wide the signed values grow.
    let separators = |line: &str| {
        line.char_indices()
            .filter(|(_, c)| matches!(c, '│' | '┼'))
            .map(|(index, _)| line[..index].chars().count())
            .collect::<Vec<_>>()
    };
    assert!(
        large
            .lines()
            .all(|line| separators(line) == separators(first)),
        "{large}"
    );
    assert!(
        large.contains("9,223,372,036,854,775,808      0  -9,223,372,036,854,775,808"),
        "{large}"
    );
}

#[test]
fn comparisons_report_changed_producer_identity() {
    let temp = tempfile::tempdir().unwrap();
    write(temp.path(), "main.py", "print(1)\n");
    let current = measure(temp.path());
    assert_eq!(
        current["metadata"]["locmetry_version"],
        env!("CARGO_PKG_VERSION")
    );
    assert_ne!(current["metadata"]["locmetry_version"], "0.1.6");
    let mut released = current.clone();
    released["metadata"]["locmetry_version"] = json!("0.1.6");
    let before = save(temp.path(), "before.json", &released);
    let after = save(temp.path(), "after.json", &current);
    let result = json(&compare(&before, &after, &[]), 0);
    assert!(result["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding["code"] == "counter_changed"));
}

#[test]
fn folder_presence_points_to_the_evidence_that_establishes_it() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    fs::create_dir(&root).unwrap();
    let before_value = measure(&root);
    let before = save(temp.path(), "before.json", &before_value);
    write(&root, "unknown/deep/file.zzzz", "unknown\n");
    write(&root, "failed/broken.ipynb", "{}");
    let after_value = measure(&root);
    let after = save(temp.path(), "after.json", &after_value);
    let result = json(&compare(&before, &after, &["--output", "-"]), 2);
    for (folder, pointer) in [
        ("unknown", "/metadata/coverage/unrecognized"),
        ("unknown/deep", "/metadata/coverage/unrecognized"),
        ("failed", "/metadata/coverage/parse_failures"),
    ] {
        let finding = result["findings"]
            .as_array()
            .unwrap()
            .iter()
            .find(|f| {
                f["code"] == "presence_changed"
                    && f["entity"]["kind"] == "folder"
                    && f["entity"]["path"] == folder
            })
            .unwrap();
        assert!(
            finding["paths"]
                .as_array()
                .unwrap()
                .contains(&json!(pointer)),
            "{finding}"
        );
        assert!(!finding["paths"]
            .as_array()
            .unwrap()
            .contains(&json!("/breakdown")));
        assert!(after_value.pointer(pointer).is_some());
    }
    let mut module_only = before_value;
    module_only["metadata"]["modules"] = json!([{"path":"module-only","detector":"npm"}]);
    let module_report = save(temp.path(), "module.json", &module_only);
    let result = json(&compare(&before, &module_report, &["--output", "-"]), 0);
    let finding = result["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["entity"]["kind"] == "folder" && f["entity"]["path"] == "module-only")
        .unwrap();
    assert_eq!(finding["paths"], json!(["/metadata/modules"]));
}

#[test]
fn backend_inaccuracy_is_partial_even_without_identified_failure_paths() {
    let temp = tempfile::tempdir().unwrap();
    write(temp.path(), "main.py", "print(1)\n");
    let mut input = measure(temp.path());
    input["metadata"]["backend_inaccurate"] = json!(true);
    assert_eq!(input["metadata"]["coverage"]["unread"], json!([]));
    assert_eq!(input["metadata"]["coverage"]["parse_failures"], json!([]));
    let saved = save(temp.path(), "input.json", &input);
    let result = json(&compare(&saved, &saved, &[]), 2);
    assert!(result["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["code"] == "coverage_gap"));
}

/// Only schema 3 is read; an earlier report fails as invalid, naming its version.
#[test]
fn older_measurement_schemas_are_invalid() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "main.py", "a = 1\n");
    let current = save(temp.path(), "current.json", &measure(&root));
    for version in ["1", "2"] {
        let mut old = measure(&root);
        old["schema_version"] = json!(version);
        old.as_object_mut().unwrap().remove("document_type");
        old.as_object_mut().unwrap().remove("detail");
        let old = save(temp.path(), "old.json", &old);
        let output = compare(&old, &current, &[]);
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains(&format!("unknown variant `{version}`, expected `3`")),
            "{error}"
        );
    }
}
