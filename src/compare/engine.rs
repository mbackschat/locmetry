use super::*;
use crate::paths::{ancestors, member_of, parent, ROOT};
use anyhow::{ensure, Context};
use std::{collections::BTreeSet, fs, io::Read, path::Path};

mod findings;
mod folders;
use findings::*;
use folders::*;

/// Reads a saved report. A byte-order mark selects UTF-8 or UTF-16, because
/// Windows PowerShell 5.1 writes UTF-16LE for `>` and a UTF-8 mark for
/// `Out-File -Encoding utf8` (docs/proposals/archive/windows-support.md); without a
/// mark, the bytes pass through unchanged as UTF-8.
pub fn read_input(path: &Path) -> Result<MeasurementReport> {
    let bytes = fs::read(path).with_context(|| format!("Cannot read '{}'", path.display()))?;
    let mut text = String::new();
    encoding_rs_io::DecodeReaderBytesBuilder::new()
        .build(bytes.as_slice())
        .read_to_string(&mut text)
        .with_context(|| format!("Invalid measurement report encoding '{}'", path.display()))?;
    serde_json::from_str(&text)
        .with_context(|| format!("Invalid measurement report '{}'", path.display()))
}

/// Refuses an output that is an input under another name. File identity,
/// rather than path comparison, also catches symbolic and hard links, on
/// Windows through the volume serial and file index.
pub fn validate_destination(path: &Path, inputs: [&Path; 2]) -> Result<()> {
    if path.exists() {
        for input in inputs {
            ensure!(
                !same_file::is_same_file(path, input)?,
                "Comparison output must not overwrite input '{}'",
                input.display()
            );
        }
    }
    Ok(())
}

fn keys<K: Ord + Clone, V, W>(a: &BTreeMap<K, V>, b: &BTreeMap<K, W>) -> BTreeSet<K> {
    a.keys().chain(b.keys()).cloned().collect()
}
pub(super) fn changed_map<K: Ord + Clone>(
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
        detail: Some(Detail::Full),
        schema_version: Version::V3,
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
impl Comparison {
    /// Focus a full workspace comparison on one exact member name, retaining input provenance.
    ///
    /// Report-level findings and partial status still describe the complete input measurements;
    /// selecting an unaffected member cannot certify a partially measured workspace.
    pub fn select_member(mut self, name: &str) -> Result<Self> {
        let members = self
            .members
            .as_mut()
            .context("'--member' requires two workspace reports")?;
        let selected = members.get(name).with_context(|| {
            format!(
                "Unknown workspace member '{name}'. Available: {}",
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
