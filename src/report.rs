use crate::{Classification, Purpose};
use serde::{Deserialize, Serialize, Serializer};
use std::{collections::BTreeMap, fmt};

#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq, Eq)]
pub struct Counts {
    pub files: u64,
    pub code: u64,
    pub comments: u64,
    pub blanks: u64,
    pub lines: u64,
}

impl Counts {
    pub fn is_empty(&self) -> bool {
        self.files == 0
    }

    pub fn add(&mut self, other: &Self) {
        self.files += other.files;
        self.code += other.code;
        self.comments += other.comments;
        self.blanks += other.blanks;
        self.lines += other.lines;
    }
}

/// Content kind of a counted file, derived from its backend language.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    Source,
    Data,
    Documentation,
    Other,
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Kind::Source => "source",
            Kind::Data => "data",
            Kind::Documentation => "documentation",
            Kind::Other => "other",
        })
    }
}

#[derive(Debug, Serialize)]
pub struct FileRecord {
    pub path: String,
    pub language: String,
    pub kind: Kind,
    /// Language mapping that supplied `language` for a file the backend does
    /// not recognize; absent when the backend detected the language.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language_mapping: Option<String>,
    #[serde(flatten)]
    pub classification: Classification,
    #[serde(serialize_with = "line_counts")]
    pub counts: Counts,
    /// Per-language counts; serialized only for files with embedded languages,
    /// because a single entry repeats `language` and `counts`.
    #[serde(
        skip_serializing_if = "single_language",
        serialize_with = "language_line_counts"
    )]
    pub languages: BTreeMap<String, Counts>,
}

/// Counts of one file, without `files`, which is always 1 there.
#[derive(Serialize)]
struct LineCounts {
    code: u64,
    comments: u64,
    blanks: u64,
    lines: u64,
}

impl From<&Counts> for LineCounts {
    fn from(counts: &Counts) -> Self {
        Self {
            code: counts.code,
            comments: counts.comments,
            blanks: counts.blanks,
            lines: counts.lines,
        }
    }
}

fn line_counts<S: Serializer>(counts: &Counts, serializer: S) -> Result<S::Ok, S::Error> {
    LineCounts::from(counts).serialize(serializer)
}

fn language_line_counts<S: Serializer>(
    languages: &BTreeMap<String, Counts>,
    serializer: S,
) -> Result<S::Ok, S::Error> {
    serializer.collect_map(
        languages
            .iter()
            .map(|(name, counts)| (name, LineCounts::from(counts))),
    )
}

fn single_language(languages: &BTreeMap<String, Counts>) -> bool {
    languages.len() <= 1
}

/// Aggregated counts; zero counts and empty maps are omitted from JSON
/// (except `all_counted`), so an absent entry means zero.
#[derive(Debug, Default, Serialize)]
pub struct Summary {
    pub all_counted: Counts,
    #[serde(skip_serializing_if = "Counts::is_empty")]
    pub source: Counts,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub by_kind_purpose: BTreeMap<String, Counts>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub source_purpose: BTreeMap<Purpose, Counts>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub source_categories: BTreeMap<String, Counts>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub source_languages: BTreeMap<String, Counts>,
}

impl Summary {
    pub fn purpose(&self, purpose: Purpose) -> Counts {
        self.source_purpose
            .get(&purpose)
            .copied()
            .unwrap_or_default()
    }

    fn add(&mut self, row: &FileRecord) {
        self.all_counted.add(&row.counts);
        let purpose = row.classification.purpose;
        self.by_kind_purpose
            .entry(format!("{}:{purpose}", row.kind))
            .or_default()
            .add(&row.counts);
        if row.kind == Kind::Source {
            self.source.add(&row.counts);
            self.source_purpose
                .entry(purpose)
                .or_default()
                .add(&row.counts);
            self.source_categories
                .entry(format!("{purpose}:{}", row.classification.category))
                .or_default()
                .add(&row.counts);
            for (language, counts) in &row.languages {
                self.source_languages
                    .entry(language.clone())
                    .or_default()
                    .add(counts);
            }
        }
    }
}

#[derive(Debug, Default, Serialize)]
pub struct Folder {
    pub direct: Summary,
    pub recursive: Summary,
}

#[derive(Debug, Serialize)]
pub struct ReportSummary {
    #[serde(flatten)]
    pub totals: Summary,
    pub folders: BTreeMap<String, Folder>,
}

pub fn build_report(rows: &[FileRecord]) -> ReportSummary {
    let mut totals = Summary::default();
    let mut folders: BTreeMap<String, Folder> = BTreeMap::from([(".".into(), Folder::default())]);
    for row in rows {
        totals.add(row);
        let mut parent = row.path.rsplit_once('/').map_or(".", |(parent, _)| parent);
        folders.entry(parent.into()).or_default().direct.add(row);
        loop {
            folders.entry(parent.into()).or_default().recursive.add(row);
            if parent == "." {
                break;
            }
            parent = parent.rsplit_once('/').map_or(".", |(parent, _)| parent);
        }
    }
    ReportSummary { totals, folders }
}
