use serde_json::{json, Value};
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn run(root: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .current_dir(root)
        .env("XDG_CONFIG_HOME", root.join("personal"))
        .args(args)
        .output()
        .unwrap()
}

fn document(root: &Path, args: &[&str], code: i32) -> Value {
    let out = run(root, args);
    assert_eq!(
        out.status.code(),
        Some(code),
        "{args:?}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    serde_json::from_slice(&out.stdout).unwrap()
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

fn write(root: &Path, name: &str, contents: &str) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, contents).unwrap();
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
            help.contains("Run locmetry for source-count tables. Add --verbose to inspect classification, scope and coverage notes. For optional project configuration, use locmetry --guide configure."),
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
            &["--guide", "configure", "--depth", "2", early]
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
        ("src/sample.js", "unclassified", "historical-sample"),
        ("scripts/tool.py", "production", "tooling"),
        ("unknown.py", "unclassified", "fallback"),
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
    assert!(!description.contains("read `summary.unclassified_folders` and add entries"));
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
    assert_eq!(
        independent["summary"]["source_purpose"]["unclassified"]["code"],
        1
    );
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
