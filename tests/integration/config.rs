use crate::common;
use common::write_all;
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

/// A declared category list catches typos and invented names, while
/// configurations without one keep free-form categories.
#[test]
fn declared_categories_restrict_entry_categories() {
    let entry = |category: &str| {
        format!("[[rules]]\nid = \"a\"\nmatch = [\"x/**\"]\npurpose = \"production\"\ncategory = \"{category}\"\n")
    };
    let declared = "version = 1\n[categories]\nproof = \"Machine-checked theorems.\"\n";
    for allowed in [
        "proof",
        "implementation",
        "build-tooling",
        "third-party-wrapper",
        "test",
        "production",
    ] {
        Policy::from_toml(&format!("{declared}{}", entry(allowed))).unwrap();
    }
    let error = Policy::from_toml(&format!("{declared}{}", entry("prof"))).unwrap_err();
    let message = format!("{error:#}");
    assert!(
        message.contains("\"prof\"") && message.contains("proof"),
        "{message}"
    );
    Policy::from_toml(&format!("version = 1\n{}", entry("anything"))).unwrap();
    let snapshot = policy(&format!("{declared}{}", entry("proof")))
        .snapshot()
        .clone();
    assert_eq!(
        serde_json::to_value(snapshot).unwrap()["categories"]["proof"],
        "Machine-checked theorems."
    );
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
purpose = "other"
[[rules]]
id = "any-depth"
match = ["**/vendor/**"]
purpose = "other"
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
purpose = "other"
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
    assert_eq!(data.category, "unclaimed");
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

/// Locks the placement contract (docs/PRODUCT.md, Matching and Precedence): the
/// stage order is fixed, so where `[[overrides]]` and `[[rules]]` appear in
/// the file is irrelevant, while order within each list decides overlaps.
#[test]
fn only_the_order_within_each_entry_list_matters() {
    let entry = |table: &str, id: &str, glob: &str, purpose: &str| {
        format!("[[{table}]]\nid = \"{id}\"\nmatch = [\"{glob}\"]\npurpose = \"{purpose}\"\n")
    };
    let archive = entry("overrides", "archive", "old/**", "other");
    let tools = entry("rules", "tools", "tools/**", "production");
    let generated = entry("rules", "generated", "tools/gen/**", "other");
    let decisions = |entries: &[&str]| {
        let policy = policy(&format!("version = 1\n{}", entries.concat()));
        ["old/a.test.ts", "tools/gen/b.ts", "tools/c.ts"]
            .map(|path| rule(&policy, path, Kind::Source))
    };
    let expected = ["archive", "tools", "tools"];
    for placement in [
        [archive.as_str(), &tools, &generated],
        [&tools, &archive, &generated],
        [&tools, &generated, &archive],
    ] {
        assert_eq!(decisions(&placement), expected, "{placement:?}");
    }
    assert_eq!(
        decisions(&[&generated, &tools, &archive]),
        ["archive", "generated", "tools"]
    );
}

#[test]
fn builtin_test_conventions_cover_common_layouts_but_not_ambiguous_fixtures() {
    let policy = Policy::defaults();
    for (path, convention) in [
        ("tests/cli.rs", "test-directory"),
        ("pkg/src/__tests__/a.ts", "test-directory"),
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
        ("e2e/src/specs/album.e2e-spec.ts", "js-e2e-spec-file"),
        ("core/src/main/java/ParserTests.java", "jvm-test-class"),
        ("cmd/main_test.go", "go-test-file"),
        ("src/tests.rs", "rust-test-module"),
        ("crates/core/src/output/tests.rs", "rust-test-module"),
        ("tool/test_cli.py", "python-test-file"),
        ("tool/conftest.py", "python-test-file"),
        (
            "app/integration_test/login_test.dart",
            "flutter-integration-test",
        ),
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
        // `spec` folders also hold specifications, e.g. a reference project's
        // `.claude/skills/spec/` tooling (docs/research/loc-counting-tools.md).
        ".claude/skills/spec/scripts/emit.mjs",
        // Only the NestJS/Angular `.e2e-spec` suffix is a test name; a bare
        // `-spec` suffix also names specifications.
        "src/api-spec.ts",
        // Flutter's `integration_test/` holds Dart; other languages there are
        // left to project entries.
        "tools/integration_test/run.py",
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
    let rows = file_records(&scan(root, policy).unwrap().payload, policy).unwrap();
    rows.into_iter().map(|row| row.path).collect()
}

#[test]
fn repository_policy_excludes_the_demo_without_affecting_direct_demo_scans() {
    let root = tempfile::tempdir().unwrap();
    let config =
        fs::read_to_string(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join(".locmetryrc"))
            .unwrap();
    write_all(
        root.path(),
        &[
            (".locmetryrc", &config),
            ("main.py", "print(1)\n"),
            ("examples/cli-workspace/demo/package.json", "{}\n"),
            ("examples/cli-workspace/demo/src/app.js", "app();\n"),
            ("examples/cli-workspace/demo/image.asset", "synthetic\n"),
            (
                "examples/cli-workspace/demo/.locmetryrc",
                "version = 1\n[[rules]]\nid = \"fixture-policy\"\nmatch = [\"src/**\"]\npurpose = \"production\"\n",
            ),
        ],
    );
    let measure = |path: &std::path::Path| {
        common::json(
            &common::locmetry()
                .current_dir(path)
                .args(["--no-user-config", "--files", "--output", "-"])
                .output()
                .unwrap(),
            0,
        )
    };
    let outer = measure(root.path());
    assert_eq!(outer["files"].as_array().unwrap().len(), 1);
    assert_eq!(outer["files"][0]["path"], "main.py");
    assert!(outer["metadata"]["modules"].as_array().unwrap().is_empty());
    assert!(outer["metadata"]["coverage"]["unrecognized"]
        .as_array()
        .unwrap()
        .is_empty());

    let inner = measure(&root.path().join("examples/cli-workspace/demo"));
    let app = inner["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|file| file["path"] == "src/app.js")
        .unwrap();
    assert_eq!(app["rule"], "fixture-policy");
    assert_eq!(app["purpose"], "production");
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
    let product =
        fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/docs/PRODUCT.md")).unwrap();
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
    let rows = file_records(&result.unwrap().payload, &policy).unwrap();
    let paths: Vec<_> = rows.iter().map(|row| row.path.as_str()).collect();
    assert_eq!(paths, ["gen/sub/b.ts", "src/a.ts"]);
}

fn mapped_tree() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    write_all(
        temp.path(),
        &[
            (
                "build.gradle",
                "// Build.\napply plugin: 'java'\n\ndependencies {}\n",
            ),
            (".vscode/settings.jsonc", "{\n  // comment\n  \"a\": 1\n}\n"),
            ("data/events.jsonl", "{\"a\":1}\n{\"a\":2}\n"),
            (
                "gradle.properties",
                "# comment\norg.gradle.jvmargs=-Xmx2g\n",
            ),
            ("assets/logo.xyz", "binary\n"),
        ],
    );
    temp
}

fn mapping_of(rows: &[locmetry::FileRecord], path: &str) -> (String, Kind, Option<String>) {
    let row = rows
        .iter()
        .find(|row| row.path == path)
        .unwrap_or_else(|| panic!("missing {path}"));
    (row.language.clone(), row.kind, row.language_mapping.clone())
}

#[test]
fn builtin_language_mappings_count_files_the_backend_does_not_recognize() {
    let temp = mapped_tree();
    let policy = Policy::defaults();
    let scan = scan(temp.path(), &policy).unwrap();
    let rows = scan.records(&policy).unwrap();
    assert_eq!(
        mapping_of(&rows, "build.gradle"),
        (
            "Groovy".into(),
            Kind::Source,
            Some("builtin:gradle-script".into())
        )
    );
    let gradle = rows.iter().find(|row| row.path == "build.gradle").unwrap();
    assert_eq!(
        (
            gradle.counts.code,
            gradle.counts.comments,
            gradle.counts.blanks
        ),
        (2, 1, 1)
    );
    assert_eq!(
        mapping_of(&rows, ".vscode/settings.jsonc"),
        ("JSON".into(), Kind::Data, Some("builtin:jsonc".into()))
    );
    assert_eq!(
        mapping_of(&rows, "data/events.jsonl").2,
        Some("builtin:json-lines".into())
    );
    assert_eq!(
        mapping_of(&rows, "gradle.properties"),
        (
            "INI".into(),
            Kind::Data,
            Some("builtin:java-properties".into())
        )
    );
    assert_eq!(scan.coverage.unrecognized, ["assets/logo.xyz"]);
    assert_eq!(scan.coverage.mapped_files, 4);
    assert_eq!(scan.coverage.counted_files, 4);
    assert!(!scan.inaccurate);
}

#[test]
fn project_mappings_precede_builtin_mappings_and_feed_classification() {
    let temp = mapped_tree();
    let policy = policy(
        r#"version = 1
[[languages.mappings]]
id = "xyz-as-python"
match = ["**/*.xyz"]
language = "Python"
[[languages.mappings]]
id = "gradle-as-kotlin"
match = ["*.gradle"]
language = "Kotlin"
[[rules]]
id = "build-tooling"
match = ["*.gradle"]
kind = ["source"]
purpose = "production"
category = "build-tooling"
"#,
    );
    let rows = scan(temp.path(), &policy)
        .unwrap()
        .records(&policy)
        .unwrap();
    assert_eq!(
        mapping_of(&rows, "assets/logo.xyz").2,
        Some("xyz-as-python".into())
    );
    assert_eq!(
        mapping_of(&rows, "build.gradle"),
        (
            "Kotlin".into(),
            Kind::Source,
            Some("gradle-as-kotlin".into())
        )
    );
    let gradle = rows.iter().find(|row| row.path == "build.gradle").unwrap();
    assert_eq!(gradle.classification.category, "build-tooling");
}

#[test]
fn disabled_builtin_mappings_leave_unrecognized_files_visible() {
    let temp = mapped_tree();
    let disabled = policy("version = 1\n[languages]\nbuiltin = false");
    let scan = scan(temp.path(), &disabled).unwrap();
    assert_eq!(scan.coverage.mapped_files, 0);
    assert_eq!(scan.coverage.unrecognized.len(), 5);
    assert!(scan.records(&disabled).unwrap().is_empty());
}

#[cfg(unix)]
#[test]
fn unreadable_mapped_files_are_reported_as_unread_and_inaccurate() {
    use std::os::unix::fs::PermissionsExt;
    let temp = tempfile::tempdir().unwrap();
    write_all(temp.path(), &[("build.gradle", "apply plugin: 'java'\n")]);
    let file = temp.path().join("build.gradle");
    fs::set_permissions(&file, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read(&file).is_ok() {
        return; // Permission bits do not restrict this user (for example root).
    }
    let result = scan(temp.path(), &Policy::defaults());
    fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
    let scan = result.unwrap();
    assert_eq!(scan.coverage.unread, ["build.gradle"]);
    assert!(scan.inaccurate);
}

#[test]
fn invalid_language_mappings_fail_before_scanning() {
    let mapping = |body: &str| format!("version = 1\n[[languages.mappings]]\n{body}");
    for (toml, message) in [
        (
            mapping("id = \"a\"\nmatch = [\"**/*.x\"]\nlanguage = \"Groovyy\""),
            "Unknown language",
        ),
        (
            mapping("id = \"a\"\nmatch = []\nlanguage = \"Groovy\""),
            "nonempty",
        ),
        (
            mapping("id = \"builtin:a\"\nmatch = [\"**\"]\nlanguage = \"Groovy\""),
            "reserved",
        ),
        (
            mapping("id = \"a\"\nmatch = [\"**\"]\nlanguage = \"Groovy\"\nkind = \"source\""),
            "unknown field",
        ),
        (
            format!(
                "{}\n[[languages.mappings]]\nid = \"a\"\nmatch = [\"**\"]\nlanguage = \"JSON\"",
                mapping("id = \"a\"\nmatch = [\"**\"]\nlanguage = \"Groovy\"")
            ),
            "unique",
        ),
    ] {
        let error = Policy::from_toml(&toml).unwrap_err();
        assert!(
            format!("{error:#}").contains(message),
            "{toml:?}: {error:#}"
        );
    }
}

#[test]
fn builtin_build_conventions_separate_build_scripts_and_wrappers() {
    let policy = Policy::defaults();
    for (path, rule, purpose, category) in [
        (
            "packages/core/build.gradle.kts",
            "builtin:build-script",
            "production",
            "build-tooling",
        ),
        (
            "settings.gradle",
            "builtin:build-script",
            "production",
            "build-tooling",
        ),
        (
            "native/CMakeLists.txt",
            "builtin:build-script",
            "production",
            "build-tooling",
        ),
        (
            "Makefile",
            "builtin:build-script",
            "production",
            "build-tooling",
        ),
        (
            "service/pom.xml",
            "builtin:build-script",
            "production",
            "build-tooling",
        ),
        (
            "gradlew",
            "builtin:build-wrapper",
            "other",
            "third-party-wrapper",
        ),
        (
            "gradle/wrapper/gradle-wrapper.properties",
            "builtin:build-wrapper",
            "other",
            "third-party-wrapper",
        ),
        (
            "service/mvnw.cmd",
            "builtin:build-wrapper",
            "other",
            "third-party-wrapper",
        ),
        (
            "src/test/resources/app/build.gradle",
            "builtin:test-directory",
            "test",
            "test",
        ),
        (
            "gradle/libs.versions.toml",
            "fallback",
            "other",
            "unclaimed",
        ),
    ] {
        let result = policy.classify(path, Kind::Source);
        assert_eq!(
            (
                result.rule.as_str(),
                result.purpose.to_string().as_str(),
                result.category.as_str()
            ),
            (rule, purpose, category),
            "{path}"
        );
    }
}

#[test]
fn ownership_overrides_and_disabling_take_precedence_over_build_conventions() {
    let owned = policy(
        "version = 1\n[[overrides]]\nid = \"consumer-tests\"\nmatch = [\"consumer-tests/**\"]\npurpose = \"test\"\n",
    );
    assert_eq!(
        rule(&owned, "consumer-tests/jvm/build.gradle.kts", Kind::Source),
        "consumer-tests"
    );
    let disabled = policy("version = 1\n[classification]\nbuiltin_build = false");
    assert_eq!(rule(&disabled, "build.gradle", Kind::Source), "fallback");
}

#[test]
fn retired_json_profiles_fail_with_a_migration_hint() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("rules.json");
    fs::write(&path, r#"{"project":"p","root":".","rules":[]}"#).unwrap();
    let error = Policy::load(&path).unwrap_err();
    assert!(
        format!("{error:#}").contains("translate it to TOML"),
        "{error:#}"
    );
}

/// Purposes and kinds are named twice, for display and for serialization;
/// both spellings reach users (table and Markdown versus JSON), so they must
/// agree for every variant.
#[test]
fn display_names_match_serialized_names() {
    for purpose in locmetry::Purpose::ALL {
        assert_eq!(serde_json::json!(purpose), purpose.to_string());
    }
    for kind in Kind::ALL {
        assert_eq!(serde_json::json!(kind), kind.to_string());
    }
}

/// Reports name the embedded counter's version; `BACKEND` states it by hand
/// because Cargo exposes no dependency versions to code, so it must follow the
/// exact pin in Cargo.toml on every upgrade.
#[test]
fn backend_name_follows_the_pinned_tokei_version() {
    let manifest =
        std::fs::read_to_string(concat!(env!("CARGO_MANIFEST_DIR"), "/Cargo.toml")).unwrap();
    let pin = manifest
        .lines()
        .find_map(|line| line.strip_prefix("tokei = { version = \"="))
        .and_then(|rest| rest.split('"').next())
        .expect("Cargo.toml pins tokei exactly");
    assert_eq!(locmetry::BACKEND, format!("tokei {pin} (embedded)"));
}

/// Locks the 0.5 clean break: the old purpose name fails with a pointer to its
/// replacement wherever a purpose is written, instead of a bare unknown variant.
#[test]
fn the_old_unclassified_purpose_names_its_replacement() {
    for toml in [
        "version = 1\n[classification]\nfallback = \"unclassified\"",
        "version = 1\n[[rules]]\nid = \"a\"\nmatch = [\"x/**\"]\npurpose = \"unclassified\"",
    ] {
        let error = format!("{:#}", Policy::from_toml(toml).unwrap_err());
        assert!(
            error.contains("\"unclassified\" was renamed to \"other\""),
            "{error}"
        );
    }
    let policy =
        policy("version = 1\n[[rules]]\nid = \"a\"\nmatch = [\"x/**\"]\npurpose = \"other\"");
    assert_eq!(
        policy.classify("x/a.rs", Kind::Source).purpose.to_string(),
        "other"
    );
}

/// Locks readable configuration errors: stderr sanitizing flattens newlines,
/// so the parser's multi-line snippet became one garbled line; errors now
/// name the position and the problem on one line instead.
#[test]
fn configuration_errors_name_the_position_on_one_line() {
    let temp = tempfile::tempdir().unwrap();
    common::write(
        temp.path(),
        ".locmetryrc",
        "version = 1\n\n[[rules]]\nid = \"a\"\nmatch = [\"x/**\"]\npurpose = \"unclassified\"\n",
    );
    let output = common::locmetry().arg(temp.path()).output().unwrap();
    assert_eq!(output.status.code(), Some(1));
    let error = String::from_utf8(output.stderr).unwrap();
    assert_eq!(error.lines().count(), 1, "{error}");
    assert!(
        error.contains(
            ".locmetryrc: line 6, column 11: purpose \"unclassified\" was renamed to \"other\""
        ),
        "{error}"
    );
    assert!(!error.contains('|') && !error.contains('^'), "{error}");
}
