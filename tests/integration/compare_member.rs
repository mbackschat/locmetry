use crate::common;
use crate::compare::*;
use common::{json, write};
use serde_json::json;
use std::fs;

#[test]
fn member_selection_scopes_every_count_and_detail_without_rescanning() {
    let temp = tempfile::tempdir().unwrap();
    let api = temp.path().join("api");
    let other = temp.path().join("api-extra");
    write(
        &api,
        ".locmetryrc",
        "version = 1\n[classification]\nfallback = \"production\"\n",
    );
    write(&api, "src/main.py", "print(1)\n");
    write(&api, "tests/test_main.py", "assert True\n");
    write(&other, "src/main.py", "print(1)\n");
    let before = save(temp.path(), "before.json", &workspace(&[&api, &other]));
    write(&api, "src/nested/extra.py", "print(2)\nprint(3)\n");
    write(&api, "package.json", "{}\n");
    write(
        &other,
        "src/main.py",
        "print(1)\nprint(2)\nprint(3)\nprint(4)\n",
    );
    let after = save(temp.path(), "after.json", &workspace(&[&api, &other]));
    fs::remove_dir_all(&api).unwrap();
    fs::remove_dir_all(&other).unwrap();
    let all = json(&compare(&before, &after, &["--format", "json"]), 0);
    let selected = json(
        &compare(&before, &after, &["--member", "api", "--format", "json"]),
        0,
    );
    assert_eq!(selected["member"], "api");
    change(&selected["summary"]["source"], 2, 4, 2);
    assert_eq!(selected["summary"]["all_counted"]["delta"]["code"], 3);
    assert_eq!(
        selected["summary"]["composition"]["source"]["matched"]["code"],
        2
    );
    assert_eq!(
        selected["members"]
            .as_object()
            .unwrap()
            .keys()
            .collect::<Vec<_>>(),
        ["api"]
    );
    assert_eq!(selected["before"], all["before"]);
    assert_eq!(selected["after"], all["after"]);
    assert!(selected["folders"]
        .as_object()
        .unwrap()
        .keys()
        .all(|p| p == "api" || p.starts_with("api/")));
    for key in ["breakdown", "packages"] {
        assert!(!selected[key].as_array().unwrap().is_empty());
        assert!(selected[key].as_array().unwrap().iter().all(|row| {
            let p = row["folder"].as_str().unwrap();
            p == "api" || p.starts_with("api/")
        }));
    }
    assert!(selected["findings"]
        .as_array()
        .unwrap()
        .iter()
        .all(|f| f["member"].is_null() || f["member"] == "api"));
    let headline = json(
        &compare(
            &before,
            &after,
            &["--member", "api", "--format", "headline"],
        ),
        0,
    );
    assert_eq!(headline["member"], "api");
    assert_eq!(headline["summary"], selected["summary"]);
    assert!(headline.get("folders").is_none());
    let schema = json(&run(&["--schema", "comparison"]), 0);
    jsonschema::validator_for(&schema)
        .unwrap()
        .validate(&selected)
        .unwrap();
    jsonschema::validator_for(&schema)
        .unwrap()
        .validate(&headline)
        .unwrap();
    let report: locmetry::compare::Comparison = serde_json::from_value(selected).unwrap();
    let styled = locmetry::compare::render_table_with_theme(
        &report,
        locmetry::style::Theme::TERMINAL,
        locmetry::LanguageVariants::Split,
    );
    let title = styled.lines().next().unwrap();
    let provenance = styled
        .lines()
        .find(|line| line.starts_with("Input metadata,"))
        .unwrap();
    assert!(title.starts_with("Net change:"), "{title}");
    assert!(
        !title.contains('\x1b') && !provenance.contains('\x1b'),
        "{styled}"
    );
    let full: locmetry::compare::Comparison = serde_json::from_value(all).unwrap();
    let full_styled = locmetry::compare::render_table_with_theme(
        &full,
        locmetry::style::Theme::TERMINAL,
        locmetry::LanguageVariants::Split,
    );
    assert!(
        full_styled.starts_with("\x1b[1mNet change:"),
        "{full_styled}"
    );
    assert!(
        styled.contains("\x1b[1mMember: api (both)\x1b[0m"),
        "{styled}"
    );
    assert_eq!(
        common::unstyled(&styled),
        locmetry::compare::render_table_with_theme(
            &report,
            locmetry::style::Theme::PLAIN,
            locmetry::LanguageVariants::Split
        )
    );
    assert!(
        locmetry::compare::render_markdown(&report, locmetry::LanguageVariants::Split)
            .contains("**Member: api (both)**")
    );
    for format in ["table", "markdown"] {
        let output = compare(&before, &after, &["--member", "api", "--format", format]);
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout).unwrap();
        assert!(text.contains("Member: api (both)"), "{text}");
        assert!(text.contains("Source total"), "{text}");
        assert!(text.contains("Test/prod (Code): 1.00 → 0.33"), "{text}");
        assert!(text.contains("src (both)"), "{text}");
        assert_eq!(
            text.contains("src/nested (after only)"),
            format == "markdown",
            "{text}"
        );
    }
}

#[test]
fn member_selection_preserves_arrivals_departures_and_zero_count_presence() {
    let temp = tempfile::tempdir().unwrap();
    let api = temp.path().join("api");
    let worker = temp.path().join("worker");
    let empty = temp.path().join("empty");
    write(&api, "src/main.py", "print(1)\n");
    write(&worker, "src/main.py", "print(2)\n");
    fs::create_dir(&empty).unwrap();
    let before = save(temp.path(), "before.json", &workspace(&[&api]));
    let after = save(temp.path(), "after.json", &workspace(&[&worker, &empty]));
    for (name, presence, before_code, after_code, delta) in [
        ("api", "before_only", 1, 0, -1),
        ("worker", "after_only", 0, 1, 1),
        ("empty", "after_only", 0, 0, 0),
    ] {
        let value = json(
            &compare(&before, &after, &["--member", name, "--format", "json"]),
            0,
        );
        change(&value["summary"]["source"], before_code, after_code, delta);
        assert_eq!(value["members"][name]["presence"], presence);
        assert_eq!(value["presence_summary"]["members"][presence], 1);
        assert_eq!(
            value["presence_summary"]["folders"][presence],
            if name == "empty" { 1 } else { 2 }
        );
        assert_eq!(
            value["summary"]["composition"]["source"]["matched"]["code"],
            0
        );
        assert_eq!(
            value["summary"]["composition"]["source"][if presence == "before_only" {
                "leaving"
            } else {
                "entering"
            }]["code"],
            before_code + after_code
        );
        let text = String::from_utf8(
            compare(&before, &after, &["--member", name, "--format", "table"]).stdout,
        )
        .unwrap();
        assert!(
            text.contains(&format!("Member: {name} ({})", presence.replace('_', " "))),
            "{text}"
        );
        assert!(
            !text.contains("No recorded count or presence changes."),
            "{text}"
        );
    }
}

#[test]
fn member_selection_rejects_unknown_names_single_roots_and_invalid_inputs_before_writing() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("api");
    write(&root, "src/main.py", "print(1)\n");
    let single = save(temp.path(), "single.json", &measure(&root));
    let report = workspace(&[&root]);
    let full = save(temp.path(), "workspace.json", &report);
    let destination = temp.path().join("result.json");
    fs::write(&destination, "keep me").unwrap();
    for (a, b, name, error) in [
        (&full, &full, "ap", "Unknown workspace member"),
        (&full, &full, "API", "Unknown workspace member"),
        (&full, &full, "api/src", "Unknown workspace member"),
        (
            &single,
            &single,
            "api",
            "'--member' requires two workspace reports",
        ),
    ] {
        let output = compare(
            a,
            b,
            &["--member", name, "--output", destination.to_str().unwrap()],
        );
        assert_eq!(output.status.code(), Some(1));
        assert!(
            String::from_utf8_lossy(&output.stderr).contains(error),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stdout.is_empty());
        assert_eq!(fs::read_to_string(&destination).unwrap(), "keep me");
    }
    let mut invalid = report;
    invalid["summary"]["source"]["code"] = serde_json::json!(999);
    let invalid = save(temp.path(), "invalid.json", &invalid);
    let output = compare(
        &full,
        &invalid,
        &["--member", "api", "--output", destination.to_str().unwrap()],
    );
    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stderr).contains("Invalid after measurement"));
    assert_eq!(fs::read_to_string(&destination).unwrap(), "keep me");
}

#[test]
fn member_selection_retains_whole_input_partial_status_and_provenance() {
    let temp = tempfile::tempdir().unwrap();
    let api = temp.path().join("api");
    let web = temp.path().join("web");
    write(&api, "main.py", "print(1)\n");
    write(&web, "main.py", "print(2)\n");
    let mut report = workspace(&[&api, &web]);
    report["metadata"]["backend_inaccurate"] = json!(true);
    report["metadata"]["coverage"]["selected_files"] = json!(3);
    report["metadata"]["coverage"]["parse_failures"] = json!([{
        "path": "web/bad.ipynb", "reason": "invalid_notebook", "message": "invalid"
    }]);
    report["metadata"]["members"][1]["coverage"]["selected_files"] = json!(2);
    report["metadata"]["members"][1]["coverage"]["parse_failed_files"] = json!(1);
    let saved = save(temp.path(), "input.json", &report);
    let selected = json(
        &compare(&saved, &saved, &["--member", "api", "--format", "json"]),
        2,
    );
    change(&selected["summary"]["source"], 1, 1, 0);
    assert_eq!(
        selected["before"]["metadata"]["coverage"],
        report["metadata"]["coverage"]
    );
    assert!(selected["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["code"] == "coverage_gap" && f["member"].is_null()));
    assert!(!selected["findings"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["member"] == "web"));
    for format in ["table", "markdown"] {
        let out = compare(&saved, &saved, &["--member", "api", "--format", format]);
        assert_eq!(out.status.code(), Some(2));
        let text = String::from_utf8(out.stdout).unwrap();
        assert!(text.contains("whole workspace inputs"), "{text}");
        assert!(
            text.contains("Members before: web. Members after: web"),
            "{text}"
        );
    }
}
