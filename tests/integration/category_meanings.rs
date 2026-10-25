//! The meanings a policy declares in `[categories]` beside the category rows
//! of human reports, so a shared report explains its own names (feedback of
//! 2026-10-25).
use crate::cli::{project, run};
use crate::common::{stdout, write_all};

const POLICY: &str = r#"version = 1
[categories]
tooling = "Maintained developer commands"
archived = "Retired code and frozen copies"
[[rules]]
id = "tools"
match = ["tools/**"]
purpose = "production"
category = "tooling"
"#;

fn measured() -> tempfile::TempDir {
    project(&[
        (".locmetryrc", POLICY),
        ("src/main.py", "x = 1\n"),
        ("tools/check.py", "x = 1\n"),
    ])
}

/// The line naming `category` and its meaning, ignoring the padding between them.
fn meaning_line<'a>(text: &'a str, category: &str, meaning: &str) -> Option<&'a str> {
    text.lines()
        .find(|line| line.contains(category) && line.contains(meaning))
}

#[test]
fn markdown_reports_the_meaning_of_each_declared_category_it_shows() {
    let temp = measured();
    let text = stdout(run(&temp.path().join("project"), &["--format", "markdown"]));
    assert!(
        text.contains(
            "| Category | Meaning |\n| --- | --- |\n| tooling | Maintained developer commands |\n"
        ),
        "{text}"
    );
    // A declared category no row shows has nothing to explain.
    assert!(!text.contains("Retired code"), "{text}");
}

#[test]
fn verbose_category_tables_list_meanings_and_default_tables_do_not() {
    let temp = measured();
    let root = temp.path().join("project");
    let verbose = stdout(run(&root, &["--show-categories", "--verbose", "--no-bars"]));
    assert!(
        meaning_line(&verbose, "tooling", "Maintained developer commands").is_some(),
        "{verbose}"
    );
    assert!(!verbose.contains("Retired code"), "{verbose}");
    let plain = stdout(run(&root, &["--show-categories", "--no-bars"]));
    assert!(!plain.contains("Maintained developer commands"), "{plain}");
}

#[test]
fn a_category_without_a_declared_meaning_is_not_listed() {
    let temp = project(&[("src/main.py", "x = 1\n")]);
    let text = stdout(run(&temp.path().join("project"), &["--format", "markdown"]));
    assert!(!text.contains("| Category | Meaning |"), "{text}");
}

/// Members of a workspace may declare one name differently; a meaning that
/// holds for one member only would mislabel the merged row, so it is left out.
#[test]
fn workspace_reports_list_only_meanings_every_declaring_member_shares() {
    let temp = tempfile::tempdir().unwrap();
    let policy = |meaning: &str| {
        format!(
            "version = 1\n[categories]\ntooling = \"Maintained developer commands\"\nfixture = \"{meaning}\"\n[[rules]]\nid = \"tools\"\nmatch = [\"tools/**\"]\npurpose = \"production\"\ncategory = \"tooling\"\n[[rules]]\nid = \"fixtures\"\nmatch = [\"fixtures/**\"]\npurpose = \"test\"\ncategory = \"fixture\"\n"
        )
    };
    let (a, b) = (
        policy("Inputs of the parser tests"),
        policy("Recorded server responses"),
    );
    write_all(
        temp.path(),
        &[
            ("a/.locmetryrc", &a),
            ("a/tools/x.py", "x = 1\n"),
            ("a/fixtures/x.py", "x = 1\n"),
            ("b/.locmetryrc", &b),
            ("b/tools/x.py", "x = 1\n"),
            ("b/fixtures/x.py", "x = 1\n"),
        ],
    );
    let text = stdout(run(
        temp.path(),
        &[
            "workspace",
            "--root",
            "a",
            "--root",
            "b",
            "--format",
            "markdown",
        ],
    ));
    assert!(
        text.contains("| tooling | Maintained developer commands |"),
        "{text}"
    );
    assert!(!text.contains("| fixture |"), "{text}");
}
