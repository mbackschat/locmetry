//! The reference topic: exit status, environment, files and generated
//! documents in one guide that the man page renders
//! (docs/proposals/archive/manual-and-reference.md).
use crate::common;
use crate::guide_sections::{section, source};
use std::collections::BTreeSet;
use std::path::Path;

fn help() -> String {
    common::stdout(
        common::locmetry()
            .env("COLUMNS", "100")
            .env("NO_COLOR", "1")
            .arg("--help")
            .output()
            .unwrap(),
    )
}

/// The names Locmetry's own source looks up: direct `env::var` and `var_os`
/// reads, and the injected lookup of the personal defaults file, whose
/// `var(...)` and `absolute(...)` calls cover both platform branches.
fn variables_read() -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    let mut folders = vec![Path::new(env!("CARGO_MANIFEST_DIR")).join("src")];
    while let Some(folder) = folders.pop() {
        for entry in std::fs::read_dir(folder).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                folders.push(path);
            } else if path.extension().is_some_and(|extension| extension == "rs") {
                let text = std::fs::read_to_string(&path).unwrap();
                for call in ["var(\"", "var_os(\"", "absolute(\""] {
                    for (at, _) in text.match_indices(call) {
                        let name = &text[at + call.len()..];
                        let name = &name[..name.find('"').unwrap()];
                        if name.chars().all(|c| c.is_ascii_uppercase() || c == '_') {
                            names.insert(name.to_owned());
                        }
                    }
                }
            }
        }
    }
    names
}

/// The backticked names that open the bullets of a section.
fn bullet_names(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|line| line.strip_prefix("- `"))
        .map(|rest| rest[..rest.find('`').unwrap()].to_owned())
        .collect()
}

#[test]
fn the_environment_section_lists_exactly_the_variables_the_source_reads() {
    let read = variables_read();
    assert!(
        read.contains("COLUMNS") && read.contains("APPDATA"),
        "the source scan found {read:?}"
    );
    let listed = bullet_names(section(&source("reference"), "environment"));
    assert_eq!(listed, read);
}

/// Help's width comes from clap 4.6.7 (`HelpTemplate::term_w`), not from the
/// share-bar width: a terminal on stdout, stderr or stdin wins over `COLUMNS`,
/// and without either help wraps at 100. Only the last two can be locked
/// without a terminal; the guide must state the 100, not the bars' 80.
#[test]
fn help_wraps_to_columns_and_otherwise_to_100_columns() {
    let widest = |columns: Option<&str>| {
        let mut command = common::locmetry();
        command.env("NO_COLOR", "1").env_remove("COLUMNS");
        if let Some(columns) = columns {
            command.env("COLUMNS", columns);
        }
        let output = command
            .arg("--help")
            .stdin(std::process::Stdio::null())
            .output()
            .unwrap();
        let help = common::stdout(output);
        help.lines().map(|line| line.chars().count()).max().unwrap()
    };
    assert_eq!(widest(Some("60")), 60);
    assert_eq!(widest(None), 100);
    let reference = source("reference");
    let bullet = section(&reference, "environment")
        .lines()
        .find(|line| line.starts_with("- `COLUMNS`"))
        .unwrap();
    assert!(bullet.contains("100"), "{bullet}");
}

#[test]
fn help_and_the_reference_name_the_same_exit_statuses() {
    let help = help().split_whitespace().collect::<Vec<_>>().join(" ");
    let in_help: BTreeSet<String> = ["0", "1", "2"]
        .into_iter()
        .filter(|status| help.contains(&format!("Exit {status} ")))
        .map(String::from)
        .collect();
    let in_guide = bullet_names(section(&source("reference"), "exit-status"));
    assert_eq!(in_help, in_guide);
    assert_eq!(in_guide.len(), 3);
}

/// The man page renders the reference guide, so it may use only what the
/// renderer supports: headings, paragraphs, flat bullet lists, inline code,
/// code blocks and links.
#[test]
fn the_reference_guide_stays_within_the_man_page_subset() {
    let mut fenced = false;
    for line in source("reference").lines() {
        if line.starts_with("```") {
            fenced = !fenced;
            continue;
        }
        if fenced || line.is_empty() {
            continue;
        }
        assert!(
            !line.starts_with('|')
                && !line.starts_with(' ')
                && !line.starts_with('<')
                && !line.starts_with('>')
                && !line.starts_with("![")
                && !line.starts_with("* ")
                && !line.starts_with("#### ")
                && !line.chars().next().unwrap().is_ascii_digit(),
            "unsupported Markdown in the reference guide: {line}"
        );
    }
    assert!(!fenced, "unterminated code block");
}
