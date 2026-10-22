//! Share bars: the inline and standalone bar charts beside terminal tables.
use super::*;

/// Glyphs per purpose bar segment, and the comment bar's; distinct shapes keep
/// segments readable without color, as human presentation requires.
pub(super) const BAR_GLYPHS: [char; 3] = ['█', '▓', '░'];

pub(super) const COMMENT_GLYPH: char = '▓';

/// Bar cells: a readable minimum, and a maximum beyond which wider terminals
/// add length without adding legible detail.
pub(super) const BAR_CELLS: std::ops::RangeInclusive<usize> = 10..=40;

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
pub(super) struct BarScale {
    pub(super) code: u64,
    pub(super) density: f64,
}

impl BarScale {
    pub(super) fn of(rows: &[RowCode]) -> Self {
        Self {
            code: rows.iter().map(RowCode::code).max().unwrap_or(0),
            density: rows.iter().map(RowCode::density).fold(0.0, f64::max),
        }
    }
}

impl BarKind {
    /// The percentage column beside inline bars, set apart like a ratio.
    pub(super) fn column(self) -> Column {
        let column = match self {
            BarKind::Purpose => Column::new("Share", ColumnRole::Share),
            BarKind::Comments => {
                Column::new(COMMENT_SHARE_TITLE, ColumnRole::Ratio(Ratio::CommentShare))
            }
        };
        column.separated()
    }

    pub(super) fn definition(self) -> &'static str {
        match self {
            BarKind::Purpose => "Share = Code / total Code",
            BarKind::Comments => COMMENT_SHARE_DEFINITION,
        }
    }

    pub(super) fn scale_note(self) -> &'static str {
        match self {
            BarKind::Purpose => ". The largest row fills the bar.",
            BarKind::Comments => ". The densest row fills the bar.",
        }
    }

    /// Title of the bar column and of the chart below a table; it names what
    /// the bars mean, since a single purpose turns purpose bars to comments.
    pub(super) fn heading(self) -> &'static str {
        match self {
            BarKind::Purpose => "Code share",
            BarKind::Comments => "Comment density",
        }
    }

    /// Code share takes the Code hue; comment density stays plain.
    pub(super) fn heading_role(self) -> Role {
        match self {
            BarKind::Purpose => Role::Code,
            BarKind::Comments => Role::Heading,
        }
    }

    pub(super) fn percent(self, row: RowCode, total: RowCode) -> String {
        match self {
            BarKind::Purpose => share_of(row.code(), total.code()),
            BarKind::Comments => comment_share(row.code(), row.comments),
        }
    }

    /// Glyph, hue and name of each segment the bars can show; plain hues,
    /// never bold, keep bars quieter than the numbers.
    pub(super) fn segments(self, shown: &[Purpose]) -> Vec<(char, Role, &'static str)> {
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

    pub(super) fn key(self, shown: &[Purpose], theme: Theme) -> String {
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
pub(super) fn share_bars(
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
pub(super) fn share_of(part: u64, total: u64) -> String {
    match total {
        0 => "-".to_string(),
        _ => format!("{:.1}%", part as f64 * 100.0 / total as f64),
    }
}

/// One row's bar in `width` cells at `scale`; a row too small for one cell
/// gets a sliver in its largest segment's hue, a row without any count nothing.
pub(super) fn bar(
    kind: BarKind,
    row: RowCode,
    scale: BarScale,
    width: usize,
    theme: Theme,
) -> String {
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
pub(super) fn inline_bar_width(table_width: usize, title: &str, columns: usize) -> Option<usize> {
    let room = columns.checked_sub(table_width + 2)?;
    let minimum = (*BAR_CELLS.start()).max(title.chars().count());
    (room >= minimum).then(|| room.min(*BAR_CELLS.end()))
}

/// Each body row's code and the table total, the purposes counted, and where
/// the bars go.
pub(super) struct BarRows<'a> {
    pub(super) rows: &'a [RowCode],
    pub(super) total: RowCode,
    pub(super) columns: &'a PurposeColumns<'a>,
    pub(super) bars: Bars,
    /// Adds the key line that explains inline bars.
    pub(super) key: bool,
}

/// A table with bars: beside its rows when the minimum inline bar fits,
/// adding the bar kind's percentage column unless the table has it; otherwise
/// followed by a chart for [`Bars::Always`] and left as it is for
/// [`Bars::Fit`]. Returns the table and what follows its legends: the inline
/// bar key, when requested, or the chart.
pub(super) fn bar_table(
    table_columns: &Columns,
    body: &[Vec<String>],
    total: Option<Vec<String>>,
    bars: &BarRows,
    theme: Theme,
) -> (RenderedTable, String) {
    let BarRows {
        rows,
        total: total_row,
        columns: purposes,
        bars,
        key,
    } = *bars;
    let plain_table =
        |theme| PlainTable::new(table_columns, body, total.as_deref()).render(None, theme);
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
        if let Some(cells) = &mut inline_total {
            cells.push(kind.percent(total_row, total_row));
        }
    }
    let table = PlainTable::new(&inline_columns, &inline_body, inline_total.as_deref());
    let Some(cells) = inline_bar_width(table.width(), kind.heading(), columns) else {
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
    if !key {
        return (table.render(Some(&bars), theme), String::new());
    }
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
    (table.render(Some(&bars), theme), key)
}

/// Whole cells per segment: the row's length rounds its fraction of `scale`
/// across `width`, and largest remainders distribute it so the segments add up
/// exactly.
pub(super) fn bar_cells(segments: &[u64; 3], scale: u64, width: usize) -> [usize; 3] {
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

/// Bars for terminal tables, with the columns available to them.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bars {
    Off,
    /// Purpose bars beside each table that leaves room for them; omitted elsewhere.
    Fit(usize),
    /// Bars beside each table that leaves room for them; a chart below elsewhere.
    Always(usize, BarKind),
}
