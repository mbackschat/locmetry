use crate::common::{self, write};
use serde_json::Value;
use std::{collections::BTreeSet, fs, path::Path, process::Output};

fn help(command: &str) -> String {
    let mut cmd = common::locmetry();
    if !command.is_empty() {
        cmd.arg(command);
    }
    let out = cmd.arg("--help").output().unwrap();
    assert!(out.status.success());
    String::from_utf8(out.stdout).unwrap()
}

fn example(text: &str, task: &str) -> Vec<String> {
    common::help_examples(text)
        .into_iter()
        .find(|(label, _)| label == task)
        .unwrap_or_else(|| panic!("missing executable help task: {task}"))
        .1
        .strip_prefix("locmetry ")
        .unwrap()
        .split_whitespace()
        .map(|s| s.trim_matches('\'').to_owned())
        .collect()
}

fn execute(root: &Path, text: &str, task: &str) -> Output {
    let out = common::locmetry()
        .current_dir(root)
        .args(example(text, task))
        .env("NO_COLOR", "1")
        .output()
        .unwrap();
    assert!(
        out.status.success(),
        "{task}: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    out
}

fn fixture(root: &Path) {
    for project in ["service", "api"] {
        write(
            root,
            &format!("{project}/Cargo.toml"),
            "[package]\nname = 'demo'\nversion = '0.1.0'\n",
        );
        write(root, &format!("{project}/src/main.rs"), "fn main() {}\n");
        write(
            root,
            &format!("{project}/tests/smoke.rs"),
            "#[test]\nfn smoke() {}\n",
        );
    }
    write(root, "web/package.json", "{}\n");
    write(root, "web/src/index.ts", "export const a = 1;\n");
    write(root, "repos.toml", "version = 1\nroots = ['web', 'api']\n");
}

fn saved(root: &Path, name: &str) -> Value {
    serde_json::from_str(&fs::read_to_string(root.join(name)).unwrap()).unwrap()
}

#[test]
fn every_visible_option_has_a_workflow_in_the_inventory() {
    let shared = [
        "--no-color",
        "--no-user-config",
        "--output",
        "--print",
        "--files",
        "--format",
        "--show-categories",
        "--verbose",
        "--show-bars",
        "--no-bars",
        "--table-purpose",
        "--table-category",
        "--table-language",
        "--tables-only",
        "--split-languages",
        "--sort",
        "--markdown-depth",
        "--help",
    ];
    let policy = [
        "--root",
        "--config",
        "--exclude",
        "--hidden",
        "--respect-ignore-files",
    ];
    for (command, expected) in [
        (
            "",
            [
                shared.as_slice(),
                policy.as_slice(),
                &[
                    "--subfolder",
                    "--show-folders",
                    "--folder-rows",
                    "--schema",
                    "--guide",
                    "--section",
                    "--version",
                ],
            ]
            .concat(),
        ),
        (
            "workspace",
            [shared.as_slice(), &["--root", "--workspace-file"]].concat(),
        ),
        (
            "compare",
            vec![
                "--no-color",
                "--member",
                "--format",
                "--output",
                "--tables-only",
                "--split-languages",
                "--help",
            ],
        ),
        (
            "explain",
            [
                policy.as_slice(),
                &[
                    "--no-color",
                    "--workspace-file",
                    "--no-user-config",
                    "--format",
                    "--help",
                ],
            ]
            .concat(),
        ),
    ] {
        let text = help(command);
        let actual: BTreeSet<_> = text
            .lines()
            .filter(|line| {
                line.starts_with("      --")
                    || line.starts_with("  -h, --")
                    || line.starts_with("  -V, --")
            })
            .filter_map(|line| line.split_whitespace().find(|word| word.starts_with("--")))
            .map(|word| word.split(['[', '=', ',']).next().unwrap())
            .collect();
        assert_eq!(
            actual,
            expected.into_iter().collect(),
            "assign any new option to a documented workflow: {command}"
        );
    }
}

#[test]
fn root_help_exposes_result_automation_and_policy_boundaries() {
    let text = help("").split_whitespace().collect::<Vec<_>>().join(" ");
    for fact in [
        "Lines = Code + Comments + Blanks",
        "Test/prod = Test Code / Production Code",
        "summary.all_counted",
        "Unavailable counts are not zero",
        "Exit 2 writes a partial report",
        "overrides precede test/build conventions",
        "language mappings",
        "--schema user-config",
        "--table-* options filter tables, not JSON",
        "Build modules infer conventional layouts",
    ] {
        assert!(text.contains(fact), "missing task interpretation: {fact}");
    }
}

#[test]
fn command_summaries_name_every_supported_form() {
    // Root help lists only these summaries, so an agent that stops there must
    // learn both workspace forms and that explain takes several paths
    // (agent regression desk review, 2026-10-21, F9).
    let text = help("").split_whitespace().collect::<Vec<_>>().join(" ");
    for form in [
        "Measure several repositories or one project's folders as one virtual monorepo",
        "Explain selection and classification of existing root-relative files",
    ] {
        assert!(text.contains(form), "missing command form: {form}");
    }
}

#[test]
fn root_scope_and_view_examples_execute_the_displayed_commands() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    write(
        root.path(),
        "service/src/vendor/copied.rs",
        "fn copied() {}\n",
    );
    let text = help("");
    let selected = common::json(
        &execute(
            root.path(),
            &text,
            "Measure selected source, including ignored files, except vendor",
        ),
        0,
    );
    assert_eq!(selected["summary"]["source"]["code"], 1);
    assert_eq!(selected["metadata"]["scope"]["subfolder"], "src");
    let full = common::json(&execute(root.path(), &text, "Read full JSON on stdout"), 0);
    assert_eq!(full["summary"]["source"]["code"], 4);
    assert!(full["files"]
        .as_array()
        .unwrap()
        .iter()
        .any(|f| f["purpose"] == "test"));
    let table = execute(
        root.path(),
        &text,
        "Show adaptive folder detail and production categories",
    );
    assert!(String::from_utf8(table.stdout)
        .unwrap()
        .contains("Comment %"));
    execute(root.path(), &text, "Save a sorted Markdown tree to depth 3");
    assert!(fs::read_to_string(root.path().join("report.md"))
        .unwrap()
        .contains("Source"));
}

#[test]
fn compare_help_captures_full_inputs_and_exports_net_change() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let text = help("compare");
    execute(
        root.path(),
        &text,
        "Capture a full baseline before changing files or policy",
    );
    write(
        root.path(),
        "service/src/main.rs",
        "fn main() {}\nfn added() {}\n",
    );
    execute(
        root.path(),
        &text,
        "Capture the new state after the intended change",
    );
    assert!(
        String::from_utf8(execute(root.path(), &text, "Inspect net change as a table").stdout)
            .unwrap()
            .contains("+1")
    );
    execute(root.path(), &text, "Save the complete comparison");
    assert_eq!(
        saved(root.path(), "change.json")["summary"]["source"]["delta"]["code"],
        1
    );
    execute(root.path(), &text, "Save a Markdown comparison");
    assert!(fs::read_to_string(root.path().join("change.md"))
        .unwrap()
        .contains("+1"));
    execute(root.path(), &text, "Capture a workspace baseline instead");
    assert!(saved(root.path(), "before.json")["metadata"]["members"].is_array());
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    for fact in [
        "not Git additions/deletions",
        "headlines cannot be baselines",
        "matching subfolder restrictions",
        "does not prove source deletion",
        "--schema comparison",
    ] {
        assert!(flat.contains(fact), "{fact}");
    }
}

#[test]
fn workspace_help_measures_both_forms_and_explains_member_identity() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let text = help("workspace");
    execute(
        root.path(),
        &text,
        "Measure roots directly, keeping each member's policy",
    );
    let direct = saved(root.path(), "workspace.json");
    execute(root.path(), &text, "Measure a workspace file");
    let file = saved(root.path(), "workspace.json");
    assert_eq!(direct["summary"], file["summary"]);
    assert_eq!(file["summary"]["members"]["api"]["source"]["code"], 3);
    assert_eq!(file["summary"]["members"]["web"]["source"]["code"], 1);
    execute(
        root.path(),
        &text,
        "Export member, category and language detail",
    );
    assert!(fs::read_to_string(root.path().join("workspace.md"))
        .unwrap()
        .contains("## Members"));
    let explanation = common::json(&execute(root.path(), &text, "Explain a member file"), 0);
    assert_eq!(explanation["member"], "web");
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    for fact in [
        "relative to the workspace file",
        "summary.members",
        "sum(Test Code) / sum(Production Code)",
        "each member's .locmetryrc",
        "unique",
        "non-overlapping",
    ] {
        assert!(flat.contains(fact), "{fact}");
    }
}

#[test]
fn explain_help_executes_root_workspace_and_exclusion_recipes() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let text = help("explain");
    let counted = common::json(
        &execute(root.path(), &text, "Read a complete JSON explanation"),
        0,
    );
    assert_eq!(counted["result"]["state"], "counted");
    assert_eq!(counted["result"]["decision"]["stage"], "module");
    let several = execute(
        root.path(),
        &text,
        "Check several files, one JSON line each",
    );
    let stages: Vec<Value> = String::from_utf8(several.stdout)
        .unwrap()
        .lines()
        .map(|line| {
            serde_json::from_str::<Value>(line).unwrap()["result"]["decision"]["stage"].clone()
        })
        .collect();
    assert_eq!(stages, ["module", "convention"]);
    let member = common::json(&execute(root.path(), &text, "Diagnose a member file"), 0);
    assert_eq!(member["path"], "web/src/index.ts");
    let excluded = common::json(
        &execute(
            root.path(),
            &text,
            "Check an exclusion under a proposed scope",
        ),
        0,
    );
    assert_eq!(excluded["result"]["state"], "excluded");
    let flat = text.split_whitespace().collect::<Vec<_>>().join(" ");
    for fact in [
        "not_selected",
        "unrecognized",
        "unread",
        "parse_failed",
        "symlink",
        "no --output option",
        "root-relative",
        "--schema explanation",
        "JSON Lines",
    ] {
        assert!(flat.contains(fact), "{fact}");
    }
}

#[test]
fn routine_comparison_recipes_make_extra_reference_reading_optional() {
    let root = help("").split_whitespace().collect::<Vec<_>>().join(" ");
    assert!(
        root.contains("For routine capture-and-compare tasks, start with locmetry compare --help")
    );
    assert!(root.contains("Extra guide/schema reading is optional for that route"));
    let compare = help("compare")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    assert!(compare.contains(
        "For two full reports of the same mode and subfolder scope, execute the recipes below"
    ));
    assert!(compare.contains("Inspect summary and findings before drawing conclusions"));
    assert!(compare.contains("Additional guide/schema reading is optional for this route"));
}

#[test]
fn compare_help_drills_into_a_workspace_member_with_scoped_folder_totals() {
    let root = tempfile::tempdir().unwrap();
    fixture(root.path());
    let text = help("compare");
    execute(root.path(), &text, "Capture a workspace baseline instead");
    write(
        root.path(),
        "api/src/main.rs",
        "fn main() {}\nfn added() {}\n",
    );
    write(
        root.path(),
        "web/src/index.ts",
        "export const a = 1;\nexport const b = 2;\nexport const c = 3;\n",
    );
    execute(
        root.path(),
        &text,
        "Capture the workspace after the intended change",
    );
    let output = execute(
        root.path(),
        &text,
        "Inspect one workspace member and its immediate subfolders",
    );
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(text.contains("Member: api (both)"), "{text}");
    let folder = text
        .lines()
        .find(|line| line.contains("src (both)"))
        .unwrap();
    assert_eq!(
        folder.split_whitespace().collect::<Vec<_>>(),
        ["src", "(both)", "1", "2", "+1", "│", "1", "2", "+1"]
    );
    let total = text
        .lines()
        .find(|line| line.starts_with(" Source total"))
        .unwrap();
    assert_eq!(
        total.split_whitespace().collect::<Vec<_>>(),
        ["Source", "total", "3", "4", "+1", "│", "3", "4", "+1"]
    );
    let flat = help("compare")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    for fact in [
        "exact member name",
        "either report",
        "unknown names",
        "Member: NAME",
        "Source total",
        "whole workspace inputs",
    ] {
        assert!(flat.contains(fact), "{fact}");
    }
}

/// Help sizes in bytes at `COLUMNS=100`, measured 2026-10-22 plus about 3%,
/// never above the earlier budget.
/// Agents read help before ordinary use, so growth beyond a budget must be a
/// reviewed decision (docs/agent-regression-checks.md#help-size-guard); size
/// alone does not measure efficiency, so a budget is a stop, not a target.
const HELP_BUDGETS: [(&[&str], usize); 8] = [
    (&["-h"], 4_450),
    (&["--help"], 12_950),
    (&["workspace", "-h"], 2_500),
    (&["workspace", "--help"], 6_900),
    (&["compare", "-h"], 990),
    (&["compare", "--help"], 4_875),
    (&["explain", "-h"], 1_460),
    (&["explain", "--help"], 3_400),
];

#[test]
fn help_stays_within_its_reading_budget() {
    for (args, budget) in HELP_BUDGETS {
        let out = common::locmetry()
            .env("COLUMNS", "100")
            .env("NO_COLOR", "1")
            .args(args)
            .output()
            .unwrap();
        assert!(out.status.success(), "{args:?}");
        let size = out.stdout.len();
        assert!(
            size <= budget,
            "`locmetry {}` prints {size} bytes, over its budget of {budget}; run an agent regression check before raising it",
            args.join(" ")
        );
    }
}
