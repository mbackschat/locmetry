//! Guards that keep guide and schema sections addressable: heading form,
//! resolving references, a complete index and stable published ids
//! (docs/proposals/archive/guide-sections.md, Guard).
use crate::common;
use crate::guide_sections::{guide_ids, headings, slug, source, SCHEMAS, TOPICS};
use serde_json::Value;
use std::collections::BTreeSet;
use std::{fs, path::Path};

const PUBLISHED: &str = "tests/fixtures/guide-section-ids.txt";

fn repository(path: &str) -> String {
    fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join(path)).unwrap()
}

fn locmetry(args: &[&str]) -> std::process::Output {
    common::locmetry()
        .env("COLUMNS", "100")
        .env("NO_COLOR", "1")
        .args(args)
        .output()
        .unwrap()
}

fn guides() -> impl Iterator<Item = &'static str> {
    TOPICS.into_iter().filter(|topic| *topic != "index")
}

fn schema_ids(topic: &str) -> Vec<String> {
    crate::guide_sections::schema_ids(&common::json(&locmetry(&["--schema", topic]), 0))
}

/// Every id that can be requested now, as "guide TOPIC ID" or "schema TOPIC ID".
fn current_ids() -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
    for topic in guides() {
        for id in guide_ids(&source(topic)) {
            ids.insert(format!("guide {topic} {id}"));
        }
    }
    for topic in SCHEMAS {
        for id in schema_ids(topic) {
            ids.insert(format!("schema {topic} {id}"));
        }
    }
    ids
}

/// Every text that can carry a section reference: rendered help, schemas and
/// the documents users and agents read, including recorded guides.
fn reference_sources() -> Vec<(String, String)> {
    let mut texts = Vec::new();
    for args in [
        vec!["--help"],
        vec!["-h"],
        vec!["workspace", "--help"],
        vec!["compare", "--help"],
        vec!["explain", "--help"],
    ] {
        let out = locmetry(&args);
        texts.push((args.join(" "), String::from_utf8(out.stdout).unwrap()));
    }
    for topic in SCHEMAS {
        let out = locmetry(&["--schema", topic]);
        texts.push((
            format!("--schema {topic}"),
            String::from_utf8(out.stdout).unwrap(),
        ));
    }
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = vec![
        "README.md".to_owned(),
        "docs/PRODUCT.md".to_owned(),
        "docs/measurement.md".to_owned(),
    ];
    for folder in ["docs/guides", "docs/cli", "docs/showcase", "docs/cookbooks"] {
        for entry in fs::read_dir(root.join(folder)).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|extension| extension == "md") {
                files.push(path.strip_prefix(root).unwrap().display().to_string());
            }
        }
    }
    for file in files {
        let text = repository(&file);
        texts.push((file, text));
    }
    texts
}

/// Each `--section ID` in a text, with the topic option and value before it.
/// Whitespace is normalized first, because help wraps lines.
fn references(text: &str) -> Vec<(String, Vec<String>)> {
    let words: Vec<&str> = text.split_whitespace().collect();
    let mut found = Vec::new();
    for (index, word) in words.iter().enumerate() {
        let Some(id) = words.get(index + 1) else {
            continue;
        };
        let id = id.trim_end_matches(['`', ',', '.', ';', ')', ':']);
        if word.trim_start_matches('`') != "--section"
            || id.is_empty()
            // An all-capitals word such as `ID` is a placeholder, not a section.
            || id.chars().all(|c| c.is_ascii_uppercase())
            || !id
                .chars()
                .all(|c| c.is_alphanumeric() || matches!(c, '-' | '_'))
        {
            continue;
        }
        // Repeated sections share the topic before the first: skip earlier `--section ID` pairs.
        let mut topic = index;
        while topic >= 2 && words[topic - 2].trim_start_matches('`') == "--section" {
            topic -= 2;
        }
        let before = |back: usize| {
            topic
                .checked_sub(back)
                .map(|at| words[at].trim_start_matches('`'))
                .unwrap_or_default()
                .to_owned()
        };
        found.push((
            words[topic.saturating_sub(2)..=index + 1].join(" "),
            vec![before(2), before(1), "--section".to_owned(), id.to_owned()],
        ));
    }
    found
}

#[test]
fn guide_headings_are_plain_text_with_unique_ids() {
    for topic in guides() {
        let text = source(topic);
        let mut seen = BTreeSet::new();
        for (_, heading, _) in headings(&text) {
            assert!(
                !heading.contains(['`', '*', '[', ']']) && !heading.trim_end().ends_with('#'),
                "{topic}: '{heading}' must be plain text, so its id is predictable"
            );
            let id = slug(heading);
            assert!(!id.is_empty(), "{topic}: '{heading}' has an empty id");
            assert!(seen.insert(id.clone()), "{topic}: duplicate id {id}");
        }
    }
}

#[test]
fn every_section_reference_resolves() {
    let mut checked = 0;
    for (name, text) in reference_sources() {
        for (context, args) in references(&text) {
            assert!(
                matches!(args[0].as_str(), "--guide" | "--schema"),
                "{name}: '{context}' must name its topic as --guide TOPIC or --schema TOPIC"
            );
            let args: Vec<&str> = args.iter().map(String::as_str).collect();
            let out = locmetry(&args);
            assert_eq!(
                out.status.code(),
                Some(0),
                "{name}: '{context}': {}",
                String::from_utf8_lossy(&out.stderr)
            );
            checked += 1;
        }
    }
    assert!(checked >= 40, "only {checked} references found");
}

#[test]
fn the_index_routes_to_every_guide_section() {
    let index = source("index");
    let routed: BTreeSet<(String, String)> = references(&index)
        .into_iter()
        .filter(|(_, args)| args[0] == "--guide")
        .map(|(_, args)| (args[1].clone(), args[3].clone()))
        .collect();
    for topic in guides() {
        let text = source(topic);
        // A routed section covers its subsections, which it prints with it.
        let mut parents: Vec<(usize, String)> = Vec::new();
        for (level, heading, _) in headings(&text) {
            parents.retain(|(other, _)| *other < level);
            let id = slug(heading);
            let covered = routed.contains(&(topic.to_owned(), id.clone()))
                || parents
                    .iter()
                    .any(|(_, parent)| routed.contains(&(topic.to_owned(), parent.clone())));
            assert!(
                covered,
                "the index routes to no section covering {topic} {id}"
            );
            parents.push((level, id));
        }
    }
}

#[test]
fn published_ids_stay_requestable_unless_their_removal_is_breaking() {
    // Ids ignore case, so a published id stays requestable under any spelling.
    let published: BTreeSet<String> = repository(PUBLISHED)
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_lowercase)
        .collect();
    let current: BTreeSet<String> = current_ids().iter().map(|id| id.to_lowercase()).collect();
    let unpublished: Vec<_> = current.difference(&published).collect();
    assert!(
        unpublished.is_empty(),
        "add new section ids to {PUBLISHED}: {unpublished:?}"
    );
    let changelog = repository("CHANGELOG.md");
    let unreleased = changelog
        .split_once("## [Unreleased]")
        .unwrap()
        .1
        .split("\n## [")
        .next()
        .unwrap();
    for removed in published.difference(&current) {
        let id = removed.rsplit(' ').next().unwrap();
        assert!(
            unreleased
                .lines()
                .any(|line| line.contains("**Breaking**") && line.contains(&format!("`{id}`"))),
            "{removed} no longer resolves: record its removal as **Breaking** under Unreleased, or keep it"
        );
    }
}

#[test]
fn routed_reads_stay_within_a_third_of_the_whole_documents() {
    // The baseline is the batch every configuration session read whole before
    // sections existed, measured at its revision; tests/fixtures/guide-section-routes.json.
    let manifest: Value =
        serde_json::from_str(&repository("tests/fixtures/guide-section-routes.json")).unwrap();
    let limit = manifest["baseline"]["bytes"].as_f64().unwrap()
        * manifest["maximum_share"].as_f64().unwrap();
    for (name, commands) in manifest["routes"].as_object().unwrap() {
        let bytes: usize = commands
            .as_array()
            .unwrap()
            .iter()
            .map(|command| {
                let args: Vec<&str> = command
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|a| a.as_str().unwrap())
                    .collect();
                let out = locmetry(&args);
                assert_eq!(out.status.code(), Some(0), "{name}: {args:?}");
                out.stdout.len()
            })
            .sum();
        assert!(
            bytes as f64 <= limit,
            "{name}: {bytes} bytes exceed {limit:.0}"
        );
    }
}
