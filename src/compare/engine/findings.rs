//! Comparison findings beyond counts: changed producer and policy settings, and folder or member presence.
use super::*;

pub(super) fn presence(before: bool, after: bool) -> Presence {
    match (before, after) {
        (true, true) => Presence::Both,
        (true, false) => Presence::BeforeOnly,
        (false, true) => Presence::AfterOnly,
        (false, false) => unreachable!("union member exists on a side"),
    }
}

pub(super) fn count_presence(counts: &mut PresenceCounts, p: Presence) {
    match p {
        Presence::BeforeOnly => counts.before_only += 1,
        Presence::AfterOnly => counts.after_only += 1,
        Presence::Both => {}
    }
}

pub(super) fn context(
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

pub(super) fn presence_finding(
    entity: Entity,
    p: Presence,
    member: Option<String>,
    paths: &[&str],
) -> Option<Finding> {
    let side = match p {
        Presence::Both => return None,
        Presence::BeforeOnly => Side::Before,
        Presence::AfterOnly => Side::After,
    };
    Some(Finding::PresenceChanged {
        context: context(side, entity.presence_message(p, ""), member, paths),
        entity,
        presence: p,
    })
}

pub(super) fn normalize_metadata(m: &mut Metadata) {
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

pub(super) fn policy(
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

pub(super) fn settings_findings(
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
                "Project names differ. Pairing does not verify repository identity.",
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
                ("unrecognized file", "unrecognized files"),
            ),
            (
                m.coverage.unread.len() as u64,
                "/metadata/coverage/unread",
                ("unreadable file", "unreadable files"),
            ),
            (
                m.coverage.parse_failures.as_ref().map_or(0, |f| f.len()) as u64,
                "/metadata/coverage/parse_failures",
                (
                    "file with a parsing or counting failure",
                    "files with parsing or counting failures",
                ),
            ),
        ] {
            if n > 0 {
                findings.push(Finding::CoverageGap {
                    folder: None,
                    context: context(
                        side,
                        format!(
                            "{n} {}. Their counts are unknown, not zero.",
                            if n == 1 { description.0 } else { description.1 }
                        ),
                        None,
                        &[path],
                    ),
                });
            }
        }
        if m.coverage.parse_failures.is_none() {
            findings.push(Finding::CoverageGap{folder:None,context:context(side,"Notebook parsing failure evidence was not recorded. Historic counts are not certified.",None,&["/metadata/coverage/parse_failures"])});
        }
        if m.backend_inaccurate {
            findings.push(Finding::CoverageGap {
                folder: None,
                context: context(
                    side,
                    "Input is marked inaccurate. Net change compares recorded measured subsets.",
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
                context: context(side, format!("Folder {} contains unavailable counts. Its recorded totals and recursive parents cover measured subsets only.", crate::output::quoted(&folder)), member_for(&folder, m.members.is_some()), &["/metadata/coverage"]),
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
