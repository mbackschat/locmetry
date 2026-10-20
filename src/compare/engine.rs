use super::*;
use crate::paths::{ancestors, member_of, parent, ROOT};
use anyhow::{ensure, Context};
use std::{collections::BTreeSet, fs, path::Path};

pub fn read_input(path: &Path) -> Result<MeasurementReport> {
    /// Only the version: recognizing an old report skips every other field
    /// instead of building a document tree of a possibly large report.
    #[derive(serde::Deserialize)]
    struct Probe {
        schema_version: Option<String>,
    }
    let bytes = fs::read(path).with_context(|| format!("Cannot read {}", path.display()))?;
    let invalid = || format!("Invalid measurement report {}", path.display());
    let probe: Probe = serde_json::from_slice(&bytes).with_context(invalid)?;
    // Version 1 used the purpose names retired in 0.5; translating them would
    // keep two vocabularies alive, so old reports are re-measured instead.
    ensure!(
        probe.schema_version.as_deref() != Some("1"),
        "{} has report schema version 1 from locmetry 0.4 or earlier; measure again with this version to compare",
        path.display()
    );
    serde_json::from_slice(&bytes).with_context(invalid)
}

pub fn validate_destination(path: &Path, inputs: [&Path; 2]) -> Result<()> {
    if path.exists() {
        let target = path.canonicalize()?;
        for input in inputs {
            ensure!(
                target != input.canonicalize()?,
                "Comparison output must not overwrite input {}",
                input.display()
            );
            #[cfg(unix)]
            {
                use std::os::unix::fs::MetadataExt;
                let a = fs::metadata(path)?;
                let b = fs::metadata(input)?;
                ensure!(
                    a.dev() != b.dev() || a.ino() != b.ino(),
                    "Comparison output must not overwrite a hard-link alias of input {}",
                    input.display()
                );
            }
        }
    }
    Ok(())
}

fn keys<K: Ord + Clone, V, W>(a: &BTreeMap<K, V>, b: &BTreeMap<K, W>) -> BTreeSet<K> {
    a.keys().chain(b.keys()).cloned().collect()
}
fn changed_map<K: Ord + Clone>(
    a: &BTreeMap<K, Counts>,
    b: &BTreeMap<K, Counts>,
) -> Result<BTreeMap<K, CountChange>> {
    keys(a, b)
        .into_iter()
        .map(|k| {
            Ok((
                k.clone(),
                CountChange::new(
                    a.get(&k).copied().unwrap_or_default(),
                    b.get(&k).copied().unwrap_or_default(),
                )?,
            ))
        })
        .collect()
}
pub(super) fn dimensions(a: &crate::Summary, b: &crate::Summary) -> Result<Dimensions> {
    Ok(Dimensions {
        all_counted: CountChange::new(a.all_counted, b.all_counted)?,
        source: CountChange::new(a.source, b.source)?,
        by_kind_purpose: changed_map(&a.by_kind_purpose, &b.by_kind_purpose)?,
        source_purpose: changed_map(&a.source_purpose, &b.source_purpose)?,
        source_categories: changed_map(&a.source_categories, &b.source_categories)?,
        source_languages: changed_map(&a.source_languages, &b.source_languages)?,
    })
}
fn presence(before: bool, after: bool) -> Presence {
    match (before, after) {
        (true, true) => Presence::Both,
        (true, false) => Presence::BeforeOnly,
        (false, true) => Presence::AfterOnly,
        (false, false) => unreachable!("union member exists on a side"),
    }
}
fn count_presence(counts: &mut PresenceCounts, p: Presence) {
    match p {
        Presence::BeforeOnly => counts.before_only += 1,
        Presence::AfterOnly => counts.after_only += 1,
        Presence::Both => {}
    }
}
fn context(
    side: Side,
    message: impl Into<String>,
    member: Option<String>,
    paths: &[&str],
) -> FindingContext {
    FindingContext {
        side,
        message: message.into(),
        member,
        paths: paths.iter().map(|s| s.to_string()).collect(),
    }
}
fn presence_finding(
    entity: Entity,
    p: Presence,
    member: Option<String>,
    paths: &[&str],
) -> Option<Finding> {
    if p == Presence::Both {
        return None;
    }
    let (side, message) = match (&entity, p) {
        (Entity::Member { .. }, Presence::BeforeOnly) => (
            Side::Before,
            format!(
                "{} left the workspace measurement; its source may still exist.",
                entity.named()
            ),
        ),
        (Entity::Member { .. }, _) => (
            Side::After,
            format!("{} joined the workspace measurement.", entity.named()),
        ),
        (entity, Presence::BeforeOnly) => (
            Side::Before,
            format!(
                "{} appears only in the before report; it may be deleted or no longer measured.",
                entity.named()
            ),
        ),
        (entity, _) => (
            Side::After,
            format!(
                "{} appears only in the after report; it may be new or newly measured.",
                entity.named()
            ),
        ),
    };
    Some(Finding::PresenceChanged {
        context: context(side, message, member, paths),
        entity,
        presence: p,
    })
}

#[derive(Clone, Copy, Default)]
struct Pair {
    all: Counts,
    source: Counts,
}
#[derive(Default)]
struct Folder {
    evidence: BTreeSet<&'static str>,
    direct: Pair,
    recursive: Pair,
}
/// `path` and the folders holding it, up to the root, or in a workspace up
/// to its member, whose totals the workspace root does not repeat.
fn lineage(path: &str, workspace: bool) -> impl Iterator<Item = &str> {
    let mut done = false;
    std::iter::once(path)
        .chain(ancestors(path))
        .take_while(move |folder| {
            let take = !done;
            done = *folder == ROOT || (workspace && !folder.contains('/'));
            take
        })
}
fn folders(report: &MeasurementReport) -> Result<BTreeMap<String, Folder>> {
    let mut result = BTreeMap::<String, Folder>::new();
    let workspace = report.metadata.members.is_some();
    let mut add_path = |path: &str, evidence: &'static str| {
        for folder in lineage(path, workspace) {
            result
                .entry(folder.into())
                .or_default()
                .evidence
                .insert(evidence);
        }
    };
    match &report.metadata.members {
        Some(members) => {
            for m in members {
                add_path(&m.name, "/metadata/members");
            }
        }
        None => add_path(".", "/metadata/root"),
    }
    for row in report.breakdown.iter().flatten() {
        add_path(&row.folder, "/breakdown");
    }
    for module in report.metadata.modules.iter().flatten() {
        add_path(&module.path, "/metadata/modules");
    }
    let coverage = &report.metadata.coverage;
    for path in coverage.unrecognized.iter().flatten() {
        add_path(parent(path), "/metadata/coverage/unrecognized");
    }
    for path in &coverage.unread {
        add_path(parent(path), "/metadata/coverage/unread");
    }
    for failure in coverage.parse_failures.iter().flatten() {
        add_path(parent(&failure.path), "/metadata/coverage/parse_failures");
    }
    for row in report.breakdown.iter().flatten() {
        let mut direct = true;
        for path in lineage(&row.folder, workspace) {
            let folder = result.get_mut(path).expect("ancestor inserted");
            folder.recursive.all = folder.recursive.all.checked_add(&row.counts)?;
            if row.kind == Kind::Source {
                folder.recursive.source = folder.recursive.source.checked_add(&row.counts)?;
            }
            if direct {
                folder.direct.all = folder.direct.all.checked_add(&row.counts)?;
                if row.kind == Kind::Source {
                    folder.direct.source = folder.direct.source.checked_add(&row.counts)?;
                }
                direct = false;
            }
        }
    }
    Ok(result)
}
fn folder_counts(a: Pair, b: Pair) -> Result<FolderCounts> {
    Ok(FolderCounts {
        all_counted: CountChange::new(a.all, b.all)?,
        source: CountChange::new(a.source, b.source)?,
    })
}
fn packages(report: &MeasurementReport) -> BTreeSet<(String, String)> {
    report
        .metadata
        .modules
        .iter()
        .flatten()
        .map(|m| (m.path.clone(), m.detector.clone()))
        .collect()
}
fn member_for(path: &str, workspace: bool) -> Option<String> {
    workspace.then(|| member_of(path).into())
}

pub fn compare(
    mut before: MeasurementReport,
    mut after: MeasurementReport,
    before_name: String,
    after_name: String,
) -> Result<Comparison> {
    before
        .validate()
        .with_context(|| format!("Invalid before measurement {before_name}"))?;
    after
        .validate()
        .with_context(|| format!("Invalid after measurement {after_name}"))?;
    before
        .require_comparison_evidence()
        .with_context(|| format!("Cannot compare before report {before_name}"))?;
    after
        .require_comparison_evidence()
        .with_context(|| format!("Cannot compare after report {after_name}"))?;
    let workspace = before.metadata.members.is_some();
    ensure!(
        workspace == after.metadata.members.is_some(),
        "Cannot compare single-root and workspace measurements"
    );
    let restriction = |report: &MeasurementReport| {
        report
            .metadata
            .scope
            .as_ref()
            .and_then(|s| s.subfolder.as_deref())
            .map(str::to_owned)
    };
    ensure!(
        restriction(&before) == restriction(&after),
        "Cannot compare different subfolder restrictions: before: {}, after: {}",
        restriction(&before).as_deref().unwrap_or("unrestricted"),
        restriction(&after).as_deref().unwrap_or("unrestricted")
    );
    normalize_metadata(&mut before.metadata);
    normalize_metadata(&mut after.metadata);
    let mut result = Comparison {
        document_type: DocumentType::Comparison,
        schema_version: Version::V2,
        before: Input {
            filename: before_name,
            project: before.project.clone(),
            metadata: before.metadata.clone(),
        },
        after: Input {
            filename: after_name,
            project: after.project.clone(),
            metadata: after.metadata.clone(),
        },
        summary: Summary {
            dimensions: dimensions(&before.summary.totals, &after.summary.totals)?,
            composition: None,
        },
        member: None,
        members: None,
        folders: None,
        packages: None,
        breakdown: None,
        presence_summary: PresenceSummary::default(),
        findings: settings_findings(&before, &after)?,
        report: None,
    };
    if workspace {
        let mut members = BTreeMap::new();
        let mut matched_before = Pair::default();
        let mut matched_after = Pair::default();
        let mut entering = Pair::default();
        let mut leaving = Pair::default();
        for name in keys(&before.summary.members, &after.summary.members) {
            let a = before.summary.members.get(&name);
            let b = after.summary.members.get(&name);
            let p = presence(a.is_some(), b.is_some());
            count_presence(&mut result.presence_summary.members, p);
            let zero = crate::Summary::default();
            let changes = dimensions(a.unwrap_or(&zero), b.unwrap_or(&zero))?;
            match p {
                Presence::Both => {
                    add_pair(&mut matched_before, a.unwrap())?;
                    add_pair(&mut matched_after, b.unwrap())?;
                }
                Presence::AfterOnly => add_pair(&mut entering, b.unwrap())?,
                Presence::BeforeOnly => add_pair(&mut leaving, a.unwrap())?,
            }
            result.findings.extend(presence_finding(
                Entity::Member { name: name.clone() },
                p,
                Some(name.clone()),
                &["/metadata/members"],
            ));
            members.insert(
                name,
                MemberChange {
                    presence: p,
                    dimensions: changes,
                },
            );
        }
        result.summary.composition = Some(Composition {
            all_counted: Contribution {
                matched: CountChange::new(matched_before.all, matched_after.all)?.delta,
                entering: entering.all,
                leaving: leaving.all,
            },
            source: Contribution {
                matched: CountChange::new(matched_before.source, matched_after.source)?.delta,
                entering: entering.source,
                leaving: leaving.source,
            },
        });
        result.members = Some(members);
    }
    let a = folders(&before)?;
    let b = folders(&after)?;
    let mut folder_rows = BTreeMap::new();
    for path in keys(&a, &b) {
        let p = presence(a.contains_key(&path), b.contains_key(&path));
        count_presence(&mut result.presence_summary.folders, p);
        let empty = Folder::default();
        let av = a.get(&path).unwrap_or(&empty);
        let bv = b.get(&path).unwrap_or(&empty);
        folder_rows.insert(
            path.clone(),
            FolderChange {
                presence: p,
                direct: folder_counts(av.direct, bv.direct)?,
                recursive: folder_counts(av.recursive, bv.recursive)?,
            },
        );
        result.findings.extend(presence_finding(
            Entity::Folder { path: path.clone() },
            p,
            member_for(&path, workspace),
            &match p {
                Presence::BeforeOnly => &av.evidence,
                _ => &bv.evidence,
            }
            .iter()
            .copied()
            .collect::<Vec<_>>(),
        ));
    }
    result.folders = Some(folder_rows);
    let a = packages(&before);
    let b = packages(&after);
    let mut package_rows = Vec::new();
    for key in a.union(&b) {
        let p = presence(a.contains(key), b.contains(key));
        count_presence(&mut result.presence_summary.packages, p);
        package_rows.push(PackageChange {
            folder: key.0.clone(),
            detector: key.1.clone(),
            presence: p,
        });
        result.findings.extend(presence_finding(
            Entity::Package {
                path: key.0.clone(),
                detector: key.1.clone(),
            },
            p,
            member_for(&key.0, workspace),
            &["/metadata/modules"],
        ));
    }
    result.packages = Some(package_rows);
    let cells = |r: &MeasurementReport| {
        r.breakdown
            .iter()
            .flatten()
            .map(|r| (r.key(), r.counts))
            .collect::<BTreeMap<_, _>>()
    };
    let a = cells(&before);
    let b = cells(&after);
    result.breakdown = Some(
        changed_map(&a, &b)?
            .into_iter()
            .map(
                |((folder, kind, purpose, category, language), counts)| BreakdownChange {
                    folder,
                    kind,
                    purpose,
                    category,
                    language,
                    counts,
                },
            )
            .collect(),
    );
    result.findings.sort_by_key(|f| {
        (
            f.code(),
            f.context().member.clone(),
            f.context().side,
            f.context().paths.clone(),
            serde_json::to_string(f).expect("serializable finding"),
        )
    });
    Ok(result)
}
fn add_pair(pair: &mut Pair, s: &crate::Summary) -> Result<()> {
    pair.all = pair.all.checked_add(&s.all_counted)?;
    pair.source = pair.source.checked_add(&s.source)?;
    Ok(())
}

fn normalize_metadata(m: &mut Metadata) {
    if let Some(modules) = &mut m.modules {
        modules.sort_by(|a, b| (&a.path, &a.detector).cmp(&(&b.path, &b.detector)));
    }
    if let Some(members) = &mut m.members {
        members.sort_by(|a, b| a.name.cmp(&b.name));
    }
    if let Some(paths) = &mut m.coverage.unrecognized {
        paths.sort();
    }
    m.coverage.unread.sort();
    if let Some(failures) = &mut m.coverage.parse_failures {
        failures.sort_by(|a, b| a.path.cmp(&b.path));
    }
}
fn policy(
    scope: &crate::contract::Selection,
    config: &crate::EffectiveConfig,
) -> Result<serde_json::Value> {
    let mut value = serde_json::json!({"scope":scope,"config":config});
    for pointer in ["/scope/exclude", "/config/scope/exclude"] {
        if let Some(serde_json::Value::Array(values)) = value.pointer_mut(pointer) {
            values.sort_by_key(serde_json::Value::to_string);
            values.dedup();
        }
    }
    Ok(value)
}
fn settings_findings(
    before: &MeasurementReport,
    after: &MeasurementReport,
) -> Result<Vec<Finding>> {
    let a = &before.metadata;
    let b = &after.metadata;
    let mut findings = Vec::new();
    let counter_paths = [
        (
            "/metadata/locmetry_version",
            a.locmetry_version != b.locmetry_version,
        ),
        ("/metadata/backend", a.backend != b.backend),
        (
            "/metadata/backend_configuration",
            a.backend_configuration != b.backend_configuration,
        ),
    ]
    .into_iter()
    .filter_map(|(p, diff)| diff.then_some(p))
    .collect::<Vec<_>>();
    if !counter_paths.is_empty() {
        findings.push(Finding::CounterChanged {
            context: context(
                Side::Both,
                "Recorded producer or counter settings differ.",
                None,
                &counter_paths,
            ),
        });
    }
    if before.project != after.project {
        findings.push(Finding::IdentityChanged {
            context: context(
                Side::Both,
                "Project names differ; pairing does not verify repository identity.",
                None,
                &["/project"],
            ),
        });
    }
    if serde_json::to_value(&a.kinds)? != serde_json::to_value(&b.kinds)? {
        findings.push(Finding::PolicyChanged {
            context: context(
                Side::Both,
                "Content-kind mappings differ.",
                None,
                &["/metadata/kinds"],
            ),
        });
    }
    if packages(before) != packages(after) {
        findings.push(Finding::ModulesChanged {
            context: context(
                Side::Both,
                "Recorded build-module sets differ.",
                None,
                &["/metadata/modules"],
            ),
        });
    }
    match (&a.members, &b.members) {
        (Some(a), Some(b)) => {
            for member in a {
                if let Some(other) = b.iter().find(|m| m.name == member.name) {
                    if policy(&member.scope, member.config.as_ref().expect("eligible"))?
                        != policy(&other.scope, other.config.as_ref().expect("eligible"))?
                    {
                        findings.push(Finding::PolicyChanged {
                            context: context(
                                Side::Both,
                                "Recorded member scope or effective policy differs.",
                                Some(member.name.clone()),
                                &["/metadata/members"],
                            ),
                        });
                    }
                }
            }
        }
        (None, None) => {
            if policy(
                a.scope.as_ref().expect("validated"),
                a.config.as_ref().expect("eligible"),
            )? != policy(
                b.scope.as_ref().expect("validated"),
                b.config.as_ref().expect("eligible"),
            )? {
                findings.push(Finding::PolicyChanged {
                    context: context(
                        Side::Both,
                        "Recorded scope or effective policy differs.",
                        None,
                        &["/metadata/scope", "/metadata/config"],
                    ),
                });
            }
        }
        _ => unreachable!("mode checked"),
    }
    for (side, m) in [(Side::Before, a), (Side::After, b)] {
        for (n, path, description) in [
            (
                m.coverage.unrecognized_files,
                "/metadata/coverage/unrecognized",
                "unrecognized files",
            ),
            (
                m.coverage.unread.len() as u64,
                "/metadata/coverage/unread",
                "unreadable files",
            ),
            (
                m.coverage.parse_failures.as_ref().map_or(0, |f| f.len()) as u64,
                "/metadata/coverage/parse_failures",
                "files with parsing/counting failures",
            ),
        ] {
            if n > 0 {
                findings.push(Finding::CoverageGap {
                    folder: None,
                    context: context(
                        side,
                        format!("{n} {description}; their counts are unknown, not zero."),
                        None,
                        &[path],
                    ),
                });
            }
        }
        if m.coverage.parse_failures.is_none() {
            findings.push(Finding::CoverageGap{folder:None,context:context(side,"Notebook parsing failure evidence was not recorded; historic counts are not certified.",None,&["/metadata/coverage/parse_failures"])});
        }
        if m.backend_inaccurate {
            findings.push(Finding::CoverageGap {
                folder: None,
                context: context(
                    side,
                    "Input is marked inaccurate; net change compares recorded measured subsets.",
                    None,
                    &["/metadata/backend_inaccurate"],
                ),
            });
        }
        let unavailable = m
            .coverage
            .unrecognized
            .iter()
            .flatten()
            .chain(&m.coverage.unread)
            .chain(m.coverage.parse_failures.iter().flatten().map(|f| &f.path));
        let affected: BTreeSet<_> = unavailable.map(|path| parent(path).to_string()).collect();
        for folder in affected {
            findings.push(Finding::CoverageGap {
                context: context(side, format!("Folder {folder} contains unavailable counts; its recorded totals and recursive parents cover measured subsets only."), member_for(&folder, m.members.is_some()), &["/metadata/coverage"]),
                folder: Some(folder),
            });
        }
        for member in m.members.iter().flatten() {
            if member.coverage.parse_failed_files.is_none() {
                findings.push(Finding::CoverageGap {
                    folder: None,
                    context: context(
                        side,
                        "Member parsing failure evidence was not recorded.",
                        Some(member.name.clone()),
                        &["/metadata/members"],
                    ),
                });
            }
        }
    }
    Ok(findings)
}

impl Comparison {
    /// Focus a full workspace comparison on one exact member name, retaining input provenance.
    ///
    /// Report-level findings and partial status still describe the complete input measurements;
    /// selecting an unaffected member cannot certify a partially measured workspace.
    pub fn select_member(mut self, name: &str) -> Result<Self> {
        let members = self
            .members
            .as_mut()
            .context("--member requires two workspace reports")?;
        let selected = members.get(name).with_context(|| {
            format!(
                "Unknown workspace member {name:?}; available: {}",
                members.keys().cloned().collect::<Vec<_>>().join(", ")
            )
        })?;
        let contribution = |counts: &CountChange| {
            let mut value = Contribution {
                matched: SignedCounts::default(),
                entering: Counts::default(),
                leaving: Counts::default(),
            };
            match selected.presence {
                Presence::Both => value.matched = counts.delta,
                Presence::BeforeOnly => value.leaving = counts.before,
                Presence::AfterOnly => value.entering = counts.after,
            }
            value
        };
        self.summary = Summary {
            dimensions: selected.dimensions.clone(),
            composition: Some(Composition {
                source: contribution(&selected.dimensions.source),
                all_counted: contribution(&selected.dimensions.all_counted),
            }),
        };
        let mut presence = PresenceSummary::default();
        count_presence(&mut presence.members, selected.presence);
        members.retain(|member, _| member == name);
        let folders = self
            .folders
            .as_mut()
            .context("Member selection requires full folder evidence")?;
        folders.retain(|path, _| crate::paths::within(path, name));
        for folder in folders.values() {
            count_presence(&mut presence.folders, folder.presence);
        }
        let packages = self
            .packages
            .as_mut()
            .context("Member selection requires full package evidence")?;
        packages.retain(|package| crate::paths::within(&package.folder, name));
        for package in packages {
            count_presence(&mut presence.packages, package.presence);
        }
        self.breakdown
            .as_mut()
            .context("Member selection requires full breakdown evidence")?
            .retain(|row| crate::paths::within(&row.folder, name));
        self.findings.retain(|finding| {
            finding
                .context()
                .member
                .as_deref()
                .is_none_or(|member| member == name)
        });
        self.presence_summary = presence;
        self.member = Some(name.to_string());
        Ok(self)
    }
}
