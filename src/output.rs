mod bars;
mod folders;
mod grid;
mod markdown;
mod notes;
mod purposes;
mod selection;
mod terminal;

pub use bars::*;
pub(crate) use grid::*;
pub use markdown::render_markdown;
use markdown::*;
use notes::*;
pub(crate) use purposes::*;
pub use terminal::{render_table_with_options, TableOptions};

pub(crate) use selection::folded_note;
use selection::TableSelection;
pub use selection::{CategoryPattern, Fold, LanguageVariants};

use crate::report::Summary;
use crate::style::{Role, Theme};
use crate::{
    build_systems, review, unclaimed_folders, Counts, Coverage, FileRecord, FolderShare, Kind,
    ModuleFile, ModuleShare, Purpose, ReportSummary, SubmoduleShare,
};
use anyhow::{ensure, Context, Result};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

use crate::style::plain;

/// UTC measurement time from recorded Unix seconds; extreme historical values retain their numeric evidence.
pub fn measurement_time(seconds: u64) -> String {
    i64::try_from(seconds)
        .ok()
        .and_then(|seconds| chrono::DateTime::from_timestamp(seconds, 0))
        .map_or_else(
            || format!("{seconds} Unix seconds (outside the supported UTC calendar range)"),
            |date| date.format("%Y-%m-%d %H:%M:%S UTC").to_string(),
        )
}

pub(crate) fn cell(value: &str) -> String {
    plain(value)
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('|', "&#124;")
        .replace('`', "&#96;")
}

/// Files, Lines, Code, Comments and Blanks, as count tables list them.
fn count_cells(counts: &Counts) -> [String; 5] {
    [
        counts.files,
        counts.lines,
        counts.code,
        counts.comments,
        counts.blanks,
    ]
    .map(grouped)
}

fn count_row(output: &mut String, label: &str, counts: &Counts) {
    writeln!(
        output,
        "| {} | {} |",
        cell(label),
        count_cells(counts).join(" | ")
    )
    .unwrap();
}

/// Row order of the flat tables in the table and Markdown formats.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RowOrder {
    /// Largest code first, ties by label, so the order is stable.
    Code,
    /// Alphabetical by label.
    Name,
}

/// What a measurement found beside the per-file records, for the notes of
/// both human formats.
pub struct Findings<'a> {
    pub subfolder: Option<&'a str>,
    pub coverage: &'a Coverage,
    pub modules: &'a [ModuleShare],
    /// The largest files module layouts claimed, by code.
    pub module_files: &'a [ModuleFile],
    pub submodules: &'a [SubmoduleShare],
    pub unused_entries: &'a [String],
    pub unused_exclusions: &'a [String],
    /// Workspace members in name order; empty for a single root.
    pub members: &'a [MemberView],
}

/// A workspace member as the human formats show it: a member row, and
/// the findings of the per-member note.
pub struct MemberView {
    pub name: String,
    pub unclaimed_files: u64,
    pub unclaimed_code: u64,
    pub unread_files: usize,
    pub parse_failed_files: usize,
}

/// Tests often live in another language than their subject, such as Kotlin
/// tests for Java code; docs/plans/archive/test-production-ratios.md requires the note.
const LANGUAGE_RATIO_NOTE: &str =
    "Tests written in another language than the code they test skew per-language Test/prod.";
const CATEGORY_LEGEND: &str =
    "Rows of one purpose add up to that purpose's Code, and all rows add up to the total Code.";

/// Sorts labelled rows by `order`, where `code` gives a row's code lines.
fn ordered<T>(
    mut rows: Vec<(String, T)>,
    code: impl Fn(&T) -> u64,
    order: RowOrder,
) -> Vec<(String, T)> {
    match order {
        RowOrder::Code => rows.sort_by(|a, b| code(&b.1).cmp(&code(&a.1)).then(a.0.cmp(&b.0))),
        RowOrder::Name => rows.sort_by(|a, b| a.0.cmp(&b.0)),
    }
    rows
}

/// `count` with a comma between groups of three digits, so large counts read
/// at a glance in the human formats; the JSON report keeps plain integers.
pub(crate) fn grouped(count: u64) -> String {
    let digits = count.to_string();
    let mut output = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            output.push(',');
        }
        output.push(digit);
    }
    output
}

/// A signed change grouped like [`grouped`]; no change is a plain `0`, which
/// tables mute like any other zero.
pub(crate) fn signed(change: i128) -> String {
    let magnitude = grouped(change.unsigned_abs() as u64);
    match change.signum() {
        1 => format!("+{magnitude}"),
        -1 => format!("-{magnitude}"),
        _ => magnitude,
    }
}

/// Label of a module folder; `.` is the scan root.
pub(crate) fn module_label(path: &str) -> &str {
    match path {
        "." => "scan root",
        path => path,
    }
}

/// `noun`, pluralized with `s` unless `count` is one.
fn plural(count: u64, noun: &str) -> String {
    let consonant_y = noun
        .strip_suffix('y')
        .filter(|stem| stem.ends_with(|c: char| !"aeiou".contains(c)));
    match (count, consonant_y) {
        (1, _) => noun.to_string(),
        (_, Some(stem)) => format!("{stem}ies"),
        (_, None) => format!("{noun}s"),
    }
}

/// `count` followed by `noun`, pluralized unless it is one.
pub(crate) fn counted(count: u64, noun: &str) -> String {
    format!("{} {}", grouped(count), plural(count, noun))
}

/// `names` as prose: `A`, `A and B`, or `A, B and C`.
pub(crate) fn listed(mut names: Vec<String>) -> String {
    match names.pop() {
        None => String::new(),
        Some(last) if names.is_empty() => last,
        Some(last) => format!("{} and {last}", names.join(", ")),
    }
}

/// A literal value inside a sentence, single-quoted as the human output
/// wording rules require (docs/help-design.md, Human Output Wording).
pub(crate) fn quoted(value: &str) -> String {
    format!("'{value}'")
}

/// A library's message with its backtick quotes made single quotes, as the
/// human output wording rules require (docs/help-design.md, Human Output Wording).
pub(crate) fn single_quoted(message: &str) -> String {
    message.replace('`', "'")
}

/// Source counts and code per purpose for each language. As in the
/// breakdown, every language of a file adds its lines, and only the file's own
/// language counts the file.
fn source_languages(rows: &[FileRecord]) -> BTreeMap<String, (Counts, RowCode)> {
    let mut languages = BTreeMap::<&str, (Counts, RowCode)>::new();
    for row in rows.iter().filter(|row| row.kind == Kind::Source) {
        for (language, counts) in &row.languages {
            let mut counts = *counts;
            if *language != row.language {
                counts.files = 0;
            }
            let (total, code) = languages.entry(language).or_default();
            total.add(&counts);
            code.purposes[row.classification.purpose.index()] += counts.code;
            code.comments += counts.comments;
        }
    }
    languages
        .into_iter()
        .map(|(language, entry)| (language.to_string(), entry))
        .collect()
}

/// Source counts and code per `purpose:category` key; each key holds a single
/// purpose, so its row's bar has one segment.
fn source_categories(rows: &[FileRecord]) -> BTreeMap<String, (Counts, RowCode)> {
    let mut categories = BTreeMap::<String, (Counts, RowCode)>::new();
    for row in rows.iter().filter(|row| row.kind == Kind::Source) {
        let purpose = row.classification.purpose;
        let key = crate::report::category_key(purpose, &row.classification.category);
        let (total, code) = categories.entry(key).or_default();
        total.add(&row.counts);
        code.purposes[purpose.index()] += row.counts.code;
        code.comments += row.counts.comments;
    }
    categories
}

pub(crate) fn section_title(title: &str, theme: Theme) -> String {
    format!(
        "{}\n\n",
        theme.strong(Role::Heading, &format!("▸ {}", plain(title)))
    )
}

fn resolve_destination(path: &Path) -> Result<PathBuf> {
    if fs::symlink_metadata(path).is_ok() {
        return Ok(dunce::canonicalize(path)?);
    }
    let name = path.file_name().context("Invalid output path")?;
    let parent = path
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    Ok(resolve_destination(parent)?.join(name))
}

pub fn validate_output(output: &Path, root: &Path, config: Option<&Path>) -> Result<()> {
    let given = quoted(&output.display().to_string());
    let output = resolve_destination(output)?;
    ensure!(
        !output.starts_with(dunce::canonicalize(root)?),
        "Output {given} must be outside the read-only scan root"
    );
    let config = config.map(dunce::canonicalize).transpose()?;
    ensure!(
        config.as_ref() != Some(&output),
        "Output {given} would overwrite the configuration"
    );
    Ok(())
}

/// Serializes a report with one top-level key per line and one table row per
/// line: still one JSON document, but `grep`, `wc -l`, and `diff` work on rows.
pub fn report_text(document: &Value) -> Result<String> {
    let mut entries = Vec::new();
    for (key, value) in document
        .as_object()
        .context("Report must be a JSON object")?
    {
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
    fs::write(output, text).with_context(|| format!("Cannot write report '{}'", output.display()))
}

fn display_project(project: &str, subfolder: Option<&str>) -> String {
    match subfolder {
        Some(folder) => format!("{} · subfolder {}", plain(project), plain(folder)),
        None => plain(project),
    }
}

fn human_folders<'a>(
    summary: &'a ReportSummary,
    subfolder: Option<&str>,
    depth: usize,
) -> Vec<(String, &'a crate::Summary)> {
    let mut folders = Vec::new();
    if let Some(root) = subfolder.filter(|_| depth > 0) {
        if let Some(folder) = summary.folders.get(root) {
            if folder.direct.source.files > 0 {
                folders.push((format!("{root} (direct files)"), &folder.direct));
            }
        }
    }
    for (path, folder) in &summary.folders {
        let level = match subfolder {
            Some(root) => match path.strip_prefix(root).and_then(|p| p.strip_prefix('/')) {
                Some(relative) => relative.split('/').count(),
                None => continue,
            },
            None => path.split('/').count(),
        };
        if level <= depth {
            folders.push((path.clone(), &folder.recursive));
        }
    }
    folders
}

#[cfg(test)]
mod tests;
