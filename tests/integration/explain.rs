use crate::common;
use common::write;
use serde_json::{json, Value};
use std::{fs, path::Path, process::Output};
fn run(root: &Path, path: &str, args: &[&str]) -> Output {
    common::locmetry()
        .args([
            "explain",
            path,
            "--root",
            root.to_str().unwrap(),
            "--no-user-config",
        ])
        .args(args)
        .output()
        .unwrap()
}
fn document(output: Output, status: i32) -> Value {
    let value = common::json(&output, status);
    if value["document_type"] == "explanation" {
        assert_valid(&value);
    }
    value
}
fn assert_valid(explanation: &Value) {
    let schema = locmetry::explain::schema();
    let validator = jsonschema::validator_for(&schema).unwrap();
    assert!(
        validator.is_valid(explanation),
        "{:?}",
        validator.iter_errors(explanation).collect::<Vec<_>>()
    );
}
#[test]
fn missing_targets_report_absence_instead_of_a_spelling_alias() {
    let temp = tempfile::tempdir().unwrap();
    write(temp.path(), "src/main.rs", "fn main() {}\n");
    for path in ["src/missing.rs", "missing/x.py"] {
        let output = run(temp.path(), path, &[]);
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8(output.stderr).unwrap();
        // With several targets, only the named path tells which one is missing.
        assert!(
            error.contains(&format!(
                "Explanation target '{path}' does not exist or cannot be inspected"
            )),
            "{error}"
        );
        assert!(!error.contains("exact spelling"), "{error}");
    }
    let output = common::locmetry()
        .current_dir(temp.path())
        .args(["--subfolder", "nope", "--format", "table"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    let error = String::from_utf8(output.stderr).unwrap();
    assert!(
        error.contains("Subfolder target 'nope' does not exist or cannot be inspected"),
        "{error}"
    );
}
#[test]
fn explains_first_match_module_inference_and_mapping_with_actual_file_counts() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(root, "package.json", "{}\n");
    write(root, "src/main.js", "let a = 1;\n");
    write(root, "src/example.test.js", "assert(true);\n");
    write(root, "ops/task.custom", "echo ok\n");
    write(root,".locmetryrc","version = 1\n[[overrides]]\nid = \"archive\"\nmatch = [\"src/example.test.js\"]\npurpose = \"other\"\nreason = \"Historical example\"\n[[rules]]\nid = \"first\"\nmatch = [\"ops/**\"]\npurpose = \"production\"\n[[rules]]\nid = \"second\"\nmatch = [\"ops/**\"]\npurpose = \"test\"\n[[languages.mappings]]\nid = \"custom\"\nmatch = [\"**/*.custom\"]\nlanguage = \"BASH\"\n");
    let module = document(run(root, "src/main.js", &[]), 0);
    assert_eq!(module["document_type"], "explanation");
    assert_eq!(module["result"]["state"], "counted");
    assert_eq!(module["result"]["decision"]["stage"], "module");
    assert_eq!(module["result"]["decision"]["module"]["detector"], "npm");
    assert_eq!(module["result"]["file"]["counts"]["code"], 1);
    let override_ = document(run(root, "src/example.test.js", &[]), 0);
    assert_eq!(override_["result"]["file"]["rule"], "archive");
    assert_eq!(override_["result"]["decision"]["stage"], "override");
    assert_eq!(
        override_["result"]["decision"]["reason"],
        "Historical example"
    );
    let mapping = document(run(root, "ops/task.custom", &[]), 0);
    assert_eq!(mapping["result"]["file"]["rule"], "first");
    assert_eq!(mapping["result"]["file"]["language_mapping"], "custom");
    assert_eq!(
        mapping["result"]["decision"]["matched_patterns"],
        json!(["ops/**"])
    );
    let schema = common::locmetry()
        .args(["--schema", "explanation"])
        .output()
        .unwrap();
    let schema = document(schema, 0);
    let validator = jsonschema::validator_for(&schema).unwrap();
    for result in [&module, &override_, &mapping] {
        assert!(
            validator.is_valid(result),
            "{:?}",
            validator.iter_errors(result).collect::<Vec<_>>()
        );
    }
}
#[test]
fn explains_pruned_exclusions_ignore_and_hidden_filters_without_claiming_an_ignore_rule() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        ".locmetryrc",
        "version = 1\n[scope]\nexclude = [\"vendor/**\"]\n",
    );
    write(root, "vendor/lib.py", "print(1)\n");
    write(root, ".ignore", "ignored.py\n");
    write(root, "ignored.py", "print(1)\n");
    write(root, ".hidden.py", "print(1)\n");
    let excluded = document(run(root, "vendor/lib.py", &[]), 0);
    assert_eq!(excluded["result"]["state"], "excluded");
    assert_eq!(excluded["result"]["at"], "vendor");
    assert_eq!(excluded["result"]["patterns"], json!(["vendor/**"]));
    let ignored = document(run(root, "ignored.py", &[]), 0);
    assert_eq!(ignored["result"]["state"], "not_selected");
    assert_eq!(ignored["result"]["reason"], "ignore_policy");
    let enabled = document(
        run(root, "ignored.py", &["--respect-ignore-files", "false"]),
        0,
    );
    assert_eq!(enabled["result"]["state"], "counted");
    let hidden = document(run(root, ".hidden.py", &["--hidden", "false"]), 0);
    assert_eq!(hidden["result"]["reason"], "hidden_policy");
}
#[test]
fn explains_unrecognized_and_parse_failed_targets_without_counting_unrelated_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(root, "unknown.zzzz", "unknown\n");
    write(root, "broken.ipynb", "{}");
    fs::write(root.join("notice.hbs"), b"caf\xe9\n").unwrap();
    write(root, "main.py", "print(1)\n");
    let unknown = document(run(root, "unknown.zzzz", &[]), 0);
    assert_eq!(unknown["result"]["state"], "unrecognized");
    let failed = document(run(root, "broken.ipynb", &[]), 2);
    assert_eq!(failed["result"]["state"], "parse_failed");
    assert_eq!(failed["result"]["failure"]["reason"], "invalid_notebook");
    let failed = document(run(root, "notice.hbs", &[]), 2);
    assert_eq!(failed["result"]["state"], "parse_failed");
    assert_eq!(failed["result"]["failure"]["reason"], "invalid_encoding");
    let counted = document(run(root, "main.py", &[]), 0);
    assert_eq!(counted["result"]["state"], "counted");
    assert_eq!(counted["result"]["decision"]["stage"], "fallback");
    for format in ["table", "markdown"] {
        let output = run(root, "broken.ipynb", &["--format", format]);
        assert_eq!(output.status.code(), Some(2));
        assert!(String::from_utf8_lossy(&output.stdout).contains("invalid_notebook"));
    }
}
#[test]
fn workspace_queries_use_member_policy_and_qualified_paths() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "api/.locmetryrc",
        "version = 1\n[classification]\nfallback = \"production\"\n",
    );
    write(root, "api/main.py", "print(1)\n");
    write(root, "web/main.py", "print(1)\n");
    write(
        root,
        "repos.toml",
        "version = 1\nroots = [\"api\",\"web\"]\n",
    );
    let run = |path: &str, args: &[&str]| {
        common::locmetry()
            .args([
                "explain",
                path,
                "--workspace-file",
                root.join("repos.toml").to_str().unwrap(),
                "--no-user-config",
            ])
            .args(args)
            .output()
            .unwrap()
    };
    let api = document(run("api/main.py", &[]), 0);
    assert_eq!(api["member"], "api");
    assert_eq!(api["path"], "api/main.py");
    assert_eq!(api["result"]["file"]["path"], "api/main.py");
    assert_eq!(api["result"]["file"]["purpose"], "production");
    let web = document(run("web/main.py", &[]), 0);
    assert_eq!(web["result"]["file"]["purpose"], "other");
    assert_eq!(
        run("api/main.py", &["--hidden", "false"]).status.code(),
        Some(1)
    );
    let unknown = run("missing/main.py", &[]);
    assert_eq!(unknown.status.code(), Some(1));
    assert!(String::from_utf8(unknown.stderr)
        .unwrap()
        .contains("Unknown workspace member 'missing'"));
    // Errors name the MEMBER/PATH the user typed, not only the member-relative path.
    for path in ["api/missing.py", "api/missing/main.py"] {
        let output = run(path, &[]);
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8(output.stderr).unwrap();
        assert!(
            error.contains(&format!("Cannot explain '{path}'")),
            "{error}"
        );
        assert!(
            error.contains("does not exist or cannot be inspected"),
            "{error}"
        );
    }
}

#[test]
fn counted_explanations_equal_measurement_records_including_embedded_languages() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    for (name, text) in [
        ("package.json", "{}\n"),
        ("src/main.js", "let a = 1;\n"),
        ("src/main.test.js", "assert(true);\n"),
        ("src/page.html", "<script>\nlet b = 2;\n</script>\n"),
        ("src/vendor/package.json", "{}\n"),
        ("src/vendor/main.js", "let c = 3;\n"),
    ] {
        write(root, name, text);
    }
    let report = document(
        common::locmetry()
            .args([
                "--root",
                root.to_str().unwrap(),
                "--no-user-config",
                "--files",
                "--output",
                "-",
            ])
            .output()
            .unwrap(),
        0,
    );
    for file in report["files"].as_array().unwrap() {
        let explanation = document(run(root, file["path"].as_str().unwrap(), &[]), 0);
        assert_eq!(&explanation["result"]["file"], file);
    }
    let convention = document(run(root, "src/main.test.js", &[]), 0);
    assert_eq!(convention["result"]["decision"]["stage"], "convention");
    assert_eq!(
        convention["result"]["decision"]["matched_patterns"],
        json!(["**/*.{test,spec}.{js,jsx,mjs,cjs,ts,tsx,mts,cts}"])
    );
    let disabled = document(run(root, "src/main.js", &["--exclude", "package.json"]), 0);
    assert_eq!(disabled["result"]["decision"]["stage"], "fallback");
}

#[test]
fn personal_defaults_explicit_config_and_path_bases_match_measurement() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    let defaults = temp.path().join("defaults");
    write(&root, "main.py", "print(1)\n");
    write(&root, "task.custom", "echo ok\n");
    write(&defaults,"locmetry/config.toml","version = 1\n[scope]\nexclude = [\"main.py\"]\n[[languages.mappings]]\nid = \"custom\"\nmatch = [\"*.custom\"]\nlanguage = \"BASH\"\n");
    write(&root, ".locmetryrc", "invalid config");
    write(temp.path(), "replacement.toml", "version = 1\n");
    let command = |target: &str, args: &[&str]| {
        common::locmetry()
            .current_dir(temp.path())
            .env("XDG_CONFIG_HOME", &defaults)
            .args([
                "explain",
                target,
                "--root",
                "project",
                "--config",
                "replacement.toml",
            ])
            .args(args)
            .output()
            .unwrap()
    };
    let excluded = document(command("main.py", &[]), 0);
    assert_eq!(excluded["result"]["state"], "excluded");
    assert!(
        std::path::Path::new(excluded["user_config_path"].as_str().unwrap())
            .ends_with("locmetry/config.toml")
    );
    let enabled = document(command("./main.py", &["--no-user-config"]), 0);
    assert_eq!(enabled["path"], "main.py");
    assert_eq!(enabled["result"]["state"], "counted");
    assert!(enabled["user_config_path"].is_null());
    let mapped = document(command("task.custom", &[]), 0);
    assert_eq!(mapped["result"]["file"]["language_mapping"], "custom");
    let unmapped = document(command("task.custom", &["--no-user-config"]), 0);
    assert_eq!(unmapped["result"]["state"], "unrecognized");
    for target in [
        "missing.py",
        ".",
        "../replacement.toml",
        root.join("main.py").to_str().unwrap(),
    ] {
        assert_eq!(command(target, &[]).status.code(), Some(1), "{target}");
    }
}

#[test]
fn overlapping_filters_do_not_invent_a_winning_cause() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(root, ".secret/main.py", "print(1)\n");
    write(root, ".ignore", ".secret/\n");
    let result = document(run(root, ".secret/main.py", &["--hidden", "false"]), 0);
    assert_eq!(
        result["result"],
        json!({"state":"not_selected","reason":"reason_not_recorded"})
    );
    let selected = document(
        run(
            root,
            ".secret/main.py",
            &["--hidden", "true", "--respect-ignore-files", "false"],
        ),
        0,
    );
    assert_eq!(selected["result"]["state"], "counted");
}

#[cfg(unix)]
#[test]
fn symlinks_stay_untraversed_and_unread_counts_stay_unavailable() {
    use std::os::unix::fs::{symlink, PermissionsExt};
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("root");
    write(&root, "main.py", "print(1)\n");
    write(temp.path(), "outside/main.py", "print(2)\n");
    symlink(temp.path().join("outside"), root.join("linked")).unwrap();
    symlink("missing.py", root.join("dangling.py")).unwrap();
    for (target, at) in [("linked/main.py", "linked"), ("dangling.py", "dangling.py")] {
        let explanation = document(run(&root, target, &[]), 0);
        assert_eq!(explanation["result"], json!({"state":"symlink","at":at}));
    }
    fs::set_permissions(root.join("main.py"), fs::Permissions::from_mode(0o000)).unwrap();
    let unread = run(&root, "main.py", &[]);
    let actually_unreadable = fs::read(root.join("main.py")).is_err();
    fs::set_permissions(root.join("main.py"), fs::Permissions::from_mode(0o600)).unwrap();
    if actually_unreadable {
        assert_eq!(document(unread, 2)["result"]["state"], "unread");
    }
}

#[test]
fn schema_examples_are_valid_and_workspace_only_loads_the_queried_policy() {
    let schema = document(
        common::locmetry()
            .args(["--schema", "explanation"])
            .output()
            .unwrap(),
        0,
    );
    let validator = jsonschema::validator_for(&schema).unwrap();
    let examples = schema["examples"]
        .as_array()
        .expect("schema should demonstrate a complete explanation");
    assert!(!examples.is_empty());
    for example in examples {
        assert!(validator.is_valid(example));
    }
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(root, "api/package.json", "{}\n");
    write(root, "api/src/main.js", "let a = 1;\n");
    write(root, "web/.locmetryrc", "invalid");
    write(
        root,
        "repos.toml",
        "version = 1\nroots = [\"api\",\"web\"]\n",
    );
    let result = document(
        common::locmetry()
            .args([
                "explain",
                "api/src/main.js",
                "--workspace-file",
                root.join("repos.toml").to_str().unwrap(),
                "--no-user-config",
            ])
            .output()
            .unwrap(),
        0,
    );
    assert_eq!(result["result"]["decision"]["module"]["path"], "api");
    assert!(validator.is_valid(&result));
}

#[test]
fn troubleshooting_example_matches_the_executable() {
    let guide = include_str!("../../docs/guides/selection.md");
    let example = guide
        .split_once("<!-- explanation-result -->")
        .unwrap()
        .1
        .split_once("```json\n")
        .unwrap()
        .1
        .split_once("```")
        .unwrap()
        .0;
    let expected: Value = serde_json::from_str(example).unwrap();
    let temp = tempfile::tempdir().unwrap();
    write(temp.path(), "main.py", "print(1)\n");
    assert_eq!(
        document(run(temp.path(), "main.py", &[]), 0)["result"],
        expected
    );
}

#[test]
fn aliases_and_directory_queries_fail_instead_of_claiming_filtering() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(root, "src/main.rs", "fn main() {}\n");
    write(root, "cafe\u{301}.py", "print(1)\n");
    for alias in ["SRC/MAIN.RS", "café.py"] {
        let output = run(root, alias, &[]);
        assert_eq!(
            output.status.code(),
            Some(1),
            "{alias}: {}",
            String::from_utf8_lossy(&output.stdout)
        );
        if root.join(alias).exists() {
            assert!(String::from_utf8_lossy(&output.stderr).contains("exact spelling"));
        }
    }
    for directory in [".", "./", "src"] {
        let output = run(root, directory, &[]);
        assert_eq!(output.status.code(), Some(1));
        let error = String::from_utf8_lossy(&output.stderr);
        assert!(
            error.contains(&format!("Explanation target '{directory}' must be a file")),
            "{error}"
        );
    }
    write(root, "repos.toml", "version = 1\nroots = [\"src\"]\n");
    let output = common::locmetry()
        .args([
            "explain",
            "src",
            "--workspace-file",
            root.join("repos.toml").to_str().unwrap(),
            "--no-user-config",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("' must be a file"));
}

#[test]
fn command_help_describes_formats_without_promising_measurement_headlines() {
    for command in ["explain", "compare"] {
        let output = common::locmetry()
            .args([command, "--help"])
            .output()
            .unwrap();
        assert!(output.status.success());
        let help = String::from_utf8(output.stdout).unwrap();
        assert!(!help.contains("compact headline"));
        assert!(!help.contains("Source code per language and folder"));
        assert!(help.contains("JSON"));
    }
}

/// Several targets print one unchanged explanation document per line, in
/// argument order, so an agent checks representative files in one call and
/// line-oriented tools and `jq` read the stream alike.
#[test]
fn several_paths_print_one_explanation_per_line_in_argument_order() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(root, "main.py", "print(1)\n");
    write(root, "tests/test_main.py", "assert True\n");
    write(root, "broken.ipynb", "{}");
    let lines = |output: Output, status: i32| -> Vec<Value> {
        assert_eq!(
            output.status.code(),
            Some(status),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        String::from_utf8(output.stdout)
            .unwrap()
            .lines()
            .map(|line| {
                let explanation = serde_json::from_str(line).unwrap();
                assert_valid(&explanation);
                explanation
            })
            .collect()
    };
    let run = |paths: &[&str], args: &[&str]| {
        common::locmetry()
            .arg("explain")
            .args(paths)
            .args(["--root", root.to_str().unwrap(), "--no-user-config"])
            .args(args)
            .output()
            .unwrap()
    };
    let explained = lines(
        run(&["tests/test_main.py", "main.py"], &["--format", "json"]),
        0,
    );
    assert!(explained.iter().all(|e| e["schema_version"] == "3"));
    let summary: Vec<_> = explained
        .iter()
        .map(|e| (e["path"].clone(), e["result"]["file"]["purpose"].clone()))
        .collect();
    assert_eq!(
        summary,
        [
            (json!("tests/test_main.py"), json!("test")),
            (json!("main.py"), json!("other"))
        ]
    );
    // A partial target makes the command partial without hiding the others.
    let explained = lines(run(&["main.py", "broken.ipynb"], &["--format", "json"]), 2);
    assert_eq!(explained[1]["result"]["state"], "parse_failed");
    // Every target is inspected before output, so a failing one prints nothing.
    let output = run(&["main.py", "missing.py"], &["--format", "json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    for format in ["table", "markdown"] {
        let text = String::from_utf8(
            run(&["main.py", "tests/test_main.py"], &["--format", format]).stdout,
        )
        .unwrap();
        assert!(
            text.find("tests/test_main.py") > text.find("main.py"),
            "{text}"
        );
    }
}

#[test]
fn several_workspace_paths_use_each_members_policy() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        "api/.locmetryrc",
        "version = 1\n[classification]\nfallback = \"production\"\n",
    );
    write(root, "api/main.py", "print(1)\n");
    write(root, "web/main.py", "print(1)\n");
    write(
        root,
        "repos.toml",
        "version = 1\nroots = [\"api\",\"web\"]\n",
    );
    let output = common::locmetry()
        .args(["explain", "web/main.py", "api/main.py", "--workspace-file"])
        .arg(root.join("repos.toml"))
        .args(["--no-user-config", "--format", "json"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let purposes: Vec<(String, String)> = String::from_utf8(output.stdout)
        .unwrap()
        .lines()
        .map(|line| {
            let value: Value = serde_json::from_str(line).unwrap();
            (
                value["member"].as_str().unwrap().into(),
                value["result"]["file"]["purpose"].as_str().unwrap().into(),
            )
        })
        .collect();
    assert_eq!(
        purposes,
        [
            ("web".into(), "other".into()),
            ("api".into(), "production".into())
        ]
    );
}

#[test]
fn table_counts_name_each_figure_with_its_number_and_unit() {
    let temp = tempfile::tempdir().unwrap();
    write(
        temp.path(),
        "Cargo.toml",
        "[package]\nname = \"d\"\nversion = \"0.1.0\"\n",
    );
    write(temp.path(), "src/lib.rs", "// a\n\nfn a() {}\n\n");
    write(temp.path(), "tool.py", "print(1)\n");
    for (path, counts, module) in [
        (
            "tool.py",
            "Counts: 1 code, 0 comments, 0 blanks, 1 line\n",
            None,
        ),
        (
            "src/lib.rs",
            "Counts: 1 code, 1 comment, 2 blanks, 4 lines\n",
            Some("Module: scan root (cargo)\n"),
        ),
    ] {
        let text = common::stdout(run(temp.path(), path, &["--format", "table"]));
        assert!(text.contains(counts), "{text}");
        if let Some(module) = module {
            assert!(text.contains(module), "{text}");
        }
    }
}
