use crate::common;
use common::write;
use serde_json::{json, Value};
use std::{fs, path::Path, process::Output};
fn run(root: &Path, args: &[&str]) -> Output {
    common::locmetry()
        .args(["--root", root.to_str().unwrap(), "--no-user-config"])
        .args(args)
        .output()
        .unwrap()
}
fn report(root: &Path, args: &[&str], status: i32) -> Value {
    let out = run(root, &[args, &["--files", "--output", "-"]].concat());
    let value = common::json(&out, status);
    assert!(jsonschema::validator_for(&locmetry::report_schema())
        .unwrap()
        .is_valid(&value));
    serde_json::from_value::<locmetry::contract::MeasurementReport>(value.clone())
        .unwrap()
        .validate()
        .unwrap();
    value
}
fn fixture(root: &Path) {
    write(root, "web/package.json", "{}\n");
    write(root, "web/src/cart.ts", "const cart = 1;\n");
    write(
        root,
        "web/src/components/view.ts",
        "const a = 1;\nconst b = 2;\n",
    );
    write(root, "web/src/components/deep/item.ts", "const item = 1;\n");
    write(root, "web/src/cart.test.ts", "test();\n");
    write(root, "web/src-old/outside.ts", "const outside = 1;\n");
    write(root, "backend/build.gradle", "plugins {}\n");
    write(root, "backend/src/main/X.java", "class X {}\n");
}
#[test]
fn subfolder_preserves_context_and_matches_whole_project_records_and_breakdown() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fixture(root);
    let whole = report(root, &[], 0);
    let part = report(root, &["--subfolder", "web/./src/"], 0);
    let selected: Vec<_> = whole["files"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|f| f["path"].as_str().unwrap().starts_with("web/src/"))
        .cloned()
        .collect();
    assert_eq!(part["files"], json!(selected));
    assert_eq!(part["summary"]["source"]["code"], 5);
    let expected: u64 = whole["breakdown"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|r| {
            let p = r["folder"].as_str().unwrap();
            p == "web/src" || p.starts_with("web/src/")
        })
        .map(|r| r["code"].as_u64().unwrap_or(0))
        .sum();
    assert_eq!(part["summary"]["all_counted"]["code"], expected);
    assert_eq!(part["metadata"]["scope"]["subfolder"], "web/src");
    assert_eq!(part["metadata"]["modules"], whole["metadata"]["modules"]);
    assert_eq!(
        part["summary"]["build_modules"],
        json!({"npm":{"modules":1,"code":4}})
    );
    assert_eq!(part["summary"]["top_folders"]["web"]["production"], 4);
    let mut dot = report(root, &["--subfolder", "."], 0);
    let mut whole = whole;
    for v in [&mut dot, &mut whole] {
        v["metadata"]
            .as_object_mut()
            .unwrap()
            .remove("measured_at_unix_seconds");
    }
    assert_eq!(dot, whole);
}
#[test]
fn subfolder_selection_distinguishes_empty_directories_from_filtered_targets() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fixture(root);
    fs::create_dir(root.join("empty")).unwrap();
    assert_eq!(
        report(root, &["--subfolder", "empty"], 0)["metadata"]["coverage"]["selected_files"],
        0
    );
    write(root, ".ignore", "web/\n");
    let out = run(root, &["--subfolder", "web/src"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("--respect-ignore-files false"));
    report(
        root,
        &["--subfolder", "web/src", "--respect-ignore-files", "false"],
        0,
    );
    fs::remove_file(root.join(".ignore")).unwrap();
    fs::create_dir(root.join(".git")).unwrap();
    write(root, ".gitignore", "web/\n");
    let ignored = run(root, &["--subfolder", "web/src"]);
    assert_eq!(ignored.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&ignored.stderr).contains("--respect-ignore-files false"));
    fs::remove_file(root.join(".gitignore")).unwrap();
    for (args, cause) in [
        (
            vec!["--subfolder", "web/src", "--exclude", "web/**"],
            "web/**",
        ),
        (
            vec!["--subfolder", ".hidden", "--hidden", "false"],
            "--hidden true",
        ),
    ] {
        fs::create_dir_all(root.join(".hidden")).unwrap();
        let out = run(root, &args);
        assert_eq!(out.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&out.stderr).contains(cause));
    }
    write(root, "only_filtered/a.py", "print(1)\n");
    assert_eq!(
        report(
            root,
            &[
                "--subfolder",
                "only_filtered",
                "--exclude",
                "only_filtered/a.py"
            ],
            0
        )["summary"]["all_counted"]["lines"],
        0
    );
}
#[test]
fn subfolder_failure_coverage_and_unused_policy_are_local() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fixture(root);
    write(root, "broken.ipynb", "{}");
    write(root,".locmetryrc","version=1\n[scope]\nexclude=['backend/**']\n[[rules]]\nid='outside'\nmatch=['scripts/**']\npurpose='production'\n");
    write(root, "scripts/tool.py", "print(1)\n");
    let part = report(root, &["--subfolder", "web/src"], 0);
    assert_eq!(part["metadata"]["coverage"]["parse_failures"], json!([]));
    assert_eq!(part["summary"]["unused_entries"], json!(["outside"]));
    assert_eq!(part["summary"]["unused_exclusions"], json!(["backend/**"]));
    for format in ["table", "markdown"] {
        let mut args = vec!["--subfolder", "web/src", "--format", format];
        if format == "table" {
            args.push("--verbose");
        }
        let out = run(root, &args);
        assert!(out.status.success());
        let text = String::from_utf8(out.stdout).unwrap();
        assert!(
            text.contains("check policy use with an unrestricted measurement"),
            "{text}"
        );
        assert!(!text.contains("exclude a folder as"), "{text}");
    }
    write(root, "web/src/broken.ipynb", "{}");
    let part = report(root, &["--subfolder", "web/src"], 2);
    assert_eq!(
        part["metadata"]["coverage"]["parse_failures"][0]["path"],
        "web/src/broken.ipynb"
    );
}
#[test]
fn inactive_config_warning_uses_diagnostic_prefix_and_absolute_paths() {
    let temp = tempfile::tempdir().unwrap();
    let root = dunce::canonicalize(temp.path()).unwrap();
    fixture(&root);
    write(&root, ".locmetryrc", "version=1\n");
    write(&root, "web/src/.locmetryrc", "invalid toml");
    for args in [
        vec!["--subfolder", "web/src"],
        vec!["--subfolder", "web/src", "--config", ".locmetryrc"],
    ] {
        let out = common::locmetry()
            .current_dir(&root)
            .args(["--no-user-config", "--root", "."])
            .args(args)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(String::from_utf8(out.stderr).unwrap(), format!(
            "locmetry: {} is inactive; project configuration: {}. --subfolder retains project policy.\n",
            root.join("web").join("src").join(".locmetryrc").display(), root.join(".locmetryrc").display()
        ));
    }
}

#[test]
fn subfolder_paths_and_subcommands_reject_invalid_restrictions() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fixture(root);
    for path in [
        "",
        "../web",
        "web/../web",
        "/tmp",
        "missing",
        "web/src/cart.ts",
    ] {
        let out = run(root, &["--subfolder", path]);
        assert_eq!(out.status.code(), Some(1), "{path}");
        assert!(out.stdout.is_empty());
    }
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink("web", root.join("alias")).unwrap();
        assert_eq!(
            run(root, &["--subfolder", "alias/src"]).status.code(),
            Some(1)
        );
    }
    for cmd in ["workspace", "compare", "explain"] {
        let out = common::locmetry()
            .args([cmd, "--subfolder", "web/src"])
            .output()
            .unwrap();
        assert!(!out.status.success());
        let stderr = String::from_utf8_lossy(&out.stderr);
        assert!(
            stderr.contains("unexpected argument '--subfolder'"),
            "{stderr}"
        );
    }
    assert_eq!(
        run(root, &["--subfolder", "web", "--subfolder", "web/src"])
            .status
            .code(),
        Some(1)
    );
}
#[test]
fn subfolder_human_views_start_at_children_and_keep_project_paths() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path();
    fixture(root);
    for format in ["table", "markdown"] {
        let mut args = vec!["--subfolder", "web/src", "--format", format];
        if format == "table" {
            args.push("--show-folders");
        }
        let out = run(root, &args);
        assert!(out.status.success());
        let text = String::from_utf8(out.stdout).unwrap();
        assert!(text.contains("subfolder web/src"), "{text}");
        assert!(text.contains("web/src/components"), "{text}");
        assert!(text.contains("web/src (direct files)"), "{text}");
        if format == "markdown" {
            assert!(text.contains("web/src/components/deep"), "{text}");
        }
    }
}
#[test]
fn subfolder_compare_requires_matching_scope_and_preserves_descriptor_removal_oracle() {
    let dir = tempfile::tempdir().unwrap();
    let root = dir.path().join("project");
    fixture(&root);
    let before = report(&root, &["--subfolder", "web/src"], 0);
    let whole = report(&root, &[], 0);
    fs::remove_file(root.join("web/package.json")).unwrap();
    let after = report(&root, &["--subfolder", "web/src"], 0);
    let decode =
        |v: Value| serde_json::from_value::<locmetry::contract::MeasurementReport>(v).unwrap();
    let before = decode(before);
    let after = decode(after);
    let whole = decode(whole);
    assert!(
        locmetry::compare::compare(whole, after.clone(), "whole".into(), "part".into())
            .err()
            .unwrap()
            .to_string()
            .contains("before: unrestricted, after: web/src")
    );
    let change =
        locmetry::compare::compare(before, after, "before".into(), "after".into()).unwrap();
    let value = serde_json::to_value(change).unwrap();
    assert_eq!(value["summary"]["source"]["delta"]["code"], 0);
    assert_eq!(value["summary"]["all_counted"]["delta"]["lines"], 0);
    assert_eq!(
        value["summary"]["source_purpose"]["production"]["delta"]["code"],
        -4
    );
}

#[test]
fn subfolder_receipts_reject_malformed_scope_and_outside_measurement_evidence() {
    let temp = tempfile::tempdir().unwrap();
    fixture(temp.path());
    let base = report(temp.path(), &["--subfolder", "web/src"], 0);
    let valid = |v: Value| {
        serde_json::from_value::<locmetry::contract::MeasurementReport>(v)
            .is_ok_and(|r| r.validate().is_ok())
    };
    for bad in [
        json!(null),
        json!(""),
        json!("."),
        json!("web/"),
        json!("web//src"),
        json!("/web"),
        json!("web/../src"),
        json!("web/./src"),
    ] {
        let mut value = base.clone();
        value["metadata"]["scope"]["subfolder"] = bad.clone();
        assert!(!valid(value), "accepted malformed restriction {bad}");
    }
    for path in ["backend/x.ts", "web/src-old/x.ts", "web/src"] {
        let mut value = base.clone();
        value["files"][0]["path"] = json!(path);
        assert!(!valid(value), "{path}");
    }
    let mut outside = base.clone();
    outside["review"] = json!([{"path":"elsewhere.py","language":"Python","category":"unclaimed","rule":"fallback","code":0,"lines":0}]);
    assert!(!valid(outside));
    let mut outside = base.clone();
    outside["summary"]["unclaimed_folders"] = json!([{"folder":"web","files":0,"code":0}]);
    assert!(!valid(outside));
    let mut modules = base.clone();
    modules["summary"]["build_modules"]["gradle"] = json!({"modules":1,"code":0});
    assert!(!valid(modules));
    assert!(valid(base));
}

#[test]
fn subfolder_preserves_nested_and_python_context_and_does_not_read_inactive_config() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(root, "pyproject.toml", "[project]\nname='demo'\n");
    write(root, "pkg/__init__.py", "");
    write(root, "pkg/inner/a.py", "print(1)\n");
    write(root, "pkg/inner/.locmetryrc", "invalid toml");
    let out = run(root, &["--subfolder", "pkg/inner", "--output", "-"]);
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stderr).contains("is inactive"));
    let part = report(root, &["--subfolder", "pkg/inner"], 0);
    assert_eq!(part["files"][0]["rule"], "builtin:python-module");
    write(root, "pkg/inner/package.json", "{}\n");
    let part = report(root, &["--subfolder", "pkg/inner"], 0);
    let file = part["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["path"] == "pkg/inner/a.py")
        .unwrap();
    assert_eq!(file["rule"], "fallback");
    let part = report(
        root,
        &[
            "--subfolder",
            "pkg/inner",
            "--exclude",
            "pkg/inner/package.json",
        ],
        0,
    );
    assert_eq!(part["files"][0]["rule"], "builtin:python-module");
}

#[test]
fn subfolder_comparison_refusal_protects_output_and_same_scope_preserves_receipt() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    fixture(&root);
    let a = report(&root, &["--subfolder", "web/src"], 0);
    let b = report(&root, &["--subfolder", "backend/src"], 0);
    let before = temp.path().join("before.json");
    let after = temp.path().join("after.json");
    let dest = temp.path().join("change.json");
    fs::write(&before, serde_json::to_vec(&a).unwrap()).unwrap();
    fs::write(&after, serde_json::to_vec(&b).unwrap()).unwrap();
    fs::write(&dest, "keep me").unwrap();
    let compare = |format: &str| {
        common::locmetry()
            .args([
                "compare",
                before.to_str().unwrap(),
                after.to_str().unwrap(),
                "--format",
                format,
                "--output",
                dest.to_str().unwrap(),
            ])
            .output()
            .unwrap()
    };
    let out = compare("json");
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(String::from_utf8_lossy(&out.stderr).contains("before: web/src, after: backend/src"));
    assert_eq!(fs::read_to_string(&dest).unwrap(), "keep me");
    fs::write(&after, serde_json::to_vec(&a).unwrap()).unwrap();
    for format in ["json", "table", "markdown"] {
        let out = compare(format);
        assert!(
            out.status.success(),
            "{}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert!(out.stdout.is_empty());
        let saved = fs::read(&dest).unwrap();
        if format == "json" {
            let v: Value = serde_json::from_slice(&saved).unwrap();
            assert_eq!(v["before"]["metadata"]["scope"]["subfolder"], "web/src");
            assert!(jsonschema::validator_for(&locmetry::compare::schema())
                .unwrap()
                .is_valid(&v));
        } else {
            assert!(String::from_utf8_lossy(&saved).contains("web/src"));
        }
    }
    assert_eq!(
        run(
            &root,
            &[
                "--subfolder",
                "web/src",
                "--output",
                root.join("elsewhere.json").to_str().unwrap()
            ]
        )
        .status
        .code(),
        Some(1)
    );
}

#[test]
fn subfolder_zero_code_modules_aliases_literal_names_and_explanation_are_consistent() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(root, "package.json", "{}\n");
    write(root, "src/[literal]/empty.ts", "\n");
    let v = report(root, &["--subfolder", "src/[literal]"], 0);
    assert_eq!(
        v["summary"]["build_modules"],
        json!({"npm":{"modules":1,"code":0}})
    );
    let out = common::locmetry()
        .args([
            "explain",
            "src/[literal]/empty.ts",
            "--root",
            root.to_str().unwrap(),
            "--no-user-config",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    let explanation: Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(explanation["result"]["file"], v["files"][0]);
    if root.join("SRC").exists() {
        let out = run(root, &["--subfolder", "SRC/[literal]"]);
        assert_eq!(out.status.code(), Some(1));
        assert!(String::from_utf8_lossy(&out.stderr).contains("exact spelling"));
    }
    write(root, ".hidden/a.py", "print(1)\n");
    write(root, ".ignore", ".hidden/\n");
    let out = run(root, &["--subfolder", ".hidden", "--hidden", "false"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stderr).contains("cause could not be established"));
    write(root, "src/[literal]/.locmetryrc", "version=1\n");
    let out = run(
        root,
        &[
            "--subfolder",
            "src/[literal]",
            "--config",
            root.join("src/[literal]/.locmetryrc").to_str().unwrap(),
        ],
    );
    assert!(out.status.success());
    assert!(!String::from_utf8_lossy(&out.stderr).contains("is inactive"));
}

#[test]
fn workspace_report_cannot_mix_project_folders_with_roots() {
    let temp = tempfile::tempdir().unwrap();
    let (web, api) = (temp.path().join("web"), temp.path().join("api"));
    fixture(&web);
    fixture(&api);
    let out = common::locmetry()
        .args(["workspace", "--root", web.to_str().unwrap()])
        .args(["--root", api.to_str().unwrap(), "--no-user-config"])
        .args(["--output", "-"])
        .output()
        .unwrap();
    assert!(out.status.success());
    let mut v: Value = serde_json::from_slice(&out.stdout).unwrap();
    v["metadata"]["members"][0]["scope"]["subfolder"] = json!("web/src");
    let report: locmetry::contract::MeasurementReport = serde_json::from_value(v).unwrap();
    assert!(report
        .validate()
        .unwrap_err()
        .to_string()
        .contains("mix project folders"));
}

#[cfg(unix)]
#[test]
fn subtree_unread_files_are_partial_but_unrelated_traversal_failure_aborts() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fixture(root);
    let inaccessible = root.join("web/src/cart.ts");
    fs::set_permissions(&inaccessible, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read(&inaccessible).is_err() {
        let value = report(root, &["--subfolder", "web/src"], 2);
        assert_eq!(
            value["metadata"]["coverage"]["unread"],
            json!(["web/src/cart.ts"])
        );
    }
    fs::set_permissions(&inaccessible, fs::Permissions::from_mode(0o600)).unwrap();
    let outside = root.join("backend");
    fs::set_permissions(&outside, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read_dir(&outside).is_err() {
        let out = run(root, &["--subfolder", "web/src"]);
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stdout.is_empty());
    }
    fs::set_permissions(&outside, fs::Permissions::from_mode(0o700)).unwrap();
}

#[test]
fn subfolder_reports_allow_ancestor_submodules_but_reject_outside_breakdown_and_coverage() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(
        root,
        ".gitmodules",
        "[submodule \"lib\"]\npath = vendor/lib\n",
    );
    write(root, "vendor/lib/src/a.py", "print(1)\n");
    write(root, "vendor/lib/outside.py", "print(2)\n");
    write(root, "vendor/lib/src/unknown.bin", "unknown");
    let base = report(root, &["--subfolder", "vendor/lib/src"], 0);
    assert_eq!(
        base["summary"]["submodules"],
        json!([{"path":"vendor/lib","files":1,"code":1}])
    );
    for pointer in [
        "/metadata/coverage/unrecognized/0",
        "/breakdown/0/folder",
        "/summary/submodules/0/path",
    ] {
        let mut value = base.clone();
        *value.pointer_mut(pointer).unwrap() = json!("elsewhere");
        let record: locmetry::contract::MeasurementReport = serde_json::from_value(value).unwrap();
        assert!(record.validate().is_err(), "{pointer}");
    }
}

#[cfg(unix)]
#[test]
fn subfolder_human_labels_flatten_controls_while_json_preserves_path_identity() {
    let temp = tempfile::tempdir().unwrap();
    write(temp.path(), "src/odd\nname/a.py", "print(1)\n");
    let value = report(temp.path(), &["--subfolder", "src/odd\nname"], 0);
    assert_eq!(value["metadata"]["scope"]["subfolder"], "src/odd\nname");
    let out = run(
        temp.path(),
        &["--subfolder", "src/odd\nname", "--format", "table"],
    );
    let text = String::from_utf8(out.stdout).unwrap();
    let root = text
        .lines()
        .find(|line| line.starts_with("Root: "))
        .unwrap();
    assert!(root.ends_with(", subfolder src/odd name"), "{text}");
}

#[test]
fn module_contributions_follow_deciding_rules_and_deepest_matching_directories() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    write(root, "package.json", "{}\n");
    write(root, "src/top.ts", "const top = 1;\n");
    write(root, "src/plugin/package.json", "{}\n");
    write(
        root,
        "src/plugin/src/view.ts",
        "const a = 1;\nconst b = 2;\n",
    );
    write(root, "src/plugin/build.gradle", "plugins {}\n");
    write(
        root,
        "src/plugin/src/main/kotlin/App.kt",
        "class App {\nval x = 1\n}\n",
    );
    write(
        root,
        "src/plugin-old/file.ts",
        "const a = 1;\nconst b = 2;\nconst c = 3;\nconst d = 4;\n",
    );
    write(root, "src/empty/package.json", "{}\n");
    write(root, "src/empty/src/zero.ts", "");
    write(root, "src/manual.ts", "const manual = 1;\n");
    write(
        root,
        ".locmetryrc",
        "version=1\n[[overrides]]\nid='manual'\nmatch=['src/manual.ts']\npurpose='production'\n",
    );
    let value = report(root, &["--subfolder", "src"], 0);
    assert_eq!(
        value["summary"]["build_modules"],
        json!({
            "npm": {"modules": 3, "code": 7},
            "gradle": {"modules": 1, "code": 3},
        })
    );
    assert_eq!(value["summary"]["source_purpose"]["production"]["code"], 12);
    let mut forged = value;
    forged["summary"]["build_modules"]["npm"]["code"] = json!(6);
    forged["summary"]["build_modules"]["gradle"]["code"] = json!(4);
    let error = serde_json::from_value::<locmetry::contract::MeasurementReport>(forged)
        .unwrap()
        .validate()
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains("build_modules conflicts with measured file contributions"),
        "{error}"
    );
}
