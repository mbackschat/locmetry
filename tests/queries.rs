mod common;
use std::{fs, path::Path, process::Output};

fn run(root: &Path, args: &[&str]) -> Output {
    common::locmetry()
        .current_dir(root)
        .env("XDG_CONFIG_HOME", root.join("personal"))
        .args(args)
        .output()
        .unwrap()
}

#[test]
fn schema_queries_reject_explicit_unrelated_inputs_in_either_order() {
    rejects_unrelated_inputs("--schema", "config");
}

#[test]
fn guide_queries_reject_explicit_unrelated_inputs_in_either_order() {
    rejects_unrelated_inputs("--guide", "configure");
}

fn rejects_unrelated_inputs(flag: &str, topic: &str) {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join(".locmetryrc"), "invalid configuration").unwrap();
    for extra in [
        vec!["missing"],
        vec!["--root", "missing"],
        vec!["--config", "missing"],
        vec!["--subfolder", "src"],
        vec!["--exclude", "x"],
        vec!["--hidden", "true"],
        vec!["--respect-ignore-files", "false"],
        vec!["--no-user-config"],
        vec!["--sort", "code"],
        vec!["--depth", "2"],
        vec!["--categories"],
        vec!["--verbose"],
        vec!["--files"],
        vec!["--format", "json"],
        vec!["--folder-rows", "3"],
        vec!["--folders"],
        vec!["--output", "saved.json"],
    ] {
        for first in [true, false] {
            let query = [flag, topic];
            let args: Vec<_> = if first {
                query.iter().chain(&extra)
            } else {
                extra.iter().chain(&query)
            }
            .copied()
            .collect();
            let out = run(root.path(), &args);
            assert_eq!(out.status.code(), Some(1), "{args:?}");
            assert!(out.stdout.is_empty(), "{args:?}");
            let error = String::from_utf8_lossy(&out.stderr);
            assert!(error.contains(flag), "{args:?}: {error}");
            assert!(
                error.contains(if extra[0] == "missing" {
                    "DIR"
                } else {
                    extra[0]
                }),
                "{args:?}: {error}"
            );
            assert!(!root.path().join("saved.json").exists());
        }
    }
    for command in [
        vec!["workspace", "--root", "missing"],
        vec!["compare", "before.json", "after.json"],
        vec!["explain", "missing"],
    ] {
        let mut args = vec![flag, topic];
        args.extend(command);
        let out = run(root.path(), &args);
        assert_eq!(out.status.code(), Some(1));
        assert!(out.stdout.is_empty());
    }
}

#[test]
fn schemas_ignore_configuration_and_implicit_defaults_but_keep_help_early_exits() {
    let root = tempfile::tempdir().unwrap();
    fs::write(root.path().join(".locmetryrc"), "invalid configuration").unwrap();
    fs::create_dir_all(root.path().join("personal/locmetry")).unwrap();
    fs::write(root.path().join("personal/locmetry/config.toml"), "invalid").unwrap();
    for schema in [
        "config",
        "user-config",
        "workspace",
        "report",
        "comparison",
        "explanation",
    ] {
        let out = run(root.path(), &["--schema", schema]);
        assert_eq!(out.status.code(), Some(0), "{schema}");
        assert!(out.stderr.is_empty());
        let doc: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
        assert!(doc.get("$schema").is_some());
    }
    for help in ["--help", "-h", "--version"] {
        let out = run(root.path(), &["--schema", "config", "--depth", "2", help]);
        assert_eq!(out.status.code(), Some(0));
        assert!(out.stderr.is_empty());
    }
}
