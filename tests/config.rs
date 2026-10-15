use locmetry::{file_records, scan, Kind, Policy, ScopeOverrides};
use std::fs;

fn policy(toml: &str) -> Policy {
    Policy::from_toml(toml).unwrap()
}

fn rule(policy: &Policy, path: &str, kind: Kind) -> String {
    policy.classify(path, kind).rule
}

#[test]
fn invalid_configurations_fail_with_specific_messages() {
    for (toml, message) in [
        ("", "version"),
        ("version = 2", "Unsupported configuration version 2"),
        ("version = 1\nbackend = \"scc\"", "unknown variant"),
        ("version = 1\nunknown = true", "unknown field"),
        ("version = 1\n[scope]\nhiden = true", "unknown field"),
        ("version = 1\n[classification]\nfallback = \"maybe\"", "unknown variant"),
        (
            "version = 1\n[[overrides]]\nid = \"a\"\nmatch = [\"x/**\"]\npurpose = \"test\"\n[[rules]]\nid = \"a\"\nmatch = [\"y/**\"]\npurpose = \"test\"",
            "unique",
        ),
        ("version = 1\n[[rules]]\nid = \"fallback\"\nmatch = [\"**\"]\npurpose = \"test\"", "reserved"),
        ("version = 1\n[[rules]]\nid = \"builtin:x\"\nmatch = [\"**\"]\npurpose = \"test\"", "reserved"),
        ("version = 1\n[[rules]]\nid = \"a\"\nmatch = []\npurpose = \"test\"", "nonempty"),
        ("version = 1\n[[rules]]\nid = \"a\"\nmatch = [\"**\"]\nkind = []\npurpose = \"test\"", "nonempty"),
        ("version = 1\n[[rules]]\nid = \"a\"\nmatch = [\"**\"]\nkind = [\"code\"]\npurpose = \"test\"", "unknown variant"),
        ("version = 1\n[[rules]]\nid = \"a\"\nmatch = [\"src/[\"]\npurpose = \"test\"", "Invalid glob"),
        ("version = 1\n[[rules]]\nid = \"a\"\nmatch = [\"/src/**\"]\npurpose = \"test\"", "relative"),
        ("version = 1\n[[rules]]\nid = \"a\"\nmatch = [\"!src/**\"]\npurpose = \"test\"", "negation"),
        ("version = 1\n[[rules]]\nid = \"a\"\nmatch = [\"../src/**\"]\npurpose = \"test\"", "relative"),
        ("version = 1\n[scope]\nexclude = [\"src/\"]", "relative"),
    ] {
        let error = Policy::from_toml(toml).unwrap_err();
        assert!(format!("{error:#}").contains(message), "{toml:?}: {error:#}");
    }
}

#[test]
fn globs_follow_the_root_relative_contract() {
    let policy = policy(
        r#"version = 1
[classification]
builtin_tests = false
[[rules]]
id = "root-only"
match = ["materials/**"]
purpose = "unclassified"
[[rules]]
id = "any-depth"
match = ["**/vendor/**"]
purpose = "unclassified"
[[rules]]
id = "one-segment"
match = ["packages/*/src/**"]
purpose = "production"
[[rules]]
id = "alternatives"
match = ["tools/{gen,report}/*.{mjs,ts}"]
purpose = "production"
[[rules]]
id = "classes"
match = ["v[0-9]/?.rs"]
purpose = "production"
[[rules]]
id = "escaped"
match = ["literal/\\*.txt"]
purpose = "production"
"#,
    );
    for (path, expected) in [
        ("materials/a.ts", "root-only"),
        ("packages/materials/a.ts", "fallback"),
        ("vendor/a.ts", "any-depth"),
        ("a/b/vendor/c/d.ts", "any-depth"),
        ("packages/x/src/deep/a.ts", "one-segment"),
        ("packages/x/y/src/a.ts", "fallback"),
        ("tools/gen/a.mjs", "alternatives"),
        ("tools/report/b.ts", "alternatives"),
        ("tools/other/b.ts", "fallback"),
        ("v1/a.rs", "classes"),
        ("v1/ab.rs", "fallback"),
        ("V1/a.rs", "fallback"),
        ("literal/*.txt", "escaped"),
        ("literal/a.txt", "fallback"),
    ] {
        assert_eq!(rule(&policy, path, Kind::Source), expected, "{path}");
    }
}

#[test]
fn stages_apply_overrides_then_builtin_tests_then_rules_then_fallback() {
    let policy = policy(
        r#"version = 1
[classification]
fallback = "production"
[[overrides]]
id = "archived-evidence"
match = ["docs/archive/**"]
purpose = "unclassified"
category = "evidence"
[[rules]]
id = "application"
match = ["src/**"]
kind = ["source"]
purpose = "production"
category = "implementation"
reason = "Owned application source."
"#,
    );
    let archived = policy.classify("docs/archive/probe.test.ts", Kind::Source);
    assert_eq!(
        (archived.rule.as_str(), archived.category.as_str()),
        ("archived-evidence", "evidence")
    );
    assert_eq!(
        rule(&policy, "src/a.test.ts", Kind::Source),
        "builtin:js-test-file"
    );
    assert_eq!(rule(&policy, "src/a.ts", Kind::Source), "application");
    let data = policy.classify("src/a.json", Kind::Data);
    assert_eq!(
        (data.rule.as_str(), data.purpose.to_string()),
        ("fallback", "production".into())
    );
    assert_eq!(data.category, "production");
    let tests = policy.classify("src/a.test.ts", Kind::Source);
    assert_eq!(
        (tests.purpose.to_string(), tests.category),
        ("test".into(), "test".into())
    );
    let disabled = Policy::from_toml(
        "version = 1\n[classification]\nbuiltin_tests = false\n[[rules]]\nid = \"app\"\nmatch = [\"src/**\"]\npurpose = \"production\"",
    )
    .unwrap();
    assert_eq!(rule(&disabled, "src/a.test.ts", Kind::Source), "app");
}

#[test]
fn builtin_test_conventions_cover_common_layouts_but_not_ambiguous_fixtures() {
    let policy = Policy::defaults();
    for (path, convention) in [
        ("tests/cli.rs", "test-directory"),
        ("pkg/src/__tests__/a.ts", "test-directory"),
        ("spec/models/user_spec.rb", "test-directory"),
        ("core/src/commonTest/kotlin/A.kt", "gradle-test-source-set"),
        (
            "core/src/releaseGuardTest/java/A.java",
            "gradle-test-source-set",
        ),
        (
            "core/src/testFixtures/java/A.java",
            "gradle-test-source-set",
        ),
        ("web/__snapshots__/a.ts.snap", "snapshot-directory"),
        ("web/a.spec.tsx", "js-test-file"),
        ("core/src/main/java/ParserTests.java", "jvm-test-class"),
        ("cmd/main_test.go", "go-test-file"),
        ("tool/test_cli.py", "python-test-file"),
        ("tool/conftest.py", "python-test-file"),
    ] {
        assert_eq!(
            rule(&policy, path, Kind::Source),
            format!("builtin:{convention}"),
            "{path}"
        );
    }
    for path in [
        "src/fixtures/a.ts",
        "src/latest/a.ts",
        "src/main/java/Testing.java",
        "contest.py",
    ] {
        assert_eq!(rule(&policy, path, Kind::Source), "fallback", "{path}");
    }
}

fn tree(files: &[&str]) -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    for path in files {
        let path = temp.path().join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "let x = 1;\n").unwrap();
    }
    temp
}

fn scanned(root: &std::path::Path, policy: &Policy) -> Vec<String> {
    let rows = file_records(scan(root, policy).unwrap().payload, policy).unwrap();
    rows.into_iter().map(|row| row.path).collect()
}

#[test]
fn default_and_configured_exclusions_are_additive() {
    let temp = tree(&[
        "src/a.ts",
        "build/gen.ts",
        "node_modules/x/a.ts",
        "pkg/node_modules/y/a.ts",
        ".build/a.ts",
        ".git/hooks/a.ts",
        "materials/a.ts",
        "pkg/materials/a.ts",
    ]);
    let paths = scanned(
        temp.path(),
        &policy("version = 1\n[scope]\nexclude = [\"materials/**\"]"),
    );
    assert_eq!(paths, ["build/gen.ts", "pkg/materials/a.ts", "src/a.ts"]);
    let all = scanned(
        temp.path(),
        &policy("version = 1\n[scope]\ndefault_excludes = false"),
    );
    assert!(all.contains(&".git/hooks/a.ts".to_string()), "{all:?}");
    assert!(all.contains(&"node_modules/x/a.ts".to_string()), "{all:?}");
}

#[test]
fn hidden_and_ignore_settings_control_selection_but_not_explicit_exclusions() {
    let temp = tree(&["src/a.ts", ".tool/a.ts", "ignored/a.ts", "excluded/a.ts"]);
    fs::write(temp.path().join(".ignore"), "ignored/\n").unwrap();
    let base = "version = 1\n[scope]\nexclude = [\"excluded/**\"]\n";
    assert_eq!(
        scanned(temp.path(), &policy(base)),
        [".tool/a.ts", "src/a.ts"]
    );
    assert_eq!(
        scanned(temp.path(), &policy(&format!("{base}hidden = false"))),
        ["src/a.ts"]
    );
    assert_eq!(
        scanned(
            temp.path(),
            &policy(&format!("{base}respect_ignore_files = false"))
        ),
        [".tool/a.ts", "ignored/a.ts", "src/a.ts"]
    );
}

#[test]
fn cli_scope_overrides_take_precedence_over_project_settings() {
    let temp = tree(&["src/a.ts", ".tool/a.ts", "gen/a.ts"]);
    let mut policy = policy("version = 1\n[scope]\nhidden = true");
    policy
        .override_scope(&ScopeOverrides {
            hidden: Some(false),
            respect_ignore_files: None,
            exclude: vec!["gen/**".into()],
        })
        .unwrap();
    assert_eq!(scanned(temp.path(), &policy), ["src/a.ts"]);
}

#[test]
fn product_example_configuration_is_valid() {
    let product = fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/PRODUCT.md")).unwrap();
    let example = product
        .split("```toml\n")
        .nth(1)
        .and_then(|rest| rest.split("```").next())
        .unwrap();
    let policy = Policy::from_toml(example).unwrap();
    assert_eq!(
        rule(&policy, "packages/a/bin/cli.ts", Kind::Source),
        "application-source"
    );
    assert_eq!(
        rule(&policy, "docs/archive/old.ts", Kind::Source),
        "archived-evidence"
    );
    assert_eq!(
        rule(&policy, "docs/archive/notes.md", Kind::Documentation),
        "fallback"
    );
    assert_eq!(rule(&policy, "other/a.ts", Kind::Source), "unmapped-source");
}

#[cfg(unix)]
#[test]
fn excluded_trees_are_not_traversed_and_single_level_patterns_do_not_prune() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tree(&["src/a.ts", "gen/a.ts", "gen/sub/b.ts", "private/a.ts"]);
    let private = temp.path().join("private");
    fs::set_permissions(&private, fs::Permissions::from_mode(0o000)).unwrap();
    let result = scan(
        temp.path(),
        &policy("version = 1\n[scope]\nexclude = [\"private/**\", \"gen/*\"]"),
    );
    fs::set_permissions(&private, fs::Permissions::from_mode(0o755)).unwrap();
    let policy = policy("version = 1\n[scope]\nexclude = [\"private/**\", \"gen/*\"]");
    let rows = file_records(result.unwrap().payload, &policy).unwrap();
    let paths: Vec<_> = rows.iter().map(|row| row.path.as_str()).collect();
    assert_eq!(paths, ["gen/sub/b.ts", "src/a.ts"]);
}
