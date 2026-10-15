//! Shared JSON contract generation. TOML-only transforms belong at the config boundary.
use schemars::{generate::SchemaSettings, transform::RecursiveTransform, JsonSchema, Schema};
use serde_json::Value;

pub(crate) fn settings() -> SchemaSettings {
    SchemaSettings::draft2020_12().with_transform(RecursiveTransform(integer_bounds))
}

pub(crate) fn json<T: JsonSchema>() -> Value {
    settings()
        .into_generator()
        .into_root_schema_for::<T>()
        .to_value()
}

/// JSON Schema format annotations alone do not enforce the Rust integer range.
fn integer_bounds(schema: &mut Schema) {
    match schema.get("format").and_then(Value::as_str) {
        Some("uint64") => {
            schema.insert("maximum".into(), u64::MAX.into());
        }
        Some("int64") => {
            schema.insert("minimum".into(), i64::MIN.into());
            schema.insert("maximum".into(), i64::MAX.into());
        }
        _ => {}
    }
}

/// Schemars does not constrain enum map keys by default; reuse the reader's enum schema.
pub(crate) fn purpose_map<T: JsonSchema>(generator: &mut schemars::SchemaGenerator) -> Schema {
    let mut schema = <std::collections::BTreeMap<String, T>>::json_schema(generator);
    schema.insert(
        "propertyNames".into(),
        generator.subschema_for::<crate::Purpose>().to_value(),
    );
    schema
}

pub(crate) fn folder_purpose_map(generator: &mut schemars::SchemaGenerator) -> Schema {
    let mut schema =
        <std::collections::BTreeMap<String, std::collections::BTreeMap<String, u64>>>::json_schema(
            generator,
        );
    schema.insert(
        "additionalProperties".into(),
        purpose_map::<u64>(generator).to_value(),
    );
    schema
}
