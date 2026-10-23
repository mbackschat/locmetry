use crate::cli::*;
use serde_json::{json, Value};
use std::fs;

/// Unclaimed source is located, not just counted: folders partition it,
/// single-child chains collapse to the deepest folder, and the largest
/// folder splits while the list stays short.
#[test]
fn unclaimed_source_is_located_by_folder() {
    let temp = project(&[
        (
            "docs/planning/m1/evidence/a.js",
            "let a = 1;\nlet b = 2;\nlet c = 3;\n",
        ),
        ("docs/planning/m1/evidence/b.js", "let d = 1;\nlet e = 2;\n"),
        ("docs/guides/ex/c.js", "let f = 1;\n"),
        ("tools/x.py", "x = 1\n"),
        ("root.py", "y = 1\n"),
    ]);
    let (success, document, stderr) = headline(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    assert_eq!(
        document["summary"]["unclaimed_folders"],
        json!([
            {"folder": "docs/planning/m1/evidence", "files": 2, "code": 5},
            {"folder": ".", "files": 1, "code": 1},
            {"folder": "docs/guides/ex", "files": 1, "code": 1},
            {"folder": "tools", "files": 1, "code": 1}
        ])
    );
    let output = run(&temp.path().join("project"), &["--format", "markdown"]);
    let markdown = String::from_utf8(output.stdout).unwrap();
    assert!(
        markdown.contains("| docs/planning/m1/evidence | 2 | 5 |"),
        "{markdown}"
    );
}

/// Review lists only source no stage decided: code an entry or convention
/// deliberately assigns to `other` is counted as other but not
/// reported as still needing a decision.
#[test]
fn review_lists_only_unclaimed_source() {
    const PARKED: &str = "version = 1\n[[rules]]\nid = \"parked\"\nmatch = [\"proto/**\"]\npurpose = \"other\"\ncategory = \"prototype\"\n";
    let temp = project(&[
        (".locmetryrc", PARKED),
        ("proto/model.py", "a = 1\nb = 2\n"),
        ("gradlew", "#!/bin/sh\necho build\n"),
        ("loose/y.py", "y = 1\n"),
    ]);
    let root = temp.path().join("project");
    let (success, document, stderr) = report(&root, &[]);
    assert!(success, "{stderr}");
    assert_eq!(document["summary"]["source_purpose"]["other"]["files"], 3);
    assert_eq!(
        document["summary"]["unclaimed_folders"],
        json!([{"folder": "loose", "files": 1, "code": 1}])
    );
    let reviewed: Vec<&str> = document["review"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["path"].as_str().unwrap())
        .collect();
    assert_eq!(reviewed, ["loose/y.py"]);

    let table = String::from_utf8(run(&root, &["--verbose", "--format", "table"]).stdout).unwrap();
    assert!(
        table.contains(
            " Unclaimed source: 1 code line in 1 file, here:\n   loose  1 code line  1 file\n"
        ),
        "{table}"
    );
    let markdown = String::from_utf8(run(&root, &["--format", "markdown"]).stdout).unwrap();
    let review = &markdown[markdown.find("## Source requiring review").unwrap()..];
    assert!(review.contains("| loose/y.py |"), "{review}");
    assert!(
        !review.contains("proto/model.py") && !review.contains("gradlew"),
        "{review}"
    );

    fs::remove_dir_all(root.join("loose")).unwrap();
    let table = String::from_utf8(run(&root, &["--verbose", "--format", "table"]).stdout).unwrap();
    assert!(!table.contains("Unclaimed source"), "{table}");
    // The documented next step stays readable when it is done: an empty list, not a missing key.
    let (_, document, _) = headline(&root, &[]);
    assert_eq!(document["summary"]["unclaimed_folders"], json!([]));
}

/// The Markdown folder table keeps the path hierarchy: a folder's subfolders
/// follow it directly, even when a sibling name sorts between them as a plain
/// string (`-` sorts before `/`).
#[test]
fn markdown_folder_table_keeps_subfolders_under_their_parent() {
    let temp = project(&[
        ("web/src/a.ts", "export const a = 1;\n"),
        ("web-admin/b.ts", "export const b = 1;\n"),
    ]);
    let markdown =
        String::from_utf8(run(&temp.path().join("project"), &["--format", "markdown"]).stdout)
            .unwrap();
    let table = &markdown[markdown.find("## Folder breakdown").unwrap()..];
    assert!(
        is_ascending(&positions(
            table,
            &["| web |", "| web/src |", "| web-admin |"]
        )),
        "{table}"
    );
}

/// Git submodules are often other teams' code; locmetry points at them with
/// their size, read from `.gitmodules` without invoking Git, and leaves the
/// scope decision to the project.
#[test]
fn git_submodules_are_reported_until_excluded() {
    let temp = project(&[
        (
            ".gitmodules",
            r#"[submodule "lib"]
path = "vendor/lib" ; inline comment
url = https://example.com/lib.git
[submodule "gone"]
path = not-checked-out
[include]
path = src
"#,
        ),
        ("vendor/lib/a.py", "a = 1\nb = 2\n"),
        ("src/main.py", "x = 1\n"),
    ]);
    let root = temp.path().join("project");
    let (success, document, stderr) = headline(&root, &[]);
    assert!(success, "{stderr}");
    assert_eq!(
        document["summary"]["submodules"],
        json!([{"path": "vendor/lib", "files": 1, "code": 2}])
    );
    let table = String::from_utf8(run(&root, &["--verbose", "--format", "table"]).stdout).unwrap();
    assert!(
        table.contains(" Git submodules: vendor/lib (2 code lines). If they are not this project's code, exclude them with '--exclude' or 'scope.exclude'.\n"),
        "{table}"
    );
    let (_, document, _) = headline(&root, &["--exclude", "vendor/lib/**"]);
    assert!(document["summary"].get("submodules").is_none());
    let table = String::from_utf8(
        run(
            &root,
            &[
                "--verbose",
                "--format",
                "table",
                "--exclude",
                "vendor/lib/**",
            ],
        )
        .stdout,
    )
    .unwrap();
    assert!(!table.contains("Git submodules"), "{table}");
}

/// An entry that decides no file is a typo or is always preempted by an
/// earlier stage, such as a rule losing to a test convention; reports name it
/// without failing, since an entry may be empty in some checkouts.
#[test]
fn entries_that_decide_no_file_are_reported() {
    let config = "version = 1\n[[rules]]\nid = \"used\"\nmatch = [\"src/**\"]\npurpose = \"production\"\n[[rules]]\nid = \"ghost\"\nmatch = [\"nowhere/**\"]\npurpose = \"production\"\n[[rules]]\nid = \"shadowed\"\nmatch = [\"**/*.test.ts\"]\npurpose = \"production\"\n";
    let temp = project(&[
        (".locmetryrc", config),
        ("src/a.ts", "export const a = 1;\n"),
        ("src/a.test.ts", "test('a', () => {});\n"),
    ]);
    let root = temp.path().join("project");
    let (success, document, stderr) = headline(&root, &[]);
    assert!(success, "{stderr}");
    assert_eq!(
        document["summary"]["unused_entries"],
        json!(["ghost", "shadowed"])
    );
    let table = String::from_utf8(run(&root, &["--verbose", "--format", "table"]).stdout).unwrap();
    assert!(
        table.contains(" Entries that decided no file: ghost, shadowed.\n"),
        "{table}"
    );
}

/// An exclusion that excludes nothing is usually a folder written without
/// `/**` or a misspelled path; reports name it without failing. Default
/// exclusions and the configuration's own path are left out, because they
/// routinely match nothing, here the hidden `.locmetryrc` with `hidden = false`.
#[test]
fn exclusions_that_exclude_nothing_are_reported() {
    let config = "version = 1\n[scope]\nhidden = false\nexclude = [\"vendor/**\", \"third_party\", \"**/*.gen.ts\", \"vendor/**\", \"build/**\"]\n";
    let temp = project(&[
        (".locmetryrc", config),
        (".ignore", "build/\n"),
        ("src/a.ts", "export const a = 1;\n"),
        ("vendor/lib.js", "var b = 1;\n"),
        ("third_party/c.ts", "let c = 1;\n"),
        ("build/out.js", "var d = 1;\n"),
        ("dist/e.ts", "let e = 1;\n"),
    ]);
    let root = temp.path().join("project");
    let (success, document, stderr) =
        headline(&root, &["--exclude", "dist/**", "--exclude", "missing/**"]);
    assert!(success, "{stderr}");
    assert_eq!(
        document["summary"]["unused_exclusions"],
        json!(["third_party", "**/*.gen.ts", "build/**", "missing/**"])
    );
    let table = String::from_utf8(run(&root, &["--verbose", "--format", "table"]).stdout).unwrap();
    assert!(
        table.contains(" Exclusions that excluded nothing: third_party, **/*.gen.ts, build/**. Exclude a folder as '<folder>/**'. An ignore file may already hide the path.\n"),
        "{table}"
    );
    let markdown = String::from_utf8(run(&root, &["--format", "markdown"]).stdout).unwrap();
    assert!(
        markdown.contains("Exclusions that excluded nothing: third_party"),
        "{markdown}"
    );
    let (_, document, _) = headline(&root, &["--no-user-config"]);
    assert_eq!(
        document["summary"]["unused_exclusions"],
        json!(["third_party", "**/*.gen.ts", "build/**"])
    );
}

/// Categories split each purpose; the terminal table shows them on request,
/// one row per `purpose:category`, so it stays one screen by default.
#[test]
fn categories_table_is_shown_on_request() {
    let temp = project(&[
        (".locmetryrc", "version = 1\n[[rules]]\nid = \"tools\"\nmatch = [\"tools/**\"]\npurpose = \"production\"\ncategory = \"tooling\"\n"),
        ("src/a.rs", "fn a() {}\nfn b() {}\n"),
        ("tools/t.py", "x = 1\n"),
        ("tests/t.rs", "fn t() {}\n"),
    ]);
    let root = temp.path().join("project");
    let plain = String::from_utf8(run(&root, &["--format", "table"]).stdout).unwrap();
    assert!(!plain.contains("Category"), "{plain}");
    let table =
        String::from_utf8(run(&root, &["--format", "table", "--show-categories"]).stdout).unwrap();
    let expected = "
 Category            Files  Lines  Code
 ──────────────────────────────────────
 other:unclaimed         1      2     2
 production:tooling      1      1     1
 test:test               1      1     1
 Rows of one purpose add up to that purpose's Code, and all rows add up to the total Code.
";
    assert!(table.contains(expected), "{table}");
}

/// Readers see which build systems decided production and how much code each
/// one claimed. A module whose layout covers no production source is left out
/// of the summary but stays in the Markdown module list, where 0 hints at a
/// misleading build file.
#[test]
fn reports_name_the_build_modules_that_decided_production() {
    let temp = project(&[
        ("backend/build.gradle.kts", "plugins { kotlin(\"jvm\") }\n"),
        ("backend/src/main/kotlin/A.kt", "fun a() = 1\nfun b() = 2\n"),
        ("backend/src/test/kotlin/ATest.kt", "fun t() = 1\n"),
        ("web/package.json", "{}\n"),
        ("web/src/a.ts", "export const a = 1;\n"),
        ("tools/package.json", "{}\n"),
        ("tools/lint.js", "let x = 1;\n"),
        ("legacy/pom.xml", "<project/>\n"),
    ]);
    let root = temp.path().join("project");
    let (success, document, stderr) = headline(&root, &[]);
    assert!(success, "{stderr}");
    assert_eq!(
        document["summary"]["build_modules"],
        json!({
            "gradle": {"modules": 1, "code": 2},
            "npm": {"modules": 1, "code": 1}
        })
    );
    let table = String::from_utf8(run(&root, &["--verbose", "--format", "table"]).stdout).unwrap();
    assert!(
        table.contains(
            " Production from build modules: 2 code lines in 1 'gradle' module, 1 code line in 1 'npm' module.\n"
        ),
        "{table}"
    );
    let markdown = String::from_utf8(run(&root, &["--format", "markdown"]).stdout).unwrap();
    let section = &markdown[markdown.find("## Build modules").expect(&markdown)..];
    for row in [
        "| gradle | 1 | 2 |",
        "| npm | 1 | 1 |",
        "| backend | gradle | 2 |",
        "| tools | npm | 0 |",
        "| legacy | maven | 0 |",
    ] {
        assert!(section.contains(row), "{row} in {section}");
    }

    fs::write(
        root.join(".locmetryrc"),
        "version = 1\n[classification]\nbuiltin_modules = false\n",
    )
    .unwrap();
    let (_, document, _) = headline(&root, &[]);
    assert!(document["summary"].get("build_modules").is_none());
    let table = String::from_utf8(run(&root, &["--verbose", "--format", "table"]).stdout).unwrap();
    assert!(!table.contains("Build modules"), "{table}");
    let markdown = String::from_utf8(run(&root, &["--format", "markdown"]).stdout).unwrap();
    assert!(!markdown.contains("## Build modules"), "{markdown}");
}

/// Module layouts claim production without review, so the largest files they
/// claimed are named where generated or vendored code would hide; a file an
/// entry or convention decided is not listed.
#[test]
fn reports_name_the_largest_files_build_modules_claimed() {
    const POLICY: &str = "version = 1\n[[overrides]]\nid = \"generated\"\nmatch = [\"web/src/gen.ts\"]\npurpose = \"other\"\ncategory = \"generated\"\nreason = \"generator header\"\n";
    let lines = |count: usize| "let a = 1;\n".repeat(count);
    let temp = project(&[
        (".locmetryrc", POLICY),
        ("backend/build.gradle.kts", "plugins { kotlin(\"jvm\") }\n"),
        ("backend/src/main/kotlin/A.kt", "fun a() = 1\nfun b() = 2\n"),
        ("web/package.json", "{}\n"),
        ("web/src/client.ts", &lines(7)),
        ("web/src/gen.ts", &lines(9)),
        ("web/src/a.test.ts", &lines(8)),
        ("web/src/a.ts", &lines(1)),
        ("web/src/b.ts", &lines(1)),
        ("web/src/c.ts", &lines(1)),
        ("web/src/d.ts", &lines(1)),
    ]);
    let root = temp.path().join("project");
    let (success, document, stderr) = headline(&root, &[]);
    assert!(success, "{stderr}");
    // Each fixture line is code, so lines equal code.
    let file = |path: &str, code: u64, module: &str, detector: &str| {
        json!({
            "path": path,
            "lines": code,
            "code": code,
            "module": module,
            "detector": detector
        })
    };
    assert_eq!(
        document["summary"]["largest_module_files"],
        json!([
            file("web/src/client.ts", 7, "web", "npm"),
            file("backend/src/main/kotlin/A.kt", 2, "backend", "gradle"),
            file("web/src/a.ts", 1, "web", "npm"),
            file("web/src/b.ts", 1, "web", "npm"),
            file("web/src/c.ts", 1, "web", "npm"),
        ])
    );
    let table = String::from_utf8(run(&root, &["--verbose", "--format", "table"]).stdout).unwrap();
    assert!(
        table.contains(
            " Largest files counted as Production only by a build layout. If one is generated or vendored, exclude or reclassify it in '.locmetryrc':\n   web/src/client.ts             7 code lines  web (npm)\n   backend/src/main/kotlin/A.kt  2 code lines  backend (gradle)\n"
        ),
        "{table}"
    );
    let markdown = String::from_utf8(run(&root, &["--format", "markdown"]).stdout).unwrap();
    let section = &markdown[markdown.find("## Build modules").expect(&markdown)..];
    assert!(
        section.contains("| web/src/client.ts | 7 | 7 | web | npm |"),
        "{section}"
    );
    // Saved reports carrying file records must list exactly these files.
    let (_, full, _) = report(&root, &[]);
    let validate = |document: Value| {
        serde_json::from_value::<locmetry::contract::MeasurementReport>(document)
            .unwrap()
            .validate()
    };
    validate(full.clone()).unwrap();
    // Producers before this field recorded no list; their reports stay valid.
    let mut historical = full.clone();
    historical["summary"]
        .as_object_mut()
        .unwrap()
        .remove("largest_module_files");
    validate(historical).unwrap();
    let mut forged = full;
    forged["summary"]["largest_module_files"][0]["path"] = json!("web/src/gen.ts");
    let error = validate(forged).unwrap_err().to_string();
    assert!(
        error.contains("largest_module_files conflicts with measured files"),
        "{error}"
    );

    fs::write(
        root.join(".locmetryrc"),
        "version = 1\n[classification]\nbuiltin_modules = false\n",
    )
    .unwrap();
    // Recorded as an empty list, so absence keeps meaning "not recorded".
    let (_, document, _) = headline(&root, &[]);
    assert_eq!(document["summary"]["largest_module_files"], json!([]));
    let table = String::from_utf8(run(&root, &["--verbose", "--format", "table"]).stdout).unwrap();
    assert!(
        !table.contains("Largest files counted as Production"),
        "{table}"
    );
}

/// A folder holding most of the unclaimed code splits even when many
/// top-level folders already fill the list, so the bulk is located precisely.
#[test]
fn dominant_unclaimed_folders_split_beyond_the_row_limit() {
    let mut files: Vec<(String, String)> = (0..9)
        .map(|n| (format!("f{n}/x.js"), "let a = 1;\n".to_string()))
        .collect();
    files.push(("big/one/a.js".into(), "let a = 1;\n".repeat(20)));
    files.push(("big/two/b.js".into(), "let b = 1;\n".repeat(10)));
    let borrowed: Vec<(&str, &str)> = files
        .iter()
        .map(|(p, c)| (p.as_str(), c.as_str()))
        .collect();
    let temp = project(&borrowed);
    let (success, document, stderr) = headline(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    let folders = document["summary"]["unclaimed_folders"].as_array().unwrap();
    assert_eq!(
        folders[0],
        json!({"folder": "big/one", "files": 1, "code": 20})
    );
    assert_eq!(
        folders[1],
        json!({"folder": "big/two", "files": 1, "code": 10})
    );
    assert_eq!(folders.len(), 11);
}
