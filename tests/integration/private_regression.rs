use locmetry::{build_report, file_records, language_kind, Kind, Policy};
use serde_json::Value;
use std::{collections::BTreeMap, env, fs, path::Path};

/// Totals with per-folder summaries, the shape saved measurements used.
fn summary_json(rows: &[locmetry::FileRecord]) -> Value {
    let report = build_report(rows);
    let mut value = serde_json::to_value(&report.totals).unwrap();
    value["folders"] = serde_json::to_value(&report.folders).unwrap();
    value
}

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

/// Renames a historical kind or purpose name in a summary key to its current spelling.
///
/// Keys combine a kind and a purpose (`source:other`), a purpose and a
/// category (`other:wrapper`), or name a purpose alone.
fn current_key_spelling(key: &str) -> String {
    key.split(':')
        .map(|part| current_name_spelling(part).unwrap_or(part))
        .collect::<Vec<_>>()
        .join(":")
}

/// The current spelling of a renamed kind or purpose, if `name` is a historical one.
fn current_name_spelling(name: &str) -> Option<&'static str> {
    match name {
        "data/config" => Some("data"),
        "unclassified" => Some("other"),
        _ => None,
    }
}

/// Renames historical kind and purpose names to their current spelling.
///
/// Saved measurements predate the `data/config` kind's rename to `data` in
/// schema `poc-3` and the `unclassified` purpose's rename to `other` in
/// locmetry 0.5; everything else, including every count and classification,
/// must still match exactly.
fn current_spelling(value: Value) -> Value {
    match value {
        Value::String(text) => match current_name_spelling(&text) {
            Some(name) => Value::String(name.into()),
            None => Value::String(text),
        },
        Value::Array(items) => Value::Array(items.into_iter().map(current_spelling).collect()),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| (current_key_spelling(&key), current_spelling(value)))
                .collect(),
        ),
        other => other,
    }
}

fn is_count(value: &Value) -> bool {
    value.get("code").is_some()
}

/// Applies the leaner schema-1 serialization to a saved measurement.
///
/// Saved measurements predate it: records then repeated single-language counts
/// and `counts.files`, and summaries listed zero counts. No count changes.
fn current_serialization(value: Value) -> Value {
    match value {
        Value::Object(map) if map.contains_key("code") => Value::Object(map),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| (key, current_serialization(value)))
                .filter(|(key, value)| {
                    let zero = is_count(value) && value["files"] == 0 && key != "all_counted";
                    let empty = value.as_object().is_some_and(|map| map.is_empty());
                    !(zero || empty)
                })
                .collect(),
        ),
        other => other,
    }
}

/// Removes per-record redundancy the way schema 1 serializes file records.
fn current_record(mut record: Value) -> Value {
    let record_map = record.as_object_mut().unwrap();
    record_map["counts"]
        .as_object_mut()
        .unwrap()
        .remove("files");
    let languages = record_map["languages"].as_object_mut().unwrap();
    if languages.len() <= 1 {
        record_map.remove("languages");
    } else {
        for counts in languages.values_mut() {
            counts.as_object_mut().unwrap().remove("files");
        }
    }
    record
}

/// Drops the rule ID of files the historical profiles left unmatched.
///
/// Those profiles reported `fallback`; their TOML translations end with an
/// explicit catch-all entry that reproduces the same purpose and category.
fn without_fallback_rule(mut record: Value, expected: &Value) -> Value {
    if expected["rule"] == "fallback" {
        record.as_object_mut().unwrap().remove("rule");
    }
    record
}

#[test]
#[ignore = "Requires private configurations, saved measurements, and local reviewed example paths"]
fn reviewed_configurations_reproduce_saved_measurements() {
    let directory = env::var("LOCMETRY_PRIVATE_FIXTURES")
        .expect("Set LOCMETRY_PRIVATE_FIXTURES to the ignored fixture directory");
    let mut checked = 0;
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        let Some(stem) = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_suffix(".locmetry.toml"))
        else {
            continue;
        };
        let sibling = |suffix: &str| path.with_file_name(format!("{stem}{suffix}"));
        let policy = Policy::load(&path).unwrap();
        let baseline = current_spelling(read_json(&sibling("-measurement.json")));
        let raw = read_json(&sibling("-measurement.tokei.json"));
        let reviewed = read_json(&sibling(".examples.json"));
        let kinds: BTreeMap<_, _> = baseline["files"]
            .as_array()
            .unwrap()
            .iter()
            .map(|file| {
                (
                    file["path"].as_str().unwrap(),
                    file["language"].as_str().unwrap(),
                )
            })
            .collect();
        let root = Path::new(reviewed["root"].as_str().unwrap());
        for example in reviewed["examples"].as_array().unwrap() {
            let name = example["path"].as_str().unwrap();
            assert!(
                root.join(name).exists(),
                "Reviewed example no longer exists"
            );
            match example["excluded"].as_bool() {
                Some(true) => assert!(policy.is_excluded(name)),
                _ => {
                    let kind = kinds
                        .get(name)
                        .map_or(Kind::Source, |language| language_kind(language));
                    let result = policy.classify(name, kind);
                    assert_eq!(result.purpose.to_string(), example["purpose"]);
                    assert_eq!(result.category, example["category"]);
                }
            }
        }
        let rows = file_records(&raw, &policy).unwrap();
        let actual = serde_json::to_value(&rows).unwrap();
        let actual = actual.as_array().unwrap();
        let expected: Vec<_> = baseline["files"]
            .as_array()
            .unwrap()
            .iter()
            .cloned()
            .map(current_record)
            .collect();
        assert_eq!(actual.len(), expected.len(), "File inventory size differs");
        for (actual, expected) in actual.iter().zip(&expected) {
            assert_eq!(
                without_fallback_rule(actual.clone(), expected),
                without_fallback_rule(expected.clone(), expected),
                "File record differs"
            );
        }
        assert!(
            summary_json(&rows) == current_serialization(baseline["summary"].clone()),
            "Totals differ from saved measurement"
        );
        checked += 1;
    }
    assert!(checked > 0, "No private configurations found");
}
