use super::*;
use crate::output::{self, Column, ColumnRole, Columns};
use crate::style::{Role, Theme};
use crate::{Counts, Kind};
use std::fmt::Write;

mod notes;

fn label(value: &str, markdown: bool) -> String {
    match markdown {
        true => crate::output::cell(value),
        false => crate::style::plain(value),
    }
}
/// Lines and Code before, after and net change, as every comparison grid shows them.
const CHANGE_COLUMNS: [Column; 6] = [
    Column::new("Lines before", ColumnRole::Count).under("Lines", "before"),
    Column::new("Lines after", ColumnRole::Count).under("Lines", "after"),
    Column::new("Lines change", ColumnRole::Change).under("Lines", "change"),
    Column::new("Code before", ColumnRole::Count)
        .under("Code", "before")
        .separated(),
    Column::new("Code after", ColumnRole::Count).under("Code", "after"),
    Column::new("Code change", ColumnRole::Change).under("Code", "change"),
];
/// Changes beside the Totals figures, muted as the measurement table mutes
/// Comments, Blanks and Files.
const SECONDARY_CHANGE_COLUMNS: [Column; 3] = [
    Column::new("Comments change", ColumnRole::Secondary)
        .under("Change", "Comments")
        .separated(),
    Column::new("Blanks change", ColumnRole::Secondary).under("Change", "Blanks"),
    Column::new("Files change", ColumnRole::Secondary).under("Change", "Files"),
];

/// One row of a comparison grid.
struct Row {
    label: String,
    before: Counts,
    after: Counts,
}

impl Row {
    fn new(label: impl Into<String>, change: &CountChange) -> Self {
        Self {
            label: label.into(),
            before: change.before,
            after: change.after,
        }
    }

    /// Cells for `columns`, which start with the label and [`CHANGE_COLUMNS`];
    /// the change is recomputed wide, so a row summed for display cannot overflow.
    fn cells(&self, secondary: bool, markdown: bool) -> Vec<String> {
        let change =
            |before: u64, after: u64| output::signed(i128::from(after) - i128::from(before));
        let (before, after) = (&self.before, &self.after);
        let mut cells = vec![
            label(&self.label, markdown),
            output::grouped(before.lines),
            output::grouped(after.lines),
            change(before.lines, after.lines),
            output::grouped(before.code),
            output::grouped(after.code),
            change(before.code, after.code),
        ];
        if secondary {
            cells.extend([
                change(before.comments, after.comments),
                change(before.blanks, after.blanks),
                change(before.files, after.files),
            ]);
        }
        cells
    }
}

/// A titled comparison grid; [`Grid::write`] writes nothing without body
/// rows, so a grid never shows only a total.
struct Grid<'t> {
    title: &'t str,
    /// Heading of the label column.
    label: &'static str,
    rows: Vec<Row>,
    total: Option<Row>,
    /// Adds [`SECONDARY_CHANGE_COLUMNS`].
    secondary: bool,
}

impl Grid<'_> {
    fn write(self, output: &mut String, markdown: bool, theme: Theme) {
        if self.rows.is_empty() {
            return;
        }
        let mut columns = vec![Column::new(self.label, ColumnRole::Label)];
        columns.extend(CHANGE_COLUMNS);
        if self.secondary {
            columns.extend(SECONDARY_CHANGE_COLUMNS);
        }
        let cells = |row: &Row| row.cells(self.secondary, markdown);
        let body: Vec<_> = self.rows.iter().map(cells).collect();
        let total = self.total.as_ref().map(cells);
        match markdown {
            true => writeln!(output, "\n## {}\n", self.title),
            false => writeln!(output, "\n{}", theme.paint(Role::Heading, self.title)),
        }
        .unwrap();
        output.push_str(&output::shared_table(
            &Columns::of(&columns),
            &body,
            total.as_deref(),
            markdown,
            theme,
        ));
    }
}

/// Changes per content kind present in either report; together they make up
/// All counted, which validation guarantees `by_kind_purpose` sums to.
fn kind_rows(dimensions: &Dimensions) -> Vec<Row> {
    Kind::ALL
        .into_iter()
        .filter_map(|kind| {
            let prefix = format!("{kind}:");
            let changes: Vec<_> = dimensions
                .by_kind_purpose
                .iter()
                .filter(|(key, _)| key.starts_with(&prefix))
                .map(|(_, change)| change)
                .collect();
            let sum = |side: fn(&CountChange) -> Counts| {
                changes.iter().fold(Counts::default(), |sum, change| {
                    sum.checked_add(&side(change))
                        .expect("kind totals are bounded by all_counted")
                })
            };
            (!changes.is_empty()).then(|| Row {
                label: kind_title(kind).to_string(),
                before: sum(|change| change.before),
                after: sum(|change| change.after),
            })
        })
        .collect()
}

fn kind_title(kind: Kind) -> &'static str {
    match kind {
        Kind::Source => "Source",
        Kind::Data => "Data",
        Kind::Documentation => "Documentation",
        Kind::Unknown => "Unknown",
    }
}

fn render(c: &Comparison, markdown: bool, theme: Theme) -> String {
    let mut out = String::new();
    let title = format!(
        "Net change: {} → {}",
        label(&c.before.project, markdown),
        label(&c.after.project, markdown)
    );
    match markdown {
        true => writeln!(out, "# {title}"),
        false => writeln!(out, "{}", theme.paint(Role::Heading, &title)),
    }
    .unwrap();
    if let Some(folder) = c
        .before
        .metadata
        .scope
        .as_ref()
        .and_then(|s| s.subfolder.as_deref())
    {
        writeln!(
            out,
            "Subfolder: {} (project-relative; measured contributions only)",
            label(folder, markdown)
        )
        .unwrap();
    }
    out.push('\n');
    for note in [
        "Net change = after − before, from the two saved reports.",
        "Unreadable files count as 0; a folder seen in only one report may be newly visible, not newly created.",
    ] {
        writeln!(out, "{}", theme.paint(Role::Muted, note)).unwrap();
    }
    let dimensions = &c.summary.dimensions;
    let source_total = || Some(Row::new("Source total", &dimensions.source));
    let changed = |change: &CountChange| change.delta.code != 0 || change.delta.lines != 0;
    Grid {
        title: "Totals",
        label: "Kind",
        rows: kind_rows(dimensions),
        total: Some(Row::new("All counted", &dimensions.all_counted)),
        secondary: true,
    }
    .write(&mut out, markdown, theme);
    Grid {
        title: "Purpose",
        label: "Purpose",
        rows: dimensions
            .source_purpose
            .iter()
            .filter(|(_, v)| changed(v))
            .map(|(k, v)| Row::new(output::purpose_title(*k), v))
            .collect(),
        total: source_total(),
        secondary: false,
    }
    .write(&mut out, markdown, theme);
    Grid {
        title: "Languages",
        label: "Language",
        rows: dimensions
            .source_languages
            .iter()
            .filter(|(_, v)| changed(v))
            .map(|(k, v)| Row::new(k.as_str(), v))
            .collect(),
        total: source_total(),
        secondary: false,
    }
    .write(&mut out, markdown, theme);
    if let Some(members) = &c.members {
        Grid {
            title: "Workspace members",
            label: "Member",
            rows: members
                .iter()
                .filter(|(_, v)| v.presence != Presence::Both || changed(&v.dimensions.source))
                .map(|(k, v)| {
                    Row::new(
                        format!("{k} ({})", presence_name(v.presence)),
                        &v.dimensions.source,
                    )
                })
                .collect(),
            total: source_total(),
            secondary: false,
        }
        .write(&mut out, markdown, theme);
    }
    if let Some(composition) = &c.summary.composition {
        writeln!(
            out,
            "\nWorkspace source lines: matched {:+}, entering +{}, leaving -{}; total {:+}.",
            composition.source.matched.lines,
            composition.source.entering.lines,
            composition.source.leaving.lines,
            c.summary.dimensions.source.delta.lines
        )
        .unwrap();
        writeln!(
            out,
            "Workspace all-counted lines: matched {:+}, entering +{}, leaving -{}; total {:+}.",
            composition.all_counted.matched.lines,
            composition.all_counted.entering.lines,
            composition.all_counted.leaving.lines,
            c.summary.dimensions.all_counted.delta.lines
        )
        .unwrap();
        writeln!(
            out,
            "\nWorkspace source code: matched {:+}, entering +{}, leaving -{}; total {:+}.",
            composition.source.matched.code,
            composition.source.entering.code,
            composition.source.leaving.code,
            c.summary.dimensions.source.delta.code
        )
        .unwrap();
        writeln!(
            out,
            "Workspace all-counted code: matched {:+}, entering +{}, leaving -{}; total {:+}.",
            composition.all_counted.matched.code,
            composition.all_counted.entering.code,
            composition.all_counted.leaving.code,
            c.summary.dimensions.all_counted.delta.code
        )
        .unwrap();
    }
    if let Some(folders) = &c.folders {
        let mut folders: Vec<_> = folders.iter().collect();
        folders.sort_by(|(a, _), (b, _)| a.split('/').cmp(b.split('/')));
        let mut rows: Vec<_> = folders
            .into_iter()
            .filter(|(path, v)| {
                (markdown || (c.members.is_none() && (path.as_str() == "." || !path.contains('/'))))
                    && (v.presence != Presence::Both || changed(&v.recursive.source))
            })
            .map(|(k, v)| {
                Row::new(
                    format!("{k} ({})", presence_name(v.presence)),
                    &v.recursive.source,
                )
            })
            .collect();
        // The table lists top-level folders only, so the recursive root row is
        // their source total; Markdown keeps it in place within the hierarchy.
        let total = match markdown {
            true => None,
            false => rows
                .iter()
                .position(|row| row.label.starts_with(". ("))
                .map(|index| Row {
                    label: "Source total".into(),
                    ..rows.remove(index)
                }),
        };
        Grid {
            title: "Folders (recursive; do not sum parents and children)",
            label: "Folder",
            rows,
            total,
            secondary: false,
        }
        .write(&mut out, markdown, theme);
    }
    let summary = &c.presence_summary;
    let presence: Vec<_> = [
        (summary.members.before_only, "member", "before"),
        (summary.members.after_only, "member", "after"),
        (summary.folders.before_only, "folder", "before"),
        (summary.folders.after_only, "folder", "after"),
        (summary.packages.before_only, "package", "before"),
        (summary.packages.after_only, "package", "after"),
    ]
    .into_iter()
    .filter(|(count, _, _)| *count > 0)
    .map(|(count, noun, side)| format!("{} only {side}", crate::output::counted(count, noun)))
    .collect();
    if !presence.is_empty() {
        writeln!(out, "\nOnly in one report: {}.", presence.join("; ")).unwrap();
    }
    let notes = notes::measurement_notes(c);
    if !notes.is_empty() {
        match markdown {
            true => writeln!(out, "\n## Measurement notes\n"),
            false => writeln!(out, "\n{}", theme.paint(Role::Heading, "Measurement notes")),
        }
        .unwrap();
        for (role, note) in notes {
            writeln!(out, "- {}", theme.paint(role, &label(&note, markdown))).unwrap();
        }
    }
    let findings: Vec<_> = c
        .findings
        .iter()
        .filter(|f| {
            !matches!(
                f,
                Finding::CoverageGap { .. } | Finding::CounterChanged { .. }
            )
        })
        .collect();
    if !findings.is_empty() {
        writeln!(out, "\n{}Findings", if markdown { "## " } else { "" }).unwrap();
        for f in findings {
            let context = f.context();
            writeln!(
                out,
                "\n- {} ({:?}{}): {}",
                theme.paint(Role::Information, f.code()),
                context.side,
                context
                    .member
                    .as_ref()
                    .map_or(String::new(), |m| format!(", {}", label(m, markdown))),
                label(&context.message, markdown)
            )
            .unwrap();
        }
    }
    if !c.findings.iter().any(|f| {
        matches!(
            f,
            Finding::CounterChanged { .. }
                | Finding::PolicyChanged { .. }
                | Finding::ModulesChanged { .. }
                | Finding::IdentityChanged { .. }
        )
    }) {
        writeln!(out, "\n{}", theme.paint(Role::Muted, "No recorded differences in measurement settings. Reports do not fingerprint source, ignore files or executable builds; this does not prove source-only growth.")).unwrap();
    }
    writeln!(
        out,
        "\nBefore measured at: {}\nAfter measured at: {}",
        crate::measurement_time(c.before.metadata.measured_at_unix_seconds),
        crate::measurement_time(c.after.metadata.measured_at_unix_seconds)
    )
    .unwrap();
    out
}
fn presence_name(p: Presence) -> &'static str {
    match p {
        Presence::Both => "both",
        Presence::BeforeOnly => "before only",
        Presence::AfterOnly => "after only",
    }
}
pub fn render_markdown(c: &Comparison) -> String {
    render(c, true, Theme::MARKDOWN)
}

pub fn render_table_with_theme(c: &Comparison, theme: Theme) -> String {
    render(c, false, theme)
}
