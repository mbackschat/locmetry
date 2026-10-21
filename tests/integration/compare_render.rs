use crate::common;
use crate::compare::*;
use common::{json, write};
use serde_json::json;
use std::{fs, path::Path};

fn human_comparisons(before: &Path, after: &Path, status: i32) -> Vec<String> {
    ["table", "markdown"]
        .into_iter()
        .map(|format| {
            let output = compare(before, after, &["--format", format]);
            assert_eq!(
                output.status.code(),
                Some(status),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            String::from_utf8(output.stdout).unwrap()
        })
        .collect()
}

#[test]
fn purpose_ratio_uses_source_code_and_handles_missing_purposes_on_each_side() {
    for (before, after, expected) in [
        ((5, 6), (17, 20), "1.20 → 1.18"),
        ((4, 0), (4, 2), "0.00 → 0.50"),
        ((0, 2), (2, 2), "- → 1.00"),
        ((2, 2), (0, 3), "1.00 → -"),
        ((0, 0), (0, 1), "- → -"),
        ((2, 0), (4, 0), "0.00 → 0.00"),
    ] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("project");
        write(&root, ".locmetryrc", "version = 1\n[[rules]]\nid = \"production\"\nmatch = [\"prod/**\"]\npurpose = \"production\"\n[[rules]]\nid = \"test\"\nmatch = [\"tests/**\"]\npurpose = \"test\"\n");
        write(&root, "other.py", &"print(1)\n".repeat(100));
        let mut snapshots = Vec::new();
        for (name, (production, test)) in [("before.json", before), ("after.json", after)] {
            for (folder, count) in [("prod", production), ("tests", test)] {
                let file = format!("{folder}/main.py");
                if count > 0 {
                    write(
                        &root,
                        &file,
                        &format!("{}{}", "print(1)\n".repeat(count), "# comment\n".repeat(10)),
                    );
                } else if root.join(&file).exists() {
                    fs::remove_file(root.join(&file)).unwrap();
                }
            }
            snapshots.push(save(temp.path(), name, &measure(&root)));
        }
        for text in human_comparisons(&snapshots[0], &snapshots[1], 0) {
            let expected = format!("Test/prod (Code): {expected} (before → after)");
            assert_eq!(text.matches(&expected).count(), 1, "{text}");
            let ratio = text.find(&expected).unwrap();
            let total = text[..ratio].rfind("Source total").unwrap();
            let between = &text[total..ratio];
            assert!(!between.contains("Language"), "{text}");
            if text.starts_with("# ") {
                assert!(text.contains(&format!("\n\n*{expected}*\n")), "{text}");
            } else {
                assert_eq!(between.lines().count(), 2, "{text}");
            }
        }
        for text in human_comparisons(&snapshots[1], &snapshots[1], 0) {
            assert!(!text.contains("Test/prod (Code):"), "{text}");
        }
    }
}

#[test]
fn workspace_purpose_ratio_uses_aggregate_counts_including_unchanged_rows() {
    let temp = tempfile::tempdir().unwrap();
    let api = temp.path().join("api");
    let web = temp.path().join("web");
    for (root, production) in [(&api, 2), (&web, 8)] {
        write(root, ".locmetryrc", "version = 1\n[[rules]]\nid = \"production\"\nmatch = [\"main.py\"]\npurpose = \"production\"\n");
        write(root, "main.py", &"print(1)\n".repeat(production));
    }
    write(&api, "tests/check.py", &"assert True\n".repeat(6));
    let before = save(temp.path(), "before.json", &workspace(&[&api, &web]));
    write(&api, "tests/check.py", &"assert True\n".repeat(12));
    let after = save(temp.path(), "after.json", &workspace(&[&api, &web]));
    for text in human_comparisons(&before, &after, 0) {
        assert!(
            text.contains("Test/prod (Code): 0.60 → 1.20 (before → after)"),
            "{text}"
        );
        assert!(!text.contains(" Production "), "{text}");
    }
    let full = json(&compare(&before, &after, &["--format", "json"]), 0);
    let report = serde_json::from_value(full).unwrap();
    let colored = locmetry::compare::render_table_with_theme(
        &report,
        locmetry::style::Theme::TERMINAL,
        locmetry::LanguageVariants::Split,
    );
    assert!(
        colored.contains("\x1b[90mTest/prod (Code): 0.60 → 1.20 (before → after)\x1b[0m"),
        "{colored}"
    );
}

#[test]
fn unchanged_workspace_replaces_zero_composition_with_a_no_change_message() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "main.py", "print(1)\n");
    let saved = save(temp.path(), "input.json", &workspace(&[&root]));
    let full = json(&compare(&saved, &saved, &["--format", "json"]), 0);
    assert_eq!(
        full["summary"]["composition"]["source"]["matched"]["code"],
        0
    );
    let report = serde_json::from_value(full).unwrap();
    let mut texts = human_comparisons(&saved, &saved, 0);
    texts.push(locmetry::compare::render_table_with_theme(
        &report,
        locmetry::style::Theme::TERMINAL,
        locmetry::LanguageVariants::Split,
    ));
    for text in texts {
        assert!(text.contains("All counted"), "{text}");
        assert!(
            text.contains("No recorded count or presence changes."),
            "{text}"
        );
        assert!(!text.contains("Workspace source"), "{text}");
        assert!(!text.contains("Workspace all-counted"), "{text}");
    }
}

#[test]
fn workspace_composition_keeps_offsetting_arrivals_and_departures() {
    let temp = tempfile::tempdir().unwrap();
    let old = temp.path().join("old");
    let new = temp.path().join("new");
    write(&old, "main.py", "print(1)\n");
    write(&new, "main.py", "print(2)\n");
    let before = save(temp.path(), "before.json", &workspace(&[&old]));
    let after = save(temp.path(), "after.json", &workspace(&[&new]));
    for text in human_comparisons(&before, &after, 0) {
        assert!(
            text.contains("Workspace source lines: matched +0, entering +1, leaving -1; total +0."),
            "{text}"
        );
        assert!(
            text.contains("Workspace source code: matched +0, entering +1, leaving -1; total +0."),
            "{text}"
        );
        assert!(
            !text.contains("No recorded count or presence changes."),
            "{text}"
        );
    }
}

#[test]
fn zero_workspace_totals_do_not_hide_folder_transfers_or_empty_member_presence() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "left/main.py", "print(1)\nprint(2)\n");
    write(&root, "right/main.py", "print(1)\n");
    let before = save(temp.path(), "before.json", &workspace(&[&root]));
    write(&root, "left/main.py", "print(1)\n");
    write(&root, "right/main.py", "print(1)\nprint(2)\n");
    let after = save(temp.path(), "after.json", &workspace(&[&root]));
    let empty = temp.path().join("empty");
    fs::create_dir(&empty).unwrap();
    let arrived = save(temp.path(), "arrived.json", &workspace(&[&root, &empty]));
    for (a, b) in [(&before, &after), (&after, &arrived)] {
        for text in human_comparisons(a, b, 0) {
            assert!(!text.contains("Workspace source"), "{text}");
            assert!(
                !text.contains("No recorded count or presence changes."),
                "{text}"
            );
        }
    }
}

#[test]
fn workspace_no_change_message_accounts_for_comments_blanks_and_files() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "main.py", "print(1)\n# comment\n");
    let before = save(temp.path(), "before.json", &workspace(&[&root]));
    write(&root, "main.py", "print(1)\n\n");
    let after = save(temp.path(), "after.json", &workspace(&[&root]));
    write(&root, "empty.py", "");
    let added = save(temp.path(), "added.json", &workspace(&[&root]));
    for (a, b) in [(&before, &after), (&after, &added)] {
        for text in human_comparisons(a, b, 0) {
            assert!(!text.contains("Workspace source"), "{text}");
            assert!(!text.contains("Workspace all-counted"), "{text}");
            assert!(
                !text.contains("No recorded count or presence changes."),
                "{text}"
            );
        }
    }
}

#[test]
fn human_coverage_notes_summarize_paths_without_changing_json_findings() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "a.zzz", "unknown\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    let full = json(&compare(&before, &before, &["--format", "json"]), 0);
    assert_eq!(full["findings"].as_array().unwrap().len(), 4);
    assert!(full["findings"]
        .as_array()
        .unwrap()
        .iter()
        .all(|f| f["code"] == "coverage_gap"));
    for text in human_comparisons(&before, &before, 0) {
        assert_eq!(text.matches("Unrecognized files:").count(), 1, "{text}");
        assert!(
            text.contains("same 1 path in both reports; not counted on either side."),
            "{text}"
        );
        assert!(!text.contains("coverage_gap"), "{text}");
        assert!(!text.contains("Folder . contains"), "{text}");
        assert!(
            text.contains("No recorded differences in measurement settings."),
            "{text}"
        );
    }
    let report: locmetry::compare::Comparison = serde_json::from_value(full).unwrap();
    let styled = locmetry::compare::render_table_with_theme(
        &report,
        locmetry::style::Theme::TERMINAL,
        locmetry::LanguageVariants::Split,
    );
    assert!(styled.contains("\x1b[90mUnrecognized files:"), "{styled}");
    assert!(!styled.contains("\x1b[33m"), "{styled}");
    fs::rename(root.join("a.zzz"), root.join("b.zzz")).unwrap();
    let after = save(temp.path(), "after.json", &measure(&root));
    for text in human_comparisons(&before, &after, 0) {
        assert!(
            text.contains("Unrecognized files: 1 → 1 (0 in both, 1 only before, 1 only after)"),
            "{text}"
        );
        assert!(!text.contains("same 1 path"), "{text}");
    }
}

#[test]
fn human_coverage_notes_keep_missing_evidence_distinct_from_recorded_failures() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "bad.ipynb", "invalid\n");
    let mut report = measure(&root);
    let before = save(temp.path(), "before.json", &report);
    report["metadata"]["coverage"]["parse_failures"][0]["message"] = json!("different detail");
    let after = save(temp.path(), "after.json", &report);
    for text in human_comparisons(&before, &after, 2) {
        assert!(
            text.contains("Parse failures: same 1 path/reason pair in both reports"),
            "{text}"
        );
    }
    report["metadata"]["coverage"]["parse_failures"][0]["reason"] =
        json!("unsupported_notebook_version");
    let changed = save(temp.path(), "changed.json", &report);
    for text in human_comparisons(&before, &changed, 2) {
        assert!(
            text.contains("Parse failures: 1 → 1 (0 in both, 1 only before, 1 only after)"),
            "{text}"
        );
    }
    fs::remove_file(root.join("bad.ipynb")).unwrap();
    let mut legacy = measure(&root);
    legacy["metadata"]["coverage"]
        .as_object_mut()
        .unwrap()
        .remove("parse_failures");
    let historical = save(temp.path(), "historical.json", &legacy);
    for text in human_comparisons(&historical, &historical, 0) {
        assert!(
            text.contains("Failure evidence: not recorded (before); not recorded (after)."),
            "{text}"
        );
        assert!(!text.contains("same"), "{text}");
    }
    for text in human_comparisons(&historical, &before, 2) {
        assert!(text.contains("Parse failures: not recorded → 1"), "{text}");
        assert!(!text.contains("only after"), "{text}");
    }
}

#[test]
fn human_coverage_notes_keep_partial_inputs_visible_and_warn() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "main.py", "print(1)\n");
    let mut report = measure(&root);
    report["metadata"]["backend_inaccurate"] = json!(true);
    let inaccurate = save(temp.path(), "inaccurate.json", &report);
    for text in human_comparisons(&inaccurate, &inaccurate, 2) {
        assert!(text.contains("Inaccurate input: before, after"), "{text}");
    }
    report["metadata"]["coverage"]["unread"] = json!(["unread.py"]);
    report["metadata"]["coverage"]["selected_files"] = json!(2);
    let unread = save(temp.path(), "unread.json", &report);
    for text in human_comparisons(&unread, &unread, 2) {
        assert!(
            text.contains("Unreadable files: same 1 path in both reports"),
            "{text}"
        );
    }
    let full = json(&compare(&unread, &unread, &["--format", "json"]), 2);
    let comparison: locmetry::compare::Comparison = serde_json::from_value(full).unwrap();
    let styled = locmetry::compare::render_table_with_theme(
        &comparison,
        locmetry::style::Theme::TERMINAL,
        locmetry::LanguageVariants::Split,
    );
    assert!(styled.contains("\x1b[33mUnreadable files:"), "{styled}");
    assert!(styled.contains("\x1b[33mInaccurate input:"), "{styled}");
}

#[test]
fn human_coverage_notes_name_actionable_members_and_count_missing_member_evidence() {
    let temp = tempfile::tempdir().unwrap();
    let api = temp.path().join("api");
    let web = temp.path().join("web");
    write(&api, "main.py", "print(1)\n");
    write(&web, "main.py", "print(2)\n");
    let mut report = workspace(&[&api, &web]);
    let mut legacy = report.clone();
    legacy["metadata"]["coverage"]
        .as_object_mut()
        .unwrap()
        .remove("parse_failures");
    for member in legacy["metadata"]["members"].as_array_mut().unwrap() {
        member["coverage"]
            .as_object_mut()
            .unwrap()
            .remove("parse_failed_files");
    }
    let historical = save(temp.path(), "historical-workspace.json", &legacy);
    report["metadata"]["backend_inaccurate"] = json!(true);
    report["metadata"]["coverage"]["selected_files"] = json!(3);
    report["metadata"]["coverage"]["parse_failures"] = json!([{
        "path": "api/bad.ipynb", "reason": "invalid_notebook", "message": "invalid"
    }]);
    report["metadata"]["members"][0]["coverage"]["selected_files"] = json!(2);
    report["metadata"]["members"][0]["coverage"]["parse_failed_files"] = json!(1);
    report["metadata"]["members"][1]["coverage"]
        .as_object_mut()
        .unwrap()
        .remove("parse_failed_files");
    let saved = save(temp.path(), "workspace.json", &report);
    for text in human_comparisons(&saved, &saved, 2) {
        let failures = text
            .lines()
            .find(|l| l.contains("Parse failures:"))
            .unwrap();
        assert!(
            failures.contains("Members before: api; after: api"),
            "{text}"
        );
        assert!(
            text.contains("not recorded for 1 of 2 members (before)"),
            "{text}"
        );
        assert!(
            text.contains("not recorded for 1 of 2 members (after)"),
            "{text}"
        );
        assert!(!text.contains("coverage_gap"), "{text}");
    }
    for text in human_comparisons(&historical, &saved, 2) {
        assert!(
            text.contains("Members before: not recorded; after: api."),
            "{text}"
        );
    }
}

#[test]
fn human_producer_notes_show_each_changed_field_without_asserting_counting_behavior() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "main.py", "print(1)\n");
    let mut report = measure(&root);
    report["metadata"]["locmetry_version"] = json!("old");
    let before = save(temp.path(), "before.json", &report);
    report["metadata"]["locmetry_version"] = json!("new");
    let version = save(temp.path(), "version.json", &report);
    for text in human_comparisons(&before, &version, 0) {
        assert!(text.contains("Locmetry version: old → new."), "{text}");
        assert!(text.contains("Counting corrections can differ between versions; the reports cannot show whether they did."), "{text}");
        assert!(!text.contains("counter_changed"), "{text}");
        assert!(!text.contains("No recorded differences"), "{text}");
    }
    report["metadata"]["backend"] = json!("other backend");
    report["metadata"]["backend_configuration"] = json!("other configuration");
    let after = save(temp.path(), "after.json", &report);
    for text in human_comparisons(&before, &after, 0) {
        assert!(text.contains("Backend:"), "{text}");
        assert!(text.contains("→ other backend."), "{text}");
        assert!(text.contains("Backend configuration:"), "{text}");
        assert!(text.contains("→ other configuration."), "{text}");
    }
}

#[test]
fn source_grids_include_line_changes_but_omit_data_and_file_count_only_changes() {
    for variant in ["comment", "blank", "data", "empty file"] {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().join("project");
        write(&root, "src/main.py", "print(1)\n");
        write(&root, "src/settings.json", "{}\n");
        let before = save(temp.path(), "before.json", &measure(&root));
        let mut source_delta = json!({"files":0,"code":0,"comments":0,"blanks":0,"lines":0});
        match variant {
            "comment" => {
                write(&root, "src/main.py", "print(1)\n# comment\n");
                source_delta["comments"] = json!(1);
                source_delta["lines"] = json!(1);
            }
            "blank" => {
                write(&root, "src/main.py", "print(1)\n\n");
                source_delta["blanks"] = json!(1);
                source_delta["lines"] = json!(1);
            }
            "data" => write(&root, "src/settings.json", "{\n  \"a\": 1\n}\n"),
            "empty file" => {
                write(&root, "src/empty.py", "");
                source_delta["files"] = json!(1);
            }
            _ => unreachable!(),
        }
        let after = save(temp.path(), "after.json", &measure(&root));
        let full = json(&compare(&before, &after, &["--format", "json"]), 0);
        assert_eq!(
            full["folders"]["src"]["recursive"]["source"]["delta"], source_delta,
            "{variant}"
        );
        assert_eq!(
            full["summary"]["source"]["delta"], source_delta,
            "{variant}"
        );
        if variant == "data" {
            assert_eq!(
                full["folders"]["src"]["recursive"]["all_counted"]["delta"]["code"],
                2
            );
        }
        for text in human_comparisons(&before, &after, 0) {
            for heading in [" Purpose ", " Language ", " Folder "] {
                assert_eq!(
                    text.contains(heading),
                    matches!(variant, "comment" | "blank"),
                    "{variant}: {text}"
                );
            }
        }
    }
}

#[test]
fn source_code_grids_preserve_zero_code_folder_and_member_presence() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "src/main.py", "print(1)\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    write(&root, "assets/icon.zzz", "unknown\n");
    let after = save(temp.path(), "after.json", &measure(&root));
    for text in human_comparisons(&before, &after, 0) {
        assert!(text.contains("assets (after only)"), "{text}");
        assert!(!text.contains(". (both)"), "{text}");
    }
    for text in human_comparisons(&after, &before, 0) {
        assert!(text.contains("assets (before only)"), "{text}");
    }
    write(&root, "src/settings.json", "{}\n");
    let workspace_before = save(temp.path(), "workspace-before.json", &workspace(&[&root]));
    write(&root, "src/main.py", "print(1)\n# comment\n");
    write(&root, "src/settings.json", "{\n  \"a\": 1\n}\n");
    let workspace_after = save(temp.path(), "workspace-after.json", &workspace(&[&root]));
    let full = json(
        &compare(&workspace_before, &workspace_after, &["--format", "json"]),
        0,
    );
    assert_eq!(full["members"]["project"]["source"]["delta"]["comments"], 1);
    assert_eq!(
        full["members"]["project"]["all_counted"]["delta"]["code"],
        2
    );
    for text in human_comparisons(&workspace_before, &workspace_after, 0) {
        assert!(text.contains(" Member "), "{text}");
        assert_eq!(
            text.lines()
                .any(|line| line.starts_with(" Folder ") || line.starts_with("| Folder |")),
            text.starts_with("# "),
            "{text}"
        );
    }
    let empty = temp.path().join("empty");
    fs::create_dir(&empty).unwrap();
    let arrived = save(temp.path(), "arrived.json", &workspace(&[&root, &empty]));
    for text in human_comparisons(&workspace_after, &arrived, 0) {
        assert!(text.contains("empty (after only)"), "{text}");
        assert!(!text.contains("project (both)"), "{text}");
    }
    for text in human_comparisons(&arrived, &workspace_after, 0) {
        assert!(text.contains("empty (before only)"), "{text}");
    }
    write(&root, "src/main.py", "print(1)\nprint(2)\n");
    let source_after = save(temp.path(), "source-after.json", &measure(&root));
    for text in human_comparisons(&after, &source_after, 0) {
        for heading in [" Purpose ", " Language ", " Folder "] {
            assert!(text.contains(heading), "{text}");
        }
    }
    let workspace_source_after = save(
        temp.path(),
        "workspace-source-after.json",
        &workspace(&[&root]),
    );
    for text in human_comparisons(&workspace_after, &workspace_source_after, 0) {
        assert!(text.contains("project (both)"), "{text}");
    }
}

#[test]
fn workspace_composition_colors_signed_contributions_and_mutes_signed_zero() {
    use locmetry::style::Theme;
    let temp = tempfile::tempdir().unwrap();
    let api = temp.path().join("api");
    let legacy = temp.path().join("legacy");
    let worker = temp.path().join("worker");
    write(&api, "main.py", &"print(1)\n".repeat(6));
    write(&legacy, "main.py", &"print(1)\n".repeat(2));
    let before = save(temp.path(), "before.json", &workspace(&[&api, &legacy]));
    write(&api, "main.py", &"print(1)\n".repeat(3));
    write(&worker, "main.py", &"print(1)\n".repeat(5));
    let after = save(temp.path(), "after.json", &workspace(&[&api, &worker]));
    write(&worker, "main.py", &"print(1)\n".repeat(6));
    let grown = save(temp.path(), "grown.json", &workspace(&[&api, &worker]));
    for (a, b, matched, entering, leaving, total) in [
        (
            &before,
            &after,
            "\x1b[31m-3\x1b[0m",
            "\x1b[32m+5\x1b[0m",
            "\x1b[31m-2\x1b[0m",
            "\x1b[90m+0\x1b[0m",
        ),
        (
            &after,
            &before,
            "\x1b[32m+3\x1b[0m",
            "\x1b[32m+2\x1b[0m",
            "\x1b[31m-5\x1b[0m",
            "\x1b[90m+0\x1b[0m",
        ),
        (
            &after,
            &grown,
            "\x1b[32m+1\x1b[0m",
            "\x1b[90m+0\x1b[0m",
            "\x1b[90m-0\x1b[0m",
            "\x1b[32m+1\x1b[0m",
        ),
        (
            &grown,
            &before,
            "\x1b[32m+3\x1b[0m",
            "\x1b[32m+2\x1b[0m",
            "\x1b[31m-6\x1b[0m",
            "\x1b[31m-1\x1b[0m",
        ),
    ] {
        let full = json(&compare(a, b, &["--format", "json"]), 0);
        let report = serde_json::from_value(full).unwrap();
        let colored = locmetry::compare::render_table_with_theme(
            &report,
            Theme::TERMINAL,
            locmetry::LanguageVariants::Split,
        );
        let expected =
            format!("matched {matched}, entering {entering}, leaving {leaving}; total {total}.");
        for label in [
            "source lines",
            "all-counted lines",
            "source code",
            "all-counted code",
        ] {
            assert!(
                colored.contains(&format!("Workspace {label}: {expected}")),
                "{colored}"
            );
        }
        let stripped: String = colored
            .split('\x1b')
            .map(|part| match part.strip_prefix('[') {
                Some(control) => control.split_once('m').unwrap().1,
                None => part,
            })
            .collect();
        let plain = locmetry::compare::render_table_with_theme(
            &report,
            Theme::PLAIN,
            locmetry::LanguageVariants::Split,
        );
        assert_eq!(stripped, plain);
        let markdown =
            locmetry::compare::render_markdown(&report, locmetry::LanguageVariants::Split);
        for text in human_comparisons(a, b, 0).into_iter().chain([markdown]) {
            assert!(!text.contains('\x1b'), "{text}");
            for line in plain.lines().filter(|line| line.starts_with("Workspace ")) {
                assert!(text.contains(line), "{text}");
            }
        }
    }
}

#[test]
fn comparison_theme_preserves_plain_text_and_distinguishes_signed_changes() {
    use locmetry::style::Theme;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("source");
    write(&root, "main.py", "print(1)\n\n");
    write(&root, "README.md", "# A\n\ntext\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    write(&root, "main.py", "print(1)\nprint(2)\n");
    write(&root, "README.md", "# A\n");
    let after = save(temp.path(), "after.json", &measure(&root));
    let value = json(&compare(&before, &after, &["--output", "-"]), 0);
    let report: locmetry::compare::Comparison = serde_json::from_value(value).unwrap();
    let plain = locmetry::compare::render_table_with_theme(
        &report,
        Theme::PLAIN,
        locmetry::LanguageVariants::Split,
    );
    let colored = locmetry::compare::render_table_with_theme(
        &report,
        Theme::TERMINAL,
        locmetry::LanguageVariants::Split,
    );
    for (style, value) in [("32m", "+1"), ("31m", "-2"), ("90m", "0")] {
        assert!(
            colored.split("\x1b[").any(|cell| cell
                .strip_prefix(style)
                .is_some_and(|cell| cell.trim() == value)),
            "{style} {value}\n{colored}"
        );
    }
    let stripped: String = colored
        .split('\x1b')
        .map(|s| match s.strip_prefix('[') {
            Some(control) => control.split_once('m').unwrap().1,
            None => s,
        })
        .collect();
    assert_eq!(stripped, plain);
    assert!(!plain.contains('\x1b'));
    let markdown = locmetry::compare::render_markdown(&report, locmetry::LanguageVariants::Split);
    assert!(markdown.contains("*Net change = after − before"));
    assert!(!markdown.contains('\x1b'));
    assert!(
        markdown.contains("| Source | 2 | 2 | 0 | 1 | 2 | +1 | 0 | -1 | 0 |"),
        "{markdown}"
    );
    assert!(
        markdown.contains("| **All counted** | **5** | **3** | **-2** |"),
        "{markdown}"
    );
}

#[test]
fn human_comparisons_show_line_changes_and_original_measurement_dates() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "src/main.py", "print(1)\n");
    let mut before = measure(&root);
    before["metadata"]["measured_at_unix_seconds"] = json!(0);
    write(&root, "src/main.py", "print(1)\n# comment\n\n");
    let mut after = measure(&root);
    after["metadata"]["measured_at_unix_seconds"] = json!(951827696);
    let before = save(temp.path(), "before.json", &before);
    let after = save(temp.path(), "after.json", &after);
    for text in human_comparisons(&before, &after, 0) {
        // Markdown cannot span columns, so it keeps the full titles that the
        // table splits into a group heading and the column headings below it.
        let headed = match text.starts_with("# ") {
            true => text.contains("| Lines before |"),
            false => text.contains(" Lines ") && text.contains(" before "),
        };
        assert!(headed, "{text}");
        let language = text.lines().find(|line| line.contains("Python")).unwrap();
        let cells: Vec<_> = language
            .split_whitespace()
            .filter(|c| !matches!(*c, "|" | "│"))
            .map(|c| c.trim_matches('*'))
            .collect();
        assert_eq!(cells, ["Python", "1", "3", "+2", "1", "1", "0"]);
        assert!(text.ends_with("Before measured at: 1970-01-01 00:00:00 UTC\nAfter measured at: 2000-02-29 12:34:56 UTC\n"), "{text}");
    }
}

#[test]
fn human_comparisons_group_interpretation_caveats_under_measurement_notes() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "src/main.py", "a = 1\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    write(&root, "src/extra/more.py", "b = 2\n");
    let after = save(temp.path(), "after.json", &measure(&root));
    for after in [&before, &after] {
        for text in human_comparisons(&before, after, 0) {
            let (results, notes) = text
                .split_once("Measurement notes")
                .unwrap_or_else(|| panic!("missing Measurement notes:\n{text}"));
            assert!(results.contains("All counted"), "{text}");
            let notes = notes.split("Findings").next().unwrap();
            for caveat in [
                "Net change = after − before, from the two saved reports.",
                "Unreadable files count as 0; a folder seen in only one report may be newly visible, not newly created.",
            ] {
                assert!(!results.contains(caveat), "{text}");
                assert_eq!(text.matches(caveat).count(), 1, "{text}");
                assert!(
                    notes.lines().any(|line| line.starts_with("- ") && line.contains(caveat)),
                    "{text}"
                );
            }
        }
    }
}

/// Locks the plain wording that replaced internal terms: the measurement caveats,
/// presence counts in words without zero or non-workspace parts, and findings
/// that name the folder instead of an "Entity".
#[test]
fn human_comparisons_explain_presence_in_plain_words() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "src/main.py", "a = 1\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    write(&root, "src/extra/more.py", "b = 2\n");
    let after = save(temp.path(), "after.json", &measure(&root));
    let output = compare(&before, &after, &["--format", "table"]);
    assert_eq!(output.status.code(), Some(0));
    let table = String::from_utf8(output.stdout).unwrap();
    for expected in [
        "Net change = after − before, from the two saved reports.",
        "Unreadable files count as 0; a folder seen in only one report may be newly visible, not newly created.",
        "Only in one report: 1 folder only after.",
        "Folder \"src/extra\" appears only in the after report; it may be new or newly measured.",
    ] {
        assert!(table.contains(expected), "{expected}\n{table}");
    }
    for absent in ["Entity", "members -", "Presence:"] {
        assert!(!table.contains(absent), "{absent}\n{table}");
    }
}

/// Locks that the human totals explain All counted: one row per content kind,
/// whose figures add up to the All counted row, and that the Purpose and
/// table Folders grids end in the source total, with the root folder as it.
#[test]
fn totals_split_all_counted_by_kind_and_grids_end_in_their_source_total() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("project");
    write(&root, "src/main.py", "a = 1\n");
    write(&root, "tests/test_main.py", "assert True\n");
    write(&root, "README.md", "# Project\n\ntext\n");
    write(&root, "config.json", "{\n  \"a\": 1\n}\n");
    let before = save(temp.path(), "before.json", &measure(&root));
    write(&root, "src/main.py", "a = 1\nb = 2\n");
    write(&root, "config.json", "{\n  \"a\": 1,\n  \"b\": 2\n}\n");
    let after = save(temp.path(), "after.json", &measure(&root));
    let table = String::from_utf8(compare(&before, &after, &["--format", "table"]).stdout).unwrap();
    let section = |label: &str| {
        table
            .split("\n\n")
            .find(|block| {
                block
                    .lines()
                    .any(|line| line.starts_with(&format!(" {label} ")))
            })
            .unwrap()
            .to_string()
    };
    for label in ["Kind", "Purpose", "Language", "Folder"] {
        let block = section(label);
        let lines: Vec<_> = block.lines().collect();
        assert_eq!(lines[0], lines[3].replace('┼', "─"), "{block}");
        let group_header = lines[1];
        assert!(
            group_header.contains("Lines") && group_header.contains("Code"),
            "{group_header}"
        );
        assert!(!group_header.contains('─'), "{group_header}");
    }
    for title in ["Totals", "Purpose", "Languages", "Folders"] {
        assert!(!table.contains(&format!("▸ {title}")), "{table}");
    }
    let figures = |line: &str| -> Vec<i64> {
        line.split_whitespace()
            .filter_map(|cell| cell.replace(',', "").parse().ok())
            .collect()
    };
    let totals = section("Kind");
    let rows: Vec<_> = totals
        .lines()
        .filter(|line| !figures(line).is_empty())
        .collect();
    let labels: Vec<_> = rows
        .iter()
        .map(|row| row.split("  ").next().unwrap().trim())
        .collect();
    assert_eq!(
        labels,
        ["Source", "Data", "Documentation", "All counted"],
        "{totals}"
    );
    let (kinds, all_counted) = rows.split_at(3);
    let summed = kinds
        .iter()
        .map(|row| figures(row))
        .fold(vec![0; 10], |sum, row| {
            sum.iter().zip(row).map(|(a, b)| a + b).collect()
        });
    assert_eq!(summed, figures(all_counted[0]), "{totals}");
    assert!(figures(kinds[1])[2] > 0, "data growth is visible: {totals}");

    let purpose = section("Purpose");
    assert!(
        purpose
            .lines()
            .rev()
            .nth(1)
            .unwrap()
            .starts_with(" Source total "),
        "{purpose}"
    );
    let folders = section("Folder");
    assert!(
        folders
            .lines()
            .any(|line| line.starts_with(" Source total ")),
        "{folders}"
    );
    assert!(
        folders.contains("Folders are recursive; do not sum parents and children."),
        "{folders}"
    );
    assert!(!folders.contains(". (both)"), "{folders}");
    let markdown =
        String::from_utf8(compare(&before, &after, &["--format", "markdown"]).stdout).unwrap();
    assert!(markdown.contains("| . (both) |"), "{markdown}");
}

#[test]
fn secondary_changes_keep_dim_direction_and_source_changes_match_totals() {
    use locmetry::style::Theme;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("service");
    write(&root, "src/main.py", "print(1)\n# comment\n\n");
    let before = save(temp.path(), "before.json", &workspace(&[&root]));
    write(
        &root,
        "src/main.py",
        "print(1)\nprint(2)\n# comment\n# extra\n",
    );
    write(&root, "src/extra.py", "print(3)\n");
    let after = save(temp.path(), "after.json", &workspace(&[&root]));
    for (a, b, sign) in [(&before, &after, "+"), (&after, &before, "-")] {
        let report: locmetry::compare::Comparison =
            serde_json::from_value(json(&compare(a, b, &["--format", "json"]), 0)).unwrap();
        for report in [report.clone(), report.select_member("service").unwrap()] {
            let colored = locmetry::compare::render_table_with_theme(
                &report,
                Theme::TERMINAL,
                locmetry::LanguageVariants::Split,
            );
            let plain = locmetry::compare::render_table_with_theme(
                &report,
                Theme::PLAIN,
                locmetry::LanguageVariants::Split,
            );
            assert_eq!(common::unstyled(&colored), plain);
            let source = colored
                .lines()
                .find(|line| line.contains("mSource "))
                .unwrap();
            let hue = if sign == "+" { "32" } else { "31" };
            let accent = if sign == "+" { "38;5;43" } else { "38;5;203" };
            assert_eq!(
                source.matches(&format!("\x1b[1;{accent}m")).count(),
                2,
                "{source}"
            );
            for line in colored.lines().filter(|line| line.contains("Source total")) {
                assert_eq!(
                    line.matches(&format!("\x1b[1;{accent}m")).count(),
                    2,
                    "{line}"
                );
            }
            for label in ["All counted", "Python"] {
                let row = colored.lines().find(|line| line.contains(label)).unwrap();
                assert!(
                    !row.contains("38;5;"),
                    "ordinary row with equal deltas: {row}"
                );
                assert!(
                    row.contains(&format!(";{hue}m")) || row.contains(&format!("[{hue}m")),
                    "{row}"
                );
            }
            for (style, value) in [
                (format!("2;{hue}"), format!("{sign}1")),
                (
                    format!("2;{}", if sign == "+" { "31" } else { "32" }),
                    format!("{}1", if sign == "+" { "-" } else { "+" }),
                ),
            ] {
                assert!(
                    source.split("\x1b[").any(|cell| cell
                        .strip_prefix(&format!("{style}m"))
                        .is_some_and(|cell| cell.trim() == value)),
                    "{source}"
                );
            }
        }
    }
    let unchanged: locmetry::compare::Comparison =
        serde_json::from_value(json(&compare(&before, &before, &["--format", "json"]), 0)).unwrap();
    let colored = locmetry::compare::render_table_with_theme(
        &unchanged,
        Theme::TERMINAL,
        locmetry::LanguageVariants::Split,
    );
    assert!(!colored.contains("38;5;"), "{colored}");
    let source = colored
        .lines()
        .find(|line| line.contains("mSource "))
        .unwrap();
    assert!(
        source.split("\x1b[").any(|cell| cell
            .strip_prefix("90m")
            .is_some_and(|cell| cell.trim() == "0")),
        "{source}"
    );
}
