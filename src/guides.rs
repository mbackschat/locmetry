use clap::ValueEnum;

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Guide {
    /// Find the section that answers a task.
    Index,
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
    /// Exit status, environment, files, man page and completions.
    Reference,
}

impl Guide {
    pub(crate) fn document(self) -> (&'static str, &'static str) {
        match self {
            Self::Index => (
                "docs/guides/README.md",
                include_str!("../docs/guides/README.md"),
            ),
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
            Self::Reference => (
                "docs/guides/reference.md",
                include_str!("../docs/guides/reference.md"),
            ),
        }
    }
}

/// A section heading: its level, its id and where its line starts.
struct Heading {
    level: usize,
    id: String,
    start: usize,
}

/// GitHub's heading slug, which repository links already use as fragments:
/// lowercase, without characters other than letters, digits, spaces, hyphens
/// and underscores, spaces as hyphens. `anchors()` in scripts/doc-indexes.py
/// computes the same, and the published-id list keeps the two in agreement.
fn slug(heading: &str) -> String {
    heading
        .to_lowercase()
        .chars()
        .filter(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | ' '))
        .map(|c| if c == ' ' { '-' } else { c })
        .collect()
}

/// The `##` and `###` headings of a guide. Lines in fenced code blocks and HTML
/// comments are never headings: compare.md's shell examples hold `# ` comments.
fn headings(text: &str) -> Vec<Heading> {
    let mut found = Vec::new();
    let (mut fence, mut comment, mut offset) = (None::<&str>, false, 0);
    for line in text.split_inclusive('\n') {
        let (start, trimmed) = (offset, line.trim_end());
        offset += line.len();
        let opening = trimmed.trim_start();
        if comment {
            comment = !trimmed.contains("-->");
        } else if let Some(marker) = fence {
            fence = (!opening.starts_with(marker)).then_some(marker);
        } else if opening.starts_with("```") || opening.starts_with("~~~") {
            fence = Some(&opening[..3]);
        } else if opening.starts_with("<!--") {
            comment = !trimmed.contains("-->");
        } else if let Some((level, heading)) = trimmed
            .strip_prefix("### ")
            .map(|heading| (3, heading))
            .or_else(|| trimmed.strip_prefix("## ").map(|heading| (2, heading)))
        {
            found.push(Heading {
                level,
                id: slug(heading),
                start,
            });
        }
    }
    found
}

/// A guide's section ids in document order.
pub(crate) fn section_ids(text: &str) -> Vec<String> {
    headings(text)
        .into_iter()
        .map(|heading| heading.id)
        .collect()
}

/// One section of a guide: its heading through the line before the next
/// heading of the same or a higher level, so it includes its subsections.
pub(crate) fn section<'a>(text: &'a str, id: &str) -> Option<&'a str> {
    let all = headings(text);
    let index = all.iter().position(|heading| heading.id == id)?;
    let level = all[index].level;
    let end = all[index + 1..]
        .iter()
        .find(|heading| heading.level <= level)
        .map_or(text.len(), |heading| heading.start);
    Some(&text[all[index].start..end])
}
