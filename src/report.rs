use crate::{
    modules::{Module, ModuleOwners},
    paths::{ancestors, below, parent, ROOT},
    Classification, Purpose,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize, Serializer};
use std::{collections::BTreeMap, fmt};

#[derive(Clone, Copy, Debug, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Counts {
    /// Physical files, except source-language summaries where this counts participation and may overlap.
    pub files: u64,
    /// Nonnegative code lines.
    pub code: u64,
    /// Nonnegative comment lines, including documentation embedded in source.
    pub comments: u64,
    /// Nonnegative blank lines.
    pub blanks: u64,
    /// Total lines: code + comments + blanks. Validation rejects overflow and inconsistent sums.
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
    /// Language names the embedded counter does not know, as in statistics a newer counter saved; unlike unrecognized files, they are counted.
    Unknown,
}

impl Kind {
    pub const ALL: [Self; 4] = [Self::Source, Self::Data, Self::Documentation, Self::Unknown];
}

impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Kind::Source => "source",
            Kind::Data => "data",
            Kind::Documentation => "documentation",
            Kind::Unknown => "unknown",
        })
    }
}

#[derive(Clone, Debug)]
pub struct FileRecord {
    pub path: String,
    pub language: String,
    pub kind: Kind,
    /// Mapping provenance when recognition needed an explicit language.
    pub language_mapping: Option<String>,
    pub classification: Classification,
    pub counts: Counts,
    /// Runtime participation includes the host; the public contract omits redundant single-language detail.
    pub languages: BTreeMap<String, Counts>,
    /// Lines of inline `#[cfg(test)]` items, counted with the file's own
    /// purpose; zero in test files and in files without such items.
    pub inline_tests: Counts,
}

impl Serialize for FileRecord {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        crate::contract::File::from(self).serialize(serializer)
    }
}

impl FileRecord {
    /// Unclaimed source, which a configuration may still need to classify.
    pub fn needs_review(&self) -> bool {
        self.kind == Kind::Source && self.classification.is_unclaimed()
    }
}

/// Aggregated counts; zero counts and empty maps are omitted from JSON
/// (except `all_counted`), so an absent entry means zero.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Summary {
    /// All counted kinds; always present, even when empty.
    pub all_counted: Counts,
    #[serde(skip_serializing_if = "Counts::is_empty")]
    #[serde(default)]
    /// Source kind only; omitted when no source files were counted.
    pub source: Counts,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    #[serde(default)]
    /// All kinds, keyed kind:purpose; additive across keys.
    pub by_kind_purpose: BTreeMap<String, Counts>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    #[serde(default)]
    /// Source only, keyed production/test/other; additive.
    #[schemars(schema_with = "crate::schema::purpose_map::<Counts>")]
    pub source_purpose: BTreeMap<Purpose, Counts>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    #[serde(default)]
    /// Source only, keyed purpose:category; additive.
    pub source_categories: BTreeMap<String, Counts>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    #[serde(default)]
    /// Source line contributions by language; line categories add, file participation can overlap.
    pub source_languages: BTreeMap<String, Counts>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    #[serde(default)]
    /// Inline Rust `#[cfg(test)]` items within source of each purpose except test, whose lines that purpose already counts; `files` counts files containing any. Absent when none were found, and in reports of earlier producers.
    #[schemars(schema_with = "crate::schema::purpose_map::<Counts>")]
    pub inline_tests: BTreeMap<Purpose, Counts>,
}

impl Summary {
    pub fn purpose(&self, purpose: Purpose) -> Counts {
        self.source_purpose
            .get(&purpose)
            .copied()
            .unwrap_or_default()
    }

    /// Adds `row`, whose summary keys `keys` holds; a file adds to every
    /// folder above it, so the keys are formatted once per file.
    fn add(&mut self, row: &FileRecord, keys: &RowKeys) {
        self.all_counted.add(&row.counts);
        add_at(&mut self.by_kind_purpose, &keys.kind_purpose, &row.counts);
        if row.kind == Kind::Source {
            self.source.add(&row.counts);
            self.source_purpose
                .entry(row.classification.purpose)
                .or_default()
                .add(&row.counts);
            add_at(&mut self.source_categories, &keys.category, &row.counts);
            for (language, counts) in &row.languages {
                add_at(&mut self.source_languages, language, counts);
            }
            if row.inline_tests.lines > 0 {
                self.inline_tests
                    .entry(row.classification.purpose)
                    .or_default()
                    .add(&row.inline_tests);
            }
        }
    }
}

/// Adds `counts` under `key`, allocating the key only when it is new.
fn add_at(map: &mut BTreeMap<String, Counts>, key: &str, counts: &Counts) {
    match map.get_mut(key) {
        Some(total) => total.add(counts),
        None => map.entry(key.into()).or_default().add(counts),
    }
}

/// `kind:purpose`, the key of [`Summary::by_kind_purpose`].
pub(crate) fn kind_purpose_key(kind: Kind, purpose: Purpose) -> String {
    format!("{kind}:{purpose}")
}

/// `purpose:category`, the key of [`Summary::source_categories`].
pub(crate) fn category_key(purpose: Purpose, category: &str) -> String {
    format!("{purpose}:{category}")
}

/// A file's summary keys.
struct RowKeys {
    kind_purpose: String,
    category: String,
}

impl RowKeys {
    fn of(row: &FileRecord) -> Self {
        let purpose = row.classification.purpose;
        Self {
            kind_purpose: kind_purpose_key(row.kind, purpose),
            category: category_key(purpose, &row.classification.category),
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
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BreakdownRow {
    pub folder: String,
    pub kind: Kind,
    pub purpose: Purpose,
    pub category: String,
    pub language: String,
    #[serde(flatten)]
    pub counts: Counts,
}

/// A breakdown cell's identity: folder, kind, purpose, category and language.
pub(crate) type BreakdownKey = (String, Kind, Purpose, String, String);

impl BreakdownRow {
    pub(crate) fn key(&self) -> BreakdownKey {
        (
            self.folder.clone(),
            self.kind,
            self.purpose,
            self.category.clone(),
            self.language.clone(),
        )
    }
}

pub fn breakdown(rows: &[FileRecord]) -> Vec<BreakdownRow> {
    let mut cells = BTreeMap::<BreakdownKey, Counts>::new();
    for row in rows {
        let folder = parent(&row.path);
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

/// An unclaimed source file, listed so configurations can be tuned.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReviewEntry {
    pub path: String,
    pub language: String,
    pub category: String,
    pub rule: String,
    pub code: u64,
    pub lines: u64,
}

/// Unclaimed source files, largest first.
pub fn review(rows: &[FileRecord]) -> Vec<ReviewEntry> {
    let mut entries: Vec<_> = rows
        .iter()
        .filter(|row| row.needs_review())
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

/// Unclaimed source code below one folder; `.` holds the files directly in
/// the scan root.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FolderShare {
    pub folder: String,
    pub files: u64,
    pub code: u64,
}

/// Rows a folder list may grow to before the largest folder stops splitting;
/// enough to locate the bulk while fitting one terminal screen.
const FOLDER_ROWS: usize = 8;

/// Whether `path` counts toward the share of `folder`, where [`ROOT`] holds
/// only the files directly in the scan root, so shares never overlap.
pub(crate) fn share_holds(folder: &str, path: &str) -> bool {
    match folder {
        ROOT => !path.contains('/'),
        _ => below(path, folder).is_some(),
    }
}

pub(crate) fn collapse_folder(
    mut folder: String,
    mut only_child: impl FnMut(&str) -> Option<String>,
) -> String {
    while let Some(child) = only_child(&folder) {
        folder = child;
    }
    folder
}

/// Folders that together hold all unclaimed source, largest first, so a
/// reader can inspect where it lives rather than only how much there is.
///
/// Starting from the top-level folders, a chain of single subfolders
/// collapses to its deepest folder, and the largest folder without files of
/// its own splits into its subfolders while the list stays within
/// [`FOLDER_ROWS`] rows, or regardless of rows while it holds most of the
/// code, so a dominant folder is always located precisely. The folders never
/// overlap, so their counts add up.
pub fn unclaimed_folders(rows: &[FileRecord]) -> Vec<FolderShare> {
    let files: Vec<(&str, u64)> = rows
        .iter()
        .filter(|row| row.needs_review())
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
                        ROOT => child.to_string(),
                        _ => format!("{folder}/{child}"),
                    });
                }
                Some(None) => direct = true,
                None => {}
            }
        }
        (subfolders, direct)
    };
    let collapse = |folder: String| {
        collapse_folder(folder, |folder| match children(folder) {
            (subfolders, false) if subfolders.len() == 1 => subfolders.into_iter().next(),
            _ => None,
        })
    };
    let share = |folder: &str| {
        let inside: Vec<u64> = files
            .iter()
            .filter(|(path, _)| share_holds(folder, path))
            .map(|(_, code)| *code)
            .collect();
        FolderShare {
            folder: folder.into(),
            files: inside.len() as u64,
            code: inside.iter().sum(),
        }
    };
    let total: u64 = files.iter().map(|(_, code)| code).sum();
    let (top, root_files) = children(ROOT);
    let mut folders: Vec<String> = top.into_iter().map(collapse).collect();
    if root_files {
        folders.push(ROOT.into());
    }
    loop {
        let largest = folders
            .iter()
            .filter(|folder| *folder != ROOT)
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

/// The entry of folder `path`, looked up before inserting, so only a new
/// folder allocates its key.
fn folder<'a>(folders: &'a mut BTreeMap<String, Folder>, path: &str) -> &'a mut Folder {
    if !folders.contains_key(path) {
        folders.insert(path.into(), Folder::default());
    }
    folders.get_mut(path).expect("inserted above")
}

pub fn build_report(rows: &[FileRecord]) -> ReportSummary {
    let mut totals = Summary::default();
    let mut folders: BTreeMap<String, Folder> = BTreeMap::from([(ROOT.into(), Folder::default())]);
    for row in rows {
        let keys = RowKeys::of(row);
        totals.add(row, &keys);
        let direct = parent(&row.path);
        folder(&mut folders, direct).direct.add(row, &keys);
        for path in std::iter::once(direct).chain(ancestors(direct)) {
            folder(&mut folders, path).recursive.add(row, &keys);
        }
    }
    ReportSummary { totals, folders }
}

/// Source code that one build module classified as production.
#[derive(Debug, Serialize)]
pub struct ModuleShare {
    pub path: String,
    pub detector: &'static str,
    pub files: u64,
    pub lines: u64,
    pub code: u64,
}

/// Source code each adopted module classified, in module order. A file
/// belongs to the innermost module of the detector that decided it, which is
/// the module whose layout claimed it.
pub fn module_shares(modules: &[Module], rows: &[FileRecord]) -> Vec<ModuleShare> {
    let mut shares: Vec<ModuleShare> = modules
        .iter()
        .map(|module| ModuleShare {
            path: module.path.clone(),
            detector: module.detector,
            files: 0,
            lines: 0,
            code: 0,
        })
        .collect();
    let owners = ModuleOwners::new(modules.iter().map(|m| (m.path.as_str(), m.detector)));
    for row in rows.iter().filter(|row| row.kind == Kind::Source) {
        if let Some(index) = owners.owner(&row.path, &row.classification.rule) {
            shares[index].files += 1;
            shares[index].lines += row.counts.lines;
            shares[index].code += row.counts.code;
        }
    }
    shares
}

/// A source file that a build module's layout classified as production.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ModuleFile {
    pub path: String,
    pub lines: u64,
    pub code: u64,
    /// Folder of the module whose layout claimed the file; `.` is the scan root.
    pub module: String,
    pub detector: String,
}

/// Files [`largest_module_files`] lists: enough to expose a generated or
/// vendored file, which dwarfs hand-written ones, in one glance.
pub(crate) const MODULE_FILES: usize = 5;

/// The largest source files module layouts claimed, by code and then path.
/// No other finding lists them, because a layout decides production without
/// review; generated or vendored code inside a module surfaces here.
pub fn largest_module_files(modules: &[Module], rows: &[FileRecord]) -> Vec<ModuleFile> {
    let owners = ModuleOwners::new(modules.iter().map(|m| (m.path.as_str(), m.detector)));
    let files: Vec<ModuleFile> = rows
        .iter()
        .filter(|row| row.kind == Kind::Source)
        .filter_map(|row| {
            let module = &modules[owners.owner(&row.path, &row.classification.rule)?];
            Some(ModuleFile {
                path: row.path.clone(),
                lines: row.counts.lines,
                code: row.counts.code,
                module: module.path.clone(),
                detector: module.detector.into(),
            })
        })
        .collect();
    rank_module_files(files)
}

/// The [`MODULE_FILES`] largest of `files`, by code and then path.
pub(crate) fn rank_module_files(mut files: Vec<ModuleFile>) -> Vec<ModuleFile> {
    files.sort_by(|a, b| b.code.cmp(&a.code).then_with(|| a.path.cmp(&b.path)));
    files.truncate(MODULE_FILES);
    files
}

/// Modules and classified source code of one build system.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BuildSystemShare {
    pub modules: u64,
    pub code: u64,
}

/// [`module_shares`] per build system, keyed by detector ID.
pub fn build_systems(shares: &[ModuleShare]) -> BTreeMap<&'static str, BuildSystemShare> {
    let mut systems = BTreeMap::<_, BuildSystemShare>::new();
    for share in shares {
        let system = systems.entry(share.detector).or_default();
        system.modules += 1;
        system.code += share.code;
    }
    systems
}

/// Measured source of one Git submodule.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SubmoduleShare {
    pub path: String,
    pub files: u64,
    pub code: u64,
}

/// Source measured inside each submodule `path`; submodules without measured
/// source, because they are excluded or not checked out, are left out, so the
/// hint disappears once the configuration decides their scope.
pub fn submodule_shares(paths: &[String], rows: &[FileRecord]) -> Vec<SubmoduleShare> {
    paths
        .iter()
        .map(|path| {
            let mut share = SubmoduleShare {
                path: path.clone(),
                files: 0,
                code: 0,
            };
            for row in rows.iter().filter(|row| row.kind == Kind::Source) {
                if below(&row.path, path).is_some() {
                    share.files += 1;
                    share.code += row.counts.code;
                }
            }
            share
        })
        .filter(|share| share.files > 0)
        .collect()
}

/// IDs of `entries` that decided no file: a glob that matches nothing, or an
/// entry an earlier stage always preempts, such as a rule behind a test
/// convention.
pub fn unused_entries<'a>(
    entries: impl IntoIterator<Item = &'a str>,
    rows: &[FileRecord],
) -> Vec<String> {
    let decided: std::collections::BTreeSet<&str> = rows
        .iter()
        .map(|row| row.classification.rule.as_str())
        .collect();
    entries
        .into_iter()
        .filter(|id| !decided.contains(id))
        .map(String::from)
        .collect()
}
