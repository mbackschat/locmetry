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
    let header = table
        .lines()
        .find(|line| line.starts_with("Measure "))
        .unwrap();
    assert_eq!(header, "Measure               Before  After  Net change");
}

#[test]
fn comparison_totals_fit_their_contents_and_keep_large_signed_values_aligned() {
    use locmetry::{compare::render_table, Counts};
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "main.py", "print(1)\n");
    let saved = save(temp.path(), "input.json", &measure(&root));
    let full = document(compare(&saved, &saved, &["--format", "json"]), 0);
    let mut report: locmetry::compare::Comparison = serde_json::from_value(full).unwrap();
    let table = render_table(&report);
    let totals = table
        .split("Totals\n")
        .nth(1)
        .unwrap()
        .split("\n\n")
        .next()
        .unwrap();
    assert!(totals.lines().all(|line| line.len() == 47), "{totals}");
    assert!(
        totals.contains("All counted files          1      1          +0"),
        "{totals}"
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
    report.summary.dimensions.source = report.summary.dimensions.all_counted;
    let table = render_table(&report);
    let totals = table
        .split("Totals\n")
        .nth(1)
        .unwrap()
        .split("\n\n")
        .next()
        .unwrap();
    assert!(totals.lines().all(|line| line.len() == 70), "{totals}");
    for line in totals.lines() {
        assert_eq!(&line[20..22], "  ");
        assert_eq!(&line[41..43], "  ");
        assert_eq!(&line[48..50], "  ");
    }
    assert!(
        totals.contains("All counted code      9223372036854775808      0  -9223372036854775808"),
        "{totals}"
    );
}

#[test]
fn producer_identity_and_docs_distinguish_the_checkout_from_stable_installs() {
    let readme = include_str!("../README.md");
    let stable = readme
        .split_once("latest stable release is **")
        .unwrap()
        .1
        .split_once("**")
        .unwrap()
        .0;
    assert!(readme.contains(&format!("### Stable release (v{stable})")));
    assert!(readme.contains(&format!("--tag v{stable}")));
    match env!("CARGO_PKG_VERSION_PRE") {
        "" => assert_eq!(stable, env!("CARGO_PKG_VERSION")),
        _ => {
            assert!(readme.contains(&format!(
                "This development checkout identifies as `{}`",
                env!("CARGO_PKG_VERSION")
            )));
            assert!(!readme.contains(&format!("--tag v{}", env!("CARGO_PKG_VERSION"))));
        }
    }
    assert!(readme.contains(&format!(
        "\"locmetry_version\": \"{}\"",
        env!("CARGO_PKG_VERSION")
    )));
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
    let result = document(compare(&before, &after, &["--output", "-"]), 2);
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
    let result = document(compare(&before, &module_report, &["--output", "-"]), 0);
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
    let result = document(compare(&saved, &saved, &[]), 2);
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
fn human_coverage_notes_summarize_paths_without_changing_json_findings() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "a.zzz", "unknown\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    let full = document(compare(&before, &before, &["--format", "json"]), 0);
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
    let full = document(compare(&unread, &unread, &["--format", "json"]), 2);
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
        let full = document(compare(&before, &after, &["--format", "json"]), 0);
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
            for heading in ["Purpose", "Languages", "Folders (recursive"] {
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
    let full = document(
        compare(&workspace_before, &workspace_after, &["--format", "json"]),
        0,
    );
    assert_eq!(full["members"]["project"]["source"]["delta"]["comments"], 1);
    assert_eq!(
        full["members"]["project"]["all_counted"]["delta"]["code"],
        2
    );
    for text in human_comparisons(&workspace_before, &workspace_after, 0) {
        assert!(text.contains("Workspace members"), "{text}");
        assert_eq!(
            text.contains("Folders (recursive"),
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
        for heading in ["Purpose", "Languages", "Folders (recursive"] {
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
fn comparison_theme_preserves_plain_text_and_distinguishes_signed_changes() {
    use locmetry::style::Theme;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("source");
    write(&root, "main.py", "print(1)\n\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    write(&root, "main.py", "print(1)\nprint(2)\n");
    let after = save(temp.path(), "after.json", &measure(&root));
    let value = document(compare(&before, &after, &["--output", "-"]), 0);
    let report: locmetry::compare::Comparison = serde_json::from_value(value).unwrap();
    let plain = locmetry::compare::render_table(&report);
    let colored = locmetry::compare::render_table_with_theme(&report, Theme::TERMINAL);
    for expected in [
        "\x1b[36m        +1",
        "\x1b[35m        -1",
        "\x1b[90m        +0",
    ] {
        assert!(colored.contains(expected), "{colored}");
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
    assert!(markdown.contains("*After minus before"));
    assert!(!markdown.contains('\x1b'));
    assert!(markdown.contains("| Source code | 1 | 2 | +1 |"));
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
        assert!(text.contains("Lines before"), "{text}");
        let language = text.lines().find(|line| line.contains("Python")).unwrap();
        let cells: Vec<_> = language
            .split_whitespace()
            .filter(|c| *c != "|")
            .map(|c| c.trim_matches('*'))
            .collect();
        assert_eq!(cells, ["Python", "1", "3", "+2", "1", "1", "+0"]);
        assert!(text.ends_with("Before measured at: 1970-01-01 00:00:00 UTC\nAfter measured at: 2000-02-29 12:34:56 UTC\n"), "{text}");
    }
}
