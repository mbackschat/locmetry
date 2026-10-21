//! Semantic validation of stored evidence; comparison eligibility is a separate check.
use super::*;
use crate::paths::{below, member_of, within};
use anyhow::{bail, ensure, Context, Result};
use std::collections::BTreeSet;

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
        "invalid root-relative path {path:?}"
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
        "invalid purpose key {key:?}"
    );
    Ok(())
}
fn validate_totals(s: &Summary, context: &str) -> Result<()> {
    s.all_counted.validate(context)?;
    s.source.validate(context)?;
    for counts in [&s.all_counted, &s.source]
        .into_iter()
        .chain(s.by_kind_purpose.values())
        .chain(s.source_purpose.values())
        .chain(s.source_categories.values())
        .chain(s.source_languages.values())
    {
        ensure!(
            counts.files > 0 || counts.lines == 0,
            "{context}: nonzero totals need physical files"
        );
    }
    for counts in s.source_languages.values() {
        ensure!(
            counts.files <= s.source.files,
            "{context}: language participation exceeds source files"
        );
    }
    for c in s
        .by_kind_purpose
        .values()
        .chain(s.source_purpose.values())
        .chain(s.source_categories.values())
        .chain(s.source_languages.values())
    {
        c.validate(context)?;
    }
    for key in s.by_kind_purpose.keys() {
        let (kind, purpose) = key.split_once(':').context("kind:purpose key required")?;
        ensure!(
            Kind::ALL.iter().any(|k| k.to_string() == kind),
            "invalid kind {kind}"
        );
        purpose_key(purpose, PurposeKey::Purpose)?;
    }
    for key in s.source_categories.keys() {
        purpose_key(key, PurposeKey::Category)?;
    }
    ensure!(
        total(s.by_kind_purpose.values())? == s.all_counted,
        "{context}: by_kind_purpose totals conflict with all_counted"
    );
    ensure!(
        total(s.source_purpose.values())? == s.source,
        "{context}: source_purpose totals conflict with source"
    );
    ensure!(
        total(s.source_categories.values())? == s.source,
        "{context}: source_categories totals conflict with source"
    );
    ensure!(same_lines(total(s.source_languages.values())?,s.source), "{context}: source_languages line totals conflict with source (file participation may overlap)");
    for p in Purpose::ALL {
        let code = s.source_purpose.get(&p).copied().unwrap_or_default();
        ensure!(
            s.by_kind_purpose
                .get(&format!("source:{p}"))
                .copied()
                .unwrap_or_default()
                == code,
            "{context}: source kind/purpose totals conflict"
        );
        let categories = total(
            s.source_categories
                .iter()
                .filter(|(key, _)| key.starts_with(&format!("{p}:")))
                .map(|(_, c)| c),
        )?;
        ensure!(
            categories == code,
            "{context}: category totals conflict for {p}"
        );
    }
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
                "metadata: single-root fields required; workspace_path needs members"
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
                "measured path {p:?} is outside subfolder {folder:?}"
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
                "measured folder {p:?} is outside subfolder {folder:?}"
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
        if let (Some(files), Some(modules)) = (&self.files, &self.metadata.modules) {
            let mut contributions = BTreeMap::<(&str, &str), u64>::new();
            let owners = crate::modules::ModuleOwners::new(
                modules
                    .iter()
                    .map(|m| (m.path.as_str(), m.detector.as_str())),
            );
            for file in files.iter().filter(|f| f.kind == Kind::Source) {
                if let Some(index) = owners.owner(&file.path, &file.rule) {
                    let owner = &modules[index];
                    let code = contributions
                        .entry((&owner.detector, &owner.path))
                        .or_default();
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
        }
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
            let mut largest: Vec<(u64, u64, &str, &str, &str)> = files
                .iter()
                .filter(|f| f.kind == Kind::Source)
                .filter_map(|f| {
                    let owner = &modules[owners.owner(&f.path, &f.rule)?];
                    Some((
                        f.counts.code,
                        f.counts.lines,
                        &*f.path,
                        &*owner.path,
                        &*owner.detector,
                    ))
                })
                .collect();
            largest.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.2.cmp(b.2)));
            largest.truncate(crate::report::MODULE_FILES);
            let recorded = recorded.iter().map(|f| {
                (
                    f.code,
                    f.lines,
                    f.path.as_str(),
                    f.module.as_str(),
                    f.detector.as_str(),
                )
            });
            ensure!(
                largest.into_iter().eq(recorded),
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

    fn validate_breakdown(&self) -> Result<()> {
        if let Some(rows) = &self.breakdown {
            unique(
                rows.iter()
                    .map(|r| (&r.folder, r.kind, r.purpose, &r.category, &r.language)),
                "breakdown",
            )?;
            for row in rows {
                path(&row.folder, Root::Allowed)?;
                row.counts.validate("breakdown")?;
            }
            let derived = totals_from_rows(rows.iter())?;
            same_totals(&self.summary.totals, &derived, Files::AtLeast, "breakdown")?;
            let mut top = BTreeMap::<String, BTreeMap<Purpose, u64>>::new();
            for row in rows.iter().filter(|r| r.kind == Kind::Source) {
                let name = member_of(&row.folder);
                let n = top
                    .entry(name.into())
                    .or_default()
                    .entry(row.purpose)
                    .or_default();
                *n = checked_sum([*n, row.counts.code], "top_folders")?;
            }
            ensure!(
                top == self.summary.top_folders,
                "top_folders conflicts with breakdown"
            );
        }
        Ok(())
    }

    fn validate_files(&self, coverage_paths: &[&String]) -> Result<()> {
        let c = &self.metadata.coverage;
        if let Some(files) = &self.files {
            unique(files.iter().map(|f| &f.path), "files")?;
            ensure!(
                files.len() as u64 == c.counted_files,
                "files count conflicts with coverage"
            );
            for file in files {
                path(&file.path, Root::Rejected)?;
                Counts::from(file.counts).validate("file")?;
                ensure!(
                    !coverage_paths.contains(&&file.path),
                    "file is both counted and unavailable"
                );
                if let Some(languages) = &file.languages {
                    ensure!(
                        languages.contains_key(&file.language),
                        "file languages must retain host participation"
                    );
                    let counts = languages
                        .values()
                        .map(|c| Counts::from(*c))
                        .collect::<Vec<_>>();
                    for c in &counts {
                        c.validate("file languages")?;
                    }
                    ensure!(
                        same_lines(total(&counts)?, file.counts.into()),
                        "file languages do not reconcile"
                    );
                }
            }
            // Reuse measurement aggregation after checked bounds above; no second counting policy.
            let records = files
                .iter()
                .map(|f| crate::FileRecord {
                    path: f.path.clone(),
                    language: f.language.clone(),
                    kind: f.kind,
                    language_mapping: f.language_mapping.clone(),
                    classification: crate::Classification {
                        purpose: f.purpose,
                        category: f.category.clone(),
                        rule: f.rule.clone(),
                    },
                    counts: f.counts.into(),
                    languages: match &f.languages {
                        Some(languages) => languages
                            .iter()
                            .map(|(k, v)| (k.clone(), (*v).into()))
                            .collect(),
                        None => BTreeMap::from([(f.language.clone(), f.counts.into())]),
                    },
                })
                .collect::<Vec<_>>();
            let file_counts = records.iter().map(|f| f.counts).collect::<Vec<_>>();
            ensure!(
                total(&file_counts)? == self.summary.totals.all_counted,
                "file totals conflict with summary"
            );
            let derived = crate::build_report(&records);
            same_totals(&self.summary.totals, &derived.totals, Files::Exact, "files")?;
            if let Some(rows) = &self.breakdown {
                let expected: BTreeMap<_, _> = crate::breakdown(&records)
                    .iter()
                    .map(|r| (r.key(), r.counts))
                    .collect();
                let actual: BTreeMap<_, _> = rows.iter().map(|r| (r.key(), r.counts)).collect();
                ensure!(actual == expected, "files conflict with breakdown");
            }
            if let Some(review) = &self.review {
                let expected = crate::review(&records);
                let canonical = |rows: &[crate::ReviewEntry]| {
                    rows.iter()
                        .map(|r| {
                            (
                                r.path.clone(),
                                r.language.clone(),
                                r.category.clone(),
                                r.rule.clone(),
                                r.code,
                                r.lines,
                            )
                        })
                        .collect::<BTreeSet<_>>()
                };
                ensure!(
                    canonical(review) == canonical(&expected),
                    "files conflict with review"
                );
            }
        }
        Ok(())
    }

    fn validate_members(&self, coverage_paths: &[&String]) -> Result<()> {
        let m = &self.metadata;
        let c = &m.coverage;
        match &m.members {
            Some(members) => {
                let names = members
                    .iter()
                    .map(|m| m.name.clone())
                    .collect::<BTreeSet<_>>();
                ensure!(
                    names == self.summary.members.keys().cloned().collect(),
                    "summary.members identities conflict"
                );
                let mut merged = Summary::default();
                for member in members {
                    let totals = &self.summary.members[&member.name];
                    validate_totals(totals, &format!("member {}", member.name))?;
                    merge_totals(&mut merged, totals)?;
                    let c = &member.coverage;
                    ensure!(
                        checked_sum(
                            [
                                c.counted_files,
                                c.unrecognized_files,
                                c.unread_files,
                                c.parse_failed_files.unwrap_or_default()
                            ],
                            "member selected_files"
                        )? == c.selected_files,
                        "member selected_files does not reconcile"
                    );
                    ensure!(
                        c.counted_files == totals.all_counted.files
                            && c.mapped_files <= c.counted_files,
                        "member coverage conflicts with totals"
                    );
                    let member_path = |p: &str| below(p, &member.name).is_some();
                    if let Some(rows) = &self.breakdown {
                        let derived = totals_from_rows(
                            rows.iter().filter(|r| within(&r.folder, &member.name)),
                        )?;
                        same_totals(totals, &derived, Files::AtLeast, "member breakdown")?;
                    }
                    ensure!(
                        self.metadata
                            .coverage
                            .unread
                            .iter()
                            .filter(|p| member_path(p))
                            .count() as u64
                            == c.unread_files,
                        "member unread_files does not reconcile"
                    );
                    if let Some(paths) = &self.metadata.coverage.unrecognized {
                        ensure!(
                            paths.iter().filter(|p| member_path(p)).count() as u64
                                == c.unrecognized_files,
                            "member unrecognized_files does not reconcile"
                        );
                    }
                    if let (Some(failures), Some(n)) =
                        (&self.metadata.coverage.parse_failures, c.parse_failed_files)
                    {
                        ensure!(
                            failures.iter().filter(|f| member_path(&f.path)).count() as u64 == n,
                            "member parse_failed_files does not reconcile"
                        );
                    }
                }
                same_totals(
                    &self.summary.totals,
                    &merged,
                    Files::Exact,
                    "workspace members",
                )?;
                for (expected, values) in [
                    (
                        c.selected_files,
                        members
                            .iter()
                            .map(|m| m.coverage.selected_files)
                            .collect::<Vec<_>>(),
                    ),
                    (
                        c.counted_files,
                        members.iter().map(|m| m.coverage.counted_files).collect(),
                    ),
                    (
                        c.mapped_files,
                        members.iter().map(|m| m.coverage.mapped_files).collect(),
                    ),
                    (
                        c.unrecognized_files,
                        members
                            .iter()
                            .map(|m| m.coverage.unrecognized_files)
                            .collect(),
                    ),
                ] {
                    ensure!(
                        checked_sum(values, "workspace coverage")? == expected,
                        "workspace coverage does not reconcile"
                    );
                }
                let belongs = |p: &str| names.contains(member_of(p));
                for p in coverage_paths {
                    ensure!(belongs(p), "coverage path has no workspace member");
                }
                for row in self.breakdown.iter().flatten() {
                    ensure!(
                        belongs(&row.folder),
                        "breakdown folder has no workspace member"
                    );
                }
                for module in m.modules.iter().flatten() {
                    ensure!(belongs(&module.path), "module has no workspace member");
                }
            }
            None => ensure!(
                self.summary.members.is_empty(),
                "single-root summary cannot have members"
            ),
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
                "Full measurement report required; missing {}. Capture a full measurement with --output FILE or --format json. A headline cannot reconstruct the missing baseline; rerunning measures the current tree",
                missing.join(", ")
            );
        }
        Ok(())
    }
}

fn add<K: Ord>(map: &mut BTreeMap<K, Counts>, key: K, counts: &Counts) -> Result<()> {
    let value = map.entry(key).or_default();
    *value = value.checked_add(counts)?;
    Ok(())
}
fn totals_from_rows<'a>(rows: impl Iterator<Item = &'a crate::BreakdownRow>) -> Result<Summary> {
    let mut s = Summary::default();
    for r in rows {
        s.all_counted = s.all_counted.checked_add(&r.counts)?;
        add(
            &mut s.by_kind_purpose,
            crate::report::kind_purpose_key(r.kind, r.purpose),
            &r.counts,
        )?;
        if r.kind == Kind::Source {
            s.source = s.source.checked_add(&r.counts)?;
            add(&mut s.source_purpose, r.purpose, &r.counts)?;
            add(
                &mut s.source_categories,
                crate::report::category_key(r.purpose, &r.category),
                &r.counts,
            )?;
            add(&mut s.source_languages, r.language.clone(), &r.counts)?;
        }
    }
    Ok(s)
}
fn merge_totals(into: &mut Summary, s: &Summary) -> Result<()> {
    into.all_counted = into.all_counted.checked_add(&s.all_counted)?;
    into.source = into.source.checked_add(&s.source)?;
    for (k, v) in &s.by_kind_purpose {
        add(&mut into.by_kind_purpose, k.clone(), v)?;
    }
    for (k, v) in &s.source_purpose {
        add(&mut into.source_purpose, *k, v)?;
    }
    for (k, v) in &s.source_categories {
        add(&mut into.source_categories, k.clone(), v)?;
    }
    for (k, v) in &s.source_languages {
        add(&mut into.source_languages, k.clone(), v)?;
    }
    Ok(())
}
/// How file counts must agree: exactly, or at least as many when the expected
/// side derives from breakdown rows, which count a file only for its own
/// language, not for each language it contains.
#[derive(Clone, Copy)]
enum Files {
    Exact,
    AtLeast,
}

fn same_map<K: Ord>(
    actual: &BTreeMap<K, Counts>,
    expected: &BTreeMap<K, Counts>,
    files: Files,
) -> bool {
    actual.keys().chain(expected.keys()).all(|key| {
        let a = actual.get(key).copied().unwrap_or_default();
        let b = expected.get(key).copied().unwrap_or_default();
        match files {
            Files::Exact => a == b,
            Files::AtLeast => same_lines(a, b) && a.files >= b.files,
        }
    })
}

fn same_totals(
    actual: &Summary,
    expected: &Summary,
    language_files: Files,
    context: &str,
) -> Result<()> {
    ensure!(
        actual.all_counted == expected.all_counted
            && actual.source == expected.source
            && same_map(
                &actual.by_kind_purpose,
                &expected.by_kind_purpose,
                Files::Exact
            )
            && same_map(
                &actual.source_purpose,
                &expected.source_purpose,
                Files::Exact
            )
            && same_map(
                &actual.source_categories,
                &expected.source_categories,
                Files::Exact
            ),
        "{context}: aggregate totals conflict"
    );
    ensure!(
        same_map(
            &actual.source_languages,
            &expected.source_languages,
            language_files
        ),
        "{context}: language counts conflict"
    );
    Ok(())
}
