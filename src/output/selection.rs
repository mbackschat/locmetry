//! Which source a table view counts: purpose, category and language
//! selections and the folding of language variants into their family's row.

use super::{counted, listed, plural, purpose_title};
use crate::{Counts, FileRecord, Kind, Purpose, ReportSummary};
use std::collections::BTreeMap;

/// Measured source languages that `languages` leaves out of `rows`, largest
/// code first, with that code; none without a selection.
fn excluded_languages<'r>(
    rows: impl Iterator<Item = &'r FileRecord>,
    languages: &[String],
) -> Vec<(String, u64)> {
    if languages.is_empty() {
        return Vec::new();
    }
    let mut excluded = BTreeMap::<&str, u64>::new();
    for row in rows.filter(|row| row.kind == Kind::Source) {
        for (language, counts) in &row.languages {
            if !languages.contains(language) {
                *excluded.entry(language).or_default() += counts.code;
            }
        }
    }
    let mut excluded: Vec<(String, u64)> = excluded
        .into_iter()
        .map(|(language, code)| (language.to_string(), code))
        .collect();
    excluded.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    excluded
}

fn purpose_selected(row: &FileRecord, purposes: &[Purpose]) -> bool {
    row.kind != Kind::Source || purposes.contains(&row.classification.purpose)
}

/// One `--table-category` entry: a purpose, which selects all its categories,
/// or a `purpose:category` key as the category table prints it. `*` in either
/// part matches any run of characters (docs/proposals/archive/table-category-filter.md).
#[derive(Clone, Debug)]
pub struct CategoryPattern {
    purpose: String,
    category: Option<String>,
}

impl CategoryPattern {
    /// Rejects a purpose part that can match no purpose, since that would
    /// silently empty every table; categories are the project's own names.
    pub fn parse(text: &str) -> Result<Self, String> {
        let (purpose, category) = match text.split_once(':') {
            Some((_, "")) => return Err("A category name must follow ':'.".into()),
            Some((purpose, category)) => (purpose, Some(category.to_string())),
            None => (text, None),
        };
        if !purpose.contains('*') && !Purpose::ALL.iter().any(|p| p.to_string() == purpose) {
            return Err(format!(
                "Unknown purpose '{purpose}'. Use 'production', 'test' or 'other', alone or before ':CATEGORY'."
            ));
        }
        Ok(Self {
            purpose: purpose.to_string(),
            category,
        })
    }

    /// Whether this entry can select source of `purpose` at all.
    pub(super) fn names(&self, purpose: Purpose) -> bool {
        wildcard(&self.purpose, &purpose.to_string())
    }

    /// Whether any source in `rows` falls under this entry; category names
    /// are the project's own and case-sensitive, so a miss is worth naming.
    pub fn matches_any(&self, rows: &[FileRecord]) -> bool {
        rows.iter()
            .any(|row| row.kind == Kind::Source && self.matches(row))
    }

    fn matches(&self, row: &FileRecord) -> bool {
        self.names(row.classification.purpose)
            && self
                .category
                .as_deref()
                .is_none_or(|category| wildcard(category, &row.classification.category))
    }
}

impl std::fmt::Display for CategoryPattern {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.category {
            Some(category) => write!(f, "{}:{category}", self.purpose),
            None => f.write_str(&self.purpose),
        }
    }
}

/// Whether `pattern`, whose `*` matches any run of characters, covers all of `text`.
fn wildcard(pattern: &str, text: &str) -> bool {
    let mut parts = pattern.split('*');
    let Some(mut rest) = text.strip_prefix(parts.next().unwrap_or_default()) else {
        return false;
    };
    let parts: Vec<&str> = parts.collect();
    let Some((last, middle)) = parts.split_last() else {
        return rest.is_empty();
    };
    for part in middle {
        match rest.find(part) {
            Some(at) => rest = &rest[at + part.len()..],
            None => return false,
        }
    }
    rest.len() >= last.len() && rest.ends_with(last)
}

fn category_selected(row: &FileRecord, categories: &[CategoryPattern]) -> bool {
    row.kind != Kind::Source || categories.iter().any(|pattern| pattern.matches(row))
}

/// `row` reduced to the selected `languages`: an embedded language keeps its
/// own lines, and only the file's own language counts the file, as in the
/// unfiltered language table. Source without a selected language drops out.
fn language_selected(row: &FileRecord, languages: &[String]) -> Option<FileRecord> {
    if row.kind != Kind::Source {
        return Some(row.clone());
    }
    let kept: BTreeMap<String, Counts> = row
        .languages
        .iter()
        .filter(|(language, _)| languages.contains(language))
        .map(|(language, counts)| (language.clone(), *counts))
        .collect();
    if kept.is_empty() {
        return None;
    }
    let mut counts = Counts::default();
    for language in kept.values() {
        counts.add(language);
    }
    counts.files = u64::from(kept.contains_key(&row.language));
    Some(FileRecord {
        counts,
        languages: kept,
        ..row.clone()
    })
}

/// A variant shown within its family's row, such as (`TSX`, `TypeScript`).
pub type Fold = (String, &'static str);

/// How table and Markdown views show language variants such as TSX: within
/// their family's row, such as TypeScript, or as rows of their own. Counting
/// and JSON keep the backend's names either way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LanguageVariants {
    Folded,
    Split,
}

impl LanguageVariants {
    /// The row a source language appears in.
    pub fn row_name(self, language: &str) -> String {
        match (self, crate::language_family(language)) {
            (Self::Folded, Some(family)) => family.into(),
            _ => language.into(),
        }
    }

    /// `rows` and their summary as these views count them: a variant's lines
    /// join its family's entry, and a variant file counts as a file of its
    /// family. `None` when nothing folds, so the view keeps the measured ones.
    pub(super) fn folded(self, rows: &[FileRecord]) -> Option<(Vec<FileRecord>, ReportSummary)> {
        let variant = |language: &String| crate::language_family(language).is_some();
        if self == Self::Split || !rows.iter().any(|row| row.languages.keys().any(variant)) {
            return None;
        }
        let rows: Vec<FileRecord> = rows
            .iter()
            .map(|row| {
                let mut languages = BTreeMap::<String, Counts>::new();
                for (language, counts) in &row.languages {
                    let entry = languages.entry(self.row_name(language)).or_default();
                    entry.add(counts);
                    // Each entry records its file once, as the backend does.
                    entry.files = 1;
                }
                FileRecord {
                    language: self.row_name(&row.language),
                    languages,
                    ..row.clone()
                }
            })
            .collect();
        let summary = crate::build_report(&rows);
        Some((rows, summary))
    }

    /// The folds among `names`, ordered by family and then variant.
    pub fn folds<'n>(self, names: impl IntoIterator<Item = &'n str>) -> Vec<Fold> {
        if self == Self::Split {
            return Vec::new();
        }
        let folds: std::collections::BTreeSet<(&str, &str)> = names
            .into_iter()
            .filter_map(|name| crate::language_family(name).map(|family| (family, name)))
            .collect();
        folds
            .into_iter()
            .map(|(family, variant)| (variant.to_string(), family))
            .collect()
    }

    /// The folds in source that `purposes`, `categories` and the `languages`
    /// row names select; an empty `categories` or `languages` selects all.
    pub(super) fn source_folds(
        self,
        rows: &[FileRecord],
        purposes: &[Purpose],
        categories: &[CategoryPattern],
        languages: &[String],
    ) -> Vec<Fold> {
        let names = rows
            .iter()
            .filter(|row| row.kind == Kind::Source && purpose_selected(row, purposes))
            .filter(|row| categories.is_empty() || category_selected(row, categories))
            .flat_map(|row| row.languages.keys().map(String::as_str))
            .filter(|name| {
                let row = crate::language_family(name).unwrap_or(name);
                languages.is_empty() || languages.iter().any(|language| language == row)
            });
        self.folds(names)
    }
}

/// The muted line naming the folds a view applied; `None` without any.
pub(crate) fn folded_note(folds: &[Fold]) -> Option<String> {
    let folds: Vec<String> = folds
        .iter()
        .map(|(variant, family)| format!("{variant} → {family}"))
        .collect();
    (!folds.is_empty()).then(|| format!("Folded: {}.", folds.join(", ")))
}

/// The source a table selection keeps and what it leaves out. Purposes filter
/// first, categories next and languages last, so the code the notes exclude
/// and the shown total add up to the full measurement.
pub(super) struct TableSelection {
    /// Selected records; `None` when the selection keeps everything.
    pub(super) rows: Option<Vec<FileRecord>>,
    /// Source code a strict purpose subset leaves out.
    pub(super) purposes_excluded: Option<u64>,
    /// Category keys left out with their code, largest first.
    pub(super) categories_excluded: Vec<(String, u64)>,
    /// Whether the categories keep only part of Production's or Test's code,
    /// so Test/prod would compare unrelated slices of the two.
    pub(super) sliced: bool,
    pub(super) languages_excluded: Vec<(String, u64)>,
}

impl TableSelection {
    pub(super) fn of(
        rows: &[FileRecord],
        purposes: &[Purpose],
        categories: &[CategoryPattern],
        languages: &[String],
    ) -> Self {
        let source_code = |rows: &[FileRecord]| -> u64 {
            rows.iter()
                .filter(|row| row.kind == Kind::Source)
                .map(|row| row.counts.code)
                .sum()
        };
        let mut selected = None;
        let mut purposes_excluded = None;
        if purposes.len() < Purpose::ALL.len() {
            let kept: Vec<FileRecord> = rows
                .iter()
                .filter(|row| purpose_selected(row, purposes))
                .cloned()
                .collect();
            purposes_excluded = Some(source_code(rows) - source_code(&kept));
            selected = Some(kept);
        }
        let current = selected.as_deref().unwrap_or(rows);
        let (categories_excluded, sliced) = excluded_categories(current, categories);
        if !categories_excluded.is_empty() {
            selected = Some(
                current
                    .iter()
                    .filter(|row| category_selected(row, categories))
                    .cloned()
                    .collect(),
            );
        }
        let current = selected.as_deref().unwrap_or(rows);
        let languages_excluded = excluded_languages(current.iter(), languages);
        if !languages_excluded.is_empty() {
            selected = Some(
                current
                    .iter()
                    .filter_map(|row| language_selected(row, languages))
                    .collect(),
            );
        }
        Self {
            rows: selected,
            purposes_excluded,
            categories_excluded,
            sliced,
            languages_excluded,
        }
    }

    /// Names what the tables leave out, so filtered totals are not read as
    /// complete ones. Purpose titles keep "Other" from reading as "others".
    pub(super) fn notes(&self, purposes: &[Purpose]) -> Vec<String> {
        // The command and the table already name what is shown, so a note
        // names only what is left out, under a subtle label.
        let note = |names: Vec<String>, noun: &str, code: u64| {
            format!(
                "Not shown ({}): {} ({}).",
                plural(names.len() as u64, noun),
                listed(names),
                counted(code, "code line")
            )
        };
        let mut notes = Vec::new();
        if let Some(code) = self.purposes_excluded {
            let excluded = Purpose::ALL
                .into_iter()
                .filter(|purpose| !purposes.contains(purpose))
                .map(|purpose| purpose_title(purpose).to_string())
                .collect();
            notes.push(note(excluded, "purpose", code));
        }
        for (excluded, noun) in [
            (&self.categories_excluded, "category"),
            (&self.languages_excluded, "language"),
        ] {
            if !excluded.is_empty() {
                let code = excluded.iter().map(|(_, code)| code).sum();
                notes.push(note(largest_names(excluded), noun, code));
            }
        }
        notes
    }
}

/// The largest names name the bulk of what is missing; a long tail only
/// needs its size, and a single remaining name is no longer than counting it.
fn largest_names(excluded: &[(String, u64)]) -> Vec<String> {
    const NAMED: usize = 5;
    let named = match excluded.len() > NAMED + 1 {
        true => NAMED,
        false => excluded.len(),
    };
    let mut names: Vec<String> = excluded
        .iter()
        .take(named)
        .map(|(name, _)| name.clone())
        .collect();
    if excluded.len() > named {
        names.push(format!("{} more", excluded.len() - named));
    }
    names
}

/// Category keys `categories` leaves out of `rows`, largest code first, and
/// whether Production or Test keeps only part of its code; none without a
/// selection.
fn excluded_categories(
    rows: &[FileRecord],
    categories: &[CategoryPattern],
) -> (Vec<(String, u64)>, bool) {
    if categories.is_empty() {
        return (Vec::new(), false);
    }
    let mut excluded = BTreeMap::<String, u64>::new();
    let mut kept = [0u64; 3];
    let mut left = [0u64; 3];
    for row in rows.iter().filter(|row| row.kind == Kind::Source) {
        let purpose = row.classification.purpose;
        if category_selected(row, categories) {
            kept[purpose.index()] += row.counts.code;
        } else {
            left[purpose.index()] += row.counts.code;
            let key = crate::report::category_key(purpose, &row.classification.category);
            *excluded.entry(key).or_default() += row.counts.code;
        }
    }
    let sliced = [Purpose::Production, Purpose::Test]
        .iter()
        .any(|purpose| kept[purpose.index()] > 0 && left[purpose.index()] > 0);
    let mut excluded: Vec<(String, u64)> = excluded.into_iter().collect();
    excluded.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    (excluded, sliced)
}
