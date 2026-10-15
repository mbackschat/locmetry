use crate::{Counts, FileRecord, Kind, Profile};
use anyhow::{bail, ensure, Context, Result};
use ignore::{overrides::OverrideBuilder, WalkBuilder};
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

/// Selection policy applied by [`scan`], recorded with every report.
pub fn scope_policy(profile: &Profile) -> Value {
    json!({
        "hidden": true,
        "respect_ignore_files": true,
        "ignore_files": [".gitignore (inside a Git work tree)", ".ignore", TOOL_IGNORE_FILE],
        "symlinks": "not followed",
        "exclude_git_metadata": true,
        "exclude_prefixes": profile.prefixes(),
    })
}

/// Files selected for counting that are missing from the backend's records.
#[derive(Debug, Default, Serialize)]
pub struct Coverage {
    pub selected_files: usize,
    pub counted_files: usize,
    /// Selected files whose language the backend does not recognize.
    pub unrecognized: Vec<String>,
    /// Recognized files the backend could not read; their counts are missing.
    pub unread: Vec<String>,
}

pub struct Scan {
    pub payload: Value,
    pub coverage: Coverage,
}

pub fn scan(root: &Path, profile: &Profile) -> Result<Scan> {
    let root = root.canonicalize().context("Scan root does not exist")?;
    ensure!(root.is_dir(), "Scan root must be a directory");
    std::fs::read_dir(&root).context("Cannot read scan root")?;
    // Tokei v15 src/utils/fs.rs anchors overrides to CWD; select scope here so
    // root-relative exclusions remain independent of the caller's directory.
    let mut overrides = OverrideBuilder::new(&root);
    overrides.add("!.git")?;
    for prefix in profile.prefixes() {
        overrides.add(&format!("!/{prefix}/"))?;
    }
    let mut walker = WalkBuilder::new(&root);
    walker
        .hidden(false)
        .add_custom_ignore_filename(TOOL_IGNORE_FILE)
        .overrides(overrides.build()?);
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
    let mut payload = serde_json::to_value(languages)?;
    let mut counted = BTreeSet::new();
    // Preserve backend statistics, but normalize paths to the public report contract.
    // `children` repeats embedded reports with their own absolute paths.
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
                        .strip_prefix(&root)
                        .context("Backend path escaped scan root")?,
                )?;
                ensure!(
                    !profile.is_excluded(&name),
                    "Backend returned an excluded file: {name}"
                );
                counted.insert(name.clone());
                report["name"] = Value::String(name);
            }
        }
    }
    let mut coverage = Coverage {
        selected_files: paths.len(),
        counted_files: counted.len(),
        ..Default::default()
    };
    for path in &paths {
        let name = relative_path(path.strip_prefix(&root)?)?;
        if counted.contains(&name) {
            continue;
        }
        match LanguageType::from_path(path, &config) {
            Some(_) => coverage.unread.push(name),
            None => coverage.unrecognized.push(name),
        }
    }
    coverage.unread.sort();
    coverage.unrecognized.sort();
    Ok(Scan { payload, coverage })
}

/// Joins the plain components of a root-relative path with `/`.
///
/// The platform's own path parser splits components, so `:` and `\\` stay
/// literal in Unix file names, while Windows drive prefixes, absolute paths,
/// and parent references are rejected as escaping the scan root.
fn relative_path(path: &Path) -> Result<String> {
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

pub fn file_records(payload: Value, profile: &Profile) -> Result<Vec<FileRecord>> {
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
                !profile.is_excluded(&path),
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
                classification: profile.classify(&path),
                path,
                language: language.clone(),
                kind,
                counts,
                languages,
            });
        }
    }
    rows.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(rows)
}
