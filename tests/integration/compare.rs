use crate::common;
use common::{json, write};
use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Output,
};

fn run(args: &[&str]) -> Output {
    common::locmetry().args(args).output().unwrap()
}
fn measure(root: &Path) -> Value {
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
fn save(root: &Path, name: &str, report: &Value) -> PathBuf {
    let path = root.join(name);
    fs::write(&path, serde_json::to_vec(report).unwrap()).unwrap();
    path
}
fn compare(before: &Path, after: &Path, args: &[&str]) -> Output {
    let mut command = common::locmetry();
    command
        .args(["compare", before.to_str().unwrap(), after.to_str().unwrap()])
        .args(args)
        .output()
        .unwrap()
}
fn change(value: &Value, before: u64, after: u64, delta: i64) {
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
        let text = locmetry::compare::render_table_with_theme(&report, theme);
        assert!(!text.contains("▸ Totals"), "{text}");
        assert!(!text.contains("▸ Purpose"), "{text}");
        assert!(text.contains("All counted"), "{text}");
    }
    let plain = locmetry::compare::render_table_with_theme(&report, locmetry::style::Theme::PLAIN);
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
    assert!(locmetry::compare::render_markdown(&report).contains("## Totals"));
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
    assert_eq!(full["schema_version"], "2");
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
            "version" => bad["schema_version"] = json!("3"),
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

fn workspace(roots: &[&Path]) -> Value {
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

#[test]
fn released_version_one_reports_are_refused_with_a_remedy() {
    for n in 0..=6 {
        let path = PathBuf::from(format!("tests/fixtures/reports/v0.1.{n}-root.json"));
        let output = compare(&path, &path, &[]);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("measure again"));
    }
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
        render_table_with_theme(report, Theme::PLAIN)
            .lines()
            .skip_while(|line| !line.contains(" Lines "))
            .take_while(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    };
    let unchanged = totals(&report);
    let width = |line: &str| line.chars().count();
    assert!(
        unchanged
            .lines()
            .all(|line| width(line) == width(unchanged.lines().next().unwrap())),
        "{unchanged}"
    );
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
    assert!(
        large.lines().all(|line| width(line) == width(first)),
        "{large}"
    );
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

fn human_comparisons(before: &Path, after: &Path, status: i32) -> Vec<String> {
    ["table", "markdown"]
        .into_iter()
        .map(|format| {
            let output = compare(before, after, &["--format", format]);
            assert_eq!(
                output.status.code(),
                Some(status),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8(output.stdout).unwrap()
        })
        .collect()
}

#[test]
fn purpose_ratio_uses_source_code_and_handles_missing_purposes_on_each_side() {
    for (before, after, expected) in [
        ((5, 6), (17, 20), "1.20 → 1.18"),
        ((4, 0), (4, 2), "0.00 → 0.50"),
        ((0, 2), (2, 2), "- → 1.00"),
        ((2, 2), (0, 3), "1.00 → -"),
        ((0, 0), (0, 1), "- → -"),
        ((2, 0), (4, 0), "0.00 → 0.00"),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("project");
        write(&root, ".locmetryrc", "version = 1\n[[rules]]\nid = \"production\"\nmatch = [\"prod/**\"]\npurpose = \"production\"\n[[rules]]\nid = \"test\"\nmatch = [\"tests/**\"]\npurpose = \"test\"\n");
        write(&root, "other.py", &"print(1)\n".repeat(100));
        let mut snapshots = Vec::new();
        for (name, (production, test)) in [("before.json", before), ("after.json", after)] {
            for (folder, count) in [("prod", production), ("tests", test)] {
                let file = format!("{folder}/main.py");
                if count > 0 {
                    write(
                        &root,
                        &file,
                        &format!("{}{}", "print(1)\n".repeat(count), "# comment\n".repeat(10)),
                    );
                } else if root.join(&file).exists() {
                    fs::remove_file(root.join(&file)).unwrap();
                }
            }
            snapshots.push(save(temp.path(), name, &measure(&root)));
        }
        for text in human_comparisons(&snapshots[0], &snapshots[1], 0) {
            let expected = format!("Test/prod (Code): {expected} (before → after)");
            assert_eq!(text.matches(&expected).count(), 1, "{text}");
            let ratio = text.find(&expected).unwrap();
            let total = text[..ratio].rfind("Source total").unwrap();
            let between = &text[total..ratio];
            assert!(!between.contains("Language"), "{text}");
            if text.starts_with("# ") {
                assert!(text.contains(&format!("\n\n*{expected}*\n")), "{text}");
            } else {
                assert_eq!(between.lines().count(), 2, "{text}");
            }
        }
        for text in human_comparisons(&snapshots[1], &snapshots[1], 0) {
            assert!(!text.contains("Test/prod (Code):"), "{text}");
        }
    }
}

#[test]
fn workspace_purpose_ratio_uses_aggregate_counts_including_unchanged_rows() {
    let temp = tempfile::tempdir().unwrap();
    let api = temp.path().join("api");
    let web = temp.path().join("web");
    for (root, production) in [(&api, 2), (&web, 8)] {
        write(root, ".locmetryrc", "version = 1\n[[rules]]\nid = \"production\"\nmatch = [\"main.py\"]\npurpose = \"production\"\n");
        write(root, "main.py", &"print(1)\n".repeat(production));
    }
    write(&api, "tests/check.py", &"assert True\n".repeat(6));
    let before = save(temp.path(), "before.json", &workspace(&[&api, &web]));
    write(&api, "tests/check.py", &"assert True\n".repeat(12));
    let after = save(temp.path(), "after.json", &workspace(&[&api, &web]));
    for text in human_comparisons(&before, &after, 0) {
        assert!(
            text.contains("Test/prod (Code): 0.60 → 1.20 (before → after)"),
            "{text}"
        );
        assert!(!text.contains(" Production "), "{text}");
    }
    let full = json(&compare(&before, &after, &["--format", "json"]), 0);
    let report = serde_json::from_value(full).unwrap();
    let colored =
        locmetry::compare::render_table_with_theme(&report, locmetry::style::Theme::TERMINAL);
    assert!(
        colored.contains("\x1b[90mTest/prod (Code): 0.60 → 1.20 (before → after)\x1b[0m"),
        "{colored}"
    );
}

#[test]
fn unchanged_workspace_replaces_zero_composition_with_a_no_change_message() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "main.py", "print(1)\n");
    let saved = save(temp.path(), "input.json", &workspace(&[&root]));
    let full = json(&compare(&saved, &saved, &["--format", "json"]), 0);
    assert_eq!(
        full["summary"]["composition"]["source"]["matched"]["code"],
        0
    );
    let report = serde_json::from_value(full).unwrap();
    let mut texts = human_comparisons(&saved, &saved, 0);
    texts.push(locmetry::compare::render_table_with_theme(
        &report,
        locmetry::style::Theme::TERMINAL,
    ));
    for text in texts {
        assert!(text.contains("All counted"), "{text}");
        assert!(
            text.contains("No recorded count or presence changes."),
            "{text}"
        );
        assert!(!text.contains("Workspace source"), "{text}");
        assert!(!text.contains("Workspace all-counted"), "{text}");
    }
}

#[test]
fn workspace_composition_keeps_offsetting_arrivals_and_departures() {
    let temp = tempfile::tempdir().unwrap();
    let old = temp.path().join("old");
    let new = temp.path().join("new");
    write(&old, "main.py", "print(1)\n");
    write(&new, "main.py", "print(2)\n");
    let before = save(temp.path(), "before.json", &workspace(&[&old]));
    let after = save(temp.path(), "after.json", &workspace(&[&new]));
    for text in human_comparisons(&before, &after, 0) {
        assert!(
            text.contains("Workspace source lines: matched +0, entering +1, leaving -1; total +0."),
            "{text}"
        );
        assert!(
            text.contains("Workspace source code: matched +0, entering +1, leaving -1; total +0."),
            "{text}"
        );
        assert!(
            !text.contains("No recorded count or presence changes."),
            "{text}"
        );
    }
}

#[test]
fn zero_workspace_totals_do_not_hide_folder_transfers_or_empty_member_presence() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "left/main.py", "print(1)\nprint(2)\n");
    write(&root, "right/main.py", "print(1)\n");
    let before = save(temp.path(), "before.json", &workspace(&[&root]));
    write(&root, "left/main.py", "print(1)\n");
    write(&root, "right/main.py", "print(1)\nprint(2)\n");
    let after = save(temp.path(), "after.json", &workspace(&[&root]));
    let empty = temp.path().join("empty");
    fs::create_dir(&empty).unwrap();
    let arrived = save(temp.path(), "arrived.json", &workspace(&[&root, &empty]));
    for (a, b) in [(&before, &after), (&after, &arrived)] {
        for text in human_comparisons(a, b, 0) {
            assert!(!text.contains("Workspace source"), "{text}");
            assert!(
                !text.contains("No recorded count or presence changes."),
                "{text}"
            );
        }
    }
}

#[test]
fn workspace_no_change_message_accounts_for_comments_blanks_and_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "main.py", "print(1)\n# comment\n");
    let before = save(temp.path(), "before.json", &workspace(&[&root]));
    write(&root, "main.py", "print(1)\n\n");
    let after = save(temp.path(), "after.json", &workspace(&[&root]));
    write(&root, "empty.py", "");
    let added = save(temp.path(), "added.json", &workspace(&[&root]));
    for (a, b) in [(&before, &after), (&after, &added)] {
        for text in human_comparisons(a, b, 0) {
            assert!(!text.contains("Workspace source"), "{text}");
            assert!(!text.contains("Workspace all-counted"), "{text}");
            assert!(
                !text.contains("No recorded count or presence changes."),
                "{text}"
            );
        }
    }
}

#[test]
fn human_coverage_notes_summarize_paths_without_changing_json_findings() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "a.zzz", "unknown\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    let full = json(&compare(&before, &before, &["--format", "json"]), 0);
    assert_eq!(full["findings"].as_array().unwrap().len(), 4);
    assert!(full["findings"]
        .as_array()
        .unwrap()
        .iter()
        .all(|f| f["code"] == "coverage_gap"));
    for text in human_comparisons(&before, &before, 0) {
        assert_eq!(text.matches("Unrecognized files:").count(), 1, "{text}");
        assert!(
            text.contains("same 1 path in both reports; not counted on either side."),
            "{text}"
        );
        assert!(!text.contains("coverage_gap"), "{text}");
        assert!(!text.contains("Folder . contains"), "{text}");
        assert!(
            text.contains("No recorded differences in measurement settings."),
            "{text}"
        );
    }
    let report: locmetry::compare::Comparison = serde_json::from_value(full).unwrap();
    let styled =
        locmetry::compare::render_table_with_theme(&report, locmetry::style::Theme::TERMINAL);
    assert!(styled.contains("\x1b[90mUnrecognized files:"), "{styled}");
    assert!(!styled.contains("\x1b[33m"), "{styled}");
    fs::rename(root.join("a.zzz"), root.join("b.zzz")).unwrap();
    let after = save(temp.path(), "after.json", &measure(&root));
    for text in human_comparisons(&before, &after, 0) {
        assert!(
            text.contains("Unrecognized files: 1 → 1 (0 in both, 1 only before, 1 only after)"),
            "{text}"
        );
        assert!(!text.contains("same 1 path"), "{text}");
    }
}

#[test]
fn human_coverage_notes_keep_missing_evidence_distinct_from_recorded_failures() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "bad.ipynb", "invalid\n");
    let mut report = measure(&root);
    let before = save(temp.path(), "before.json", &report);
    report["metadata"]["coverage"]["parse_failures"][0]["message"] = json!("different detail");
    let after = save(temp.path(), "after.json", &report);
    for text in human_comparisons(&before, &after, 2) {
        assert!(
            text.contains("Parse failures: same 1 path/reason pair in both reports"),
            "{text}"
        );
    }
    report["metadata"]["coverage"]["parse_failures"][0]["reason"] =
        json!("unsupported_notebook_version");
    let changed = save(temp.path(), "changed.json", &report);
    for text in human_comparisons(&before, &changed, 2) {
        assert!(
            text.contains("Parse failures: 1 → 1 (0 in both, 1 only before, 1 only after)"),
            "{text}"
        );
    }
    fs::remove_file(root.join("bad.ipynb")).unwrap();
    let mut legacy = measure(&root);
    legacy["metadata"]["coverage"]
        .as_object_mut()
        .unwrap()
        .remove("parse_failures");
    let historical = save(temp.path(), "historical.json", &legacy);
    for text in human_comparisons(&historical, &historical, 0) {
        assert!(
            text.contains("Failure evidence: not recorded (before); not recorded (after)."),
            "{text}"
        );
        assert!(!text.contains("same"), "{text}");
    }
    for text in human_comparisons(&historical, &before, 2) {
        assert!(text.contains("Parse failures: not recorded → 1"), "{text}");
        assert!(!text.contains("only after"), "{text}");
    }
}

#[test]
fn human_coverage_notes_keep_partial_inputs_visible_and_warn() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "main.py", "print(1)\n");
    let mut report = measure(&root);
    report["metadata"]["backend_inaccurate"] = json!(true);
    let inaccurate = save(temp.path(), "inaccurate.json", &report);
    for text in human_comparisons(&inaccurate, &inaccurate, 2) {
        assert!(text.contains("Inaccurate input: before, after"), "{text}");
    }
    report["metadata"]["coverage"]["unread"] = json!(["unread.py"]);
    report["metadata"]["coverage"]["selected_files"] = json!(2);
    let unread = save(temp.path(), "unread.json", &report);
    for text in human_comparisons(&unread, &unread, 2) {
        assert!(
            text.contains("Unreadable files: same 1 path in both reports"),
            "{text}"
        );
    }
    let full = json(&compare(&unread, &unread, &["--format", "json"]), 2);
    let comparison: locmetry::compare::Comparison = serde_json::from_value(full).unwrap();
    let styled =
        locmetry::compare::render_table_with_theme(&comparison, locmetry::style::Theme::TERMINAL);
    assert!(styled.contains("\x1b[33mUnreadable files:"), "{styled}");
    assert!(styled.contains("\x1b[33mInaccurate input:"), "{styled}");
}

#[test]
fn human_coverage_notes_name_actionable_members_and_count_missing_member_evidence() {
    let temp = tempfile::tempdir().unwrap();
    let api = temp.path().join("api");
    let web = temp.path().join("web");
    write(&api, "main.py", "print(1)\n");
    write(&web, "main.py", "print(2)\n");
    let mut report = workspace(&[&api, &web]);
    let mut legacy = report.clone();
    legacy["metadata"]["coverage"]
        .as_object_mut()
        .unwrap()
        .remove("parse_failures");
    for member in legacy["metadata"]["members"].as_array_mut().unwrap() {
        member["coverage"]
            .as_object_mut()
            .unwrap()
            .remove("parse_failed_files");
    }
    let historical = save(temp.path(), "historical-workspace.json", &legacy);
    report["metadata"]["backend_inaccurate"] = json!(true);
    report["metadata"]["coverage"]["selected_files"] = json!(3);
    report["metadata"]["coverage"]["parse_failures"] = json!([{
        "path": "api/bad.ipynb", "reason": "invalid_notebook", "message": "invalid"
    }]);
    report["metadata"]["members"][0]["coverage"]["selected_files"] = json!(2);
    report["metadata"]["members"][0]["coverage"]["parse_failed_files"] = json!(1);
    report["metadata"]["members"][1]["coverage"]
        .as_object_mut()
        .unwrap()
        .remove("parse_failed_files");
    let saved = save(temp.path(), "workspace.json", &report);
    for text in human_comparisons(&saved, &saved, 2) {
        let failures = text
            .lines()
            .find(|l| l.contains("Parse failures:"))
            .unwrap();
        assert!(
            failures.contains("Members before: api; after: api"),
            "{text}"
        );
        assert!(
            text.contains("not recorded for 1 of 2 members (before)"),
            "{text}"
        );
        assert!(
            text.contains("not recorded for 1 of 2 members (after)"),
            "{text}"
        );
        assert!(!text.contains("coverage_gap"), "{text}");
    }
    for text in human_comparisons(&historical, &saved, 2) {
        assert!(
            text.contains("Members before: not recorded; after: api."),
            "{text}"
        );
    }
}

#[test]
fn human_producer_notes_show_each_changed_field_without_asserting_counting_behavior() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "main.py", "print(1)\n");
    let mut report = measure(&root);
    report["metadata"]["locmetry_version"] = json!("old");
    let before = save(temp.path(), "before.json", &report);
    report["metadata"]["locmetry_version"] = json!("new");
    let version = save(temp.path(), "version.json", &report);
    for text in human_comparisons(&before, &version, 0) {
        assert!(text.contains("Locmetry version: old → new."), "{text}");
        assert!(text.contains("Counting corrections can differ between versions; the reports cannot show whether they did."), "{text}");
        assert!(!text.contains("counter_changed"), "{text}");
        assert!(!text.contains("No recorded differences"), "{text}");
    }
    report["metadata"]["backend"] = json!("other backend");
    report["metadata"]["backend_configuration"] = json!("other configuration");
    let after = save(temp.path(), "after.json", &report);
    for text in human_comparisons(&before, &after, 0) {
        assert!(text.contains("Backend:"), "{text}");
        assert!(text.contains("→ other backend."), "{text}");
        assert!(text.contains("Backend configuration:"), "{text}");
        assert!(text.contains("→ other configuration."), "{text}");
    }
}

#[test]
fn source_grids_include_line_changes_but_omit_data_and_file_count_only_changes() {
    for variant in ["comment", "blank", "data", "empty file"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("project");
        write(&root, "src/main.py", "print(1)\n");
        write(&root, "src/settings.json", "{}\n");
        let before = save(temp.path(), "before.json", &measure(&root));
        let mut source_delta = json!({"files":0,"code":0,"comments":0,"blanks":0,"lines":0});
        match variant {
            "comment" => {
                write(&root, "src/main.py", "print(1)\n# comment\n");
                source_delta["comments"] = json!(1);
                source_delta["lines"] = json!(1);
            }
            "blank" => {
                write(&root, "src/main.py", "print(1)\n\n");
                source_delta["blanks"] = json!(1);
                source_delta["lines"] = json!(1);
            }
            "data" => write(&root, "src/settings.json", "{\n  \"a\": 1\n}\n"),
            "empty file" => {
                write(&root, "src/empty.py", "");
                source_delta["files"] = json!(1);
            }
            _ => unreachable!(),
        }
        let after = save(temp.path(), "after.json", &measure(&root));
        let full = json(&compare(&before, &after, &["--format", "json"]), 0);
        assert_eq!(
            full["folders"]["src"]["recursive"]["source"]["delta"], source_delta,
            "{variant}"
        );
        assert_eq!(
            full["summary"]["source"]["delta"], source_delta,
            "{variant}"
        );
        if variant == "data" {
            assert_eq!(
                full["folders"]["src"]["recursive"]["all_counted"]["delta"]["code"],
                2
            );
        }
        for text in human_comparisons(&before, &after, 0) {
            for heading in [" Purpose ", " Language ", " Folder "] {
                assert_eq!(
                    text.contains(heading),
                    matches!(variant, "comment" | "blank"),
                    "{variant}: {text}"
                );
            }
        }
    }
}

#[test]
fn source_code_grids_preserve_zero_code_folder_and_member_presence() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "src/main.py", "print(1)\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    write(&root, "assets/icon.zzz", "unknown\n");
    let after = save(temp.path(), "after.json", &measure(&root));
    for text in human_comparisons(&before, &after, 0) {
        assert!(text.contains("assets (after only)"), "{text}");
        assert!(!text.contains(". (both)"), "{text}");
    }
    for text in human_comparisons(&after, &before, 0) {
        assert!(text.contains("assets (before only)"), "{text}");
    }
    write(&root, "src/settings.json", "{}\n");
    let workspace_before = save(temp.path(), "workspace-before.json", &workspace(&[&root]));
    write(&root, "src/main.py", "print(1)\n# comment\n");
    write(&root, "src/settings.json", "{\n  \"a\": 1\n}\n");
    let workspace_after = save(temp.path(), "workspace-after.json", &workspace(&[&root]));
    let full = json(
        &compare(&workspace_before, &workspace_after, &["--format", "json"]),
        0,
    );
    assert_eq!(full["members"]["project"]["source"]["delta"]["comments"], 1);
    assert_eq!(
        full["members"]["project"]["all_counted"]["delta"]["code"],
        2
    );
    for text in human_comparisons(&workspace_before, &workspace_after, 0) {
        assert!(text.contains(" Member "), "{text}");
        assert_eq!(
            text.lines()
                .any(|line| line.starts_with(" Folder ") || line.starts_with("| Folder |")),
            text.starts_with("# "),
            "{text}"
        );
    }
    let empty = temp.path().join("empty");
    fs::create_dir(&empty).unwrap();
    let arrived = save(temp.path(), "arrived.json", &workspace(&[&root, &empty]));
    for text in human_comparisons(&workspace_after, &arrived, 0) {
        assert!(text.contains("empty (after only)"), "{text}");
        assert!(!text.contains("project (both)"), "{text}");
    }
    for text in human_comparisons(&arrived, &workspace_after, 0) {
        assert!(text.contains("empty (before only)"), "{text}");
    }
    write(&root, "src/main.py", "print(1)\nprint(2)\n");
    let source_after = save(temp.path(), "source-after.json", &measure(&root));
    for text in human_comparisons(&after, &source_after, 0) {
        for heading in [" Purpose ", " Language ", " Folder "] {
            assert!(text.contains(heading), "{text}");
        }
    }
    let workspace_source_after = save(
        temp.path(),
        "workspace-source-after.json",
        &workspace(&[&root]),
    );
    for text in human_comparisons(&workspace_after, &workspace_source_after, 0) {
        assert!(text.contains("project (both)"), "{text}");
    }
}

#[test]
fn workspace_composition_colors_signed_contributions_and_mutes_signed_zero() {
    use locmetry::style::Theme;
    let temp = tempfile::tempdir().unwrap();
    let api = temp.path().join("api");
    let legacy = temp.path().join("legacy");
    let worker = temp.path().join("worker");
    write(&api, "main.py", &"print(1)\n".repeat(6));
    write(&legacy, "main.py", &"print(1)\n".repeat(2));
    let before = save(temp.path(), "before.json", &workspace(&[&api, &legacy]));
    write(&api, "main.py", &"print(1)\n".repeat(3));
    write(&worker, "main.py", &"print(1)\n".repeat(5));
    let after = save(temp.path(), "after.json", &workspace(&[&api, &worker]));
    write(&worker, "main.py", &"print(1)\n".repeat(6));
    let grown = save(temp.path(), "grown.json", &workspace(&[&api, &worker]));
    for (a, b, matched, entering, leaving, total) in [
        (
            &before,
            &after,
            "\x1b[31m-3\x1b[0m",
            "\x1b[32m+5\x1b[0m",
            "\x1b[31m-2\x1b[0m",
            "\x1b[90m+0\x1b[0m",
        ),
        (
            &after,
            &before,
            "\x1b[32m+3\x1b[0m",
            "\x1b[32m+2\x1b[0m",
            "\x1b[31m-5\x1b[0m",
            "\x1b[90m+0\x1b[0m",
        ),
        (
            &after,
            &grown,
            "\x1b[32m+1\x1b[0m",
            "\x1b[90m+0\x1b[0m",
            "\x1b[90m-0\x1b[0m",
            "\x1b[32m+1\x1b[0m",
        ),
        (
            &grown,
            &before,
            "\x1b[32m+3\x1b[0m",
            "\x1b[32m+2\x1b[0m",
            "\x1b[31m-6\x1b[0m",
            "\x1b[31m-1\x1b[0m",
        ),
    ] {
        let full = json(&compare(a, b, &["--format", "json"]), 0);
        let report = serde_json::from_value(full).unwrap();
        let colored = locmetry::compare::render_table_with_theme(&report, Theme::TERMINAL);
        let expected =
            format!("matched {matched}, entering {entering}, leaving {leaving}; total {total}.");
        for label in [
            "source lines",
            "all-counted lines",
            "source code",
            "all-counted code",
        ] {
            assert!(
                colored.contains(&format!("Workspace {label}: {expected}")),
                "{colored}"
            );
        }
        let stripped: String = colored
            .split('\x1b')
            .map(|part| match part.strip_prefix('[') {
                Some(control) => control.split_once('m').unwrap().1,
                None => part,
            })
            .collect();
        let plain = locmetry::compare::render_table_with_theme(&report, Theme::PLAIN);
        assert_eq!(stripped, plain);
        let markdown = locmetry::compare::render_markdown(&report);
        for text in human_comparisons(a, b, 0).into_iter().chain([markdown]) {
            assert!(!text.contains('\x1b'), "{text}");
            for line in plain.lines().filter(|line| line.starts_with("Workspace ")) {
                assert!(text.contains(line), "{text}");
            }
        }
    }
}

#[test]
fn comparison_theme_preserves_plain_text_and_distinguishes_signed_changes() {
    use locmetry::style::Theme;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("source");
    write(&root, "main.py", "print(1)\n\n");
    write(&root, "README.md", "# A\n\ntext\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    write(&root, "main.py", "print(1)\nprint(2)\n");
    write(&root, "README.md", "# A\n");
    let after = save(temp.path(), "after.json", &measure(&root));
    let value = json(&compare(&before, &after, &["--output", "-"]), 0);
    let report: locmetry::compare::Comparison = serde_json::from_value(value).unwrap();
    let plain = locmetry::compare::render_table_with_theme(&report, Theme::PLAIN);
    let colored = locmetry::compare::render_table_with_theme(&report, Theme::TERMINAL);
    for (style, value) in [("32m", "+1"), ("31m", "-2"), ("90m", "0")] {
        assert!(
            colored.split("\x1b[").any(|cell| cell
                .strip_prefix(style)
                .is_some_and(|cell| cell.trim() == value)),
            "{style} {value}\n{colored}"
        );
    }
    let stripped: String = colored
        .split('\x1b')
        .map(|s| match s.strip_prefix('[') {
            Some(control) => control.split_once('m').unwrap().1,
            None => s,
        })
        .collect();
    assert_eq!(stripped, plain);
    assert!(!plain.contains('\x1b'));
    let markdown = locmetry::compare::render_markdown(&report);
    assert!(markdown.contains("*Net change = after − before"));
    assert!(!markdown.contains('\x1b'));
    assert!(
        markdown.contains("| Source | 2 | 2 | 0 | 1 | 2 | +1 | 0 | -1 | 0 |"),
        "{markdown}"
    );
    assert!(
        markdown.contains("| **All counted** | **5** | **3** | **-2** |"),
        "{markdown}"
    );
}

#[test]
fn human_comparisons_show_line_changes_and_original_measurement_dates() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "src/main.py", "print(1)\n");
    let mut before = measure(&root);
    before["metadata"]["measured_at_unix_seconds"] = json!(0);
    write(&root, "src/main.py", "print(1)\n# comment\n\n");
    let mut after = measure(&root);
    after["metadata"]["measured_at_unix_seconds"] = json!(951827696);
    let before = save(temp.path(), "before.json", &before);
    let after = save(temp.path(), "after.json", &after);
    for text in human_comparisons(&before, &after, 0) {
        // Markdown cannot span columns, so it keeps the full titles that the
        // table splits into a group heading and the column headings below it.
        let headed = match text.starts_with("# ") {
            true => text.contains("| Lines before |"),
            false => text.contains(" Lines ") && text.contains(" before "),
        };
        assert!(headed, "{text}");
        let language = text.lines().find(|line| line.contains("Python")).unwrap();
        let cells: Vec<_> = language
            .split_whitespace()
            .filter(|c| !matches!(*c, "|" | "│"))
            .map(|c| c.trim_matches('*'))
            .collect();
        assert_eq!(cells, ["Python", "1", "3", "+2", "1", "1", "0"]);
        assert!(text.ends_with("Before measured at: 1970-01-01 00:00:00 UTC\nAfter measured at: 2000-02-29 12:34:56 UTC\n"), "{text}");
    }
}

/// Locks the plain wording that replaced internal terms: the header caveats,
/// presence counts in words without zero or non-workspace parts, and findings
/// that name the folder instead of an "Entity".
#[test]
fn human_comparisons_explain_presence_in_plain_words() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "src/main.py", "a = 1\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    write(&root, "src/extra/more.py", "b = 2\n");
    let after = save(temp.path(), "after.json", &measure(&root));
    let output = compare(&before, &after, &["--format", "table"]);
    assert_eq!(output.status.code(), Some(0));
    let table = String::from_utf8(output.stdout).unwrap();
    for expected in [
        "Net change = after − before, from the two saved reports.",
        "Unreadable files count as 0; a folder seen in only one report may be newly visible, not newly created.",
        "Only in one report: 1 folder only after.",
        "Folder \"src/extra\" appears only in the after report; it may be new or newly measured.",
    ] {
        assert!(table.contains(expected), "{expected}\n{table}");
    }
    for absent in ["Entity", "members -", "Presence:"] {
        assert!(!table.contains(absent), "{absent}\n{table}");
    }
}

/// Locks that the human totals explain All counted: one row per content kind,
/// whose figures add up to the All counted row, and that the Purpose and
/// table Folders grids end in the source total, with the root folder as it.
#[test]
fn totals_split_all_counted_by_kind_and_grids_end_in_their_source_total() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "src/main.py", "a = 1\n");
    write(&root, "tests/test_main.py", "assert True\n");
    write(&root, "README.md", "# Project\n\ntext\n");
    write(&root, "config.json", "{\n  \"a\": 1\n}\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    write(&root, "src/main.py", "a = 1\nb = 2\n");
    write(&root, "config.json", "{\n  \"a\": 1,\n  \"b\": 2\n}\n");
    let after = save(temp.path(), "after.json", &measure(&root));
    let table = String::from_utf8(compare(&before, &after, &["--format", "table"]).stdout).unwrap();
    let section = |label: &str| {
        table
            .split("\n\n")
            .find(|block| {
                block
                    .lines()
                    .any(|line| line.starts_with(&format!(" {label} ")))
            })
            .unwrap()
            .to_string()
    };
    for label in ["Kind", "Purpose", "Language", "Folder"] {
        let block = section(label);
        let lines: Vec<_> = block.lines().collect();
        assert_eq!(lines[0], lines[3].replace('┼', "─"), "{block}");
        let group_header = lines[1];
        assert!(
            group_header.contains("Lines") && group_header.contains("Code"),
            "{group_header}"
        );
        assert!(!group_header.contains('─'), "{group_header}");
    }
    for title in ["Totals", "Purpose", "Languages", "Folders"] {
        assert!(!table.contains(&format!("▸ {title}")), "{table}");
    }
    let figures = |line: &str| -> Vec<i64> {
        line.split_whitespace()
            .filter_map(|cell| cell.replace(',', "").parse().ok())
            .collect()
    };
    let totals = section("Kind");
    let rows: Vec<_> = totals
        .lines()
        .filter(|line| !figures(line).is_empty())
        .collect();
    let labels: Vec<_> = rows
        .iter()
        .map(|row| row.split("  ").next().unwrap().trim())
        .collect();
    assert_eq!(
        labels,
        ["Source", "Data", "Documentation", "All counted"],
        "{totals}"
    );
    let (kinds, all_counted) = rows.split_at(3);
    let summed = kinds
        .iter()
        .map(|row| figures(row))
        .fold(vec![0; 10], |sum, row| {
            sum.iter().zip(row).map(|(a, b)| a + b).collect()
        });
    assert_eq!(summed, figures(all_counted[0]), "{totals}");
    assert!(figures(kinds[1])[2] > 0, "data growth is visible: {totals}");

    let purpose = section("Purpose");
    assert!(
        purpose
            .lines()
            .rev()
            .nth(1)
            .unwrap()
            .starts_with(" Source total "),
        "{purpose}"
    );
    let folders = section("Folder");
    assert!(
        folders
            .lines()
            .any(|line| line.starts_with(" Source total ")),
        "{folders}"
    );
    assert!(
        folders.contains("Folders are recursive; do not sum parents and children."),
        "{folders}"
    );
    assert!(!folders.contains(". (both)"), "{folders}");
    let markdown =
        String::from_utf8(compare(&before, &after, &["--format", "markdown"]).stdout).unwrap();
    assert!(markdown.contains("| . (both) |"), "{markdown}");
}

/// Locks the 0.5 clean break: version-1 reports use the retired vocabulary, so
/// compare names the version and the remedy instead of failing on a key.
#[test]
fn version_one_reports_are_rejected_with_a_remedy() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "main.py", "a = 1\n");
    let mut old = measure(&root);
    old["schema_version"] = json!("1");
    let old = save(temp.path(), "old.json", &old);
    let current = save(temp.path(), "current.json", &measure(&root));
    let output = compare(&old, &current, &[]);
    assert_eq!(output.status.code(), Some(1));
    let error = String::from_utf8_lossy(&output.stderr);
    assert!(error.contains("schema version 1"), "{error}");
    assert!(error.contains("measure again"), "{error}");
}
