//! Purpose columns: per-row production, test and other code with their ratio and comment share.
use super::*;

/// Test code lines per production code line, `-` without production code.
pub(crate) fn test_ratio(production: u64, test: u64) -> String {
    match production {
        0 => "-".to_string(),
        _ => format!("{:.2}", test as f64 / production as f64),
    }
}

/// Code per purpose and comments of one table row: what its purpose columns,
/// ratio and bars describe.
#[derive(Clone, Copy, Default)]
pub(super) struct RowCode {
    pub(super) purposes: [u64; 3],
    pub(super) comments: u64,
}

impl RowCode {
    pub(super) fn of(purposes: &BTreeMap<Purpose, u64>, comments: u64) -> Self {
        Self {
            purposes: Purpose::ALL.map(|purpose| purposes.get(&purpose).copied().unwrap_or(0)),
            comments,
        }
    }

    pub(super) fn of_totals(totals: &Summary) -> Self {
        Self {
            purposes: Purpose::ALL.map(|purpose| totals.purpose(purpose).code),
            comments: totals.source.comments,
        }
    }

    pub(super) fn code(&self) -> u64 {
        self.purposes.iter().sum()
    }

    pub(super) fn purpose(&self, purpose: Purpose) -> u64 {
        self.purposes[purpose.index()]
    }

    /// Comments per line of code or comment, 0 without either.
    pub(super) fn density(&self) -> f64 {
        match self.code() + self.comments {
            0 => 0.0,
            lines => self.comments as f64 / lines as f64,
        }
    }
}

/// Comments per line of code or comment, as SonarQube's comment density;
/// `-` without either.
pub(super) fn comment_share(code: u64, comments: u64) -> String {
    share_of(comments, code + comments)
}

pub(super) const COMMENT_SHARE_TITLE: &str = "Comment %";

pub(super) const COMMENT_SHARE_DEFINITION: &str = "Comment % = Comments / (Code + Comments)";

pub(crate) fn kind_title(kind: Kind) -> &'static str {
    match kind {
        Kind::Source => "Source",
        Kind::Data => "Data",
        Kind::Documentation => "Documentation",
        Kind::Unknown => "Unknown",
    }
}

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
pub(super) struct PurposeColumns<'a> {
    pub(super) shown: &'a [Purpose],
}

/// Every purpose: the unfiltered view, and Markdown's.
pub(super) const ALL_PURPOSES: PurposeColumns<'static> = PurposeColumns {
    shown: &Purpose::ALL,
};

impl PurposeColumns<'_> {
    /// One purpose would only repeat Code, so it gets no column.
    pub(super) fn purposes(&self) -> &[Purpose] {
        match self.shown.len() {
            0 | 1 => &[],
            _ => self.shown,
        }
    }

    pub(super) fn ratio(&self) -> Ratio {
        match self.shown.contains(&Purpose::Production) && self.shown.contains(&Purpose::Test) {
            true => Ratio::TestPerProduction,
            false => Ratio::CommentShare,
        }
    }

    /// The purpose columns, as one group, and the ratio after them.
    pub(super) fn table_columns(&self) -> Vec<Column> {
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

    pub(super) fn ratio_column(&self) -> Column {
        let ratio = self.ratio();
        let title = match ratio {
            Ratio::TestPerProduction => "Test/prod",
            Ratio::CommentShare => COMMENT_SHARE_TITLE,
        };
        Column::new(title, ColumnRole::Ratio(ratio)).separated()
    }

    /// The cells of [`PurposeColumns::table_columns`] for `row`.
    pub(super) fn cells(&self, row: RowCode) -> Vec<String> {
        let mut cells: Vec<_> = self
            .purposes()
            .iter()
            .map(|purpose| grouped(row.purpose(*purpose)))
            .collect();
        cells.push(self.ratio_cell(row));
        cells
    }

    pub(super) fn ratio_cell(&self, row: RowCode) -> String {
        match self.ratio() {
            Ratio::TestPerProduction => {
                test_ratio(row.purpose(Purpose::Production), row.purpose(Purpose::Test))
            }
            Ratio::CommentShare => comment_share(row.code(), row.comments),
        }
    }

    /// Purpose bars need purposes to split; with one, bars chart comments,
    /// as the purposes' columns give way to Code alone.
    pub(super) fn bar_kind(&self, requested: BarKind) -> BarKind {
        match (requested, self.purposes().is_empty()) {
            (BarKind::Purpose, true) => BarKind::Comments,
            (kind, _) => kind,
        }
    }

    /// How the columns add up: `first` and `last` frame the purpose sum and
    /// the ratio's definition.
    pub(super) fn legend(&self, first: Option<&str>, last: Option<&str>) -> String {
        let sum = (self.purposes().len() > 1).then(|| {
            let titles: Vec<_> = self.purposes().iter().map(|p| purpose_title(*p)).collect();
            format!("{} = Code", titles.join(" + "))
        });
        let ratio = match self.ratio() {
            Ratio::TestPerProduction => "Test/prod = Test / Production (Code)",
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
        parts.join(" · ")
    }
}
