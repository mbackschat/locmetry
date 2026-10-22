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
    // One file per remaining classification stage: an override beating the
    // npm module layout, a build convention and a language mapping.
    let generated = file("web/src/api/client.ts");
    assert_eq!(
        (
            &generated["purpose"],
            &generated["category"],
            &generated["rule"]
        ),
        (
            &"other".into(),
            &"generated".into(),
            &"generated-client".into()
        )
    );
    let makefile = file("service/Makefile");
    assert_eq!(
        (&makefile["category"], &makefile["rule"]),
        (&"build-tooling".into(), &"builtin:build-script".into())
    );
    let template = file("web/src/orders/receipt.acmetpl");
    assert_eq!(
        (&template["language"], &template["purpose"]),
        (&"HTML".into(), &"production".into())
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
    // 17 production lines plus the Makefile's two and the template's one;
    // Other adds the generated client's line to the unclaimed experiment's.
    let purposes = &report["summary"]["source_purpose"];
    assert_eq!(report["summary"]["source"]["code"], 29);
    let code = |purpose: &str| purposes[purpose]["code"].as_u64();
    assert_eq!(
        [code("production"), code("test"), code("other")],
        [Some(20), Some(7), Some(2)]
    );
    assert_eq!(report["metadata"]["coverage"]["mapped_files"], 1);
}

/// The fixture README promises that member configurations express the root
/// policy relative to each member, so every file must get the same decision,
/// as it must when the members are folders under the root's own policy.
#[test]
fn sample_workspace_members_decide_like_the_combined_root() {
    let measure = |args: &[&str]| {
        let output = common::locmetry()
            .args(args)
            .args(["--no-user-config", "--format", "json", "--files"])
            .output()
            .unwrap();
        let report = common::json(&output, 0);
        let mut decisions: Vec<_> = report["files"]
            .as_array()
            .unwrap()
            .iter()
            .map(|f| {
                let field = |name: &str| f[name].as_str().unwrap().to_string();
                (
                    field("path"),
                    field("purpose"),
                    field("category"),
                    field("rule"),
                )
            })
            .collect();
        decisions.sort();
        decisions
    };
    let combined: Vec<_> = measure(&["--root", SAMPLE])
        .into_iter()
        .filter(|(path, ..)| !path.ends_with(".locmetryrc"))
        .collect();
    // Independent roots with their own policies, and the same folders under
    // the combined root's single policy.
    for workspace_file in [
        "examples/cli-workspace/workspace.toml",
        "examples/cli-workspace/folders.toml",
    ] {
        let members: Vec<_> = measure(&["workspace", "--workspace-file", workspace_file])
            .into_iter()
            .filter(|(path, ..)| !path.ends_with(".locmetryrc"))
            .collect();
        assert_eq!(members, combined, "{workspace_file}");
    }
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
        "edit the examples in src/app/cli/long_help.rs `root_examples!`, then mirror them in README"
    );
    assert_eq!(help("help"), long);
    assert!(!help("-h").contains("Examples:"));
}
