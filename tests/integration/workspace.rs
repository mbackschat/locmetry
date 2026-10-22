//! `locmetry workspace` measures several roots as one virtual monorepo; the
//! oracle throughout is a regular run of each member (docs/PRODUCT.md, Workspace
//! Mode).
use crate::common;
use common::{json, write_all};
use serde_json::{json, Map, Value};
use std::{fs, path::Path};

/// The binary isolated from the developer's own user defaults.
fn locmetry(cwd: &Path, args: &[&str]) -> std::process::Output {
    common::locmetry()
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap()
}

fn stderr_of(output: &std::process::Output) -> String {
    assert!(!output.status.success(), "unexpected success");
    String::from_utf8_lossy(&output.stderr).into_owned()
}

const WEB_CONFIG: &str = "version = 1\n[scope]\nexclude = [\"third_party\"]\n[[rules]]\nid = \"tooling\"\nmatch = [\"scripts/**\"]\npurpose = \"production\"\ncategory = \"tooling\"\n[[rules]]\nid = \"ghost\"\nmatch = [\"nowhere/**\"]\npurpose = \"production\"\n";

/// Two sibling repositories, `web` (npm, with a configuration) and `api`
/// (Cargo, with a Git submodule), beside an unrelated folder.
fn workspace() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    write_all(
        &temp.path().join("web"),
        &[
            (".locmetryrc", WEB_CONFIG),
            ("package.json", "{}\n"),
            ("src/index.ts", "export const a = 1;\nexport const b = 2;\n"),
            ("src/index.test.ts", "test('a', () => {});\n"),
            ("scripts/release.ts", "let r = 1;\n"),
            ("prototype/p.ts", "let p = 1;\n"),
            ("third_party/x.ts", "let x = 1;\n"),
            ("README.md", "# Web\n"),
        ],
    );
    write_all(
        &temp.path().join("api"),
        &[
            ("Cargo.toml", "[package]\nname = \"api\"\n"),
            ("src/lib.rs", "pub fn a() {}\n"),
            ("tests/it.rs", "#[test]\nfn t() {}\n"),
            (
                ".gitmodules",
                "[submodule \"proto\"]\n\tpath = vendor/proto\n",
            ),
            ("vendor/proto/gen.rs", "fn g() {}\n"),
            ("data/seed.json", "{}\n"),
        ],
    );
    write_all(&temp.path().join("other"), &[("o.ts", "let o = 1;\n")]);
    temp
}

/// Full report through `--output -`, without the measurement time.
fn full_report(cwd: &Path, args: &[&str]) -> Value {
    let args = [args, &["--output", "-", "--files"]].concat();
    let mut report = json(&locmetry(cwd, &args), 0);
    report["metadata"]
        .as_object_mut()
        .unwrap()
        .remove("measured_at_unix_seconds");
    report
}

fn prefixed(member: &str, path: &str) -> String {
    match path {
        "." => member.to_string(),
        _ => format!("{member}/{path}"),
    }
}

fn add_counts(total: &mut Value, counts: &Value) {
    for (key, value) in counts.as_object().unwrap() {
        let sum = total[key].as_u64().unwrap_or(0) + value.as_u64().unwrap();
        total[key] = json!(sum);
    }
}

/// Sums `kind:purpose` counts over reports, the totals a merge must reproduce.
fn summed_kind_purpose(reports: &[&Value]) -> Map<String, Value> {
    let mut total = Map::new();
    for report in reports {
        for (key, counts) in report["summary"]["by_kind_purpose"].as_object().unwrap() {
            add_counts(total.entry(key.clone()).or_insert(json!({})), counts);
        }
    }
    total
}

#[test]
fn workspace_merges_members_as_prefixed_top_level_folders() {
    let temp = workspace();
    let base = temp.path();
    let web = full_report(&base.join("web"), &[]);
    let api = full_report(&base.join("api"), &[]);
    let merged = full_report(base, &["workspace", "--root", "web", "--root", "api"]);

    assert_eq!(merged["project"], "api + web");
    let mut expected_files: Vec<Value> = [("api", &api), ("web", &web)]
        .into_iter()
        .flat_map(|(member, report)| {
            report["files"].as_array().unwrap().iter().map(move |file| {
                let mut file = file.clone();
                file["path"] = json!(prefixed(member, file["path"].as_str().unwrap()));
                file
            })
        })
        .collect();
    expected_files.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    assert_eq!(merged["files"], json!(expected_files));
    assert_eq!(
        merged["summary"]["by_kind_purpose"],
        Value::Object(summed_kind_purpose(&[&web, &api]))
    );
    let top: Vec<&String> = merged["summary"]["top_folders"]
        .as_object()
        .unwrap()
        .keys()
        .collect();
    assert_eq!(top, ["api", "web"]);
    assert_eq!(
        merged["summary"]["submodules"],
        json!([{"path": "api/vendor/proto", "files": 1, "code": 1}])
    );
    assert_eq!(
        merged["summary"]["build_modules"],
        json!({"cargo": {"modules": 1, "code": 1}, "npm": {"modules": 1, "code": 2}})
    );
    assert_eq!(
        merged["metadata"]["modules"],
        json!([{"path": "api", "detector": "cargo"}, {"path": "web", "detector": "npm"}])
    );
    assert_eq!(
        merged["summary"]["largest_module_files"],
        json!([
            {"path": "web/src/index.ts", "lines": 2, "code": 2, "module": "web", "detector": "npm"},
            {"path": "api/src/lib.rs", "lines": 1, "code": 1, "module": "api", "detector": "cargo"},
        ])
    );
    assert_eq!(
        merged["summary"]["unclaimed_folders"],
        json!([
            {"folder": "api/vendor/proto", "files": 1, "code": 1},
            {"folder": "web/prototype", "files": 1, "code": 1},
            {"folder": "web/third_party", "files": 1, "code": 1},
        ])
    );
    assert_eq!(
        merged["metadata"]["coverage"]["selected_files"],
        json!(
            api["metadata"]["coverage"]["selected_files"]
                .as_u64()
                .unwrap()
                + web["metadata"]["coverage"]["selected_files"]
                    .as_u64()
                    .unwrap()
        )
    );

    let metadata = merged["metadata"].as_object().unwrap();
    for single_root in ["root", "scope", "config_path", "config"] {
        assert!(!metadata.contains_key(single_root), "{single_root}");
    }
    let members = merged["metadata"]["members"].as_array().unwrap();
    let names: Vec<&str> = members
        .iter()
        .map(|m| m["name"].as_str().unwrap())
        .collect();
    assert_eq!(names, ["api", "web"]);
    let web_member = &members[1];
    assert_eq!(web_member["root"], web["metadata"]["root"]);
    let config = Path::new(web["metadata"]["root"].as_str().unwrap()).join(".locmetryrc");
    assert_eq!(web_member["config_path"], json!(config));
    assert_eq!(web_member["scope"], web["metadata"]["scope"]);
    assert_eq!(web_member["config"], web["metadata"]["config"]);
    assert_eq!(web_member["unused_entries"], json!(["ghost"]));
    assert_eq!(web_member["unused_exclusions"], json!(["third_party"]));
    assert_eq!(members[0]["config_path"], Value::Null);
    assert_eq!(members[0]["unused_entries"], json!([]));
    let summary = merged["summary"].as_object().unwrap();
    assert!(!summary.contains_key("unused_entries"));
    assert!(!summary.contains_key("unused_exclusions"));
}

/// The headline stays compact: members keep their findings but not their
/// effective configuration, which only the report file carries.
#[test]
fn workspace_headline_drops_member_configurations() {
    let temp = workspace();
    let headline = json(
        &locmetry(
            temp.path(),
            &["workspace", "--root", "web", "--root", "api"],
        ),
        0,
    );
    let members = headline["metadata"]["members"].as_array().unwrap();
    assert!(members.iter().all(|member| member.get("config").is_none()));
    assert_eq!(members[1]["unused_entries"], json!(["ghost"]));
    assert!(headline.get("files").is_none());
}

/// Locks determinism: members run in parallel, yet the order of `--root`
/// flags and of thread completion never changes the report.
#[test]
fn workspace_reports_do_not_depend_on_root_order() {
    let temp = workspace();
    let forward = full_report(
        temp.path(),
        &["workspace", "--root", "web", "--root", "api"],
    );
    let backward = full_report(
        temp.path(),
        &["workspace", "--root", "api", "--root", "web"],
    );
    assert_eq!(forward, backward);
}

#[test]
fn workspace_notes_name_the_member_of_each_finding() {
    let temp = workspace();
    let args = ["workspace", "--root", "web", "--root", "api", "--format"];
    let table = String::from_utf8(
        locmetry(temp.path(), &[&args[..], &["table", "--verbose"]].concat()).stdout,
    )
    .unwrap();
    let roots: Vec<_> = ["api", "web"]
        .map(|name| {
            dunce::canonicalize(temp.path().join(name))
                .unwrap()
                .display()
                .to_string()
        })
        .into();
    assert!(
        table.starts_with(" ─") && table.lines().nth(1).unwrap().starts_with(" Language "),
        "{table}"
    );
    assert!(
        table.contains(&format!("\nRoots: {}\nMeasured at: ", roots.join(", "))),
        "{table}"
    );
    assert!(
        table.contains("Entries that decided no file: ghost (web)."),
        "{table}"
    );
    assert!(
        table.contains("Exclusions that excluded nothing: third_party (web)."),
        "{table}"
    );
    assert!(
        table.contains("Git submodules: api/vendor/proto"),
        "{table}"
    );
    let markdown =
        String::from_utf8(locmetry(temp.path(), &[&args[..], &["markdown"]].concat()).stdout)
            .unwrap();
    assert!(
        markdown.contains("Entries that decided no file: ghost (web)."),
        "{markdown}"
    );
}

/// Single-root settings belong to each member's configuration, and a flag
/// before the subcommand would leave the mode ambiguous.
#[test]
fn workspace_rejects_single_root_flags() {
    let temp = workspace();
    let roots = ["workspace", "--root", "web", "--root", "api"];
    for extra in [
        &["--config", "web/.locmetryrc"][..],
        &["--exclude", "x/**"],
        &["--hidden", "false"],
        &["--respect-ignore-files", "false"],
    ] {
        let stderr = stderr_of(&locmetry(temp.path(), &[&roots[..], extra].concat()));
        assert!(stderr.contains(extra[0]), "{extra:?}: {stderr}");
    }
    let before = stderr_of(&locmetry(
        temp.path(),
        &["--files", "workspace", "--root", "web"],
    ));
    assert!(before.contains("workspace"), "{before}");
    stderr_of(&locmetry(temp.path(), &["workspace"]));
    let accepted = locmetry(
        temp.path(),
        &[
            &roots[..],
            &[
                "--no-user-config",
                "--show-categories",
                "--sort",
                "name",
                "--format",
                "table",
            ],
        ]
        .concat(),
    );
    assert!(
        accepted.status.success(),
        "{}",
        String::from_utf8_lossy(&accepted.stderr)
    );
}

#[test]
fn workspace_members_must_be_distinct_and_disjoint() {
    let temp = workspace();
    write_all(&temp.path().join("copy/web"), &[("a.ts", "let a = 1;\n")]);
    for (roots, message) in [
        (&["web", "copy/web"][..], "web"),
        (&["web", "./web"], "web"),
        (&["web", "web/src"], "inside"),
        (&["web", "missing"], "missing"),
    ] {
        let args: Vec<&str> = std::iter::once("workspace")
            .chain(roots.iter().flat_map(|root| ["--root", root]))
            .collect();
        let stderr = stderr_of(&locmetry(temp.path(), &args));
        assert!(stderr.contains(message), "{roots:?}: {stderr}");
    }
}

#[test]
fn an_invalid_member_configuration_fails_the_workspace_naming_the_member() {
    let temp = workspace();
    write_all(
        &temp.path().join("api"),
        &[(".locmetryrc", "version = 2\n")],
    );
    let stderr = stderr_of(&locmetry(
        temp.path(),
        &["workspace", "--root", "web", "--root", "api"],
    ));
    assert!(stderr.contains("Workspace member 'api'"), "{stderr}");
}

#[test]
fn workspace_reports_must_be_written_outside_every_member() {
    let temp = workspace();
    let stderr = stderr_of(&locmetry(
        temp.path(),
        &[
            "workspace",
            "--root",
            "web",
            "--root",
            "api",
            "--output",
            "api/report.json",
        ],
    ));
    assert!(stderr.contains("outside"), "{stderr}");
    let report = temp.path().join("report.json");
    let output = locmetry(
        temp.path(),
        &[
            "workspace",
            "--root",
            "web",
            "--root",
            "api",
            "--output",
            report.to_str().unwrap(),
        ],
    );
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(report.is_file());
}

/// A workspace file spares repeating `--root`: its roots are relative to the
/// file, not the working directory, so the run is the same from anywhere.
#[test]
fn a_workspace_file_lists_roots_relative_to_itself() {
    let temp = workspace();
    let base = temp.path();
    let api = dunce::canonicalize(base.join("api")).unwrap();
    let file = base.join("defs/cg.toml");
    write_all(
        base,
        &[(
            "defs/cg.toml",
            &format!(
                "version = 1\nroots = [\"../web\", {:?}]\n",
                api.to_str().unwrap()
            ),
        )],
    );
    let from_roots = full_report(base, &["workspace", "--root", "web", "--root", "api"]);
    let mut from_file = full_report(
        &base.join("other"),
        &["workspace", "--workspace-file", "../defs/cg.toml"],
    );
    assert_eq!(
        from_file["metadata"]["workspace_path"],
        json!(dunce::canonicalize(&file).unwrap())
    );
    from_file["metadata"]
        .as_object_mut()
        .unwrap()
        .remove("workspace_path");
    assert_eq!(from_file, from_roots);
    assert!(from_roots["metadata"].get("workspace_path").is_none());
}

/// Like a project configuration, a workspace file is measurement policy, not
/// content, so the member that contains it does not count it.
#[test]
fn a_workspace_file_inside_a_member_is_not_measured() {
    let temp = workspace();
    write_all(
        temp.path(),
        &[(
            "web/workspace.toml",
            "version = 1\nroots = [\".\", \"../api\"]\n",
        )],
    );
    let report = full_report(
        temp.path(),
        &["workspace", "--workspace-file", "web/workspace.toml"],
    );
    let paths: Vec<&str> = report["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| file["path"].as_str().unwrap())
        .collect();
    assert!(!paths.contains(&"web/workspace.toml"), "{paths:?}");
    assert_eq!(
        report["metadata"]["members"][1]["unused_exclusions"],
        json!(["third_party"])
    );
}

#[test]
fn invalid_workspace_files_fail_before_measuring() {
    let temp = workspace();
    write_all(
        temp.path(),
        &[
            ("v2.toml", "version = 2\nroots = [\"web\"]\n"),
            (
                "extra.toml",
                "version = 1\nroots = [\"web\"]\nexclude = [\"x\"]\n",
            ),
            ("ok.toml", "version = 1\nroots = [\"web\"]\n"),
        ],
    );
    for (file, message) in [
        ("v2.toml", "version"),
        ("extra.toml", "exclude"),
        ("missing.toml", "missing.toml"),
    ] {
        let stderr = stderr_of(&locmetry(
            temp.path(),
            &["workspace", "--workspace-file", file],
        ));
        assert!(stderr.contains(message), "{file}: {stderr}");
    }
    let both = stderr_of(&locmetry(
        temp.path(),
        &["workspace", "--workspace-file", "ok.toml", "--root", "api"],
    ));
    assert!(both.contains("--root"), "{both}");
    let overwrite = stderr_of(&locmetry(
        temp.path(),
        &[
            "workspace",
            "--workspace-file",
            "ok.toml",
            "--output",
            "ok.toml",
        ],
    ));
    assert!(overwrite.contains("overwrite"), "{overwrite}");
}

/// The summary keys a member's totals repeat; the oracle is that member's
/// regular run.
const MEMBER_TOTALS: [&str; 6] = [
    "all_counted",
    "source",
    "by_kind_purpose",
    "source_purpose",
    "source_categories",
    "source_languages",
];

#[test]
fn workspace_summaries_break_totals_down_by_member() {
    let temp = workspace();
    let base = temp.path();
    let merged = full_report(base, &["workspace", "--root", "web", "--root", "api"]);
    let headline = json(
        &locmetry(base, &["workspace", "--root", "web", "--root", "api"]),
        0,
    );
    let members: Vec<&String> = merged["summary"]["members"]
        .as_object()
        .unwrap()
        .keys()
        .collect();
    assert_eq!(members, ["api", "web"]);
    for (index, member) in ["api", "web"].into_iter().enumerate() {
        let own = full_report(&base.join(member), &[]);
        let expected: Map<String, Value> = MEMBER_TOTALS
            .iter()
            .filter_map(|key| Some((key.to_string(), own["summary"].get(*key)?.clone())))
            .collect();
        assert_eq!(
            merged["summary"]["members"][member],
            Value::Object(expected)
        );
        assert_eq!(
            headline["summary"]["members"][member],
            merged["summary"]["members"][member]
        );
        let coverage = &own["metadata"]["coverage"];
        assert_eq!(
            merged["metadata"]["members"][index]["coverage"],
            json!({
                "selected_files": coverage["selected_files"],
                "counted_files": coverage["counted_files"],
                "mapped_files": coverage["mapped_files"],
                "unrecognized_files": coverage["unrecognized_files"],
                "unread_files": coverage["unread"].as_array().unwrap().len(),
                "parse_failed_files": coverage["parse_failures"].as_array().unwrap().len(),
            })
        );
    }
}

/// Cells of the table row whose first cell is `label`, below the header
/// that starts with `header`; column-group separators are not cells.
fn table_row(table: &str, header: &str, label: &str) -> Vec<String> {
    table
        .lines()
        .skip_while(|line| !line.trim_start().starts_with(header))
        .find(|line| line.split_whitespace().next() == Some(label))
        .unwrap_or_else(|| panic!("no {label} row below {header} in:\n{table}"))
        .split_whitespace()
        .filter(|cell| *cell != "│")
        .map(String::from)
        .collect()
}

#[test]
fn workspace_table_compares_repositories() {
    let temp = workspace();
    let output = locmetry(
        temp.path(),
        &[
            "workspace",
            "--root",
            "web",
            "--root",
            "api",
            "--format",
            "table",
            "--verbose",
        ],
    );
    let table = String::from_utf8(output.stdout).unwrap();
    assert_eq!(
        table_row(&table, "Member", "Member"),
        [
            "Member",
            "Lines",
            "Code",
            "Prod.",
            "Test",
            "Other",
            "Test/prod",
            "Share"
        ]
    );
    assert_eq!(
        table_row(&table, "Member", "web"),
        ["web", "6", "6", "3", "1", "2", "0.33", "60.0%"]
    );
    assert_eq!(
        table_row(&table, "Member", "api"),
        ["api", "4", "4", "1", "2", "1", "2.00", "40.0%"]
    );
    assert_eq!(
        table_row(&table, "Member", "Total"),
        ["Total", "10", "10", "4", "3", "3", "0.75", "100.0%"]
    );
    let row_index = |label: &str| {
        table
            .lines()
            .skip_while(|line| !line.starts_with(" Member"))
            .position(|line| line.starts_with(&format!(" {label} ")))
    };
    assert!(
        row_index("web") < row_index("api"),
        "largest first:\n{table}"
    );
    assert!(!table.contains(" Folder "), "{table}");
    assert!(
        table.contains(" Production + Test + Other = Code · Test/prod = Test / Production (Code) · Share = Code / workspace Code\n"),
        "{table}"
    );
    assert!(
        table.contains(" Unclaimed source by member: 'api' has 1 unclaimed file (1 code line), 'web' has 2 unclaimed files (2 code lines).\n"),
        "{table}"
    );
}

#[test]
fn workspace_markdown_breaks_categories_and_languages_down_by_repository() {
    let temp = workspace();
    let output = locmetry(
        temp.path(),
        &[
            "workspace",
            "--root",
            "web",
            "--root",
            "api",
            "--format",
            "markdown",
        ],
    );
    let markdown = String::from_utf8(output.stdout).unwrap();
    for expected in [
        "## Members\n",
        "| Member | Lines | **Code** | Production | Test | Other | Test/prod | Share |\n",
        "| web | 6 | 6 | 3 | 1 | 2 | 0.33 | 60.0% |\n",
        "| **Total** | **10** | **10** | **4** | **3** | **3** | **0.75** | **100.0%** |\n",
        "Production + Test + Other = Code · Test/prod = Test / Production (Code) · Share = Code / workspace Code",
        "Each total is the sum of its member columns.",
        "## Source categories by member\n",
        "| Category | api Lines | api Code | web Lines | web Code | Total Lines | Total Code |\n",
        "| test:test | 2 | 2 | 1 | 1 | 3 | 3 |\n",
        "| production:tooling | 0 | 0 | 1 | 1 | 1 | 1 |\n",
        "## Source languages by member\n",
        "| Language | api Lines | api Code | web Lines | web Code | Total Lines | Total Code |\n",
        "| TypeScript | 0 | 0 | 6 | 6 | 6 | 6 |\n",
        "| Rust | 4 | 4 | 0 | 0 | 4 | 4 |\n",
        "Members to review: 'api' has 1 unclaimed file (1 code line), 'web' has 2 unclaimed files (2 code lines).",
    ] {
        assert!(markdown.contains(expected), "{expected:?} missing in:\n{markdown}");
    }
}

/// A regular run keeps its folder rows; the repository dimension exists only
/// in workspace mode.
#[test]
fn regular_runs_have_no_repository_dimension() {
    let temp = workspace();
    let web = temp.path().join("web");
    let headline = json(&locmetry(&web, &[]), 0);
    assert!(headline["summary"].get("members").is_none());
    for folders in [false, true] {
        let mut args = vec!["--format", "table"];
        if folders {
            args.push("--show-folders");
        }
        let table = String::from_utf8(locmetry(&web, &args).stdout).unwrap();
        assert_eq!(table.contains(" Folder "), folders, "{table}");
        assert!(!table.contains(" Member "), "{table}");
    }
}

/// Workspace bars chart repository shares under the repository table, which
/// takes the folder table's place, so the shares cover the workspace once.
#[test]
fn workspace_bars_chart_repository_shares() {
    let temp = workspace();
    // Narrow enough that no bar fits beside a table, so both charts appear.
    let output = common::locmetry()
        .current_dir(temp.path())
        .env("COLUMNS", "60")
        .args([
            "workspace",
            "--root",
            "web",
            "--root",
            "api",
            "--format",
            "table",
            "--show-bars",
        ])
        .output()
        .unwrap();
    assert!(output.status.success());
    let table = String::from_utf8(output.stdout).unwrap();
    let charts: Vec<_> = table.split("\n Code share ").skip(1).collect();
    assert_eq!(charts.len(), 2, "{table}");
    let repositories = charts[1].split("\n\n").next().unwrap();
    let shares: Vec<f64> = repositories
        .lines()
        .skip(1)
        .map(|line| {
            assert!(
                line.starts_with(" web ") || line.starts_with(" api "),
                "{line}"
            );
            line.split_whitespace()
                .nth(1)
                .unwrap()
                .trim_end_matches('%')
                .parse()
                .unwrap()
        })
        .collect();
    assert_eq!(shares.len(), 2, "{repositories}");
    assert!(
        (shares.iter().sum::<f64>() - 100.0).abs() < 0.11,
        "{repositories}"
    );
}

/// The percentage column comment bars add is set apart like every ratio, also
/// after the repository table's Share column.
#[test]
fn comment_bars_separate_their_percentage_column_after_share() {
    let temp = workspace();
    let output = common::locmetry()
        .env("COLUMNS", "300")
        .current_dir(temp.path())
        .args([
            "workspace",
            "--root",
            "web",
            "--root",
            "api",
            "--format",
            "table",
            "--show-bars=comments",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{}", stderr_of(&output));
    let table = String::from_utf8(output.stdout).unwrap();
    let header = table
        .lines()
        .find(|line| line.starts_with(" Member "))
        .unwrap();
    assert!(
        header.ends_with(" Share │ Comment %  Comment density"),
        "{header}"
    );
}

/// A workspace prints its repository table beside the saved report too.
#[test]
fn workspace_print_shows_the_table_beside_the_saved_report() {
    let temp = workspace();
    let saved = temp.path().join("report.json");
    let output = locmetry(
        temp.path(),
        &[
            "workspace",
            "--root",
            "web",
            "--root",
            "api",
            "--output",
            saved.to_str().unwrap(),
            "--print",
            "table",
        ],
    );
    assert!(output.status.success());
    let printed = String::from_utf8(output.stdout).unwrap();
    assert!(printed.contains("\n Member "), "{printed}");
    assert!(fs::read_to_string(saved).unwrap().contains("\"members\""));
}

const MONOREPO_CONFIG: &str = "version = 1
[scope]
exclude = [\"web/third_party/**\", \"gone/**\"]
[[rules]]
id = \"web-tooling\"
match = [\"web/scripts/**\"]
purpose = \"production\"
category = \"tooling\"
[[rules]]
id = \"api-legacy\"
match = [\"api/legacy/**\"]
purpose = \"other\"
category = \"legacy\"
[[rules]]
id = \"docs-tooling\"
match = [\"docs/tools/**\"]
purpose = \"production\"
category = \"tooling\"
";

/// One repository, one policy, two component folders and a folder outside
/// both; the workspace file names the components.
fn monorepo() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    write_all(
        &temp.path().join("mono"),
        &[
            (".locmetryrc", MONOREPO_CONFIG),
            (
                "workspace.toml",
                "version = 1\nproject = \".\"\nfolders = [\"web\", \"api\"]\n",
            ),
            ("web/package.json", "{}\n"),
            ("web/src/a.ts", "export const a = 1;\n"),
            ("web/src/a.test.ts", "test('a', () => {});\n"),
            ("web/scripts/r.ts", "let r = 1;\n"),
            ("web/third_party/x.ts", "let x = 1;\n"),
            ("api/Cargo.toml", "[package]\nname = \"api\"\n"),
            ("api/src/lib.rs", "pub fn a() {}\n"),
            ("api/legacy/old.rs", "fn old() {}\n"),
            ("docs/tools/gen.ts", "let g = 1;\n"),
        ],
    );
    temp
}

/// Folder members are the project's own `--subfolder` runs: same policy, same
/// module context and the same project-relative paths, so those runs are the
/// oracle for every file and every member total.
#[test]
fn folder_members_share_the_project_policy_and_match_subfolder_runs() {
    let temp = monorepo();
    let project = temp.path().join("mono");
    let report = full_report(
        temp.path(),
        &["workspace", "--workspace-file", "mono/workspace.toml"],
    );
    let mut expected_files = Vec::new();
    for member in ["api", "web"] {
        let run = full_report(temp.path(), &["--root", "mono", "--subfolder", member]);
        expected_files.extend(run["files"].as_array().unwrap().iter().cloned());
        for key in MEMBER_TOTALS {
            assert_eq!(
                report["summary"]["members"][member][key], run["summary"][key],
                "{member} {key}"
            );
        }
    }
    expected_files.sort_by_key(|file| file["path"].as_str().unwrap().to_string());
    assert_eq!(report["files"], json!(expected_files));
    let members = report["metadata"]["members"].as_array().unwrap();
    let project = dunce::canonicalize(&project).unwrap();
    for (member, folder) in members.iter().zip(["api", "web"]) {
        assert_eq!(member["name"], folder);
        assert_eq!(member["root"], json!(project));
        assert_eq!(member["scope"]["subfolder"], folder);
        assert_eq!(member["config_path"], json!(project.join(".locmetryrc")));
    }
}

/// One policy means one set of findings: an entry is unused only when no
/// member folder used it, judged against the measured folders.
#[test]
fn folder_members_report_unused_policy_once_for_the_workspace() {
    let temp = monorepo();
    let report = full_report(
        temp.path(),
        &["workspace", "--workspace-file", "mono/workspace.toml"],
    );
    assert_eq!(report["summary"]["unused_entries"], json!(["docs-tooling"]));
    assert_eq!(report["summary"]["unused_exclusions"], json!(["gone/**"]));
    for member in report["metadata"]["members"].as_array().unwrap() {
        assert_eq!(member["unused_entries"], json!([]), "{member}");
        assert_eq!(member["unused_exclusions"], json!([]), "{member}");
    }
    let verbose = String::from_utf8(
        locmetry(
            temp.path(),
            &[
                "workspace",
                "--workspace-file",
                "mono/workspace.toml",
                "--format",
                "table",
                "--verbose",
                "--no-color",
            ],
        )
        .stdout,
    )
    .unwrap();
    assert!(verbose.contains("docs-tooling"), "{verbose}");
    assert!(!verbose.contains("docs-tooling ("), "{verbose}");
    assert!(verbose.contains("Project: "), "{verbose}");
    assert!(verbose.contains("(folders: api, web)"), "{verbose}");
}

#[test]
fn folder_members_explain_member_paths_with_the_project_policy() {
    let temp = monorepo();
    let explanation = json(
        &locmetry(
            temp.path(),
            &[
                "explain",
                "web/scripts/r.ts",
                "--workspace-file",
                "mono/workspace.toml",
                "--format",
                "json",
            ],
        ),
        0,
    );
    assert_eq!(explanation["path"], "web/scripts/r.ts");
    assert_eq!(explanation["member"], "web");
    assert_eq!(explanation["result"]["file"]["path"], "web/scripts/r.ts");
    assert_eq!(explanation["result"]["file"]["rule"], "web-tooling");
}

#[test]
fn invalid_folder_workspaces_fail_before_measuring() {
    let temp = monorepo();
    write_all(
        &temp.path().join("mono"),
        &[
            (
                "both.toml",
                "version = 1\nroots = [\"web\"]\nproject = \".\"\nfolders = [\"web\"]\n",
            ),
            ("bare.toml", "version = 1\nproject = \".\"\n"),
            ("orphan.toml", "version = 1\nfolders = [\"web\"]\n"),
            (
                "whole.toml",
                "version = 1\nproject = \".\"\nfolders = [\".\"]\n",
            ),
            (
                "nested.toml",
                "version = 1\nproject = \".\"\nfolders = [\"web\", \"web/src\"]\n",
            ),
            (
                "twins.toml",
                "version = 1\nproject = \".\"\nfolders = [\"web/src\", \"api/src\"]\n",
            ),
            (
                "absent.toml",
                "version = 1\nproject = \".\"\nfolders = [\"mobile\"]\n",
            ),
            ("empty.toml", "version = 1\nproject = \".\"\nfolders = []\n"),
        ],
    );
    for (file, message) in [
        ("both.toml", "roots"),
        ("bare.toml", "folders"),
        ("orphan.toml", "project"),
        ("whole.toml", "project root"),
        ("nested.toml", "inside"),
        ("twins.toml", "share the name 'src'"),
        ("absent.toml", "mobile"),
        ("empty.toml", "folders"),
    ] {
        let stderr = stderr_of(&locmetry(
            temp.path(),
            &["workspace", "--workspace-file", &format!("mono/{file}")],
        ));
        assert!(stderr.contains(message), "{file}: {stderr}");
    }
}
