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
id = "queries"
match = ["app/highlights.scm"]
language = "unrecognized"
[[languages.corrections]]
id = "latex"
match = ["app/**", "tex/**"]
language = "TeX"
"#,
        &[
            ("app/highlights.scm", "(identifier) @variable\n"),
            ("app/x.cls", "\\ProvidesClass{x}\n"),
        ],
    );
    let summary = &document["summary"];
    assert_eq!(
        entry(summary, "queries"),
        &json!({"id": "queries", "stage": "correction", "files": 1, "code": 0})
    );
    assert_eq!(
        entry(summary, "latex"),
        &json!({
            "id": "latex", "stage": "correction", "files": 1, "code": 1,
            "preempted": [{"by": "queries", "files": 1}],
            "unused_patterns": ["tex/**"]
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

const MAPPINGS: &str = r#"version = 1
[[languages.mappings]]
id = "launchers"
match = ["bin/*"]
language = "JavaScript"
[[languages.mappings]]
id = "spec-as-text"
match = ["**/*.spec", "bin/*"]
language = "Plain Text"
"#;

const MAPPED: &[(&str, &str)] = &[
    ("bin/launch", "console.log(1)\n"),
    (
        "pkg/app.spec",
        "Name: app\nVersion: 1\n\n%description\nAn app.\n",
    ),
];

/// Feedback of 2026-10-25 on 0.11.1: a mapping of `*.spec` files, which the
/// counter already reads as RPM Specfile, counted nothing, and neither the
/// audit nor the verbose notes said so.
#[test]
fn mappings_report_their_files_and_the_files_the_counter_already_recognizes() {
    let summary = &measured(MAPPINGS, MAPPED)["summary"];
    assert_eq!(
        entry(summary, "launchers"),
        &json!({"id": "launchers", "stage": "mapping", "files": 1, "code": 1})
    );
    assert_eq!(
        entry(summary, "spec-as-text"),
        &json!({
            "id": "spec-as-text", "stage": "mapping", "files": 0, "code": 0,
            "preempted": [{"by": "builtin:recognition", "files": 1}, {"by": "launchers", "files": 1}],
            "unused_patterns": ["**/*.spec", "bin/*"]
        })
    );
    assert_eq!(summary["unused_entries"], json!(["spec-as-text"]));
}

#[test]
fn verbose_output_names_mappings_of_files_the_counter_already_recognizes() {
    let mut files = MAPPED.to_vec();
    files.push((".locmetryrc", MAPPINGS));
    let temp = project(&files);
    let root = temp.path().join("project");
    let note = "Language mappings that matched files the counter already recognizes: spec-as-text (1 file). A mapping applies only to unrecognized files; a '[[languages.corrections]]' entry changes the language of a recognized file.";
    for format in ["table", "markdown"] {
        let text = crate::common::stdout(run(&root, &["--format", format, "--verbose"]));
        assert!(text.contains(note), "{text}");
        // Mappings have no `except`, so the advice for entries that lost files does not apply.
        assert!(!text.contains("lost matching files"), "{text}");
    }
}

/// Personal mappings apply to every project, so one that matches nothing in
/// this project is expected and stays out of the project's audit.
#[test]
fn personal_mappings_stay_out_of_the_audit() {
    let temp = project(&[("pkg/app.spec", "Name: app\n")]);
    let home = temp.path().join("home");
    std::fs::create_dir_all(home.join("locmetry")).unwrap();
    std::fs::write(
        home.join("locmetry/config.toml"),
        "version = 1\n[[languages.mappings]]\nid = \"spec\"\nmatch = [\"**/*.spec\"]\nlanguage = \"Plain Text\"\n",
    )
    .unwrap();
    let output = locmetry()
        .current_dir(temp.path().join("project"))
        .env("XDG_CONFIG_HOME", &home)
        .args(["--format", "json"])
        .output()
        .unwrap();
    let document = crate::common::json(&output, 0);
    assert!(
        document["summary"].get("policy_entries").is_none(),
        "{document}"
    );
    assert!(
        document["summary"].get("unused_entries").is_none(),
        "{document}"
    );
}

#[test]
fn a_mapping_shares_the_id_space_of_project_entries() {
    let config = "version = 1\n[[rules]]\nid = \"x\"\nmatch = [\"a/**\"]\npurpose = \"production\"\ncategory = \"tooling\"\n[[languages.mappings]]\nid = \"x\"\nmatch = [\"bin/*\"]\nlanguage = \"JavaScript\"\n";
    let temp = project(&[(".locmetryrc", config), ("src/a.rs", "fn a() {}\n")]);
    let (success, _, stderr) = report(&temp.path().join("project"), &[]);
    assert!(!success);
    assert!(stderr.contains("Entry ID 'x' is used more than once. IDs are unique across overrides, rules, refinements and language mappings and corrections."), "{stderr}");
}

const FACADES: &str = r#"version = 1
[[rules]]
id = "typescript"
match = ["src/**"]
purpose = "production"
[[refinements]]
id = "typescript-logic"
purpose = "production"
match = ["src/**"]
content = '^\s*(if|for|while)\b'
category = "logic"
[[refinements]]
id = "kotlin-facades"
purpose = "production"
match = ["src/**"]
content = "from [\"']#kotlin/"
category = "facade"
reason = "Ported modules reach Kotlin through the import map"
"#;

const IMPORT: &str = "import { x } from \"#kotlin/x\";\n";

fn facades() -> tempfile::TempDir {
    project(&[
        (".locmetryrc", FACADES),
        ("src/facade.ts", &format!("{IMPORT}export const y = x;\n")),
        (
            "src/mixed.ts",
            &format!("{IMPORT}if (x) {{\n  run();\n}}\n"),
        ),
        (
            "src/comment.ts",
            &format!("{IMPORT}// if (x) {{ run(); }}\n"),
        ),
        ("src/plain.ts", "export const z = 1;\n"),
    ])
}

/// Only lines the counter counts as code are tested, and ordered refinements
/// still decide: a comment holding the keyword does not make a file logic.
#[test]
fn a_content_refinement_matches_code_lines_and_not_comments() {
    let temp = facades();
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    let category = |path: &str| file(&document, path)["category"].clone();
    assert_eq!(category("src/facade.ts"), "facade");
    assert_eq!(category("src/mixed.ts"), "logic");
    assert_eq!(category("src/comment.ts"), "facade");
    assert_eq!(category("src/plain.ts"), "production");
    let plain = file(&document, "src/plain.ts");
    assert!(plain.get("refinement").is_none());
    assert_eq!(file(&document, "src/mixed.ts")["rule"], "typescript");
}

#[test]
fn content_refinements_keep_purpose_totals() {
    let (with, without) = {
        let files = [
            ("src/a.ts", "import { x } from \"#kotlin/x\";\n"),
            ("src/b.ts", "if (a) {\n}\n"),
        ];
        let plain = FACADES.split("[[refinements]]").next().unwrap();
        (measured(FACADES, &files), measured(plain, &files))
    };
    assert_eq!(
        with["summary"]["source_purpose"],
        without["summary"]["source_purpose"]
    );
    assert_ne!(
        with["summary"]["source_categories"],
        without["summary"]["source_categories"]
    );
}

#[test]
fn content_refinements_respect_except_kind_and_leave_marked_and_unclaimed_files_alone() {
    let document = measured(
        "version = 1\n[[refinements]]\nid = \"looks\"\npurpose = \"production\"\nmatch = [\"**\"]\nexcept = [\"skip.go\"]\nkind = [\"source\"]\ncontent = \"package\"\ncategory = \"core\"\n",
        &[
            ("go.mod", "module demo\n\ngo 1.21\n"),
            ("main.go", "package main\n\nfunc main() {}\n"),
            ("skip.go", "package main\n\nvar y = 1\n"),
            ("gen.go", "// Code generated by tool. DO NOT EDIT.\n\npackage main\n\nvar x = 1\n"),
            ("notes/x.py", "package = 1\n"),
            ("README.md", "package\n"),
        ],
    );
    let rule = |path: &str| rule_of(&document, path);
    assert_eq!(rule("main.go"), "production:core@builtin:go-module");
    assert_eq!(
        rule("skip.go"),
        "production:implementation@builtin:go-module"
    );
    assert_eq!(
        rule("gen.go"),
        "production:generated@builtin:go-generated-header"
    );
    assert_eq!(rule("notes/x.py"), "other:unclaimed@fallback");
    assert!(file(&document, "README.md").get("refinement").is_none());
}

#[test]
fn a_notebook_does_not_match_a_content_refinement() {
    let notebook = r##"{"cells":[{"cell_type":"code","metadata":{},"outputs":[],"execution_count":null,"source":["print(1)\n"]}],"metadata":{},"nbformat":4,"nbformat_minor":5}"##;
    let document = measured(
        "version = 1\n[[rules]]\nid = \"nb\"\nmatch = [\"**\"]\npurpose = \"production\"\n[[refinements]]\nid = \"printing\"\npurpose = \"production\"\nmatch = [\"**\"]\ncontent = \"print\"\ncategory = \"scripts\"\n",
        &[("a.ipynb", notebook), ("b.py", "print(1)\n")],
    );
    assert_eq!(file(&document, "a.ipynb")["category"], "production");
    assert_eq!(file(&document, "b.py")["category"], "scripts");
}

#[test]
fn an_invalid_content_pattern_fails_loading_with_the_regex_message_and_a_hint() {
    let config = "version = 1\n[[refinements]]\nid = \"broken\"\npurpose = \"production\"\nmatch = [\"**\"]\ncontent = \"(\"\ncategory = \"x\"\n";
    let temp = project(&[(".locmetryrc", config), ("src/a.rs", "fn a() {}\n")]);
    let (success, _, stderr) = report(&temp.path().join("project"), &[]);
    assert!(!success);
    assert!(stderr.contains("refinement 'broken'"), "{stderr}");
    assert!(stderr.contains("unclosed group"), "{stderr}");
    assert!(stderr.contains("Rust regex syntax"), "{stderr}");
}

#[test]
fn explain_shows_the_code_line_a_content_refinement_matched() {
    let temp = facades();
    let root = temp.path().join("project");
    let output = locmetry()
        .current_dir(&root)
        .args(["explain", "src/mixed.ts"])
        .output()
        .unwrap();
    let document = crate::common::json(&output, 0);
    assert_eq!(
        document["result"]["decision"]["refinement"]["matched_line"],
        json!({"number": 2, "text": "if (x) {"})
    );
    let table = crate::common::stdout(run(
        &root,
        &["explain", "--format", "table", "src/mixed.ts"],
    ));
    assert!(table.contains("line 2"), "{table}");
    assert!(table.contains("if (x) {"), "{table}");
}

#[test]
fn the_audit_reports_files_a_content_refinement_left_out() {
    let temp = facades();
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    let summary = &document["summary"];
    assert_eq!(
        entry(summary, "kotlin-facades")["content_unmatched"],
        json!({"files": 1, "code": 1})
    );
    assert_eq!(
        entry(summary, "typescript-logic")["content_unmatched"],
        json!({"files": 3, "code": 4})
    );
    assert_eq!(entry(summary, "kotlin-facades")["files"], 2);
    assert_eq!(
        entry(summary, "kotlin-facades")["preempted"],
        json!([{"by": "typescript-logic", "files": 1}])
    );
}

#[test]
fn a_content_refinement_that_relabels_nothing_is_listed_as_unused() {
    let document = measured(
        "version = 1\n[[refinements]]\nid = \"never\"\npurpose = \"production\"\nmatch = [\"**\"]\ncontent = \"zzz\"\ncategory = \"x\"\n[[rules]]\nid = \"all\"\nmatch = [\"**\"]\npurpose = \"production\"\n",
        &[("a.ts", "export const a = 1;\n")],
    );
    assert_eq!(document["summary"]["unused_entries"], json!(["never"]));
    assert_eq!(
        entry(&document["summary"], "never")["content_unmatched"],
        json!({"files": 1, "code": 1})
    );
}

fn explained_line(temp: &tempfile::TempDir, path: &str) -> Value {
    let output = locmetry()
        .current_dir(temp.path().join("project"))
        .args(["explain", path])
        .output()
        .unwrap();
    crate::common::json(&output, 0)["result"]["decision"]["refinement"]["matched_line"].clone()
}

#[test]
fn content_refinements_apply_through_every_record_builder() {
    let temp = facades();
    let root = temp.path().join("project");
    let config = locmetry::Policy::from_toml(FACADES).unwrap();
    let scan = locmetry::scan(&root, &config).unwrap();
    let rows = locmetry::file_records(&scan.payload, &config, &root).unwrap();
    let row = rows.iter().find(|row| row.path == "src/mixed.ts").unwrap();
    assert_eq!(row.classification.category, "logic");
}

#[test]
fn a_matched_line_is_capped_in_json_and_in_the_table() {
    let long = format!("const a = \"{}\";\n", "é".repeat(500));
    let temp = project(&[
        (
            ".locmetryrc",
            "version = 1\n[[rules]]\nid = \"all\"\nmatch = [\"**\"]\npurpose = \"production\"\n[[refinements]]\nid = \"consts\"\npurpose = \"production\"\nmatch = [\"**\"]\ncontent = \"const\"\ncategory = \"x\"\n",
        ),
        ("a.ts", &long),
    ]);
    let line = explained_line(&temp, "a.ts");
    let text = line["text"].as_str().unwrap();
    assert_eq!(text.chars().count(), 201, "{text}");
    assert!(text.ends_with('…'));
    assert!(text.starts_with("const a = \"éé"));
    let table = crate::common::stdout(run(
        &temp.path().join("project"),
        &["explain", "--format", "table", "a.ts"],
    ));
    let row = table
        .lines()
        .find(|l| l.contains("Refinement line"))
        .unwrap();
    assert!(row.contains('…') && row.chars().count() < 260, "{row}");
}

#[test]
fn line_numbers_follow_the_counter_for_crlf_files_and_lone_carriage_returns() {
    let temp = project(&[
        (".locmetryrc", FACADES),
        (
            "src/crlf.ts",
            "// note\r\nconst a = 1;\r\nif (a) {\r\n}\r\n",
        ),
        ("src/lone.ts", "const a = 1;\rconst b = 2;\nif (a) {\n}\n"),
    ]);
    assert_eq!(
        explained_line(&temp, "src/crlf.ts"),
        json!({"number": 3, "text": "if (a) {"})
    );
    assert_eq!(
        explained_line(&temp, "src/lone.ts"),
        json!({"number": 2, "text": "if (a) {"})
    );
}

#[test]
fn a_glob_whose_files_failed_the_content_is_not_an_unused_pattern() {
    let document = measured(
        "version = 1\n[[refinements]]\nid = \"never\"\npurpose = \"production\"\nmatch = [\"**\", \"gone/**\"]\ncontent = \"zzz\"\ncategory = \"x\"\n[[rules]]\nid = \"all\"\nmatch = [\"**\"]\npurpose = \"production\"\n",
        &[("a.ts", "export const a = 1;\n")],
    );
    assert_eq!(
        entry(&document["summary"], "never")["unused_patterns"],
        json!(["gone/**"])
    );
}

#[test]
fn verbose_notes_blame_the_content_expression_not_the_glob() {
    let temp = project(&[
        (
            ".locmetryrc",
            "version = 1\n[[rules]]\nid = \"all\"\nmatch = [\"**\"]\npurpose = \"production\"\n[[refinements]]\nid = \"never\"\npurpose = \"production\"\nmatch = [\"**\"]\ncontent = \"zzz\"\ncategory = \"x\"\n",
        ),
        ("a.ts", "export const a = 1;\n"),
    ]);
    let root = temp.path().join("project");
    for format in ["table", "markdown"] {
        let text = crate::common::stdout(run(&root, &["--format", format, "--verbose"]));
        assert!(
            text.contains(
                "Entries whose 'content' expression matched no code line in files their globs selected: never (1 file)."
            ),
            "{text}"
        );
        assert!(!text.contains("decided no file: never"), "{text}");
        assert!(!text.contains("Patterns that decided no file"), "{text}");
    }
}
