use serde_json::{json, Value};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .current_dir(root)
        .args(["--no-user-config"])
        .args(args)
        .output()
        .unwrap()
}

fn cell(kind: &str, source: Value) -> Value {
    let mut cell = json!({"id":"example", "cell_type":kind, "metadata":{}, "source":source});
    if kind == "code" {
        cell["execution_count"] = Value::Null;
        cell["outputs"] = json!([]);
    }
    cell
}

fn notebook(mut cells: Vec<Value>) -> Value {
    for (index, cell) in cells.iter_mut().enumerate() {
        cell["id"] = json!(format!("cell-{index}"));
    }
    json!({"nbformat":4,"nbformat_minor":5,"metadata":{},"cells":cells})
}

fn write(root: &Path, value: &Value) {
    fs::write(
        root.join("example.ipynb"),
        serde_json::to_vec(value).unwrap(),
    )
    .unwrap();
}

fn full(root: &Path) -> Value {
    let output = run(root, &["--files", "--output", "-"]);
    assert_eq!(
        output.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    checked_report(&output)
}

fn checked_report(output: &Output) -> Value {
    let report: locmetry::contract::MeasurementReport =
        serde_json::from_slice(&output.stdout).unwrap();
    report.validate().unwrap();
    serde_json::from_slice(&output.stdout).unwrap()
}

fn counts(code: u64, comments: u64, blanks: u64) -> Value {
    json!({"code":code,"comments":comments,"blanks":blanks,"lines":code+comments+blanks})
}

#[test]
fn notebook_cells_count_once_with_independent_line_oracles() {
    let cases = [
        (vec![cell("code", json!(["print(1)\n"]))], counts(1, 0, 0)),
        (
            vec![cell("code", json!("print(1)\nprint(2)\n"))],
            counts(2, 0, 0),
        ),
        (
            vec![cell("markdown", json!("first\n\nlast\n"))],
            counts(0, 2, 1),
        ),
        (
            vec![cell(
                "markdown",
                json!("heading\n```python\nprint(1)\n```\n"),
            )],
            counts(0, 4, 0),
        ),
        (
            vec![
                cell("raw", json!("raw text\n")),
                cell("code", json!("print(1)\n")),
            ],
            counts(1, 1, 0),
        ),
        (vec![], counts(0, 0, 0)),
    ];
    for (cells, expected) in cases {
        let root = tempfile::tempdir().unwrap();
        let mut value = notebook(cells);
        value["metadata"] = json!({"kernelspec":{"name":"python3","display_name":"Python 3","language":"python"},"language_info":{"name":"python","file_extension":".py"}});
        write(root.path(), &value);
        let report = full(root.path());
        assert_eq!(report["files"][0]["counts"], expected);
        assert_eq!(report["files"][0]["language"], "Jupyter Notebooks");
        assert_eq!(
            report["summary"]["source_languages"]["Jupyter Notebooks"]["files"],
            1
        );
        assert_eq!(report["metadata"]["coverage"]["parse_failures"], json!([]));
        assert_eq!(report["summary"]["all_counted"]["lines"], expected["lines"]);
        let rows = report["breakdown"].as_array().unwrap();
        for key in ["code", "comments", "blanks", "lines"] {
            assert_eq!(
                rows.iter()
                    .map(|row| row[key].as_u64().unwrap())
                    .sum::<u64>(),
                expected[key].as_u64().unwrap()
            );
        }
        assert_eq!(
            rows.iter()
                .map(|row| row["files"].as_u64().unwrap())
                .sum::<u64>(),
            1
        );
    }
}

#[test]
fn notebook_source_fragments_and_optional_metadata_preserve_counts() {
    let root = tempfile::tempdir().unwrap();
    for metadata in [
        json!({}),
        json!({"language_info":{"name":"python"}}),
        json!({"kernelspec":{"name":"python3","display_name":"Python 3","language":"python"}}),
    ] {
        for source in [
            json!("print(1)\n# comment\n\n"),
            json!(["print(", "1)\n# comment\n", "\n"]),
        ] {
            let mut value = notebook(vec![cell("code", source)]);
            value["metadata"] = metadata.clone();
            write(root.path(), &value);
            assert_eq!(full(root.path())["files"][0]["counts"], counts(1, 1, 1));
        }
    }
    for minor in 0..=5 {
        let mut value = notebook(vec![cell("code", json!("print(1)\n"))]);
        value["nbformat_minor"] = json!(minor);
        write(root.path(), &value);
        assert_eq!(full(root.path())["files"][0]["counts"], counts(1, 0, 0));
    }
}

#[test]
fn notebook_language_metadata_precedence_and_extensions_select_the_cell_counter() {
    let root = tempfile::tempdir().unwrap();
    for metadata in [
        json!({"language_info":{"name":"javascript"},"kernelspec":{"language":"python"}}),
        json!({"kernelspec":{"language":"javascript"}}),
        json!({"language_info":{"file_extension":".js"}}),
    ] {
        let mut value = notebook(vec![cell("code", json!("// comment\nconst x = 1;\n"))]);
        value["metadata"] = metadata;
        write(root.path(), &value);
        assert_eq!(full(root.path())["files"][0]["counts"], counts(1, 1, 0));
    }
}

#[test]
fn malformed_and_unsupported_notebooks_are_unavailable_instead_of_zero() {
    let mut version = notebook(vec![]);
    version["nbformat"] = json!(5);
    let mut language = notebook(vec![cell("code", json!("code\n"))]);
    language["metadata"] =
        json!({"language_info":{"name":"unknown-language"},"kernelspec":{"language":"python"}});
    let cases = [
        (String::new(), "invalid_notebook"),
        ("{ invalid".into(), "invalid_notebook"),
        (
            json!({"hello":"not a notebook"}).to_string(),
            "invalid_notebook",
        ),
        (
            notebook(vec![cell("code", json!(42))]).to_string(),
            "invalid_notebook",
        ),
        (version.to_string(), "unsupported_notebook_version"),
        (language.to_string(), "unsupported_notebook_language"),
        (
            notebook(vec![cell("future", json!("text\n"))]).to_string(),
            "unsupported_notebook_cell",
        ),
    ];
    for (content, reason) in cases {
        let root = tempfile::tempdir().unwrap();
        fs::write(root.path().join("example.ipynb"), content).unwrap();
        fs::write(root.path().join("valid.py"), "print(1)\n").unwrap();
        for args in [vec!["--files", "--output", "-"], vec!["--format", "json"]] {
            let output = run(root.path(), &args);
            assert_eq!(output.status.code(), Some(2), "{reason}");
            let report = checked_report(&output);
            assert_eq!(report["metadata"]["backend_inaccurate"], true);
            let coverage = &report["metadata"]["coverage"];
            assert_eq!(coverage["selected_files"], 2);
            assert_eq!(coverage["counted_files"], 1);
            assert_eq!(coverage["unread"], json!([]));
            assert_eq!(coverage["parse_failures"].as_array().unwrap().len(), 1);
            assert_eq!(coverage["parse_failures"][0]["path"], "example.ipynb");
            assert_eq!(coverage["parse_failures"][0]["reason"], reason);
            assert!(!coverage["parse_failures"][0]["message"]
                .as_str()
                .unwrap()
                .is_empty());
            assert_eq!(report["summary"]["all_counted"]["code"], 1);
            let stderr = String::from_utf8_lossy(&output.stderr);
            assert!(stderr.contains("example.ipynb"), "{stderr}");
            assert!(!stderr.contains("Unreadable files: none identified"));
        }
        for format in ["table", "markdown"] {
            let output = run(root.path(), &["--format", format]);
            assert_eq!(output.status.code(), Some(2));
            let human = String::from_utf8_lossy(&output.stdout);
            assert!(human.contains("example.ipynb"), "{human}");
            assert!(human.contains(reason), "{human}");
        }
    }
}

#[test]
fn mapped_notebooks_use_the_same_counter_and_failure_reporting() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join(".locmetryrc"), "version = 1\n[[languages.mappings]]\nid = \"notebooks\"\nmatch = [\"*.customnb\"]\nlanguage = \"Jupyter Notebooks\"\n").unwrap();
    fs::write(
        root.path().join("valid.customnb"),
        notebook(vec![cell("code", json!("print(1)\n"))]).to_string(),
    )
    .unwrap();
    fs::write(root.path().join("bad.customnb"), "{}").unwrap();
    let output = run(root.path(), &["--files", "--output", "-"]);
    assert_eq!(output.status.code(), Some(2));
    let report = checked_report(&output);
    assert_eq!(report["files"][0]["counts"], counts(1, 0, 0));
    assert_eq!(report["files"][0]["language_mapping"], "notebooks");
    assert_eq!(report["metadata"]["coverage"]["mapped_files"], 1);
    assert_eq!(
        report["metadata"]["coverage"]["parse_failures"][0]["path"],
        "bad.customnb"
    );
}

#[test]
fn workspace_notebook_failures_retain_member_coverage_and_prefixed_paths() {
    let root = tempfile::tempdir().unwrap();
    for name in ["bad", "good"] {
        fs::create_dir(root.path().join(name)).unwrap();
    }
    fs::write(root.path().join("bad/broken.ipynb"), "{}").unwrap();
    write(
        &root.path().join("good"),
        &notebook(vec![cell("code", json!("print(1)\n"))]),
    );
    for format in ["json", "table", "markdown"] {
        let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
            .current_dir(root.path())
            .args([
                "workspace",
                "--no-user-config",
                "--root",
                "good",
                "--root",
                "bad",
                "--format",
                format,
            ])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        if format == "json" {
            let report = checked_report(&output);
            assert_eq!(
                report["metadata"]["coverage"]["parse_failures"][0]["path"],
                "bad/broken.ipynb"
            );
            assert_eq!(
                report["metadata"]["members"][0]["coverage"]["parse_failed_files"],
                1
            );
            assert_eq!(
                report["metadata"]["members"][1]["coverage"]["parse_failed_files"],
                0
            );
            assert_eq!(report["summary"]["source"]["code"], 1);
        } else {
            assert!(String::from_utf8_lossy(&output.stdout).contains("bad/broken.ipynb"));
        }
    }
}

#[test]
fn report_schema_distinguishes_historical_absence_from_recorded_parse_failures() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), &notebook(vec![]));
    let mut report = full(root.path());
    let schema: Value =
        serde_json::from_slice(&run(root.path(), &["--schema", "report"]).stdout).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    assert_eq!(report["metadata"]["coverage"]["parse_failures"], json!([]));
    assert!(validator.is_valid(&report));
    report["metadata"]["coverage"]
        .as_object_mut()
        .unwrap()
        .remove("parse_failures");
    assert!(validator.is_valid(&report));
    report["metadata"]["coverage"]["parse_failures"] = Value::Null;
    assert!(!validator.is_valid(&report));
    report["metadata"]["coverage"]["parse_failures"] =
        json!([{"path":"x.ipynb","reason":"invalid_notebook","message":"Invalid JSON"}]);
    assert!(validator.is_valid(&report));
    report["metadata"]["coverage"]["parse_failures"][0]["reason"] = json!("surprise");
    assert!(!validator.is_valid(&report));
}

#[test]
fn notebook_bom_and_cell_boundaries_preserve_independent_counts() {
    let root = tempfile::tempdir().unwrap();
    let value = notebook(vec![
        cell("code", json!("print(1)")),
        cell("code", json!("# comment\n\n")),
    ]);
    let text = value.to_string();
    let mut bytes = vec![0xff, 0xfe];
    for unit in text.encode_utf16() {
        bytes.extend(unit.to_le_bytes());
    }
    fs::write(root.path().join("example.ipynb"), bytes).unwrap();
    assert_eq!(full(root.path())["files"][0]["counts"], counts(1, 1, 1));
}

#[cfg(unix)]
#[test]
fn unreadable_notebooks_remain_io_failures() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir().unwrap();
    write(root.path(), &notebook(vec![]));
    let path = root.path().join("example.ipynb");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o000)).unwrap();
    let restricted = fs::read(&path).is_err();
    let output = run(root.path(), &["--output", "-"]);
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).unwrap();
    if !restricted {
        return;
    }
    assert_eq!(output.status.code(), Some(2));
    let report = checked_report(&output);
    assert_eq!(
        report["metadata"]["coverage"]["unread"],
        json!(["example.ipynb"])
    );
    assert_eq!(report["metadata"]["coverage"]["parse_failures"], json!([]));
}
