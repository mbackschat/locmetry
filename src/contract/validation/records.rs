//! Record-level evidence: breakdown rows, file records and workspace members,
//! each reconciled with the summary they claim to add up to.
use super::*;

impl MeasurementReport {
    pub(super) fn validate_breakdown(&self) -> Result<()> {
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

    pub(super) fn validate_files(&self, coverage_paths: &[&String]) -> Result<()> {
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
                if let Some(inline) = file.inline_tests {
                    Counts::from(inline).validate("file inline_tests")?;
                    ensure!(
                        file.kind == Kind::Source
                            && file.purpose != Purpose::Test
                            && inline.code <= file.counts.code
                            && inline.comments <= file.counts.comments
                            && inline.blanks <= file.counts.blanks,
                        "file inline_tests must lie within non-test source counts"
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
                    inline_tests: f.inline_tests.map(Counts::from).unwrap_or_default(),
                    module: f.module.clone(),
                })
                .collect::<Vec<_>>();
            let file_counts = records.iter().map(|f| f.counts).collect::<Vec<_>>();
            ensure!(
                total(&file_counts)? == self.summary.totals.all_counted,
                "file totals conflict with summary"
            );
            let derived = crate::build_report(&records);
            same_totals(&self.summary.totals, &derived.totals, Files::Exact, "files")?;
            ensure!(
                derived.totals.inline_tests == self.summary.totals.inline_tests,
                "files conflict with summary inline_tests"
            );
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

    pub(super) fn validate_members(&self, coverage_paths: &[&String]) -> Result<()> {
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
}
