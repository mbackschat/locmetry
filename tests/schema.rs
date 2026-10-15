//! `--schema` lets agents discover the configuration contract, including its
//! semantics, from the binary alone.
use serde_json::{json, Value};
use std::process::Command;

fn schema(which: &str) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .args(["--schema", which])
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn toml_value(input: &str) -> Value {
    serde_json::to_value(toml::from_str::<toml::Table>(input).unwrap()).unwrap()
}

fn is_valid(schema: &Value, toml: &str) -> bool {
    jsonschema::validator_for(schema)
        .unwrap()
        .is_valid(&toml_value(toml))
}

/// Every fenced TOML block of a documentation file.
fn toml_blocks(path: &str) -> Vec<String> {
    let text = std::fs::read_to_string(path).unwrap();
    text.split("```toml\n")
        .skip(1)
        .map(|block| block.split("```").next().unwrap().to_string())
        .collect()
}

/// A JSON pointer's `description`, which must exist.
fn description<'a>(schema: &'a Value, pointer: &str) -> &'a str {
    schema
        .pointer(&format!("{pointer}/description"))
        .and_then(Value::as_str)
        .unwrap_or_else(|| panic!("no description at {pointer}"))
}

/// Every documented TOML example must load: workspace files, recognized by
/// their `roots`, against the workspace schema, all others as configurations.
#[test]
fn documented_examples_satisfy_their_schemas() {
    let schema = schema("config");
    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    let (workspaces, blocks): (Vec<String>, Vec<String>) = ["README.md", "docs/PRODUCT.md"]
        .iter()
        .flat_map(|path| toml_blocks(path))
        .partition(|block| block.lines().any(|line| line.starts_with("roots")));
    assert!(blocks.len() >= 3);
    // TOML has no null, so a schema allowing it would mislead editors and agents.
    assert!(!schema.to_string().contains("\"null\""));
    for block in blocks {
        assert!(is_valid(&schema, &block), "invalid per schema:\n{block}");
        locmetry::Policy::from_toml(&block).unwrap();
    }
    let workspace = self::schema("workspace");
    assert!(!workspaces.is_empty(), "the README shows a workspace file");
    for block in workspaces {
        assert!(is_valid(&workspace, &block), "invalid per schema:\n{block}");
    }
}

#[test]
fn config_schema_rejects_what_the_parser_rejects() {
    let schema = schema("config");
    for invalid in [
        "version = 2",
        "version = 1\nunknown = true",
        "version = 1\n[classification]\nbuiltin_module = false",
        "version = 1\n[[rules]]\nid = \"a\"\nmatch = []\npurpose = \"production\"",
        "version = 1\n[[rules]]\nid = \"a\"\nmatch = [\"**\"]\npurpose = \"shipped\"",
        "version = 1\n[[rules]]\nid = \"a\"\nmatch = [\"**\"]\nkind = [\"code\"]\npurpose = \"test\"",
        "version = 1\n[[languages.mappings]]\nid = \"a\"\nmatch = [\"**/*.x\"]\nlanguage = \"NoSuchLanguage\"",
    ] {
        assert!(!is_valid(&schema, invalid), "accepted: {invalid}");
        assert!(locmetry::Policy::from_toml(invalid).is_err(), "{invalid}");
    }
}

#[test]
fn config_schema_explains_precedence_and_switches() {
    let schema = schema("config");
    let root = description(&schema, "");
    let stages: Vec<usize> = [
        "`overrides`",
        "built-in test conventions",
        "`rules`",
        "build modules",
        "fallback",
    ]
    .iter()
    .map(|stage| root.find(stage).unwrap_or_else(|| panic!("{stage}")))
    .collect();
    assert!(stages.is_sorted(), "stages out of order in: {root}");

    for pointer in ["/properties/overrides", "/properties/rules"] {
        assert!(description(&schema, pointer).contains("first matching"));
    }
    let classification = schema
        .pointer("/properties/classification")
        .and_then(|reference| reference["$ref"].as_str())
        .and_then(|reference| reference.strip_prefix('#'))
        .unwrap();
    let switch = |name: &str| description(&schema, &format!("{classification}/properties/{name}"));
    for name in ["builtin_tests", "builtin_build", "builtin_modules"] {
        assert_eq!(
            schema.pointer(&format!("{classification}/properties/{name}/default")),
            Some(&json!(true)),
            "{name} must advertise its configuration default"
        );
    }
    assert!(switch("builtin_tests").contains("swift-test-target"));
    assert!(switch("builtin_build").contains("build-wrapper"));
    let modules = switch("builtin_modules");
    for build_file in [
        "package.json",
        "build.gradle.kts",
        "go.mod",
        "Package.swift",
    ] {
        assert!(modules.contains(build_file), "{build_file} in {modules}");
    }
    assert!(modules.contains("`false`"));
}

/// Blind agents guessed the entry shape, widened rules over build-module
/// source, and never reviewed source that modules claimed (round 4,
/// docs/research/real-project-trials.md). The description must state placement, layering, that
/// trap, and the review step, and its TOML example must be a valid configuration.
#[test]
fn config_schema_states_order_layers_and_a_valid_example() {
    let schema = schema("config");
    let root = description(&schema, "");
    for statement in [
        "where `[[overrides]]` and `[[rules]]` appear in the file does not matter",
        "`--config` replaces `.locmetryrc`",
        "same classification",
        "source a build module would claim",
        "`*` also matches names that start with a dot",
        "`docs/**`",
        "nothing reports it as unclaimed",
    ] {
        assert!(root.contains(statement), "{statement:?} missing in: {root}");
    }
    let example = root
        .split("```toml\n")
        .nth(1)
        .and_then(|block| block.split("```").next())
        .expect("a fenced TOML example");
    assert!(is_valid(&schema, example), "{example}");
    assert!(locmetry::Policy::from_toml(example).is_ok(), "{example}");
    assert!(description(&schema, "/properties/rules").contains("build-module source"));
}

/// Agents test-driving the schema parked maintained tooling and examples as
/// `unclassified` because the purposes read as "shipped" versus "not
/// shipped"; the descriptions must carry docs/PRODUCT.md's definition instead.
#[test]
fn config_schema_defines_production_as_maintained_non_test_source() {
    let schema = schema("config");
    let purposes = schema
        .pointer("/$defs/Purpose/oneOf")
        .and_then(Value::as_array)
        .unwrap();
    let meaning = |purpose: &str| {
        purposes
            .iter()
            .find(|variant| variant["const"] == purpose)
            .and_then(|variant| variant["description"].as_str())
            .unwrap()
            .to_string()
    };
    let production = meaning("production");
    for term in [
        "Maintained non-test source",
        "tooling",
        "example",
        "`category`",
    ] {
        assert!(production.contains(term), "{term} in {production}");
    }
    let unclassified = meaning("unclassified");
    assert!(unclassified.contains("not maintained"), "{unclassified}");
    assert!(unclassified.contains("is production"), "{unclassified}");
}

#[test]
fn config_schema_lists_language_names() {
    let schema = schema("config");
    let names = schema
        .pointer("/$defs/LanguageMapping/properties/language/enum")
        .and_then(Value::as_array)
        .unwrap();
    assert!(names.contains(&json!("Groovy")));
    assert!(names.contains(&json!("BASH")));
}

#[test]
fn user_config_schema_allows_only_scope_and_language_defaults() {
    let schema = schema("user-config");
    let [block] = &toml_blocks("docs/PRODUCT.md")[1..] else {
        panic!("docs/PRODUCT.md should hold the project and user-defaults examples");
    };
    assert!(is_valid(&schema, block));
    for project_only in [
        "version = 1\n[[rules]]\nid = \"a\"\nmatch = [\"**\"]\npurpose = \"test\"",
        "version = 1\n[classification]\nbuiltin_modules = false",
        "version = 1\n[scope]\nhidden = false",
    ] {
        assert!(!is_valid(&schema, project_only), "accepted: {project_only}");
    }
    assert!(description(&schema, "").contains("never classify"));
}

/// A project exercising every optional report field: a rule with a reason,
/// a build module, a mapped language, embedded languages, an unrecognized
/// file, unclaimed source, and an exclusion that excludes nothing.
fn report_fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    let files = [
        (
            ".locmetryrc",
            "version = 1\n[scope]\nexclude = [\"nowhere/**\"]\n[[rules]]\nid = \"tools\"\nmatch = [\"tools/**\"]\npurpose = \"production\"\ncategory = \"tooling\"\nreason = \"Maintained scripts.\"\n[[rules]]\nid = \"ghost\"\nmatch = [\"nowhere/**\"]\npurpose = \"production\"\n",
        ),
        (".gitmodules", "[submodule \"lib\"]\n\tpath = vendor/lib\n"),
        ("vendor/lib/a.py", "a = 1\n"),
        ("backend/build.gradle.kts", "plugins { kotlin(\"jvm\") }\n"),
        ("backend/src/main/kotlin/A.kt", "fun a() = 1\n"),
        ("backend/legacy.gradle", "apply plugin: 'java'\n"),
        ("tools/run.py", "print(1)\n"),
        ("proto/x.py", "x = 1\n"),
        ("README.md", "# Title\n\n```rust\nfn main() {}\n```\n"),
        ("logo.png", "not an image\n"),
    ];
    for (path, content) in files {
        let path = temp.path().join("project").join(path);
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(path, content).unwrap();
    }
    temp
}

fn measure(temp: &tempfile::TempDir, args: &[&str]) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .env("XDG_CONFIG_HOME", "/nonexistent/locmetry-tests")
        .current_dir(temp.path().join("project"))
        .args(args)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

/// Actual output must satisfy the generated contract, including strict nested objects.
#[test]
fn report_schema_describes_the_headline_and_the_full_report() {
    let schema = schema("report");
    let validator = jsonschema::validator_for(&schema).unwrap();
    let temp = report_fixture();
    let report_file = temp.path().join("report.json");
    let documents = [
        measure(&temp, &["--format", "json"]),
        measure(
            &temp,
            &[
                "--format",
                "json",
                "--output",
                report_file.to_str().unwrap(),
            ],
        ),
        measure(&temp, &["--files", "--output", "-"]),
        serde_json::from_str(&std::fs::read_to_string(&report_file).unwrap()).unwrap(),
    ];
    let full = &documents[2];
    for key in ["submodules", "unused_entries", "unused_exclusions"] {
        assert!(full["summary"].get(key).is_some(), "fixture lacks {key}");
    }
    assert!(full["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|file| file.get("languages").is_some() && file.get("language_mapping").is_none()));
    assert!(full["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|file| file.get("language_mapping").is_some()));
    for document in &documents {
        let errors: Vec<String> = validator
            .iter_errors(document)
            .map(|error| format!("{} at {}", error, error.instance_path()))
            .collect();
        assert!(errors.is_empty(), "{errors:#?}");
    }
    let mut surprise = full.clone();
    surprise["summary"]["surprise"] = json!(1);
    assert!(!validator.is_valid(&surprise), "unknown keys must fail");

    let code = description(&schema, "/$defs/LineCounts/properties/code");
    assert!(!code.is_empty());
}

/// Workspace reports keep the regular shape but replace the single-root
/// metadata with `members`; both the headline and the full report must
/// validate, so the schema cannot drift from either mode.
#[test]
fn report_schema_describes_workspace_reports() {
    let schema = schema("report");
    let validator = jsonschema::validator_for(&schema).unwrap();
    let temp = report_fixture();
    let output = |args: &[&str]| {
        let output = Command::new(env!("CARGO_BIN_EXE_locmetry"))
            .env("XDG_CONFIG_HOME", "/nonexistent/locmetry-tests")
            .current_dir(temp.path())
            .args(args)
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        serde_json::from_slice::<Value>(&output.stdout).unwrap()
    };
    let full = output(&["workspace", "--root", "project", "--files", "--output", "-"]);
    let headline = output(&["workspace", "--root", "project"]);
    std::fs::write(
        temp.path().join("ws.toml"),
        "version = 1\nroots = [\"project\"]\n",
    )
    .unwrap();
    let from_file = output(&["workspace", "--file", "ws.toml"]);
    assert!(full["metadata"]["members"][0].get("config").is_some());
    assert!(from_file["metadata"].get("workspace_path").is_some());
    for document in [&full, &headline, &from_file] {
        let errors: Vec<String> = validator
            .iter_errors(document)
            .map(|error| format!("{} at {}", error, error.instance_path()))
            .collect();
        assert!(errors.is_empty(), "{errors:#?}");
    }
    let members = schema
        .pointer("/$defs/Metadata/properties/members/description")
        .and_then(Value::as_str)
        .unwrap();
    assert!(members.contains("workspace"), "{members}");
}

#[test]
fn workspace_schema_describes_the_member_file() {
    let schema = schema("workspace");
    assert!(is_valid(
        &schema,
        "version = 1\nroots = [\"../web\", \"/srv/api\"]"
    ));
    assert!(!is_valid(
        &schema,
        "version = 1\nroots = [\"web\"]\nexclude = []"
    ));
    assert!(!is_valid(&schema, "version = 1"));
    let roots = description(&schema, "/properties/roots");
    assert!(roots.contains("relative to"), "{roots}");
}

#[test]
fn report_schema_rejects_negative_counts_and_unknown_effective_policy() {
    let schema = schema("report");
    let validator = jsonschema::validator_for(&schema).unwrap();
    let temp = report_fixture();
    let full = measure(&temp, &["--files", "--output", "-"]);
    for pointer in [
        "/summary/all_counted/code",
        "/metadata/coverage/selected_files",
        "/breakdown/0/files",
        "/files/0/counts/lines",
    ] {
        let mut invalid = full.clone();
        *invalid.pointer_mut(pointer).unwrap() = json!(-1);
        assert!(!validator.is_valid(&invalid), "negative count at {pointer}");
    }
    let mut invalid = full.clone();
    invalid["metadata"]["config"]["surprise"] = json!(true);
    assert!(
        !validator.is_valid(&invalid),
        "unknown effective-policy key"
    );
    let mut invalid = full;
    invalid["metadata"]["config"]["rules"][0]["purpose"] = json!("shipped");
    assert!(
        !validator.is_valid(&invalid),
        "invalid effective-policy purpose"
    );
}

#[test]
fn mapping_schemas_accept_backend_variant_names_as_well_as_display_names() {
    let schema = schema("config");
    for language in ["Bash", "Cpp"] {
        let config=format!("version = 1\n[[languages.mappings]]\nid = \"custom\"\nmatch = [\"**/*.custom\"]\nlanguage = \"{language}\"\n");
        locmetry::Policy::from_toml(&config).unwrap();
        assert!(is_valid(&schema, &config), "backend accepts {language}");
    }
}
