//! Module credit by layout coverage (docs/proposals/module-credit-by-coverage.md):
//! a module is credited with the production source its layout covers,
//! whichever stage decided it, so an entry that only names a category does
//! not make the module look smaller.
use crate::cli::*;
use serde_json::{json, Value};
use std::process::Command;

const POLICY: &str = r#"version = 1
[[rules]]
id = "android"
match = ["app/src/main/kotlin/android/**"]
purpose = "production"
category = "native"

[[rules]]
id = "parked"
match = ["app/src/main/kotlin/parked/**"]
purpose = "other"
category = "parked"
"#;

fn fixture() -> tempfile::TempDir {
    project(&[
        (".locmetryrc", POLICY),
        ("app/build.gradle.kts", "plugins {}\n"),
        (
            "app/src/main/kotlin/App.kt",
            "fun a() = 1\nfun b() = 2\nfun c() = 3\n",
        ),
        (
            "app/src/main/kotlin/android/Runner.kt",
            "fun r() = 1\nfun s() = 2\n",
        ),
        (
            "app/src/main/kotlin/parked/Old.kt",
            "fun o() = 1\n".repeat(4).as_str(),
        ),
        ("app/src/test/kotlin/AppTest.kt", "fun t() = 1\n"),
        ("empty/build.gradle.kts", "plugins {}\n"),
    ])
}

fn file<'a>(document: &'a Value, path: &str) -> &'a Value {
    document["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"] == path)
        .unwrap_or_else(|| panic!("{path} is not in the report"))
}

#[test]
fn entries_that_keep_production_keep_the_modules_credit() {
    let temp = fixture();
    let root = temp.path().join("project");
    let (success, document, stderr) = report(&root, &[]);
    assert!(success, "{stderr}");
    let app = json!({"path": "app", "detector": "gradle"});
    assert_eq!(file(&document, "app/src/main/kotlin/App.kt")["module"], app);
    let runner = file(&document, "app/src/main/kotlin/android/Runner.kt");
    assert_eq!(
        (&runner["rule"], &runner["module"]),
        (&json!("android"), &app)
    );
    for path in [
        "app/src/main/kotlin/parked/Old.kt",
        "app/src/test/kotlin/AppTest.kt",
    ] {
        assert!(file(&document, path).get("module").is_none(), "{path}");
    }
    // `empty` covers no production source: absent from the summary, kept as
    // detected context.
    assert_eq!(
        document["summary"]["build_modules"],
        json!({"gradle": {"modules": 1, "code": 5}})
    );
    assert_eq!(
        document["metadata"]["modules"],
        json!([app, {"path": "empty", "detector": "gradle"}])
    );
    // Only files a layout decided are listed for review.
    let listed: Vec<_> = document["summary"]["largest_module_files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| file["path"].as_str().unwrap())
        .collect();
    assert_eq!(listed, ["app/src/main/kotlin/App.kt"]);

    let table = String::from_utf8(run(&root, &["--verbose", "--format", "table"]).stdout).unwrap();
    assert!(
        table.contains(" Production from build modules: 5 code lines in 1 'gradle' module.\n"),
        "{table}"
    );
    let markdown = String::from_utf8(run(&root, &["--format", "markdown"]).stdout).unwrap();
    let section = &markdown[markdown.find("## Build modules").expect(&markdown)..];
    for row in [
        "| gradle | 1 | 5 | 5 |",
        "| app | gradle | 5 | 5 |",
        "| empty | gradle | 0 | 0 |",
    ] {
        assert!(section.contains(row), "{row} in {section}");
    }
}

/// The report contract recomputes credit from `module`, so a reader detects a
/// summary or a file record that disagrees with it.
#[test]
fn the_report_contract_recomputes_credit_from_file_modules() {
    let temp = fixture();
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    let validate = |value: &Value| {
        serde_json::from_value::<locmetry::contract::MeasurementReport>(value.clone())
            .unwrap()
            .validate()
    };
    validate(&document).unwrap();
    let index = |path: &str| {
        document["files"]
            .as_array()
            .unwrap()
            .iter()
            .position(|file| file["path"] == path)
            .unwrap()
    };
    let mut dropped = document.clone();
    dropped["files"][index("app/src/main/kotlin/android/Runner.kt")]
        .as_object_mut()
        .unwrap()
        .remove("module");
    let error = validate(&dropped).unwrap_err().to_string();
    assert!(error.contains("build_modules conflicts"), "{error}");
    let mut misplaced = document.clone();
    misplaced["files"][index("app/src/main/kotlin/parked/Old.kt")]["module"] =
        json!({"path": "app", "detector": "gradle"});
    let error = validate(&misplaced).unwrap_err().to_string();
    assert!(error.contains("module"), "{error}");
}

#[test]
fn explain_names_the_module_an_entry_decided_file_credits() {
    let temp = fixture();
    let root = temp.path().join("project");
    let explain = |path: &str| {
        let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
            .args(["explain", path, "--no-user-config", "--format", "table"])
            .current_dir(&root)
            .output()
            .unwrap();
        assert!(output.status.success());
        String::from_utf8(output.stdout).unwrap()
    };
    let runner = explain("app/src/main/kotlin/android/Runner.kt");
    assert!(
        runner.contains("Module credit: app (gradle), whose layout covers this file\n"),
        "{runner}"
    );
    assert!(!runner.contains("\nModule: "), "{runner}");
    let app = explain("app/src/main/kotlin/App.kt");
    assert!(app.contains("\nModule: app (gradle)\n"), "{app}");
    assert!(!app.contains("Module credit"), "{app}");
    let parked = explain("app/src/main/kotlin/parked/Old.kt");
    assert!(!parked.contains("Module"), "{parked}");
}
