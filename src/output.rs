use crate::{breakdown, Counts, Coverage, FileRecord, Kind, Purpose, ReportSummary};
use anyhow::{ensure, Context, Result};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

fn cell(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('|', "&#124;")
        .replace('`', "&#96;")
        .replace(['\r', '\n'], " ")
}

fn count_row(output: &mut String, label: &str, counts: &Counts) {
    writeln!(
        output,
        "| {} | {} | {} | {} | {} | {} |",
        cell(label),
        counts.files,
        counts.code,
        counts.comments,
        counts.blanks,
        counts.lines
    )
    .unwrap();
}

fn table(output: &mut String, title: &str, label: &str, counts: &BTreeMap<String, Counts>) {
    writeln!(output, "\n## {title}\n\n| {label} | Files | Code | Comments | Blanks | Lines |\n| --- | ---: | ---: | ---: | ---: | ---: |").unwrap();
    for (label, counts) in counts {
        count_row(output, label, counts);
    }
}

pub fn render_markdown(
    project: &str,
    summary: &ReportSummary,
    rows: &[FileRecord],
    coverage: &Coverage,
    depth: usize,
) -> String {
    let mut output = format!("# {}: lines of code by purpose\n\nProduction means maintained non-test source, including separately categorized tooling. Data, configuration, and documentation are counted apart from source.\n\n## Source purpose\n\n| Purpose | Files | Code | Comments | Blanks | Lines |\n| --- | ---: | ---: | ---: | ---: | ---: |\n", cell(project));
    for purpose in Purpose::ALL {
        count_row(
            &mut output,
            &purpose.to_string(),
            &summary.totals.purpose(purpose),
        );
    }
    table(
        &mut output,
        "Source categories",
        "Category",
        &summary.totals.source_categories,
    );
    let other = summary
        .totals
        .by_kind_purpose
        .iter()
        .filter(|(key, _)| !key.starts_with("source:"))
        .map(|(key, value)| (key.clone(), *value))
        .collect();
    table(&mut output, "Other counted content", "Kind:purpose", &other);
    table(
        &mut output,
        "Source languages",
        "Language",
        &summary.totals.source_languages,
    );
    write!(output, "\n## Folder breakdown\n\nRecursive source code totals to depth {depth}. Parent rows include their children; do not sum all displayed rows. The report file's additive `breakdown` rows cover every depth. Multi-language files count once physically but contribute to each contained language.\n\n| Folder | Production | Test | Unclassified | Total source code |\n| --- | ---: | ---: | ---: | ---: |\n").unwrap();
    for (folder, values) in &summary.folders {
        let totals = &values.recursive;
        if folder.split('/').count() > depth || totals.source.files == 0 {
            continue;
        }
        writeln!(
            output,
            "| {} | {} | {} | {} | {} |",
            cell(folder),
            totals.purpose(Purpose::Production).code,
            totals.purpose(Purpose::Test).code,
            totals.purpose(Purpose::Unclassified).code,
            totals.source.code
        )
        .unwrap();
    }
    let mut unknown: Vec<_> = rows
        .iter()
        .filter(|r| r.kind == Kind::Source && r.classification.purpose == Purpose::Unclassified)
        .collect();
    unknown.sort_by(|a, b| b.counts.code.cmp(&a.counts.code).then(a.path.cmp(&b.path)));
    writeln!(output, "\n## Source requiring review\n\n{} files. Largest 25 below; the report file's `review` list names all of them with their rule.\n\n| Path | Code | Category |\n| --- | ---: | --- |", unknown.len()).unwrap();
    for row in unknown.into_iter().take(25) {
        writeln!(
            output,
            "| {} | {} | {} |",
            cell(&row.path),
            row.counts.code,
            cell(&row.classification.category)
        )
        .unwrap();
    }
    render_coverage(&mut output, coverage);
    output
}

/// `count` followed by `noun`, pluralized with `s` unless it is one.
fn counted(count: u64, noun: &str) -> String {
    match count {
        1 => format!("1 {noun}"),
        _ => format!("{count} {noun}s"),
    }
}

/// Aligned plain-text table: the first column left-aligned, the others right.
/// Rule lines separate the header, and the body from an optional total row.
fn text_table(header: &[&str], body: &[Vec<String>], total: Option<Vec<String>>) -> String {
    let header: Vec<String> = header.iter().map(|title| title.to_string()).collect();
    let all = std::iter::once(&header).chain(body).chain(total.as_ref());
    let mut widths = vec![0; header.len()];
    for row in all {
        for (width, value) in widths.iter_mut().zip(row) {
            *width = (*width).max(value.chars().count());
        }
    }
    let line = |row: &[String]| {
        let cells: Vec<String> = row
            .iter()
            .zip(&widths)
            .enumerate()
            .map(|(column, (value, width))| match column {
                0 => format!("{value:<width$}"),
                _ => format!("{value:>width$}"),
            })
            .collect();
        format!(" {}\n", cells.join("  "))
    };
    let rule = format!(
        " {}\n",
        "─".repeat(widths.iter().sum::<usize>() + 2 * (widths.len() - 1))
    );
    let mut output = line(&header) + &rule;
    for row in body {
        output += &line(row);
    }
    if let Some(total) = total {
        output += &rule;
        output += &line(&total);
    }
    output
}

/// Terminal report for people: source code per language and top-level folder
/// split by purpose, then one line each for what the tables leave out.
pub fn render_table(
    project: &str,
    summary: &ReportSummary,
    rows: &[FileRecord],
    coverage: &Coverage,
) -> String {
    let mut languages = BTreeMap::<String, (Counts, BTreeMap<Purpose, u64>)>::new();
    for row in breakdown(rows)
        .iter()
        .filter(|row| row.kind == Kind::Source)
    {
        let (counts, purposes) = languages.entry(row.language.clone()).or_default();
        counts.add(&row.counts);
        *purposes.entry(row.purpose).or_default() += row.counts.code;
    }
    let mut languages: Vec<_> = languages.into_iter().collect();
    languages.sort_by(|a, b| b.1 .0.code.cmp(&a.1 .0.code).then(a.0.cmp(&b.0)));
    let purpose_cells = |code: &dyn Fn(Purpose) -> u64| {
        Purpose::ALL
            .into_iter()
            .map(|purpose| code(purpose).to_string())
            .collect::<Vec<_>>()
    };
    let body: Vec<Vec<String>> = languages
        .iter()
        .map(|(language, (counts, purposes))| {
            let mut row = vec![language.clone(), counts.files.to_string()];
            row.extend(purpose_cells(&|purpose| {
                purposes.get(&purpose).copied().unwrap_or(0)
            }));
            row.extend([counts.code, counts.comments, counts.blanks].map(|n| n.to_string()));
            row
        })
        .collect();
    let source = &summary.totals.source;
    let mut total = vec!["Total".to_string(), source.files.to_string()];
    total.extend(purpose_cells(&|purpose| {
        summary.totals.purpose(purpose).code
    }));
    total.extend([source.code, source.comments, source.blanks].map(|n| n.to_string()));
    let mut output = format!("\n {project}: source code lines by purpose\n\n");
    output += &text_table(
        &[
            "Language",
            "Files",
            "Production",
            "Test",
            "Unclassified",
            "Code",
            "Comments",
            "Blanks",
        ],
        &body,
        Some(total),
    );

    let mut folders: Vec<_> = summary.top_folders().into_iter().collect();
    let folder_code = |purposes: &BTreeMap<Purpose, u64>| purposes.values().sum::<u64>();
    folders.sort_by(|a, b| {
        folder_code(&b.1)
            .cmp(&folder_code(&a.1))
            .then(a.0.cmp(&b.0))
    });
    if !folders.is_empty() {
        let body: Vec<Vec<String>> = folders
            .iter()
            .map(|(folder, purposes)| {
                // `.` in top_folders holds only the files directly in the root.
                let label = match folder.as_str() {
                    "." => "(root files)".to_string(),
                    _ => folder.clone(),
                };
                let mut row = vec![label];
                row.extend(purpose_cells(&|purpose| {
                    purposes.get(&purpose).copied().unwrap_or(0)
                }));
                row.push(folder_code(purposes).to_string());
                row
            })
            .collect();
        output += "\n";
        output += &text_table(
            &["Folder", "Production", "Test", "Unclassified", "Code"],
            &body,
            None,
        );
    }

    let mut notes = Vec::new();
    let mut other_kinds = BTreeMap::<Kind, Counts>::new();
    for row in rows.iter().filter(|row| row.kind != Kind::Source) {
        other_kinds.entry(row.kind).or_default().add(&row.counts);
    }
    if !other_kinds.is_empty() {
        let kinds: Vec<String> = other_kinds
            .iter()
            .map(|(kind, counts)| {
                format!(
                    "{} ({})",
                    counted(counts.files, &format!("{kind} file")),
                    counted(counts.lines, "line")
                )
            })
            .collect();
        notes.push(format!("Not source code: {}.", kinds.join(", ")));
    }
    let unclassified = summary.totals.purpose(Purpose::Unclassified);
    if unclassified.files > 0 {
        notes.push(format!(
            "Review: {} ({}); details with --format markdown.",
            counted(unclassified.files, "unclassified source file"),
            counted(unclassified.code, "code line")
        ));
    }
    let mut gaps = Vec::new();
    if coverage.unrecognized_files > 0 {
        let mut extensions: Vec<_> = coverage.unrecognized_extensions.iter().collect();
        extensions.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        let extensions: Vec<String> = extensions
            .into_iter()
            .take(5)
            .map(|(extension, files)| format!("{extension}: {files}"))
            .collect();
        gaps.push(format!(
            "{} ({})",
            counted(coverage.unrecognized_files as u64, "unrecognized file"),
            extensions.join(", ")
        ));
    }
    if !coverage.unread.is_empty() {
        gaps.push(counted(coverage.unread.len() as u64, "unreadable file"));
    }
    if !gaps.is_empty() {
        notes.push(format!(
            "Not counted: {}; missing lines are not zero.",
            gaps.join(", ")
        ));
    }
    if !notes.is_empty() {
        output += "\n";
        for note in notes {
            writeln!(output, " {note}").unwrap();
        }
    }
    output
}

fn render_coverage(output: &mut String, coverage: &Coverage) {
    writeln!(output, "\n## Coverage\n\nFiles selected: {}; counted: {}, including {} through language mappings; not recognized: {}; recognized but unreadable: {}. Their lines are missing, not zero. Counts describe the working copy, including untracked files, not a commit.", coverage.selected_files, coverage.counted_files, coverage.mapped_files, coverage.unrecognized_files, coverage.unread.len()).unwrap();
    let mut extensions: Vec<_> = coverage.unrecognized_extensions.iter().collect();
    extensions.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    if !extensions.is_empty() {
        output.push_str("\n| Unrecognized extension | Files |\n| --- | ---: |\n");
        for (extension, files) in extensions.into_iter().take(15) {
            writeln!(output, "| {} | {files} |", cell(extension)).unwrap();
        }
    }
    if !coverage.unread.is_empty() {
        output.push_str("\n| Unreadable file |\n| --- |\n");
        for path in coverage.unread.iter().take(25) {
            writeln!(output, "| {} |", cell(path)).unwrap();
        }
    }
}

fn resolve_destination(path: &Path) -> Result<PathBuf> {
    if fs::symlink_metadata(path).is_ok() {
        return Ok(path.canonicalize()?);
    }
    let name = path.file_name().context("Invalid output path")?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    Ok(resolve_destination(parent)?.join(name))
}

pub fn validate_output(output: &Path, root: &Path, config: Option<&Path>) -> Result<()> {
    let output = resolve_destination(output)?;
    ensure!(
        !output.starts_with(root.canonicalize()?),
        "Output must be outside the read-only scan root"
    );
    let config = config.map(Path::canonicalize).transpose()?;
    ensure!(
        config.as_ref() != Some(&output),
        "Output would overwrite the configuration: {}",
        output.display()
    );
    Ok(())
}

/// Report keys in output order; absent keys are skipped.
const REPORT_KEYS: [&str; 7] = [
    "schema_version",
    "project",
    "metadata",
    "summary",
    "review",
    "breakdown",
    "files",
];

/// Serializes a report with one top-level key per line and one table row per
/// line: still one JSON document, but `grep`, `wc -l`, and `diff` work on rows.
pub fn report_text(document: &Value) -> Result<String> {
    let mut entries = Vec::new();
    for key in REPORT_KEYS {
        let Some(value) = document.get(key) else {
            continue;
        };
        let value = match value {
            Value::Array(rows) if !rows.is_empty() => {
                let rows = rows
                    .iter()
                    .map(serde_json::to_string)
                    .collect::<Result<Vec<_>, _>>()?;
                format!("[\n{}\n]", rows.join(",\n"))
            }
            _ => serde_json::to_string(value)?,
        };
        entries.push(format!("{}:{value}", serde_json::to_string(key)?));
    }
    Ok(format!("{{\n{}\n}}\n", entries.join(",\n")))
}

pub fn write_report(output: &Path, text: &str) -> Result<()> {
    if let Some(parent) = output.parent().filter(|p| !p.as_os_str().is_empty()) {
        fs::create_dir_all(parent)?;
    }
    fs::write(output, text).with_context(|| format!("Cannot write report {}", output.display()))
}
