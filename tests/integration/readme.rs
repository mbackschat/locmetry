use crate::common;
use std::fs;

const SAMPLE: &str = "examples/cli-workspace/acme-orders";

#[test]
fn sample_workspace_records_real_classification_and_embedded_counts() {
    let output = common::locmetry()
        .args([
            "--root",
            SAMPLE,
            "--no-user-config",
            "--format",
            "json",
            "--files",
        ])
        .output()
        .unwrap();
    let report = common::json(&output, 0);
    let files = report["files"].as_array().unwrap();
    let file = |path: &str| files.iter().find(|f| f["path"] == path).unwrap().clone();
    assert_eq!(file("service/src/main.go")["purpose"], "production");
    assert_eq!(
        file("web/src/orders/Order.test.ts")["rule"],
        "builtin:js-test-file"
    );
    assert_eq!(file("service/scripts/check.py")["category"], "tooling");
    assert_eq!(
        file("service/experiments/probe.py")["category"],
        "unclaimed"
    );
    assert!(!files
        .iter()
        .any(|f| f["path"].as_str().unwrap().contains("third_party")
            || f["path"] == "service/ignored.py"
            || f["path"] == "service/.hidden.py"));
    assert!(report["breakdown"]
        .as_array()
        .unwrap()
        .iter()
        .any(|row| row["language"] == "JavaScript" && row["files"] == 0 && row["code"] == 1));
    assert_eq!(report["summary"]["source"]["code"], 25);
}

fn help(flag: &str) -> String {
    let output = common::locmetry().arg(flag).output().unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}

// Long help owns the examples; the README table is a checked copy, so neither can drift.
#[test]
fn readme_common_commands_mirror_long_help_examples() {
    let long = help("--help");
    let examples = common::help_examples(&long);
    let readme = fs::read_to_string("README.md").unwrap();
    let table: Vec<(String, String)> = readme
        .split_once("## Common commands\n\n| Task | Command |\n| --- | --- |\n")
        .unwrap()
        .1
        .lines()
        .take_while(|line| line.starts_with('|'))
        .map(|row| {
            let (task, command) = row
                .strip_prefix("| ")
                .and_then(|row| row.strip_suffix(" |"))
                .and_then(|row| row.split_once(" | "))
                .unwrap();
            (task.to_owned(), command.trim_matches('`').to_owned())
        })
        .collect();
    assert!(!examples.is_empty());
    assert_eq!(
        table, examples,
        "edit the examples in src/main.rs `after_long_help`, then mirror them in README"
    );
    assert_eq!(help("help"), long);
    assert!(!help("-h").contains("Examples:"));
}
