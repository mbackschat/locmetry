//! Folder and package lineage: which folders and members a change belongs to.
use super::*;

#[derive(Clone, Copy, Default)]
pub(super) struct Pair {
    pub(super) all: Counts,
    pub(super) source: Counts,
}

#[derive(Default)]
pub(super) struct Folder {
    pub(super) evidence: BTreeSet<&'static str>,
    pub(super) direct: Pair,
    pub(super) recursive: Pair,
}

/// `path` and the folders holding it, up to the root, or in a workspace up
/// to its member, whose totals the workspace root does not repeat.
pub(super) fn lineage(path: &str, workspace: bool) -> impl Iterator<Item = &str> {
    let mut done = false;
    std::iter::once(path)
        .chain(ancestors(path))
        .take_while(move |folder| {
            let take = !done;
            done = *folder == ROOT || (workspace && !folder.contains('/'));
            take
        })
}

pub(super) fn folders(report: &MeasurementReport) -> Result<BTreeMap<String, Folder>> {
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

pub(super) fn folder_counts(a: Pair, b: Pair) -> Result<FolderCounts> {
    Ok(FolderCounts {
        all_counted: CountChange::new(a.all, b.all)?,
        source: CountChange::new(a.source, b.source)?,
    })
}

pub(super) fn packages(report: &MeasurementReport) -> BTreeSet<(String, String)> {
    report
        .metadata
        .modules
        .iter()
        .flatten()
        .map(|m| (m.path.clone(), m.detector.clone()))
        .collect()
}

pub(super) fn member_for(path: &str, workspace: bool) -> Option<String> {
    workspace.then(|| member_of(path).into())
}

pub(super) fn add_pair(pair: &mut Pair, s: &crate::Summary) -> Result<()> {
    pair.all = pair.all.checked_add(&s.all_counted)?;
    pair.source = pair.source.checked_add(&s.source)?;
    Ok(())
}
