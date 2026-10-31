use crate::{
    config::{CorrectedLanguage, GeneratedEvidence},
    modules, Counts, FileRecord, Kind, Module, Policy, Purpose,
};
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

mod generated;
mod handlebars;
mod inline_tests;
mod languages;
mod lean;
mod notebook;
mod records;
mod selection;
mod submodules;

pub use languages::*;
pub use records::*;
pub use selection::*;

#[derive(Clone, Debug, Serialize, Deserialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ParseFailureReason {
    /// Readable Handlebars or Lean bytes cannot be decoded as UTF-8 or BOM-directed text.
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

/// Ignore file honored in addition to the `ignore` crate's standard files.
const TOOL_IGNORE_FILE: &str = ".tokeignore";

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
    /// Correction ID for each selected file a language correction decided,
    /// counted or left unrecognized.
    pub corrections: BTreeMap<String, String>,
    pub coverage: Coverage,
    /// Some counts are missing because files could not be read or parsed.
    pub inaccurate: bool,
    /// Modules that the selected build descriptors declare.
    pub modules: Vec<Module>,
    /// Configured exclusions that left out nothing, see [`Policy::unused_exclusions`].
    pub unused_exclusions: Vec<String>,
    /// Counts of inline `#[cfg(test)]` items per counted Rust file that has any.
    pub inline_tests: BTreeMap<String, Counts>,
    /// Generated-code markers of counted files, read on demand; `None` when
    /// markers are off or no module was detected, since a marker refines
    /// only what a module layout decides.
    markers: Option<generated::Markers>,
}

impl Scan {
    /// The generated-code marker evidence of a counted `file`, whether or not
    /// a marker applies to it.
    pub(crate) fn marker(&self, file: &FileRecord) -> Option<GeneratedEvidence> {
        let recognized = file.language_mapping.is_none() && file.language_correction.is_none();
        self.markers
            .as_ref()?
            .evidence(&file.path, &file.language, recognized)
    }

    /// File records from backend and mapped statistics, sorted by path.
    pub fn records(&self, policy: &Policy) -> Result<Vec<FileRecord>> {
        let marker = |recognized| {
            move |path: &str, language: &str| {
                self.markers.as_ref()?.evidence(path, language, recognized)
            }
        };
        let mut rows = file_records_with(&self.payload, policy, marker(true))?;
        for mut row in file_records_with(&self.mapped, policy, marker(false))? {
            row.language_mapping = self.mappings.get(&row.path).cloned();
            row.language_correction = self.corrections.get(&row.path).cloned();
            rows.push(row);
        }
        // In a test file, inline tests are already counted as test.
        for row in &mut rows {
            if row.classification.purpose != Purpose::Test {
                row.inline_tests = self
                    .inline_tests
                    .get(&row.path)
                    .copied()
                    .unwrap_or_default();
            }
        }
        rows.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(rows)
    }
}

/// Measure a selected directory while retaining the ordinary project module context.
pub fn scan_subfolder(root: &Path, policy: &Policy, subfolder: &str) -> Result<Scan> {
    scan_selection(root, policy, Some(Target::Subfolder(subfolder))).map(|(scan, _)| scan)
}

pub fn scan(root: &Path, policy: &Policy) -> Result<Scan> {
    scan_selection(root, policy, None).map(|(scan, _)| scan)
}

/// The canonical, readable scan root. Errors name `root` as the user typed it,
/// per the human output wording rules (docs/help-design.md).
pub fn scan_root(root: &Path) -> Result<PathBuf> {
    let named = crate::output::quoted(&root.display().to_string());
    let canonical =
        dunce::canonicalize(root).with_context(|| format!("Cannot open scan root {named}"))?;
    ensure!(canonical.is_dir(), "Scan root {named} must be a directory");
    std::fs::read_dir(&canonical).with_context(|| format!("Cannot read scan root {named}"))?;
    Ok(canonical)
}

pub(crate) fn scan_selection(
    root: &Path,
    policy: &Policy,
    target: Option<Target<'_>>,
) -> Result<(Scan, Option<Exclusion>)> {
    let root = scan_root(root)?;
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
                    "Subfolder '{folder}' is excluded at '{}' by {}. Revise the exclusion policy.",
                    excluded.at,
                    excluded
                        .patterns
                        .iter()
                        .map(|pattern| crate::output::quoted(pattern))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
            }
            let reason = match filter_reason(&root, policy.scope(), Target::Subfolder(folder))? {
                FilterReason::IgnorePolicy => "ignore policy. Try '--respect-ignore-files false', which changes the whole project policy and module context.",
                FilterReason::HiddenPolicy => "hidden policy. Try '--hidden true', which changes the whole project policy and module context.",
                FilterReason::ReasonNotRecorded => "cause could not be established. Inspect overlapping scope filters.",
            };
            bail!("Subfolder '{folder}' is not selected: {reason}");
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
    let selected_files = paths.len();
    // Corrections replace the counter's recognition, so their files bypass it.
    let mut corrections = BTreeMap::new();
    let mut corrected = Vec::new();
    for path in std::mem::take(&mut paths) {
        let name = relative_path(path.strip_prefix(&root)?)?;
        match policy.corrected_language(&name) {
            Some((id, language)) => {
                corrections.insert(name.clone(), id.to_owned());
                corrected.push((path, name, language));
            }
            None => paths.push(path),
        }
    }
    let config = tokei::Config {
        hidden: Some(true),
        no_ignore: Some(true),
        ..Default::default()
    };
    let mut coverage = Coverage {
        selected_files,
        ..Default::default()
    };
    let languages = count_selected(&root, &paths, &config, &mut coverage)?;
    let rust_files: Vec<PathBuf> = languages
        .get(&LanguageType::Rust)
        .map(|rust| {
            rust.reports
                .iter()
                .map(|report| report.name.clone())
                .collect()
        })
        .unwrap_or_default();
    let mut inaccurate = languages.values().any(|language| language.inaccurate);
    let mut payload = serde_json::to_value(languages)?;
    let counted = normalize_reports(&mut payload, &root, policy)?;
    let inline_tests = inline_rust_tests(&root, &rust_files, &config)?;
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
                if count_as(language, path, &name, &config, &mut mapped, &mut coverage) {
                    mappings.insert(name, id.to_owned());
                } else {
                    inaccurate = true;
                }
            }
        }
    }
    let mut corrected_files = 0;
    for (path, name, language) in corrected {
        match language {
            CorrectedLanguage::Unrecognized => coverage.unrecognized.push(name),
            CorrectedLanguage::Counted(language) => {
                if count_as(language, &path, &name, &config, &mut mapped, &mut coverage) {
                    corrected_files += 1;
                } else {
                    inaccurate = true;
                }
            }
        }
    }
    let mut mapped = serde_json::to_value(mapped)?;
    let mapped_names = normalize_reports(&mut mapped, &root, policy)?;
    let markers = (policy.reads_generated_markers() && !modules.is_empty()).then(|| {
        let all_counted = counted.iter().chain(&mapped_names).cloned().collect();
        generated::Markers::load(&root, &all_counted)
    });
    coverage.mapped_files = mappings.len();
    coverage.counted_files = counted.len() + mappings.len() + corrected_files;
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
            corrections,
            coverage,
            inaccurate,
            modules,
            unused_exclusions,
            inline_tests,
            markers,
        },
        exclusion,
    ))
}

/// Inline test counts of the counted Rust files `rust_files`. The text is read
/// again because Tokei parses files itself; an unreadable file is already
/// reported by counting, so it adds nothing here.
fn inline_rust_tests(
    root: &Path,
    rust_files: &[PathBuf],
    config: &tokei::Config,
) -> Result<BTreeMap<String, Counts>> {
    let rust: Vec<(String, &PathBuf)> = rust_files
        .iter()
        .map(|path| Ok((relative_path(path.strip_prefix(root)?)?, path)))
        .collect::<Result<_>>()?;
    Ok(rust
        .par_iter()
        .filter_map(|(name, path)| {
            let text = decoded_text(path, ParseFailureReason::InvalidEncoding, "Rust").ok()?;
            let counts = inline_tests::counts(&text, config);
            (counts.lines > 0).then(|| (name.clone(), counts))
        })
        .collect())
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
            // A file whose extension several languages share is read to
            // decide its language; counting reuses those bytes.
            LanguageType::from_path_with_bytes(path, config)
                .map(|(kind, bytes)| (path, kind, parse_file(kind, path, bytes, config)))
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

/// Counts `path` with `language` into `counted`, the statistics of files the
/// counter's own recognition did not count; a read or parse failure is
/// recorded in `coverage` instead, and `false` returned.
fn count_as(
    language: LanguageType,
    path: &Path,
    name: &str,
    config: &tokei::Config,
    counted: &mut BTreeMap<LanguageType, tokei::Language>,
    coverage: &mut Coverage,
) -> bool {
    match parse_file(language, path, None, config) {
        Ok(report) => {
            counted.entry(language).or_default().add_report(report);
            true
        }
        Err(ParseError::Read) => {
            coverage.unread.push(name.to_owned());
            false
        }
        Err(ParseError::Content(reason, message)) => {
            coverage.parse_failures.push(ParseFailure {
                path: name.to_owned(),
                reason,
                message,
            });
            false
        }
    }
}

/// Counts `path` as `language`, from `bytes` when detection already read them.
fn parse_file(
    language: LanguageType,
    path: &Path,
    bytes: Option<Vec<u8>>,
    config: &tokei::Config,
) -> std::result::Result<tokei::Report, ParseError> {
    match (language, bytes) {
        (Handlebars, _) => handlebars::parse(path),
        (Jupyter, _) => notebook::parse(path, config),
        (Lean, _) => lean::parse(path),
        (_, Some(bytes)) => language
            .parse_bytes(path.to_owned(), bytes, config)
            .map_err(|_| ParseError::Read),
        (_, None) => language
            .parse(path.to_owned(), config)
            .map_err(|_| ParseError::Read),
    }
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

#[cfg(test)]
mod tests;
