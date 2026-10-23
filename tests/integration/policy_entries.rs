//! Reviewable policy entries (docs/proposals/policy-entry-review.md): `except`
//! lists on classification entries.
use crate::cli::*;
use crate::common::locmetry;
use serde_json::Value;

fn measured(config: &str, files: &[(&str, &str)]) -> Value {
    let files = [&[(".locmetryrc", config)], files].concat();
    let temp = project(&files);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    document
}

fn rule_of(document: &Value, path: &str) -> String {
    let file = document["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"] == path)
        .unwrap_or_else(|| panic!("no record for {path}"));
    format!("{}:{}@{}", file["purpose"], file["category"], file["rule"]).replace('"', "")
}

const LAWS: &str = r#"version = 1
[[overrides]]
id = "agreement-laws"
match = ["**/*AgreementTest.java"]
except = ["**/FieldAgreementTest.java"]
purpose = "test"
category = "agreement-law"
"#;

#[test]
fn an_excepted_file_falls_through_to_the_next_stage() {
    let document = measured(
        LAWS,
        &[
            ("src/test/java/RuleAgreementTest.java", "class A {}\n"),
            ("src/test/java/FieldAgreementTest.java", "class B {}\n"),
        ],
    );
    assert_eq!(
        rule_of(&document, "src/test/java/RuleAgreementTest.java"),
        "test:agreement-law@agreement-laws"
    );
    assert_eq!(
        rule_of(&document, "src/test/java/FieldAgreementTest.java"),
        "test:test@builtin:test-directory"
    );
    assert_eq!(
        document["metadata"]["config"]["overrides"][0]["except"][0],
        "**/FieldAgreementTest.java"
    );
}

#[test]
fn explain_shows_the_stage_an_excepted_file_falls_through_to() {
    let temp = project(&[
        (".locmetryrc", LAWS),
        ("src/test/java/FieldAgreementTest.java", "class B {}\n"),
    ]);
    let output = locmetry()
        .current_dir(temp.path().join("project"))
        .args(["explain", "src/test/java/FieldAgreementTest.java"])
        .output()
        .unwrap();
    let document = crate::common::json(&output, 0);
    assert_eq!(document["result"]["decision"]["stage"], "convention");
}

#[test]
fn an_empty_or_invalid_except_list_is_rejected() {
    for (except, message) in [
        ("[]", "needs a nonempty except array"),
        ("[\"a/{b\"]", "Invalid except in entry"),
    ] {
        let config =
            format!("version = 1\n[[rules]]\nid = \"app\"\nmatch = [\"src/**\"]\nexcept = {except}\npurpose = \"production\"\n");
        let temp = project(&[(".locmetryrc", &config), ("src/a.rs", "fn a() {}\n")]);
        let (success, _, stderr) = report(&temp.path().join("project"), &[]);
        assert!(!success);
        assert!(stderr.contains(message), "{stderr}");
    }
}
