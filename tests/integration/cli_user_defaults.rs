use crate::cli::*;
use crate::common;
use common::locmetry;
use serde_json::{json, Value};
use std::{fs, path::Path};

/// Runs with user defaults read from `home`, the `XDG_CONFIG_HOME` directory.
fn report_with_user_defaults(cwd: &Path, home: &Path, args: &[&str]) -> (bool, Value, String) {
    let args = [args, &["--output", "-", "--files"]].concat();
    let output = locmetry()
        .current_dir(cwd)
        .env("XDG_CONFIG_HOME", home)
        .args(&args)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let document = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    (output.status.success(), document, stderr)
}

fn user_defaults(temp: &tempfile::TempDir, content: &str) -> std::path::PathBuf {
    let home = temp.path().join("home");
    fs::create_dir_all(home.join("locmetry")).unwrap();
    fs::write(home.join("locmetry/config.toml"), content).unwrap();
    home
}

const USER_DEFAULTS: &str = "version = 1
[scope]
exclude = [\"**/*.generated.ts\"]
[[languages.mappings]]
id = \"tasks\"
match = [\"**/*.runbook\"]
language = \"BASH\"
";

fn languages_by_path(document: &Value) -> Vec<(String, String)> {
    document["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            (
                file["path"].as_str().unwrap().into(),
                file["language"].as_str().unwrap().into(),
            )
        })
        .collect()
}

/// User exclusions sit between the defaults and the project's, and user
/// mappings count files the backend does not recognize.
#[test]
fn user_defaults_add_exclusions_and_language_mappings() {
    let config = format!("{APP_RULE}[scope]\nexclude = [\"vendor/**\"]\n");
    let temp = project(&[
        (".locmetryrc", config.as_str()),
        ("src/a.ts", "let a = 1;\n"),
        ("src/b.generated.ts", "let b = 1;\n"),
        ("vendor/c.ts", "let c = 1;\n"),
        ("scripts/build.runbook", "echo hi\n"),
    ]);
    let home = user_defaults(&temp, USER_DEFAULTS);
    let (success, document, stderr) =
        report_with_user_defaults(&temp.path().join("project"), &home, &[]);
    assert!(success, "{stderr}");
    assert_eq!(
        languages_by_path(&document),
        [
            ("scripts/build.runbook".into(), "BASH".into()),
            ("src/a.ts".into(), "TypeScript".into()),
        ]
    );
    let metadata = &document["metadata"];
    assert_eq!(
        metadata["scope"]["exclude"],
        json!([
            "**/.git/**",
            "**/node_modules/**",
            "**/.build/**",
            "**/.lake/**",
            "**/*.generated.ts",
            "vendor/**",
            ".locmetryrc"
        ])
    );
    assert!(
        std::path::Path::new(metadata["user_config_path"].as_str().unwrap())
            .ends_with("locmetry/config.toml")
    );
}

/// The project knows its files best, so its mappings win over user ones.
#[test]
fn project_language_mappings_take_precedence_over_user_defaults() {
    let config = format!(
        "{APP_RULE}[[languages.mappings]]\nid = \"project-tasks\"\nmatch = [\"**/*.runbook\"]\nlanguage = \"Python\"\n"
    );
    let temp = project(&[
        (".locmetryrc", config.as_str()),
        ("build.runbook", "x = 1\n"),
    ]);
    let home = user_defaults(&temp, USER_DEFAULTS);
    let (success, document, stderr) =
        report_with_user_defaults(&temp.path().join("project"), &home, &[]);
    assert!(success, "{stderr}");
    assert_eq!(
        languages_by_path(&document),
        [("build.runbook".into(), "Python".into())]
    );
}

#[test]
fn no_user_config_ignores_user_defaults() {
    let temp = project(&[("src/b.generated.ts", "let b = 1;\n")]);
    let home = user_defaults(&temp, USER_DEFAULTS);
    let (success, document, stderr) =
        report_with_user_defaults(&temp.path().join("project"), &home, &["--no-user-config"]);
    assert!(success, "{stderr}");
    assert_eq!(document["files"][0]["path"], "src/b.generated.ts");
    assert_eq!(document["metadata"]["user_config_path"], Value::Null);
}

/// User defaults may only narrow scope and teach languages; anything that
/// would classify, widen scope, or collide with project IDs fails the run.
#[test]
fn user_defaults_reject_classification_scope_widening_and_id_clashes() {
    let clash = "version = 1\n[[languages.mappings]]\nid = \"tasks\"\nmatch = [\"*.x\"]\nlanguage = \"BASH\"\n";
    for (user, project_config) in [
        (APP_RULE.to_string(), APP_RULE),
        (
            "version = 1\n[scope]\nhidden = false\n".to_string(),
            APP_RULE,
        ),
        (
            "version = 1\n[classification]\nfallback = \"test\"\n".to_string(),
            APP_RULE,
        ),
        ("version = 2\n".to_string(), APP_RULE),
        (USER_DEFAULTS.to_string(), clash),
    ] {
        let temp = project(&[(".locmetryrc", project_config), ("a.ts", "let a = 1;\n")]);
        let home = user_defaults(&temp, &user);
        let (success, _, stderr) =
            report_with_user_defaults(&temp.path().join("project"), &home, &[]);
        assert!(!success, "accepted user defaults:\n{user}");
        assert!(stderr.contains("config.toml"), "{stderr}");
    }
}
