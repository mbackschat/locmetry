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
