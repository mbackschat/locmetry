use anyhow::{ensure, Result};
use locmetry::{
    build_report, render_table_with_options, style::Theme, Classification, Counts, Coverage,
    FileRecord, Findings, Kind, Purpose, RowOrder, TableOptions,
};
use serde::Deserialize;
use std::{collections::BTreeSet, fs, path::Path};

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Group {
    folder: String,
    language: String,
    purpose: Purpose,
    files: u64,
    code: u64,
    comments: u64,
    blanks: u64,
}

fn fixture() -> Vec<Group> {
    serde_json::from_str(include_str!("fixtures/readme/report.json")).unwrap()
}

fn render(groups: &[Group], folders: bool) -> String {
    let mut keys = BTreeSet::new();
    let mut records = Vec::new();
    for group in groups {
        assert!(group.files > 0);
        assert!(keys.insert((&group.folder, &group.language, group.purpose)));
        for index in 0..group.files {
            // The renderer takes per-file records. Distribute aggregate counts in memory;
            // the illustrative fixture does not claim file contents or create source files.
            let share = |total: u64| total / group.files + u64::from(index < total % group.files);
            let code = share(group.code);
            let comments = share(group.comments);
            let blanks = share(group.blanks);
            let counts = Counts {
                files: 1,
                code,
                comments,
                blanks,
                lines: code + comments + blanks,
            };
            let name = format!("{}-{}-{index}", group.language, group.purpose);
            records.push(FileRecord {
                path: if group.folder == "." {
                    name
                } else {
                    format!("{}/{name}", group.folder)
                },
                language: group.language.clone(),
                kind: Kind::Source,
                language_mapping: None,
                classification: Classification {
                    purpose: group.purpose,
                    category: "illustrative".into(),
                    rule: "illustrative".into(),
                },
                counts,
                languages: [(group.language.clone(), counts)].into(),
            });
        }
    }
    render_table_with_options(
        "acme-orders",
        &build_report(&records),
        &records,
        &Findings {
            subfolder: None,
            coverage: &Coverage::default(),
            modules: &[],
            submodules: &[],
            unused_entries: &[],
            unused_exclusions: &[],
            members: &[],
        },
        TableOptions {
            categories: false,
            folders: false,
            verbose: false,
            order: RowOrder::Code,
            theme: Theme::PLAIN,
            folder_rows: folders.then_some(30),
        },
    )
}

fn tables(groups: &[Group]) -> [(String, String); 2] {
    let language = render(groups, false);
    let combined = render(groups, true);
    let (_, folders) = combined.split_once("\n Folder ").unwrap();
    [
        ("readme-language".into(), language),
        ("readme-folders".into(), format!(" Folder {folders}")),
    ]
}

fn block<'a>(readme: &'a str, marker: &str) -> &'a str {
    readme
        .split_once(&format!("<!-- {marker} -->\n```text\n"))
        .unwrap()
        .1
        .split_once("```\n")
        .unwrap()
        .0
}

fn reconcile(table: &str, groups: &[Group], folders: bool) -> Result<()> {
    let columns = if folders { 6 } else { 9 };
    let mut coverage = vec![0; groups.len()];
    let mut total_seen = false;
    let mut row_count = 0;
    for line in table.lines() {
        let plain = line.replace(',', "").replace('│', " ");
        let cells: Vec<_> = plain.split_whitespace().collect();
        if cells.len() <= columns {
            continue;
        }
        let (label, values) = cells.split_at(cells.len() - columns);
        if values[0].parse::<u64>().is_err() {
            continue;
        }
        let label = label.join(" ");
        let is_total = label == "Total";
        ensure!(!total_seen, "row after Total");
        total_seen = is_total;
        let selected: Vec<_> = groups
            .iter()
            .enumerate()
            .filter(|(_, group)| {
                is_total
                    || if !folders {
                        group.language == label
                    } else if label == "(root files)" {
                        group.folder == "."
                    } else if let Some(path) = label.strip_suffix(" (direct files)") {
                        group.folder == path
                    } else {
                        group.folder == label || group.folder.starts_with(&format!("{label}/"))
                    }
            })
            .collect();
        ensure!(!selected.is_empty(), "unknown row {label}");
        let sum =
            |get: fn(&Group) -> u64| selected.iter().map(|(_, group)| get(group)).sum::<u64>();
        let files = sum(|g| g.files);
        let code = sum(|g| g.code);
        let comments = sum(|g| g.comments);
        let blanks = sum(|g| g.blanks);
        let purposes: Vec<u64> = Purpose::ALL
            .iter()
            .map(|purpose| {
                selected
                    .iter()
                    .filter(|(_, group)| group.purpose == *purpose)
                    .map(|(_, group)| group.code)
                    .sum()
            })
            .collect();
        let mut expected = if folders {
            vec![code + comments + blanks, code]
        } else {
            vec![files, code + comments + blanks, code, comments, blanks]
        };
        expected.extend(&purposes);
        for (actual, expected) in values.iter().zip(expected) {
            ensure!(
                actual.parse::<u64>()? == expected,
                "incorrect {label}: {line}"
            );
        }
        let ratio_cell = values[columns - 1];
        if purposes[0] == 0 {
            ensure!(ratio_cell == "-", "ratio without production: {label}");
        } else {
            let actual = ratio_cell.parse::<f64>()?;
            let ratio = purposes[1] as f64 / purposes[0] as f64;
            ensure!((actual - ratio).abs() <= 0.005, "incorrect ratio: {label}");
        }
        if !is_total {
            row_count += 1;
            for (index, _) in selected {
                coverage[index] += 1;
            }
        }
    }
    ensure!(total_seen, "missing Total");
    ensure!(
        coverage.iter().all(|count| *count == 1),
        "rows must cover each contribution once"
    );
    if folders {
        ensure!(row_count == 30, "sample must demonstrate the 30-row budget");
    }
    Ok(())
}

#[test]
fn readme_tables_match_renderer_and_reconcile() {
    let groups = fixture();
    let readme = fs::read_to_string("README.md").unwrap();
    for (marker, rendered) in tables(&groups) {
        let published = block(&readme, &marker);
        reconcile(published, &groups, marker == "readme-folders").unwrap();
        assert_eq!(
            published, rendered,
            "refresh with the ignored refresh_readme_tables test"
        );
    }
}

#[test]
fn arithmetic_guard_rejects_wrong_counts_and_duplicate_folder_rows() {
    let groups = fixture();
    let [(_, language), (_, folders)] = tables(&groups);
    reconcile(&language, &groups, false).unwrap();
    reconcile(&folders, &groups, true).unwrap();
    let count = language
        .split_whitespace()
        .find(|cell| cell.replace(',', "").parse::<u64>().is_ok())
        .unwrap();
    let incremented = count.replace(',', "").parse::<u64>().unwrap() + 1;
    let wrong_count = language.replacen(count, &incremented.to_string(), 1);
    assert!(reconcile(&wrong_count, &groups, false).is_err());
    let row = folders
        .lines()
        .find(|line| line.starts_with(" backend/"))
        .unwrap();
    let duplicate = folders.replacen(row, &format!("{row}\n{row}"), 1);
    assert!(reconcile(&duplicate, &groups, true).is_err());
}

#[test]
#[ignore = "explicit maintenance command: rewrites the two marked README tables"]
fn refresh_readme_tables() {
    let path = Path::new("README.md");
    let mut readme = fs::read_to_string(path).unwrap();
    let groups = fixture();
    for (marker, rendered) in tables(&groups) {
        reconcile(&rendered, &groups, marker == "readme-folders").unwrap();
        let start = readme
            .find(&format!("<!-- {marker} -->\n```text\n"))
            .unwrap()
            + format!("<!-- {marker} -->\n```text\n").len();
        let end = start + readme[start..].find("```\n").unwrap();
        readme.replace_range(start..end, &rendered);
    }
    fs::write(path, readme).unwrap();
}

fn help(flag: &str) -> String {
    let output = std::process::Command::new(env!("CARGO_BIN_EXE_locmetry"))
        .arg(flag)
        .output()
        .unwrap();
    assert!(output.status.success());
    String::from_utf8(output.stdout).unwrap()
}

// Long help owns the examples; the README table is a checked copy, so neither can drift.
#[test]
fn readme_common_commands_mirror_long_help_examples() {
    let long = help("--help");
    let examples: Vec<(&str, &str)> = long
        .split_once("\nExamples:\n")
        .expect("long help ends with examples")
        .1
        .split("\n\n")
        .next()
        .unwrap()
        .lines()
        .collect::<Vec<_>>()
        .chunks(2)
        .map(|pair| {
            let task = pair[0].strip_prefix("  ").unwrap();
            let command = pair[1].strip_prefix("    ").unwrap();
            (task.strip_suffix(':').unwrap(), command)
        })
        .collect();
    let readme = fs::read_to_string("README.md").unwrap();
    let table: Vec<(&str, &str)> = readme
        .split_once("## Common commands\n\n| Task | Command |\n| --- | --- |\n")
        .unwrap()
        .1
        .lines()
        .take_while(|line| line.starts_with('|'))
        .map(|row| {
            let (task, command) = row
                .strip_prefix("| ")
                .and_then(|row| row.strip_suffix(" |"))
                .and_then(|row| row.split_once(" | "))
                .unwrap();
            (task, command.trim_matches('`'))
        })
        .collect();
    assert!(!examples.is_empty());
    assert_eq!(
        table, examples,
        "edit the examples in src/main.rs `after_long_help`, then mirror them in README"
    );
    assert_eq!(help("help"), long);
    assert!(!help("-h").contains("Examples:"));
}
