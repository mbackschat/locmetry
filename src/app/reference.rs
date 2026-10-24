//! The standalone reference queries: `--guide` and `--schema`, whole or one section.
use super::*;
use serde_json::Value;
use std::io::Write;

fn topic_name(topic: impl ValueEnum) -> String {
    topic
        .to_possible_value()
        .expect("every topic is listed")
        .get_name()
        .to_owned()
}

/// Prints a guide, or one section of it, after a line naming its source.
pub(crate) fn run_guide(guide: Guide, section: Option<&str>) -> Result<Outcome> {
    let (path, text) = guide.document();
    let (source, text) = match section {
        None => (path.to_owned(), text),
        Some(id) => {
            let text = crate::guides::section(text, id).map_err(|ids| {
                anyhow::anyhow!(
                    "Guide '{}' has no section '{id}'. Sections: {}",
                    topic_name(guide),
                    ids.join(", ")
                )
            })?;
            (format!("{path}#{id}"), text)
        }
    };
    let mut stdout = std::io::stdout().lock();
    writeln!(
        stdout,
        "Source: {source} (locmetry {})\n",
        env!("CARGO_PKG_VERSION")
    )?;
    stdout.write_all(text.as_bytes())?;
    Ok(Outcome::Complete)
}

/// Prints a schema, or one section of it as a fragment whose `$ref` values
/// resolve against the full schema.
///
/// `overview` is the root with `properties` and `$defs` reduced to their
/// names, so it carries the root description and lists every other section.
/// The guard in tests/integration/guide_sections.rs keeps the three namespaces
/// apart, so an id selects one value.
pub(crate) fn run_schema(schema: Schema, section: Option<&str>) -> Result<Outcome> {
    let document = match schema {
        Schema::Config => locmetry::config_schema(),
        Schema::UserConfig => locmetry::user_config_schema(),
        Schema::Report => locmetry::report_schema(),
        Schema::Comparison => locmetry::compare::schema(),
        Schema::Explanation => locmetry::explain::schema(),
        Schema::Workspace => locmetry::workspace_schema(),
    };
    let document = serde_json::to_value(document)?;
    let selected = match section {
        None => document,
        Some(id) => schema_section(&document, id).ok_or_else(|| {
            anyhow::anyhow!(
                "Schema '{}' has no section '{id}'. Sections: {}",
                topic_name(schema),
                schema_sections(&document).join(", ")
            )
        })?,
    };
    println!("{}", serde_json::to_string_pretty(&selected)?);
    Ok(Outcome::Complete)
}

const NAMESPACES: [&str; 2] = ["properties", "$defs"];

fn schema_sections(document: &Value) -> Vec<String> {
    let mut ids = vec!["overview".to_owned()];
    for namespace in NAMESPACES {
        if let Some(members) = document[namespace].as_object() {
            ids.extend(members.keys().cloned());
        }
    }
    ids
}

fn schema_section(document: &Value, id: &str) -> Option<Value> {
    if id == "overview" {
        let mut overview = document.clone();
        for namespace in NAMESPACES {
            if let Some(Value::Object(members)) = overview.get_mut(namespace) {
                let names = members.keys().cloned().map(Value::String).collect();
                overview[namespace] = Value::Array(names);
            }
        }
        return Some(overview);
    }
    NAMESPACES
        .iter()
        .find_map(|namespace| document[namespace].get(id).cloned())
}
