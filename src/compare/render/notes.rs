use super::{Comparison, Metadata, Role};
use std::collections::BTreeSet;

fn set_summary<T: Ord>(before: &BTreeSet<T>, after: &BTreeSet<T>, unit: &str) -> String {
    if before == after {
        let plural = if before.len() == 1 { "" } else { "s" };
        format!("same {} {unit}{plural} in both reports", before.len())
    } else {
        let shared = before.intersection(after).count();
        format!(
            "{} → {} ({shared} in both, {} only before, {} only after)",
            before.len(),
            after.len(),
            before.len() - shared,
            after.len() - shared
        )
    }
}

fn failure_members<'a>(metadata: &Metadata, paths: impl Iterator<Item = &'a str>) -> String {
    let paths: Vec<_> = paths.collect();
    metadata
        .members
        .iter()
        .flatten()
        .filter(|member| {
            paths
                .iter()
                .any(|path| crate::paths::below(path, &member.name).is_some())
        })
        .map(|member| member.name.as_str())
        .collect::<Vec<_>>()
        .join(", ")
}

fn member_suffix(before: String, after: String) -> String {
    if before.is_empty() && after.is_empty() {
        return String::new();
    }
    let shown = |names: String| {
        if names.is_empty() {
            "none".into()
        } else {
            names
        }
    };
    format!(
        " Members before: {}; after: {}.",
        shown(before),
        shown(after)
    )
}

fn parse_failure_members(metadata: &Metadata) -> String {
    match (&metadata.members, &metadata.coverage.parse_failures) {
        (None, _) => String::new(),
        (Some(_), None) => "not recorded".into(),
        (Some(_), Some(failures)) => {
            failure_members(metadata, failures.iter().map(|f| f.path.as_str()))
        }
    }
}

pub(super) fn measurement_notes(c: &Comparison) -> Vec<(Role, String)> {
    let before = &c.before.metadata;
    let after = &c.after.metadata;
    let a = &before.coverage;
    let b = &after.coverage;
    let sides = [("before", before), ("after", after)];
    let mut notes = vec![
        (Role::Muted, "Net change = after − before, from the two saved reports.".into()),
        (Role::Muted, "Unreadable files add nothing to the recorded counts: their lines are unknown, not zero.".into()),
        (Role::Muted, "A folder seen in only one report may be newly visible, not newly created.".into()),
    ];
    let inaccurate: Vec<_> = sides
        .iter()
        .filter_map(|(side, m)| m.backend_inaccurate.then_some(*side))
        .collect();
    if !inaccurate.is_empty() {
        notes.push((Role::Warning, format!("Inaccurate input: {}; recorded counts may be partial, even without identified failure paths.", inaccurate.join(", "))));
    }

    let unread_a: BTreeSet<_> = a.unread.iter().collect();
    let unread_b: BTreeSet<_> = b.unread.iter().collect();
    if !unread_a.is_empty() || !unread_b.is_empty() {
        notes.push((
            Role::Warning,
            format!(
                "Unreadable files: {}; not counted on the affected side(s).{}",
                set_summary(&unread_a, &unread_b, "path"),
                member_suffix(
                    failure_members(before, a.unread.iter().map(String::as_str)),
                    failure_members(after, b.unread.iter().map(String::as_str)),
                )
            ),
        ));
    }

    let failures = |m: &Metadata| {
        m.coverage.parse_failures.as_ref().map(|failures| {
            failures
                .iter()
                .map(|f| (f.path.clone(), f.reason.to_string()))
                .collect::<BTreeSet<_>>()
        })
    };
    let failed_a = failures(before);
    let failed_b = failures(after);
    let has_failures = failed_a.iter().chain(&failed_b).any(|set| !set.is_empty());
    if has_failures {
        let summary = match (&failed_a, &failed_b) {
            (Some(a), Some(b)) => set_summary(a, b, "path/reason pair"),
            (None, Some(b)) => format!("not recorded → {}", b.len()),
            (Some(a), None) => format!("{} → not recorded", a.len()),
            (None, None) => unreachable!("at least one recorded failure"),
        };
        notes.push((Role::Warning, format!(
            "Parse failures: {summary}; compared by path/reason, not counted on the affected side(s).{}",
            member_suffix(
                parse_failure_members(before),
                parse_failure_members(after),
            )
        )));
    }

    let mut missing = Vec::new();
    for (side, metadata) in sides {
        if metadata.coverage.parse_failures.is_none() {
            missing.push(format!("not recorded ({side})"));
        }
        if let Some(members) = &metadata.members {
            let count = members
                .iter()
                .filter(|m| m.coverage.parse_failed_files.is_none())
                .count();
            if count > 0 {
                missing.push(format!(
                    "not recorded for {count} of {} members ({side})",
                    members.len()
                ));
            }
        }
    }
    if !missing.is_empty() {
        notes.push((
            Role::Information,
            format!(
                "Failure evidence: {}. Missing evidence does not certify historical counts.",
                missing.join("; ")
            ),
        ));
    }

    let unknown_a: BTreeSet<_> = a.unrecognized.iter().flatten().collect();
    let unknown_b: BTreeSet<_> = b.unrecognized.iter().flatten().collect();
    let has_unknown = !unknown_a.is_empty() || !unknown_b.is_empty();
    if has_unknown {
        let counting = if unknown_a == unknown_b {
            "not counted on either side"
        } else {
            "not counted on the affected side(s)"
        };
        notes.push((
            Role::Muted,
            format!(
                "Unrecognized files: {}; {counting}.",
                set_summary(&unknown_a, &unknown_b, "path")
            ),
        ));
    }
    if has_unknown || has_failures || !unread_a.is_empty() || !unread_b.is_empty() {
        notes.push((Role::Muted, "Folder totals containing these files cover counted files only; per-folder detail is in JSON findings[].folder.".into()));
    }

    for (name, a, b) in [
        (
            "Locmetry version",
            &before.locmetry_version,
            &after.locmetry_version,
        ),
        ("Backend", &before.backend, &after.backend),
        (
            "Backend configuration",
            &before.backend_configuration,
            &after.backend_configuration,
        ),
    ] {
        if a != b {
            notes.push((Role::Information, format!("{name}: {a} → {b}.")));
        }
    }
    if before.locmetry_version != after.locmetry_version {
        notes.push((Role::Muted, "Counting corrections can differ between versions; the reports cannot show whether they did.".into()));
    }
    notes
}
