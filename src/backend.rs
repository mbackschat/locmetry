use crate::{modules, Counts, FileRecord, Kind, Module, Policy};
use anyhow::{bail, ensure, Context, Result};
use ignore::WalkBuilder;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
};
use tokei::LanguageType;

use LanguageType::*;

mod handlebars;
mod notebook;
mod submodules;

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParseFailureReason {
    /// Readable Handlebars bytes cannot be decoded as UTF-8 or BOM-directed text.
    InvalidEncoding,
    /// Invalid encoding, JSON or counting structure; includes zero-byte input.
    InvalidNotebook,
    /// Notebook version outside the supported v4.0–v4.5 range; not necessarily invalid nbformat.
    UnsupportedNotebookVersion,
    /// Explicit code-language declaration unavailable to the cell counter.
    UnsupportedNotebookLanguage,
    /// Cell type outside code, markdown and raw; the whole notebook remains uncounted.
    UnsupportedNotebookCell,
}

impl std::fmt::Display for ParseFailureReason {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(match self {
            Self::InvalidEncoding => "invalid_encoding",
            Self::InvalidNotebook => "invalid_notebook",
            Self::UnsupportedNotebookVersion => "unsupported_notebook_version",
            Self::UnsupportedNotebookLanguage => "unsupported_notebook_language",
            Self::UnsupportedNotebookCell => "unsupported_notebook_cell",
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ParseFailure {
    /// Root-relative path, prefixed with its member name in workspace reports.
    pub path: String,
    /// Stable reason for unavailable counts; branch on this code instead of message text.
    pub reason: ParseFailureReason,
    /// Human-readable detail, without a stable text-parsing contract.
    pub message: String,
}

impl std::fmt::Display for ParseFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}: {} — {}", self.path, self.reason, self.message)
    }
}

/// Languages counted as data or configuration rather than source.
///
/// Tokei recognizes mostly programming languages, so source is the default
/// kind; these lists name the reviewed exceptions for Tokei 15.0.0.
pub const DATA_LANGUAGES: &[LanguageType] = &[
    Bean,
    DotNetResource,
    Edn,
    FEN,
    ForgeConfig,
    Hex,
    Hledger,
    Ini,
    IntelHex,
    Json,
    MsBuild,
    NuGetConfig,
    Po,
    RON,
    Svg,
    Toml,
    UnrealPlugin,
    UnrealProject,
    UrWebProject,
    VerilogArgsFile,
    VisualStudioProject,
    VisualStudioSolution,
    XcodeConfig,
    Xml,
    Yaml,
];

pub const DOCUMENTATION_LANGUAGES: &[LanguageType] = &[
    AsciiDoc,
    Djot,
    Markdown,
    Mdx,
    Org,
    ReStructuredText,
    Tex,
    Text,
    Typst,
    UnrealDeveloperMarkdown,
];

/// Variants that human views show within their family's row: the same
/// language with embedded markup, or its header and module units. C Header
/// stays apart because Tokei assigns every `.h` to it, including C++ and
/// Objective-C headers; Fortran has no family name Tokei knows.
const LANGUAGE_FAMILIES: &[(LanguageType, LanguageType)] = &[
    (Tsx, TypeScript),
    (GlimmerTs, TypeScript),
    (Jsx, JavaScript),
    (GlimmerJs, JavaScript),
    (CppHeader, Cpp),
    (CppModule, Cpp),
    (UnrealShaderHeader, UnrealShader),
];

/// The family a variant's display name joins, such as TypeScript for TSX;
/// `None` for every other name.
pub fn language_family(name: &str) -> Option<&'static str> {
    LANGUAGE_FAMILIES
        .iter()
        .find(|(variant, _)| variant.name() == name)
        .map(|(_, family)| family.name())
}

/// The display name of the source language `name` denotes, in any case.
/// Data and documentation names are refused: tables count only source, so
/// selecting one would leave them empty.
pub fn source_language(name: &str) -> Result<&'static str, String> {
    let language = LanguageType::from_name(name)
        .or_else(|| {
            LanguageType::list()
                .iter()
                .map(|(language, _)| *language)
                .find(|language| language.name().eq_ignore_ascii_case(name))
        })
        .ok_or_else(|| format!("unknown language {name:?}; use a name from the Language table"))?;
    match language_kind(language.name()) {
        Kind::Source => Ok(language.name()),
        kind => Err(format!("{} is {kind}, not source", language.name())),
    }
}

/// Ignore file honored in addition to the `ignore` crate's standard files.
const TOOL_IGNORE_FILE: &str = ".tokeignore";

/// Content kind of a backend language name; names unknown to this Tokei
/// version, for example from saved payloads, stay visible as `Unknown`.
pub fn language_kind(name: &str) -> Kind {
    match LanguageType::from_name(name) {
        None => Kind::Unknown,
        Some(language) if DATA_LANGUAGES.contains(&language) => Kind::Data,
        Some(language) if DOCUMENTATION_LANGUAGES.contains(&language) => Kind::Documentation,
        Some(_) => Kind::Source,
    }
}

/// Submodule scope hints from the root's `.gitmodules`, without invoking Git.
/// Missing or unreadable files yield no hints; this never changes scan selection.
pub fn git_submodules(root: &Path) -> Vec<String> {
    std::fs::read_to_string(root.join(".gitmodules"))
        .map(|text| submodules::paths(&text))
        .unwrap_or_default()
}

/// Effective selection policy applied by [`scan`], recorded with every report.
pub fn scope_policy(policy: &Policy) -> crate::contract::Selection {
    let scope = policy.scope();
    let ignore_files = match scope.respect_ignore_files() {
        true => vec![
            ".gitignore (inside a Git work tree)",
            ".ignore",
            TOOL_IGNORE_FILE,
        ],
        false => Vec::new(),
    };
    crate::contract::Selection {
        subfolder: None,
        hidden: scope.hidden(),
        respect_ignore_files: scope.respect_ignore_files(),
        ignore_files: ignore_files.into_iter().map(String::from).collect(),
        symlinks: "not followed".into(),
        exclude: scope.exclusions(),
    }
}

/// Files selected for counting that are missing from the backend's records.
#[derive(Debug, Default, Serialize)]
pub struct Coverage {
    pub selected_files: usize,
    /// Files with counts, including those counted through language mappings.
    pub counted_files: usize,
    /// Counted files whose language came from a language mapping.
    pub mapped_files: usize,
    pub unrecognized_files: usize,
    /// Unrecognized files per extension (`(none)` without one).
    pub unrecognized_extensions: BTreeMap<String, usize>,
    /// Selected files whose language neither the backend nor a mapping recognizes.
    pub unrecognized: Vec<String>,
    /// Recognized files that could not be read; their counts are missing.
    pub unread: Vec<String>,
    /// Always recorded, including when empty; older reports lack this evidence.
    pub parse_failures: Vec<ParseFailure>,
}

impl Coverage {
    /// Adds `other`'s files, with its paths mapped through `path`, as a
    /// workspace merges its members.
    pub fn merge(&mut self, other: &Coverage, path: impl Fn(&str) -> String) {
        self.selected_files += other.selected_files;
        self.counted_files += other.counted_files;
        self.mapped_files += other.mapped_files;
        self.unrecognized_files += other.unrecognized_files;
        for (extension, files) in &other.unrecognized_extensions {
            *self
                .unrecognized_extensions
                .entry(extension.clone())
                .or_default() += files;
        }
        self.unrecognized
            .extend(other.unrecognized.iter().map(|name| path(name)));
        self.unread
            .extend(other.unread.iter().map(|name| path(name)));
        self.parse_failures
            .extend(other.parse_failures.iter().map(|failure| ParseFailure {
                path: path(&failure.path),
                ..failure.clone()
            }));
    }
}

pub struct Scan {
    /// Statistics as returned by Tokei, with root-relative paths.
    pub payload: Value,
    /// Statistics for files counted through language mappings, in the same shape.
    pub mapped: Value,
    /// Mapping ID for each mapped file.
    pub mappings: BTreeMap<String, String>,
    pub coverage: Coverage,
    /// Some counts are missing because files could not be read or parsed.
    pub inaccurate: bool,
    /// Modules that the selected build descriptors declare.
    pub modules: Vec<Module>,
    /// Configured exclusions that left out nothing, see [`Policy::unused_exclusions`].
    pub unused_exclusions: Vec<String>,
}

impl Scan {
    /// File records from backend and mapped statistics, sorted by path.
    pub fn records(&self, policy: &Policy) -> Result<Vec<FileRecord>> {
        let mut rows = file_records(&self.payload, policy)?;
        for mut row in file_records(&self.mapped, policy)? {
            row.language_mapping = self.mappings.get(&row.path).cloned();
            rows.push(row);
        }
        rows.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(rows)
    }
}

pub(crate) struct Selection {
    pub paths: Vec<PathBuf>,
    pub target_selected: bool,
    applied: BTreeSet<usize>,
    pub exclusion: Option<Exclusion>,
}

pub(crate) struct Exclusion {
    pub at: String,
    pub patterns: Vec<String>,
}

#[derive(Clone, Copy)]
pub(crate) enum Target<'a> {
    File(&'a str),
    Subfolder(&'a str),
}
impl<'a> Target<'a> {
    fn path(self) -> &'a str {
        match self {
            Self::File(path) | Self::Subfolder(path) => path,
        }
    }
    fn contains(self, path: &str) -> bool {
        match self {
            Self::File(target) => target == path,
            Self::Subfolder(target) => crate::paths::below(path, target).is_some(),
        }
    }
}

/// Check on-disk spelling before selection; aliases must not become false scope diagnoses.
/// Project path identity follows directory entries, including on alias-tolerant APFS
/// (docs/PRODUCT.md, Path Explanations and Subfolder Measurement).
pub(crate) fn inspect_path(root: &Path, path: &str, label: &str) -> Result<Option<String>> {
    let mut current = root.to_owned();
    let mut relative = PathBuf::new();
    for component in Path::new(path).components() {
        let exact = std::fs::read_dir(&current)
            .with_context(|| format!("Cannot inspect {label} path spelling"))?
            .collect::<std::io::Result<Vec<_>>>()?
            .iter()
            .any(|entry| entry.file_name() == component.as_os_str());
        current.push(component);
        let metadata = std::fs::symlink_metadata(&current)
            .with_context(|| format!("{label} target does not exist or cannot be inspected"))?;
        ensure!(
            exact,
            "{label} target must use the exact spelling of directory entries: {path}"
        );
        relative.push(component);
        if metadata.file_type().is_symlink() {
            return Ok(Some(relative_path(&relative)?));
        }
    }
    Ok(None)
}

/// Resolve one literal directory without changing the project's path base.
pub fn subfolder_path(root: &Path, given: &Path) -> Result<Option<String>> {
    ensure!(!given.as_os_str().is_empty(), "Subfolder must not be empty");
    ensure!(
        given
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir)),
        "Subfolder must be root-relative without parent components"
    );
    if given.components().all(|c| c == Component::CurDir) {
        return Ok(None);
    }
    let path = relative_path(given)?;
    ensure!(
        inspect_path(root, &path, "Subfolder")?.is_none(),
        "Subfolder must not pass through a symlink: {path}"
    );
    ensure!(
        root.join(&path).is_dir(),
        "Subfolder target must be a directory"
    );
    Ok(Some(path))
}

/// Measure a selected directory while retaining the ordinary project module context.
pub fn scan_subfolder(root: &Path, policy: &Policy, subfolder: &str) -> Result<Scan> {
    scan_selection(root, policy, Some(Target::Subfolder(subfolder))).map(|(scan, _)| scan)
}

#[derive(Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FilterReason {
    /// Disabling only ignore filtering selects the target; no winning ignore-file rule is claimed.
    IgnorePolicy,
    /// Enabling only hidden paths selects the target.
    HiddenPolicy,
    /// No single-setting probe established a cause, including overlapping filters.
    ReasonNotRecorded,
}

pub(crate) fn filter_reason(
    root: &Path,
    scope: &crate::Scope,
    target: Target<'_>,
) -> Result<FilterReason> {
    let selected_with = |overrides| -> Result<bool> {
        let scope = scope.with_overrides(&overrides)?;
        Ok(select_paths(root, &scope, Some(target))?.target_selected)
    };
    if scope.respect_ignore_files()
        && selected_with(crate::ScopeOverrides {
            respect_ignore_files: Some(false),
            ..Default::default()
        })?
    {
        Ok(FilterReason::IgnorePolicy)
    } else if !scope.hidden()
        && selected_with(crate::ScopeOverrides {
            hidden: Some(true),
            ..Default::default()
        })?
    {
        Ok(FilterReason::HiddenPolicy)
    } else {
        Ok(FilterReason::ReasonNotRecorded)
    }
}

pub fn scan(root: &Path, policy: &Policy) -> Result<Scan> {
    scan_selection(root, policy, None).map(|(scan, _)| scan)
}

pub(crate) fn select_paths(
    root: &Path,
    scope: &crate::Scope,
    target: Option<Target<'_>>,
) -> Result<Selection> {
    let root = root.to_owned();
    // Tokei v15 src/utils/fs.rs anchors overrides to CWD and uses gitignore
    // syntax; select scope here so root-relative globs keep docs/PRODUCT.md semantics.
    let scope = scope.clone();
    let mut walker = WalkBuilder::new(&root);
    walker
        .standard_filters(scope.respect_ignore_files())
        .hidden(!scope.hidden());
    if scope.respect_ignore_files() {
        walker.add_custom_ignore_filename(TOOL_IGNORE_FILE);
    }
    let walk_root = root.clone();
    let applied = Arc::new(Mutex::new(BTreeSet::new()));
    let recorder = Arc::clone(&applied);
    let exclusion = Arc::new(Mutex::new(None));
    let trace = Arc::clone(&exclusion);
    let target_path = target.map(|t| t.path().to_owned());
    let subfolder = match target {
        Some(Target::Subfolder(path)) => Some(path.to_owned()),
        _ => None,
    };
    walker.filter_entry(move |entry| {
        let Ok(path) = entry
            .path()
            .strip_prefix(&walk_root)
            .map_err(anyhow::Error::from)
            .and_then(relative_path)
        else {
            return true; // The root itself; non-UTF-8 names fail later with context.
        };
        let directory = entry.file_type().is_some_and(|kind| kind.is_dir());
        let excluded_by = scope.excluded_by(&path, directory);
        let selected = excluded_by.is_empty();
        if !selected
            && target_path.as_ref().is_some_and(|target| {
                target == &path || (directory && crate::paths::below(target, &path).is_some())
            })
        {
            let patterns = scope.exclusions();
            *trace.lock().expect("single-threaded traversal") = Some(Exclusion {
                at: path.clone(),
                patterns: excluded_by
                    .iter()
                    .map(|&index| patterns[index].clone())
                    .collect(),
            });
        }
        if subfolder.as_ref().is_none_or(|folder| {
            path == *folder
                || Target::Subfolder(folder).contains(&path)
                || (directory && Target::Subfolder(&path).contains(folder))
        }) {
            recorder
                .lock()
                .expect("the walk records exclusions on one thread")
                .extend(excluded_by);
        }
        selected
    });
    let mut paths = Vec::new();
    let mut target_selected = false;
    for entry in walker.build() {
        let entry = entry.context("Cannot traverse scan root")?;
        if target.is_some_and(|target| entry.path() == root.join(target.path())) {
            target_selected = true;
        }
        if entry.file_type().is_some_and(|kind| kind.is_file()) {
            paths.push(entry.into_path());
        }
    }
    let applied = std::mem::take(&mut *applied.lock().expect("the walk has finished"));
    let exclusion = exclusion.lock().expect("the walk has finished").take();
    Ok(Selection {
        paths,
        target_selected,
        applied,
        exclusion,
    })
}

pub(crate) fn scan_selection(
    root: &Path,
    policy: &Policy,
    target: Option<Target<'_>>,
) -> Result<(Scan, Option<Exclusion>)> {
    let root = root.canonicalize().context("Scan root does not exist")?;
    ensure!(root.is_dir(), "Scan root must be a directory");
    std::fs::read_dir(&root).context("Cannot read scan root")?;
    let Selection {
        mut paths,
        target_selected,
        applied,
        exclusion,
    } = select_paths(&root, policy.scope(), target)?;
    if let Some(Target::Subfolder(folder)) = target {
        if !target_selected {
            if let Some(excluded) = &exclusion {
                bail!(
                    "Subfolder {folder} is excluded at {} by {}; revise the exclusion policy",
                    excluded.at,
                    excluded.patterns.join(", ")
                );
            }
            let reason = match filter_reason(&root, policy.scope(), Target::Subfolder(folder))? {
                FilterReason::IgnorePolicy => "ignore policy; try --respect-ignore-files false (changes the whole project policy and module context)",
                FilterReason::HiddenPolicy => "hidden policy; try --hidden true (changes the whole project policy and module context)",
                FilterReason::ReasonNotRecorded => "cause could not be established; inspect overlapping scope filters",
            };
            bail!("Subfolder {folder} is not selected: {reason}");
        }
    }
    let names = paths
        .iter()
        .map(|path| relative_path(path.strip_prefix(&root)?))
        .collect::<Result<Vec<_>>>()?;
    let modules = modules::detect(&names.iter().map(String::as_str).collect::<Vec<_>>());
    if let Some(target) = target {
        paths.retain(|path| {
            relative_path(path.strip_prefix(&root).expect("selected beneath root"))
                .is_ok_and(|path| target.contains(&path))
        });
    }
    let unused_exclusions = policy.unused_exclusions(&applied);
    let config = tokei::Config {
        hidden: Some(true),
        no_ignore: Some(true),
        ..Default::default()
    };
    let mut coverage = Coverage {
        selected_files: paths.len(),
        ..Default::default()
    };
    let languages = count_selected(&root, &paths, &config, &mut coverage)?;
    let mut inaccurate = languages.values().any(|language| language.inaccurate);
    let mut payload = serde_json::to_value(languages)?;
    let counted = normalize_reports(&mut payload, &root, policy)?;
    let failed: BTreeSet<_> = coverage
        .parse_failures
        .iter()
        .map(|failure| failure.path.clone())
        .collect();
    let mut mapped = BTreeMap::<LanguageType, tokei::Language>::new();
    let mut mappings = BTreeMap::new();
    for path in &paths {
        let name = relative_path(path.strip_prefix(&root)?)?;
        if counted.contains(&name) || failed.contains(&name) {
            continue;
        }
        if LanguageType::from_path(path, &config).is_some() {
            coverage.unread.push(name);
            continue;
        }
        match policy.mapped_language(&name) {
            None => coverage.unrecognized.push(name),
            Some((id, language)) => {
                let report = parse_file(language, path, &config);
                match report {
                    Ok(report) => {
                        mapped.entry(language).or_default().add_report(report);
                        mappings.insert(name, id.to_owned());
                    }
                    Err(error) => {
                        match error {
                            ParseError::Read => coverage.unread.push(name),
                            ParseError::Content(reason, message) => {
                                coverage.parse_failures.push(ParseFailure {
                                    path: name,
                                    reason,
                                    message,
                                })
                            }
                        }
                        inaccurate = true;
                    }
                }
            }
        }
    }
    let mut mapped = serde_json::to_value(mapped)?;
    normalize_reports(&mut mapped, &root, policy)?;
    coverage.mapped_files = mappings.len();
    coverage.counted_files = counted.len() + mappings.len();
    coverage.unread.sort();
    coverage.parse_failures.sort_by(|a, b| a.path.cmp(&b.path));
    coverage.unrecognized.sort();
    coverage.unrecognized_files = coverage.unrecognized.len();
    for path in &coverage.unrecognized {
        let name = path.rsplit('/').next().unwrap_or(path);
        let extension = match name.rsplit_once('.') {
            Some((stem, extension)) if !stem.is_empty() => extension,
            _ => "(none)",
        };
        *coverage
            .unrecognized_extensions
            .entry(extension.to_owned())
            .or_default() += 1;
    }
    Ok((
        Scan {
            payload,
            mapped,
            mappings,
            coverage,
            inaccurate,
            modules,
            unused_exclusions,
        },
        exclusion,
    ))
}

fn count_selected(
    root: &Path,
    paths: &[PathBuf],
    config: &tokei::Config,
    coverage: &mut Coverage,
) -> Result<tokei::Languages> {
    let mut languages = tokei::Languages::new();
    // Tokei 15's src/utils/fs.rs re-walks every get_statistics input. Parse
    // selected files directly to avoid that traversal and preserve selection
    // if a file becomes a directory between selection and counting.
    let parsed: Vec<_> = paths
        .par_iter()
        .filter_map(|path| {
            LanguageType::from_path(path, config)
                .map(|kind| (path, kind, parse_file(kind, path, config)))
        })
        .collect();
    for (path, kind, result) in parsed {
        let language = languages.entry(kind).or_default();
        match result {
            Ok(report) => language.add_report(report),
            Err(error) => {
                if let ParseError::Content(reason, message) = error {
                    coverage.parse_failures.push(ParseFailure {
                        path: relative_path(path.strip_prefix(root)?)?,
                        reason,
                        message,
                    });
                }
                language.mark_inaccurate();
            }
        }
    }
    for language in languages.values_mut() {
        language.total();
    }
    Ok(languages)
}

pub(super) enum ParseError {
    Read,
    Content(ParseFailureReason, String),
}

fn decoded_text(
    path: &Path,
    reason: ParseFailureReason,
    label: &str,
) -> std::result::Result<String, ParseError> {
    use std::io::Read;
    // Preserve Tokei 15's BOM-aware decoding, but keep access and content
    // failures distinct as required by PRODUCT's coverage contract.
    let bytes = std::fs::read(path).map_err(|_| ParseError::Read)?;
    let mut text = String::new();
    encoding_rs_io::DecodeReaderBytesBuilder::new()
        .build(bytes.as_slice())
        .read_to_string(&mut text)
        .map_err(|error| {
            ParseError::Content(reason, format!("Cannot decode {label} encoding: {error}"))
        })?;
    Ok(text)
}

fn parse_file(
    language: LanguageType,
    path: &Path,
    config: &tokei::Config,
) -> std::result::Result<tokei::Report, ParseError> {
    match language {
        Handlebars => handlebars::parse(path),
        Jupyter => notebook::parse(path, config),
        _ => language
            .parse(path.to_owned(), config)
            .map_err(|_| ParseError::Read),
    }
}

/// Rewrites report paths to the public root-relative form and returns them.
///
/// `children` repeats embedded reports with their own absolute paths.
fn normalize_reports(
    payload: &mut Value,
    root: &Path,
    policy: &Policy,
) -> Result<BTreeSet<String>> {
    let mut counted = BTreeSet::new();
    for group in payload
        .as_object_mut()
        .context("Invalid backend language map")?
        .values_mut()
    {
        let mut lists = Vec::new();
        for (key, value) in group.as_object_mut().into_iter().flatten() {
            match key.as_str() {
                "reports" => lists.push(value),
                "children" => lists.extend(
                    value
                        .as_object_mut()
                        .into_iter()
                        .flat_map(|c| c.values_mut()),
                ),
                _ => {}
            }
        }
        for reports in lists {
            for report in reports.as_array_mut().into_iter().flatten() {
                let name = report["name"]
                    .as_str()
                    .context("Invalid backend report path")?;
                let name = relative_path(
                    Path::new(name)
                        .strip_prefix(root)
                        .context("Backend path escaped scan root")?,
                )?;
                ensure!(
                    !policy.is_excluded(&name),
                    "Backend returned an excluded file: {name}"
                );
                counted.insert(name.clone());
                report["name"] = Value::String(name);
            }
        }
    }
    Ok(counted)
}

/// Joins the plain components of a root-relative path with `/`.
///
/// The platform's own path parser splits components, so `:` and `\\` stay
/// literal in Unix file names, while Windows drive prefixes, absolute paths,
/// and parent references are rejected as escaping the scan root.
pub(crate) fn relative_path(path: &Path) -> Result<String> {
    let mut parts = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => parts.push(
                part.to_str()
                    .context("Non-UTF-8 file path cannot be represented in JSON")?,
            ),
            Component::CurDir => {}
            _ => bail!("File outside scan root: {}", path.display()),
        }
    }
    ensure!(!parts.is_empty(), "Empty backend path");
    Ok(parts.join("/"))
}

#[derive(Deserialize)]
struct Stats {
    code: u64,
    comments: u64,
    blanks: u64,
    #[serde(default)]
    blobs: BTreeMap<String, Stats>,
}

#[derive(Deserialize)]
struct RawReport {
    name: String,
    stats: Stats,
}

#[derive(Deserialize)]
struct Group {
    #[serde(default)]
    reports: Vec<RawReport>,
}

impl Stats {
    fn own(&self) -> Counts {
        Counts {
            files: 1,
            code: self.code,
            comments: self.comments,
            blanks: self.blanks,
            lines: self.code + self.comments + self.blanks,
        }
    }

    fn subtree(&self) -> Counts {
        let mut counts = self.own();
        for blob in self.blobs.values() {
            counts.add(&blob.subtree());
        }
        counts
    }
}

fn language_counts(
    language: &str,
    stats: &Stats,
    host: Kind,
    result: &mut BTreeMap<String, Counts>,
) {
    let mut counts = stats.own();
    for (child, stats) in &stats.blobs {
        if host != Kind::Documentation && language_kind(child) == Kind::Documentation {
            // Tokei 15.0.0 parses Rust `///` and `//!` blocks, doctests included,
            // as Markdown blobs (src/language/embedding.rs:170); physically they
            // are comments of the host, so they must not add source code.
            let documentation = stats.subtree();
            counts.comments += documentation.code + documentation.comments;
            counts.blanks += documentation.blanks;
            counts.lines += documentation.lines;
        } else {
            language_counts(child, stats, host, result);
        }
    }
    let total = result.entry(language.into()).or_default();
    total.add(&counts);
    total.files = 1;
}

pub fn file_records(payload: &Value, policy: &Policy) -> Result<Vec<FileRecord>> {
    let groups = payload
        .as_object()
        .context("Invalid backend language map")?
        .iter()
        .filter(|(language, _)| *language != "Total")
        .map(|(language, group)| Ok((language.clone(), Group::deserialize(group)?)))
        .collect::<serde_json::Result<BTreeMap<String, Group>>>()
        .context("Invalid backend statistics")?;
    let mut rows = Vec::new();
    let mut seen = BTreeSet::new();
    for (language, group) in groups {
        for report in group.reports {
            let path = relative_path(Path::new(&report.name))?;
            ensure!(seen.insert(path.clone()), "Duplicate physical file: {path}");
            ensure!(
                !policy.is_excluded(&path),
                "Backend returned an excluded file: {path}"
            );
            let kind = language_kind(&language);
            let mut languages = BTreeMap::new();
            language_counts(&language, &report.stats, kind, &mut languages);
            let mut counts = Counts::default();
            for value in languages.values() {
                counts.add(value);
            }
            counts.files = 1;
            rows.push(FileRecord {
                classification: policy.classify(&path, kind),
                path,
                language: language.clone(),
                kind,
                language_mapping: None,
                counts,
                languages,
            });
        }
    }
    rows.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(rows)
}

#[cfg(test)]
mod selected_file_tests {
    use super::*;

    #[test]
    fn selected_file_statistics_match_tokei_including_embedded_languages() {
        let temp = tempfile::tempdir().unwrap();
        let root = temp.path().canonicalize().unwrap();
        let mut paths = Vec::new();
        for (name, source) in [
            ("a.rs", "// comment\n\nfn main() {}\n"),
            ("b.rs", "fn other() {}\n"),
            ("tool", "#!/usr/bin/env python\nprint(1)\n"),
            (
                "page.html",
                "<script>\nconst x = 1;\n</script>\n<style>\na { color: red; }\n</style>\n",
            ),
            ("second.html", "<script>\nconst y = 2;\n</script>\n"),
            ("README.md", "# Example\n\n```rust\nfn item() {}\n```\n"),
            ("unknown.zzz", "unrecognized\n"),
        ] {
            let path = root.join(name);
            std::fs::write(&path, source).unwrap();
            paths.push(path);
        }
        let config = tokei::Config {
            hidden: Some(true),
            no_ignore: Some(true),
            ..Default::default()
        };
        let mut expected = tokei::Languages::new();
        expected.get_statistics(&paths, &[], &config);
        let actual = count_selected(&root, &paths, &config, &mut Coverage::default()).unwrap();
        let ordered = |mut languages: tokei::Languages| {
            for language in languages.values_mut() {
                language.reports.sort_by(|a, b| a.name.cmp(&b.name));
                for reports in language.children.values_mut() {
                    reports.sort_by(|a, b| a.name.cmp(&b.name));
                }
            }
            serde_json::to_value(languages).unwrap()
        };
        assert_eq!(ordered(actual), ordered(expected));
    }

    #[test]
    fn a_selected_file_removed_before_counting_marks_counts_unavailable() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("removed.rs");
        std::fs::write(&path, "fn main() {}\n").unwrap();
        let policy = Policy::load_layers(None, None).unwrap();
        let selected = select_paths(temp.path(), policy.scope(), None).unwrap();
        std::fs::remove_file(path).unwrap();
        let languages = count_selected(
            temp.path(),
            &selected.paths,
            &tokei::Config::default(),
            &mut Coverage::default(),
        )
        .unwrap();
        assert!(languages
            .values()
            .all(|language| language.reports.is_empty()));
        assert!(languages.values().any(|language| language.inaccurate));
    }

    #[test]
    fn counting_does_not_expand_a_selected_file_replaced_by_a_directory() {
        for filename in ["chosen.rs", "chosen.py"] {
            let temp = tempfile::tempdir().unwrap();
            let root = temp.path().canonicalize().unwrap();
            let path = root.join(filename);
            std::fs::write(&path, "original\n").unwrap();
            let policy = Policy::load_layers(None, None).unwrap();
            let selected = select_paths(&root, policy.scope(), None).unwrap();
            assert_eq!(selected.paths.as_slice(), std::slice::from_ref(&path));
            std::fs::remove_file(&path).unwrap();
            std::fs::create_dir(&path).unwrap();
            std::fs::write(path.join("unselected.rs"), "fn added() {}\n").unwrap();
            let languages = count_selected(
                &root,
                &selected.paths,
                &tokei::Config::default(),
                &mut Coverage::default(),
            )
            .unwrap();
            assert!(languages
                .values()
                .all(|language| language.reports.is_empty()));
            assert!(languages.values().any(|language| language.inaccurate));
        }
    }
}
