mod folders;

use crate::report::Summary;
use crate::style::{Role, Theme};
use crate::{
    build_systems, review, unclaimed_folders, Counts, Coverage, FileRecord, FolderShare, Kind,
    ModuleShare, Purpose, ReportSummary, SubmoduleShare,
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

/// Repository rows with each row's code, and the total row: source lines,
/// the purpose columns and ratio, and share of the workspace's source code.
fn repository_rows(
    summary: &ReportSummary,
    members: &[MemberView],
    order: RowOrder,
    columns: &PurposeColumns,
) -> (Vec<Vec<String>>, Vec<RowCode>, Vec<String>) {
    let workspace = RowCode::of_totals(&summary.totals);
    let cells = |label: &str, totals: &Summary, code: RowCode| {
        let mut row = vec![
            label.to_string(),
            grouped(totals.source.lines),
            grouped(totals.source.code),
        ];
        row.extend(columns.cells(code));
        row.push(share_of(code.code(), workspace.code()));
        row
    };
    let (body, codes) = member_totals(summary, members, order)
        .into_iter()
        .map(|(name, totals)| {
            let code = RowCode::of_totals(totals);
            (cells(&name, totals, code), code)
        })
        .unzip();
    (body, codes, cells("Total", &summary.totals, workspace))
}

fn repository_columns(purposes: &PurposeColumns) -> Columns {
    let mut columns = Columns::with_purposes(&code_columns("Repository"), purposes);
    columns
        .0
        .push(Column::new("Share", ColumnRole::Share).separated());
    columns
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

/// Tests often live in another language than their subject, such as Kotlin
/// tests for Java code; docs/plans/archive/test-production-ratios.md requires the note.
const LANGUAGE_RATIO_NOTE: &str =
    "Tests written in another language than the code they test skew per-language Test/prod.";
const CATEGORY_LEGEND: &str =
    "Rows of one purpose add up to that purpose's Code; all rows add up to the total Code.";

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
        let (body, _, total) = repository_rows(summary, findings.members, order, &ALL_PURPOSES);
        writeln!(
            output,
            "\n## Repositories\n\n{}",
            repository_columns(&ALL_PURPOSES).markdown_header()
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
            Theme::MARKDOWN.paint(
                Role::Muted,
                &ALL_PURPOSES.legend(None, Some("Share = Code / workspace Code"))
            )
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
    let non_source = summary
        .totals
        .by_kind_purpose
        .iter()
        .filter(|(key, _)| !key.starts_with("source:"))
        .map(|(key, value)| (key.clone(), *value))
        .collect();
    table(
        &mut output,
        "Non-source content",
        "Kind:purpose",
        &non_source,
        order,
    );
    let languages = Columns(
        LANGUAGE_COLUMNS
            .into_iter()
            .chain([ALL_PURPOSES.ratio_column()])
            .collect(),
    );
    writeln!(
        output,
        "\n## Source languages\n\n{}\n\n{}",
        Theme::MARKDOWN.paint(Role::Muted, LANGUAGE_RATIO_NOTE),
        languages.markdown_header()
    )
    .unwrap();
    for (language, (counts, code)) in ordered(
        source_languages(rows).into_iter().collect(),
        |(counts, _)| counts.code,
        order,
    ) {
        writeln!(
            output,
            "| {} | {} | {} |",
            cell(&language),
            count_cells(&counts).join(" | "),
            ALL_PURPOSES.ratio_cell(code)
        )
        .unwrap();
    }
    writeln!(output, "\n## Folder breakdown\n\n{}\n\n{}", Theme::MARKDOWN.paint(Role::Muted, &format!("Recursive source code totals to depth {depth}. Parent rows include their children; do not sum all displayed rows. The report file's additive `breakdown` rows cover every depth. Multi-language files count once physically but contribute to each contained language. {}", ALL_PURPOSES.legend(None, None))), Columns::with_purposes(&code_columns("Folder"), &ALL_PURPOSES).markdown_header()).unwrap();
    // Compare by path components: as plain strings, `web-admin` would sort
    // between `web` and `web/src`, because `-` sorts before `/`.
    let mut folders = human_folders(summary, findings.subfolder, depth);
    folders.sort_by(|a, b| a.0.split('/').cmp(b.0.split('/')));
    for (folder, totals) in folders {
        if totals.source.files == 0 {
            continue;
        }
        writeln!(
            output,
            "| {} | {} | {} | {} |",
            cell(&folder),
            grouped(totals.source.lines),
            grouped(totals.source.code),
            ALL_PURPOSES.cells(RowCode::of_totals(totals)).join(" | ")
        )
        .unwrap();
    }
    render_build_modules(&mut output, findings.modules, order);
    let unknown = review(rows);
    writeln!(
        output,
        "\n## Source requiring review\n\n{} of unclaimed source: no rule, convention, or build module claims it. Code an entry or convention assigns to Other on purpose is not listed.",
        counted(unknown.len() as u64, "file")
    )
    .unwrap();
    let folders = unclaimed_folders(rows);
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
                        .filter(|entry| crate::report::share_holds(&share.folder, &entry.path))
                        .map(|entry| entry.lines)
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
            grouped(row.lines),
            grouped(row.code),
            cell(&row.category)
        )
        .unwrap();
    }
    render_coverage(&mut output, findings);
    output
}

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

/// `noun`, pluralized with `s` unless `count` is one.
fn plural(count: u64, noun: &str) -> String {
    match count {
        1 => noun.to_string(),
        _ => format!("{noun}s"),
    }
}

/// `count` followed by `noun`, pluralized with `s` unless it is one.
pub(crate) fn counted(count: u64, noun: &str) -> String {
    format!("{} {}", grouped(count), plural(count, noun))
}

/// Column groups of a terminal table and the column that carries Code.
///
/// The purpose columns split Code, while Comments and Blanks sit beside it, so
/// separators and an accented Code column show which numbers belong together.
#[derive(Default)]
struct Layout {
    /// Columns preceded by a `│` separator.
    breaks: Vec<usize>,
    /// Column carrying the primary metric.
    strong: Option<usize>,
    secondary: Vec<usize>,
    /// Signed changes, colored by direction.
    changes: Vec<usize>,
    /// The purpose each purpose column splits Code by.
    purposes: Vec<(usize, Purpose)>,
}

/// What a measurement table column holds, which decides its styling in text
/// tables and its emphasis and alignment in Markdown.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum ColumnRole {
    /// Row labels, left-aligned.
    Label,
    Count,
    /// A count beside the main figures, such as Files, Comments or Blanks.
    Secondary,
    /// Code, the primary metric the purpose columns split.
    Code,
    Purpose(Purpose),
    Ratio(Ratio),
    /// Share of the total Code.
    Share,
    /// A signed net change between two reports.
    Change,
}

/// One column of a measurement table.
#[derive(Clone, Copy)]
pub(crate) struct Column {
    title: &'static str,
    /// Heading for terminal tables when `title` would be wider than every
    /// value, so a long title does not widen a column of short figures.
    short: Option<&'static str>,
    role: ColumnRole,
    /// Preceded by a `│` separator, starting a group of related figures.
    separated: bool,
}

impl Column {
    pub(crate) const fn new(title: &'static str, role: ColumnRole) -> Self {
        Self {
            title,
            short: None,
            role,
            separated: false,
        }
    }

    pub(crate) const fn abbreviated(self, short: &'static str) -> Self {
        Self {
            short: Some(short),
            ..self
        }
    }

    pub(crate) const fn separated(self) -> Self {
        Self {
            separated: true,
            ..self
        }
    }
}

/// The language table's columns before its purpose columns.
const LANGUAGE_COLUMNS: [Column; 6] = [
    Column::new("Language", ColumnRole::Label),
    Column::new("Files", ColumnRole::Secondary),
    Column::new("Lines", ColumnRole::Count),
    Column::new("Code", ColumnRole::Code),
    Column::new("Comments", ColumnRole::Secondary).separated(),
    Column::new("Blanks", ColumnRole::Secondary),
];

/// The columns of a folder or repository table before its purpose columns.
const fn code_columns(label: &'static str) -> [Column; 3] {
    [
        Column::new(label, ColumnRole::Label),
        Column::new("Lines", ColumnRole::Count),
        Column::new("Code", ColumnRole::Code),
    ]
}

/// A table's columns in order; its header, text layout and Markdown header
/// all follow from them, so they cannot disagree.
#[derive(Clone)]
pub(crate) struct Columns(Vec<Column>);

impl Columns {
    pub(crate) fn of(columns: &[Column]) -> Self {
        Self(columns.to_vec())
    }

    /// `lead` followed by the purpose columns and ratio `purposes` shows.
    fn with_purposes(lead: &[Column], purposes: &PurposeColumns) -> Self {
        Self(
            lead.iter()
                .copied()
                .chain(purposes.table_columns())
                .collect(),
        )
    }

    /// Terminal headings: each column's short title where its full title is
    /// wider than all of its values in `body` and `total`.
    fn fitted_titles(&self, body: &[Vec<String>], total: Option<&[String]>) -> Vec<&'static str> {
        self.0
            .iter()
            .enumerate()
            .map(|(index, column)| {
                let widest = body
                    .iter()
                    .map(Vec::as_slice)
                    .chain(total)
                    .filter_map(|row| row.get(index))
                    .map(|value| plain(value).chars().count())
                    .max()
                    .unwrap_or(0);
                match column.short {
                    Some(short) if column.title.chars().count() > widest => short,
                    _ => column.title,
                }
            })
            .collect()
    }

    fn has(&self, role: ColumnRole) -> bool {
        self.0.iter().any(|column| column.role == role)
    }

    fn layout(&self) -> Layout {
        let mut layout = Layout::default();
        for (index, column) in self.0.iter().enumerate() {
            if column.separated {
                layout.breaks.push(index);
            }
            match column.role {
                ColumnRole::Code => layout.strong = Some(index),
                ColumnRole::Secondary | ColumnRole::Ratio(_) | ColumnRole::Share => {
                    layout.secondary.push(index)
                }
                ColumnRole::Purpose(purpose) => layout.purposes.push((index, purpose)),
                ColumnRole::Change => layout.changes.push(index),
                ColumnRole::Label | ColumnRole::Count => {}
            }
        }
        layout
    }

    /// Header and alignment rows; bold Code marks the column the purpose
    /// columns split, as the terminal's accent does.
    fn markdown_header(&self) -> String {
        let titles: Vec<String> = self
            .0
            .iter()
            .map(|column| match column.role {
                ColumnRole::Code => format!("**{}**", column.title),
                _ => column.title.to_string(),
            })
            .collect();
        let alignments: Vec<&str> = self
            .0
            .iter()
            .map(|column| match column.role {
                ColumnRole::Label => "---",
                _ => "---:",
            })
            .collect();
        format!("| {} |\n| {} |", titles.join(" | "), alignments.join(" | "))
    }
}

/// Separator between `column` and the column before it.
fn gap(layout: &Layout, column: usize) -> &'static str {
    match layout.breaks.contains(&column) {
        true => " │ ",
        false => "  ",
    }
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
    PlainTable::new(header, body, total.as_deref()).render(layout, None, theme)
}

/// `columns` over `body` and an optional bold `total` row, as an aligned text
/// table or a Markdown table; Markdown cells must already be escaped. Compare
/// renders its grids through it, so both commands share one table style.
pub(crate) fn shared_table(
    columns: &Columns,
    body: &[Vec<String>],
    total: Option<&[String]>,
    markdown: bool,
    theme: Theme,
) -> String {
    if !markdown {
        return PlainTable::new(&columns.fitted_titles(body, total), body, total).render(
            &columns.layout(),
            None,
            theme,
        );
    }
    let mut output = columns.markdown_header() + "\n";
    for row in body {
        writeln!(output, "| {} |", row.join(" | ")).unwrap();
    }
    if let Some(total) = total {
        let cells: Vec<_> = total
            .iter()
            .map(|value| theme.paint(Role::Heading, value))
            .collect();
        writeln!(output, "| {} |", cells.join(" | ")).unwrap();
    }
    output
}

/// Already styled bars after each body row, under their kind's titled column;
/// bars stay outside the column widths, so they never change the alignment.
struct InlineBars {
    kind: BarKind,
    cells: usize,
    bars: Vec<String>,
}

/// A table's cells without control characters, and each column's width.
struct PlainTable {
    header: Vec<String>,
    body: Vec<Vec<String>>,
    total: Option<Vec<String>>,
    widths: Vec<usize>,
}

impl PlainTable {
    fn new(header: &[&str], body: &[Vec<String>], total: Option<&[String]>) -> Self {
        let clean_row = |row: &[String]| row.iter().map(|value| plain(value)).collect::<Vec<_>>();
        let header: Vec<String> = header.iter().map(|title| plain(title)).collect();
        let body: Vec<_> = body.iter().map(|row| clean_row(row)).collect();
        let total = total.map(clean_row);
        let all = std::iter::once(&header).chain(&body).chain(total.as_ref());
        let mut widths = vec![0; header.len()];
        for row in all {
            for (width, value) in widths.iter_mut().zip(row) {
                *width = (*width).max(value.chars().count());
            }
        }
        Self {
            header,
            body,
            total,
            widths,
        }
    }

    /// Characters per line without bars, as [`PlainTable::render`] lays it out.
    fn width(&self, layout: &Layout) -> usize {
        let gaps: usize = (1..self.widths.len())
            .map(|column| gap(layout, column).chars().count())
            .sum();
        1 + self.widths.iter().sum::<usize>() + gaps
    }

    fn render(&self, layout: &Layout, inline: Option<&InlineBars>, theme: Theme) -> String {
        // Hue identifies a column: cyan (Primary) for Code, one hue per purpose.
        // Rows differ by weight, so headings and totals keep their column's hue.
        let line = |row: &[String], kind: RowKind, trailer: Option<&str>| {
            let mut text = String::from(" ");
            for (column, (value, width)) in row.iter().zip(&self.widths).enumerate() {
                if column > 0 {
                    text += &theme.paint(Role::Muted, gap(layout, column));
                }
                let cell = match column {
                    0 => format!("{value:<width$}"),
                    _ => format!("{value:>width$}"),
                };
                let purpose = layout
                    .purposes
                    .iter()
                    .find(|(index, _)| *index == column)
                    .map(|(_, purpose)| *purpose);
                let role = match (column, value.as_str(), purpose, kind) {
                    (0, _, _, _) => Some(Role::Heading),
                    (_, "-" | "0" | "0.00" | "0.0%", _, _) => Some(Role::Muted),
                    (_, _, Some(purpose), RowKind::Body) => Some(Role::Purpose(purpose)),
                    (_, _, Some(purpose), _) => Some(Role::StrongPurpose(purpose)),
                    _ if layout.strong == Some(column) => Some(Role::Primary),
                    (_, _, _, RowKind::Header) => Some(Role::Heading),
                    _ if layout.changes.contains(&column) => Some(match value.starts_with('-') {
                        true => Role::Decrease,
                        false => Role::Increase,
                    }),
                    _ if layout.secondary.contains(&column) => Some(Role::Secondary),
                    (_, _, _, RowKind::Total) => Some(Role::Heading),
                    _ => None,
                };
                text += &match role {
                    Some(role) => theme.paint(role, &cell),
                    None => cell,
                };
            }
            if let Some(trailer) = trailer {
                text += "  ";
                text += trailer;
            }
            text + "\n"
        };
        let mut rule = String::from(" ");
        for (column, width) in self.widths.iter().enumerate() {
            if column > 0 {
                rule += match layout.breaks.contains(&column) {
                    true => "─┼─",
                    false => "──",
                };
            }
            rule += &"─".repeat(*width);
        }
        let title = inline.map(|bars| theme.paint(bars.kind.heading_role(), bars.kind.heading()));
        if let Some(bars) = inline {
            rule += &format!("──{}", "─".repeat(bars.cells));
        }
        rule += "\n";
        let rule = theme.paint(Role::Muted, &rule);
        let mut output = line(&self.header, RowKind::Header, title.as_deref()) + &rule;
        for (index, row) in self.body.iter().enumerate() {
            let bar = inline
                .and_then(|bars| bars.bars.get(index))
                .filter(|bar| !bar.is_empty());
            output += &line(row, RowKind::Body, bar.map(String::as_str));
        }
        if let Some(total) = &self.total {
            output += &rule;
            output += &line(total, RowKind::Total, None);
        }
        output
    }
}

/// Glyphs per purpose bar segment, and the comment bar's; distinct shapes keep
/// segments readable without color, as human presentation requires.
const BAR_GLYPHS: [char; 3] = ['█', '▓', '░'];
const COMMENT_GLYPH: char = '▓';
/// Bar cells: a readable minimum, and a maximum beyond which wider terminals
/// add length without adding legible detail.
const BAR_CELLS: std::ops::RangeInclusive<usize> = 10..=40;

/// Code per purpose and comments of one table row: what its purpose columns,
/// ratio and bars describe.
#[derive(Clone, Copy, Default)]
struct RowCode {
    purposes: [u64; 3],
    comments: u64,
}

impl RowCode {
    fn of(purposes: &BTreeMap<Purpose, u64>, comments: u64) -> Self {
        Self {
            purposes: Purpose::ALL.map(|purpose| purposes.get(&purpose).copied().unwrap_or(0)),
            comments,
        }
    }

    fn of_totals(totals: &Summary) -> Self {
        Self {
            purposes: Purpose::ALL.map(|purpose| totals.purpose(purpose).code),
            comments: totals.source.comments,
        }
    }

    fn code(&self) -> u64 {
        self.purposes.iter().sum()
    }

    fn purpose(&self, purpose: Purpose) -> u64 {
        self.purposes[purpose.index()]
    }

    /// Comments per line of code or comment, 0 without either.
    fn density(&self) -> f64 {
        match self.code() + self.comments {
            0 => 0.0,
            lines => self.comments as f64 / lines as f64,
        }
    }
}

/// Comments per line of code or comment, as SonarQube's comment density;
/// `-` without either.
fn comment_share(code: u64, comments: u64) -> String {
    share_of(comments, code + comments)
}

const COMMENT_SHARE_TITLE: &str = "Comment %";
const COMMENT_SHARE_DEFINITION: &str = "Comment % = Comments / (Code + Comments)";

pub(crate) fn purpose_title(purpose: Purpose) -> &'static str {
    match purpose {
        Purpose::Production => "Production",
        Purpose::Test => "Test",
        Purpose::Other => "Other",
    }
}

/// The ratio after a table's purpose columns.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Ratio {
    /// Test code per production line, when both purposes are counted.
    TestPerProduction,
    /// Comment density, which replaces a ratio that lacks one of its purposes.
    CommentShare,
}

/// The purposes a table counts, which decide its purpose columns and ratio.
struct PurposeColumns<'a> {
    shown: &'a [Purpose],
}

/// Every purpose: the unfiltered view, and Markdown's.
const ALL_PURPOSES: PurposeColumns<'static> = PurposeColumns {
    shown: &Purpose::ALL,
};

impl PurposeColumns<'_> {
    /// One purpose would only repeat Code, so it gets no column.
    fn purposes(&self) -> &[Purpose] {
        match self.shown.len() {
            0 | 1 => &[],
            _ => self.shown,
        }
    }

    fn ratio(&self) -> Ratio {
        match self.shown.contains(&Purpose::Production) && self.shown.contains(&Purpose::Test) {
            true => Ratio::TestPerProduction,
            false => Ratio::CommentShare,
        }
    }

    /// The purpose columns, as one group, and the ratio after them.
    fn table_columns(&self) -> Vec<Column> {
        let purposes = self.purposes().iter().enumerate().map(|(index, purpose)| {
            let column = Column::new(purpose_title(*purpose), ColumnRole::Purpose(*purpose));
            let column = match purpose {
                Purpose::Production => column.abbreviated("Prod."),
                Purpose::Test | Purpose::Other => column,
            };
            match index {
                0 => column.separated(),
                _ => column,
            }
        });
        purposes.chain([self.ratio_column()]).collect()
    }

    fn ratio_column(&self) -> Column {
        let ratio = self.ratio();
        let title = match ratio {
            Ratio::TestPerProduction => "Test/prod",
            Ratio::CommentShare => COMMENT_SHARE_TITLE,
        };
        Column::new(title, ColumnRole::Ratio(ratio)).separated()
    }

    /// The cells of [`PurposeColumns::table_columns`] for `row`.
    fn cells(&self, row: RowCode) -> Vec<String> {
        let mut cells: Vec<_> = self
            .purposes()
            .iter()
            .map(|purpose| grouped(row.purpose(*purpose)))
            .collect();
        cells.push(self.ratio_cell(row));
        cells
    }

    fn ratio_cell(&self, row: RowCode) -> String {
        match self.ratio() {
            Ratio::TestPerProduction => {
                test_ratio(row.purpose(Purpose::Production), row.purpose(Purpose::Test))
            }
            Ratio::CommentShare => comment_share(row.code(), row.comments),
        }
    }

    /// Purpose bars need purposes to split; with one, bars chart comments,
    /// as the purposes' columns give way to Code alone.
    fn bar_kind(&self, requested: BarKind) -> BarKind {
        match (requested, self.purposes().is_empty()) {
            (BarKind::Purpose, true) => BarKind::Comments,
            (kind, _) => kind,
        }
    }

    /// How the columns add up: `first` and `last` frame the purpose sum and
    /// the ratio's definition.
    fn legend(&self, first: Option<&str>, last: Option<&str>) -> String {
        let sum = (self.purposes().len() > 1).then(|| {
            let titles: Vec<_> = self.purposes().iter().map(|p| purpose_title(*p)).collect();
            format!("{} = Code", titles.join(" + "))
        });
        let ratio = match self.ratio() {
            Ratio::TestPerProduction => "Test/prod = Test / Production",
            Ratio::CommentShare => COMMENT_SHARE_DEFINITION,
        };
        let parts: Vec<String> = [
            first.map(String::from),
            sum,
            Some(ratio.into()),
            last.map(String::from),
        ]
        .into_iter()
        .flatten()
        .collect();
        format!("{}.", parts.join("; "))
    }

    /// Names the selection, so filtered totals are not read as complete ones.
    /// Column titles name the purposes, so "Other" cannot read as "others".
    fn filter_note(&self) -> Option<String> {
        let names = |purposes: Vec<Purpose>| {
            purposes
                .iter()
                .map(|purpose| purpose_title(*purpose))
                .collect::<Vec<_>>()
                .join(" and ")
        };
        let excluded: Vec<Purpose> = Purpose::ALL
            .into_iter()
            .filter(|purpose| !self.shown.contains(purpose))
            .collect();
        (!excluded.is_empty()).then(|| {
            format!(
                "Purposes shown: {}; {} excluded.",
                names(self.shown.to_vec()),
                names(excluded)
            )
        })
    }
}

/// What share bars show.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BarKind {
    /// Each row's Code split by purpose, scaled to the largest row.
    Purpose,
    /// Each row's comment density, scaled to the densest row.
    Comments,
}

/// What fills a bar: the largest row's Code, or the highest comment density.
#[derive(Clone, Copy)]
struct BarScale {
    code: u64,
    density: f64,
}

impl BarScale {
    fn of(rows: &[RowCode]) -> Self {
        Self {
            code: rows.iter().map(RowCode::code).max().unwrap_or(0),
            density: rows.iter().map(RowCode::density).fold(0.0, f64::max),
        }
    }
}

impl BarKind {
    /// The percentage column beside inline bars, set apart like a ratio.
    fn column(self) -> Column {
        let column = match self {
            BarKind::Purpose => Column::new("Share", ColumnRole::Share),
            BarKind::Comments => {
                Column::new(COMMENT_SHARE_TITLE, ColumnRole::Ratio(Ratio::CommentShare))
            }
        };
        column.separated()
    }

    fn definition(self) -> &'static str {
        match self {
            BarKind::Purpose => "Share = Code / total Code",
            BarKind::Comments => COMMENT_SHARE_DEFINITION,
        }
    }

    fn scale_note(self) -> &'static str {
        match self {
            BarKind::Purpose => "; the largest row fills the bar.",
            BarKind::Comments => "; the densest row fills the bar.",
        }
    }

    /// Title of the bar column and of the chart below a table; it names what
    /// the bars mean, since a single purpose turns purpose bars to comments.
    fn heading(self) -> &'static str {
        match self {
            BarKind::Purpose => "Code share",
            BarKind::Comments => "Comment density",
        }
    }

    /// Code share takes the Code hue; comment density stays plain.
    fn heading_role(self) -> Role {
        match self {
            BarKind::Purpose => Role::Primary,
            BarKind::Comments => Role::Heading,
        }
    }

    fn percent(self, row: RowCode, total: RowCode) -> String {
        match self {
            BarKind::Purpose => share_of(row.code(), total.code()),
            BarKind::Comments => comment_share(row.code(), row.comments),
        }
    }

    /// Glyph, hue and name of each segment the bars can show; plain hues,
    /// never bold, keep bars quieter than the numbers.
    fn segments(self, shown: &[Purpose]) -> Vec<(char, Role, &'static str)> {
        match self {
            BarKind::Purpose => shown
                .iter()
                .map(|purpose| {
                    let glyph = BAR_GLYPHS[purpose.index()];
                    (glyph, Role::Purpose(*purpose), purpose_title(*purpose))
                })
                .collect(),
            BarKind::Comments => vec![(COMMENT_GLYPH, Role::Secondary, COMMENT_SHARE_TITLE)],
        }
    }

    fn key(self, shown: &[Purpose], theme: Theme) -> String {
        self.segments(shown)
            .into_iter()
            .map(|(glyph, role, name)| theme.paint(role, &format!("{glyph} {name}")))
            .collect::<Vec<_>>()
            .join("  ")
    }
}

/// Bars under a table: each row's percentage, then its bar in the width
/// `columns` leaves after label and percentage, so the bars never depend on
/// how wide the table above them is.
fn share_bars(
    rows: &[(String, RowCode)],
    total: RowCode,
    kind: BarKind,
    columns: usize,
    shown: &[Purpose],
    theme: Theme,
) -> String {
    let rows: Vec<(String, RowCode)> = rows
        .iter()
        .map(|(label, row)| (plain(label), *row))
        .collect();
    let label_width = rows
        .iter()
        .map(|(label, _)| label.chars().count())
        .max()
        .unwrap_or(0);
    let percents: Vec<String> = rows
        .iter()
        .map(|(_, row)| kind.percent(*row, total))
        .collect();
    let percent_width = percents.iter().map(String::len).max().unwrap_or(0);
    let width = columns
        .saturating_sub(1 + label_width + 2 + 2 + percent_width)
        .clamp(*BAR_CELLS.start(), *BAR_CELLS.end());
    let codes: Vec<RowCode> = rows.iter().map(|(_, row)| *row).collect();
    let scale = BarScale::of(&codes);
    let mut output = format!(
        " {}  {}\n",
        theme.paint(kind.heading_role(), kind.heading()),
        kind.key(shown, theme)
    );
    for ((label, row), percent) in rows.iter().zip(percents) {
        let bar = bar(kind, *row, scale, width, theme);
        let percent = match row.code() {
            0 => theme.paint(Role::Muted, &format!("{percent:>percent_width$}")),
            _ => format!("{percent:>percent_width$}"),
        };
        output += &format!(
            " {}  {percent}",
            theme.paint(Role::Heading, &format!("{label:<label_width$}"))
        );
        if !bar.is_empty() {
            output += &format!("  {bar}");
        }
        output += "\n";
    }
    output
}

/// The row's percentage of `total`, `-` without a total.
fn share_of(part: u64, total: u64) -> String {
    match total {
        0 => "-".to_string(),
        _ => format!("{:.1}%", part as f64 * 100.0 / total as f64),
    }
}

/// One row's bar in `width` cells at `scale`; a row too small for one cell
/// gets a sliver in its largest segment's hue, a row without any count nothing.
fn bar(kind: BarKind, row: RowCode, scale: BarScale, width: usize, theme: Theme) -> String {
    let (amounts, cells) = match kind {
        BarKind::Purpose => (
            row.purposes.to_vec(),
            bar_cells(&row.purposes, scale.code, width).to_vec(),
        ),
        BarKind::Comments => {
            let length = match scale.density {
                0.0 => 0,
                densest => (row.density() / densest * width as f64).round() as usize,
            };
            (vec![row.comments], vec![length])
        }
    };
    let segments: Vec<_> = kind
        .segments(&Purpose::ALL)
        .into_iter()
        .zip(amounts)
        .zip(cells)
        .map(|(((glyph, role, _), amount), cells)| (glyph, role, amount, cells))
        .collect();
    let bar: String = segments
        .iter()
        .filter(|(.., cells)| *cells > 0)
        .map(|(glyph, role, _, cells)| theme.paint(*role, &glyph.to_string().repeat(*cells)))
        .collect();
    // The first of equally large segments, as `max_by_key` keeps the last.
    let largest = segments
        .iter()
        .rev()
        .filter(|(_, _, amount, _)| *amount > 0)
        .max_by_key(|(_, _, amount, _)| *amount);
    match (bar.is_empty(), largest) {
        (true, Some((_, role, ..))) => theme.paint(*role, "▏"),
        _ => bar,
    }
}

/// Bar cells beside a table `table_width` wide, after a two-space gap, when the
/// minimum bar fits within `columns`: the chart's minimum, widened to the bar
/// column's `title` so the header line never outgrows the rows.
fn inline_bar_width(table_width: usize, title: &str, columns: usize) -> Option<usize> {
    let room = columns.checked_sub(table_width + 2)?;
    let minimum = (*BAR_CELLS.start()).max(title.chars().count());
    (room >= minimum).then(|| room.min(*BAR_CELLS.end()))
}

/// Each body row's code and the table total, the purposes counted, and where
/// the bars go.
struct BarRows<'a> {
    rows: &'a [RowCode],
    total: RowCode,
    columns: &'a PurposeColumns<'a>,
    bars: Bars,
}

/// A table with bars: beside its rows when the minimum inline bar fits,
/// adding the bar kind's percentage column unless the table has it; otherwise
/// followed by a chart for [`Bars::Always`] and left as it is for
/// [`Bars::Fit`]. Returns the table and what follows its legends.
fn bar_table(
    table_columns: &Columns,
    body: &[Vec<String>],
    total: Vec<String>,
    bars: &BarRows,
    theme: Theme,
) -> (String, String) {
    let BarRows {
        rows,
        total: total_row,
        columns: purposes,
        bars,
    } = *bars;
    let plain_table = |theme| {
        text_table(
            &table_columns.fitted_titles(body, Some(&total)),
            body,
            Some(total.clone()),
            &table_columns.layout(),
            theme,
        )
    };
    let (columns, chart, kind) = match bars {
        Bars::Off => return (plain_table(theme), String::new()),
        Bars::Fit(columns) => (columns, false, BarKind::Purpose),
        Bars::Always(columns, kind) => (columns, true, kind),
    };
    let kind = purposes.bar_kind(kind);
    let shown = purposes.shown;
    let has_percent = table_columns.has(kind.column().role);
    let mut inline_columns = table_columns.clone();
    let mut inline_body = body.to_vec();
    let mut inline_total = total.clone();
    if !has_percent {
        inline_columns.0.push(kind.column());
        for (cells, row) in inline_body.iter_mut().zip(rows) {
            cells.push(kind.percent(*row, total_row));
        }
        inline_total.push(kind.percent(total_row, total_row));
    }
    let inline_layout = inline_columns.layout();
    let table = PlainTable::new(
        &inline_columns.fitted_titles(&inline_body, Some(&inline_total)),
        &inline_body,
        Some(&inline_total),
    );
    let Some(cells) = inline_bar_width(table.width(&inline_layout), kind.heading(), columns) else {
        let table = plain_table(theme);
        if !chart {
            return (table, String::new());
        }
        let labelled: Vec<_> = body
            .iter()
            .map(|cells| cells[0].clone())
            .zip(rows.iter().copied())
            .collect();
        let chart = share_bars(&labelled, total_row, kind, columns, shown, theme);
        return (table, format!("\n{chart}"));
    };
    let scale = BarScale::of(rows);
    let bars = InlineBars {
        kind,
        cells,
        bars: rows
            .iter()
            .map(|row| bar(kind, *row, scale, cells, theme))
            .collect(),
    };
    let defined = match has_percent {
        true => String::new(),
        false => format!("{}. ", kind.definition()),
    };
    let key = format!(
        " {}{}{}\n",
        theme.paint(Role::Muted, &format!("{defined}Bars: ")),
        kind.key(shown, theme),
        theme.paint(Role::Muted, kind.scale_note())
    );
    (table.render(&inline_layout, Some(&bars), theme), key)
}

/// Whole cells per segment: the row's length rounds its fraction of `scale`
/// across `width`, and largest remainders distribute it so the segments add up
/// exactly.
fn bar_cells(segments: &[u64; 3], scale: u64, width: usize) -> [usize; 3] {
    let row: u64 = segments.iter().sum();
    if scale == 0 || row == 0 {
        return [0; 3];
    }
    let (row, scale, width) = (row as u128, scale as u128, width as u128);
    let length = (row * width * 2 + scale) / (scale * 2);
    let mut cells = segments.map(|count| (count as u128 * length / row) as usize);
    let mut remainders: Vec<(u128, usize)> = (0..3)
        .map(|index| (segments[index] as u128 * length % row, index))
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

/// Presentation choices for terminal measurement tables; they do not change saved evidence.
pub struct TableOptions {
    pub categories: bool,
    pub folders: bool,
    pub verbose: bool,
    pub order: RowOrder,
    pub theme: Theme,
    pub folder_rows: Option<usize>,
    pub bars: Bars,
    /// Purposes whose source the tables count, in `Purpose::ALL` order.
    pub purposes: Vec<Purpose>,
}

/// Bars for terminal tables, with the columns available to them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bars {
    Off,
    /// Purpose bars beside each table that leaves room for them; omitted elsewhere.
    Fit(usize),
    /// Bars beside each table that leaves room for them; a chart below elsewhere.
    Always(usize, BarKind),
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

/// Render terminal tables, optionally partitioning folders within a row budget.
pub fn render_table_with_options(
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
        purposes,
    } = options;
    // A purpose selection counts only that source, so every table, total and
    // bar is rebuilt from the selected records.
    let selected: Vec<FileRecord>;
    let selected_summary;
    let (summary, rows) = match purposes.len() < Purpose::ALL.len() {
        true => {
            selected = rows
                .iter()
                .filter(|row| {
                    row.kind != Kind::Source || purposes.contains(&row.classification.purpose)
                })
                .cloned()
                .collect();
            selected_summary = crate::build_report(&selected);
            (&selected_summary, &selected[..])
        }
        false => (summary, rows),
    };
    let columns = PurposeColumns { shown: &purposes };
    let languages = ordered(
        source_languages(rows).into_iter().collect(),
        |(counts, _)| counts.code,
        order,
    );
    let codes: Vec<RowCode> = languages.iter().map(|(_, (_, code))| *code).collect();
    let body: Vec<Vec<String>> = languages
        .iter()
        .zip(&codes)
        .map(|((language, (counts, _)), code)| {
            let mut row = vec![language.clone()];
            row.extend(count_cells(counts));
            row.extend(columns.cells(*code));
            row
        })
        .collect();
    let total_code = RowCode::of_totals(&summary.totals);
    let mut total = vec!["Total".to_string()];
    total.extend(count_cells(&summary.totals.source));
    total.extend(columns.cells(total_code));
    // No title: the footer names the measured root, which a title repeated.
    let mut output = String::new();
    // With bars, a table returns what follows its legends: an inline bar key
    // or the chart below it.
    let table =
        |table_columns: &Columns, body: &[Vec<String>], total: Vec<String>, rows: &[RowCode]| {
            let rows = BarRows {
                rows,
                total: total_code,
                columns: &columns,
                bars,
            };
            bar_table(table_columns, body, total, &rows, theme)
        };
    let (text, after) = table(
        &Columns::with_purposes(&LANGUAGE_COLUMNS, &columns),
        &body,
        total,
        &codes,
    );
    output += &text;
    let mut legends = vec![columns.legend(Some("Lines = Code + Comments + Blanks"), None)];
    if columns.ratio() == Ratio::TestPerProduction {
        legends.push(LANGUAGE_RATIO_NOTE.into());
    }
    legends.extend(columns.filter_note());
    for legend in legends {
        output += &format!(" {}\n", theme.paint(Role::Muted, &legend));
    }
    output += &after;

    // A workspace's top-level folders are its members, so repository rows,
    // which add the test ratio and share, take the folder rows' place.
    if !findings.members.is_empty() {
        let (body, codes, total) = repository_rows(summary, findings.members, order, &columns);
        let (text, after) = table(&repository_columns(&columns), &body, total, &codes);
        output += "\n";
        output += &text;
        let legend = columns.legend(None, Some("Share = Code / workspace Code"));
        output += &format!(" {}\n", theme.paint(Role::Muted, &legend));
        output += &after;
    }
    if findings.members.is_empty() && (show_folders || folder_rows.is_some()) {
        let adaptive =
            folder_rows.map(|budget| folders::adaptive(summary, findings.subfolder, budget, order));
        let standard = folders::standard(summary, findings.subfolder, order);
        let folders = adaptive.as_ref().map_or(&standard, |view| &view.rows);
        if !folders.is_empty() {
            let codes: Vec<RowCode> = folders
                .iter()
                .map(|folder| RowCode::of(&folder.purposes, folder.counts.comments))
                .collect();
            let body: Vec<Vec<String>> = folders
                .iter()
                .zip(&codes)
                .map(|(folder, code)| {
                    let mut row = vec![
                        folder.label(),
                        grouped(folder.counts.lines),
                        grouped(folder.counts.code),
                    ];
                    row.extend(columns.cells(*code));
                    row
                })
                .collect();
            let mut total = vec![
                "Total".to_string(),
                grouped(summary.totals.source.lines),
                grouped(summary.totals.source.code),
            ];
            total.extend(columns.cells(total_code));
            let (text, after) = table(
                &Columns::with_purposes(&code_columns("Folder"), &columns),
                &body,
                total,
                &codes,
            );
            output += "\n";
            output += &text;
            output += &format!(
                " {}\n",
                theme.paint(Role::Muted, &columns.legend(None, None))
            );
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
                    breaks: vec![],
                    strong: Some(3),
                    secondary: vec![1],
                    ..Default::default()
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
        let folders = unclaimed_folders(rows);
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
        let extensions: Vec<String> = extensions_by_files(coverage)
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
                notes.push(TerminalNote::Text(failure.to_string()));
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

/// Unrecognized extensions, most files first, ties by name.
fn extensions_by_files(coverage: &Coverage) -> Vec<(&String, &usize)> {
    let mut extensions: Vec<_> = coverage.unrecognized_extensions.iter().collect();
    extensions.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    extensions
}

fn render_coverage(output: &mut String, findings: &Findings) {
    let coverage = findings.coverage;
    writeln!(output, "\n## Coverage\n\nFiles selected: {}; counted: {}, including {} through language mappings; not recognized: {}; recognized but unreadable: {}. Their lines are missing, not zero. Counts describe the working copy, including untracked files, not a commit.", grouped(coverage.selected_files as u64), grouped(coverage.counted_files as u64), grouped(coverage.mapped_files as u64), grouped(coverage.unrecognized_files as u64), grouped(coverage.unread.len() as u64)).unwrap();
    let extensions = extensions_by_files(coverage);
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

/// Qualifies a policy note measured in a subfolder only.
fn subtree(subfolder: Option<&str>) -> &'static str {
    match subfolder {
        Some(_) => " in this subtree",
        None => "",
    }
}

/// Names configuration entries that decided no file, which usually means a
/// glob that matches nothing or an entry an earlier stage preempts.
fn unused_entries_note(entries: &[String], subfolder: Option<&str>) -> Option<String> {
    (!entries.is_empty()).then(|| {
        format!(
            "Entries that decided no file{}: {}.",
            subtree(subfolder),
            entries.join(", ")
        )
    })
}

/// Scoped non-use cannot establish whether project exclusions need repair.
fn unused_exclusions_note(patterns: &[String], subfolder: Option<&str>) -> Option<String> {
    (!patterns.is_empty()).then(|| {
        format!(
            "Exclusions that excluded nothing{}: {}; {}.",
            subtree(subfolder),
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

    /// A row with this Code per purpose and no comments.
    fn code(purposes: [u64; 3]) -> RowCode {
        RowCode {
            purposes,
            comments: 0,
        }
    }

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
            breaks: vec![],
            strong: None,
            secondary: vec![],
            ..Default::default()
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
            let render = |theme| {
                let options = TableOptions {
                    categories: false,
                    folders: false,
                    verbose: false,
                    order: RowOrder::Name,
                    theme,
                    folder_rows: None,
                    bars: Bars::Off,
                    purposes: Purpose::ALL.to_vec(),
                };
                render_table_with_options(&summary, &rows, &findings, options)
            };
            let (styled, plain) = (render(Theme::TERMINAL), render(Theme::PLAIN));
            assert!(plain.starts_with(" Language "), "{plain}");
            assert!(!plain.contains("Code by purpose"));
            assert!(styled.starts_with(" \x1b[1mLanguage"), "{styled}");
            let legend = ALL_PURPOSES.legend(Some("Lines = Code + Comments + Blanks"), None);
            assert!(styled.contains(&format!("\x1b[90m{legend}\x1b[0m")));
            if !members.is_empty() {
                let legend = ALL_PURPOSES.legend(None, Some("Share = Code / workspace Code"));
                assert!(styled.contains(&format!("\x1b[90m{legend}\x1b[0m")));
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
            &["Name", "Code", "Comments", "Blanks", "Other"],
            &[vec![
                "api".into(),
                "12".into(),
                "3".into(),
                "0".into(),
                "4".into(),
            ]],
            None,
            &Layout {
                breaks: vec![2, 4],
                strong: Some(1),
                secondary: vec![2, 3],
                ..Default::default()
            },
            Theme::TERMINAL,
        );
        assert!(table.contains("\x1b[1mapi \x1b[0m"), "{table}");
        assert!(table.contains("\x1b[2m       3\x1b[0m"), "{table}");
        assert!(table.contains("\x1b[90m     0\x1b[0m"), "{table}");
        assert!(table.contains("    4\n"), "{table}");
        assert!(!table.contains("\x1b[33m"));
    }

    /// Each purpose keeps one hue from header to total, so the header doubles as
    /// the legend; zeros stay gray so an absent purpose recedes.
    #[test]
    fn purpose_columns_carry_their_purpose_hue_from_header_to_total() {
        let table = text_table(
            &["Name", "Code", "Production", "Test", "Other"],
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
                breaks: vec![],
                strong: Some(1),
                purposes: (2..).zip(Purpose::ALL).collect(),
                ..Layout::default()
            },
            Theme::TERMINAL,
        );
        let lines: Vec<_> = table.lines().collect();
        let (header, body, total) = (lines[0], lines[2], lines[4]);
        assert!(header.contains("\x1b[1;36mCode\x1b[0m"), "{header}");
        assert!(header.contains("\x1b[1;32mProduction\x1b[0m"), "{header}");
        assert!(header.contains("\x1b[1;35mTest\x1b[0m"), "{header}");
        assert!(header.contains("\x1b[1;34mOther\x1b[0m"), "{header}");
        assert!(body.contains("\x1b[32m         5\x1b[0m"), "{body}");
        assert!(body.contains("\x1b[35m   4\x1b[0m"), "{body}");
        assert!(body.contains("\x1b[90m    0\x1b[0m"), "{body}");
        assert!(total.contains("\x1b[1;32m         5\x1b[0m"), "{total}");
        assert!(total.contains("\x1b[1;35m   4\x1b[0m"), "{total}");
    }

    /// Bars join a table only when the minimum inline bar fits beside it, and
    /// never narrower than their column title; the threshold follows the
    /// table's width, not a fixed terminal size.
    #[test]
    fn inline_bars_need_the_minimum_bar_beside_the_table() {
        let threshold = 90 + 2 + 10;
        assert_eq!(inline_bar_width(90, "Code share", threshold - 1), None);
        assert_eq!(inline_bar_width(90, "Code share", threshold), Some(10));
        assert_eq!(inline_bar_width(90, "Code share", threshold + 7), Some(17));
        assert_eq!(
            inline_bar_width(90, "Code share", 1000),
            Some(*BAR_CELLS.end())
        );

        let threshold = 90 + 2 + "Comment density".len();
        assert_eq!(inline_bar_width(90, "Comment density", threshold - 1), None);
        assert_eq!(inline_bar_width(90, "Comment density", threshold), Some(15));
    }

    /// Automatic bars only decorate a table they fit beside: otherwise the
    /// table stays exactly as without bars, with neither Share column nor chart.
    /// The oracle is the language table's former hand-written layout: Code
    /// accented, Comments and the purposes and ratio each starting a group,
    /// counts beside Code and the ratio secondary.
    #[test]
    fn columns_derive_the_layout_their_roles_describe() {
        let layout = Columns::with_purposes(&LANGUAGE_COLUMNS, &ALL_PURPOSES).layout();
        assert_eq!(layout.strong, Some(3));
        assert_eq!(layout.breaks, [4, 6, 9]);
        assert_eq!(layout.secondary, [1, 4, 5, 9]);
        assert_eq!(
            layout.purposes,
            [
                (6, Purpose::Production),
                (7, Purpose::Test),
                (8, Purpose::Other)
            ]
        );

        // One purpose leaves only the ratio, which still starts its group.
        let production = PurposeColumns {
            shown: &[Purpose::Production],
        };
        let layout = Columns::with_purposes(&code_columns("Folder"), &production).layout();
        assert_eq!((layout.breaks, layout.secondary), (vec![3], vec![3]));
    }

    /// A short title replaces a full one only while the full title would be
    /// wider than every value; Markdown keeps the full title regardless.
    #[test]
    fn short_titles_apply_only_when_the_full_title_outgrows_its_values() {
        let columns = Columns::with_purposes(&code_columns("Folder"), &ALL_PURPOSES);
        let row = |production: &str| {
            vec![
                "src".into(),
                "1".into(),
                "1".into(),
                production.into(),
                "0".into(),
                "0".into(),
                "-".into(),
            ]
        };
        let narrow = columns.fitted_titles(&[row("9,978")], None);
        assert_eq!(narrow[3], "Prod.");
        let wide = columns.fitted_titles(&[row("1,234,567,890")], None);
        assert_eq!(wide[3], "Production");
        assert!(columns.markdown_header().contains("| Production |"));
    }

    #[test]
    fn markdown_headers_bold_code_and_right_align_figures() {
        assert_eq!(
            Columns::with_purposes(&code_columns("Folder"), &ALL_PURPOSES).markdown_header(),
            "| Folder | Lines | **Code** | Production | Test | Other | Test/prod |\n| --- | ---: | ---: | ---: | ---: | ---: | ---: |"
        );
    }

    #[test]
    fn fitting_bars_leave_tables_untouched_when_they_do_not_fit() {
        let body = [
            vec!["api".to_string(), "9".into()],
            vec!["web".into(), "3".into()],
        ];
        let total = vec!["Total".to_string(), "12".into()];
        let columns = Columns(vec![
            Column::new("Name", ColumnRole::Label),
            Column::new("Code", ColumnRole::Code),
        ]);
        let codes = [code([6, 3, 0]), code([0, 0, 3])];
        let render = |bars| {
            let rows = BarRows {
                rows: &codes,
                total: code([12, 0, 0]),
                columns: &ALL_PURPOSES,
                bars,
            };
            bar_table(&columns, &body, total.clone(), &rows, Theme::PLAIN)
        };
        let plain = text_table(
            &columns.fitted_titles(&body, Some(&total)),
            &body,
            Some(total.clone()),
            &columns.layout(),
            Theme::PLAIN,
        );
        assert_eq!(render(Bars::Fit(30)), (plain, String::new()));
        let (wide, key) = render(Bars::Fit(60));
        assert!(
            wide.contains(" api       9 │  75.0%  █████████████████████████▓▓▓▓▓▓▓▓▓▓▓▓\n"),
            "{wide}"
        );
        assert!(key.contains("Bars:"), "{key}");
        let (_, chart) = render(Bars::Always(30, BarKind::Purpose));
        assert!(chart.contains("Code share"), "{chart}");
    }

    /// The largest row fills the bar; others scale to it, split by purpose into
    /// whole cells that add up to their length. Percentages are of total Code,
    /// and the glyphs keep purposes apart without color.
    #[test]
    fn share_bars_split_each_row_by_purpose_within_the_available_width() {
        let rows = [
            ("api".to_string(), code([6, 3, 1])),
            ("web".to_string(), code([0, 0, 5])),
            ("ops".to_string(), code([4, 0, 0])),
            ("dev".to_string(), code([0, 1, 0])),
        ];
        let plain = share_bars(
            &rows,
            code([20, 0, 0]),
            BarKind::Purpose,
            33,
            &Purpose::ALL,
            Theme::PLAIN,
        );
        assert_eq!(
            plain,
            " Code share  █ Production  ▓ Test  ░ Other\n \
             api  50.0%  ████████████▓▓▓▓▓▓░░\n \
             web  25.0%  ░░░░░░░░░░\n \
             ops  20.0%  ████████\n \
             dev   5.0%  ▓▓\n"
        );
        let styled = share_bars(
            &rows,
            code([20, 0, 0]),
            BarKind::Purpose,
            33,
            &Purpose::ALL,
            Theme::TERMINAL,
        );
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
    /// The bar keeps a readable minimum and stops at 40 cells.
    #[test]
    fn share_bars_mark_slivers_and_clamp_their_width() {
        let rows = [
            ("big".to_string(), code([999, 0, 0])),
            ("one".to_string(), code([0, 1, 0])),
            ("nil".to_string(), code([0, 0, 0])),
        ];
        let text = share_bars(
            &rows,
            code([1000, 0, 0]),
            BarKind::Purpose,
            33,
            &Purpose::ALL,
            Theme::PLAIN,
        );
        assert!(text.contains("\n one   0.1%  ▏\n"), "{text}");
        assert!(text.contains("\n nil   0.0%\n"), "{text}");
        let bar = |columns| {
            let text = share_bars(
                &rows,
                code([1000, 0, 0]),
                BarKind::Purpose,
                columns,
                &Purpose::ALL,
                Theme::PLAIN,
            );
            let line = text.lines().nth(1).unwrap().to_string();
            line.chars().filter(|c| *c == '█').count()
        };
        assert_eq!(bar(10), 10);
        assert_eq!(bar(500), 40);
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
                breaks: vec![],
                strong: Some(1),
                secondary: vec![2],
                ..Default::default()
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
            breaks: vec![2],
            strong: Some(1),
            secondary: vec![],
            ..Default::default()
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
