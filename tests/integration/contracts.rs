use crate::common;
use locmetry::contract::MeasurementReport;
use serde_json::{json, Value};

/// A full report with file detail, freshly measured from a synthetic project
/// shaped like the release fixtures' input: a build module, production and
/// test source, documentation, an unrecognized file, unclaimed source and an
/// unused rule, so every report section the validation tests mutate is populated.
fn current(workspace: bool) -> Value {
    let temp = tempfile::tempdir().unwrap();
    common::write_all(
        temp.path(),
        &[
            ("package.json", "{\"name\": \"synthetic\"}\n"),
            ("src/index.js", "let a = 1;\nlet b = 2;\n"),
            ("tests/test_main.py", "assert True\n"),
            ("README.md", "# Synthetic\n\n```python\nprint(1)\n```\n"),
            ("assets/blob.xyz", "opaque\n"),
            ("scripts/tool.py", "print(1)\n"),
            (
                ".locmetryrc",
                "version = 1\n[scope]\nexclude = [\"never/**\"]\n[[rules]]\nid = \"unused\"\nmatch = [\"nothing/**\"]\npurpose = \"test\"\n",
            ),
        ],
    );
    let mut command = common::locmetry();
    if workspace {
        command.arg("workspace");
    }
    let output = command
        .args([
            "--root",
            temp.path().to_str().unwrap(),
            "--files",
            "--output",
            "-",
        ])
        .output()
        .unwrap();
    common::json(&output, 0)
}

#[test]
fn measurement_envelope_identifies_full_and_headline_projections() {
    for workspace in [false, true] {
        let full = current(workspace);
        assert_eq!(full["document_type"], "measurement");
        assert_eq!(full["detail"], "full");
        assert_eq!(full["schema_version"], "3");
        let typed: MeasurementReport = serde_json::from_value(full.clone()).unwrap();
        let headline = serde_json::to_value(typed.headline()).unwrap();
        assert_eq!(headline["document_type"], "measurement");
        assert_eq!(headline["detail"], "headline");
        assert_eq!(headline["schema_version"], "3");
        for (field, value) in [
            ("document_type", json!("comparison")),
            ("detail", json!("summary")),
            ("schema_version", json!("2")),
        ] {
            let mut invalid = full.clone();
            invalid[field] = value;
            assert!(serde_json::from_value::<MeasurementReport>(invalid).is_err());
        }
        for field in ["document_type", "detail"] {
            let mut invalid = full.clone();
            invalid.as_object_mut().unwrap().remove(field);
            assert!(serde_json::from_value::<MeasurementReport>(invalid).is_err());
        }
        for (mut invalid, detail) in [(full, "headline"), (headline, "full")] {
            invalid["detail"] = json!(detail);
            let typed: MeasurementReport = serde_json::from_value(invalid).unwrap();
            assert!(typed.validate().is_err(), "{detail}");
        }
    }
}

/// Locks the 0.5 clean break: every released version-1 report is refused by
/// the typed reader, because its purpose names no longer exist.
#[test]
fn version_one_release_reports_are_refused() {
    let names = (0..=6)
        .map(|minor| format!("v0.1.{minor}-root"))
        .chain((4..=6).map(|minor| format!("v0.1.{minor}-workspace")));
    for name in names {
        let value: Value = serde_json::from_str(
            &std::fs::read_to_string(format!("tests/fixtures/reports/{name}.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(value["schema_version"], "1", "{name}");
        assert!(
            serde_json::from_value::<MeasurementReport>(value).is_err(),
            "{name}"
        );
    }
}

/// Schema 3 grows within its version before 1.0, so a field the current
/// producer adds must not invalidate reports of the latest release: absence
/// means "not recorded" (the v0.7.3 fixtures predate `largest_module_files`).
#[test]
fn latest_release_reports_stay_valid_and_comparable() {
    for name in ["v0.7.3-root", "v0.7.3-workspace"] {
        let path = format!("tests/fixtures/reports/{name}.json");
        let value: Value = serde_json::from_str(&std::fs::read_to_string(&path).unwrap()).unwrap();
        assert_eq!(value["schema_version"], "3", "{name}");
        serde_json::from_value::<MeasurementReport>(value)
            .unwrap()
            .validate()
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        let output = common::locmetry()
            .args(["compare", &path, &path, "--format", "json"])
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
}

#[test]
fn current_root_and_workspace_contracts_round_trip_without_shape_changes() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("main.py"), "print(1)\n").unwrap();
    for workspace in [false, true] {
        let mut command = common::locmetry();
        if workspace {
            command.arg("workspace");
        }
        let output = command
            .args([
                "--root",
                temp.path().to_str().unwrap(),
                "--no-user-config",
                "--files",
                "--output",
                "-",
            ])
            .output()
            .unwrap();
        assert!(output.status.success());
        let value: Value = serde_json::from_slice(&output.stdout).unwrap();
        let typed: MeasurementReport = serde_json::from_value(value.clone()).unwrap();
        typed.validate().unwrap();
        assert_eq!(serde_json::to_value(typed).unwrap(), value);
    }
}

#[test]
fn typed_contract_rejects_unknown_keys_invalid_modes_and_negative_counts() {
    for (pointer, replacement) in [
        ("/summary/all_counted/code", json!(-1)),
        ("/metadata/coverage/parse_failures", Value::Null),
        ("/metadata/config/surprise", json!(true)),
        ("/summary/surprise", json!(true)),
    ] {
        let mut value = current(false);
        let (parent, key) = pointer.rsplit_once('/').unwrap();
        value.pointer_mut(parent).unwrap()[key] = replacement;
        assert!(
            serde_json::from_value::<MeasurementReport>(value).is_err(),
            "{pointer}"
        );
    }
}

#[test]
fn semantic_validation_rejects_conflicting_counts_coverage_and_duplicate_rows() {
    for (pointer, replacement, message) in [
        ("/summary/all_counted/lines", json!(999), "lines"),
        (
            "/metadata/coverage/selected_files",
            json!(999),
            "selected_files",
        ),
        (
            "/summary/source_purpose/production/code",
            json!(999),
            "lines",
        ),
        (
            "/summary/top_folders/src/production",
            json!(999),
            "top_folders",
        ),
    ] {
        let mut value = current(false);
        let (parent, key) = pointer.rsplit_once('/').unwrap();
        value.pointer_mut(parent).unwrap()[key] = replacement;
        let typed: MeasurementReport = serde_json::from_value(value).unwrap();
        assert!(typed.validate().unwrap_err().to_string().contains(message));
    }
    for field in ["breakdown", "review", "files"] {
        let mut value = current(false);
        let rows = value[field].as_array_mut().unwrap();
        rows.push(rows[0].clone());
        let typed: MeasurementReport = serde_json::from_value(value).unwrap();
        assert!(
            typed
                .validate()
                .unwrap_err()
                .to_string()
                .contains("duplicate"),
            "{field}"
        );
    }
    let mut value = current(false);
    value["summary"]["all_counted"]["code"] = json!(u64::MAX);
    let typed: MeasurementReport = serde_json::from_value(value).unwrap();
    assert!(typed
        .validate()
        .unwrap_err()
        .to_string()
        .contains("overflow"));
}

#[test]
fn validity_is_separate_from_comparison_eligibility() {
    for workspace in [false, true] {
        let report: MeasurementReport = serde_json::from_value(current(workspace)).unwrap();
        report.validate().unwrap();
        assert!(report.require_comparison_evidence().is_ok());
        let headline = report.headline();
        headline.validate().unwrap();
        assert!(headline.require_comparison_evidence().is_err());
    }
}

#[test]
fn duplicate_dynamic_map_keys_retain_the_last_value_before_validation() {
    let mut value = current(false);
    value["summary"]["source_languages"]=serde_json::from_str(r#"{"Python":{"files":1,"code":9,"comments":0,"blanks":0,"lines":9},"Python":{"files":1,"code":1,"comments":0,"blanks":0,"lines":1}}"#).unwrap();
    let typed: MeasurementReport = serde_json::from_value(value).unwrap();
    assert_eq!(typed.summary.totals.source_languages["Python"].code, 1);
}

#[test]
fn workspace_modes_and_duplicate_metadata_identities_are_rejected() {
    for field in ["members", "modules"] {
        let mut value = current(true);
        let rows = value["metadata"][field].as_array_mut().unwrap();
        rows.push(rows[0].clone());
        let typed: MeasurementReport = serde_json::from_value(value).unwrap();
        assert!(typed
            .validate()
            .unwrap_err()
            .to_string()
            .contains("duplicate"));
    }
    let mut value = current(true);
    value["metadata"]["root"] = json!("/synthetic/invalid");
    let typed: MeasurementReport = serde_json::from_value(value.clone()).unwrap();
    assert!(typed.validate().unwrap_err().to_string().contains("mixed"));
    assert!(!jsonschema::validator_for(&locmetry::report_schema())
        .unwrap()
        .is_valid(&value));
}

#[test]
fn explicit_zero_summary_entries_equal_omission() {
    let mut value = current(false);
    value["summary"]["source_categories"]["production:empty"] =
        json!({"files":0,"code":0,"comments":0,"blanks":0,"lines":0});
    let typed: MeasurementReport = serde_json::from_value(value).unwrap();
    typed.validate().unwrap();
}

#[test]
fn schema_examples_are_valid_measurement_headlines() {
    let schema = locmetry::report_schema();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let examples = schema["examples"].as_array().unwrap();
    assert!(!examples.is_empty());
    for example in examples {
        assert!(validator.is_valid(example));
        serde_json::from_value::<MeasurementReport>(example.clone())
            .unwrap()
            .validate()
            .unwrap();
    }
}

#[test]
fn nonzero_physical_totals_need_counted_files() {
    let mut value = current(false);
    for pointer in ["/summary/all_counted", "/summary/source"] {
        value.pointer_mut(pointer).unwrap()["files"] = json!(0);
    }
    for field in [
        "by_kind_purpose",
        "source_purpose",
        "source_categories",
        "source_languages",
    ] {
        for counts in value["summary"][field]
            .as_object_mut()
            .unwrap()
            .values_mut()
        {
            counts["files"] = json!(0);
        }
    }
    value.as_object_mut().unwrap().remove("files");
    for row in value["breakdown"].as_array_mut().unwrap() {
        row["files"] = json!(0);
    }
    value["metadata"]["coverage"]["counted_files"] = json!(0);
    value["metadata"]["coverage"]["selected_files"] =
        value["metadata"]["coverage"]["unrecognized_files"].clone();
    let typed: MeasurementReport = serde_json::from_value(value).unwrap();
    assert!(typed
        .validate()
        .unwrap_err()
        .to_string()
        .contains("physical files"));
}

#[test]
fn schemas_and_reader_reject_invalid_purpose_map_keys() {
    let schema = jsonschema::validator_for(&locmetry::report_schema()).unwrap();
    for invalid in ["bogus", "Test"] {
        for path in ["/summary/source_purpose", "/summary/top_folders/src"] {
            let mut value = current(false);
            let map = value.pointer_mut(path).unwrap().as_object_mut().unwrap();
            let valid = map.values().next().unwrap().clone();
            map.insert(invalid.into(), valid);
            assert!(serde_json::from_value::<MeasurementReport>(value.clone()).is_err());
            assert!(!schema.is_valid(&value), "{path}: {invalid}");
        }
    }
}

#[test]
fn language_participation_cannot_be_less_than_host_files_in_breakdown() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("one.py"), "print(1)\n").unwrap();
    std::fs::write(temp.path().join("two.py"), "print(2)\n").unwrap();
    for workspace in [false, true] {
        let mut command = common::locmetry();
        if workspace {
            command.arg("workspace");
        }
        let output = command
            .args([
                "--root",
                temp.path().to_str().unwrap(),
                "--no-user-config",
                "--output",
                "-",
            ])
            .output()
            .unwrap();
        assert!(output.status.success());
        let mut value: Value = serde_json::from_slice(&output.stdout).unwrap();
        value["summary"]["source_languages"]["Python"]["files"] = json!(1);
        if workspace {
            let name = value["metadata"]["members"][0]["name"]
                .as_str()
                .unwrap()
                .to_owned();
            value["summary"]["members"][name]["source_languages"]["Python"]["files"] = json!(1);
        }
        let report: MeasurementReport = serde_json::from_value(value).unwrap();
        assert!(
            report.validate().is_err(),
            "understated host participation: workspace={workspace}"
        );
    }
}

#[test]
fn member_language_participation_is_checked_even_when_workspace_totals_reconcile() {
    let temp = tempfile::tempdir().unwrap();
    for member in ["a", "b"] {
        let root = temp.path().join(member);
        std::fs::create_dir(&root).unwrap();
        for name in ["one.py", "two.py"] {
            std::fs::write(root.join(name), "print(1)\n").unwrap();
        }
        std::fs::write(root.join("main.js"), "let a = 1;\n").unwrap();
    }
    let output = common::locmetry()
        .args([
            "workspace",
            "--root",
            temp.path().join("a").to_str().unwrap(),
            "--root",
            temp.path().join("b").to_str().unwrap(),
            "--no-user-config",
            "--output",
            "-",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let mut value: Value = serde_json::from_slice(&output.stdout).unwrap();
    value["summary"]["members"]["a"]["source_languages"]["Python"]["files"] = json!(1);
    value["summary"]["members"]["b"]["source_languages"]["Python"]["files"] = json!(3);
    let report: MeasurementReport = serde_json::from_value(value).unwrap();
    assert!(report
        .validate()
        .unwrap_err()
        .to_string()
        .contains("member breakdown: language counts conflict"));
}
