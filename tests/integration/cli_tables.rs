use crate::cli::*;
use crate::common;
use common::locmetry;
use serde_json::json;
use std::fs;

#[cfg(unix)]
/// The complete report guards section separation while preserving source
/// totals, purpose splits, verbose notes and root context.
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
        &["--verbose", "--show-folders", "--format", "table"],
    );
    assert!(output.status.success());
    let expected = " ────────────────────────────────────────────────────────────────────────────────
 Language  Files  Lines  Code │ Comments  Blanks │ Prod.  Test  Other │ Test/prod
 ─────────────────────────────┼──────────────────┼────────────────────┼──────────
 Rust          3      5     4 │        1       0 │     2     2      0 │      1.00
 Python        1      1     1 │        0       0 │     0     0      1 │         -
 ─────────────────────────────┼──────────────────┼────────────────────┼──────────
 Total         4      6     5 │        1       0 │     2     2      1 │      1.00
 Lines = Code + Comments + Blanks; Production + Test + Other = Code; Test/prod = Test / Production (Code).
 Tests written in another language than the code they test skew per-language Test/prod.

 ────────────────────────────────────────────────────
 Folder  Lines  Code │ Prod.  Test  Other │ Test/prod
 ────────────────────┼────────────────────┼──────────
 src         3     2 │     2     0      0 │      0.00
 tests       2     2 │     0     2      0 │         -
 proto       1     1 │     0     0      1 │         -
 ────────────────────┼────────────────────┼──────────
 Total       6     5 │     2     2      1 │      1.00
 Production + Test + Other = Code; Test/prod = Test / Production (Code).

 Not source code: 1 data file (1 line), 1 documentation file (1 line); --files --output - lists each file's kind.
 Unclaimed source: 1 code line in 1 file, here:
   proto  1 code line  1 file
 To inspect or refine purpose classification, use locmetry --guide configure; --files --output - shows each file's rule.
 Unrecognized files: 1 unrecognized file (png: 1); missing lines are not zero.
";
    let root = dunce::canonicalize(temp.path().join("project")).unwrap();
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
        "Production + Test + Other = Code; Test/prod = Test / Production (Code).",
        "| Folder | Lines | **Code** | Production | Test | Other | Test/prod |\n",
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
        .args(["--format", "table", "--show-folders", "--show-bars"])
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
        .args(["--format", "json", "--show-bars"])
        .output()
        .unwrap();
    assert!(String::from_utf8(json.stderr)
        .unwrap()
        .contains("--show-bars has no effect here; it applies only to table output"));
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
            .args(["--format", "table", "--show-folders", "--show-bars"])
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
    assert!(wide.contains("Bars: █ Production  ▓ Test  ░ Other; the largest row fills the bar."));

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

/// Category rows take bars like language and folder rows: each row is one
/// purpose, so its bar has that purpose's glyph, and the share is of total Code.
#[test]
fn category_rows_carry_share_bars_in_their_purpose() {
    let temp = project(&[
        (".locmetryrc", APP_RULE),
        ("src/lib.rs", "fn a() {}\nfn b() {}\nfn c() {}\n"),
        ("tests/it.rs", "fn t() {}\n"),
    ]);
    let root = temp.path().join("project");
    let table = |columns: &str| {
        let output = locmetry()
            .current_dir(&root)
            .env("COLUMNS", columns)
            .args(["--format", "table", "--show-categories", "--show-bars"])
            .output()
            .unwrap();
        assert!(output.status.success());
        String::from_utf8(output.stdout).unwrap()
    };
    let wide = table("300");
    let header = wide
        .lines()
        .find(|line| line.starts_with(" Category"))
        .unwrap();
    assert!(header.ends_with("│ Share  Code share"), "{header}");
    let production = wide
        .lines()
        .find(|line| line.starts_with(" production:"))
        .unwrap();
    assert!(
        production.ends_with(&format!("│ 75.0%  {}", "█".repeat(40))),
        "{production}"
    );
    let test = wide
        .lines()
        .find(|line| line.starts_with(" test:"))
        .unwrap();
    assert!(
        test.ends_with(&format!("│ 25.0%  {}", "▓".repeat(13))),
        "{test}"
    );
    let legend = wide
        .lines()
        .skip_while(|line| !line.starts_with(" Category"))
        .find(|line| line.contains("Bars: "))
        .unwrap();
    assert!(
        legend.contains("Share = Code / total Code. Bars: █ Production  ▓ Test  ░ Other"),
        "{legend}"
    );

    let narrow = table("40");
    assert!(
        narrow.contains("\n test:test              25.0%  ▓▓▓\n"),
        "{narrow}"
    );
}

/// Human reports start with their tables and end with the measured context:
/// the root, any subfolder, then the measurement time as the final line.
#[test]
fn human_reports_end_with_the_measured_root_before_the_time() {
    let temp = project(&[("src/lib.rs", "fn a() {}\n")]);
    let root = temp.path().join("project");
    let canonical = dunce::canonicalize(&root).unwrap();
    let stdout = |args: &[&str]| String::from_utf8(run(&root, args).stdout).unwrap();
    let table = stdout(&["--format", "table"]);
    assert!(table.starts_with(" ─"), "{table}");
    assert!(
        table.lines().nth(1).unwrap().starts_with(" Language "),
        "{table}"
    );
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
    assert!(subfolder.starts_with(" ─"), "{subfolder}");
    assert!(
        subfolder.lines().nth(1).unwrap().starts_with(" Language "),
        "{subfolder}"
    );
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
    let both = run(&root, &["--format", "table", "--show-bars", "--no-bars"]);
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
            "--show-folders",
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
    assert!(
        printed.starts_with(" ─") && printed.lines().nth(1).unwrap().starts_with(" Language "),
        "{printed}"
    );
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
    let table = stdout(&["--show-folders", "--format", "table"]);
    assert!(is_ascending(&positions(
        &table,
        &["\n Rust", "\n Python", "\n z ", "\n a "]
    )));
    let table = stdout(&["--show-folders", "--format", "table", "--sort", "name"]);
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
        String::from_utf8(run(&root, &["--verbose", "--show-folders", "--format", "table"]).stdout)
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
