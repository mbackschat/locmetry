use serde_json::{json, Value};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .args(args)
        .output()
        .unwrap()
}
fn write(root: &Path, name: &str, text: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
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
    let mut command = Command::new(env!("CARGO_BIN_EXE_locmetry"));
    command
        .args(["compare", before.to_str().unwrap(), after.to_str().unwrap()])
        .args(args)
        .output()
        .unwrap()
}
fn document(output: Output, status: i32) -> Value {
    assert_eq!(
        output.status.code(),
        Some(status),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}
fn change(value: &Value, before: u64, after: u64, delta: i64) {
    assert_eq!(value["before"]["code"], before);
    assert_eq!(value["after"]["code"], after);
    assert_eq!(value["delta"]["code"], delta);
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
    let full = document(compare(&before, &after, &["--output", "-"]), 0);
    assert_eq!(full["document_type"], "comparison");
    assert_eq!(full["schema_version"], "1");
    change(&full["summary"]["source"], 3, 4, 1);
    assert_eq!(
        full["summary"]["source"]["delta"],
        json!({"files":0,"code":1,"comments":1,"blanks":-1,"lines":1})
    );
    change(&full["summary"]["source_purpose"]["production"], 2, 3, 1);
    change(&full["folders"]["src"]["direct"]["source"], 2, 3, 1);
    assert!(full["findings"].as_array().unwrap().is_empty());
    let reversed = document(compare(&after, &before, &["--output", "-"]), 0);
    change(&reversed["summary"]["source"], 4, 3, -1);
    let same = document(compare(&before, &before, &["--output", "-"]), 0);
    change(&same["summary"]["all_counted"], 3, 3, 0);
    let schema = document(run(&["--schema", "comparison"]), 0);
    assert!(jsonschema::validator_for(&schema).unwrap().is_valid(&full));
    let headline = document(compare(&before, &after, &[]), 0);
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
    let full = document(compare(&before, &after, &["--output", "-"]), 0);
    change(&full["summary"]["all_counted"], 5, 2, -3);
    change(&full["summary"]["source"], 2, 2, 0);
    change(&full["summary"]["source_purpose"]["production"], 2, 0, -2);
    change(&full["summary"]["source_purpose"]["unclassified"], 0, 2, 2);
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
    let full = document(compare(&before, &before, &["--output", "-"]), 0);
    assert!(full["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["code"] == "coverage_gap"));
    write(&root, "broken.ipynb", "{}");
    let after = save(temp.path(), "after.json", &measure(&root));
    let partial = document(compare(&before, &after, &["--output", "-"]), 2);
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
            "version" => bad["schema_version"] = json!("2"),
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
    let mut command = Command::new(env!("CARGO_BIN_EXE_locmetry"));
    command.arg("workspace");
    for root in roots {
        command.arg("--root").arg(root);
    }
    let output = command
        .args(["--no-user-config", "--output", "-"])
        .output()
        .unwrap();
    document(output, 0)
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
    let full = document(compare(&before, &after, &["--output", "-"]), 0);
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
    assert_eq!(
        full["presence_summary"]["members"],
        json!({"before_only":1,"after_only":2})
    );
    let headline = document(compare(&before, &after, &[]), 0);
    assert_eq!(headline["presence_summary"], full["presence_summary"]);
    assert!(headline["findings"]
        .as_array()
        .unwrap()
        .iter()
        .all(|f| f["code"] != "presence_changed"));
    let schema = document(run(&["--schema", "comparison"]), 0);
    assert!(jsonschema::validator_for(&schema).unwrap().is_valid(&full));
    let disjoint_before = save(temp.path(), "disjoint-before.json", &workspace(&[&legacy]));
    let disjoint_after = save(temp.path(), "disjoint-after.json", &workspace(&[&empty]));
    let disjoint = document(
        compare(&disjoint_before, &disjoint_after, &["--output", "-"]),
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
    let full = document(compare(&before, &after, &["--output", "-"]), 0);
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
    let original = document(compare(&before, &after, &["--output", "-"]), 0);
    let mut reordered = value.clone();
    for field in ["breakdown", "files", "review"] {
        reordered[field].as_array_mut().unwrap().reverse();
    }
    save(temp.path(), "after.json", &reordered);
    assert_eq!(
        document(compare(&before, &after, &["--output", "-"]), 0),
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
    let changed = document(compare(&before, &after, &["--output", "-"]), 0);
    assert!(changed["findings"].as_array().unwrap().is_empty());
    reordered["metadata"]["config"]["rules"]
        .as_array_mut()
        .unwrap()
        .reverse();
    save(temp.path(), "after.json", &reordered);
    let changed = document(compare(&before, &after, &["--output", "-"]), 0);
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
    after_value["summary"]["top_folders"]["."]["unclassified"] = json!(u64::MAX);
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
    let full = document(compare(&before, &after, &["--output", "-"]), 0);
    let headline = document(compare(&before, &after, &[]), 0);
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
    let schema = document(run(&["--schema", "comparison"]), 0);
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

#[test]
fn legacy_full_reports_compare_and_early_reports_name_the_missing_evidence() {
    for n in 2..=6 {
        let path = PathBuf::from(format!("tests/fixtures/reports/v0.1.{n}-root.json"));
        let full = document(compare(&path, &path, &["--output", "-"]), 0);
        assert_eq!(full["summary"]["source"]["delta"]["code"], 0);
        assert!(full["findings"]
            .as_array()
            .unwrap()
            .iter()
            .any(|f| f["code"] == "coverage_gap"));
    }
    for n in 0..=1 {
        let path = PathBuf::from(format!("tests/fixtures/reports/v0.1.{n}-root.json"));
        let output = compare(&path, &path, &[]);
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&output.stderr).contains("metadata.modules"));
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
    let headline = document(
        compare(&before, &after, &["--output", out.to_str().unwrap()]),
        0,
    );
    assert_eq!(headline["report"], out.to_str().unwrap());
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
        Some(1)
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
    let header = table
        .lines()
        .find(|line| line.starts_with("Measure "))
        .unwrap();
    assert_eq!(
        header,
        format!(
            "{:<40} {:>12} {:>12} {:>12}",
            "Measure", "Before", "After", "Net change"
        )
    );
}

#[test]
fn development_counter_is_distinct_from_the_last_release() {
    let temp = tempfile::tempdir().unwrap();
    write(temp.path(), "main.py", "print(1)\n");
    let current = measure(temp.path());
    assert_ne!(current["metadata"]["locmetry_version"], "0.1.6");
    let mut released = current.clone();
    released["metadata"]["locmetry_version"] = json!("0.1.6");
    let before = save(temp.path(), "before.json", &released);
    let after = save(temp.path(), "after.json", &current);
    let result = document(compare(&before, &after, &[]), 0);
    assert!(result["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|finding| finding["code"] == "counter_changed"));
}
