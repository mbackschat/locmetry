use clap::ValueEnum;

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Guide {
    /// Create or evaluate project configuration with evidence and open questions.
    Configure,
    /// Diagnose missing files, unexpected classifications and subfolder scope.
    Selection,
    /// Interpret counts, coverage, output formats and exports.
    Reports,
    /// Capture full baselines and interpret saved-report net change.
    Compare,
    /// Reference category names, worked cases and naming conventions.
    Categories,
}

impl Guide {
    pub(crate) fn document(self) -> (&'static str, &'static str) {
        match self {
            Self::Configure => (
                "docs/guides/configure.md",
                include_str!("../docs/guides/configure.md"),
            ),
            Self::Selection => (
                "docs/guides/selection.md",
                include_str!("../docs/guides/selection.md"),
            ),
            Self::Reports => (
                "docs/guides/reports.md",
                include_str!("../docs/guides/reports.md"),
            ),
            Self::Compare => (
                "docs/guides/compare.md",
                include_str!("../docs/guides/compare.md"),
            ),
            Self::Categories => (
                "docs/guides/categories.md",
                include_str!("../docs/guides/categories.md"),
            ),
        }
    }
}
