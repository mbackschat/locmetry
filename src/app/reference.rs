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

/// Rejects a repeated id and names the unknown ones, before anything is
/// printed, so a failed request leaves stdout empty (proposal, Amendment 1).
fn check_ids(
    kind: &str,
    topic: String,
    ids: &[String],
    valid: &[String],
    known: impl Fn(&str) -> bool,
) -> Result<()> {
    if let Some(id) = ids.iter().enumerate().find_map(|(index, id)| {
        ids[..index]
            .iter()
            .any(|earlier| earlier.to_lowercase() == id.to_lowercase())
            .then_some(id)
    }) {
        anyhow::bail!("Section '{id}' is requested twice.");
    }
    let unknown: Vec<String> = ids
        .iter()
        .filter(|id| !known(id))
        .map(|id| format!("'{id}'"))
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
    Ok(())
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
        // Ids ignore case; the source line names the guide's own spelling.
        let canonical = |id: &str| {
            valid
                .iter()
                .find(|known| known.to_lowercase() == id.to_lowercase())
                .cloned()
        };
        check_ids("Guide", topic_name(guide), sections, &valid, |id| {
            canonical(id).is_some()
        })?;
        for id in sections {
            let id = canonical(id).expect("checked above");
            let section = crate::guides::section(text, &id).expect("checked above");
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
        check_ids(
            "Schema",
            topic_name(schema),
            sections,
            &schema_sections(&document),
            |id| schema_section(&document, id).is_some(),
        )?;
        let mut selected = Map::new();
        for id in sections {
            let (name, value) = schema_section(&document, id).expect("checked above");
            selected.insert(name, value);
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

/// The section an id selects, with the schema's own spelling of its name.
fn schema_section(document: &Value, id: &str) -> Option<(String, Value)> {
    if id.eq_ignore_ascii_case("overview") {
        let mut overview = document.clone();
        let names = definitions(document)
            .into_iter()
            .map(Value::String)
            .collect();
        if overview.get("$defs").is_some() {
            overview["$defs"] = Value::Array(names);
        }
        return Some(("overview".to_owned(), overview));
    }
    ["$defs", "properties"].iter().find_map(|namespace| {
        document[namespace]
            .as_object()?
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(id))
            .map(|(name, value)| (name.clone(), value.clone()))
    })
}
