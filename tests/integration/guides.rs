use crate::common;
use common::write;
use serde_json::{json, Value};
use std::{fs, path::Path, process::Output};

fn run(root: &Path, args: &[&str]) -> Output {
    common::locmetry()
        .current_dir(root)
        .env("XDG_CONFIG_HOME", root.join("personal"))
        .args(args)
        .output()
        .unwrap()
}

fn document(root: &Path, args: &[&str], code: i32) -> Value {
    common::json(&run(root, args), code)
}

fn source(topic: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("docs/guides/{topic}.md")),
    )
    .unwrap()
}

fn block<'a>(text: &'a str, marker: &str, language: &str) -> &'a str {
    text.split_once(&format!("<!-- {marker} -->"))
        .unwrap()
        .1
        .split_once(&format!("```{language}\n"))
        .unwrap()
        .1
        .split_once("```")
        .unwrap()
        .0
}

fn expected(topic: &str, marker: &str) -> Value {
    serde_json::from_str(block(&source(topic), marker, "json")).unwrap()
}

#[test]
fn guides_print_their_exact_sources_without_loading_configuration() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), ".locmetryrc", "invalid toml");
    write(root.path(), "personal/locmetry/config.toml", "invalid toml");
    for topic in ["configure", "selection", "reports", "compare"] {
        let out = run(root.path(), &["--guide", topic]);
        assert_eq!(
            out.status.code(),
            Some(0),
            "{topic}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(out.stderr.is_empty());
        let expected = format!(
            "Source: docs/guides/{topic}.md; locmetry {}\n\n{}",
            env!("CARGO_PKG_VERSION"),
            source(topic)
        );
        assert_eq!(out.stdout, expected.as_bytes());
        assert!(!out.stdout.contains(&27));
    }
}

#[test]
fn help_supplies_the_starting_instruction_and_root_level_next_commands() {
    let root = tempfile::tempdir().unwrap();
    for args in [vec!["--help"], vec!["-h"], vec!["help"]] {
        let out = run(root.path(), &args);
        assert_eq!(out.status.code(), Some(0));
        let help = String::from_utf8(out.stdout)
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            help.contains("Run locmetry for source counts by language. Add --show-folders for the folder table, or --verbose to inspect classification, scope and coverage notes. For optional project configuration, use locmetry --guide configure."),
            "{args:?}: {help}"
        );
        if args != ["-h"] {
            for description in [
                "configure: Create or evaluate project configuration with evidence and open questions",
                "selection: Diagnose missing files, unexpected classifications and subfolder scope",
                "reports: Interpret counts, coverage, output formats and exports",
                "compare: Capture full baselines and interpret saved-report net change",
            ] {
                assert!(help.contains(description), "{help}");
            }
        }
    }
    for (command, topics) in [
        ("explain", vec!["selection"]),
        ("compare", vec!["compare"]),
        ("workspace", vec!["configure", "reports"]),
    ] {
        for flag in ["--help", "-h"] {
            let out = run(root.path(), &[command, flag]);
            assert_eq!(out.status.code(), Some(0));
            let help = String::from_utf8(out.stdout)
                .unwrap()
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ");
            for topic in &topics {
                assert!(
                    help.contains(&format!("locmetry --guide {topic}")),
                    "{help}"
                );
            }
        }
    }
}

#[test]
fn guide_errors_are_actionable_and_queries_are_exclusive() {
    let root = tempfile::tempdir().unwrap();
    for args in [
        vec!["--guide"],
        vec!["--guide", "index"],
        vec!["--guide", "Configure"],
        vec!["--guide", "./configure.md"],
    ] {
        let out = run(root.path(), &args);
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stdout.is_empty());
        let error = String::from_utf8_lossy(&out.stderr);
        assert!(error.contains("--help"), "{error}");
        for topic in ["configure", "selection", "reports", "compare"] {
            assert!(error.contains(topic), "{error}");
        }
    }
    for args in [
        vec!["--guide", "reports", "--schema", "report"],
        vec!["--schema", "report", "--guide", "reports"],
    ] {
        let out = run(root.path(), &args);
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stdout.is_empty());
        let error = String::from_utf8_lossy(&out.stderr);
        assert!(error.contains("--guide") && error.contains("--schema"));
    }
    for early in ["--help", "-h", "--version"] {
        assert!(run(
            root.path(),
            &["--guide", "configure", "--markdown-depth", "2", early]
        )
        .status
        .success());
    }
}

#[test]
fn configuration_example_preserves_decisions_and_open_questions() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "package.json", "{}\n");
    write(root.path(), "src/app.js", "export const app = 1;\n");
    write(root.path(), "src/sample.js", "export const sample = 1;\n");
    write(root.path(), "scripts/tool.py", "print(1)\n");
    write(root.path(), "unknown.py", "print(2)\n");
    let before = document(root.path(), &["--no-user-config", "--format", "json"], 0);
    assert_eq!(before["summary"]["source_purpose"]["production"]["code"], 2);
    assert!(!before["review"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["path"] == "src/sample.js"));
    write(
        root.path(),
        ".locmetryrc",
        block(&source("configure"), "configuration-example", "toml"),
    );
    let after = document(root.path(), &["--no-user-config", "--format", "json"], 0);
    let projection = json!({"source_purpose":after["summary"]["source_purpose"], "unclaimed": after["review"].as_array().unwrap().iter().map(|r| r["path"].clone()).collect::<Vec<_>>()});
    assert_eq!(projection, expected("configure", "configuration-result"));
    for (path, purpose, rule) in [
        ("src/app.js", "production", "builtin:npm-module"),
        ("src/sample.js", "other", "historical-sample"),
        ("scripts/tool.py", "production", "tooling"),
        ("unknown.py", "other", "fallback"),
    ] {
        let result = document(
            root.path(),
            &["explain", path, "--no-user-config", "--format", "json"],
            0,
        );
        assert_eq!(result["result"]["file"]["purpose"], purpose);
        assert_eq!(result["result"]["file"]["rule"], rule);
    }
    let schema = document(root.path(), &["--schema", "config"], 0);
    let description = schema["description"].as_str().unwrap();
    assert!(description.contains("open question"));
    assert!(!description.contains("read `summary.unclaimed_folders` and add entries"));
}

#[test]
fn configuration_mapping_example_keeps_recognition_separate_from_purpose() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "bin/launch", "console.log('start');\n");
    let guide = source("configure");
    let mapping = block(&guide, "recognition-mapping", "toml");
    let baseline = document(
        root.path(),
        &["--no-user-config", "--files", "--output", "-"],
        0,
    );
    assert_eq!(
        baseline["metadata"]["coverage"]["unrecognized"],
        json!(["bin/launch"])
    );

    write(root.path(), ".locmetryrc", mapping);
    let mapped = document(
        root.path(),
        &["--no-user-config", "--files", "--output", "-"],
        0,
    );
    let launcher = mapped["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"] == "bin/launch")
        .unwrap();
    assert_eq!(launcher["counts"]["code"], 1);
    assert_eq!(launcher["language_mapping"], "node-launcher");
    assert_eq!(launcher["purpose"], "other");
    assert_eq!(launcher["category"], "unclaimed");
    assert_eq!(launcher["rule"], "fallback");
}

#[test]
fn configuration_category_example_requires_an_override_without_changing_purpose() {
    let root = tempfile::tempdir().unwrap();
    write(
        root.path(),
        "__test__/helpers/setup.js",
        "export const setup = 1;\n",
    );
    let guide = source("configure");
    let ordinary = block(&guide, "category-rule", "toml");
    let override_entry = block(&guide, "category-override", "toml");
    write(root.path(), ".locmetryrc", ordinary);
    let classified = document(
        root.path(),
        &["--no-user-config", "--files", "--output", "-"],
        0,
    );
    let helper = document(
        root.path(),
        &[
            "explain",
            "__test__/helpers/setup.js",
            "--no-user-config",
            "--format",
            "json",
        ],
        0,
    );
    assert_eq!(helper["result"]["file"]["purpose"], "test");
    assert_eq!(helper["result"]["file"]["category"], "test");
    assert_eq!(helper["result"]["decision"]["stage"], "convention");

    write(root.path(), ".locmetryrc", override_entry);
    let refined = document(
        root.path(),
        &["--no-user-config", "--files", "--output", "-"],
        0,
    );
    assert_eq!(
        classified["summary"]["source_purpose"],
        refined["summary"]["source_purpose"]
    );
    let helper = document(
        root.path(),
        &[
            "explain",
            "__test__/helpers/setup.js",
            "--no-user-config",
            "--format",
            "json",
        ],
        0,
    );
    assert_eq!(helper["result"]["file"]["purpose"], "test");
    assert_eq!(helper["result"]["file"]["category"], "test-support");
    assert_eq!(helper["result"]["decision"]["stage"], "override");
}

#[test]
fn documented_discovery_path_and_topic_destinations_are_executable() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "main.py", "print(1)\n");
    let guide = source("configure");
    for line in block(&guide, "discovery-commands", "sh").lines() {
        let args: Vec<_> = line.split_whitespace().skip(1).collect();
        let out = run(root.path(), &args);
        assert!(
            out.status.success(),
            "{line}: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        if args.contains(&"headline") {
            let value: Value = serde_json::from_slice(&out.stdout).unwrap();
            assert_eq!(value["summary"]["source"]["code"], 1);
        }
    }
    for (topic, commands) in [
        (
            "configure",
            vec![
                "locmetry --schema config",
                "locmetry --root . --format headline",
                "locmetry --schema user-config",
                "locmetry workspace --help",
                "locmetry --schema workspace",
                "locmetry --guide reports",
                "locmetry --guide selection",
            ],
        ),
        (
            "selection",
            vec![
                "locmetry explain --help",
                "locmetry --schema explanation",
                "locmetry --guide configure",
                "locmetry --guide reports",
            ],
        ),
        (
            "reports",
            vec![
                "locmetry --schema report",
                "locmetry --guide selection",
                "locmetry --guide configure",
                "locmetry --guide compare",
            ],
        ),
        (
            "compare",
            vec![
                "locmetry compare --help",
                "locmetry --schema comparison",
                "locmetry --guide reports",
            ],
        ),
    ] {
        let text = source(topic);
        for command in commands {
            assert!(text.contains(command), "{topic} must provide {command}");
            let args: Vec<_> = command.split_whitespace().skip(1).collect();
            let out = run(root.path(), &args);
            assert!(
                out.status.success(),
                "{topic}: {command}: {}",
                String::from_utf8_lossy(&out.stderr)
            );
            if args.first() == Some(&"--schema") {
                let value: Value = serde_json::from_slice(&out.stdout).unwrap();
                assert!(value.get("$schema").is_some());
            }
        }
    }
}

#[test]
fn report_example_distinguishes_unavailable_counts_from_zero() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "empty.py", "");
    write(root.path(), "broken.ipynb", "{}");
    let value = document(
        root.path(),
        &["--no-user-config", "--files", "--format", "json"],
        2,
    );
    let projection = json!({"backend_inaccurate":value["metadata"]["backend_inaccurate"], "selected_files":value["metadata"]["coverage"]["selected_files"], "counted_files":value["metadata"]["coverage"]["counted_files"], "parse_failures":value["metadata"]["coverage"]["parse_failures"].as_array().unwrap().iter().map(|f|json!({"path":f["path"],"reason":f["reason"]})).collect::<Vec<_>>(), "files":value["files"].as_array().unwrap().iter().map(|f|json!({"path":f["path"],"code":f["counts"]["code"]})).collect::<Vec<_>>()});
    assert_eq!(projection, expected("reports", "coverage-result"));
}

#[test]
fn report_guide_saves_markdown_and_prints_full_file_evidence_from_one_measurement() {
    let temp = tempfile::tempdir().unwrap();
    write(temp.path(), "service/src/main.rs", "fn a() {}\nfn b() {}\n");
    let guide = source("reports");
    let row = guide
        .lines()
        .find(|line| line.starts_with("| Save Markdown and print full per-file JSON |"))
        .unwrap();
    let command = row.split('`').nth(1).unwrap();
    let args: Vec<_> = command.split_whitespace().skip(1).collect();
    let output = run(temp.path(), &args);
    let value = common::json(&output, 0);
    assert!(output.stderr.is_empty());
    assert_eq!(value["detail"], "full");
    assert_eq!(value["summary"]["source"]["code"], 2);
    assert_eq!(value["files"][0]["path"], "src/main.rs");
    assert_eq!(value["files"][0]["counts"]["code"], 2);
    let markdown = fs::read_to_string(temp.path().join("report.md")).unwrap();
    assert!(markdown.contains("## Folder breakdown"));
    assert!(markdown.contains("| src | 2 | 2 |"));
}

#[test]
fn comparison_example_and_scope_refusal_match_documented_evidence() {
    let root = tempfile::tempdir().unwrap();
    write(root.path(), "demo/main.py", "print(1)\n");
    for (file, contents) in [
        ("before.json", "print(1)\n"),
        ("after.json", "print(1)\nprint(2)\n"),
    ] {
        write(root.path(), "demo/main.py", contents);
        let out = run(
            root.path(),
            &["--root", "demo", "--no-user-config", "--output", file],
        );
        assert!(out.status.success());
        assert!(out.stdout.is_empty() && out.stderr.is_empty());
    }
    let result = document(
        root.path(),
        &["compare", "before.json", "after.json", "--format", "json"],
        0,
    );
    let counts = &result["summary"]["source"];
    assert_eq!(
        json!({"before":counts["before"]["code"], "after":counts["after"]["code"], "delta":counts["delta"]["code"]}),
        expected("compare", "comparison-result")
    );
    write(root.path(), "demo/src/app.py", "print(1)\n");
    write(root.path(), "demo/package.json", "{}\n");
    let whole = document(
        root.path(),
        &["--root", "demo", "--no-user-config", "--format", "json"],
        0,
    );
    let subtree = document(
        root.path(),
        &[
            "--root",
            "demo",
            "--subfolder",
            "src",
            "--no-user-config",
            "--format",
            "json",
        ],
        0,
    );
    let independent = document(
        root.path(),
        &["--root", "demo/src", "--no-user-config", "--format", "json"],
        0,
    );
    assert_eq!(whole["summary"]["source_purpose"]["production"]["code"], 1);
    assert_eq!(
        subtree["summary"]["source_purpose"]["production"]["code"],
        1
    );
    assert_eq!(independent["summary"]["source_purpose"]["other"]["code"], 1);
    fs::write(
        root.path().join("subtree.json"),
        serde_json::to_vec(&subtree).unwrap(),
    )
    .unwrap();
    let out = run(
        root.path(),
        &[
            "compare",
            "before.json",
            "subtree.json",
            "--output",
            "refused.json",
        ],
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("subfolder"));
    assert!(!root.path().join("refused.json").exists());
}

#[test]
fn long_help_offers_the_tested_configuration_route_without_changing_short_help() {
    let root = tempfile::tempdir().unwrap();
    let addition = concat!(
        include_str!("../fixtures/agent-explorability/help-routing.txt"),
        include_str!("../fixtures/agent-explorability/help-recipe.txt")
    );
    for args in [vec!["--help"], vec!["help"]] {
        let output = run(root.path(), &args);
        let help = String::from_utf8(output.stdout).unwrap();
        assert_eq!(help.matches(addition).count(), 1);
    }
    for args in [
        vec!["-h"],
        vec!["workspace", "--help"],
        vec!["compare", "--help"],
        vec!["explain", "--help"],
    ] {
        let output = run(root.path(), &args);
        assert!(!String::from_utf8(output.stdout)
            .unwrap()
            .contains("Classify maintained scripts"));
    }
}

#[test]
fn configuration_example_from_help_preserves_conventions_and_unresolved_review() {
    let root = tempfile::tempdir().unwrap();
    let output = run(root.path(), &["--help"]);
    let help = String::from_utf8(output.stdout).unwrap();
    let example = help
        .split_once("  Classify maintained scripts in service/.locmetryrc (TOML):\n")
        .expect("help supplies an executable configuration example")
        .1;
    let configuration = example
        .lines()
        .take_while(|line| line.is_empty() || line.starts_with("    "))
        .map(|line| line.strip_prefix("    ").unwrap_or(line))
        .collect::<Vec<_>>()
        .join("\n");
    write(root.path(), "service/.locmetryrc", &configuration);
    write(
        root.path(),
        "service/scripts/release.py",
        "print('release')\n",
    );
    write(
        root.path(),
        "service/scripts/release_test.py",
        "assert True\n",
    );
    write(root.path(), "service/scripts/build.gradle", "plugins {}\n");
    write(
        root.path(),
        "service/scratch/probe.py",
        "print('undecided')\n",
    );
    let command = example
        .split_once(
            "  Verify the policy with per-file decisions and a human overview:\n    locmetry ",
        )
        .unwrap()
        .1
        .lines()
        .next()
        .unwrap();
    let output = run(root.path(), &command.split_whitespace().collect::<Vec<_>>());
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8(output.stdout).unwrap().contains("Code"));
    let report: Value =
        serde_json::from_str(&fs::read_to_string(root.path().join("report.json")).unwrap())
            .unwrap();
    let files = report["files"].as_array().unwrap();
    for (path, purpose, category) in [
        ("scripts/release.py", "production", "tooling"),
        ("scripts/release_test.py", "test", "test"),
        ("scripts/build.gradle", "production", "build-tooling"),
        ("scratch/probe.py", "other", "unclaimed"),
    ] {
        let file = files.iter().find(|file| file["path"] == path).unwrap();
        assert_eq!(file["purpose"], purpose, "{path}");
        assert_eq!(file["category"], category, "{path}");
    }
    assert_eq!(report["review"].as_array().unwrap().len(), 1);
    assert_eq!(report["review"][0]["path"], "scratch/probe.py");
}
