//! The built-in detection reference (docs/built-in-detection.md) lists every
//! built-in in the order it applies, so a detector or convention cannot ship
//! undocumented and the documented order stays the executable one.
use crate::cli::*;
use serde_json::Value;
use std::fs;

/// IDs in the first column of the table under `heading`, in row order.
fn documented(page: &str, heading: &str) -> Vec<String> {
    let section = page
        .split(&format!("\n## {heading}\n"))
        .nth(1)
        .unwrap_or_else(|| panic!("the reference has no section '{heading}'"));
    let section = section.split("\n## ").next().unwrap();
    section
        .lines()
        .filter_map(|line| line.strip_prefix("| `"))
        .map(|line| line.split('`').next().unwrap().to_string())
        .collect()
}

fn recorded(document: &Value, key: &str) -> Vec<String> {
    document["metadata"]["config"][key]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| {
            let id = entry["id"].as_str().or(entry.as_str()).unwrap();
            id.strip_prefix("builtin:").unwrap_or(id).to_string()
        })
        .collect()
}

#[test]
fn the_reference_lists_every_built_in_in_the_order_it_applies() {
    let page = fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/docs/built-in-detection.md"
    ))
    .expect("docs/built-in-detection.md");
    let temp = project(&[("a.py", "x = 1\n")]);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    assert_eq!(
        [
            documented(&page, "Test Conventions"),
            documented(&page, "Build Conventions")
        ]
        .concat(),
        recorded(&document, "builtin_conventions")
    );
    assert_eq!(
        documented(&page, "Language Mappings"),
        recorded(&document, "builtin_language_mappings")
    );
    assert_eq!(
        documented(&page, "Generated-Code Markers"),
        recorded(&document, "builtin_generated_markers")
    );
    assert_eq!(
        documented(&page, "Module Detectors"),
        locmetry::module_detector_ids().collect::<Vec<_>>()
    );
}
