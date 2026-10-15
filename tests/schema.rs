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

#[test]
fn documented_configurations_satisfy_the_config_schema() {
    let schema = schema("config");
    assert_eq!(
        schema["$schema"],
        "https://json-schema.org/draft/2020-12/schema"
    );
    let blocks: Vec<String> = ["README.md", "PRODUCT.md"]
        .iter()
        .flat_map(|path| toml_blocks(path))
        .collect();
    assert!(blocks.len() >= 3);
    // TOML has no null, so a schema allowing it would mislead editors and agents.
    assert!(!schema.to_string().contains("\"null\""));
    for block in blocks {
        assert!(is_valid(&schema, &block), "invalid per schema:\n{block}");
        locmetry::Policy::from_toml(&block).unwrap();
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

/// Agents test-driving the schema parked maintained tooling and examples as
/// `unclassified` because the purposes read as "shipped" versus "not
/// shipped"; the descriptions must carry PRODUCT.md's definition instead.
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
    let [block] = &toml_blocks("PRODUCT.md")[1..] else {
        panic!("PRODUCT.md should hold the project and user-defaults examples");
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
