use locmetry::contract::MeasurementReport;
use serde_json::{json, Value};
use std::process::Command;

fn fixture(name: &str) -> Value {
    serde_json::from_str(
        &std::fs::read_to_string(format!("tests/fixtures/reports/{name}.json")).unwrap(),
    )
    .unwrap()
}

#[test]
fn historical_releases_decode_without_inventing_failure_evidence() {
    let schema = jsonschema::validator_for(&locmetry::report_schema()).unwrap();
    for minor in 0..=6 {
        for mode in if minor >= 4 {
            vec!["root", "workspace"]
        } else {
            vec!["root"]
        } {
            let name = format!("v0.1.{minor}-{mode}");
            let value = fixture(&name);
            let report: MeasurementReport =
                serde_json::from_value(value.clone()).unwrap_or_else(|e| panic!("{name}: {e}"));
            assert!(
                schema.is_valid(&value),
                "{name}: {:?}",
                schema.iter_errors(&value).collect::<Vec<_>>()
            );
            report
                .validate()
                .unwrap_or_else(|e| panic!("{name}: {e:#}"));
            assert!(report.metadata.coverage.parse_failures.is_none());
            // Per-file detail is optional in every historical full export.
            let mut aggregate = value;
            aggregate.as_object_mut().unwrap().remove("files");
            let report: MeasurementReport = serde_json::from_value(aggregate).unwrap();
            let headline = serde_json::to_value(report.headline(None)).unwrap();
            assert!(schema.is_valid(&headline), "{name} headline");
        }
    }
}

#[test]
fn current_root_and_workspace_contracts_round_trip_without_shape_changes() {
    let temp = tempfile::tempdir().unwrap();
    std::fs::write(temp.path().join("main.py"), "print(1)\n").unwrap();
    for workspace in [false, true] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_locmetry"));
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
        let mut value = fixture("v0.1.6-root");
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
        let mut value = fixture("v0.1.6-root");
        let (parent, key) = pointer.rsplit_once('/').unwrap();
        value.pointer_mut(parent).unwrap()[key] = replacement;
        let typed: MeasurementReport = serde_json::from_value(value).unwrap();
        assert!(typed.validate().unwrap_err().to_string().contains(message));
    }
    for field in ["breakdown", "review", "files"] {
        let mut value = fixture("v0.1.6-root");
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
    let mut value = fixture("v0.1.6-root");
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
    for minor in 0..=6 {
        let report: MeasurementReport =
            serde_json::from_value(fixture(&format!("v0.1.{minor}-root"))).unwrap();
        report.validate().unwrap();
        assert_eq!(report.require_comparison_evidence().is_ok(), minor >= 2);
        assert!(report.headline(None).require_comparison_evidence().is_err());
    }
}

#[test]
fn duplicate_dynamic_map_keys_retain_the_last_value_before_validation() {
    let mut value = fixture("v0.1.6-root");
    value["summary"]["source_languages"]=serde_json::from_str(r#"{"Python":{"files":1,"code":9,"comments":0,"blanks":0,"lines":9},"Python":{"files":1,"code":1,"comments":0,"blanks":0,"lines":1}}"#).unwrap();
    let typed: MeasurementReport = serde_json::from_value(value).unwrap();
    assert_eq!(typed.summary.totals.source_languages["Python"].code, 1);
}

#[test]
fn workspace_modes_and_duplicate_metadata_identities_are_rejected() {
    for field in ["members", "modules"] {
        let mut value = fixture("v0.1.6-workspace");
        let rows = value["metadata"][field].as_array_mut().unwrap();
        rows.push(rows[0].clone());
        let typed: MeasurementReport = serde_json::from_value(value).unwrap();
        assert!(typed
            .validate()
            .unwrap_err()
            .to_string()
            .contains("duplicate"));
    }
    let mut value = fixture("v0.1.6-workspace");
    value["metadata"]["root"] = json!("/synthetic/invalid");
    let typed: MeasurementReport = serde_json::from_value(value.clone()).unwrap();
    assert!(typed.validate().unwrap_err().to_string().contains("mixed"));
    assert!(!jsonschema::validator_for(&locmetry::report_schema())
        .unwrap()
        .is_valid(&value));
}

#[test]
fn explicit_zero_summary_entries_equal_omission() {
    let mut value = fixture("v0.1.6-root");
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
    let mut value = fixture("v0.1.6-root");
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
fn historical_reports_round_trip_without_inventing_provenance() {
    for version in 0..=6 {
        for mode in if version >= 4 {
            vec!["root", "workspace"]
        } else {
            vec!["root"]
        } {
            let name = format!("v0.1.{version}-{mode}");
            let original = fixture(&name);
            let report: MeasurementReport = serde_json::from_value(original.clone()).unwrap();
            assert_eq!(serde_json::to_value(report).unwrap(), original, "{name}");
        }
    }
}

#[test]
fn schemas_and_reader_reject_invalid_purpose_map_keys() {
    let schema = jsonschema::validator_for(&locmetry::report_schema()).unwrap();
    for invalid in ["bogus", "Test"] {
        for path in ["/summary/source_purpose", "/summary/top_folders/src"] {
            let mut value = fixture("v0.1.6-root");
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
        let mut command = Command::new(env!("CARGO_BIN_EXE_locmetry"));
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
    let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
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
