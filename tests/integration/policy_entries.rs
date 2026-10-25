//! Reviewable policy entries (docs/proposals/archive/policy-entry-review.md): `except`
//! lists on classification entries, `summary.policy_entries`, which counts
//! what each entry decided and names what it lost or replaced, and
//! refinements, which change only the category of decided files.
use crate::cli::*;
use crate::common::locmetry;
use serde_json::{json, Value};

fn rule_of(document: &Value, path: &str) -> String {
    let file = file(document, path);
    format!("{}:{}@{}", file["purpose"], file["category"], file["rule"]).replace('"', "")
}

const LAWS: &str = r#"version = 1
[[overrides]]
id = "agreement-laws"
match = ["**/*AgreementTest.java"]
except = ["**/FieldAgreementTest.java"]
purpose = "test"
category = "agreement-law"
"#;

#[test]
fn an_excepted_file_falls_through_to_the_next_stage() {
    let document = measured(
        LAWS,
        &[
            ("src/test/java/RuleAgreementTest.java", "class A {}\n"),
            ("src/test/java/FieldAgreementTest.java", "class B {}\n"),
        ],
    );
    assert_eq!(
        rule_of(&document, "src/test/java/RuleAgreementTest.java"),
        "test:agreement-law@agreement-laws"
    );
    assert_eq!(
        rule_of(&document, "src/test/java/FieldAgreementTest.java"),
        "test:test@builtin:test-directory"
    );
    assert_eq!(
        document["metadata"]["config"]["overrides"][0]["except"][0],
        "**/FieldAgreementTest.java"
    );
}

#[test]
fn explain_shows_the_stage_an_excepted_file_falls_through_to() {
    let temp = project(&[
        (".locmetryrc", LAWS),
        ("src/test/java/FieldAgreementTest.java", "class B {}\n"),
    ]);
    let output = locmetry()
        .current_dir(temp.path().join("project"))
        .args(["explain", "src/test/java/FieldAgreementTest.java"])
        .output()
        .unwrap();
    let document = crate::common::json(&output, 0);
    assert_eq!(document["result"]["decision"]["stage"], "convention");
}

#[test]
fn an_empty_or_invalid_except_list_is_rejected() {
    for (except, message) in [
        ("[]", "needs a nonempty except array"),
        ("[\"a/{b\"]", "In 'except' of entry 'app'"),
    ] {
        let config =
            format!("version = 1\n[[rules]]\nid = \"app\"\nmatch = [\"src/**\"]\nexcept = {except}\npurpose = \"production\"\n");
        let temp = project(&[(".locmetryrc", &config), ("src/a.rs", "fn a() {}\n")]);
        let (success, _, stderr) = report(&temp.path().join("project"), &[]);
        assert!(!success);
        assert!(stderr.contains(message), "{stderr}");
    }
}

/// The entry of `id` in `summary.policy_entries`.
fn entry<'a>(summary: &'a Value, id: &str) -> &'a Value {
    summary["policy_entries"]
        .as_array()
        .unwrap_or_else(|| panic!("no policy_entries in {summary}"))
        .iter()
        .find(|entry| entry["id"] == id)
        .unwrap_or_else(|| panic!("no entry {id} in {summary}"))
}

/// a12-rulekit's draft glob was case-sensitive and missed `src/test`; only
/// the entry's small count beside its folder's size revealed it.
#[test]
fn entry_counts_reveal_an_under_matching_glob() {
    let config = "version = 1\n[[overrides]]\nid = \"differential\"\nmatch = [\"**/src/*Test/**\"]\npurpose = \"test\"\ncategory = \"differential\"\n";
    let temp = project(&[
        (".locmetryrc", config),
        ("src/test/java/A.java", "class A {}\n"),
        ("src/test/java/B.java", "class B {}\n"),
        ("src/integrationTest/java/C.java", "class C {\n}\n"),
    ]);
    let (success, headline, stderr) = headline(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    assert_eq!(
        entry(&headline["summary"], "differential"),
        &json!({
            "id": "differential", "stage": "override", "purpose": "test", "files": 1, "code": 2,
            "displaced": [{"decision": "test:test", "files": 1, "code": 2}]
        })
    );
}

#[test]
fn preempted_names_the_entry_or_convention_that_decided_first() {
    let document = measured(
        r#"version = 1
[[rules]]
id = "theory-fixtures"
match = ["Conformance/Fixtures/**"]
purpose = "test"
category = "theory-fixture"
[[rules]]
id = "theory-locks"
match = ["Conformance/**"]
purpose = "test"
category = "theory-lock"
[[rules]]
id = "app"
match = ["src/**"]
purpose = "production"
"#,
        &[
            ("Conformance/Fixtures/F.lean", "def f := 1\n"),
            ("Conformance/L1.lean", "theorem a : True := trivial\n"),
            ("Conformance/L2.lean", "theorem b : True := trivial\n"),
            ("src/a.ts", "export const a = 1;\n"),
            ("src/a.test.ts", "test();\n"),
        ],
    );
    let summary = &document["summary"];
    let locks = entry(summary, "theory-locks");
    assert_eq!(locks["files"], 2);
    assert_eq!(
        locks["preempted"],
        json!([{"by": "theory-fixtures", "files": 1}])
    );
    assert_eq!(
        entry(summary, "app")["preempted"],
        json!([{"by": "builtin:js-test-file", "files": 1}])
    );
    assert!(entry(summary, "theory-fixtures").get("preempted").is_none());
}

/// A glob counts as used only where its entry decided the file, as whole
/// entries count for `unused_entries`.
#[test]
fn unused_patterns_and_exceptions_are_listed_per_entry() {
    let document = measured(
        r#"version = 1
[[rules]]
id = "tools"
match = ["tools/**", "scripts/**", "src/**"]
except = ["tools/old.sh", "attic/**"]
purpose = "production"
category = "tooling"
"#,
        &[
            ("tools/run.sh", "echo run\n"),
            ("tools/old.sh", "echo old\n"),
            ("src/a.test.ts", "test();\n"),
        ],
    );
    let tools = entry(&document["summary"], "tools");
    assert_eq!(tools["files"], 1);
    assert_eq!(tools["unused_patterns"], json!(["scripts/**", "src/**"]));
    assert_eq!(tools["unused_exceptions"], json!(["attic/**"]));
}

const DEMO: &str = r#"version = 1
[[overrides]]
id = "demo-tests"
match = ["src/demo/**"]
purpose = "test"
category = "demo"
[[rules]]
id = "tools"
match = ["tools/**"]
purpose = "production"
category = "tooling"
"#;

fn demo() -> tempfile::TempDir {
    project(&[
        (".locmetryrc", DEMO),
        ("package.json", "{}\n"),
        (
            "src/demo/Host.ts",
            "export const host = 1;\nexport const port = 2;\n",
        ),
        ("src/demo/a.test.ts", "test();\n"),
        ("tools/x.sh", "echo x\n"),
    ])
}

/// bpmn-lean-experiment's broad override turned production demo hosts into
/// tests; `displaced` shows what each entry replaced, and claiming code no
/// stage decides replaces nothing.
#[test]
fn displaced_names_the_decisions_an_entry_replaced() {
    let temp = demo();
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    let summary = &document["summary"];
    assert_eq!(
        entry(summary, "demo-tests")["displaced"],
        json!([
            {"decision": "production:implementation", "files": 1, "code": 2},
            {"decision": "test:test", "files": 1, "code": 1}
        ])
    );
    assert!(entry(summary, "tools").get("displaced").is_none());
}

#[test]
fn verbose_output_names_entries_that_change_a_purpose() {
    let temp = demo();
    let root = temp.path().join("project");
    let note = "Entries that changed the purpose a later stage would give: demo-tests (Production to Test, 2 code lines).";
    for format in ["table", "markdown"] {
        let text = crate::common::stdout(run(&root, &["--format", format, "--verbose"]));
        assert!(text.contains(note), "{text}");
    }
    let plain = crate::common::stdout(run(&root, &["--format", "table"]));
    assert!(!plain.contains("Entries that changed"), "{plain}");
}

#[test]
fn corrections_report_their_files_and_what_an_earlier_correction_took() {
    let document = measured(
        r#"version = 1
[[languages.corrections]]
id = "proguard"
match = ["app/proguard-rules.pro"]
language = "unrecognized"
[[languages.corrections]]
id = "helm"
match = ["app/**", "chart/**"]
language = "Go HTML"
"#,
        &[
            ("app/proguard-rules.pro", "-keep class a.B\n"),
            ("app/x.tpl", "{{ .A }}\n"),
        ],
    );
    let summary = &document["summary"];
    assert_eq!(
        entry(summary, "proguard"),
        &json!({"id": "proguard", "stage": "correction", "files": 1, "code": 0})
    );
    assert_eq!(
        entry(summary, "helm"),
        &json!({
            "id": "helm", "stage": "correction", "files": 1, "code": 1,
            "preempted": [{"by": "proguard", "files": 1}],
            "unused_patterns": ["chart/**"]
        })
    );
}

#[test]
fn a_measurement_without_entries_omits_policy_entries() {
    let temp = project(&[("src/a.rs", "fn a() {}\n")]);
    let (success, headline, stderr) = headline(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    assert!(headline["summary"].get("policy_entries").is_none());
}

/// Folder members share one policy, so its entries are counted once over all
/// folders, and a glob is unused only when it is unused in every folder;
/// independent roots record their own policy's entries.
#[test]
fn workspaces_count_shared_entries_once_and_member_entries_per_member() {
    let config = "version = 1\n[[rules]]\nid = \"tools\"\nmatch = [\"a/tools/**\", \"b/tools/**\", \"c/**\"]\npurpose = \"production\"\ncategory = \"tooling\"\n";
    let temp = project(&[
        (".locmetryrc", config),
        ("a/tools/x.sh", "echo x\n"),
        ("b/tools/y.sh", "echo y\necho z\n"),
        (
            "ws.toml",
            "version = 1\nproject = \".\"\nfolders = [\"a\", \"b\"]\n",
        ),
    ]);
    let root = temp.path().join("project");
    let (success, shared, stderr) = headline(&root, &["workspace", "--workspace-file", "ws.toml"]);
    assert!(success, "{stderr}");
    assert_eq!(
        entry(&shared["summary"], "tools"),
        &json!({
            "id": "tools", "stage": "rule", "purpose": "production", "files": 2, "code": 3,
            "unused_patterns": ["c/**"]
        })
    );

    let rule = |folder: &str| {
        format!("version = 1\n[[rules]]\nid = \"tools\"\nmatch = [\"tools/**\"]\npurpose = \"production\"\ncategory = \"{folder}\"\n")
    };
    let (web, api) = (rule("web"), rule("api"));
    let temp = project(&[
        ("web/.locmetryrc", &web),
        ("web/tools/x.sh", "echo x\n"),
        ("api/.locmetryrc", &api),
        ("api/tools/y.sh", "echo y\necho z\n"),
    ]);
    let (success, independent, stderr) = report(
        &temp.path().join("project"),
        &["workspace", "--root", "web", "--root", "api"],
    );
    assert!(success, "{stderr}");
    assert!(independent["summary"].get("policy_entries").is_none());
    let members = independent["metadata"]["members"].as_array().unwrap();
    let api = members.iter().find(|m| m["name"] == "api").unwrap();
    assert_eq!(
        api["policy_entries"],
        json!([{"id": "tools", "stage": "rule", "purpose": "production", "files": 1, "code": 2}])
    );
}

const REFINED: &str = r#"version = 1
[[refinements]]
id = "property-laws"
purpose = "test"
match = ["src/laws/**"]
except = ["src/laws/Field*"]
category = "property-law"
reason = "Property-based laws"
[[refinements]]
id = "all-tests"
purpose = "test"
match = ["src/**"]
category = "suite"
"#;

fn refined() -> tempfile::TempDir {
    project(&[
        (".locmetryrc", REFINED),
        ("package.json", "{}\n"),
        ("src/laws/Core.ts", "export const core = 1;\n"),
        ("src/laws/a.test.ts", "test();\ntest();\n"),
        ("src/laws/Field.test.ts", "test();\n"),
    ])
}

/// A refinement relabels conventional tests without an override: purpose,
/// `rule`, totals and module credit stay, and files of another purpose under
/// the same glob are untouched.
#[test]
fn a_refinement_changes_only_the_category_of_files_of_its_purpose() {
    let temp = refined();
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    let law = file(&document, "src/laws/a.test.ts");
    assert_eq!(
        (
            &law["purpose"],
            &law["category"],
            &law["rule"],
            &law["refinement"]
        ),
        (
            &json!("test"),
            &json!("property-law"),
            &json!("builtin:js-test-file"),
            &json!("property-laws")
        )
    );
    let field = file(&document, "src/laws/Field.test.ts");
    assert_eq!(
        (&field["category"], &field["refinement"]),
        (&json!("suite"), &json!("all-tests"))
    );
    let core = file(&document, "src/laws/Core.ts");
    assert_eq!(
        rule_of(&document, "src/laws/Core.ts"),
        "production:implementation@builtin:npm-module"
    );
    assert!(core.get("refinement").is_none());
    let summary = &document["summary"];
    assert_eq!(summary["source_purpose"]["test"]["code"], 3);
    assert_eq!(summary["source_categories"]["test:property-law"]["code"], 2);
    assert_eq!(summary["build_modules"]["npm"]["code"], 1);
}

#[test]
fn refinements_leave_unclaimed_and_generated_files_alone() {
    let document = measured(
        "version = 1\n[[refinements]]\nid = \"everything\"\npurpose = \"production\"\nmatch = [\"**\"]\ncategory = \"core\"\n[[refinements]]\nid = \"parked\"\npurpose = \"other\"\nmatch = [\"**\"]\ncategory = \"parked\"\n",
        &[
            ("go.mod", "module demo\n\ngo 1.21\n"),
            ("main.go", "package main\n\nfunc main() {}\n"),
            ("gen.go", "// Code generated by tool. DO NOT EDIT.\n\npackage main\n\nvar x = 1\n"),
            ("notes/x.py", "print(1)\n"),
        ],
    );
    assert_eq!(
        rule_of(&document, "main.go"),
        "production:core@builtin:go-module"
    );
    assert_eq!(
        rule_of(&document, "gen.go"),
        "production:generated@builtin:go-generated-header"
    );
    assert_eq!(rule_of(&document, "notes/x.py"), "other:unclaimed@fallback");
    let summary = &document["summary"];
    assert_eq!(summary["build_modules"]["go"]["code"], 4);
    assert_eq!(
        entry(summary, "everything")["preempted"],
        json!([{"by": "builtin:go-generated-header", "files": 1}])
    );
    assert_eq!(summary["unused_entries"], json!(["parked"]));
}

#[test]
fn refinements_report_what_they_relabeled_and_what_an_earlier_one_took() {
    let temp = refined();
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    let summary = &document["summary"];
    assert_eq!(
        entry(summary, "property-laws"),
        &json!({
            "id": "property-laws", "stage": "refinement", "purpose": "test", "files": 1, "code": 2,
            "displaced": [{"decision": "test:test", "files": 1, "code": 2}]
        })
    );
    assert_eq!(
        entry(summary, "all-tests")["preempted"],
        json!([{"by": "property-laws", "files": 1}])
    );
    assert_eq!(
        document["metadata"]["config"]["refinements"][0]["id"],
        "property-laws"
    );
    let root = temp.path().join("project");
    let verbose = crate::common::stdout(run(&root, &["--format", "table", "--verbose"]));
    assert!(!verbose.contains("Entries that changed"), "{verbose}");
}

#[test]
fn explain_names_the_refinement_and_its_reason() {
    let temp = refined();
    let root = temp.path().join("project");
    let output = locmetry()
        .current_dir(&root)
        .args(["explain", "src/laws/a.test.ts"])
        .output()
        .unwrap();
    let document = crate::common::json(&output, 0);
    assert_eq!(
        document["result"]["decision"]["refinement"],
        json!({"id": "property-laws", "matched_patterns": ["src/laws/**"], "reason": "Property-based laws"})
    );
    assert_eq!(document["result"]["decision"]["stage"], "convention");
    let table = crate::common::stdout(run(
        &root,
        &["explain", "--format", "table", "src/laws/a.test.ts"],
    ));
    assert!(table.contains("property-laws"), "{table}");
    assert!(table.contains("Property-based laws"), "{table}");
}

#[test]
fn invalid_refinements_are_rejected() {
    for (config, message) in [
        ("[[refinements]]\nid = \"a\"\npurpose = \"test\"\nmatch = [\"**\"]\n", "missing field 'category'"),
        ("[[refinements]]\nid = \"a\"\nmatch = [\"**\"]\ncategory = \"x\"\n", "missing field 'purpose'"),
        (
            "[categories]\nlaw = \"Laws\"\n[[refinements]]\nid = \"a\"\npurpose = \"test\"\nmatch = [\"**\"]\ncategory = \"lwa\"\n",
            "Entry 'a' uses category 'lwa'",
        ),
        (
            "[[rules]]\nid = \"a\"\nmatch = [\"x/**\"]\npurpose = \"test\"\n[[refinements]]\nid = \"a\"\npurpose = \"test\"\nmatch = [\"**\"]\ncategory = \"x\"\n",
            "Entry ID 'a' is used more than once",
        ),
    ] {
        let config = format!("version = 1\n{config}");
        let temp = project(&[(".locmetryrc", &config), ("src/a.rs", "fn a() {}\n")]);
        let (success, _, stderr) = report(&temp.path().join("project"), &[]);
        assert!(!success);
        assert!(stderr.contains(message), "{stderr}");
    }
}

/// Feedback of 2026-10-25: a partly dead entry and files an earlier entry took
/// from a later one appeared only in JSON, so a mistyped glob went unnoticed in
/// the table a reviewer was reading.
#[test]
fn verbose_output_names_unused_globs_and_entries_an_earlier_entry_preempted() {
    let temp = project(&[
        (
            ".locmetryrc",
            r#"version = 1
[[rules]]
id = "locks"
match = ["ref/locks/**"]
purpose = "test"
category = "lock"
[[rules]]
id = "reference"
match = ["ref/**", "gone/**"]
except = ["ref/attic/**"]
purpose = "test"
category = "reference"
"#,
        ),
        ("ref/locks/a.py", "x = 1\n"),
        ("ref/b.py", "x = 1\n"),
    ]);
    let root = temp.path().join("project");
    let notes = [
        "Patterns that decided no file: reference (gone/**).",
        "Exceptions that left out no file: reference (ref/attic/**).",
        "Entries that lost matching files to an earlier entry: reference (1 file to locks). An 'except' list on the earlier entry states such a split without relying on order.",
    ];
    for format in ["table", "markdown"] {
        let text = crate::common::stdout(run(&root, &["--format", format, "--verbose"]));
        for note in notes {
            assert!(text.contains(note), "{note}\n{text}");
        }
    }
    let plain = crate::common::stdout(run(&root, &["--format", "table"]));
    assert!(!plain.contains("Patterns that decided"), "{plain}");
}
