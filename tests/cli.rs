mod common;
use common::locmetry;
use serde_json::{json, Value};
use std::{fs, path::Path};

fn fixture() -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir(temp.path().join("source")).unwrap();
    fs::write(temp.path().join("source/main.rs"), "fn main() {}\n").unwrap();
    fs::write(temp.path().join("rules.toml"), APP_RS_RULE).unwrap();
    temp
}

const APP_RS_RULE: &str =
    "version = 1\n[[rules]]\nid = \"app\"\nmatch = [\"**/*.rs\"]\npurpose = \"production\"\n";

/// Scan root and configuration of [`fixture`]; the configuration lies outside the root.
fn config_args(temp: &tempfile::TempDir) -> [std::path::PathBuf; 4] {
    [
        "--root".into(),
        temp.path().join("source"),
        "--config".into(),
        temp.path().join("rules.toml"),
    ]
}

fn run(cwd: &Path, args: &[&str]) -> std::process::Output {
    locmetry().current_dir(cwd).args(args).output().unwrap()
}

fn project(files: &[(&str, &str)]) -> tempfile::TempDir {
    let temp = tempfile::tempdir().unwrap();
    fs::create_dir_all(temp.path().join("project")).unwrap();
    common::write_all(&temp.path().join("project"), files);
    temp
}

/// Stdout headline of a run.
fn headline(cwd: &Path, args: &[&str]) -> (bool, Value, String) {
    let output = run(cwd, args);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let document = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    (output.status.success(), document, stderr)
}

/// Full report with per-file records, written to stdout through `--output -`.
fn report(cwd: &Path, args: &[&str]) -> (bool, Value, String) {
    let args = [args, &["--output", "-", "--files"]].concat();
    headline(cwd, &args)
}

fn rules_by_path(document: &Value) -> Vec<(String, String)> {
    document["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            (
                file["path"].as_str().unwrap().into(),
                file["rule"].as_str().unwrap().into(),
            )
        })
        .collect()
}

const APP_RULE: &str =
    "version = 1\n[[rules]]\nid = \"app\"\nmatch = [\"src/**\"]\npurpose = \"production\"\n";

#[test]
fn binary_measures_without_any_executables_on_path() {
    let temp = fixture();
    let output = locmetry()
        .env("PATH", "")
        .args(config_args(&temp))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let document: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        document["summary"]["source_purpose"]["production"]["code"],
        1
    );
    assert_eq!(document["metadata"]["backend"], locmetry::BACKEND);
    assert_eq!(document["metadata"]["backend_inaccurate"], false);
}

#[test]
fn report_file_is_written_verbatim_outside_the_scan_root() {
    let temp = fixture();
    let path = temp.path().join("out/run.v1");
    let output = locmetry()
        .args(config_args(&temp))
        .arg("--output")
        .arg(&path)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let report: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(report["summary"]["source"]["code"], 1);
    let written: Vec<_> = fs::read_dir(temp.path().join("out"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name())
        .collect();
    assert_eq!(written, ["run.v1"], "a run writes exactly one file");
}

#[test]
fn output_in_scan_root_is_rejected_before_writing() {
    let temp = fixture();
    let output = locmetry()
        .args(config_args(&temp))
        .arg("--output")
        .arg(temp.path().join("source/reports/report.json"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(output.stdout.is_empty());
    assert!(!temp.path().join("source/reports").exists());
}

#[test]
fn output_that_would_overwrite_the_configuration_is_rejected() {
    let temp = fixture();
    let output = locmetry()
        .args(config_args(&temp))
        .arg("--output")
        .arg(temp.path().join("rules.toml"))
        .output()
        .unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("configuration"));
    assert_eq!(
        fs::read_to_string(temp.path().join("rules.toml")).unwrap(),
        APP_RS_RULE
    );
}

#[test]
fn missing_root_and_invalid_config_fail_without_success_json() {
    let temp = fixture();
    for args in [
        vec!["--config", "absent.toml"],
        vec!["--config", "rules.toml", "--root", "absent"],
    ] {
        let result = run(temp.path(), &args);
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
    }
}

/// Exit code 2 is reserved for partial reports, so usage errors must not use
/// clap's default of 2.
#[test]
fn usage_errors_exit_1_while_help_and_version_succeed() {
    let temp = fixture();
    for args in [vec!["--bogus"], vec!["--hidden", "maybe"]] {
        let result = run(temp.path(), &args);
        assert_eq!(result.status.code(), Some(1));
        assert!(result.stdout.is_empty());
        assert!(!result.stderr.is_empty());
    }
    for args in [vec!["--help"], vec!["--version"]] {
        let result = run(temp.path(), &args);
        assert_eq!(result.status.code(), Some(0));
        assert!(!result.stdout.is_empty());
    }
}

#[test]
fn markdown_format_writes_a_readable_report_to_stdout() {
    let temp = fixture();
    let output = locmetry()
        .args(["--format", "markdown"])
        .args(config_args(&temp))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.starts_with(
        "# source: lines of code by purpose\n\n*Production means maintained non-test source, including separately categorized tooling. Data, configuration, and documentation are counted apart from source. Lines = Code + Comments + Blanks in every table with these columns, and the purpose rows add up to all source.*\n\n## Source purpose"
    ));
    assert!(stdout.contains("| production | 1 | 1 | 1 | 0 | 0 |"));
    // Measurement caveats belong with coverage, not in the opening.
    let coverage = &stdout[stdout.find("## Coverage").unwrap()..];
    assert!(coverage
        .contains("Counts describe the working copy, including untracked files, not a commit."));
    assert!(!stdout.contains("Tokei"));
}

#[cfg(unix)]
/// Locks the terminal layout: totals in the familiar Code, Comments, Blanks
/// order, then the code split by purpose under its caption; rows sorted by code,
/// then one line each for non-source content, review, and coverage gaps.
#[test]
fn table_format_splits_source_code_by_language_and_purpose() {
    let temp = project(&[
        (".locmetryrc", APP_RULE),
        ("src/lib.rs", "fn a() {}\n"),
        ("src/main.rs", "// entry\nfn main() {}\n"),
        ("tests/it.rs", "#[test]\nfn t() {}\n"),
        ("proto/x.py", "x = 1\n"),
        ("config.json", "{}\n"),
        ("README.md", "# Title\n"),
        ("logo.png", "not an image\n"),
    ]);
    let output = run(
        &temp.path().join("project"),
        &["--verbose", "--folders", "--format", "table"],
    );
    assert!(output.status.success());
    let expected = " Language  Files  Lines  Code │ Comments  Blanks │ Production  Test  Unclassified │ Test/prod
 ─────────────────────────────┼──────────────────┼────────────────────────────────┼──────────
 Rust          3      5     4 │        1       0 │          2     2             0 │      1.00
 Python        1      1     1 │        0       0 │          0     0             1 │         -
 ─────────────────────────────┼──────────────────┼────────────────────────────────┼──────────
 Total         4      6     5 │        1       0 │          2     2             1 │      1.00
 Lines = Code + Comments + Blanks; Production + Test + Unclassified = Code; Test/prod = Test / Production.
 Tests written in another language than the code they test skew per-language Test/prod.

 Folder  Lines  Code │ Production  Test  Unclassified │ Test/prod
 ────────────────────┼────────────────────────────────┼──────────
 src         3     2 │          2     0             0 │      0.00
 tests       2     2 │          0     2             0 │         -
 proto       1     1 │          0     0             1 │         -
 ────────────────────┼────────────────────────────────┼──────────
 Total       6     5 │          2     2             1 │      1.00
 Production + Test + Unclassified = Code; Test/prod = Test / Production.

 Not source code: 1 data file (1 line), 1 documentation file (1 line); --files --output - lists each file's kind.
 Unclaimed source: 1 code line in 1 file, here:
   proto  1 code line  1 file
 To inspect or refine purpose classification, use locmetry --guide configure; --files --output - shows each file's rule.
 Unrecognized files: 1 unrecognized file (png: 1); missing lines are not zero.
";
    let root = temp.path().join("project").canonicalize().unwrap();
    let expected = format!("{expected}\nRoot: {}", root.display());
    assert_eq!(
        String::from_utf8(output.stdout)
            .unwrap()
            .split("\nMeasured at:")
            .next()
            .unwrap(),
        expected
    );
    let markdown =
        String::from_utf8(run(&temp.path().join("project"), &["--format", "markdown"]).stdout)
            .unwrap();
    for row in [
        "Lines = Code + Comments + Blanks in every table with these columns",
        "Production + Test + Unclassified = Code; Test/prod = Test / Production.",
        "| Folder | Lines | **Code** | Production | Test | Unclassified | Test/prod |\n",
        "| Purpose | Files | Lines | **Code** | Comments | Blanks |\n",
        "| . | 6 | 5 | 2 | 2 | 1 | 1.00 |\n",
        "| src | 3 | 2 | 2 | 0 | 0 | 0.00 |\n",
        "| tests | 2 | 2 | 0 | 2 | 0 | - |\n",
        "| Language | Files | Lines | **Code** | Comments | Blanks | Test/prod |\n",
        "| Rust | 3 | 5 | 4 | 1 | 0 | 1.00 |\n",
        "| Python | 1 | 1 | 1 | 0 | 0 | - |\n",
        "*Tests written in another language than the code they test skew per-language Test/prod.*",
    ] {
        assert!(markdown.contains(row), "{row:?} missing in:\n{markdown}");
    }
}

/// A total ratio divides summed Test by summed Production; averaging the row
/// ratios (3.00 and 0.11) would give 1.56 instead of 0.40.
#[test]
fn language_test_ratio_totals_divide_summed_counts() {
    let production = "x = 1\n".repeat(9);
    let temp = project(&[
        (".locmetryrc", APP_RULE),
        ("src/lib.rs", "fn a() {}\n"),
        ("tests/it.rs", "fn t() {}\nfn u() {}\nfn v() {}\n"),
        ("src/app.py", &production),
        ("tests/test_app.py", "y = 1\n"),
    ]);
    let root = temp.path().join("project");
    let table = String::from_utf8(run(&root, &["--format", "table"]).stdout).unwrap();
    let ratio = |label: &str| {
        let line = table
            .lines()
            .find(|line| line.starts_with(&format!(" {label} ")))
            .unwrap();
        line.rsplit(' ').next().unwrap().to_string()
    };
    assert_eq!(ratio("Rust"), "3.00", "{table}");
    assert_eq!(ratio("Python"), "0.11", "{table}");
    assert_eq!(ratio("Total"), "0.40", "{table}");
    let markdown = String::from_utf8(run(&root, &["--format", "markdown"]).stdout).unwrap();
    assert!(
        markdown.contains("| Rust | 2 | 4 | 4 | 0 | 0 | 3.00 |\n"),
        "{markdown}"
    );
    assert!(
        markdown.contains("| Python | 2 | 10 | 10 | 0 | 0 | 0.11 |\n"),
        "{markdown}"
    );
}

/// Share bars follow each table and fit the terminal width they are given,
/// however wide the table above them is.
#[test]
fn bars_follow_language_and_folder_tables_within_the_terminal_width() {
    let temp = project(&[
        (".locmetryrc", APP_RULE),
        ("src/lib.rs", "fn a() {}\nfn b() {}\nfn c() {}\n"),
        ("tests/it.rs", "fn t() {}\n"),
    ]);
    let root = temp.path().join("project");
    let output = locmetry()
        .current_dir(&root)
        .env("COLUMNS", "40")
        .args(["--format", "table", "--folders", "--bars"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let text = String::from_utf8(output.stdout).unwrap();
    let charts: Vec<_> = text.match_indices("\n Code share ").collect();
    assert_eq!(charts.len(), 2, "{text}");
    assert!(
        text.contains("\n Rust  100.0%  ███████████████████▓▓▓▓▓▓\n"),
        "{text}"
    );
    assert!(
        text.contains("\n src    75.0%  █████████████████████████\n"),
        "{text}"
    );
    assert!(text.contains("\n tests  25.0%  ▓▓▓▓▓▓▓▓\n"), "{text}");
    let json = locmetry()
        .current_dir(&root)
        .args(["--format", "json", "--bars"])
        .output()
        .unwrap();
    assert!(String::from_utf8(json.stderr)
        .unwrap()
        .contains("--bars has no effect here; it applies only to table output"));
}

/// With room for the minimum inline bar beside a table, its rows carry their
/// share and bar; one column less moves that table's bars below it.
#[test]
fn bars_join_each_table_when_the_terminal_leaves_room_beside_it() {
    let temp = project(&[
        (".locmetryrc", APP_RULE),
        ("src/lib.rs", "fn a() {}\nfn b() {}\nfn c() {}\n"),
        ("tests/it.rs", "fn t() {}\n"),
    ]);
    let root = temp.path().join("project");
    let table = |columns: usize| {
        let output = locmetry()
            .current_dir(&root)
            .env("COLUMNS", columns.to_string())
            .args(["--format", "table", "--folders", "--bars"])
            .output()
            .unwrap();
        assert!(output.status.success());
        String::from_utf8(output.stdout).unwrap()
    };
    let wide = table(300);
    assert!(!wide.contains("\n Code share "), "no chart below: {wide}");
    let header = wide
        .lines()
        .find(|line| line.starts_with(" Language"))
        .unwrap();
    assert!(header.ends_with("│  Share  Code share"), "{header}");
    let rust = wide.lines().find(|line| line.starts_with(" Rust")).unwrap();
    assert!(
        rust.ends_with(&format!("│ 100.0%  {}{}", "█".repeat(30), "▓".repeat(10))),
        "{rust}"
    );
    let src = wide.lines().find(|line| line.starts_with(" src")).unwrap();
    assert!(
        src.ends_with(&format!("│  75.0%  {}", "█".repeat(40))),
        "{src}"
    );
    let total = wide
        .lines()
        .find(|line| line.starts_with(" Total"))
        .unwrap();
    assert!(total.ends_with("│ 100.0%"), "{total:?}");
    assert!(
        wide.contains("Bars: █ Production  ▓ Test  ░ Unclassified; the largest row fills the bar.")
    );

    // The language table is the wider one, so its threshold also fits the folders.
    // Its Total row has no bar, so its length is the table's width.
    let width = total.chars().count();
    let at = table(width + 2 + 10);
    assert!(!at.contains("\n Code share "), "{at}");
    let rust = at.lines().find(|line| line.starts_with(" Rust")).unwrap();
    assert_eq!(rust.chars().count(), width + 2 + 10, "{rust}");
    let below = table(width + 2 + 9);
    assert_eq!(below.matches("\n Code share ").count(), 1, "{below}");
    assert!(below
        .lines()
        .any(|line| line.starts_with(" src") && line.ends_with('█')));
}

/// Human reports start with their tables and end with the measured context:
/// the root, any subfolder, then the measurement time as the final line.
#[test]
fn human_reports_end_with_the_measured_root_before_the_time() {
    let temp = project(&[("src/lib.rs", "fn a() {}\n")]);
    let root = temp.path().join("project");
    let canonical = root.canonicalize().unwrap();
    let stdout = |args: &[&str]| String::from_utf8(run(&root, args).stdout).unwrap();
    let table = stdout(&["--format", "table"]);
    assert!(table.starts_with(" Language "), "{table}");
    let (context, time) = table.rsplit_once("\nMeasured at: ").unwrap();
    assert!(
        time.ends_with(" UTC\n") && !time.trim_end().contains('\n'),
        "{time}"
    );
    assert!(
        context.ends_with(&format!("\n\nRoot: {}", canonical.display())),
        "{table}"
    );
    let subfolder = stdout(&["--format", "table", "--subfolder", "src"]);
    assert!(subfolder.starts_with(" Language "), "{subfolder}");
    assert!(
        subfolder.contains(&format!(
            "\nRoot: {}, subfolder src\nMeasured at: ",
            canonical.display()
        )),
        "{subfolder}"
    );
    let markdown = stdout(&["--format", "markdown"]);
    assert!(
        markdown.contains(&format!(
            "\nRoot: `{}`\n\nMeasured at: ",
            canonical.display()
        )),
        "{markdown}"
    );
}

/// Automatic bars belong to a terminal: piped tables stay bar-free even with a
/// wide `COLUMNS`, and asking for and refusing bars at once is an error.
#[test]
fn bars_are_never_automatic_in_pipes_and_cannot_be_both_forced_and_refused() {
    let temp = project(&[("src/lib.rs", "fn a() {}\n")]);
    let root = temp.path().join("project");
    let output = locmetry()
        .current_dir(&root)
        .env("COLUMNS", "300")
        .args(["--format", "table"])
        .output()
        .unwrap();
    let text = String::from_utf8(output.stdout).unwrap();
    assert!(!text.contains("Share") && !text.contains('█'), "{text}");
    let both = run(&root, &["--format", "table", "--bars", "--no-bars"]);
    assert_eq!(both.status.code(), Some(1));
    let json = run(&root, &["--format", "json", "--no-bars"]);
    assert!(String::from_utf8(json.stderr)
        .unwrap()
        .contains("--no-bars has no effect here; it applies only to table output"));
}

/// One measurement serves an agent twice: `--print` sends a second view to
/// stdout while `--output` saves the report, and table options shape that view.
#[test]
fn print_shows_a_second_view_of_the_saved_measurement() {
    let temp = project(&[
        (".locmetryrc", APP_RULE),
        ("src/lib.rs", "fn a() {}\n"),
        ("tests/it.rs", "fn t() {}\n"),
    ]);
    let root = temp.path().join("project");
    let saved = temp.path().join("report.json");
    let saved_arg = saved.to_str().unwrap();
    let output = run(
        &root,
        &[
            "--output",
            saved_arg,
            "--print",
            "table",
            "--folders",
            "--verbose",
        ],
    );
    assert!(output.status.success());
    assert!(
        output.stderr.is_empty(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let printed = String::from_utf8(output.stdout).unwrap();
    assert!(printed.starts_with(" Language "), "{printed}");
    assert!(printed.contains("\n Folder "), "{printed}");
    assert!(printed.lines().last().unwrap().starts_with("Measured at: "));
    let report: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&saved).unwrap()).unwrap();
    assert!(report["breakdown"].is_array(), "the file keeps full JSON");

    let headline = run(&root, &["--output", saved_arg, "--print", "headline"]);
    let headline: serde_json::Value = serde_json::from_slice(&headline.stdout).unwrap();
    assert!(headline["summary"].is_object() && headline.get("breakdown").is_none());

    let alone = run(&root, &["--print", "table"]);
    assert_eq!(alone.status.code(), Some(1));
    assert!(alone.stdout.is_empty());
    assert!(String::from_utf8(alone.stderr)
        .unwrap()
        .contains("--print requires --output FILE"));
}

/// Row positions of `labels` in `text`, to compare orders without pinning layout.
fn positions(text: &str, labels: &[&str]) -> Vec<usize> {
    labels
        .iter()
        .map(|label| text.find(label).unwrap())
        .collect()
}

fn is_ascending(positions: &[usize]) -> bool {
    positions.windows(2).all(|pair| pair[0] < pair[1])
}

/// Rows sort by code, largest first, unless `--sort name` asks for the stable
/// alphabetical order; both human formats follow the same option.
#[test]
fn human_tables_sort_by_code_or_by_name() {
    // Code order (Rust, then z) disagrees with name order (Python, then a).
    let temp = project(&[
        ("z/big.rs", "fn a() {}\nfn b() {}\nfn c() {}\n"),
        ("a/small.py", "a = 1\n"),
    ]);
    let root = temp.path().join("project");
    let stdout = |args: &[&str]| String::from_utf8(run(&root, args).stdout).unwrap();
    let table = stdout(&["--folders", "--format", "table"]);
    assert!(is_ascending(&positions(
        &table,
        &["\n Rust", "\n Python", "\n z ", "\n a "]
    )));
    let table = stdout(&["--folders", "--format", "table", "--sort", "name"]);
    assert!(is_ascending(&positions(
        &table,
        &["\n Python", "\n Rust", "\n a ", "\n z "]
    )));
    let markdown = stdout(&["--format", "markdown"]);
    let languages = &markdown[markdown.find("## Source languages").unwrap()..];
    assert!(is_ascending(&positions(
        languages,
        &["| Rust |", "| Python |"]
    )));
    let markdown = stdout(&["--format", "markdown", "--sort", "name"]);
    let languages = &markdown[markdown.find("## Source languages").unwrap()..];
    assert!(is_ascending(&positions(
        languages,
        &["| Python |", "| Rust |"]
    )));
}

/// Unclassified source is located, not just counted: folders partition it,
/// single-child chains collapse to the deepest folder, and the largest
/// folder splits while the list stays short.
#[test]
fn unclassified_source_is_located_by_folder() {
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
        document["summary"]["unclassified_folders"],
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
/// deliberately leaves unclassified is counted as unclassified but not
/// reported as still needing a decision.
#[test]
fn review_lists_only_unclaimed_source() {
    const PARKED: &str = "version = 1\n[[rules]]\nid = \"parked\"\nmatch = [\"proto/**\"]\npurpose = \"unclassified\"\ncategory = \"prototype\"\n";
    let temp = project(&[
        (".locmetryrc", PARKED),
        ("proto/model.py", "a = 1\nb = 2\n"),
        ("gradlew", "#!/bin/sh\necho build\n"),
        ("loose/y.py", "y = 1\n"),
    ]);
    let root = temp.path().join("project");
    let (success, document, stderr) = report(&root, &[]);
    assert!(success, "{stderr}");
    assert_eq!(
        document["summary"]["source_purpose"]["unclassified"]["files"],
        3
    );
    assert_eq!(
        document["summary"]["unclassified_folders"],
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
    assert_eq!(document["summary"]["unclassified_folders"], json!([]));
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
        table.contains(" Git submodules: vendor/lib (2 code lines); exclude them with --exclude or scope.exclude if they are not this project's code.\n"),
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
        table.contains(" Exclusions that excluded nothing: third_party, **/*.gen.ts, build/**; exclude a folder as <folder>/**, and an ignore file may already hide the path.\n"),
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
        String::from_utf8(run(&root, &["--format", "table", "--categories"]).stdout).unwrap();
    let expected = "
 Category                   Files  Lines  Code
 ─────────────────────────────────────────────
 unclassified:unclassified      1      2     2
 production:tooling             1      1     1
 test:test                      1      1     1
 Rows of one purpose add up to that purpose's Code; all rows add up to the total Code.
";
    assert!(table.contains(expected), "{table}");
}

/// Readers see which build systems decided production and how much code each
/// one claimed; a module whose layout claimed nothing hints at a misleading
/// build file.
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
            "maven": {"modules": 1, "code": 0},
            "npm": {"modules": 2, "code": 1}
        })
    );
    let table = String::from_utf8(run(&root, &["--verbose", "--format", "table"]).stdout).unwrap();
    assert!(
        table.contains(
            " Production from build modules: gradle 2 code lines in 1 module, npm 1 code line in 2 modules.\n"
        ),
        "{table}"
    );
    let markdown = String::from_utf8(run(&root, &["--format", "markdown"]).stdout).unwrap();
    let section = &markdown[markdown.find("## Build modules").expect(&markdown)..];
    for row in [
        "| gradle | 1 | 2 |",
        "| npm | 2 | 1 |",
        "| backend | gradle | 2 |",
        "| tools | npm | 0 |",
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

/// A folder holding most of the unclassified code splits even when many
/// top-level folders already fill the list, so the bulk is located precisely.
#[test]
fn dominant_unclassified_folders_split_beyond_the_row_limit() {
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
    let folders = document["summary"]["unclassified_folders"]
        .as_array()
        .unwrap();
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

#[test]
fn explicit_table_format_overrides_the_stdout_export_default() {
    let temp = fixture();
    let result = run(
        temp.path(),
        &["--root", "source", "--format", "table", "--output", "-"],
    );
    assert_eq!(result.status.code(), Some(0));
    assert!(String::from_utf8_lossy(&result.stdout).contains("Language"));
}

/// Labelled counts read correctly for any number, including one.
#[test]
fn markdown_coverage_reads_correctly_for_single_files() {
    let temp = project(&[("diagram.png", "not an image\n")]);
    let output = run(&temp.path().join("project"), &["--format", "markdown"]);
    assert!(output.status.success());
    let markdown = String::from_utf8(output.stdout).unwrap();
    assert!(
        markdown.contains("Files selected: 1; counted: 0, including 0 through language mappings; not recognized: 1; recognized but unreadable: 0. Their lines are missing, not zero."),
        "{markdown}"
    );
}

#[test]
fn unreadable_files_produce_partial_output_naming_them_and_fail() {
    use std::os::unix::fs::PermissionsExt;
    let temp = fixture();
    let locked = temp.path().join("source/locked.py");
    fs::write(&locked, "x = 1\n").unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o000)).unwrap();
    if fs::read(&locked).is_ok() {
        return; // Permission bits do not restrict this user (for example root).
    }
    let output = locmetry().args(config_args(&temp)).output().unwrap();
    fs::set_permissions(&locked, fs::Permissions::from_mode(0o644)).unwrap();
    assert_eq!(
        output.status.code(),
        Some(2),
        "partial results use exit code 2"
    );
    assert!(String::from_utf8_lossy(&output.stderr).contains("locked.py"));
    let document: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(document["metadata"]["backend_inaccurate"], true);
    assert_eq!(document["metadata"]["coverage"]["unread"][0], "locked.py");
}

#[test]
fn configuration_is_discovered_in_the_scan_root_from_another_directory() {
    let temp = project(&[(".locmetryrc", APP_RULE), ("src/a.ts", "let a = 1;\n")]);
    let (success, document, stderr) = report(temp.path(), &["--root", "project"]);
    assert!(success, "{stderr}");
    assert_eq!(
        rules_by_path(&document),
        [("src/a.ts".into(), "app".into())]
    );
    assert_eq!(document["project"], "project");
    assert!(document["metadata"]["config_path"]
        .as_str()
        .unwrap()
        .ends_with(".locmetryrc"));
}

/// Runs with user defaults read from `home`, the `XDG_CONFIG_HOME` directory.
fn report_with_user_defaults(cwd: &Path, home: &Path, args: &[&str]) -> (bool, Value, String) {
    let args = [args, &["--output", "-", "--files"]].concat();
    let output = locmetry()
        .current_dir(cwd)
        .env("XDG_CONFIG_HOME", home)
        .args(&args)
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let document = serde_json::from_slice(&output.stdout).unwrap_or(Value::Null);
    (output.status.success(), document, stderr)
}

fn user_defaults(temp: &tempfile::TempDir, content: &str) -> std::path::PathBuf {
    let home = temp.path().join("home");
    fs::create_dir_all(home.join("locmetry")).unwrap();
    fs::write(home.join("locmetry/config.toml"), content).unwrap();
    home
}

const USER_DEFAULTS: &str = "version = 1
[scope]
exclude = [\"**/*.generated.ts\"]
[[languages.mappings]]
id = \"tasks\"
match = [\"**/*.runbook\"]
language = \"BASH\"
";

fn languages_by_path(document: &Value) -> Vec<(String, String)> {
    document["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            (
                file["path"].as_str().unwrap().into(),
                file["language"].as_str().unwrap().into(),
            )
        })
        .collect()
}

/// User exclusions sit between the defaults and the project's, and user
/// mappings count files the backend does not recognize.
#[test]
fn user_defaults_add_exclusions_and_language_mappings() {
    let config = format!("{APP_RULE}[scope]\nexclude = [\"vendor/**\"]\n");
    let temp = project(&[
        (".locmetryrc", config.as_str()),
        ("src/a.ts", "let a = 1;\n"),
        ("src/b.generated.ts", "let b = 1;\n"),
        ("vendor/c.ts", "let c = 1;\n"),
        ("scripts/build.runbook", "echo hi\n"),
    ]);
    let home = user_defaults(&temp, USER_DEFAULTS);
    let (success, document, stderr) =
        report_with_user_defaults(&temp.path().join("project"), &home, &[]);
    assert!(success, "{stderr}");
    assert_eq!(
        languages_by_path(&document),
        [
            ("scripts/build.runbook".into(), "BASH".into()),
            ("src/a.ts".into(), "TypeScript".into()),
        ]
    );
    let metadata = &document["metadata"];
    assert_eq!(
        metadata["scope"]["exclude"],
        json!([
            "**/.git/**",
            "**/node_modules/**",
            "**/.build/**",
            "**/.lake/**",
            "**/*.generated.ts",
            "vendor/**",
            ".locmetryrc"
        ])
    );
    assert!(metadata["user_config_path"]
        .as_str()
        .unwrap()
        .ends_with("locmetry/config.toml"));
}

/// The project knows its files best, so its mappings win over user ones.
#[test]
fn project_language_mappings_take_precedence_over_user_defaults() {
    let config = format!(
        "{APP_RULE}[[languages.mappings]]\nid = \"project-tasks\"\nmatch = [\"**/*.runbook\"]\nlanguage = \"Python\"\n"
    );
    let temp = project(&[
        (".locmetryrc", config.as_str()),
        ("build.runbook", "x = 1\n"),
    ]);
    let home = user_defaults(&temp, USER_DEFAULTS);
    let (success, document, stderr) =
        report_with_user_defaults(&temp.path().join("project"), &home, &[]);
    assert!(success, "{stderr}");
    assert_eq!(
        languages_by_path(&document),
        [("build.runbook".into(), "Python".into())]
    );
}

#[test]
fn no_user_config_ignores_user_defaults() {
    let temp = project(&[("src/b.generated.ts", "let b = 1;\n")]);
    let home = user_defaults(&temp, USER_DEFAULTS);
    let (success, document, stderr) =
        report_with_user_defaults(&temp.path().join("project"), &home, &["--no-user-config"]);
    assert!(success, "{stderr}");
    assert_eq!(document["files"][0]["path"], "src/b.generated.ts");
    assert_eq!(document["metadata"]["user_config_path"], Value::Null);
}

/// User defaults may only narrow scope and teach languages; anything that
/// would classify, widen scope, or collide with project IDs fails the run.
#[test]
fn user_defaults_reject_classification_scope_widening_and_id_clashes() {
    let clash = "version = 1\n[[languages.mappings]]\nid = \"tasks\"\nmatch = [\"*.x\"]\nlanguage = \"BASH\"\n";
    for (user, project_config) in [
        (APP_RULE.to_string(), APP_RULE),
        (
            "version = 1\n[scope]\nhidden = false\n".to_string(),
            APP_RULE,
        ),
        (
            "version = 1\n[classification]\nfallback = \"test\"\n".to_string(),
            APP_RULE,
        ),
        ("version = 2\n".to_string(), APP_RULE),
        (USER_DEFAULTS.to_string(), clash),
    ] {
        let temp = project(&[(".locmetryrc", project_config), ("a.ts", "let a = 1;\n")]);
        let home = user_defaults(&temp, &user);
        let (success, _, stderr) =
            report_with_user_defaults(&temp.path().join("project"), &home, &[]);
        assert!(!success, "accepted user defaults:\n{user}");
        assert!(stderr.contains("config.toml"), "{stderr}");
    }
}

fn classifications(document: &Value) -> Vec<(String, String, String)> {
    document["files"]
        .as_array()
        .unwrap()
        .iter()
        .map(|file| {
            (
                file["path"].as_str().unwrap().into(),
                file["purpose"].as_str().unwrap().into(),
                file["rule"].as_str().unwrap().into(),
            )
        })
        .collect()
}

fn triple(path: &str, purpose: &str, rule: &str) -> (String, String, String) {
    (path.into(), purpose.into(), rule.into())
}

/// Build descriptors declare modules, and each build system's source layout
/// inside a module is production; tests, build scripts, and anything outside
/// a declared source root keep their own classification.
#[test]
fn declared_modules_make_their_source_production() {
    let temp = project(&[
        ("packages/core/package.json", "{}\n"),
        ("packages/core/src/a.ts", "let a = 1;\n"),
        ("packages/core/src/a.test.ts", "let b = 1;\n"),
        ("packages/core/scripts/release.ts", "let c = 1;\n"),
        ("app/build.gradle.kts", "plugins {}\n"),
        ("app/src/main/kotlin/App.kt", "fun main() {}\n"),
        ("app/src/commonMain/kotlin/Shared.kt", "val x = 1\n"),
        ("app/src/test/kotlin/AppTest.kt", "fun t() {}\n"),
        ("legacy/pom.xml", "<project/>\n"),
        ("legacy/src/main/java/A.java", "class A {}\n"),
        ("crates/util/Cargo.toml", "[package]\n"),
        ("crates/util/src/lib.rs", "fn util() {}\n"),
        ("svc/go.mod", "module svc\n"),
        ("svc/main.go", "package main\n"),
        ("svc/internal/store.go", "package internal\n"),
        ("svc/main_test.go", "package main\n"),
        ("docs/probe.js", "let d = 1;\n"),
    ]);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    let source: Vec<_> = classifications(&document)
        .into_iter()
        .filter(|(path, _, _)| {
            !path.ends_with(".json")
                && !path.ends_with(".xml")
                && !path.ends_with(".mod")
                && !path.ends_with(".toml")
        })
        .collect();
    assert_eq!(
        source,
        [
            triple("app/build.gradle.kts", "production", "builtin:build-script"),
            triple(
                "app/src/commonMain/kotlin/Shared.kt",
                "production",
                "builtin:gradle-module"
            ),
            triple(
                "app/src/main/kotlin/App.kt",
                "production",
                "builtin:gradle-module"
            ),
            triple(
                "app/src/test/kotlin/AppTest.kt",
                "test",
                "builtin:test-directory"
            ),
            triple(
                "crates/util/src/lib.rs",
                "production",
                "builtin:cargo-module"
            ),
            triple("docs/probe.js", "unclassified", "fallback"),
            triple(
                "legacy/src/main/java/A.java",
                "production",
                "builtin:maven-module"
            ),
            triple(
                "packages/core/scripts/release.ts",
                "unclassified",
                "fallback"
            ),
            triple(
                "packages/core/src/a.test.ts",
                "test",
                "builtin:js-test-file"
            ),
            triple("packages/core/src/a.ts", "production", "builtin:npm-module"),
            triple("svc/internal/store.go", "production", "builtin:go-module"),
            triple("svc/main.go", "production", "builtin:go-module"),
            triple("svc/main_test.go", "test", "builtin:go-test-file"),
        ]
    );
    assert_eq!(
        document["metadata"]["modules"],
        json!([
            {"path": "app", "detector": "gradle"},
            {"path": "crates/util", "detector": "cargo"},
            {"path": "legacy", "detector": "maven"},
            {"path": "packages/core", "detector": "npm"},
            {"path": "svc", "detector": "go"}
        ])
    );
}

/// The innermost module decides: an outer module's layout never claims files
/// of a module nested inside it, while build systems sharing one folder
/// combine their layouts.
#[test]
fn the_innermost_module_decides_nested_layouts() {
    let temp = project(&[
        ("package.json", "{}\n"),
        ("src/index.ts", "let a = 1;\n"),
        ("src/plugins/fancy/package.json", "{}\n"),
        ("src/plugins/fancy/src/fancy.ts", "let b = 1;\n"),
        ("src/plugins/fancy/scripts/build.ts", "let c = 1;\n"),
        ("app/build.gradle.kts", "plugins {}\n"),
        ("app/package.json", "{}\n"),
        ("app/src/main/kotlin/App.kt", "fun main() {}\n"),
        ("app/src/web/ui.ts", "let d = 1;\n"),
    ]);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    let source: Vec<_> = classifications(&document)
        .into_iter()
        .filter(|(path, _, _)| path.ends_with(".ts") || path.ends_with(".kt"))
        .collect();
    assert_eq!(
        source,
        [
            triple(
                "app/src/main/kotlin/App.kt",
                "production",
                "builtin:gradle-module"
            ),
            triple("app/src/web/ui.ts", "production", "builtin:npm-module"),
            triple("src/index.ts", "production", "builtin:npm-module"),
            triple(
                "src/plugins/fancy/scripts/build.ts",
                "unclassified",
                "fallback"
            ),
            triple(
                "src/plugins/fancy/src/fancy.ts",
                "production",
                "builtin:npm-module"
            ),
        ]
    );
}

/// Locks the same-folder tie-break (docs/PRODUCT.md, Module Detection): when
/// layouts of build systems in one folder overlap, the detector whose ID sorts
/// first decides, so provenance never depends on file or detector order.
#[test]
fn overlapping_layouts_in_one_folder_resolve_in_detector_id_order() {
    let temp = project(&[
        ("package.json", "{}\n"),
        ("Cargo.toml", "[package]\n"),
        ("build.gradle.kts", "plugins {}\n"),
        ("src/lib.rs", "fn a() {}\n"),
        ("src/main/kotlin/App.kt", "fun main() {}\n"),
        ("src/web/ui.ts", "let d = 1;\n"),
    ]);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    let source: Vec<_> = classifications(&document)
        .into_iter()
        .filter(|(path, _, _)| path.starts_with("src/"))
        .collect();
    assert_eq!(
        source,
        [
            triple("src/lib.rs", "production", "builtin:cargo-module"),
            triple(
                "src/main/kotlin/App.kt",
                "production",
                "builtin:cargo-module"
            ),
            triple("src/web/ui.ts", "production", "builtin:cargo-module"),
        ]
    );
}

/// Locks determinism: module detection depends on the set of selected files,
/// never on the order a file system or caller lists them.
#[test]
fn module_detection_ignores_the_order_files_are_listed_in() {
    let files = [
        "web/package.json",
        "web/src/a.ts",
        "Cargo.toml",
        "app/build.gradle.kts",
        "app/package.json",
        "lib/pyproject.toml",
        "lib/shop/__init__.py",
    ];
    let mut reversed = files;
    reversed.reverse();
    let mut rotated = files;
    rotated.rotate_left(3);
    let detected = locmetry::detect_modules(&files);
    assert_eq!(locmetry::detect_modules(&reversed), detected);
    assert_eq!(locmetry::detect_modules(&rotated), detected);
}

/// Locks determinism end to end: the same files measured in trees created in
/// opposite orders give identical reports apart from the root path and the
/// measurement time, so directory enumeration order never leaks into results.
#[test]
fn identical_trees_give_identical_reports_regardless_of_creation_order() {
    let mut files = vec![
        ("package.json", "{}\n"),
        ("src/index.ts", "let a = 1;\n"),
        ("src/index.test.ts", "let t = 1;\n"),
        ("app/build.gradle.kts", "plugins {}\n"),
        ("app/package.json", "{}\n"),
        ("app/src/main/kotlin/App.kt", "fun main() {}\n"),
        ("scripts/z.ts", "let z = 1;\n"),
        ("scripts/a.ts", "let y = 1;\n"),
        ("docs/guide.md", "# Guide\n"),
        ("data/b.json", "{}\n"),
        ("notes.unknownext", "x\n"),
    ];
    let measure = |files: &[(&str, &str)]| {
        let temp = project(files);
        let (success, mut document, stderr) = report(&temp.path().join("project"), &[]);
        assert!(success, "{stderr}");
        let metadata = document["metadata"].as_object_mut().unwrap();
        metadata.remove("root");
        metadata.remove("measured_at_unix_seconds");
        serde_json::to_string(&document).unwrap()
    };
    let forward = measure(&files);
    files.reverse();
    assert_eq!(measure(&files), forward);
}

/// Python modules follow setuptools' discovery: the `src` layout, or flat
/// top-level packages with `__init__.py`, never names such as `scripts`.
#[test]
fn python_modules_follow_setuptools_package_discovery() {
    let temp = project(&[
        ("flat/pyproject.toml", "[project]\n"),
        ("flat/shop/__init__.py", "\n"),
        ("flat/shop/cart.py", "total = 0\n"),
        ("flat/scripts/__init__.py", "\n"),
        ("flat/scripts/seed.py", "seed = 1\n"),
        ("flat/tests/test_cart.py", "def test(): pass\n"),
        ("flat/notebook.py", "x = 1\n"),
        ("layered/setup.py", "setup()\n"),
        ("layered/src/pkg/core.py", "core = 1\n"),
    ]);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    let source: Vec<_> = classifications(&document)
        .into_iter()
        .filter(|(path, _, _)| {
            path.ends_with(".py") && !path.ends_with("__init__.py") && !path.ends_with("setup.py")
        })
        .collect();
    assert_eq!(
        source,
        [
            triple("flat/notebook.py", "unclassified", "fallback"),
            triple("flat/scripts/seed.py", "unclassified", "fallback"),
            triple("flat/shop/cart.py", "production", "builtin:python-module"),
            triple("flat/tests/test_cart.py", "test", "builtin:test-directory"),
            triple(
                "layered/src/pkg/core.py",
                "production",
                "builtin:python-module"
            ),
        ]
    );
}

/// Swift packages: `Sources/` is production, and test targets in `Tests/` or
/// Xcode-style `<Name>Tests/` folders are tests wherever they live.
#[test]
fn swift_packages_and_test_targets_are_recognized() {
    let temp = project(&[
        ("Package.swift", "// swift-tools-version:5.9\n"),
        ("Sources/App/main.swift", "print(1)\n"),
        ("Tests/AppTests/AppTests.swift", "import XCTest\n"),
        ("Plugins/Gen/plugin.swift", "let p = 1\n"),
        ("ios/MyAppTests/LoginTests.swift", "import XCTest\n"),
    ]);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    let source: Vec<_> = classifications(&document)
        .into_iter()
        .filter(|(path, _, _)| path != "Package.swift")
        .collect();
    assert_eq!(
        source,
        [
            triple("Plugins/Gen/plugin.swift", "unclassified", "fallback"),
            triple(
                "Sources/App/main.swift",
                "production",
                "builtin:swift-module"
            ),
            triple(
                "Tests/AppTests/AppTests.swift",
                "test",
                "builtin:swift-test-target"
            ),
            triple(
                "ios/MyAppTests/LoginTests.swift",
                "test",
                "builtin:swift-test-target"
            ),
        ]
    );
}

/// Lean 4 Lake packages keep their sources at the package root, so every
/// `.lean` file below a lakefile is production, except the lakefile itself,
/// test libraries named like mathlib4's `MathlibTest/`, and the `.lake/`
/// build and dependency folder.
#[test]
fn lake_packages_and_test_libraries_are_recognized() {
    let temp = project(&[
        (
            "lakefile.lean",
            "import Lake\nopen Lake DSL\npackage demo\n",
        ),
        ("Demo.lean", "import Demo.Core\n"),
        ("Demo/Core.lean", "def answer : Nat := 42\n"),
        ("DemoTest/Basic.lean", "example : 1 + 1 = 2 := rfl\n"),
        (".lake/packages/dep/lakefile.toml", "name = \"dep\"\n"),
        (".lake/packages/dep/Dep.lean", "def dep : Nat := 1\n"),
    ]);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    assert_eq!(
        classifications(&document),
        [
            triple("Demo.lean", "production", "builtin:lake-module"),
            triple("Demo/Core.lean", "production", "builtin:lake-module"),
            triple("DemoTest/Basic.lean", "test", "builtin:lean-test-library"),
            triple("lakefile.lean", "production", "builtin:build-script"),
        ]
    );
    assert_eq!(
        document["metadata"]["modules"],
        json!([{"path": ".", "detector": "lake"}])
    );
}

/// Project knowledge beats generic detection, and detection can be disabled.
#[test]
fn project_rules_precede_module_detection_which_can_be_disabled() {
    let legacy = "version = 1\n[[rules]]\nid = \"legacy\"\nmatch = [\"lib/src/legacy/**\"]\npurpose = \"unclassified\"\ncategory = \"legacy\"\n";
    let disabled = "version = 1\n[classification]\nbuiltin_modules = false\n";
    for (config, expected) in [
        (
            legacy,
            [
                triple("lib/src/legacy/old.ts", "unclassified", "legacy"),
                triple("lib/src/new.ts", "production", "builtin:npm-module"),
            ],
        ),
        (
            disabled,
            [
                triple("lib/src/legacy/old.ts", "unclassified", "fallback"),
                triple("lib/src/new.ts", "unclassified", "fallback"),
            ],
        ),
    ] {
        let temp = project(&[
            (".locmetryrc", config),
            ("lib/package.json", "{}\n"),
            ("lib/src/legacy/old.ts", "let a = 1;\n"),
            ("lib/src/new.ts", "let b = 1;\n"),
        ]);
        let (success, document, stderr) = report(&temp.path().join("project"), &[]);
        assert!(success, "{stderr}");
        let source: Vec<_> = classifications(&document)
            .into_iter()
            .filter(|(path, _, _)| path.ends_with(".ts"))
            .collect();
        assert_eq!(source, expected);
    }
}

/// The configuration is measurement policy, not project content, so the loaded
/// file must not appear as a selected file or a coverage gap.
#[test]
fn loaded_configuration_inside_the_scan_root_is_not_measured() {
    let temp = project(&[(".locmetryrc", APP_RULE), ("src/a.ts", "let a = 1;\n")]);
    let (success, document, stderr) = report(temp.path(), &["--root", "project"]);
    assert!(success, "{stderr}");
    let metadata = &document["metadata"];
    assert_eq!(metadata["coverage"]["selected_files"], 1);
    assert_eq!(metadata["coverage"]["unrecognized"], json!([]));
    assert!(metadata["scope"]["exclude"]
        .as_array()
        .unwrap()
        .contains(&json!(".locmetryrc")));
}

/// Glob metacharacters in the configuration's name match literally, so its
/// exclusion cannot hide a sibling that the unescaped pattern would match.
#[test]
fn explicit_configuration_inside_the_scan_root_is_excluded_literally() {
    let temp = project(&[
        ("cfg/lo[c].toml", APP_RULE),
        ("cfg/loc.toml", "answer = 42\n"),
        ("src/a.ts", "let a = 1;\n"),
    ]);
    let (success, document, stderr) = report(
        &temp.path().join("project"),
        &["--config", "cfg/lo[c].toml"],
    );
    assert!(success, "{stderr}");
    let paths: Vec<_> = rules_by_path(&document)
        .into_iter()
        .map(|(path, _)| path)
        .collect();
    assert_eq!(paths, ["cfg/loc.toml", "src/a.ts"]);
}

#[test]
fn explicit_configuration_does_not_change_the_scan_root() {
    let temp = project(&[("src/a.ts", "let a = 1;\n")]);
    fs::write(temp.path().join("shared.toml"), APP_RULE).unwrap();
    fs::write(temp.path().join("outside.ts"), "let b = 1;\n").unwrap();
    let (success, document, stderr) = report(
        &temp.path().join("project"),
        &["--config", "../shared.toml"],
    );
    assert!(success, "{stderr}");
    assert_eq!(
        rules_by_path(&document),
        [("src/a.ts".into(), "app".into())]
    );
}

#[test]
fn without_configuration_built_in_defaults_apply() {
    let temp = project(&[
        ("src/a.ts", "let a = 1;\n"),
        ("src/a.test.ts", "let b = 1;\n"),
        ("node_modules/x/index.js", "let c = 1;\n"),
    ]);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    assert_eq!(
        rules_by_path(&document),
        [
            ("src/a.test.ts".into(), "builtin:js-test-file".into()),
            ("src/a.ts".into(), "fallback".into()),
        ]
    );
    assert_eq!(document["metadata"]["config_path"], Value::Null);
}

#[test]
fn cli_scope_settings_override_the_project_configuration() {
    let config = format!("{APP_RULE}[scope]\nhidden = true\nexclude = [\"gen/**\"]\n");
    let temp = project(&[
        (".locmetryrc", config.as_str()),
        ("src/a.ts", "let a = 1;\n"),
        ("gen/a.ts", "let b = 1;\n"),
        ("vendor/a.ts", "let c = 1;\n"),
        (".tool/a.ts", "let d = 1;\n"),
    ]);
    let (success, document, stderr) = report(
        &temp.path().join("project"),
        &["--hidden", "false", "--exclude", "vendor/**"],
    );
    assert!(success, "{stderr}");
    assert_eq!(
        rules_by_path(&document),
        [("src/a.ts".into(), "app".into())]
    );
    assert_eq!(
        document["metadata"]["scope"]["exclude"],
        json!([
            "**/.git/**",
            "**/node_modules/**",
            "**/.build/**",
            "**/.lake/**",
            "gen/**",
            "vendor/**",
            ".locmetryrc"
        ])
    );
}

#[test]
fn invalid_discovered_configuration_fails_before_scanning() {
    let temp = project(&[
        (".locmetryrc", "version = 2\n"),
        ("src/a.ts", "let a = 1;\n"),
    ]);
    let (success, document, stderr) = headline(&temp.path().join("project"), &[]);
    assert!(!success);
    assert_eq!(document, Value::Null);
    assert!(
        stderr.contains(".locmetryrc") && stderr.contains("Unsupported configuration version 2"),
        "{stderr}"
    );
}

#[test]
fn stdout_headline_keeps_compact_evidence_without_a_report_pointer() {
    let temp = project(&[
        ("src/a.rs", "fn a() {}\n"),
        ("tools/gen.sh", "echo hi\n"),
        ("assets/logo.xyz", "x\n"),
        ("assets/icon.xyz", "x\n"),
    ]);
    let root = temp.path().join("project");
    let path = temp.path().join("report.json");
    let saved = run(&root, &["--output", path.to_str().unwrap()]);
    assert!(saved.status.success());
    assert!(saved.stdout.is_empty());
    let (success, headline, stderr) = headline(&root, &[]);
    assert!(success, "{stderr}");
    assert_eq!(headline["schema_version"], "1");
    assert!(headline.get("report").is_none());
    for absent in ["breakdown", "review", "files"] {
        assert!(headline.get(absent).is_none(), "{absent}");
    }
    assert!(headline["metadata"].get("config").is_none());
    let coverage = &headline["metadata"]["coverage"];
    assert!(coverage.get("unrecognized").is_none());
    assert_eq!(coverage["unrecognized_files"], 2);
    assert_eq!(coverage["unrecognized_extensions"], json!({"xyz": 2}));
    let top = &headline["summary"]["top_folders"];
    assert_eq!(top["src"]["unclassified"], 1);
    assert_eq!(top["tools"]["unclassified"], 1);
    assert!(headline["metadata"].get("kinds").is_none());
    let full: Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    assert_eq!(full["summary"], headline["summary"]);
    assert_eq!(
        full["metadata"]["coverage"]["unrecognized"],
        json!(["assets/icon.xyz", "assets/logo.xyz"])
    );
}

#[test]
fn report_lists_one_breakdown_row_per_line_and_records_only_on_request() {
    let temp = project(&[
        ("src/a.rs", "fn a() {}\n"),
        (
            "web/index.html",
            "<p>x</p>\n<script>\nlet x = 1;\n</script>\n",
        ),
    ]);
    let root = temp.path().join("project");
    let output = run(&root, &["--output", "-"]);
    let text = String::from_utf8(output.stdout).unwrap();
    let report: Value = serde_json::from_str(&text).unwrap();
    assert!(report.get("files").is_none(), "records only with --files");
    let breakdown = report["breakdown"].as_array().unwrap();
    for row in breakdown {
        let line = serde_json::to_string(row).unwrap();
        assert!(
            text.lines()
                .any(|text_line| text_line.trim_end_matches(',') == line),
            "row not on its own line: {line}"
        );
    }
    let row = |language: &str| {
        breakdown
            .iter()
            .find(|row| row["folder"] == "web" && row["language"] == language)
            .unwrap()
    };
    assert_eq!(row("HTML")["files"], 1);
    assert_eq!(
        (
            row("JavaScript")["files"].clone(),
            row("JavaScript")["code"].clone()
        ),
        (json!(0), json!(1)),
        "embedded languages add lines, not files"
    );
    assert_eq!(
        report["review"],
        json!([
            {"path":"web/index.html","language":"HTML","category":"unclassified","rule":"fallback","code":4,"lines":4},
            {"path":"src/a.rs","language":"Rust","category":"unclassified","rule":"fallback","code":1,"lines":1}
        ])
    );
    let (_, detailed, _) = report_with_files(&root);
    let record = |path: &str| {
        detailed["files"]
            .as_array()
            .unwrap()
            .iter()
            .find(|file| file["path"] == path)
            .unwrap()
            .clone()
    };
    let rust = record("src/a.rs");
    assert!(rust.get("languages").is_none());
    assert!(rust["counts"].get("files").is_none());
    assert_eq!(
        record("web/index.html")["languages"]["JavaScript"]["code"],
        1
    );
}

fn report_with_files(root: &Path) -> (bool, Value, String) {
    report(root, &[])
}

#[test]
fn markdown_folder_depth_is_configurable_while_json_keeps_every_folder() {
    let temp = project(&[("a/b/c/deep.rs", "fn deep() {}\n")]);
    let root = temp.path().join("project");
    // Only the folder breakdown has a depth; the review lists folders at any depth.
    let folder_table = |extra: &[&str]| {
        let args = [&["--format", "markdown"], extra].concat();
        let markdown = String::from_utf8(run(&root, &args).stdout).unwrap();
        let start = markdown.find("## Folder breakdown").unwrap();
        let end = markdown.find("## Source requiring review").unwrap();
        markdown[start..end].to_string()
    };
    assert!(!folder_table(&[]).contains("| a/b/c |"));
    assert!(folder_table(&["--depth", "3"]).contains("| a/b/c |"));
    let (_, document, _) = report(&root, &[]);
    assert_eq!(document["breakdown"][0]["folder"], "a/b/c");
}

#[test]
fn empty_roots_report_zero_counts_successfully() {
    let temp = project(&[]);
    let (success, document, stderr) = report(&temp.path().join("project"), &[]);
    assert!(success, "{stderr}");
    assert_eq!(document["metadata"]["coverage"]["selected_files"], 0);
    assert_eq!(document["summary"]["all_counted"]["files"], 0);
    assert_eq!(document["breakdown"], json!([]));
    assert_eq!(document["files"], json!([]));
}

#[test]
fn markdown_and_json_report_the_same_totals() {
    let temp = project(&[
        ("src/a.ts", "let a = 1;\n// note\n\nlet b = 2;\n"),
        ("src/a.test.ts", "let c = 1;\n"),
        ("gradlew", "#!/bin/sh\necho hi\n"),
    ]);
    let root = temp.path().join("project");
    let (_, document, _) = headline(&root, &[]);
    let markdown = String::from_utf8(run(&root, &["--format", "markdown"]).stdout).unwrap();
    for purpose in ["production", "test", "unclassified"] {
        // Absent purposes are zero in JSON; Markdown lists every purpose.
        let counts = &document["summary"]["source_purpose"][purpose];
        let count = |metric: &str| counts[metric].as_u64().unwrap_or(0);
        let row = format!(
            "| {purpose} | {} | {} | {} | {} | {} |",
            count("files"),
            count("lines"),
            count("code"),
            count("comments"),
            count("blanks")
        );
        assert!(markdown.contains(&row), "missing {row}");
    }
}

#[test]
fn breakdown_rows_add_up_to_the_summary() {
    let temp = project(&[
        ("src/a.ts", "let a = 1;\n// note\n\nlet b = 2;\n"),
        ("src/deep/b.test.ts", "let c = 1;\n"),
        ("web/index.html", "<script>\nlet x = 1;\n</script>\n"),
        ("README.md", "# Title\n"),
        ("data/x.json", "{}\n"),
    ]);
    let (_, document, _) = report(&temp.path().join("project"), &[]);
    let rows = document["breakdown"].as_array().unwrap();
    for metric in ["files", "code", "comments", "blanks", "lines"] {
        let total: u64 = rows.iter().map(|row| row[metric].as_u64().unwrap()).sum();
        assert_eq!(
            total, document["summary"]["all_counted"][metric],
            "{metric}"
        );
        let source: u64 = rows
            .iter()
            .filter(|row| row["kind"] == "source")
            .map(|row| row[metric].as_u64().unwrap())
            .sum();
        assert_eq!(source, document["summary"]["source"][metric], "{metric}");
    }
    let src: u64 = rows
        .iter()
        .filter(|row| {
            let folder = row["folder"].as_str().unwrap();
            folder == "src" || folder.starts_with("src/")
        })
        .map(|row| row["code"].as_u64().unwrap())
        .sum();
    let top = &document["summary"]["top_folders"]["src"];
    let top_code: u64 = ["production", "test", "unclassified"]
        .iter()
        .map(|purpose| top[purpose].as_u64().unwrap_or(0))
        .sum();
    assert_eq!(src, top_code, "recursive folder totals are prefix sums");
}

#[test]
fn report_metadata_records_the_effective_policy() {
    let temp = project(&[(".locmetryrc", APP_RULE), ("src/a.ts", "let a = 1;\n")]);
    let (_, document, _) = report(&temp.path().join("project"), &["--hidden", "false"]);
    let metadata = &document["metadata"];
    assert_eq!(metadata["scope"]["hidden"], false);
    assert_eq!(metadata["config"]["rules"][0]["id"], "app");
    let conventions = metadata["config"]["builtin_conventions"]
        .as_array()
        .unwrap();
    assert!(conventions
        .iter()
        .any(|c| c["id"] == "builtin:build-wrapper"));
    assert!(!metadata["config"]["builtin_language_mappings"]
        .as_array()
        .unwrap()
        .is_empty());
    assert_eq!(metadata["backend"], locmetry::BACKEND);
}

/// People read grouped digits faster: tables, notes, and Markdown group
/// thousands, while JSON keeps plain integers for machines.
#[test]
fn human_formats_group_thousands_and_json_keeps_plain_integers() {
    let source = "let a = 1;\n".repeat(1234);
    let temp = project(&[
        ("package.json", "{}\n"),
        ("src/big.ts", source.as_str()),
        ("loose/x.ts", "let x = 1;\n"),
    ]);
    let root = temp.path().join("project");
    let table =
        String::from_utf8(run(&root, &["--verbose", "--folders", "--format", "table"]).stdout)
            .unwrap();
    for expected in [
        " TypeScript      2  1,235  1,235 │",
        " src     1,234  1,234 │",
        "npm 1,234 code lines in 1 module",
    ] {
        assert!(
            table.contains(expected),
            "{expected:?} missing in:\n{table}"
        );
    }
    let markdown = String::from_utf8(run(&root, &["--format", "markdown"]).stdout).unwrap();
    assert!(
        markdown.contains("| production | 1 | 1,234 | 1,234 | 0 | 0 |"),
        "{markdown}"
    );
    let (_, document, _) = headline(&root, &[]);
    assert_eq!(document["summary"]["source"]["code"], json!(1235));
}

#[test]
fn measurement_help_explains_json_headline_and_full_report() {
    for args in [vec!["--help"], vec!["workspace", "--help"]] {
        let output = locmetry().args(args).output().unwrap();
        assert!(output.status.success());
        let help = String::from_utf8(output.stdout).unwrap();
        let help = help.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(help.contains("a JSON headline in pipes"), "{help}");
        assert!(
            help.contains("--output FILE saves only to that file, defaulting to full JSON"),
            "{help}"
        );
    }
}

#[test]
fn positional_directory_matches_explicit_root_and_discovers_its_config() {
    let temp = fixture();
    fs::write(temp.path().join("source/.locmetryrc"), APP_RS_RULE).unwrap();
    let report = |args: &[&str]| {
        let output = run(
            temp.path(),
            &[&["--files", "--output", "-"][..], args].concat(),
        );
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let mut value: Value = serde_json::from_slice(&output.stdout).unwrap();
        value["metadata"]
            .as_object_mut()
            .unwrap()
            .remove("measured_at_unix_seconds");
        value
    };
    let expected = report(&["--root", "source"]);
    assert_eq!(expected["files"][0]["purpose"], "production");
    let absolute = temp.path().join("source");
    for args in [
        vec!["source"],
        vec!["source", "--format", "json"],
        vec!["--format", "json", "source"],
        vec!["--", "source"],
    ] {
        assert_eq!(report(&args), expected, "{args:?}");
    }
    assert_eq!(
        report(&[absolute.to_str().unwrap()]),
        report(&["--root", absolute.to_str().unwrap()])
    );
    fs::rename(temp.path().join("source"), temp.path().join("source tree")).unwrap();
    assert_eq!(report(&["source tree"]), report(&["--root", "source tree"]));
}

#[test]
fn positional_directory_disambiguates_command_names_and_rejects_multiple_roots() {
    let temp = fixture();
    for name in ["compare", "explain", "workspace", "help"] {
        fs::create_dir(temp.path().join(name)).unwrap();
        fs::write(temp.path().join(name).join("main.py"), "print(1)\n").unwrap();
        for args in [vec![format!("./{name}")], vec!["--".into(), name.into()]] {
            let args: Vec<_> = args.iter().map(String::as_str).collect();
            let (ok, report, err) = headline(temp.path(), &args);
            assert!(ok, "{err}");
            assert_eq!(report["summary"]["source"]["code"], 1);
        }
        let args = match name {
            "help" => vec!["help"],
            _ => vec![name, "--help"],
        };
        let output = run(temp.path(), &args);
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8_lossy(&output.stdout).contains("Usage:"));
    }
    for args in [
        vec!["source", "--root", "source"],
        vec!["--root", "source", "source"],
        vec!["source", "other"],
        vec!["source", "workspace", "--root", "other"],
    ] {
        let output = run(temp.path(), &args);
        assert_eq!(output.status.code(), Some(1), "{args:?}");
        assert!(output.stdout.is_empty());
    }
}

#[test]
fn root_options_do_not_turn_subcommand_names_into_directories() {
    let temp = fixture();
    for name in ["compare", "workspace", "explain"] {
        fs::create_dir(temp.path().join(name)).unwrap();
        fs::write(temp.path().join(name).join("main.py"), "print(1)\n").unwrap();
        let output = run(temp.path(), &["--files", name]);
        assert_eq!(output.status.code(), Some(1), "{name}");
        assert!(output.stdout.is_empty(), "{name}");
        assert!(String::from_utf8_lossy(&output.stderr).contains(name));
        let (ok, value, err) = headline(temp.path(), &["--root", name]);
        assert!(ok, "{err}");
        assert_eq!(value["summary"]["source"]["code"], 1);
    }
}
