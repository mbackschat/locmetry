//! File records from backend statistics, with embedded documentation folded into its host.
use super::*;

/// Rewrites report paths to the public root-relative form and returns them.
///
/// `children` repeats embedded reports with their own absolute paths.
pub(super) fn normalize_reports(
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

#[derive(Deserialize)]
pub(super) struct Stats {
    pub(super) code: u64,
    pub(super) comments: u64,
    pub(super) blanks: u64,
    #[serde(default)]
    pub(super) blobs: BTreeMap<String, Stats>,
}

#[derive(Deserialize)]
pub(super) struct RawReport {
    pub(super) name: String,
    pub(super) stats: Stats,
}

#[derive(Deserialize)]
pub(super) struct Group {
    #[serde(default)]
    pub(super) reports: Vec<RawReport>,
}

impl Stats {
    pub(super) fn own(&self) -> Counts {
        Counts {
            files: 1,
            code: self.code,
            comments: self.comments,
            blanks: self.blanks,
            lines: self.code + self.comments + self.blanks,
        }
    }

    pub(super) fn subtree(&self) -> Counts {
        let mut counts = self.own();
        for blob in self.blobs.values() {
            counts.add(&blob.subtree());
        }
        counts
    }
}

pub(super) fn language_counts(
    language: &str,
    stats: &Stats,
    host: Kind,
    result: &mut BTreeMap<String, Counts>,
) {
    let mut counts = stats.own();
    for (child, stats) in &stats.blobs {
        if host != Kind::Documentation && language_kind(child) == Kind::Documentation {
            // Tokei reports documentation embedded in source, such as Vue's
            // `<template lang="markdown">` or an HTML `<script type="text/plain">`,
            // as a child language; physically it is prose of the host, so it
            // counts as the host's comments and must not add source code.
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
    file_records_with(payload, policy, |_, _| None)
}

/// File records, with `marker` asked for the generated-code evidence of each
/// source file a module layout decided, given its path and language; files
/// are read for markers in parallel and only when a marker could apply.
pub(super) fn file_records_with(
    payload: &Value,
    policy: &Policy,
    marker: impl Fn(&str, &str) -> Option<GeneratedEvidence> + Sync,
) -> Result<Vec<FileRecord>> {
    let groups = payload
        .as_object()
        .context("Invalid backend language map")?
        .iter()
        .filter(|(language, _)| *language != "Total")
        .map(|(language, group)| Ok((language.clone(), Group::deserialize(group)?)))
        .collect::<serde_json::Result<BTreeMap<String, Group>>>()
        .context("Invalid backend statistics")?;
    let mut rows = Vec::new();
    let mut markable = Vec::new();
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
            let resolution = policy.resolve(&path, kind);
            markable.push(resolution.markable);
            rows.push(FileRecord {
                module: resolution.module,
                classification: resolution.classification,
                path,
                language: language.clone(),
                kind,
                language_mapping: None,
                language_correction: None,
                counts,
                languages,
                inline_tests: Counts::default(),
                refinement: None,
            });
        }
    }
    rows.par_iter_mut()
        .zip(markable)
        .filter(|(_, markable)| *markable)
        .for_each(|(row, _)| {
            if let Some(evidence) = marker(&row.path, &row.language) {
                row.classification = evidence.classification();
            }
        });
    for row in &mut rows {
        policy.refine(row);
    }
    rows.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(rows)
}
