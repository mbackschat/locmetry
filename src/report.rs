use crate::{Classification, Purpose};
use schemars::JsonSchema;
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

/// Content kind of a counted file, derived from its language.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "lowercase")]
pub enum Kind {
    /// Every language not listed as data or documentation; the only kind in source totals.
    Source,
    /// Data and configuration formats such as JSON and YAML; reports list them in `metadata.kinds`.
    Data,
    /// Prose formats such as Markdown; reports list them in `metadata.kinds`.
    Documentation,
    /// Language names the embedded counter does not know, as in replayed older statistics.
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

/// Totals and per-folder summaries; JSON reports derive `summary` and
/// `breakdown` from it, and Markdown renders its folder table.
#[derive(Debug)]
pub struct ReportSummary {
    pub totals: Summary,
    pub folders: BTreeMap<String, Folder>,
}

impl ReportSummary {
    /// Source code lines per purpose for each top-level folder, recursively;
    /// `.` holds files directly in the root. Full counts are in the breakdown.
    pub fn top_folders(&self) -> BTreeMap<String, BTreeMap<Purpose, u64>> {
        self.folders
            .iter()
            .filter_map(|(path, folder)| match path.as_str() {
                "." => Some((path, &folder.direct)),
                _ if !path.contains('/') => Some((path, &folder.recursive)),
                _ => None,
            })
            .filter(|(_, summary)| !summary.source_purpose.is_empty())
            .map(|(path, summary)| {
                let code = summary
                    .source_purpose
                    .iter()
                    .map(|(purpose, counts)| (*purpose, counts.code));
                (path.clone(), code.collect())
            })
            .collect()
    }
}

/// Counts of one combination of folder, kind, purpose, category, and language.
///
/// `folder` is each file's parent, so a folder's recursive totals are the sum
/// over rows whose folder equals it or starts with it plus `/`. Embedded
/// languages add lines but not files, keeping `files` additive.
#[derive(Debug, Serialize)]
pub struct BreakdownRow {
    pub folder: String,
    pub kind: Kind,
    pub purpose: Purpose,
    pub category: String,
    pub language: String,
    #[serde(flatten)]
    pub counts: Counts,
}

pub fn breakdown(rows: &[FileRecord]) -> Vec<BreakdownRow> {
    let mut cells = BTreeMap::<(String, Kind, Purpose, String, String), Counts>::new();
    for row in rows {
        let folder = row.path.rsplit_once('/').map_or(".", |(parent, _)| parent);
        for (language, counts) in &row.languages {
            let mut counts = *counts;
            if *language != row.language {
                counts.files = 0;
            }
            let key = (
                folder.to_owned(),
                row.kind,
                row.classification.purpose,
                row.classification.category.clone(),
                language.clone(),
            );
            cells.entry(key).or_default().add(&counts);
        }
    }
    cells
        .into_iter()
        .map(
            |((folder, kind, purpose, category, language), counts)| BreakdownRow {
                folder,
                kind,
                purpose,
                category,
                language,
                counts,
            },
        )
        .collect()
}

/// An unclassified source file, listed so configurations can be tuned.
#[derive(Debug, Serialize)]
pub struct ReviewEntry {
    pub path: String,
    pub language: String,
    pub category: String,
    pub rule: String,
    pub code: u64,
    pub lines: u64,
}

/// Unclassified source files, largest first.
pub fn review(rows: &[FileRecord]) -> Vec<ReviewEntry> {
    let mut entries: Vec<_> = rows
        .iter()
        .filter(|row| {
            row.kind == Kind::Source && row.classification.purpose == Purpose::Unclassified
        })
        .map(|row| ReviewEntry {
            path: row.path.clone(),
            language: row.language.clone(),
            category: row.classification.category.clone(),
            rule: row.classification.rule.clone(),
            code: row.counts.code,
            lines: row.counts.lines,
        })
        .collect();
    entries.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.path.cmp(&b.path)));
    entries
}

/// Unclassified source code below one folder; `.` holds the files directly in
/// the scan root.
#[derive(Debug, Serialize)]
pub struct FolderShare {
    pub folder: String,
    pub files: u64,
    pub code: u64,
}

/// Rows a folder list may grow to before the largest folder stops splitting;
/// enough to locate the bulk while fitting one terminal screen.
const FOLDER_ROWS: usize = 8;

/// `path` relative to `folder`, if it lies below it; `""` is the scan root.
fn below<'p>(path: &'p str, folder: &str) -> Option<&'p str> {
    match folder {
        "" => Some(path),
        _ => path.strip_prefix(folder)?.strip_prefix('/'),
    }
}

/// Folders that together hold all unclassified source, largest first, so a
/// reader can inspect where it lives rather than only how much there is.
///
/// Starting from the top-level folders, a chain of single subfolders
/// collapses to its deepest folder, and the largest folder without files of
/// its own splits into its subfolders while the list stays within
/// [`FOLDER_ROWS`] rows, or regardless of rows while it holds most of the
/// code, so a dominant folder is always located precisely. The folders never
/// overlap, so their counts add up.
pub fn unclassified_folders(rows: &[FileRecord]) -> Vec<FolderShare> {
    let files: Vec<(&str, u64)> = rows
        .iter()
        .filter(|row| {
            row.kind == Kind::Source && row.classification.purpose == Purpose::Unclassified
        })
        .map(|row| (row.path.as_str(), row.counts.code))
        .collect();
    // Subfolders that hold files below `folder`, and whether files lie directly in it.
    let children = |folder: &str| {
        let mut subfolders = std::collections::BTreeSet::new();
        let mut direct = false;
        for (path, _) in &files {
            match below(path, folder).map(|rest| rest.split_once('/')) {
                Some(Some((child, _))) => {
                    subfolders.insert(match folder {
                        "" => child.to_string(),
                        _ => format!("{folder}/{child}"),
                    });
                }
                Some(None) => direct = true,
                None => {}
            }
        }
        (subfolders, direct)
    };
    let collapse = |mut folder: String| loop {
        match children(&folder) {
            (subfolders, false) if subfolders.len() == 1 => {
                folder = subfolders.into_iter().next().expect("one subfolder");
            }
            _ => return folder,
        }
    };
    let share = |folder: &str| {
        let inside: Vec<u64> = files
            .iter()
            .filter(|(path, _)| match folder {
                "." => !path.contains('/'),
                _ => below(path, folder).is_some(),
            })
            .map(|(_, code)| *code)
            .collect();
        FolderShare {
            folder: folder.into(),
            files: inside.len() as u64,
            code: inside.iter().sum(),
        }
    };
    let total: u64 = files.iter().map(|(_, code)| code).sum();
    let (top, root_files) = children("");
    let mut folders: Vec<String> = top.into_iter().map(collapse).collect();
    if root_files {
        folders.push(".".into());
    }
    loop {
        let largest = folders
            .iter()
            .filter(|folder| *folder != ".")
            .map(|folder| (share(folder).code, folder.clone()))
            .filter(|(_, folder)| matches!(children(folder), (subfolders, false) if subfolders.len() > 1))
            .max_by(|a, b| a.0.cmp(&b.0).then_with(|| b.1.cmp(&a.1)));
        let Some((code, folder)) = largest else { break };
        let (subfolders, _) = children(&folder);
        let dominant = code * 2 > total;
        if !dominant && folders.len() - 1 + subfolders.len() > FOLDER_ROWS {
            break;
        }
        folders.retain(|candidate| *candidate != folder);
        folders.extend(subfolders.into_iter().map(collapse));
    }
    let mut shares: Vec<FolderShare> = folders.iter().map(|folder| share(folder)).collect();
    shares.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.folder.cmp(&b.folder)));
    shares
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
