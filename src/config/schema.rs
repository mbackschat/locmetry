//! JSON Schemas of the configuration files, whose descriptions state the
//! built-in sets from the same constants that apply them.
use super::{
    module_rule, ConfigFile, Convention, UserDefaults, WorkspaceFile, BUILTIN, BUILTIN_BUILD_V1,
    BUILTIN_LANGUAGES_V1, BUILTIN_TESTS_V1, DEFAULT_EXCLUDES,
};
use crate::modules::DETECTORS;
use schemars::{transform::RecursiveTransform, JsonSchema, Schema, SchemaGenerator};
use serde_json::Value;
use tokei::LanguageType;

/// JSON Schema of the project configuration, with the semantics of each
/// setting and the built-in sets it switches.
pub fn config_schema() -> Value {
    schema::<ConfigFile>()
}

/// JSON Schema of the personal defaults file.
pub fn user_config_schema() -> Value {
    schema::<UserDefaults>()
}

/// JSON Schema of a workspace file.
pub fn workspace_schema() -> Value {
    schema::<WorkspaceFile>()
}

fn schema<T: JsonSchema>() -> Value {
    crate::schema::settings()
        .with_transform(RecursiveTransform(drop_null))
        .into_generator()
        .into_root_schema_for::<T>()
        .to_value()
}

/// Narrows `Option` fields' `["T", "null"]` types to `T`: TOML has no null,
/// so an optional key is simply absent.
fn drop_null(schema: &mut Schema) {
    let Some(Value::Array(types)) = schema.get_mut("type") else {
        return;
    };
    types.retain(|name| name != "null");
    if let [single] = types.as_slice() {
        let single = single.clone();
        schema.insert("type".into(), single);
    }
}

pub(super) fn language_names(_: &mut SchemaGenerator) -> Schema {
    // Tokei 15's language_type.tera.rs:345 accepts both display and enum names.
    let names: std::collections::BTreeSet<String> = LanguageType::list()
        .iter()
        .flat_map(|(language, _)| [language.name().to_string(), format!("{language:?}")])
        .collect();
    schemars::json_schema!({"type": "string", "enum": names})
}

/// Counter language names, as for mappings, plus the reserved `unrecognized`.
pub(super) fn correction_languages(generator: &mut SchemaGenerator) -> Schema {
    let mut schema = language_names(generator);
    if let Some(Value::Array(names)) = schema.get_mut("enum") {
        names.push(super::UNRECOGNIZED.into());
    }
    schema
}

/// Appends a generated list to a schema description, so the schema states the
/// built-in sets from the same constants that apply them.
fn append_description(schema: &mut Schema, text: String) {
    let description = match schema.get("description").and_then(Value::as_str) {
        Some(existing) => format!("{existing} {text}"),
        None => text,
    };
    schema.insert("description".into(), description.into());
}

fn globs(patterns: &[&str]) -> String {
    patterns
        .iter()
        .map(|pattern| format!("`{pattern}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

fn describe_conventions(schema: &mut Schema, set: &[Convention]) {
    let list = set
        .iter()
        .map(|convention| {
            format!(
                "`{BUILTIN}{}` ({}, category `{}`): {}",
                convention.id,
                convention.purpose,
                convention.category,
                globs(convention.patterns)
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    append_description(schema, format!("Conventions, in order: {list}."));
}

pub(super) fn describe_test_conventions(schema: &mut Schema) {
    describe_conventions(schema, BUILTIN_TESTS_V1);
}

pub(super) fn describe_build_conventions(schema: &mut Schema) {
    describe_conventions(schema, BUILTIN_BUILD_V1);
}

pub(super) fn describe_detectors(schema: &mut Schema) {
    let list = DETECTORS
        .iter()
        .map(|detector| {
            let build_files = detector
                .descriptors()
                .iter()
                .map(|name| format!("`{name}`"))
                .collect::<Vec<_>>()
                .join(" or ");
            format!(
                "`{}`, a folder with {build_files}: {}",
                module_rule(detector.id()),
                detector.layout()
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    append_description(
        schema,
        format!("Build files and production layouts: {list}."),
    );
}

pub(super) fn describe_default_excludes(schema: &mut Schema) {
    append_description(schema, format!("They are {}.", globs(DEFAULT_EXCLUDES)));
}

pub(super) fn describe_language_mappings(schema: &mut Schema) {
    let list = BUILTIN_LANGUAGES_V1
        .iter()
        .map(|(id, patterns, language)| {
            format!(
                "`{BUILTIN}{id}`: {} as {}",
                globs(patterns),
                language.name()
            )
        })
        .collect::<Vec<_>>()
        .join("; ");
    append_description(schema, format!("Mappings: {list}."));
}
