use crate::{Classification, Purpose};
use serde::{Deserialize, Serialize};
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
    #[serde(flatten)]
    pub classification: Classification,
    pub counts: Counts,
    pub languages: BTreeMap<String, Counts>,
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub all_counted: Counts,
    pub source: Counts,
    pub by_kind_purpose: BTreeMap<String, Counts>,
    pub source_purpose: BTreeMap<Purpose, Counts>,
    pub source_categories: BTreeMap<String, Counts>,
    pub source_languages: BTreeMap<String, Counts>,
}

impl Default for Summary {
    fn default() -> Self {
        Self {
            all_counted: Counts::default(),
            source: Counts::default(),
            by_kind_purpose: BTreeMap::new(),
            source_purpose: Purpose::ALL
                .into_iter()
                .map(|p| (p, Counts::default()))
                .collect(),
            source_categories: BTreeMap::new(),
            source_languages: BTreeMap::new(),
        }
    }
}

impl Summary {
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
