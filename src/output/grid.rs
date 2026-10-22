//! Table grids shared by the terminal and Markdown renderers: columns, rows and plain-text layout.
use super::*;

/// What a measurement table column holds, which decides its styling in text
/// tables and its emphasis and alignment in Markdown.
///
/// The purpose columns split Code, while Comments and Blanks sit beside it, so
/// separators and an accented Code column show which numbers belong together.
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
    /// A signed change beside the primary Lines and Code changes.
    SecondaryChange,
}

/// One column of a measurement table.
#[derive(Clone, Copy)]
pub(crate) struct Column {
    title: &'static str,
    /// Heading for terminal tables when `title` would be wider than every
    /// value, so a long title does not widen a column of short figures.
    short: Option<&'static str>,
    /// Terminal group heading spanning neighbouring columns of the same group,
    /// and this column's heading beneath it; Markdown keeps `title`.
    group: Option<(&'static str, &'static str)>,
    pub(super) role: ColumnRole,
    /// Preceded by a `│` separator, starting a group of related figures.
    separated: bool,
}

impl Column {
    pub(crate) const fn new(title: &'static str, role: ColumnRole) -> Self {
        Self {
            title,
            short: None,
            group: None,
            role,
            separated: false,
        }
    }

    pub(crate) const fn under(self, group: &'static str, heading: &'static str) -> Self {
        Self {
            group: Some((group, heading)),
            ..self
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

    /// Terminal heading over values at most `widest` characters wide: the
    /// group's column heading, or the short title where the full one is wider.
    pub(super) fn heading(&self, widest: usize) -> &'static str {
        match (self.group, self.short) {
            (Some((_, heading)), _) => heading,
            (None, Some(short)) if self.title.chars().count() > widest => short,
            _ => self.title,
        }
    }
}

/// The full counts of a language, purpose or category row.
pub(super) const fn count_columns(label: &'static str) -> [Column; 6] {
    [
        Column::new(label, ColumnRole::Label),
        Column::new("Files", ColumnRole::Secondary),
        Column::new("Lines", ColumnRole::Count),
        Column::new("Code", ColumnRole::Code),
        Column::new("Comments", ColumnRole::Secondary).separated(),
        Column::new("Blanks", ColumnRole::Secondary),
    ]
}

/// The language table's columns before its purpose columns.
pub(super) const LANGUAGE_COLUMNS: [Column; 6] = count_columns("Language");

/// The columns of a folder or member table before its purpose columns.
pub(super) const fn code_columns(label: &'static str) -> [Column; 3] {
    [
        Column::new(label, ColumnRole::Label),
        Column::new("Lines", ColumnRole::Count),
        Column::new("Code", ColumnRole::Code),
    ]
}

/// A table's columns in order; its header, text layout and Markdown header
/// all follow from them, so they cannot disagree.
#[derive(Clone)]
pub(crate) struct Columns(pub(super) Vec<Column>);

impl Columns {
    pub(crate) fn of(columns: &[Column]) -> Self {
        Self(columns.to_vec())
    }

    /// `lead` followed by the purpose columns and ratio `purposes` shows.
    pub(super) fn with_purposes(lead: &[Column], purposes: &PurposeColumns) -> Self {
        Self(
            lead.iter()
                .copied()
                .chain(purposes.table_columns())
                .collect(),
        )
    }

    pub(super) fn has(&self, role: ColumnRole) -> bool {
        self.0.iter().any(|column| column.role == role)
    }

    /// Header and alignment rows; bold Code marks the column the purpose
    /// columns split, as the terminal's accent does.
    pub(super) fn markdown_header(&self) -> String {
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

#[derive(Clone, Copy)]
pub(super) enum RowKind {
    Header,
    Body,
    Total,
}

/// `columns` over `body` and an optional bold `total` row, as an aligned text
/// table or a Markdown table; Markdown cells must already be escaped. Every
/// table without bars, compare's grids included, renders through it, so all
/// share one table style.
pub(crate) fn shared_table(
    columns: &Columns,
    body: &[Vec<String>],
    total: Option<&[String]>,
    markdown: bool,
    theme: Theme,
) -> String {
    if !markdown {
        return PlainTable::new(columns, body, total)
            .render(None, theme)
            .text;
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
pub(super) struct InlineBars {
    pub(super) kind: BarKind,
    pub(super) cells: usize,
    pub(super) bars: Vec<String>,
}

/// Rows whose primary deltas repeat the source total, independent of label or numeric equality.
#[derive(Clone, Copy, Default)]
pub(crate) struct SourceTotalRows {
    pub body: Option<usize>,
    pub total: bool,
}

pub(crate) fn table_with_top_rule(
    columns: &Columns,
    body: &[Vec<String>],
    total: Option<&[String]>,
    source_totals: SourceTotalRows,
    theme: Theme,
) -> String {
    let mut table = PlainTable::new(columns, body, total);
    table.source_totals = source_totals;
    let table = table.render(None, theme);
    table.top_rule(theme) + &table.text
}

/// Keep geometry with styled text so top-rule widths exclude ANSI escapes.
pub(super) struct RenderedTable {
    pub(super) text: String,
    pub(super) width: usize,
}

impl RenderedTable {
    pub(super) fn top_rule(&self, theme: Theme) -> String {
        theme.paint(Role::Muted, &format!(" {}\n", "─".repeat(self.width - 1)))
    }
}

/// Aligned plain-text table: labels left-aligned, figures right. Padding
/// precedes styling so escape codes never shift alignment. Column names and
/// legends carry the grouping semantics without a redundant caption row.
pub(super) struct PlainTable {
    pub(super) columns: Vec<Column>,
    /// Cells without control characters.
    pub(super) header: Vec<String>,
    pub(super) body: Vec<Vec<String>>,
    pub(super) total: Option<Vec<String>>,
    pub(super) widths: Vec<usize>,
    pub(super) source_totals: SourceTotalRows,
}

impl PlainTable {
    /// Values set the widths first, so a column takes its short title only
    /// where the full one would be wider than every value; a group name then
    /// widens its span's last column when it would not fit.
    pub(super) fn new(columns: &Columns, body: &[Vec<String>], total: Option<&[String]>) -> Self {
        let clean_row = |row: &[String]| row.iter().map(|value| plain(value)).collect::<Vec<_>>();
        let body: Vec<_> = body.iter().map(|row| clean_row(row)).collect();
        let total = total.map(clean_row);
        let mut widths = vec![0; columns.0.len()];
        for row in body.iter().chain(total.as_ref()) {
            for (width, value) in widths.iter_mut().zip(row) {
                *width = (*width).max(value.chars().count());
            }
        }
        let header: Vec<String> = columns
            .0
            .iter()
            .zip(&widths)
            .map(|(column, widest)| plain(column.heading(*widest)))
            .collect();
        for (width, title) in widths.iter_mut().zip(&header) {
            *width = (*width).max(title.chars().count());
        }
        let mut table = Self {
            columns: columns.0.clone(),
            header,
            body,
            total,
            widths,
            source_totals: SourceTotalRows::default(),
        };
        for (start, end) in table.spans() {
            let needed = table.group(start).chars().count();
            let width = table.span_width(start, end);
            if needed > width {
                table.widths[end] += needed - width;
            }
        }
        table
    }

    pub(super) fn group(&self, column: usize) -> &'static str {
        self.columns[column].group.map_or("", |(group, _)| group)
    }

    /// Separator between `column` and the column before it.
    pub(super) fn gap(&self, column: usize) -> &'static str {
        match self.columns[column].separated {
            true => " │ ",
            false => "  ",
        }
    }

    /// Runs of neighbouring columns under one group, never across a `│`.
    pub(super) fn spans(&self) -> Vec<(usize, usize)> {
        let mut spans: Vec<(usize, usize)> = Vec::new();
        for (index, column) in self.columns.iter().enumerate() {
            let Some((group, _)) = column.group else {
                continue;
            };
            match spans.last_mut() {
                Some((start, end))
                    if *end + 1 == index && self.group(*start) == group && !column.separated =>
                {
                    *end = index
                }
                _ => spans.push((index, index)),
            }
        }
        spans
    }

    pub(super) fn span_width(&self, start: usize, end: usize) -> usize {
        let gaps: usize = (start + 1..=end)
            .map(|column| self.gap(column).chars().count())
            .sum();
        self.widths[start..=end].iter().sum::<usize>() + gaps
    }

    /// Whitespace keeps group labels subordinate to the section title.
    pub(super) fn group_line(&self, theme: Theme) -> Option<String> {
        let spans = self.spans();
        if spans.is_empty() {
            return None;
        }
        let mut pieces: Vec<(Option<Role>, String)> = Vec::new();
        let mut column = 0;
        while column < self.widths.len() {
            if column > 0 {
                pieces.push((Some(Role::Muted), self.gap(column).into()));
            }
            match spans.iter().find(|(start, _)| *start == column) {
                Some(&(start, end)) => {
                    let name = self.group(start);
                    let fill = self.span_width(start, end) - name.chars().count();
                    let cell = format!(
                        "{}{name}{}",
                        " ".repeat(fill / 2),
                        " ".repeat(fill - fill / 2)
                    );
                    pieces.push((Some(Role::Heading), cell));
                    column = end + 1;
                }
                None => {
                    pieces.push((None, " ".repeat(self.widths[column])));
                    column += 1;
                }
            }
        }
        // Padding is trimmed before styling, which would otherwise enclose it.
        while let Some((_, piece)) = pieces.last_mut() {
            piece.truncate(piece.trim_end().len());
            match piece.is_empty() {
                true => pieces.pop(),
                false => break,
            };
        }
        let text: String = pieces
            .iter()
            .map(|(role, piece)| match role {
                Some(role) => theme.paint(*role, piece),
                None => piece.clone(),
            })
            .collect();
        Some(format!(" {text}\n"))
    }

    /// Characters per line without bars, as [`PlainTable::render`] lays it out.
    pub(super) fn width(&self) -> usize {
        1 + self.span_width(0, self.widths.len() - 1)
    }

    pub(super) fn render(&self, inline: Option<&InlineBars>, theme: Theme) -> RenderedTable {
        let line = |row: &[String], kind: RowKind, source_total: bool, trailer: Option<&str>| {
            let mut text = String::from(" ");
            for (index, (value, width)) in row.iter().zip(&self.widths).enumerate() {
                if index > 0 {
                    text += &theme.paint(Role::Muted, self.gap(index));
                }
                let column = self.columns[index].role;
                let cell = match column {
                    ColumnRole::Label => format!("{value:<width$}"),
                    _ => format!("{value:>width$}"),
                };
                let emphasized = !matches!(kind, RowKind::Body);
                let zero = matches!(value.as_str(), "-" | "0" | "0.00" | "0.0%");
                let (role, strong) = match (column, kind) {
                    (ColumnRole::Label, _) => (Some(Role::Heading), false),
                    _ if zero => (Some(Role::Muted), false),
                    (ColumnRole::Purpose(purpose), _) => (Some(Role::Purpose(purpose)), emphasized),
                    (ColumnRole::Code, _) => (Some(Role::Code), false),
                    (_, RowKind::Header) => (Some(Role::Heading), false),
                    (ColumnRole::Change, _) => {
                        let direction = match (source_total, value.starts_with('-')) {
                            (true, true) => Role::SourceTotalDecrease,
                            (true, false) => Role::SourceTotalIncrease,
                            (false, true) => Role::Decrease,
                            (false, false) => Role::Increase,
                        };
                        (Some(direction), emphasized || source_total)
                    }
                    (ColumnRole::SecondaryChange, _) => {
                        let direction = match value.starts_with('-') {
                            true => Role::SecondaryDecrease,
                            false => Role::SecondaryIncrease,
                        };
                        (Some(direction), false)
                    }
                    (ColumnRole::Secondary | ColumnRole::Ratio(_) | ColumnRole::Share, _) => {
                        (Some(Role::Secondary), false)
                    }
                    (ColumnRole::Count, RowKind::Total) => (Some(Role::Heading), false),
                    (ColumnRole::Count, RowKind::Body) => (None, false),
                };
                text += &match role {
                    Some(role) if strong => theme.strong(role, &cell),
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
                rule += match self.columns[column].separated {
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
        let mut output = self.group_line(theme).unwrap_or_default()
            + &line(&self.header, RowKind::Header, false, title.as_deref())
            + &rule;
        for (index, row) in self.body.iter().enumerate() {
            let bar = inline
                .and_then(|bars| bars.bars.get(index))
                .filter(|bar| !bar.is_empty());
            output += &line(
                row,
                RowKind::Body,
                self.source_totals.body == Some(index),
                bar.map(String::as_str),
            );
        }
        if let Some(total) = &self.total {
            output += &rule;
            output += &line(total, RowKind::Total, self.source_totals.total, None);
        }
        RenderedTable {
            text: output,
            width: self.width() + inline.map_or(0, |bars| 2 + bars.cells),
        }
    }
}

/// Widest cell of each column, in characters, for aligning note rows.
pub(super) fn column_widths<const N: usize>(rows: &[[String; N]]) -> [usize; N] {
    std::array::from_fn(|column| {
        rows.iter()
            .map(|row| row[column].chars().count())
            .max()
            .unwrap_or(0)
    })
}
