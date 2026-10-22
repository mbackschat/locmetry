//! Locks the bounded presence findings of human comparisons
//! (docs/proposals/archive/comparison-presence-summary.md): contained folders and
//! packages fold into their outermost changed folder or member, a cap ends
//! each side's list with a count, and JSON keeps every finding.
use crate::common::write;
use crate::compare::{compare, measure, save, workspace};
use serde_json::Value;
use std::path::Path;

fn human(before: &Path, after: &Path, format: &str) -> String {
    let output = compare(before, after, &["--format", format]);
    assert_eq!(output.status.code(), Some(0));
    String::from_utf8(output.stdout).unwrap()
}

fn presence_lines(text: &str) -> Vec<&str> {
    text.lines()
        .filter(|line| line.contains("presence_changed"))
        .collect()
}

#[test]
fn a_moved_subtree_folds_into_one_finding_per_side() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "keep.py", "a = 1\n");
    for index in 1..=12 {
        write(&root, &format!("old/m{index:02}/x.py"), "b = 2\n");
        write(&root, &format!("old/m{index:02}/package.json"), "{}\n");
    }
    let before = save(temp.path(), "before.json", &measure(&root));
    std::fs::rename(root.join("old"), root.join("new")).unwrap();
    let after = save(temp.path(), "after.json", &measure(&root));
    for format in ["table", "markdown"] {
        let text = human(&before, &after, format);
        let code = match format {
            "table" => "presence_changed",
            _ => "**presence_changed**",
        };
        assert_eq!(
            presence_lines(&text),
            [
                format!("- {code} (before): Folder 'old' appears only in the before report, with 12 folders and 12 packages inside it. It may be deleted or no longer measured."),
                format!("- {code} (after): Folder 'new' appears only in the after report, with 12 folders and 12 packages inside it. It may be new or newly measured."),
            ],
            "{text}"
        );
    }
    let full: Value =
        serde_json::from_slice(&compare(&before, &after, &["--format", "json"]).stdout).unwrap();
    let presence = full["findings"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|finding| finding["code"] == "presence_changed")
        .count();
    assert_eq!(presence, 2 * (1 + 12 + 12));
}

#[test]
fn a_departing_member_absorbs_its_folders() {
    let temp = tempfile::tempdir().unwrap();
    let web = temp.path().join("web");
    let api = temp.path().join("api");
    write(&web, "src/main.py", "a = 1\n");
    write(&api, "lib/deep/n.py", "b = 2\n");
    let before = save(temp.path(), "before.json", &workspace(&[&web, &api]));
    let after = save(temp.path(), "after.json", &workspace(&[&web]));
    let text = human(&before, &after, "table");
    assert_eq!(
        presence_lines(&text),
        ["- presence_changed (before, api): Member 'api' left the workspace measurement, with 3 folders inside it. Its source may still exist."],
        "{text}"
    );
}

#[test]
fn presence_beyond_ten_lines_per_side_ends_in_a_count() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "src/keep.py", "a = 1\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    for index in 1..=25 {
        write(&root, &format!("src/f{index:02}/x.py"), "b = 2\n");
    }
    write(&root, "src/f25/inner/y.py", "c = 3\n");
    let after = save(temp.path(), "after.json", &measure(&root));
    let text = human(&before, &after, "table");
    let lines = presence_lines(&text);
    assert_eq!(lines.len(), 11, "{text}");
    assert!(lines[..10]
        .iter()
        .all(|line| line
            .ends_with("appears only in the after report. It may be new or newly measured.")));
    assert_eq!(
        lines[10],
        "- presence_changed (after): 16 more folders appear only in the after report. '--format json' lists each one.",
    );
}

/// A closing line that replaced a single finding would hide its name and save
/// no line, so the cap starts only with two lines beyond it.
#[test]
fn one_line_beyond_the_cap_is_listed_by_name() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "src/keep.py", "a = 1\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    for index in 1..=11 {
        write(&root, &format!("src/f{index:02}/x.py"), "b = 2\n");
    }
    let after = save(temp.path(), "after.json", &measure(&root));
    let text = human(&before, &after, "table");
    let lines = presence_lines(&text);
    assert_eq!(lines.len(), 11, "{text}");
    assert!(
        lines.iter().all(|line| line.contains("Folder 'src/f")),
        "{text}"
    );
}
