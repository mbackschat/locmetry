use crate::{Counts, FileRecord, Kind, Policy};
use anyhow::{bail, ensure, Context, Result};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Component, Path},
};
use tokei::LanguageType;

use LanguageType::*;

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

/// Ignore file honored in addition to the `ignore` crate's standard files.
const TOOL_IGNORE_FILE: &str = ".tokeignore";

/// Content kind of a backend language name; names unknown to this Tokei
/// version, for example from saved payloads, stay visible as `Other`.
pub fn language_kind(name: &str) -> Kind {
    match LanguageType::from_name(name) {
        None => Kind::Other,
        Some(language) if DATA_LANGUAGES.contains(&language) => Kind::Data,
        Some(language) if DOCUMENTATION_LANGUAGES.contains(&language) => Kind::Documentation,
        Some(_) => Kind::Source,
    }
}

/// Effective selection policy applied by [`scan`], recorded with every report.
pub fn scope_policy(policy: &Policy) -> Value {
    let scope = policy.scope();
    let ignore_files = match scope.respect_ignore_files() {
        true => vec![
            ".gitignore (inside a Git work tree)",
            ".ignore",
            TOOL_IGNORE_FILE,
        ],
        false => Vec::new(),
    };
    json!({
        "hidden": scope.hidden(),
        "respect_ignore_files": scope.respect_ignore_files(),
        "ignore_files": ignore_files,
        "symlinks": "not followed",
        "exclude": scope.exclusions(),
    })
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
}

pub struct Scan {
    /// Statistics as returned by Tokei, with root-relative paths.
    pub payload: Value,
    /// Statistics for files counted through language mappings, in the same shape.
    pub mapped: Value,
    /// Mapping ID for each mapped file.
    pub mappings: BTreeMap<String, String>,
    pub coverage: Coverage,
    /// Some counts are missing because files could not be read.
    pub inaccurate: bool,
}

impl Scan {
    /// File records from backend and mapped statistics, sorted by path.
    pub fn records(&self, policy: &Policy) -> Result<Vec<FileRecord>> {
        let mut rows = file_records(self.payload.clone(), policy)?;
        for mut row in file_records(self.mapped.clone(), policy)? {
            row.language_mapping = self.mappings.get(&row.path).cloned();
            rows.push(row);
        }
        rows.sort_by(|a, b| a.path.cmp(&b.path));
        Ok(rows)
    }
}

pub fn scan(root: &Path, policy: &Policy) -> Result<Scan> {
    let root = root.canonicalize().context("Scan root does not exist")?;
    ensure!(root.is_dir(), "Scan root must be a directory");
    std::fs::read_dir(&root).context("Cannot read scan root")?;
    // Tokei v15 src/utils/fs.rs anchors overrides to CWD and uses gitignore
    // syntax; select scope here so root-relative globs keep PRODUCT.md semantics.
    let scope = policy.scope().clone();
    let mut walker = WalkBuilder::new(&root);
    walker
        .standard_filters(scope.respect_ignore_files())
        .hidden(!scope.hidden());
    if scope.respect_ignore_files() {
        walker.add_custom_ignore_filename(TOOL_IGNORE_FILE);
    }
    let walk_root = root.clone();
    walker.filter_entry(move |entry| {
        let Ok(path) = entry
            .path()
            .strip_prefix(&walk_root)
            .map_err(anyhow::Error::from)
            .and_then(relative_path)
        else {
            return true; // The root itself; non-UTF-8 names fail later with context.
        };
        match entry.file_type() {
            Some(kind) if kind.is_dir() => !scope.prunes_directory(&path),
            _ => !scope.is_excluded(&path),
        }
    });
    let mut paths = Vec::new();
    for entry in walker.build() {
        let entry = entry.context("Cannot traverse scan root")?;
        if entry.file_type().is_some_and(|kind| kind.is_file()) {
            paths.push(entry.into_path());
        }
    }
    let mut languages = tokei::Languages::new();
    let config = tokei::Config {
        hidden: Some(true),
        no_ignore: Some(true),
        ..Default::default()
    };
    if !paths.is_empty() {
        languages.get_statistics(&paths, &[], &config);
    }
    let mut inaccurate = languages.values().any(|language| language.inaccurate);
    let mut payload = serde_json::to_value(languages)?;
    let counted = normalize_reports(&mut payload, &root, policy)?;
    let mut coverage = Coverage {
        selected_files: paths.len(),
        ..Default::default()
    };
    let mut mapped = BTreeMap::<LanguageType, tokei::Language>::new();
    let mut mappings = BTreeMap::new();
    for path in &paths {
        let name = relative_path(path.strip_prefix(&root)?)?;
        if counted.contains(&name) {
            continue;
        }
        if LanguageType::from_path(path, &config).is_some() {
            coverage.unread.push(name);
            continue;
        }
        match policy.mapped_language(&name) {
            None => coverage.unrecognized.push(name),
            Some((id, language)) => match language.parse(path.clone(), &config) {
                Ok(report) => {
                    mapped.entry(language).or_default().add_report(report);
                    mappings.insert(name, id.to_owned());
                }
                Err(_) => {
                    coverage.unread.push(name);
                    inaccurate = true;
                }
            },
        }
    }
    let mut mapped = serde_json::to_value(mapped)?;
    normalize_reports(&mut mapped, &root, policy)?;
    coverage.mapped_files = mappings.len();
    coverage.counted_files = counted.len() + mappings.len();
    coverage.unread.sort();
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
    Ok(Scan {
        payload,
        mapped,
        mappings,
        coverage,
        inaccurate,
    })
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

pub fn file_records(payload: Value, policy: &Policy) -> Result<Vec<FileRecord>> {
    let mut groups = payload
        .as_object()
        .context("Invalid backend language map")?
        .clone();
    groups.remove("Total");
    let groups: BTreeMap<String, Group> =
        serde_json::from_value(Value::Object(groups)).context("Invalid backend statistics")?;
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
