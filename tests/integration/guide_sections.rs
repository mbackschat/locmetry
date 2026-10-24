//! Addressable guide sections and schema definitions
//! (docs/proposals/archive/guide-sections.md).
use crate::common;
use serde_json::Value;
use std::{fs, path::Path, process::Output};

pub(crate) const TOPICS: [&str; 6] = [
    "configure",
    "selection",
    "reports",
    "compare",
    "categories",
    "index",
];
pub(crate) const SCHEMAS: [&str; 6] = [
    "config",
    "user-config",
    "report",
    "comparison",
    "explanation",
    "workspace",
];

fn run(args: &[&str]) -> Output {
    let root = tempfile::tempdir().unwrap();
    common::locmetry()
        .current_dir(root.path())
        .env("XDG_CONFIG_HOME", root.path().join("personal"))
        .env("NO_COLOR", "1")
        .args(args)
        .output()
        .unwrap()
}

pub(crate) fn guide_path(topic: &str) -> String {
    match topic {
        "index" => "docs/guides/README.md".to_owned(),
        _ => format!("docs/guides/{topic}.md"),
    }
}

pub(crate) fn source(topic: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(guide_path(topic))).unwrap()
}

/// GitHub's heading slug, as `anchors()` in scripts/doc-indexes.py computes it.
pub(crate) fn slug(heading: &str) -> String {
    heading
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | ' '))
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}

/// `##` and `###` headings outside fences and HTML comments, as (level, text, byte offset).
pub(crate) fn headings(text: &str) -> Vec<(usize, &str, usize)> {
    let mut found = Vec::new();
    let (mut fence, mut comment, mut offset) = (None::<&str>, false, 0);
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim_end();
        let start = offset;
        offset += line.len();
        if comment {
            comment = !trimmed.contains("-->");
            continue;
        }
        if let Some(marker) = fence {
            if trimmed.trim_start().starts_with(marker) {
                fence = None;
            }
            continue;
        }
        let opening = trimmed.trim_start();
        if opening.starts_with("```") || opening.starts_with("~~~") {
            fence = Some(&opening[..3]);
            continue;
        }
        if opening.starts_with("<!--") {
            comment = !trimmed.contains("-->");
            continue;
        }
        for (level, prefix) in [(2, "## "), (3, "### ")] {
            if let Some(heading) = trimmed.strip_prefix(prefix) {
                found.push((level, heading, start));
            }
        }
    }
    found
}

/// A section's text: its heading up to the next heading of the same or a higher level.
pub(crate) fn section<'a>(text: &'a str, id: &str) -> &'a str {
    let all = headings(text);
    let index = all
        .iter()
        .position(|(_, heading, _)| slug(heading) == id)
        .unwrap();
    let (level, _, start) = all[index];
    let end = all[index + 1..]
        .iter()
        .find(|(other, ..)| *other <= level)
        .map_or(text.len(), |(.., offset)| *offset);
    &text[start..end]
}

pub(crate) fn guide_ids(text: &str) -> Vec<String> {
    headings(text).iter().map(|(_, h, _)| slug(h)).collect()
}

fn stderr(out: &Output) -> String {
    String::from_utf8_lossy(&out.stderr).into_owned()
}

#[test]
fn a_guide_section_prints_its_heading_through_the_next_peer_heading() {
    for (topic, id) in [
        ("categories", "choose-a-name"),
        ("categories", "write-a-reviewed-policy"),
        ("selection", "common-issues-and-useful-queries"),
        ("reports", "which-files-have-unavailable-counts"),
    ] {
        let out = run(&["--guide", topic, "--section", id]);
        assert_eq!(out.status.code(), Some(0), "{topic} {id}: {}", stderr(&out));
        assert!(out.stderr.is_empty());
        let text = source(topic);
        let expected = format!(
            "Source: {}#{id} (locmetry {})\n\n{}",
            guide_path(topic),
            env!("CARGO_PKG_VERSION"),
            section(&text, id)
        );
        assert_eq!(String::from_utf8(out.stdout).unwrap(), expected);
    }
}

#[test]
fn a_level_two_section_includes_its_subsections() {
    let out = run(&[
        "--guide",
        "selection",
        "--section",
        "common-issues-and-useful-queries",
    ]);
    let text = String::from_utf8(out.stdout).unwrap();
    assert!(text.matches("\n### ").count() >= 3, "{text}");
}

#[test]
fn every_heading_outside_fences_is_a_section_and_nothing_else_is() {
    // compare.md's fenced shell scripts contain "# " comment lines, which a scanner
    // that ignored fences would take for headings.
    for topic in TOPICS {
        let text = source(topic);
        for id in guide_ids(&text) {
            let out = run(&["--guide", topic, "--section", &id]);
            assert_eq!(out.status.code(), Some(0), "{topic} {id}: {}", stderr(&out));
        }
    }
    let out = run(&["--guide", "compare", "--section", "baseline"]);
    assert_eq!(out.status.code(), Some(1));
}

#[test]
fn an_unknown_section_fails_and_lists_the_topics_sections() {
    let out = run(&["--guide", "categories", "--section", "choose-name"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let ids = guide_ids(&source("categories")).join(", ");
    assert_eq!(
        stderr(&out),
        format!("error: Guide 'categories' has no section 'choose-name'. Sections: {ids}\n")
    );
}

#[test]
fn a_section_needs_exactly_one_topic_and_nothing_to_measure() {
    for args in [
        vec!["--section", "choose-a-name"],
        vec!["--guide", "categories", "--section", "choose-a-name", "src"],
        vec![
            "--guide",
            "categories",
            "--section",
            "choose-a-name",
            "--files",
        ],
        vec![
            "--guide",
            "categories",
            "--schema",
            "config",
            "--section",
            "x",
        ],
        vec!["--section", "x", "compare", "a.json", "b.json"],
    ] {
        let out = run(&args);
        assert_eq!(out.status.code(), Some(1), "{args:?}: {}", stderr(&out));
        assert!(out.stdout.is_empty(), "{args:?}");
    }
    let out = run(&[
        "--no-color",
        "--guide",
        "categories",
        "--section",
        "choose-a-name",
    ]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
}

#[test]
fn the_index_is_the_guides_readme() {
    let out = run(&["--guide", "index"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let expected = format!(
        "Source: docs/guides/README.md (locmetry {})\n\n{}",
        env!("CARGO_PKG_VERSION"),
        source("index")
    );
    assert_eq!(String::from_utf8(out.stdout).unwrap(), expected);
}

fn schema(topic: &str) -> Value {
    common::json(&run(&["--schema", topic]), 0)
}

#[test]
fn a_schema_section_selects_a_definition_or_a_property_ignoring_case() {
    let full = schema("config");
    // A definition wins over a property that differs only in case: the property
    // `scope` refers to the definition `Scope`, and the overview keeps its default.
    for (id, key, expected) in [
        ("rules", "rules", &full["properties"]["rules"]),
        ("Refinement", "Refinement", &full["$defs"]["Refinement"]),
        ("refinement", "Refinement", &full["$defs"]["Refinement"]),
        ("scope", "Scope", &full["$defs"]["Scope"]),
        ("OVERVIEW", "overview", &Value::Null),
    ] {
        let out = run(&["--schema", "config", "--section", id]);
        let document = common::json(&out, 0);
        // A single section keeps the keyed shape, under the schema's own spelling (proposal, Amendment 1).
        assert_eq!(
            document.as_object().unwrap().keys().collect::<Vec<_>>(),
            [key],
            "{id}"
        );
        if !expected.is_null() {
            assert_eq!(document[key], *expected, "{id}");
        }
        assert!(out.stderr.is_empty());
    }
}

#[test]
fn the_schema_overview_keeps_root_semantics_properties_and_definition_names() {
    for topic in SCHEMAS {
        let full = schema(topic);
        let overview = common::json(&run(&["--schema", topic, "--section", "overview"]), 0)
            ["overview"]
            .clone();
        let mut expected = full.clone();
        if let Some(Value::Object(members)) = full.get("$defs") {
            expected["$defs"] = Value::Array(members.keys().cloned().map(Value::String).collect());
        }
        assert_eq!(overview, expected, "{topic}");
        assert_eq!(overview["properties"], full["properties"], "{topic}");
    }
}

#[test]
fn an_unknown_schema_section_fails_and_lists_the_schemas_sections() {
    let out = run(&["--schema", "config", "--section", "Roots"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert_eq!(
        stderr(&out),
        format!(
            "error: Schema 'config' has no section 'Roots'. Sections: {}\n",
            schema_ids(&schema("config")).join(", ")
        )
    );
}

/// The ids a schema accepts: `overview`, every definition, then each property
/// no definition shadows when case is ignored.
pub(crate) fn schema_ids(full: &Value) -> Vec<String> {
    let definitions: Vec<String> = full["$defs"]
        .as_object()
        .into_iter()
        .flat_map(|m| m.keys().cloned())
        .collect();
    let mut ids = vec!["overview".to_owned()];
    ids.extend(definitions.iter().cloned());
    ids.extend(
        full["properties"]
            .as_object()
            .into_iter()
            .flat_map(|m| m.keys())
            .filter(|name| !definitions.iter().any(|d| d.eq_ignore_ascii_case(name)))
            .cloned(),
    );
    ids
}

#[test]
fn schema_section_ids_select_exactly_one_value() {
    for topic in SCHEMAS {
        let full = schema(topic);
        for namespace in ["properties", "$defs"] {
            let names: Vec<String> = full[namespace]
                .as_object()
                .into_iter()
                .flat_map(|m| m.keys().cloned())
                .collect();
            for (index, name) in names.iter().enumerate() {
                assert!(
                    !name.eq_ignore_ascii_case("overview"),
                    "{topic}: 'overview' is reserved"
                );
                assert!(
                    !names[..index]
                        .iter()
                        .any(|other| other.eq_ignore_ascii_case(name)),
                    "{topic}: {name} differs from another {namespace} entry only in case"
                );
            }
        }
        // A property a definition shadows stays readable in full in the overview.
    }
}

#[test]
fn guide_section_ids_ignore_case() {
    let out = run(&["--guide", "categories", "--section", "Choose-A-Name"]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    assert_eq!(
        out.stdout,
        run(&["--guide", "categories", "--section", "choose-a-name"]).stdout
    );
    let out = run(&[
        "--guide",
        "categories",
        "--section",
        "choose-a-name",
        "--section",
        "CHOOSE-A-NAME",
    ]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
}

#[test]
fn several_guide_sections_print_in_the_requested_order_each_as_it_reads_alone() {
    let ids = ["refine-categories", "start-with-evidence"];
    let out = run(&[
        "--guide",
        "configure",
        "--section",
        ids[0],
        "--section",
        ids[1],
    ]);
    assert_eq!(out.status.code(), Some(0), "{}", stderr(&out));
    let alone: String = ids
        .iter()
        .map(|id| {
            String::from_utf8(run(&["--guide", "configure", "--section", id]).stdout).unwrap()
        })
        .collect();
    assert_eq!(String::from_utf8(out.stdout).unwrap(), alone);
}

#[test]
fn several_schema_sections_print_one_object_keyed_in_the_requested_order() {
    let full = schema("config");
    let out = run(&[
        "--schema",
        "config",
        "--section",
        "scope",
        "--section",
        "Entry",
        "--section",
        "overview",
    ]);
    let document = common::json(&out, 0);
    let keys: Vec<&String> = document.as_object().unwrap().keys().collect();
    assert_eq!(keys, ["Scope", "Entry", "overview"]);
    assert_eq!(document["Scope"], full["$defs"]["Scope"]);
    assert_eq!(document["Entry"], full["$defs"]["Entry"]);
    // Fragments keep references into the full schema.
    assert!(document.to_string().contains("#/$defs/"));
}

#[test]
fn a_repeated_or_unknown_id_fails_before_printing_anything() {
    let out = run(&[
        "--guide",
        "categories",
        "--section",
        "choose-a-name",
        "--section",
        "choose-a-name",
    ]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert_eq!(
        stderr(&out),
        "error: Section 'choose-a-name' is requested twice.\n"
    );
    let ids = guide_ids(&source("categories")).join(", ");
    let out = run(&[
        "--guide",
        "categories",
        "--section",
        "choose-a-name",
        "--section",
        "naming",
        "--section",
        "cases",
    ]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert_eq!(
        stderr(&out),
        format!("error: Guide 'categories' has no sections 'naming', 'cases'. Sections: {ids}\n")
    );
    let out = run(&[
        "--schema",
        "config",
        "--section",
        "Entry",
        "--section",
        "Entries",
    ]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert!(stderr(&out)
        .starts_with("error: Schema 'config' has no section 'Entries'. Sections: overview, "));
}
