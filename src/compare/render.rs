use super::*;
use crate::output::{self, Column, ColumnRole, Columns};
use crate::report::kind_purpose_key;
use crate::style::{Role, Theme};
use crate::{Counts, Kind, Purpose};
use std::fmt::Write;

mod notes;

fn label(value: &str, markdown: bool) -> String {
    match markdown {
        true => crate::output::cell(value),
        false => crate::style::plain(value),
    }
}
/// One figure of a comparison row.
type Cell = fn(&Row) -> String;

/// The signed change from `before` to `after`, computed wide so a row summed
/// for display cannot overflow.
fn change(before: u64, after: u64) -> String {
    output::signed(i128::from(after) - i128::from(before))
}

/// Lines and Code before, after and net change, as every comparison grid shows them.
const CHANGE_COLUMNS: [(Column, Cell); 6] = [
    (
        Column::new("Lines before", ColumnRole::Count).under("Lines", "before"),
        |row| output::grouped(row.before.lines),
    ),
    (
        Column::new("Lines after", ColumnRole::Count).under("Lines", "after"),
        |row| output::grouped(row.after.lines),
    ),
    (
        Column::new("Lines change", ColumnRole::Change).under("Lines", "change"),
        |row| change(row.before.lines, row.after.lines),
    ),
    (
        Column::new("Code before", ColumnRole::Count)
            .under("Code", "before")
            .separated(),
        |row| output::grouped(row.before.code),
    ),
    (
        Column::new("Code after", ColumnRole::Count).under("Code", "after"),
        |row| output::grouped(row.after.code),
    ),
    (
        Column::new("Code change", ColumnRole::Change).under("Code", "change"),
        |row| change(row.before.code, row.after.code),
    ),
];
/// Secondary deltas retain direction, unlike the unsigned measurement figures.
const SECONDARY_CHANGE_COLUMNS: [(Column, Cell); 3] = [
    (
        Column::new("Comments change", ColumnRole::SecondaryChange)
            .under("Change", "Comments")
            .separated(),
        |row| change(row.before.comments, row.after.comments),
    ),
    (
        Column::new("Blanks change", ColumnRole::SecondaryChange).under("Change", "Blanks"),
        |row| change(row.before.blanks, row.after.blanks),
    ),
    (
        Column::new("Files change", ColumnRole::SecondaryChange).under("Change", "Files"),
        |row| change(row.before.files, row.after.files),
    ),
];

/// One row of a comparison grid.
struct Row {
    label: String,
    before: Counts,
    after: Counts,
    source_total: bool,
}

impl Row {
    fn new(label: impl Into<String>, change: &CountChange) -> Self {
        Self {
            label: label.into(),
            before: change.before,
            after: change.after,
            source_total: false,
        }
    }

    fn source_total(change: &CountChange) -> Self {
        Self {
            source_total: true,
            ..Self::new("Source total", change)
        }
    }
}

/// A comparison grid; [`Grid::write`] writes nothing without body
/// rows, so a grid never shows only a total.
struct Grid {
    title: &'static str,
    /// Heading of the label column.
    label: &'static str,
    rows: Vec<Row>,
    total: Option<Row>,
    /// Adds [`SECONDARY_CHANGE_COLUMNS`].
    secondary: bool,
    legend: Option<String>,
}

impl Grid {
    fn write(self, output: &mut String, markdown: bool, theme: Theme) {
        if self.rows.is_empty() {
            return;
        }
        let figures: Vec<(Column, Cell)> = CHANGE_COLUMNS
            .into_iter()
            .chain(
                self.secondary
                    .then_some(SECONDARY_CHANGE_COLUMNS)
                    .into_iter()
                    .flatten(),
            )
            .collect();
        let columns: Vec<Column> = std::iter::once(Column::new(self.label, ColumnRole::Label))
            .chain(figures.iter().map(|(column, _)| *column))
            .collect();
        let cells = |row: &Row| {
            std::iter::once(label(&row.label, markdown))
                .chain(figures.iter().map(|(_, cell)| cell(row)))
                .collect::<Vec<_>>()
        };
        let body: Vec<_> = self.rows.iter().map(cells).collect();
        let total = self.total.as_ref().map(cells);
        match markdown {
            true => writeln!(output, "\n## {}\n", self.title),
            false => writeln!(output),
        }
        .unwrap();
        let columns = Columns::of(&columns);
        output.push_str(&if !markdown {
            output::table_with_top_rule(
                &columns,
                &body,
                total.as_deref(),
                output::SourceTotalRows {
                    body: self.rows.iter().position(|row| row.source_total),
                    total: self.total.as_ref().is_some_and(|row| row.source_total),
                },
                theme,
            )
        } else {
            output::shared_table(&columns, &body, total.as_deref(), markdown, theme)
        });
        if let Some(legend) = self.legend {
            writeln!(
                output,
                "{}{}",
                if markdown { "\n" } else { " " },
                theme.paint(Role::Muted, &legend)
            )
            .unwrap();
        }
    }
}

/// Changes per content kind present in either report; together they make up
/// All counted, which validation guarantees `by_kind_purpose` sums to.
fn kind_rows(dimensions: &Dimensions) -> Vec<Row> {
    Kind::ALL
        .into_iter()
        .filter_map(|kind| {
            let changes: Vec<_> = Purpose::ALL
                .iter()
                .filter_map(|purpose| {
                    dimensions
                        .by_kind_purpose
                        .get(&kind_purpose_key(kind, *purpose))
                })
                .collect();
            let sum = |side: fn(&CountChange) -> Counts| {
                changes.iter().fold(Counts::default(), |sum, change| {
                    sum.checked_add(&side(change))
                        .expect("kind totals are bounded by all_counted")
                })
            };
            (!changes.is_empty()).then(|| Row {
                label: output::kind_title(kind).to_string(),
                before: sum(|change| change.before),
                after: sum(|change| change.after),
                source_total: kind == Kind::Source,
            })
        })
        .collect()
}

fn purpose_ratio(dimensions: &Dimensions) -> String {
    let ratio = |side: fn(&CountChange) -> Counts| {
        let code = |purpose| {
            dimensions
                .source_purpose
                .get(&purpose)
                .map_or(0, |change| side(change).code)
        };
        output::test_ratio(code(Purpose::Production), code(Purpose::Test))
    };
    format!(
        "Test/prod (Code): {} → {} (before → after)",
        ratio(|change| change.before),
        ratio(|change| change.after)
    )
}

fn counts_unchanged(c: &Comparison) -> bool {
    std::iter::once(&c.summary.dimensions)
        .chain(
            c.members
                .iter()
                .flat_map(|members| members.values().map(|m| &m.dimensions)),
        )
        .all(|d| {
            [&d.all_counted, &d.source]
                .into_iter()
                .chain(d.by_kind_purpose.values())
                .chain(d.source_purpose.values())
                .chain(d.source_categories.values())
                .chain(d.source_languages.values())
                .all(|change| change.delta == SignedCounts::default())
        })
        && c.breakdown
            .iter()
            .flatten()
            .all(|row| row.counts.delta == SignedCounts::default())
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
    if let Some(member) = &c.member {
        let presence = c
            .members
            .as_ref()
            .and_then(|members| members.get(member))
            .map(|member| presence_name(member.presence))
            .unwrap_or("not recorded");
        writeln!(
            out,
            "Member: {} ({presence}); counts and detail cover this member only.",
            label(member, markdown)
        )
        .unwrap();
        writeln!(out, "Input metadata, measurement notes, report-level findings and partial status cover the whole workspace inputs.").unwrap();
    }
    out.push('\n');
    for note in [
        "Net change = after − before, from the two saved reports.",
        "Unreadable files count as 0; a folder seen in only one report may be newly visible, not newly created.",
    ] {
        writeln!(out, "{}", theme.paint(Role::Muted, note)).unwrap();
    }
    let dimensions = &c.summary.dimensions;
    let source_total = || Some(Row::source_total(&dimensions.source));
    let changed = |change: &CountChange| change.delta.code != 0 || change.delta.lines != 0;
    let mut grids = vec![
        Grid {
            title: "Totals",
            label: "Kind",
            rows: kind_rows(dimensions),
            total: Some(Row::new("All counted", &dimensions.all_counted)),
            secondary: true,
            legend: None,
        },
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
            legend: Some(purpose_ratio(dimensions)),
        },
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
            legend: None,
        },
    ];
    if let Some(members) = c.members.as_ref().filter(|_| c.member.is_none()) {
        grids.push(Grid {
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
            legend: None,
        });
    }
    let folder_grid = c.folders.as_ref().map(|folders| {
        // The selected root bounds the table total; Markdown retains its full hierarchy.
        let root = c.member.as_deref().unwrap_or(".");
        let total = folders
            .get(root)
            .filter(|_| !markdown)
            .map(|root| Row::source_total(&root.recursive.source));
        let mut folders: Vec<_> = folders.iter().collect();
        folders.sort_by(|(a, _), (b, _)| a.split('/').cmp(b.split('/')));
        let rows: Vec<_> = folders
            .into_iter()
            .filter(|(path, v)| {
                (markdown
                    || ((c.members.is_none() || c.member.is_some())
                        && path.as_str() != root
                        && crate::paths::below(path, root)
                            .is_some_and(|relative| !relative.contains('/'))))
                    && (v.presence != Presence::Both || changed(&v.recursive.source))
            })
            .map(|(k, v)| {
                let path = if !markdown {
                    crate::paths::below(k, root).unwrap_or(k)
                } else {
                    k
                };
                Row::new(
                    format!("{path} ({})", presence_name(v.presence)),
                    &v.recursive.source,
                )
            })
            .collect();
        Grid {
            title: "Folders (recursive; do not sum parents and children)",
            label: "Folder",
            rows,
            total,
            secondary: false,
            legend: (!markdown)
                .then(|| "Folders are recursive; do not sum parents and children.".into()),
        }
    });
    for grid in grids {
        grid.write(&mut out, markdown, theme);
    }
    if let Some(composition) = c.summary.composition.as_ref().filter(|composition| {
        [&composition.source, &composition.all_counted]
            .into_iter()
            .any(|part| {
                part.matched.lines != 0
                    || part.matched.code != 0
                    || part.entering.lines != 0
                    || part.entering.code != 0
                    || part.leaving.lines != 0
                    || part.leaving.code != 0
            })
    }) {
        let scope = if c.member.is_some() {
            "Member"
        } else {
            "Workspace"
        };
        let theme = if markdown { Theme::PLAIN } else { theme };
        let signed = |value: i128| {
            let role = match value.cmp(&0) {
                std::cmp::Ordering::Less => Role::Decrease,
                std::cmp::Ordering::Equal => Role::Muted,
                std::cmp::Ordering::Greater => Role::Increase,
            };
            theme.paint(role, &format!("{value:+}"))
        };
        let leaving = |value: u64| match value {
            0 => theme.paint(Role::Muted, "-0"),
            _ => signed(-i128::from(value)),
        };
        writeln!(
            out,
            "\n{scope} source lines: matched {}, entering {}, leaving {}; total {}.",
            signed(composition.source.matched.lines.into()),
            signed(composition.source.entering.lines.into()),
            leaving(composition.source.leaving.lines),
            signed(c.summary.dimensions.source.delta.lines.into())
        )
        .unwrap();
        writeln!(
            out,
            "{scope} all-counted lines: matched {}, entering {}, leaving {}; total {}.",
            signed(composition.all_counted.matched.lines.into()),
            signed(composition.all_counted.entering.lines.into()),
            leaving(composition.all_counted.leaving.lines),
            signed(c.summary.dimensions.all_counted.delta.lines.into())
        )
        .unwrap();
        writeln!(
            out,
            "\n{scope} source code: matched {}, entering {}, leaving {}; total {}.",
            signed(composition.source.matched.code.into()),
            signed(composition.source.entering.code.into()),
            leaving(composition.source.leaving.code),
            signed(c.summary.dimensions.source.delta.code.into())
        )
        .unwrap();
        writeln!(
            out,
            "{scope} all-counted code: matched {}, entering {}, leaving {}; total {}.",
            signed(composition.all_counted.matched.code.into()),
            signed(composition.all_counted.entering.code.into()),
            leaving(composition.all_counted.leaving.code),
            signed(c.summary.dimensions.all_counted.delta.code.into())
        )
        .unwrap();
    }
    if let Some(grid) = folder_grid {
        grid.write(&mut out, markdown, theme);
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
    } else if c.members.is_some() && counts_unchanged(c) {
        writeln!(out, "\nNo recorded count or presence changes.").unwrap();
    }
    let notes = notes::measurement_notes(c);
    if !notes.is_empty() {
        match markdown {
            true => writeln!(out, "\n## Measurement notes\n"),
            false => write!(
                out,
                "\n{}",
                output::section_title("Measurement notes", theme)
            ),
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
        match markdown {
            true => writeln!(out, "\n## Findings"),
            false => write!(out, "\n{}", output::section_title("Findings", theme)),
        }
        .unwrap();
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
