//! Semantic validation of stored evidence; comparison eligibility is a separate check.
use super::*;
use crate::paths::{below, member_of, within};
use anyhow::{bail, ensure, Context, Result};
use std::collections::BTreeSet;

mod records;
mod totals;
use totals::*;

fn checked_sum(values: impl IntoIterator<Item = u64>, context: &str) -> Result<u64> {
    values.into_iter().try_fold(0u64, |total, n| {
        total
            .checked_add(n)
            .with_context(|| format!("{context}: overflow"))
    })
}

impl Counts {
    /// Adds stored counts without wrapping; comparison never interprets overflow as a delta.
    pub fn checked_add(&self, other: &Self) -> Result<Self> {
        Ok(Self {
            files: checked_sum([self.files, other.files], "files")?,
            code: checked_sum([self.code, other.code], "code")?,
            comments: checked_sum([self.comments, other.comments], "comments")?,
            blanks: checked_sum([self.blanks, other.blanks], "blanks")?,
            lines: checked_sum([self.lines, other.lines], "lines")?,
        })
    }
    fn validate(&self, context: &str) -> Result<()> {
        ensure!(
            checked_sum([self.code, self.comments, self.blanks], context)? == self.lines,
            "{context}: lines must equal code + comments + blanks"
        );
        Ok(())
    }
}

fn total<'a>(values: impl IntoIterator<Item = &'a Counts>) -> Result<Counts> {
    values
        .into_iter()
        .try_fold(Counts::default(), |sum, c| sum.checked_add(c))
}
fn same_lines(a: Counts, b: Counts) -> bool {
    LineCounts::from(a) == LineCounts::from(b)
}
fn unique<T: Ord>(keys: impl IntoIterator<Item = T>, context: &str) -> Result<()> {
    let mut seen = BTreeSet::new();
    for key in keys {
        ensure!(seen.insert(key), "{context}: duplicate identity");
    }
    Ok(())
}
/// Whether a root-relative path may name the root itself, as folders may.
#[derive(Clone, Copy)]
enum Root {
    Allowed,
    Rejected,
}

fn path(path: &str, root: Root) -> Result<()> {
    if let (Root::Allowed, crate::paths::ROOT) = (root, path) {
        return Ok(());
    }
    ensure!(
        !path.is_empty() && path.split('/').all(|part| !matches!(part, "" | "." | "..")),
        "invalid root-relative path '{path}'"
    );
    Ok(())
}
/// A summary key naming a purpose alone or a purpose and a category.
#[derive(Clone, Copy)]
enum PurposeKey {
    Purpose,
    Category,
}

fn purpose_key(key: &str, shape: PurposeKey) -> Result<()> {
    let prefix = match shape {
        PurposeKey::Purpose => key,
        PurposeKey::Category => {
            key.split_once(':')
                .context("category key must be purpose:category")?
                .0
        }
    };
    ensure!(
        Purpose::ALL.iter().any(|p| p.to_string() == prefix),
        "invalid purpose key '{key}'"
    );
    Ok(())
}
impl MeasurementReport {
    /// Validates arithmetic and identities supported by the stored detail, without rescanning.
    /// This internal boundary is exposed to CLI users only through compare input errors.
    pub fn validate(&self) -> Result<()> {
        match self.detail {
            Detail::Full => self.require_comparison_evidence()?,
            Detail::Headline => ensure!(
                self.review.is_none()
                    && self.breakdown.is_none()
                    && self.files.is_none()
                    && self.metadata.kinds.is_none()
                    && self.metadata.modules.is_none()
                    && self.metadata.config.is_none()
                    && self.metadata.coverage.unrecognized.is_none()
                    && self
                        .metadata
                        .members
                        .iter()
                        .flatten()
                        .all(|member| member.config.is_none()),
                "headline detail must omit full-report evidence"
            ),
        }
        self.validate_mode()?;
        if let Some(config) = &self.metadata.config {
            config.validate()?;
        }
        for member in self.metadata.members.iter().flatten() {
            if let Some(config) = &member.config {
                config.validate()?;
            }
        }
        validate_totals(&self.summary.totals, "summary")?;
        let coverage_paths = self.validate_coverage()?;
        self.validate_subfolder(&coverage_paths)?;
        self.validate_build_modules()?;
        self.validate_findings()?;
        self.validate_breakdown()?;
        self.validate_files(&coverage_paths)?;
        self.validate_members(&coverage_paths)
    }

    fn validate_mode(&self) -> Result<()> {
        let m = &self.metadata;
        match &m.members {
            None => ensure!(
                m.root.is_some()
                    && m.scope.is_some()
                    && m.config_path.is_some()
                    && m.workspace_path.is_none(),
                "metadata: single-root fields are required, and 'workspace_path' needs members"
            ),
            Some(members) => {
                ensure!(!members.is_empty(), "metadata.members must not be empty");
                ensure!(
                    m.root.is_none()
                        && m.scope.is_none()
                        && m.config_path.is_none()
                        && m.config.is_none(),
                    "metadata: root/workspace fields cannot be mixed"
                );
                unique(members.iter().map(|m| &m.name), "members")?;
                // Folder members all measure folders of one project; mixing
                // them with independent roots has no producer.
                let folders = members
                    .iter()
                    .filter(|m| m.scope.subfolder.is_some())
                    .count();
                ensure!(
                    folders == 0 || folders == members.len(),
                    "workspace members mix project folders with independent roots"
                );
                for member in members {
                    ensure!(!member.name.contains('/'), "invalid member name");
                    path(&member.name, Root::Rejected)?;
                }
            }
        }
        Ok(())
    }

    fn validate_subfolder(&self, coverage_paths: &[&String]) -> Result<()> {
        let Some(folder) = self
            .metadata
            .scope
            .as_ref()
            .and_then(|s| s.subfolder.as_deref())
        else {
            return Ok(());
        };
        path(folder, Root::Rejected).context("invalid subfolder restriction")?;
        let inside = |p: &str| below(p, folder).is_some();
        let folder_inside = |p: &str| within(p, folder);
        for p in coverage_paths
            .iter()
            .copied()
            .chain(self.files.iter().flatten().map(|f| &f.path))
            .chain(self.review.iter().flatten().map(|r| &r.path))
        {
            ensure!(
                inside(p),
                "measured path '{p}' is outside subfolder '{folder}'"
            );
        }
        for p in self.breakdown.iter().flatten().map(|r| &r.folder).chain(
            self.summary
                .unclaimed_folders
                .iter()
                .flatten()
                .map(|f| &f.folder),
        ) {
            ensure!(
                folder_inside(p),
                "measured folder '{p}' is outside subfolder '{folder}'"
            );
        }
        let top = member_of(folder);
        ensure!(
            self.summary.top_folders.keys().all(|p| p == top),
            "top_folders is outside subfolder ancestry"
        );
        for share in &self.summary.submodules {
            path(&share.path, Root::Allowed)?;
            ensure!(
                folder_inside(&share.path) || below(folder, &share.path).is_some(),
                "submodule share is disjoint from subfolder"
            );
            ensure!(
                share.files > 0
                    && share.files <= self.summary.totals.source.files
                    && share.code <= self.summary.totals.source.code,
                "submodule share exceeds measured source"
            );
        }
        let production = self
            .summary
            .totals
            .source_purpose
            .get(&Purpose::Production)
            .copied()
            .unwrap_or_default();
        let shares = &self.summary.build_modules;
        ensure!(
            checked_sum(shares.values().map(|s| s.modules), "build_modules")? <= production.files
                && checked_sum(shares.values().map(|s| s.code), "build_modules")?
                    <= production.code,
            "build_modules exceeds measured production contributions"
        );
        for (detector, share) in shares {
            ensure!(
                share.modules > 0,
                "build_modules must contain contributing modules"
            );
            if let Some(modules) = &self.metadata.modules {
                ensure!(
                    share.modules
                        <= modules.iter().filter(|m| &m.detector == detector).count() as u64,
                    "build_modules exceeds recorded module context"
                );
            }
        }
        Ok(())
    }

    /// Reconciles `summary.build_modules` with the file records' `module`
    /// credit. Earlier producers credited modules by `rule` and wrote no
    /// `module`; their unrestricted reports also counted modules without
    /// credit, so only their subfolder reports can be reconciled.
    fn validate_build_modules(&self) -> Result<()> {
        let (Some(files), Some(modules)) = (&self.files, &self.metadata.modules) else {
            return Ok(());
        };
        let by_module = files.iter().any(|f| f.module.is_some());
        let restricted = self
            .metadata
            .scope
            .as_ref()
            .is_some_and(|s| s.subfolder.is_some());
        if !by_module && !restricted {
            return Ok(());
        }
        let shares = &self.summary.build_modules;
        for file in files {
            let Some(module) = &file.module else {
                continue;
            };
            ensure!(
                file.kind == Kind::Source
                    && file.purpose == Purpose::Production
                    && below(&file.path, &module.path).is_some()
                    && modules
                        .iter()
                        .any(|m| m.path == module.path && m.detector == module.detector),
                "file module must be a recorded module containing production source"
            );
        }
        let owners = crate::modules::ModuleOwners::new(
            modules
                .iter()
                .map(|m| (m.path.as_str(), m.detector.as_str())),
        );
        let mut contributions = BTreeMap::<(&str, &str), u64>::new();
        for file in files.iter().filter(|f| f.kind == Kind::Source) {
            let owner = match &file.module {
                Some(module) => Some((module.detector.as_str(), module.path.as_str())),
                None if by_module => None,
                None => owners.owner(&file.path, &file.rule).map(|index| {
                    (
                        modules[index].detector.as_str(),
                        modules[index].path.as_str(),
                    )
                }),
            };
            if let Some(owner) = owner {
                let code = contributions.entry(owner).or_default();
                *code = checked_sum([*code, file.counts.code], "module contribution")?;
            }
        }
        let mut expected = BTreeMap::<&str, (u64, u64)>::new();
        for ((detector, _), code) in contributions {
            let entry = expected.entry(detector).or_default();
            entry.0 += 1;
            entry.1 = checked_sum([entry.1, code], "build_modules")?;
        }
        ensure!(
            expected.len() == shares.len()
                && expected.iter().all(|(key, (modules, code))| shares
                    .get(*key)
                    .is_some_and(|s| s.modules == *modules && s.code == *code)),
            "build_modules conflicts with measured file contributions"
        );
        Ok(())
    }

    fn validate_coverage(&self) -> Result<Vec<&String>> {
        let m = &self.metadata;
        let c = &m.coverage;
        ensure!(
            c.counted_files == self.summary.totals.all_counted.files,
            "coverage.counted_files conflicts with summary"
        );
        ensure!(
            c.mapped_files <= c.counted_files,
            "coverage.mapped_files exceeds counted_files"
        );
        let failures = c.parse_failures.as_deref().unwrap_or_default();
        ensure!(
            checked_sum(
                [
                    c.counted_files,
                    c.unrecognized_files,
                    c.unread.len() as u64,
                    failures.len() as u64
                ],
                "coverage.selected_files"
            )? == c.selected_files,
            "coverage.selected_files does not reconcile"
        );
        ensure!(
            checked_sum(
                c.unrecognized_extensions.values().copied(),
                "unrecognized_extensions"
            )? == c.unrecognized_files,
            "coverage.unrecognized_extensions does not reconcile"
        );
        ensure!(
            (c.unread.is_empty() && failures.is_empty()) || m.backend_inaccurate,
            "coverage failures require backend_inaccurate"
        );
        if let Some(paths) = &c.unrecognized {
            ensure!(
                paths.len() as u64 == c.unrecognized_files,
                "coverage.unrecognized paths do not reconcile"
            );
        }
        let coverage_paths = c
            .unrecognized
            .iter()
            .flatten()
            .chain(&c.unread)
            .chain(failures.iter().map(|f| &f.path))
            .collect::<Vec<_>>();
        unique(coverage_paths.iter().copied(), "coverage paths")?;
        for name in &coverage_paths {
            path(name, Root::Rejected)?;
        }
        Ok(coverage_paths)
    }

    fn validate_findings(&self) -> Result<()> {
        let m = &self.metadata;
        if let Some(modules) = &m.modules {
            unique(modules.iter().map(|m| (&m.path, &m.detector)), "modules")?;
            for module in modules {
                path(&module.path, Root::Allowed)?;
                ensure!(!module.detector.is_empty(), "empty module detector");
            }
        }
        unique(
            self.summary.submodules.iter().map(|m| &m.path),
            "submodules",
        )?;
        unique(
            self.summary
                .unclaimed_folders
                .iter()
                .flatten()
                .map(|f| &f.folder),
            "unclaimed_folders",
        )?;
        let module_files = self.summary.largest_module_files.iter().flatten();
        unique(
            module_files.clone().map(|f| &f.path),
            "largest_module_files",
        )?;
        for file in module_files {
            path(&file.path, Root::Rejected)?;
            path(&file.module, Root::Allowed)?;
        }
        if let (Some(files), Some(modules), Some(recorded)) =
            (&self.files, &m.modules, &self.summary.largest_module_files)
        {
            let owners = crate::modules::ModuleOwners::new(
                modules
                    .iter()
                    .map(|m| (m.path.as_str(), m.detector.as_str())),
            );
            let measured = files
                .iter()
                .filter(|f| f.kind == Kind::Source)
                .filter_map(|f| {
                    let owner = &modules[owners.owner(&f.path, &f.rule)?];
                    Some(crate::ModuleFile {
                        path: f.path.clone(),
                        lines: f.counts.lines,
                        code: f.counts.code,
                        module: owner.path.clone(),
                        detector: owner.detector.clone(),
                    })
                })
                .collect();
            ensure!(
                crate::report::rank_module_files(measured) == *recorded,
                "largest_module_files conflicts with measured files"
            );
        }
        if let Some(review) = &self.review {
            unique(review.iter().map(|r| &r.path), "review")?;
            for row in review {
                path(&row.path, Root::Rejected)?;
                ensure!(row.code <= row.lines, "review code exceeds lines");
            }
        }
        Ok(())
    }

    /// Requires full-report evidence for schema-3 validity and aggregate comparison; headlines lack this detail.
    pub fn require_comparison_evidence(&self) -> Result<()> {
        let mut missing = Vec::new();
        if self.review.is_none() {
            missing.push("review".into());
        }
        if self.breakdown.is_none() {
            missing.push("breakdown".into());
        }
        if self.metadata.kinds.is_none() {
            missing.push("metadata.kinds".into());
        }
        if self.metadata.modules.is_none() {
            missing.push("metadata.modules".into());
        }
        if self.metadata.coverage.unrecognized.is_none() {
            missing.push("metadata.coverage.unrecognized".into());
        }
        match &self.metadata.members {
            Some(members) => {
                for member in members {
                    if member.config.is_none() {
                        missing.push(format!("metadata.members[{}].config", member.name));
                    }
                }
            }
            None => {
                if self.metadata.config.is_none() {
                    missing.push("metadata.config".into());
                }
            }
        }
        if !missing.is_empty() {
            bail!(
                "A full measurement report is required. Missing: {}. Capture a full measurement with '--output FILE' or '--format json'. A headline cannot reconstruct the missing baseline, and rerunning measures the current tree.",
                missing.join(", ")
            );
        }
        Ok(())
    }
}
