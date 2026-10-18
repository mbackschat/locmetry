mod folders;

use crate::report::Summary;
use crate::style::{Role, Theme};
use crate::{
    breakdown, build_systems, unclassified_folders, Counts, Coverage, FileRecord, FolderShare,
    Kind, ModuleShare, Purpose, ReportSummary, SubmoduleShare,
};
use anyhow::{ensure, Context, Result};
use serde_json::Value;
use std::{
    collections::BTreeMap,
    fmt::Write,
    fs,
    path::{Path, PathBuf},
};

pub(crate) use crate::style::plain;

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

fn count_row(output: &mut String, label: &str, counts: &Counts) {
    writeln!(
        output,
        "| {} | {} | {} | {} | {} | {} |",
        cell(label),
        grouped(counts.files),
        grouped(counts.lines),
        grouped(counts.code),
        grouped(counts.comments),
        grouped(counts.blanks)
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
    pub submodules: &'a [SubmoduleShare],
    pub unused_entries: &'a [String],
    pub unused_exclusions: &'a [String],
    /// Workspace members in name order; empty for a single root.
    pub members: &'a [MemberView],
}

/// A workspace member as the human formats show it: a repository row, and
/// the findings of the per-repository note.
pub struct MemberView {
    pub name: String,
    pub unclaimed_files: u64,
    pub unclaimed_code: u64,
    pub unread_files: usize,
    pub parse_failed_files: usize,
}

/// Test code lines per production code line, `-` without production code.
fn test_ratio(production: u64, test: u64) -> String {
    match production {
        0 => "-".to_string(),
        _ => format!("{:.2}", test as f64 / production as f64),
    }
}

/// Repository rows and the total row: source lines, code per purpose, test
/// lines per production line, and share of the workspace's source code.
fn repository_rows(
    summary: &ReportSummary,
    members: &[MemberView],
    order: RowOrder,
) -> (Vec<Vec<String>>, Vec<String>) {
    let workspace_code = summary.totals.source.code;
    let cells = |label: &str, totals: &Summary| {
        let code = |purpose| totals.purpose(purpose).code;
        let (production, test) = (code(Purpose::Production), code(Purpose::Test));
        let ratio = test_ratio(production, test);
        let share = match workspace_code {
            0 => "-".to_string(),
            _ => format!(
                "{:.1}%",
                totals.source.code as f64 * 100.0 / workspace_code as f64
            ),
        };
        let mut row = vec![
            label.to_string(),
            grouped(totals.source.lines),
            grouped(totals.source.code),
        ];
        row.extend(Purpose::ALL.map(|purpose| grouped(code(purpose))));
        row.extend([ratio, share]);
        row
    };
    let body = member_totals(summary, members, order)
        .into_iter()
        .map(|(name, totals)| cells(&name, totals))
        .collect();
    (body, cells("Total", &summary.totals))
}

/// Each member's recursive totals, in table row order.
fn member_totals<'a>(
    summary: &'a ReportSummary,
    members: &[MemberView],
    order: RowOrder,
) -> Vec<(String, &'a Summary)> {
    static EMPTY: std::sync::LazyLock<Summary> = std::sync::LazyLock::new(Summary::default);
    let rows = members
        .iter()
        .map(|member| {
            let totals = summary
                .folders
                .get(&member.name)
                .map_or(&*EMPTY, |folder| &folder.recursive);
            (member.name.clone(), totals)
        })
        .collect();
    ordered(rows, |totals| totals.source.code, order)
}

/// Legends that state how a table's columns add up; readers otherwise
/// misread Code as a sum of Comments and Blanks.
const LANGUAGE_LEGEND: &str = "Lines = Code + Comments + Blanks; Production + Test + Unclassified = Code; Test/prod = Test / Production.";
/// Tests often live in another language than their subject, such as Kotlin
/// tests for Java code; docs/plans/archive/test-production-ratios.md requires the note.
const LANGUAGE_RATIO_NOTE: &str =
    "Tests written in another language than the code they test skew per-language Test/prod.";
const FOLDER_LEGEND: &str =
    "Production + Test + Unclassified = Code; Test/prod = Test / Production.";
const REPOSITORY_LEGEND: &str = "Production + Test + Unclassified = Code; Test/prod = Test / Production; Share = Code / workspace Code.";
const CATEGORY_LEGEND: &str =
    "Rows of one purpose add up to that purpose's Code; all rows add up to the total Code.";

const REPOSITORY_HEADER: [&str; 8] = [
    "Repository",
    "Lines",
    "Code",
    "Production",
    "Test",
    "Unclassified",
    "Test/prod",
    "Share",
];

/// A Markdown grid of source lines and code per `key` and member, with totals;
/// `of` selects the map, such as categories or languages, from a summary.
fn repository_grid(
    output: &mut String,
    title: &str,
    label: &str,
    summary: &ReportSummary,
    members: &[MemberView],
    of: impl Fn(&Summary) -> &BTreeMap<String, Counts>,
    order: RowOrder,
) {
    let names: Vec<&str> = members.iter().map(|member| member.name.as_str()).collect();
    let mut headers = vec![label.to_owned()];
    for name in &names {
        headers.extend([
            format!("{} Lines", cell(name)),
            format!("{} Code", cell(name)),
        ]);
    }
    headers.extend(["Total Lines".into(), "Total Code".into()]);
    writeln!(
        output,
        "\n## {title}\n\n{}\n\n| {} |\n| --- |{}",
        Theme::MARKDOWN.paint(
            Role::Muted,
            "Each total is the sum of its repository columns."
        ),
        headers.join(" | "),
        " ---: |".repeat(headers.len() - 1)
    )
    .unwrap();
    let rows = of(&summary.totals)
        .iter()
        .map(|(key, counts)| (key.clone(), *counts))
        .collect();
    for (key, total) in ordered(rows, |counts| counts.code, order) {
        let mut cells = vec![cell(&key)];
        for name in &names {
            let counts = summary
                .folders
                .get(*name)
                .and_then(|folder| of(&folder.recursive).get(&key))
                .copied()
                .unwrap_or_default();
            cells.extend([grouped(counts.lines), grouped(counts.code)]);
        }
        cells.extend([grouped(total.lines), grouped(total.code)]);
        writeln!(output, "| {} |", cells.join(" | ")).unwrap();
    }
}

enum MemberNotes {
    All,
    Unclaimed,
    Unavailable,
}

fn repositories_note(members: &[MemberView], kind: MemberNotes) -> Option<String> {
    let parts: Vec<String> = members
        .iter()
        .filter_map(|member| {
            let mut findings = Vec::new();
            if !matches!(kind, MemberNotes::Unavailable) && member.unclaimed_files > 0 {
                findings.push(format!(
                    "{} ({})",
                    counted(member.unclaimed_files, "unclaimed file"),
                    counted(member.unclaimed_code, "code line")
                ));
            }
            if !matches!(kind, MemberNotes::Unclaimed) && member.unread_files > 0 {
                findings.push(counted(member.unread_files as u64, "unreadable file"));
            }
            if !matches!(kind, MemberNotes::Unclaimed) && member.parse_failed_files > 0 {
                findings.push(counted(
                    member.parse_failed_files as u64,
                    "file with a parsing failure",
                ));
            }
            (!findings.is_empty()).then(|| format!("{}: {}", member.name, findings.join(", ")))
        })
        .collect();
    let label = match kind {
        MemberNotes::All => "Repositories to review",
        MemberNotes::Unclaimed => "Unclaimed source by repository",
        MemberNotes::Unavailable => "Unavailable counts",
    };
    (!parts.is_empty()).then(|| format!("{label}: {}.", parts.join("; ")))
}

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

fn table(
    output: &mut String,
    title: &str,
    label: &str,
    counts: &BTreeMap<String, Counts>,
    order: RowOrder,
) {
    writeln!(output, "\n## {title}\n\n| {label} | Files | Lines | **Code** | Comments | Blanks |\n| --- | ---: | ---: | ---: | ---: | ---: |").unwrap();
    let rows = counts
        .iter()
        .map(|(label, counts)| (label.clone(), counts))
        .collect();
    for (label, counts) in ordered(rows, |counts| counts.code, order) {
        count_row(output, &label, counts);
    }
}

pub fn render_markdown(
    project: &str,
    summary: &ReportSummary,
    rows: &[FileRecord],
    findings: &Findings,
    depth: usize,
    order: RowOrder,
) -> String {
    let project = display_project(project, findings.subfolder);
    let mut output = format!("# {}: lines of code by purpose\n\n{}\n\n## Source purpose\n\n| Purpose | Files | Lines | **Code** | Comments | Blanks |\n| --- | ---: | ---: | ---: | ---: | ---: |\n", cell(&project), Theme::MARKDOWN.paint(Role::Muted, "Production means maintained non-test source, including separately categorized tooling. Data, configuration, and documentation are counted apart from source. Lines = Code + Comments + Blanks in every table with these columns, and the purpose rows add up to all source."));
    for purpose in Purpose::ALL {
        count_row(
            &mut output,
            &purpose.to_string(),
            &summary.totals.purpose(purpose),
        );
    }
    if !findings.members.is_empty() {
        let (body, total) = repository_rows(summary, findings.members, order);
        // Bold Code marks the column the purpose columns split, as the
        // terminal table does.
        let header: Vec<String> = REPOSITORY_HEADER
            .iter()
            .map(|title| match *title {
                "Code" => "**Code**".to_string(),
                _ => title.to_string(),
            })
            .collect();
        writeln!(
            output,
            "\n## Repositories\n\n| {} |\n| --- |{}",
            header.join(" | "),
            " ---: |".repeat(REPOSITORY_HEADER.len() - 1)
        )
        .unwrap();
        for (row, is_total) in body.iter().map(|row| (row, false)).chain([(&total, true)]) {
            let cells: Vec<String> = row
                .iter()
                .map(|value| {
                    let value = cell(value);
                    if is_total {
                        Theme::MARKDOWN.paint(Role::Heading, &value)
                    } else {
                        value
                    }
                })
                .collect();
            writeln!(output, "| {} |", cells.join(" | ")).unwrap();
        }
        writeln!(
            output,
            "\n{}",
            Theme::MARKDOWN.paint(Role::Muted, REPOSITORY_LEGEND)
        )
        .unwrap();
        repository_grid(
            &mut output,
            "Source categories by repository",
            "Category",
            summary,
            findings.members,
            |totals| &totals.source_categories,
            order,
        );
        repository_grid(
            &mut output,
            "Source languages by repository",
            "Language",
            summary,
            findings.members,
            |totals| &totals.source_languages,
            order,
        );
    }
    table(
        &mut output,
        "Source categories",
        "Category",
        &summary.totals.source_categories,
        order,
    );
    let other = summary
        .totals
        .by_kind_purpose
        .iter()
        .filter(|(key, _)| !key.starts_with("source:"))
        .map(|(key, value)| (key.clone(), *value))
        .collect();
    table(
        &mut output,
        "Other counted content",
        "Kind:purpose",
        &other,
        order,
    );
    writeln!(output, "\n## Source languages\n\n{}\n\n| Language | Files | Lines | **Code** | Comments | Blanks | Test/prod |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: |", Theme::MARKDOWN.paint(Role::Muted, LANGUAGE_RATIO_NOTE)).unwrap();
    for (language, (counts, purposes)) in ordered(
        source_languages(rows).into_iter().collect(),
        |(counts, _)| counts.code,
        order,
    ) {
        let code = |purpose| purposes.get(&purpose).copied().unwrap_or(0);
        writeln!(
            output,
            "| {} | {} | {} | {} | {} | {} | {} |",
            cell(&language),
            grouped(counts.files),
            grouped(counts.lines),
            grouped(counts.code),
            grouped(counts.comments),
            grouped(counts.blanks),
            test_ratio(code(Purpose::Production), code(Purpose::Test))
        )
        .unwrap();
    }
    write!(output, "\n## Folder breakdown\n\n{}\n\n| Folder | Lines | **Code** | Production | Test | Unclassified | Test/prod |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: |\n", Theme::MARKDOWN.paint(Role::Muted, &format!("Recursive source code totals to depth {depth}. Parent rows include their children; do not sum all displayed rows. The report file's additive `breakdown` rows cover every depth. Multi-language files count once physically but contribute to each contained language. Production + Test + Unclassified = Code; Test/prod = Test / Production."))).unwrap();
    // Compare by path components: as plain strings, `web-admin` would sort
    // between `web` and `web/src`, because `-` sorts before `/`.
    let mut folders = human_folders(summary, findings.subfolder, depth);
    folders.sort_by(|a, b| a.0.split('/').cmp(b.0.split('/')));
    for (folder, values) in folders {
        let totals = values;
        if totals.source.files == 0 {
            continue;
        }
        let (production, test) = (
            totals.purpose(Purpose::Production).code,
            totals.purpose(Purpose::Test).code,
        );
        writeln!(
            output,
            "| {} | {} | {} | {} | {} | {} | {} |",
            cell(&folder),
            grouped(totals.source.lines),
            grouped(totals.source.code),
            grouped(production),
            grouped(test),
            grouped(totals.purpose(Purpose::Unclassified).code),
            test_ratio(production, test)
        )
        .unwrap();
    }
    render_build_modules(&mut output, findings.modules, order);
    let mut unknown: Vec<_> = rows.iter().filter(|row| row.needs_review()).collect();
    unknown.sort_by(|a, b| b.counts.code.cmp(&a.counts.code).then(a.path.cmp(&b.path)));
    writeln!(
        output,
        "\n## Source requiring review\n\n{} of unclassified source that no rule, convention, or build module claims. Code an entry or convention leaves unclassified on purpose is not listed.",
        counted(unknown.len() as u64, "file")
    )
    .unwrap();
    let folders = unclassified_folders(rows);
    if !folders.is_empty() {
        output.push_str("\n| Folder | Files | Lines | Code |\n| --- | ---: | ---: | ---: |\n");
        for share in &folders {
            writeln!(
                output,
                "| {} | {} | {} | {} |",
                cell(folder_label(&share.folder)),
                grouped(share.files),
                grouped(
                    unknown
                        .iter()
                        .filter(|row| match share.folder.as_str() {
                            "." => !row.path.contains('/'),
                            folder => crate::report::below(&row.path, folder).is_some(),
                        })
                        .map(|row| row.counts.lines)
                        .sum()
                ),
                grouped(share.code)
            )
            .unwrap();
        }
    }
    writeln!(output, "\nLargest 25 files; the report file's `review` list names all of them with their rule.\n\n| Path | Lines | Code | Category |\n| --- | ---: | ---: | --- |").unwrap();
    for row in unknown.into_iter().take(25) {
        writeln!(
            output,
            "| {} | {} | {} | {} |",
            cell(&row.path),
            grouped(row.counts.lines),
            grouped(row.counts.code),
            cell(&row.classification.category)
        )
        .unwrap();
    }
    render_coverage(&mut output, findings);
    output
}

/// `count` followed by `noun`, pluralized with `s` unless it is one.
/// Build systems and every module, so a reader can see which build files
/// decided production and spot a module whose layout claimed nothing.
fn render_build_modules(output: &mut String, modules: &[ModuleShare], order: RowOrder) {
    if modules.is_empty() {
        return;
    }
    output.push_str("\n## Build modules\n\nFolders whose build file declares a source layout; that layout counts as production where no override, convention, or rule decided first. Code is the source each module classified; 0 means other stages claimed its files or its layout matched none.\n\n| Build system | Modules | Lines | Code |\n| --- | ---: | ---: | ---: |\n");
    let systems = build_systems(modules)
        .into_iter()
        .map(|(id, system)| (id.to_string(), system))
        .collect();
    for (id, system) in ordered(systems, |system| system.code, order) {
        writeln!(
            output,
            "| {id} | {} | {} | {} |",
            grouped(system.modules),
            grouped(
                modules
                    .iter()
                    .filter(|module| module.detector == id)
                    .map(|module| module.lines)
                    .sum()
            ),
            grouped(system.code)
        )
        .unwrap();
    }
    output.push_str("\n| Module | Build system | Lines | Code |\n| --- | --- | ---: | ---: |\n");
    for module in modules {
        let path = match module.path.as_str() {
            "." => "(scan root)",
            path => path,
        };
        writeln!(
            output,
            "| {} | {} | {} | {} |",
            cell(path),
            module.detector,
            grouped(module.lines),
            grouped(module.code)
        )
        .unwrap();
    }
}

/// Code each build system classified, in one line.
fn build_modules_note(modules: &[ModuleShare], order: RowOrder) -> Option<String> {
    // A system whose layouts claimed nothing does not explain production;
    // JSON and the Markdown module list keep it.
    let systems: Vec<_> = build_systems(modules)
        .into_iter()
        .filter(|(_, system)| system.code > 0)
        .map(|(id, system)| (id.to_string(), system))
        .collect();
    if systems.is_empty() {
        return None;
    }
    let systems: Vec<String> = ordered(systems, |system| system.code, order)
        .into_iter()
        .map(|(id, system)| {
            format!(
                "{id} {} in {}",
                counted(system.code, "code line"),
                counted(system.modules, "module")
            )
        })
        .collect();
    Some(format!(
        "Production from build modules: {}.",
        systems.join(", ")
    ))
}

/// `count` with a comma between groups of three digits, so large counts read
/// at a glance in the human formats; the JSON report keeps plain integers.
fn grouped(count: u64) -> String {
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

fn counted(count: u64, noun: &str) -> String {
    match count {
        1 => format!("1 {noun}"),
        _ => format!("{} {noun}s", grouped(count)),
    }
}

/// Column groups of a terminal table and the column that carries Code.
///
/// The purpose columns split Code, while Comments and Blanks sit beside it, so
/// separators and an accented Code column show which numbers belong together.
struct Layout {
    /// Columns preceded by a `│` separator. Layouts may name columns a table
    /// lacks, such as the Share column `--bars` adds, which are then ignored.
    breaks: &'static [usize],
    /// Column carrying the primary metric.
    strong: Option<usize>,
    secondary: &'static [usize],
    /// First of the Production, Test and Unclassified columns.
    purposes: Option<usize>,
}

#[derive(Clone, Copy)]
enum RowKind {
    Header,
    Body,
    Total,
}

/// Aligned plain-text table: the first column left-aligned, the others right.
/// Padding precedes styling so escape codes never shift alignment. Column names
/// and legends carry the grouping semantics without a redundant caption row.
fn text_table(
    header: &[&str],
    body: &[Vec<String>],
    total: Option<Vec<String>>,
    layout: &Layout,
    theme: Theme,
) -> String {
    text_table_with_bars(header, body, total, layout, &[], theme)
}

/// [`text_table`] with an already styled bar after each body row; bars stay
/// outside the column widths, so they never change the table's alignment.
fn text_table_with_bars(
    header: &[&str],
    body: &[Vec<String>],
    total: Option<Vec<String>>,
    layout: &Layout,
    bars: &[String],
    theme: Theme,
) -> String {
    let clean_row = |row: &[String]| row.iter().map(|value| plain(value)).collect::<Vec<_>>();
    let header: Vec<String> = header.iter().map(|title| plain(title)).collect();
    let body: Vec<_> = body.iter().map(|row| clean_row(row)).collect();
    let total = total.map(|row| clean_row(&row));
    let all = std::iter::once(&header).chain(&body).chain(total.as_ref());
    let mut widths = vec![0; header.len()];
    for row in all {
        for (width, value) in widths.iter_mut().zip(row) {
            *width = (*width).max(value.chars().count());
        }
    }
    let separated = |column: usize| layout.breaks.contains(&column);
    let gap = |column: usize| match separated(column) {
        true => " │ ",
        false => "  ",
    };
    // Hue identifies a column: cyan (Primary) for Code, one hue per purpose.
    // Rows differ by weight, so headings and totals keep their column's hue.
    let line = |row: &[String], kind: RowKind| {
        let mut text = String::from(" ");
        for (column, (value, width)) in row.iter().zip(&widths).enumerate() {
            if column > 0 {
                text += &theme.paint(Role::Muted, gap(column));
            }
            let cell = match column {
                0 => format!("{value:<width$}"),
                _ => format!("{value:>width$}"),
            };
            let emphasis = match kind {
                RowKind::Header | RowKind::Total => Some(Role::Heading),
                RowKind::Body => None,
            };
            let purpose = layout
                .purposes
                .and_then(|first| Purpose::ALL.get(column.checked_sub(first)?).copied());
            let role = match (column, value.as_str(), purpose, kind) {
                (0, _, _, _) => Some(Role::Heading),
                (_, "-" | "0" | "0.00" | "0.0%", _, _) => Some(Role::Muted),
                (_, _, Some(purpose), RowKind::Body) => Some(Role::Purpose(purpose)),
                (_, _, Some(purpose), _) => Some(Role::StrongPurpose(purpose)),
                _ if layout.strong == Some(column) => Some(Role::Primary),
                (_, _, _, RowKind::Header) => emphasis,
                _ if layout.secondary.contains(&column) => Some(Role::Secondary),
                _ => emphasis,
            };
            text += &match role {
                Some(role) => theme.paint(role, &cell),
                None => cell,
            };
        }
        text + "\n"
    };
    let mut rule = String::from(" ");
    for (column, width) in widths.iter().enumerate() {
        if column > 0 {
            rule += match separated(column) {
                true => "─┼─",
                false => "──",
            };
        }
        rule += &"─".repeat(*width);
    }
    rule += "\n";
    let rule = theme.paint(Role::Muted, &rule);
    let mut output = line(&header, RowKind::Header) + &rule;
    for (index, row) in body.iter().enumerate() {
        let mut text = line(row, RowKind::Body);
        if let Some(bar) = bars.get(index).filter(|bar| !bar.is_empty()) {
            text.pop();
            text += &format!("  {bar}\n");
        }
        output += &text;
    }
    if let Some(total) = total {
        output += &rule;
        output += &line(&total, RowKind::Total);
    }
    output
}

/// Glyphs per purpose, in `Purpose::ALL` order; distinct shapes keep the split
/// readable without color, as human presentation requires.
const BAR_GLYPHS: [char; 3] = ['█', '▓', '░'];
/// Bar cells: a readable minimum, and a maximum beyond which wider terminals
/// add length without adding legible detail.
const BAR_CELLS: std::ops::RangeInclusive<usize> = 10..=100;

/// Bars under a table comparing its rows' Code, split by purpose, after each
/// row's percentage of `total` Code. The largest row fills the width `columns`
/// leaves after label and percentage, so the bars never depend on how wide the
/// table above them is.
fn share_bars(rows: &[(String, [u64; 3])], total: u64, columns: usize, theme: Theme) -> String {
    let rows: Vec<(String, [u64; 3])> = rows
        .iter()
        .map(|(label, code)| (plain(label), *code))
        .collect();
    let label_width = rows
        .iter()
        .map(|(label, _)| label.chars().count())
        .max()
        .unwrap_or(0);
    let shares: Vec<String> = rows
        .iter()
        .map(|(_, code)| share_of(code.iter().sum(), total))
        .collect();
    let share_width = shares.iter().map(String::len).max().unwrap_or(0);
    let width = columns
        .saturating_sub(1 + label_width + 2 + 2 + share_width)
        .clamp(*BAR_CELLS.start(), *BAR_CELLS.end());
    let largest = rows
        .iter()
        .map(|(_, code)| code.iter().sum())
        .max()
        .unwrap_or(0);
    let mut output = format!(
        " {}  {}\n",
        theme.paint(Role::Primary, "Code share"),
        bar_key(theme)
    );
    for ((label, code), share) in rows.iter().zip(shares) {
        let bar = bar(code, largest, width, theme);
        let row_code: u64 = code.iter().sum();
        let share = match row_code {
            0 => theme.paint(Role::Muted, &format!("{share:>share_width$}")),
            _ => format!("{share:>share_width$}"),
        };
        output += &format!(
            " {}  {share}",
            theme.paint(Role::Heading, &format!("{label:<label_width$}"))
        );
        if !bar.is_empty() {
            output += &format!("  {bar}");
        }
        output += "\n";
    }
    output
}

/// The row's percentage of `total` Code, `-` without any code.
fn share_of(code: u64, total: u64) -> String {
    match total {
        0 => "-".to_string(),
        _ => format!("{:.1}%", code as f64 * 100.0 / total as f64),
    }
}

/// The purpose glyphs and names, each in its purpose hue.
fn bar_key(theme: Theme) -> String {
    Purpose::ALL
        .into_iter()
        .zip(BAR_GLYPHS)
        .map(|(purpose, glyph)| {
            let name = match purpose {
                Purpose::Production => "Production",
                Purpose::Test => "Test",
                Purpose::Unclassified => "Unclassified",
            };
            theme.paint(Role::Purpose(purpose), &format!("{glyph} {name}"))
        })
        .collect::<Vec<_>>()
        .join("  ")
}

/// One row's bar, `width` cells for the `largest` row; a row too small for one
/// cell gets a sliver in its largest purpose, a row without code nothing.
fn bar(code: &[u64; 3], largest: u64, width: usize, theme: Theme) -> String {
    let mut bar = String::new();
    for (index, count) in bar_cells(code, largest, width).into_iter().enumerate() {
        if count > 0 {
            let glyphs = BAR_GLYPHS[index].to_string().repeat(count);
            bar += &theme.paint(Role::Purpose(Purpose::ALL[index]), &glyphs);
        }
    }
    if bar.is_empty() && code.iter().any(|code| *code > 0) {
        let largest = (0..3)
            .max_by_key(|index| (code[*index], 3 - index))
            .unwrap();
        bar += &theme.paint(Role::Purpose(Purpose::ALL[largest]), "▏");
    }
    bar
}

/// Inline bars need this many cells; below it, the chart under the table
/// shows proportions better than a stub beside it.
const INLINE_BAR_MIN: usize = 20;

/// Bar cells beside a table `table_width` wide, after a two-space gap, when the
/// minimum inline bar fits within `columns`.
fn inline_bar_width(table_width: usize, columns: usize) -> Option<usize> {
    let room = columns.checked_sub(table_width + 2)?;
    (room >= INLINE_BAR_MIN).then(|| room.min(*BAR_CELLS.end()))
}

/// Code per purpose for each body row of a table, and where its bars go.
struct BarRows<'a> {
    codes: &'a [[u64; 3]],
    total_code: u64,
    columns: usize,
}

/// A table rendered with `--bars`: beside its rows when the minimum inline bar
/// fits, adding a Share column unless `has_share`, otherwise followed by a
/// chart. Returns the table and what follows its legends.
fn bar_table(
    header: &[&str],
    body: &[Vec<String>],
    total: Vec<String>,
    layout: &Layout,
    has_share: bool,
    bars: &BarRows,
    theme: Theme,
) -> (String, String) {
    let BarRows {
        codes,
        total_code,
        columns,
    } = *bars;
    let mut inline_header = header.to_vec();
    let mut inline_body = body.to_vec();
    let mut inline_total = total.clone();
    if !has_share {
        inline_header.push("Share");
        for (row, code) in inline_body.iter_mut().zip(codes) {
            row.push(share_of(code.iter().sum(), total_code));
        }
        inline_total.push(share_of(total_code, total_code));
    }
    // Measured on the plain rendering, so the width always matches the layout.
    let width = text_table(
        &inline_header,
        &inline_body,
        Some(inline_total.clone()),
        layout,
        Theme::PLAIN,
    )
    .lines()
    .next()
    .map_or(0, |line| line.chars().count());
    let Some(cells) = inline_bar_width(width, columns) else {
        let rows: Vec<_> = body
            .iter()
            .map(|row| row[0].clone())
            .zip(codes.iter().copied())
            .collect();
        let table = text_table(header, body, Some(total), layout, theme);
        return (
            table,
            format!("\n{}", share_bars(&rows, total_code, columns, theme)),
        );
    };
    let largest = codes
        .iter()
        .map(|code| code.iter().sum())
        .max()
        .unwrap_or(0);
    let row_bars: Vec<String> = codes
        .iter()
        .map(|code| bar(code, largest, cells, theme))
        .collect();
    let defined = match has_share {
        true => "",
        false => "Share = Code / total Code. ",
    };
    let key = format!(
        " {}{}{}\n",
        theme.paint(Role::Muted, &format!("{defined}Bars: ")),
        bar_key(theme),
        theme.paint(Role::Muted, "; the largest row fills the bar.")
    );
    let table = text_table_with_bars(
        &inline_header,
        &inline_body,
        Some(inline_total),
        layout,
        &row_bars,
        theme,
    );
    (table, key)
}

/// Whole cells per purpose: the row's length rounds its fraction of `largest`
/// across `width`, and largest remainders distribute it so the segments add up
/// exactly.
fn bar_cells(code: &[u64; 3], largest: u64, width: usize) -> [usize; 3] {
    let row: u64 = code.iter().sum();
    if largest == 0 || row == 0 {
        return [0; 3];
    }
    let (row, largest, width) = (row as u128, largest as u128, width as u128);
    let length = (row * width * 2 + largest) / (largest * 2);
    let mut cells = code.map(|code| (code as u128 * length / row) as usize);
    let mut remainders: Vec<(u128, usize)> = (0..3)
        .map(|index| (code[index] as u128 * length % row, index))
        .collect();
    remainders.sort_by(|a, b| b.0.cmp(&a.0).then(a.1.cmp(&b.1)));
    let missing = length as usize - cells.iter().sum::<usize>();
    for (_, index) in remainders.into_iter().take(missing) {
        cells[index] += 1;
    }
    cells
}

/// Label of a folder share; `.` holds the files directly in the scan root.
fn folder_label(folder: &str) -> &str {
    match folder {
        "." => "(root files)",
        _ => folder,
    }
}

/// Where the unclaimed source lives, as an aligned folder list, so a reader
/// can inspect it instead of only learning its size.
fn unclaimed_note(folders: &[FolderShare], theme: Theme) -> String {
    let plural = |count: u64, noun: &str| match count {
        1 => noun.to_string(),
        _ => format!("{noun}s"),
    };
    let shown = &folders[..folders.len().min(FOLDER_LINES)];
    let rows: Vec<[String; 5]> = shown
        .iter()
        .map(|share| {
            [
                plain(folder_label(&share.folder)),
                grouped(share.code),
                plural(share.code, "code line"),
                grouped(share.files),
                plural(share.files, "file"),
            ]
        })
        .collect();
    let width = |column: usize| {
        rows.iter()
            .map(|row| row[column].chars().count())
            .max()
            .unwrap_or(0)
    };
    let (label, code, unit, files) = (width(0), width(1), width(2), width(3));
    let mut note = format!(
        "{} {} in {}, here:",
        theme.paint(Role::Information, "Unclaimed source:"),
        counted(folders.iter().map(|share| share.code).sum(), "code line"),
        counted(folders.iter().map(|share| share.files).sum(), "file")
    );
    for [folder, lines, lines_unit, count, count_unit] in &rows {
        write!(
            note,
            "\n   {folder:<label$}  {lines:>code$} {lines_unit:<unit$}  {count:>files$} {count_unit}"
        )
        .unwrap();
    }
    if folders.len() > shown.len() {
        let rest = &folders[shown.len()..];
        write!(
            note,
            "\n   and {} more {} ({})",
            rest.len(),
            plural(rest.len() as u64, "folder"),
            counted(rest.iter().map(|share| share.code).sum(), "code line")
        )
        .unwrap();
    }
    note + "\n To inspect or refine purpose classification, use locmetry --guide configure; --files --output - shows each file's rule."
}

/// Folders listed in the terminal note; the JSON headline lists all of them.
const FOLDER_LINES: usize = 8;

/// Terminal report for people: source code per language
/// split by purpose, with unavailable counts kept visible.
/// `emphasis` enables the default terminal palette; false keeps plain text.
pub fn render_table(
    project: &str,
    summary: &ReportSummary,
    rows: &[FileRecord],
    findings: &Findings,
    categories: bool,
    order: RowOrder,
    emphasis: bool,
) -> String {
    render_table_with_theme(
        project,
        summary,
        rows,
        findings,
        categories,
        order,
        Theme::terminal(emphasis),
    )
}

/// Presentation choices for terminal measurement tables; they do not change saved evidence.
pub struct TableOptions {
    pub categories: bool,
    pub folders: bool,
    pub verbose: bool,
    pub order: RowOrder,
    pub theme: Theme,
    pub folder_rows: Option<usize>,
    /// Terminal columns for share bars below each table; `None` omits them.
    pub bars: Option<usize>,
}

/// Render a measurement table using semantic presentation roles.
pub fn render_table_with_theme(
    project: &str,
    summary: &ReportSummary,
    rows: &[FileRecord],
    findings: &Findings,
    categories: bool,
    order: RowOrder,
    theme: Theme,
) -> String {
    render_table_with_options(
        project,
        summary,
        rows,
        findings,
        TableOptions {
            categories,
            folders: false,
            verbose: false,
            order,
            theme,
            folder_rows: None,
            bars: None,
        },
    )
}

/// Source counts and code per purpose for each language.
fn source_languages(rows: &[FileRecord]) -> BTreeMap<String, (Counts, BTreeMap<Purpose, u64>)> {
    let mut languages = BTreeMap::<String, (Counts, BTreeMap<Purpose, u64>)>::new();
    for row in breakdown(rows)
        .iter()
        .filter(|row| row.kind == Kind::Source)
    {
        let (counts, purposes) = languages.entry(row.language.clone()).or_default();
        counts.add(&row.counts);
        *purposes.entry(row.purpose).or_default() += row.counts.code;
    }
    languages
}

/// Render terminal tables, optionally partitioning folders within a row budget.
pub fn render_table_with_options(
    project: &str,
    summary: &ReportSummary,
    rows: &[FileRecord],
    findings: &Findings,
    options: TableOptions,
) -> String {
    let TableOptions {
        categories,
        folders: show_folders,
        verbose,
        order,
        theme,
        folder_rows,
        bars,
    } = options;
    let languages = ordered(
        source_languages(rows).into_iter().collect(),
        |(counts, _)| counts.code,
        order,
    );
    let purpose_cells = |code: &dyn Fn(Purpose) -> u64| {
        Purpose::ALL
            .into_iter()
            .map(|purpose| grouped(code(purpose)))
            .collect::<Vec<_>>()
    };
    let body: Vec<Vec<String>> = languages
        .iter()
        .map(|(language, (counts, purposes))| {
            let mut row = vec![language.clone()];
            row.extend(
                [
                    counts.files,
                    counts.lines,
                    counts.code,
                    counts.comments,
                    counts.blanks,
                ]
                .map(grouped),
            );
            let code = |purpose| purposes.get(&purpose).copied().unwrap_or(0);
            row.extend(purpose_cells(&code));
            row.push(test_ratio(code(Purpose::Production), code(Purpose::Test)));
            row
        })
        .collect();
    let source = &summary.totals.source;
    let mut total = vec!["Total".to_string()];
    total.extend(
        [
            source.files,
            source.lines,
            source.code,
            source.comments,
            source.blanks,
        ]
        .map(grouped),
    );
    let code = |purpose| summary.totals.purpose(purpose).code;
    total.extend(purpose_cells(&code));
    total.push(test_ratio(code(Purpose::Production), code(Purpose::Test)));
    let project = display_project(project, findings.subfolder);
    let mut output = format!(
        " {}\n",
        theme.paint(Role::Heading, &format!("{project} · Source code"))
    );
    let total_code = summary.totals.source.code;
    // With --bars, a table returns what follows its legends: an inline bar key
    // or the chart below it.
    let table = |header: &[&str],
                 body: &[Vec<String>],
                 total: Vec<String>,
                 layout: &Layout,
                 has_share: bool,
                 codes: &[[u64; 3]]| match bars {
        Some(columns) => bar_table(
            header,
            body,
            total,
            layout,
            has_share,
            &BarRows {
                codes,
                total_code,
                columns,
            },
            theme,
        ),
        None => (
            text_table(header, body, Some(total), layout, theme),
            String::new(),
        ),
    };
    let codes: Vec<[u64; 3]> = languages
        .iter()
        .map(|(_, (_, purposes))| {
            Purpose::ALL.map(|purpose| purposes.get(&purpose).copied().unwrap_or(0))
        })
        .collect();
    let (text, after) = table(
        &[
            "Language",
            "Files",
            "Lines",
            "Code",
            "Comments",
            "Blanks",
            "Production",
            "Test",
            "Unclassified",
            "Test/prod",
        ],
        &body,
        total,
        &Layout {
            breaks: &[4, 6, 9, 10],
            strong: Some(3),
            secondary: &[1, 4, 5, 9, 10],
            purposes: Some(6),
        },
        false,
        &codes,
    );
    output += &text;
    for legend in [LANGUAGE_LEGEND, LANGUAGE_RATIO_NOTE] {
        output += &format!(" {}\n", theme.paint(Role::Muted, legend));
    }
    output += &after;

    // A workspace's top-level folders are its members, so repository rows,
    // which add the test ratio and share, take the folder rows' place.
    if !findings.members.is_empty() {
        let (body, total) = repository_rows(summary, findings.members, order);
        let codes: Vec<[u64; 3]> = member_totals(summary, findings.members, order)
            .into_iter()
            .map(|(_, totals)| Purpose::ALL.map(|purpose| totals.purpose(purpose).code))
            .collect();
        let (text, after) = table(
            &REPOSITORY_HEADER,
            &body,
            total,
            &Layout {
                breaks: &[3, 6],
                strong: Some(2),
                secondary: &[6, 7],
                purposes: Some(3),
            },
            true,
            &codes,
        );
        output += "\n";
        output += &text;
        output += &format!(" {}\n", theme.paint(Role::Muted, REPOSITORY_LEGEND));
        output += &after;
    }
    if findings.members.is_empty() && (show_folders || folder_rows.is_some()) {
        let adaptive =
            folder_rows.map(|budget| folders::adaptive(summary, findings.subfolder, budget, order));
        let standard = folders::standard(summary, findings.subfolder, order);
        let folders = adaptive.as_ref().map_or(&standard, |view| &view.rows);
        if !folders.is_empty() {
            let body: Vec<Vec<String>> = folders
                .iter()
                .map(|folder| {
                    let code = |purpose| folder.purposes.get(&purpose).copied().unwrap_or(0);
                    let mut row = vec![
                        folder.label(),
                        grouped(folder.counts.lines),
                        grouped(folder.counts.code),
                    ];
                    row.extend(purpose_cells(&code));
                    row.push(test_ratio(code(Purpose::Production), code(Purpose::Test)));
                    row
                })
                .collect();
            let code = |purpose| summary.totals.purpose(purpose).code;
            let mut total = vec![
                "Total".to_string(),
                grouped(summary.totals.source.lines),
                grouped(summary.totals.source.code),
            ];
            total.extend(purpose_cells(&code));
            total.push(test_ratio(code(Purpose::Production), code(Purpose::Test)));
            let codes: Vec<[u64; 3]> = folders
                .iter()
                .map(|folder| {
                    Purpose::ALL.map(|purpose| folder.purposes.get(&purpose).copied().unwrap_or(0))
                })
                .collect();
            let (text, after) = table(
                &[
                    "Folder",
                    "Lines",
                    "Code",
                    "Production",
                    "Test",
                    "Unclassified",
                    "Test/prod",
                ],
                &body,
                total,
                &Layout {
                    breaks: &[3, 6, 7],
                    strong: Some(2),
                    secondary: &[6, 7],
                    purposes: Some(3),
                },
                false,
                &codes,
            );
            output += "\n";
            output += &text;
            output += &format!(" {}\n", theme.paint(Role::Muted, FOLDER_LEGEND));
            if let Some(view) = &adaptive {
                output += &format!(
                    " {}\n",
                    theme.paint(
                        Role::Muted,
                        "Adaptive folder detail; rows cover measured source once."
                    )
                );
                if let Some(hint) = &view.budget_hint {
                    output += &format!(" {}\n", theme.paint(Role::Muted, &plain(hint)));
                }
            }
            output += &after;
        }
    }
    if categories {
        let rows = summary
            .totals
            .source_categories
            .iter()
            .map(|(category, counts)| (category.clone(), *counts))
            .collect();
        let body: Vec<Vec<String>> = ordered(rows, |counts| counts.code, order)
            .into_iter()
            .map(|(category, counts)| {
                vec![
                    category,
                    grouped(counts.files),
                    grouped(counts.lines),
                    grouped(counts.code),
                ]
            })
            .collect();
        if !body.is_empty() {
            output += "\n";
            output += &text_table(
                &["Category", "Files", "Lines", "Code"],
                &body,
                None,
                &Layout {
                    breaks: &[],
                    strong: Some(3),
                    secondary: &[1],
                    purposes: None,
                },
                theme,
            );
            output += &format!(" {}\n", theme.paint(Role::Muted, CATEGORY_LEGEND));
        }
    }

    let mut notes = Vec::new();
    if verbose {
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
            notes.push(TerminalNote::Text(format!(
                "Not source code: {}; --files --output - lists each file's kind.",
                kinds.join(", ")
            )));
        }
        notes.extend(build_modules_note(findings.modules, order).map(TerminalNote::Text));
        notes.extend(submodules_note(findings.submodules).map(TerminalNote::Text));
        let folders = unclassified_folders(rows);
        if !folders.is_empty() {
            notes.push(TerminalNote::Unclaimed(folders));
        }
        notes.extend(
            unused_entries_note(findings.unused_entries, findings.subfolder)
                .map(TerminalNote::Text),
        );
        notes.extend(
            unused_exclusions_note(findings.unused_exclusions, findings.subfolder)
                .map(TerminalNote::Text),
        );
        notes.extend(
            repositories_note(findings.members, MemberNotes::Unclaimed).map(TerminalNote::Text),
        );
    }
    notes.extend(
        repositories_note(findings.members, MemberNotes::Unavailable).map(TerminalNote::Text),
    );
    let coverage = findings.coverage;
    let mut gaps = Vec::new();
    if verbose && coverage.unrecognized_files > 0 {
        let mut extensions: Vec<_> = coverage.unrecognized_extensions.iter().collect();
        extensions.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
        let extensions: Vec<String> = extensions
            .into_iter()
            .take(5)
            .map(|(extension, files)| match extension.as_str() {
                "(none)" => format!("no extension: {}", grouped(*files as u64)),
                _ => format!("{extension}: {}", grouped(*files as u64)),
            })
            .collect();
        notes.push(TerminalNote::Unrecognized(format!(
            "{} ({}); missing lines are not zero.",
            counted(coverage.unrecognized_files as u64, "unrecognized file"),
            extensions.join(", ")
        )));
    }
    if !coverage.unread.is_empty() {
        gaps.push(counted(coverage.unread.len() as u64, "unreadable file"));
    }
    if !coverage.parse_failures.is_empty() {
        gaps.push(counted(
            coverage.parse_failures.len() as u64,
            "file with a parsing failure",
        ));
        if verbose {
            for failure in coverage.parse_failures.iter().take(25) {
                notes.push(TerminalNote::Text(format!(
                    "{}: {} — {}",
                    failure.path, failure.reason, failure.message
                )));
            }
        }
    }
    if !gaps.is_empty() {
        notes.push(TerminalNote::CoverageGap(format!(
            "{}; missing lines are not zero.",
            gaps.join(", ")
        )));
    }
    if !notes.is_empty() {
        output += "\n";
        for note in notes {
            writeln!(output, " {}", note.render(theme)).unwrap();
        }
    }
    output
}

enum TerminalNote {
    Text(String),
    Unclaimed(Vec<FolderShare>),
    CoverageGap(String),
    Unrecognized(String),
}

impl TerminalNote {
    fn render(self, theme: Theme) -> String {
        match self {
            Self::Text(text) => plain(&text),
            // This list owns its line breaks and sanitizes labels before padding.
            Self::Unclaimed(folders) => unclaimed_note(&folders, theme),
            Self::Unrecognized(text) => theme.paint(
                Role::Muted,
                &format!("Unrecognized files: {}", plain(&text)),
            ),
            Self::CoverageGap(text) => format!(
                "{} {}",
                theme.paint(Role::Warning, "Not counted:"),
                plain(&text)
            ),
        }
    }
}

fn render_coverage(output: &mut String, findings: &Findings) {
    let coverage = findings.coverage;
    writeln!(output, "\n## Coverage\n\nFiles selected: {}; counted: {}, including {} through language mappings; not recognized: {}; recognized but unreadable: {}. Their lines are missing, not zero. Counts describe the working copy, including untracked files, not a commit.", grouped(coverage.selected_files as u64), grouped(coverage.counted_files as u64), grouped(coverage.mapped_files as u64), grouped(coverage.unrecognized_files as u64), grouped(coverage.unread.len() as u64)).unwrap();
    let mut extensions: Vec<_> = coverage.unrecognized_extensions.iter().collect();
    extensions.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    if !extensions.is_empty() {
        output.push_str("\n| Unrecognized extension | Files |\n| --- | ---: |\n");
        for (extension, files) in extensions.into_iter().take(15) {
            writeln!(
                output,
                "| {} | {} |",
                cell(extension),
                grouped(*files as u64)
            )
            .unwrap();
        }
    }
    if !coverage.unread.is_empty() {
        output.push_str("\n| Unreadable file |\n| --- |\n");
        for path in coverage.unread.iter().take(25) {
            writeln!(output, "| {} |", cell(path)).unwrap();
        }
    }
    if !coverage.parse_failures.is_empty() {
        output.push_str(
            "\n| File with unavailable counts | Reason | Detail |\n| --- | --- | --- |\n",
        );
        for failure in coverage.parse_failures.iter().take(25) {
            writeln!(
                output,
                "| {} | {} | {} |",
                cell(&failure.path),
                cell(&failure.reason.to_string()),
                cell(&failure.message)
            )
            .unwrap();
        }
    }
    for note in submodules_note(findings.submodules)
        .into_iter()
        .chain(unused_entries_note(
            findings.unused_entries,
            findings.subfolder,
        ))
        .chain(unused_exclusions_note(
            findings.unused_exclusions,
            findings.subfolder,
        ))
        .chain(repositories_note(findings.members, MemberNotes::All))
    {
        writeln!(output, "\n{note}").unwrap();
    }
}

/// Points at Git submodules, often other teams' code, so a reader decides
/// their scope instead of counting them unawares.
fn submodules_note(submodules: &[SubmoduleShare]) -> Option<String> {
    if submodules.is_empty() {
        return None;
    }
    let shares: Vec<String> = submodules
        .iter()
        .map(|share| format!("{} ({})", share.path, counted(share.code, "code line")))
        .collect();
    Some(format!(
        "Git submodules: {}; exclude them with --exclude or scope.exclude if they are not this project's code.",
        shares.join(", ")
    ))
}

/// Names configuration entries that decided no file, which usually means a
/// glob that matches nothing or an entry an earlier stage preempts.
fn unused_entries_note(entries: &[String], subfolder: Option<&str>) -> Option<String> {
    (!entries.is_empty()).then(|| {
        format!(
            "Entries that decided no file{}: {}.",
            if subfolder.is_some() {
                " in this subtree"
            } else {
                ""
            },
            entries.join(", ")
        )
    })
}

/// Scoped non-use cannot establish whether project exclusions need repair.
fn unused_exclusions_note(patterns: &[String], subfolder: Option<&str>) -> Option<String> {
    (!patterns.is_empty()).then(|| {
        format!(
            "Exclusions that excluded nothing{}: {}; {}.",
            if subfolder.is_some() {
                " in this subtree"
            } else {
                ""
            },
            patterns.join(", "),
            match subfolder {
                Some(_) => "check policy use with an unrestricted measurement",
                None =>
                    "exclude a folder as <folder>/**, and an ignore file may already hide the path",
            }
        )
    })
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
    fs::write(output, text).with_context(|| format!("Cannot write report {}", output.display()))
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
mod tests {
    use super::*;

    /// Remove every SGR sequence, whatever the palette emits.
    fn unstyled(text: &str) -> String {
        let mut output = String::new();
        let mut rest = text;
        while let Some((before, after)) = rest.split_once("\x1b[") {
            output += before;
            rest = after.split_once('m').unwrap().1;
        }
        output + rest
    }

    #[test]
    fn terminal_notes_sanitize_before_applying_their_own_style() {
        for theme in [Theme::PLAIN, Theme::TERMINAL] {
            assert_eq!(
                TerminalNote::Text("path\x1b[31m\nvalue\t".into()).render(theme),
                "path [31m value "
            );
            assert_eq!(
                TerminalNote::CoverageGap("ext\x1b[31m\rvalue".into()).render(theme),
                format!(
                    "{} ext [31m value",
                    theme.paint(Role::Warning, "Not counted:")
                )
            );
        }
    }

    #[test]
    fn table_sanitizes_every_cell_before_padding_and_styling() {
        let layout = Layout {
            breaks: &[],
            strong: None,
            secondary: &[],
            purposes: None,
        };
        for theme in [Theme::PLAIN, Theme::terminal(true)] {
            let actual = text_table(
                &["Na\tme", "Co\nde"],
                &[vec!["a\x1b[31mred".into(), "1\r2".into()]],
                Some(vec!["To\ttal".into(), "3\x07".into()]),
                &layout,
                theme,
            );
            let expected = text_table(
                &["Na me", "Co de"],
                &[vec!["a [31mred".into(), "1 2".into()]],
                Some(vec!["To tal".into(), "3 ".into()]),
                &layout,
                theme,
            );
            assert_eq!(actual, expected);
            assert_eq!(actual.lines().count(), 5);
        }
    }

    #[test]
    fn legends_are_muted_in_root_and_workspace_tables_without_changing_plain_output() {
        let rows = [];
        let summary = crate::build_report(&rows);
        let coverage = Coverage::default();
        let member = [MemberView {
            name: "api".into(),
            unclaimed_files: 0,
            unclaimed_code: 0,
            unread_files: 0,
            parse_failed_files: 0,
        }];
        for members in [&[][..], &member[..]] {
            let findings = Findings {
                subfolder: None,
                coverage: &coverage,
                modules: &[],
                submodules: &[],
                unused_entries: &[],
                unused_exclusions: &[],
                members,
            };
            let styled = render_table(
                "example",
                &summary,
                &rows,
                &findings,
                false,
                RowOrder::Name,
                true,
            );
            let plain = render_table(
                "example",
                &summary,
                &rows,
                &findings,
                false,
                RowOrder::Name,
                false,
            );
            assert!(
                plain.starts_with(" example · Source code\n Language"),
                "{plain}"
            );
            assert!(!plain.contains("Code by purpose"));
            assert!(styled.starts_with(" \x1b[1mexample · Source code\x1b[0m\n"));
            assert!(styled.contains(&format!("\x1b[90m{LANGUAGE_LEGEND}\x1b[0m")));
            if !members.is_empty() {
                assert!(styled.contains(&format!("\x1b[90m{REPOSITORY_LEGEND}\x1b[0m")));
            }
            assert_eq!(unstyled(&styled), plain);
        }
    }

    #[test]
    fn grouped_inserts_a_comma_before_every_third_digit_from_the_right() {
        let cases = [
            (0, "0"),
            (999, "999"),
            (1_000, "1,000"),
            (12_345, "12,345"),
            (123_456, "123,456"),
            (1_234_567, "1,234,567"),
        ];
        for (count, expected) in cases {
            assert_eq!(grouped(count), expected);
        }
    }

    #[test]
    fn table_shades_secondary_counts_and_zeros_without_judging_purpose() {
        let table = text_table(
            &["Name", "Code", "Comments", "Blanks", "Unclassified"],
            &[vec![
                "api".into(),
                "12".into(),
                "3".into(),
                "0".into(),
                "4".into(),
            ]],
            None,
            &Layout {
                breaks: &[2, 4],
                strong: Some(1),
                secondary: &[2, 3],
                purposes: None,
            },
            Theme::TERMINAL,
        );
        assert!(table.contains("\x1b[1mapi \x1b[0m"), "{table}");
        assert!(table.contains("\x1b[2m       3\x1b[0m"), "{table}");
        assert!(table.contains("\x1b[90m     0\x1b[0m"), "{table}");
        assert!(table.contains("           4\n"), "{table}");
        assert!(!table.contains("\x1b[33m"));
    }

    /// Each purpose keeps one hue from header to total, so the header doubles as
    /// the legend; zeros stay gray so an absent purpose recedes.
    #[test]
    fn purpose_columns_carry_their_purpose_hue_from_header_to_total() {
        let table = text_table(
            &["Name", "Code", "Production", "Test", "Unclassified"],
            &[vec![
                "api".into(),
                "9".into(),
                "5".into(),
                "4".into(),
                "0".into(),
            ]],
            Some(vec![
                "Total".into(),
                "9".into(),
                "5".into(),
                "4".into(),
                "0".into(),
            ]),
            &Layout {
                breaks: &[],
                strong: Some(1),
                secondary: &[],
                purposes: Some(2),
            },
            Theme::TERMINAL,
        );
        let lines: Vec<_> = table.lines().collect();
        let (header, body, total) = (lines[0], lines[2], lines[4]);
        assert!(header.contains("\x1b[1;36mCode\x1b[0m"), "{header}");
        assert!(header.contains("\x1b[1;32mProduction\x1b[0m"), "{header}");
        assert!(header.contains("\x1b[1;35mTest\x1b[0m"), "{header}");
        assert!(header.contains("\x1b[1;34mUnclassified\x1b[0m"), "{header}");
        assert!(body.contains("\x1b[32m         5\x1b[0m"), "{body}");
        assert!(body.contains("\x1b[35m   4\x1b[0m"), "{body}");
        assert!(body.contains("\x1b[90m           0\x1b[0m"), "{body}");
        assert!(total.contains("\x1b[1;32m         5\x1b[0m"), "{total}");
        assert!(total.contains("\x1b[1;35m   4\x1b[0m"), "{total}");
    }

    /// Bars join a table only when the minimum inline bar fits beside it; the
    /// threshold follows the table's width, not a fixed terminal size.
    #[test]
    fn inline_bars_need_the_minimum_bar_beside_the_table() {
        let threshold = 90 + 2 + INLINE_BAR_MIN;
        assert_eq!(inline_bar_width(90, threshold - 1), None);
        assert_eq!(inline_bar_width(90, threshold), Some(INLINE_BAR_MIN));
        assert_eq!(
            inline_bar_width(90, threshold + 7),
            Some(INLINE_BAR_MIN + 7)
        );
        assert_eq!(inline_bar_width(90, 1000), Some(*BAR_CELLS.end()));
    }

    /// The largest row fills the bar; others scale to it, split by purpose into
    /// whole cells that add up to their length. Percentages are of total Code,
    /// and the glyphs keep purposes apart without color.
    #[test]
    fn share_bars_split_each_row_by_purpose_within_the_available_width() {
        let rows = [
            ("api".to_string(), [6, 3, 1]),
            ("web".to_string(), [0, 0, 5]),
            ("ops".to_string(), [4, 0, 0]),
            ("dev".to_string(), [0, 1, 0]),
        ];
        let plain = share_bars(&rows, 20, 33, Theme::PLAIN);
        assert_eq!(
            plain,
            " Code share  █ Production  ▓ Test  ░ Unclassified\n \
             api  50.0%  ████████████▓▓▓▓▓▓░░\n \
             web  25.0%  ░░░░░░░░░░\n \
             ops  20.0%  ████████\n \
             dev   5.0%  ▓▓\n"
        );
        let styled = share_bars(&rows, 20, 33, Theme::TERMINAL);
        assert!(
            styled.starts_with(" \x1b[1;36mCode share\x1b[0m"),
            "{styled}"
        );
        assert!(
            styled.contains("\x1b[32m████████████\x1b[0m\x1b[35m▓▓▓▓▓▓\x1b[0m\x1b[34m░░\x1b[0m")
        );
        assert_eq!(unstyled(&styled), plain);
    }

    /// A row too small for one cell still shows a sliver; empty rows show none.
    /// The bar keeps a readable minimum and stops at 100 cells, one per percent.
    #[test]
    fn share_bars_mark_slivers_and_clamp_their_width() {
        let rows = [
            ("big".to_string(), [999, 0, 0]),
            ("one".to_string(), [0, 1, 0]),
            ("nil".to_string(), [0, 0, 0]),
        ];
        let text = share_bars(&rows, 1000, 33, Theme::PLAIN);
        assert!(text.contains("\n one   0.1%  ▏\n"), "{text}");
        assert!(text.contains("\n nil   0.0%\n"), "{text}");
        let bar = |columns| {
            let text = share_bars(&rows, 1000, columns, Theme::PLAIN);
            let line = text.lines().nth(1).unwrap().to_string();
            line.chars().filter(|c| *c == '█').count()
        };
        assert_eq!(bar(10), 10);
        assert_eq!(bar(500), 100);
    }

    /// Cyan marks only the strong column, so the Total row stays bold without
    /// lifting the columns the body dims.
    #[test]
    fn total_row_is_bold_but_keeps_each_column_role() {
        let table = text_table(
            &["Name", "Code", "Comments", "Test", "Blanks"],
            &[vec![
                "api".into(),
                "12".into(),
                "3".into(),
                "4".into(),
                "0".into(),
            ]],
            Some(vec![
                "Total".into(),
                "12".into(),
                "3".into(),
                "4".into(),
                "0".into(),
            ]),
            &Layout {
                breaks: &[],
                strong: Some(1),
                secondary: &[2],
                purposes: None,
            },
            Theme::TERMINAL,
        );
        let total = table.lines().last().unwrap();
        assert!(total.contains("\x1b[1mTotal\x1b[0m"), "{total}");
        assert!(total.contains("\x1b[1;36m  12\x1b[0m"), "{total}");
        assert!(total.contains("\x1b[2m       3\x1b[0m"), "{total}");
        assert!(total.contains("\x1b[1m   4\x1b[0m"), "{total}");
        assert!(total.contains("\x1b[90m     0\x1b[0m"), "{total}");
        assert!(!table.contains("\x1b[36m"), "{table}");
    }

    /// Bold must not shift alignment: without its escape codes, an
    /// emphasized table equals the plain one.
    #[test]
    fn emphasis_bolds_the_strong_column_without_shifting_alignment() {
        let layout = Layout {
            breaks: &[2],
            strong: Some(1),
            secondary: &[],
            purposes: None,
        };
        let body = [vec!["a".to_string(), "12".into(), "3".into()]];
        let total = Some(vec!["Total".to_string(), "12".into(), "3".into()]);
        let plain = text_table(
            &["Name", "Code", "Test"],
            &body,
            total.clone(),
            &layout,
            Theme::PLAIN,
        );
        let bold = text_table(
            &["Name", "Code", "Test"],
            &body,
            total,
            &layout,
            Theme::TERMINAL,
        );
        assert!(
            bold.contains("\x1b[90m"),
            "separators should recede: {bold}"
        );
        assert!(bold.contains("\x1b[1;36mCode\x1b[0m"), "{bold}");
        assert!(bold.contains("\x1b[1;36m  12\x1b[0m"), "{bold}");
        assert_eq!(unstyled(&bold), plain);
        assert!(!plain.contains('\x1b'));
    }
}
