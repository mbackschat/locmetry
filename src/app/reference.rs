//! The standalone reference queries: `--guide` and `--schema`, whole or by section.
use super::*;
use serde_json::{Map, Value};
use std::io::Write;

fn topic_name(topic: impl ValueEnum) -> String {
    topic
        .to_possible_value()
        .expect("every topic is listed")
        .get_name()
        .to_owned()
}

/// The `valid` section each requested id names, ignoring case, in request
/// order. A repeated or unknown id fails before anything is printed, so a
/// failed request leaves stdout empty (proposal, Amendment 1).
fn resolve<'a>(
    kind: &str,
    topic: String,
    ids: &[String],
    valid: &'a [String],
) -> Result<Vec<&'a str>> {
    let key = |id: &str| id.to_lowercase();
    if let Some(id) = ids.iter().enumerate().find_map(|(index, id)| {
        ids[..index]
            .iter()
            .any(|earlier| key(earlier) == key(id))
            .then_some(id)
    }) {
        anyhow::bail!("Section '{id}' is requested twice.");
    }
    let found: Vec<Option<&str>> = ids
        .iter()
        .map(|id| {
            valid
                .iter()
                .find(|name| key(name) == key(id))
                .map(String::as_str)
        })
        .collect();
    let unknown: Vec<String> = ids
        .iter()
        .zip(&found)
        .filter(|(_, name)| name.is_none())
        .map(|(id, _)| format!("'{id}'"))
        .collect();
    if !unknown.is_empty() {
        let noun = if unknown.len() == 1 {
            "section"
        } else {
            "sections"
        };
        anyhow::bail!(
            "{kind} '{topic}' has no {noun} {}. Sections: {}",
            unknown.join(", "),
            valid.join(", ")
        );
    }
    Ok(found.into_iter().flatten().collect())
}

/// Prints a guide, or the requested sections of it in order, each after a line
/// naming its source, so every section reads as it does alone.
pub(crate) fn run_guide(guide: Guide, sections: &[String]) -> Result<Outcome> {
    let (path, text) = guide.document();
    let version = env!("CARGO_PKG_VERSION");
    let mut output = String::new();
    if sections.is_empty() {
        output = format!("Source: {path} (locmetry {version})\n\n{text}");
    } else {
        let valid = crate::guides::section_ids(text);
        // The source line names the guide's own spelling of each id.
        for id in resolve("Guide", topic_name(guide), sections, &valid)? {
            let section = crate::guides::section(text, id).expect("a listed id has a section");
            output.push_str(&format!(
                "Source: {path}#{id} (locmetry {version})\n\n{section}"
            ));
        }
    }
    std::io::stdout().lock().write_all(output.as_bytes())?;
    Ok(Outcome::Complete)
}

/// Prints a schema, or one object whose keys are the requested sections in
/// order and whose values are fragments; their `$ref` values resolve against
/// the full schema.
///
/// Ids ignore case, and the keys use the schema's own spelling. `overview` is
/// the root with `$defs` reduced to names; it keeps `properties` in full,
/// which are small, because a definition wins over a property that differs
/// only in case, such as `Scope` over the property `scope`, whose default the
/// overview then still shows. The guard in tests/integration/guide_sections.rs
/// rejects names that collide within one namespace.
pub(crate) fn run_schema(schema: Schema, sections: &[String]) -> Result<Outcome> {
    let document = match schema {
        Schema::Config => locmetry::config_schema(),
        Schema::UserConfig => locmetry::user_config_schema(),
        Schema::Report => locmetry::report_schema(),
        Schema::Comparison => locmetry::compare::schema(),
        Schema::Explanation => locmetry::explain::schema(),
        Schema::Workspace => locmetry::workspace_schema(),
    };
    let document = serde_json::to_value(document)?;
    let selected = if sections.is_empty() {
        document
    } else {
        let valid = schema_sections(&document);
        let mut selected = Map::new();
        for name in resolve("Schema", topic_name(schema), sections, &valid)? {
            selected.insert(name.to_owned(), schema_section(&document, name));
        }
        Value::Object(selected)
    };
    println!("{}", serde_json::to_string_pretty(&selected)?);
    Ok(Outcome::Complete)
}

fn definitions(document: &Value) -> Vec<String> {
    document["$defs"]
        .as_object()
        .map(|members| members.keys().cloned().collect())
        .unwrap_or_default()
}

/// `overview`, every definition, then each property no definition shadows.
fn schema_sections(document: &Value) -> Vec<String> {
    let definitions = definitions(document);
    let mut ids = vec!["overview".to_owned()];
    ids.extend(definitions.iter().cloned());
    if let Some(properties) = document["properties"].as_object() {
        ids.extend(
            properties
                .keys()
                .filter(|name| !definitions.iter().any(|d| d.eq_ignore_ascii_case(name)))
                .cloned(),
        );
    }
    ids
}

/// The section a name from `schema_sections` selects; definitions come
/// before the properties they shadow.
fn schema_section(document: &Value, name: &str) -> Value {
    if name == "overview" {
        let mut overview = document.clone();
        let names = definitions(document)
            .into_iter()
            .map(Value::String)
            .collect();
        if overview.get("$defs").is_some() {
            overview["$defs"] = Value::Array(names);
        }
        return overview;
    }
    ["$defs", "properties"]
        .iter()
        .find_map(|namespace| document[namespace].get(name))
        .cloned()
        .expect("a listed name has a section")
}
