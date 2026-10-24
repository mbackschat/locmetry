//! Addressable guide sections and schema definitions
//! (docs/proposals/guide-sections.md).
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
        vec!["--guide", "categories", "--section", "a", "--section", "b"],
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
fn a_schema_section_selects_a_property_or_a_definition() {
    let full = schema("config");
    for (id, expected) in [
        ("rules", &full["properties"]["rules"]),
        ("Refinement", &full["$defs"]["Refinement"]),
    ] {
        let out = run(&["--schema", "config", "--section", id]);
        assert_eq!(common::json(&out, 0), *expected, "{id}");
        assert!(out.stderr.is_empty());
    }
}

#[test]
fn the_schema_overview_keeps_root_semantics_and_names_every_section() {
    for topic in SCHEMAS {
        let full = schema(topic);
        let overview = common::json(&run(&["--schema", topic, "--section", "overview"]), 0);
        let mut expected = full.clone();
        let object = expected.as_object_mut().unwrap();
        for key in ["properties", "$defs"] {
            if let Some(Value::Object(members)) = object.get(key) {
                let names = members.keys().cloned().map(Value::String).collect();
                object.insert(key.to_owned(), Value::Array(names));
            }
        }
        assert_eq!(overview, expected, "{topic}");
        assert_eq!(overview["description"], full["description"], "{topic}");
    }
}

#[test]
fn an_unknown_schema_section_fails_and_lists_the_schemas_sections() {
    let out = run(&["--schema", "workspace", "--section", "Roots"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    let full = schema("workspace");
    let mut ids = vec!["overview".to_owned()];
    for key in ["properties", "$defs"] {
        if let Some(members) = full[key].as_object() {
            ids.extend(members.keys().cloned());
        }
    }
    assert_eq!(
        stderr(&out),
        format!(
            "error: Schema 'workspace' has no section 'Roots'. Sections: {}\n",
            ids.join(", ")
        )
    );
}

#[test]
fn schema_section_ids_select_exactly_one_value() {
    for topic in SCHEMAS {
        let full = schema(topic);
        // Ids match exactly, so a property `scope` and its definition `Scope` stay apart.
        let mut seen = std::collections::BTreeSet::new();
        for namespace in ["properties", "$defs"] {
            for name in full[namespace]
                .as_object()
                .into_iter()
                .flat_map(|m| m.keys())
            {
                assert_ne!(name, "overview", "{topic}: 'overview' is reserved");
                assert!(
                    seen.insert(name.clone()),
                    "{topic}: {name} names two sections"
                );
            }
        }
    }
}
