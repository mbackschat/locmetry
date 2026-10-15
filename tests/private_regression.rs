use locmetry::{build_report, file_records, language_kind, Kind, Policy};
use serde_json::Value;
use std::{collections::BTreeMap, env, fs, path::Path};

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

/// Renames the historical `data/config` kind to its current `data` spelling.
///
/// Saved measurements predate the rename in schema `poc-3`; everything else,
/// including every count and classification, must still match exactly.
fn current_kind_spelling(value: Value) -> Value {
    match value {
        Value::String(text) if text == "data/config" => Value::String("data".into()),
        Value::Array(items) => Value::Array(items.into_iter().map(current_kind_spelling).collect()),
        Value::Object(map) => Value::Object(
            map.into_iter()
                .map(|(key, value)| {
                    let key = match key.strip_prefix("data/config:") {
                        Some(purpose) => format!("data:{purpose}"),
                        None => key,
                    };
                    (key, current_kind_spelling(value))
                })
                .collect(),
        ),
        other => other,
    }
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
        let baseline = current_kind_spelling(read_json(&sibling("-measurement.json")));
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
        let rows = file_records(raw, &policy).unwrap();
        let actual = serde_json::to_value(&rows).unwrap();
        let actual = actual.as_array().unwrap();
        let expected = baseline["files"].as_array().unwrap();
        assert_eq!(actual.len(), expected.len(), "File inventory size differs");
        for (actual, expected) in actual.iter().zip(expected) {
            assert_eq!(
                without_fallback_rule(actual.clone(), expected),
                without_fallback_rule(expected.clone(), expected),
                "File record differs"
            );
        }
        assert!(
            serde_json::to_value(build_report(&rows)).unwrap() == baseline["summary"],
            "Totals differ from saved measurement"
        );
        checked += 1;
    }
    assert!(checked > 0, "No private configurations found");
}
