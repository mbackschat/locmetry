use locmetry::{build_report, file_records, Profile};
use serde_json::Value;
use std::{env, fs, path::Path};

fn read_json(path: &Path) -> Value {
    serde_json::from_slice(&fs::read(path).unwrap()).unwrap()
}

#[test]
#[ignore = "Requires private profiles, saved measurements, and local reviewed example paths"]
fn reviewed_profiles_reproduce_saved_measurements() {
    let directory = env::var("LOCMETRY_PRIVATE_FIXTURES")
        .expect("Set LOCMETRY_PRIVATE_FIXTURES to the ignored fixture directory");
    let mut checked = 0;
    for entry in fs::read_dir(directory).unwrap() {
        let path = entry.unwrap().path();
        let Some(stem) = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_suffix("-rules.json"))
        else {
            continue;
        };
        let profile = Profile::from_json(&fs::read_to_string(&path).unwrap()).unwrap();
        let config = read_json(&path);
        for example in config["examples"].as_array().unwrap() {
            let name = example["path"].as_str().unwrap();
            assert!(
                profile.root().join(name).exists(),
                "Reviewed example no longer exists"
            );
            match example["excluded"].as_bool() {
                Some(true) => assert!(profile.is_excluded(name)),
                _ => {
                    let result = profile.classify(name);
                    assert_eq!(result.purpose.to_string(), example["purpose"]);
                    assert_eq!(result.category, example["category"]);
                }
            }
        }
        let baseline = read_json(&path.with_file_name(format!("{stem}-measurement.json")));
        let raw = read_json(&path.with_file_name(format!("{stem}-measurement.tokei.json")));
        {
            let rows = file_records(raw, &profile).unwrap();
            let actual = serde_json::to_value(&rows).unwrap();
            let actual = actual.as_array().unwrap();
            let expected = baseline["files"].as_array().unwrap();
            assert_eq!(actual.len(), expected.len(), "File inventory size differs");
            for (actual, expected) in actual.iter().zip(expected) {
                assert_eq!(actual, expected, "File record differs");
            }
            assert!(
                serde_json::to_value(build_report(&rows)).unwrap() == baseline["summary"],
                "Totals differ from saved measurement"
            );
        }
        checked += 1;
    }
    assert!(checked > 0, "No private profiles found");
}
