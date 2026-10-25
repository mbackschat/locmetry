//! Configuration errors name the fix, not only the fault. Each case comes from
//! the broken-configuration corpus of 2026-10-25, where agents repairing a
//! `.locmetryrc` had to read the schema or guess after these messages
//! (docs/research/agent-experiments-2026-10-25.md).
use crate::cli::{project, run};

/// Stderr of a measurement whose `.locmetryrc` is `config`, which must fail
/// with exit 1 and an empty stdout.
fn error(config: &str) -> String {
    let temp = project(&[(".locmetryrc", config), ("src/a.py", "x = 1\n")]);
    let output = run(&temp.path().join("project"), &["--format", "json"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    String::from_utf8(output.stderr).unwrap()
}

const RULE: &str = "version = 1\n[[rules]]\nid = \"r\"\n";

#[test]
fn unknown_fields_suggest_the_field_meant() {
    for (config, hint) in [
        (
            "version = 1\n[[language_mappings]]\nid = \"m\"\nmatch = [\"*.x\"]\nlanguage = \"JSON\"\n",
            "Did you mean '[[languages.mappings]]'?",
        ),
        (
            &format!("{RULE}match = [\"src/**\"]\nexclude = [\"src/gen/**\"]\npurpose = \"test\"\n"),
            "Did you mean 'except'? It leaves files to later stages, while 'exclude' under '[scope]' removes them from measurement.",
        ),
        (
            &format!("{RULE}pattern = [\"src/**\"]\npurpose = \"test\"\n"),
            "Did you mean 'match'?",
        ),
        (
            "version = 1\n[classification]\nbuiltin_test = false\n",
            "Did you mean 'builtin_tests'?",
        ),
    ] {
        let stderr = error(config);
        assert!(stderr.contains(hint), "{stderr}");
        assert!(!stderr.contains("newer"), "{stderr}");
    }
    let stderr = error(&format!(
        "{RULE}match = [\"src/**\"]\npurpose = \"test\"\ncontent = \"x\"\n"
    ));
    assert!(
        stderr.contains(&format!(
            "Locmetry {} does not know this field. A newer release may.",
            env!("CARGO_PKG_VERSION")
        )),
        "{stderr}"
    );
}

#[test]
fn an_unsupported_version_names_the_one_this_release_reads() {
    let stderr = error("version = 2\n");
    assert!(
        stderr.contains(&format!(
            "Unsupported configuration version 2. Locmetry {} reads version 1. A newer release may read this file.",
            env!("CARGO_PKG_VERSION")
        )),
        "{stderr}"
    );
}

#[test]
fn an_unknown_language_suggests_the_name_meant() {
    let mapping = |language: &str| {
        format!("version = 1\n[[languages.mappings]]\nid = \"m\"\nmatch = [\"*.x\"]\nlanguage = \"{language}\"\n")
    };
    for (language, hint) in [("json", "'JSON'"), ("Kotlinn", "'Kotlin'")] {
        let stderr = error(&mapping(language));
        assert!(
            stderr.contains(&format!(
                "Language names are case-sensitive. Did you mean {hint}?"
            )),
            "{stderr}"
        );
        assert!(
            stderr.contains("'--schema config --section LanguageName' lists them."),
            "{stderr}"
        );
    }
}

#[test]
fn type_errors_use_toml_words_and_single_quotes() {
    let stderr = error(&format!("{RULE}match = \"src/**\"\npurpose = \"test\"\n"));
    assert!(
        stderr.contains("invalid type: string 'src/**', expected an array"),
        "{stderr}"
    );
    assert!(!stderr.contains('"'), "{stderr}");
    let stderr =
        error("version = 1\n[rules]\nid = \"r\"\nmatch = [\"src/**\"]\npurpose = \"test\"\n");
    assert!(
        stderr.contains("invalid type: table, expected an array. Write '[[rules]]': each entry is a '[[rules]]' table of its own."),
        "{stderr}"
    );
    let stderr = error("version = \"1\"\n");
    assert!(
        stderr.contains("invalid type: string '1', expected an integer"),
        "{stderr}"
    );
}

#[test]
fn id_errors_say_which_rule_an_id_breaks() {
    let stderr = error(&format!(
        "{RULE}match = [\"a/**\"]\npurpose = \"test\"\n[[rules]]\nid = \"r\"\nmatch = [\"b/**\"]\npurpose = \"test\"\n"
    ));
    assert!(
        stderr.contains("Entry ID 'r' is used more than once. IDs are unique across overrides, rules, refinements and language corrections."),
        "{stderr}"
    );
    let stderr =
        error("version = 1\n[[rules]]\nid = \"\"\nmatch = [\"a/**\"]\npurpose = \"test\"\n");
    assert!(stderr.contains("An entry has an empty ID."), "{stderr}");
    let stderr = error(
        "version = 1\n[[rules]]\nid = \"builtin:x\"\nmatch = [\"a/**\"]\npurpose = \"test\"\n",
    );
    assert!(
        stderr.contains("Entry ID 'builtin:x' is reserved. 'fallback' and IDs starting with 'builtin:' name Locmetry's own stages."),
        "{stderr}"
    );
}

#[test]
fn an_invalid_glob_is_named_once_with_its_entry() {
    let stderr = error(&format!(
        "{RULE}match = [\"web/**/*.{{ts,tsx\"]\npurpose = \"test\"\n"
    ));
    assert!(
        stderr.contains("In 'match' of entry 'r': invalid glob 'web/**/*.{ts,tsx': unclosed alternate group, missing '}'"),
        "{stderr}"
    );
    assert_eq!(stderr.matches("web/**/*.{ts,tsx").count(), 1, "{stderr}");
}

#[test]
fn an_unknown_value_suggests_the_value_meant() {
    for purpose in ["tests", "Test"] {
        let stderr = error(&format!(
            "{RULE}match = [\"src/**\"]\npurpose = \"{purpose}\"\n"
        ));
        assert!(
            stderr.contains("expected one of 'production', 'test', 'other'. Did you mean 'test'?"),
            "{stderr}"
        );
    }
}
