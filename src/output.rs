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

pub(crate) fn plain(value: &str) -> String {
    value
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect()
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
        grouped(counts.code),
        grouped(counts.comments),
        grouped(counts.blanks),
        grouped(counts.lines)
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

/// Repository rows and the total row: source code, code per purpose, test
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
        let mut row = vec![label.to_string(), grouped(totals.source.code)];
        row.extend(Purpose::ALL.map(|purpose| grouped(code(purpose))));
        row.extend([ratio, share]);
        row
    };
    let empty = Summary::default();
    let rows = members
        .iter()
        .map(|member| {
            let totals = summary
                .folders
                .get(&member.name)
                .map_or(&empty, |folder| &folder.recursive);
            (member.name.clone(), totals)
        })
        .collect();
    let body = ordered(rows, |totals| totals.source.code, order)
        .into_iter()
        .map(|(name, totals)| cells(&name, totals))
        .collect();
    (body, cells("Total", &summary.totals))
}

/// Legends that state how a table's columns add up; readers otherwise
/// misread Code as a sum of Comments and Blanks.
const LANGUAGE_LEGEND: &str =
    "Code + Comments + Blanks = all lines; Production + Test + Unclassified = Code.";
const FOLDER_LEGEND: &str =
    "Production + Test + Unclassified = Code; Test/prod = Test / Production.";
const REPOSITORY_LEGEND: &str = "Production + Test + Unclassified = Code; Test/prod = Test / Production; Share = Code / workspace Code.";
const CATEGORY_LEGEND: &str =
    "Rows of one purpose add up to that purpose's Code; all rows add up to the total Code.";

const REPOSITORY_HEADER: [&str; 7] = [
    "Repository",
    "Code",
    "Production",
    "Test",
    "Unclassified",
    "Test/prod",
    "Share",
];

/// A Markdown grid of source code per `key` and member, with a total column;
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
    writeln!(
        output,
        "\n## {title}\n\n{}\n\n| {label} | {} | Total |\n| --- |{} ---: |",
        Theme::MARKDOWN.paint(Role::Muted, "Total is the sum of the repository columns."),
        names
            .iter()
            .map(|name| cell(name))
            .collect::<Vec<_>>()
            .join(" | "),
        " ---: |".repeat(names.len())
    )
    .unwrap();
    let rows = of(&summary.totals)
        .iter()
        .map(|(key, counts)| (key.clone(), counts.code))
        .collect();
    for (key, total) in ordered(rows, |code| *code, order) {
        let cells: Vec<String> = names
            .iter()
            .map(|name| {
                grouped(
                    summary
                        .folders
                        .get(*name)
                        .and_then(|folder| of(&folder.recursive).get(&key))
                        .map_or(0, |counts| counts.code),
                )
            })
            .collect();
        writeln!(
            output,
            "| {} | {} | {} |",
            cell(&key),
            cells.join(" | "),
            grouped(total)
        )
        .unwrap();
    }
}

/// Names members with unclaimed source or unavailable counts, which a person
/// can act on. Unrecognized files are left to the "Not counted" note: on real
/// repositories they are mostly images, so listing them named every member
/// (docs/research/real-project-trials.md#workspace-run).
fn repositories_note(members: &[MemberView]) -> Option<String> {
    let parts: Vec<String> = members
        .iter()
        .filter_map(|member| {
            let mut findings = Vec::new();
            if member.unclaimed_files > 0 {
                findings.push(format!(
                    "{} ({})",
                    counted(member.unclaimed_files, "unclaimed file"),
                    counted(member.unclaimed_code, "code line")
                ));
            }
            if member.unread_files > 0 {
                findings.push(counted(member.unread_files as u64, "unreadable file"));
            }
            if member.parse_failed_files > 0 {
                findings.push(counted(
                    member.parse_failed_files as u64,
                    "file with a parsing failure",
                ));
            }
            (!findings.is_empty()).then(|| format!("{}: {}", member.name, findings.join(", ")))
        })
        .collect();
    (!parts.is_empty()).then(|| format!("Repositories to review: {}.", parts.join("; ")))
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
    writeln!(output, "\n## {title}\n\n| {label} | Files | **Code** | Comments | Blanks | Lines |\n| --- | ---: | ---: | ---: | ---: | ---: |").unwrap();
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
    let mut output = format!("# {}: lines of code by purpose\n\n{}\n\n## Source purpose\n\n| Purpose | Files | **Code** | Comments | Blanks | Lines |\n| --- | ---: | ---: | ---: | ---: | ---: |\n", cell(&project), Theme::MARKDOWN.paint(Role::Muted, "Production means maintained non-test source, including separately categorized tooling. Data, configuration, and documentation are counted apart from source. Lines = Code + Comments + Blanks in every table with these columns, and the purpose rows add up to all source."));
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
    table(
        &mut output,
        "Source languages",
        "Language",
        &summary.totals.source_languages,
        order,
    );
    write!(output, "\n## Folder breakdown\n\n{}\n\n| Folder | Production | Test | Unclassified | **Total source code** | Test/prod |\n| --- | ---: | ---: | ---: | ---: | ---: |\n", Theme::MARKDOWN.paint(Role::Muted, &format!("Recursive source code totals to depth {depth}. Parent rows include their children; do not sum all displayed rows. The report file's additive `breakdown` rows cover every depth. Multi-language files count once physically but contribute to each contained language. Production + Test + Unclassified = Total source code; Test/prod = Test / Production."))).unwrap();
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
            "| {} | {} | {} | {} | {} | {} |",
            cell(&folder),
            grouped(production),
            grouped(test),
            grouped(totals.purpose(Purpose::Unclassified).code),
            grouped(totals.source.code),
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
        output.push_str("\n| Folder | Files | Code |\n| --- | ---: | ---: |\n");
        for share in &folders {
            writeln!(
                output,
                "| {} | {} | {} |",
                cell(folder_label(&share.folder)),
                grouped(share.files),
                grouped(share.code)
            )
            .unwrap();
        }
    }
    writeln!(output, "\nLargest 25 files; the report file's `review` list names all of them with their rule.\n\n| Path | Code | Category |\n| --- | ---: | --- |").unwrap();
    for row in unknown.into_iter().take(25) {
        writeln!(
            output,
            "| {} | {} | {} |",
            cell(&row.path),
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
    output.push_str("\n## Build modules\n\nFolders whose build file declares a source layout; that layout counts as production where no override, convention, or rule decided first. Code is the source each module classified; 0 means other stages claimed its files or its layout matched none.\n\n| Build system | Modules | Code |\n| --- | ---: | ---: |\n");
    let systems = build_systems(modules)
        .into_iter()
        .map(|(id, system)| (id.to_string(), system))
        .collect();
    for (id, system) in ordered(systems, |system| system.code, order) {
        writeln!(
            output,
            "| {id} | {} | {} |",
            grouped(system.modules),
            grouped(system.code)
        )
        .unwrap();
    }
    output.push_str("\n| Module | Build system | Code |\n| --- | --- | ---: |\n");
    for module in modules {
        let path = match module.path.as_str() {
            "." => "(scan root)",
            path => path,
        };
        writeln!(
            output,
            "| {} | {} | {} |",
            cell(path),
            module.detector,
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
    /// Columns preceded by a `│` separator.
    breaks: &'static [usize],
    /// Column carrying the primary metric.
    strong: Option<usize>,
    secondary: &'static [usize],
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
    let header: Vec<String> = header.iter().map(|title| title.to_string()).collect();
    let all = std::iter::once(&header).chain(body).chain(total.as_ref());
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
    let line = |row: &[String], row_role: Option<Role>| {
        let mut text = String::from(" ");
        for (column, (value, width)) in row.iter().zip(&widths).enumerate() {
            if column > 0 {
                text += &theme.paint(Role::Muted, gap(column));
            }
            let cell = match column {
                0 => format!("{value:<width$}"),
                _ => format!("{value:>width$}"),
            };
            let role = match (row_role, column, value.as_str()) {
                (Some(role), _, _) => Some(role),
                (None, 0, _) => Some(Role::Information),
                (None, _, "-" | "0" | "0.00" | "0.0%") => Some(Role::Muted),
                _ if layout.strong == Some(column) => Some(Role::Primary),
                _ if layout.secondary.contains(&column) => Some(Role::Secondary),
                _ => None,
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
    let mut output = line(&header, Some(Role::Heading)) + &rule;
    for row in body {
        output += &line(row, None);
    }
    if let Some(total) = total {
        output += &rule;
        output += &line(&total, Some(Role::Primary));
    }
    output
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
                folder_label(&share.folder).to_string(),
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
        theme.paint(Role::Warning, "Unclaimed source:"),
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
    note + "\n Classify them in the configuration; locmetry --schema config explains how, --files --output - shows each file's rule."
}

/// Folders listed in the terminal note; the JSON headline lists all of them.
const FOLDER_LINES: usize = 8;

/// Terminal report for people: source code per language and top-level folder
/// split by purpose, then one line each for what the tables leave out.
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
    let mut languages = BTreeMap::<String, (Counts, BTreeMap<Purpose, u64>)>::new();
    for row in breakdown(rows)
        .iter()
        .filter(|row| row.kind == Kind::Source)
    {
        let (counts, purposes) = languages.entry(row.language.clone()).or_default();
        counts.add(&row.counts);
        *purposes.entry(row.purpose).or_default() += row.counts.code;
    }
    let languages = ordered(
        languages.into_iter().collect(),
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
            row.extend([counts.files, counts.code, counts.comments, counts.blanks].map(grouped));
            row.extend(purpose_cells(&|purpose| {
                purposes.get(&purpose).copied().unwrap_or(0)
            }));
            row
        })
        .collect();
    let source = &summary.totals.source;
    let mut total = vec!["Total".to_string()];
    total.extend([source.files, source.code, source.comments, source.blanks].map(grouped));
    total.extend(purpose_cells(&|purpose| {
        summary.totals.purpose(purpose).code
    }));
    let project = display_project(project, findings.subfolder);
    let mut output = format!(
        " {}\n",
        theme.paint(Role::Primary, &format!("{project} · Source code"))
    );
    output += &text_table(
        &[
            "Language",
            "Files",
            "Code",
            "Comments",
            "Blanks",
            "Production",
            "Test",
            "Unclassified",
        ],
        &body,
        Some(total),
        &Layout {
            breaks: &[3, 5],
            strong: Some(2),
            secondary: &[1, 3, 4],
        },
        theme,
    );
    output += &format!(" {}\n", theme.paint(Role::Muted, LANGUAGE_LEGEND));

    let folder_code = |purposes: &BTreeMap<Purpose, u64>| purposes.values().sum::<u64>();
    // A workspace's top-level folders are its members, so repository rows,
    // which add the test ratio and share, take the folder rows' place.
    if !findings.members.is_empty() {
        let (body, total) = repository_rows(summary, findings.members, order);
        output += "\n";
        output += &text_table(
            &REPOSITORY_HEADER,
            &body,
            Some(total),
            &Layout {
                breaks: &[2, 5],
                strong: Some(1),
                secondary: &[5, 6],
            },
            theme,
        );
        output += &format!(" {}\n", theme.paint(Role::Muted, REPOSITORY_LEGEND));
    }
    let folders = ordered(
        human_top_folders(summary, findings.subfolder),
        folder_code,
        order,
    );
    if !folders.is_empty() && findings.members.is_empty() {
        let body: Vec<Vec<String>> = folders
            .iter()
            .map(|(folder, purposes)| {
                // `.` in top_folders holds only the files directly in the root.
                let label = match folder.as_str() {
                    "." => "(root files)".to_string(),
                    _ => folder.clone(),
                };
                let code = |purpose| purposes.get(&purpose).copied().unwrap_or(0);
                let mut row = vec![label, grouped(folder_code(purposes))];
                row.extend(purpose_cells(&code));
                row.push(test_ratio(code(Purpose::Production), code(Purpose::Test)));
                row
            })
            .collect();
        let code = |purpose| summary.totals.purpose(purpose).code;
        let mut total = vec!["Total".to_string(), grouped(summary.totals.source.code)];
        total.extend(purpose_cells(&code));
        total.push(test_ratio(code(Purpose::Production), code(Purpose::Test)));
        output += "\n";
        output += &text_table(
            &[
                "Folder",
                "Code",
                "Production",
                "Test",
                "Unclassified",
                "Test/prod",
            ],
            &body,
            Some(total),
            &Layout {
                breaks: &[2, 5],
                strong: Some(1),
                secondary: &[5],
            },
            theme,
        );
        output += &format!(" {}\n", theme.paint(Role::Muted, FOLDER_LEGEND));
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
            .map(|(category, counts)| vec![category, grouped(counts.files), grouped(counts.code)])
            .collect();
        if !body.is_empty() {
            output += "\n";
            output += &text_table(
                &["Category", "Files", "Code"],
                &body,
                None,
                &Layout {
                    breaks: &[],
                    strong: Some(2),
                    secondary: &[1],
                },
                theme,
            );
            output += &format!(" {}\n", theme.paint(Role::Muted, CATEGORY_LEGEND));
        }
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
    let coverage = findings.coverage;
    notes.extend(build_modules_note(findings.modules, order));
    notes.extend(submodules_note(findings.submodules));
    let folders = unclassified_folders(rows);
    if !folders.is_empty() {
        notes.push(unclaimed_note(&folders, theme));
    }
    notes.extend(unused_entries_note(
        findings.unused_entries,
        findings.subfolder,
    ));
    notes.extend(unused_exclusions_note(
        findings.unused_exclusions,
        findings.subfolder,
    ));
    notes.extend(repositories_note(findings.members));
    let mut gaps = Vec::new();
    if coverage.unrecognized_files > 0 {
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
        gaps.push(format!(
            "{} ({})",
            counted(coverage.unrecognized_files as u64, "unrecognized file"),
            extensions.join(", ")
        ));
    }
    if !coverage.unread.is_empty() {
        gaps.push(counted(coverage.unread.len() as u64, "unreadable file"));
    }
    if !coverage.parse_failures.is_empty() {
        gaps.push(counted(
            coverage.parse_failures.len() as u64,
            "file with a parsing failure",
        ));
        for failure in coverage.parse_failures.iter().take(25) {
            notes.push(format!(
                "{}: {} — {}",
                failure.path, failure.reason, failure.message
            ));
        }
    }
    if !gaps.is_empty() {
        notes.push(format!(
            "{} {}; missing lines are not zero.",
            theme.paint(Role::Warning, "Not counted:"),
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
        .chain(repositories_note(findings.members))
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

/// Names exclusions that left out nothing, which usually means a folder
/// written without `/**`, a misspelled path, or one an ignore file already hides.
fn unused_exclusions_note(patterns: &[String], subfolder: Option<&str>) -> Option<String> {
    (!patterns.is_empty()).then(|| {
        format!(
            "Exclusions that excluded nothing{}: {}; exclude a folder as <folder>/**, and an ignore file may already hide the path.",
            if subfolder.is_some() { " in this subtree" } else { "" },
            patterns.join(", ")
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

fn human_top_folders(
    summary: &ReportSummary,
    subfolder: Option<&str>,
) -> Vec<(String, BTreeMap<Purpose, u64>)> {
    match subfolder {
        None => summary.top_folders().into_iter().collect(),
        Some(_) => human_folders(summary, subfolder, 1)
            .into_iter()
            .filter(|(_, totals)| totals.source.files > 0)
            .map(|(path, totals)| {
                (
                    path,
                    totals
                        .source_purpose
                        .iter()
                        .map(|(purpose, counts)| (*purpose, counts.code))
                        .collect(),
                )
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            assert!(styled.contains(&format!("\x1b[90m{LANGUAGE_LEGEND}\x1b[0m")));
            if !members.is_empty() {
                assert!(styled.contains(&format!("\x1b[90m{REPOSITORY_LEGEND}\x1b[0m")));
            }
            assert_eq!(
                styled
                    .replace("\x1b[1m", "")
                    .replace("\x1b[90m", "")
                    .replace("\x1b[1;36m", "")
                    .replace("\x1b[36m", "")
                    .replace("\x1b[2m", "")
                    .replace("\x1b[0m", ""),
                plain
            );
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
            },
            Theme::TERMINAL,
        );
        assert!(table.contains("\x1b[36mapi \x1b[0m"), "{table}");
        assert!(table.contains("\x1b[2m       3\x1b[0m"), "{table}");
        assert!(table.contains("\x1b[90m     0\x1b[0m"), "{table}");
        assert!(table.contains("           4\n"), "{table}");
        assert!(!table.contains("\x1b[33m"));
    }

    /// Bold must not shift alignment: without its escape codes, an
    /// emphasized table equals the plain one.
    #[test]
    fn emphasis_bolds_the_strong_column_without_shifting_alignment() {
        let layout = Layout {
            breaks: &[2],
            strong: Some(1),
            secondary: &[],
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
        assert!(bold.contains("\x1b[1mCode\x1b[0m"), "{bold}");
        assert!(bold.contains("\x1b[1;36m  12\x1b[0m"), "{bold}");
        assert_eq!(
            bold.replace("\x1b[1m", "")
                .replace("\x1b[90m", "")
                .replace("\x1b[1;36m", "")
                .replace("\x1b[36m", "")
                .replace("\x1b[2m", "")
                .replace("\x1b[0m", ""),
            plain
        );
        assert!(!plain.contains('\x1b'));
    }
}
