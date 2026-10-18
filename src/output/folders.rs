use super::RowOrder;
use crate::{
    paths::{below, parent, ROOT},
    report::{collapse_folder, Summary},
    Counts, Purpose, ReportSummary,
};
use std::collections::BTreeMap;

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum RowKind {
    Folder,
    Direct,
    Other(usize),
}

pub(super) struct Row {
    path: String,
    kind: RowKind,
    pub counts: Counts,
    pub purposes: BTreeMap<Purpose, u64>,
}

impl Row {
    fn new(path: &str, kind: RowKind, summary: &Summary) -> Self {
        Self {
            path: path.into(),
            kind,
            counts: summary.source,
            purposes: summary
                .source_purpose
                .iter()
                .map(|(p, c)| (*p, c.code))
                .collect(),
        }
    }

    fn path_order(&self, other: &Self) -> std::cmp::Ordering {
        self.path.split('/').cmp(other.path.split('/'))
    }

    fn code_order(&self, other: &Self) -> std::cmp::Ordering {
        other
            .code()
            .cmp(&self.code())
            .then_with(|| self.path_order(other))
            .then_with(|| self.kind.cmp(&other.kind))
    }

    fn code(&self) -> u64 {
        self.purposes.values().sum()
    }

    pub fn label(&self) -> String {
        match self.kind {
            RowKind::Folder => self.path.clone(),
            RowKind::Direct if self.path == "." => "(root files)".into(),
            RowKind::Direct => format!("{} (direct files)", self.path),
            RowKind::Other(n) => format!("{} (other {n} child folders)", self.path),
        }
    }
}

struct Tree<'a> {
    summary: &'a ReportSummary,
    children: BTreeMap<&'a str, Vec<&'a str>>,
}

impl Tree<'_> {
    fn children(&self, path: &str) -> &[&str] {
        self.children.get(path).map_or(&[], Vec::as_slice)
    }

    fn folder(&self, path: &str) -> Row {
        self.collapse(Row::new(
            path,
            RowKind::Folder,
            &self.summary.folders[path].recursive,
        ))
    }

    fn collapse(&self, mut row: Row) -> Row {
        // With no direct source and one source-bearing child, the purposes stay
        // unchanged along the chain (docs/PRODUCT.md#adaptive-folder-detail).
        row.path = collapse_folder(row.path, |path| match self.children(path) {
            [child] if self.summary.folders[path].direct.source.files == 0 => Some((*child).into()),
            _ => None,
        });
        row
    }

    fn direct(&self, path: &str) -> Option<Row> {
        let summary = &self.summary.folders[path].direct;
        (summary.source.files > 0).then(|| Row::new(path, RowKind::Direct, summary))
    }

    fn split(&self, row: &Row, slots: usize) -> Option<Vec<Row>> {
        if row.kind != RowKind::Folder || row.code() == 0 {
            return None;
        }
        let children = self.children(&row.path);
        if children.is_empty() {
            return None;
        }
        let mut rows: Vec<_> = self.direct(&row.path).into_iter().collect();
        let mut children: Vec<_> = children
            .iter()
            .map(|path| {
                Row::new(
                    path,
                    RowKind::Folder,
                    &self.summary.folders[*path].recursive,
                )
            })
            .collect();
        children.sort_by(|a, b| a.code_order(b));
        if rows.len() + children.len() > slots {
            let named = slots.checked_sub(rows.len() + 1)?;
            if named < 2 {
                return None;
            }
            let omitted = children.split_off(named);
            let mut remainder = Row {
                path: row.path.clone(),
                kind: RowKind::Other(omitted.len()),
                counts: Counts::default(),
                purposes: BTreeMap::new(),
            };
            for child in omitted {
                remainder.counts.add(&child.counts);
                for (purpose, code) in child.purposes {
                    *remainder.purposes.entry(purpose).or_default() += code;
                }
            }
            rows.push(remainder);
        }
        // The remainder describes immediate children; collapse only the named
        // rows after allocation so its scope stays stable (docs/PRODUCT.md#adaptive-folder-detail).
        rows.extend(children.into_iter().map(|child| self.collapse(child)));
        Some(rows)
    }
}

pub(super) fn standard(
    summary: &ReportSummary,
    subfolder: Option<&str>,
    order: RowOrder,
) -> Vec<Row> {
    let root = subfolder.unwrap_or(ROOT);
    let mut rows = Vec::new();
    for (path, folder) in &summary.folders {
        if path == root {
            if folder.direct.source.files > 0 {
                rows.push(Row::new(path, RowKind::Direct, &folder.direct));
            }
        } else if below(path, root).is_some_and(|relative| !relative.contains('/'))
            && folder.recursive.source.files > 0
        {
            rows.push(Row::new(path, RowKind::Folder, &folder.recursive));
        }
    }
    rows.sort_by(|a, b| match order {
        RowOrder::Code => b.code().cmp(&a.code()).then(a.path.cmp(&b.path)),
        RowOrder::Name => a.path.cmp(&b.path),
    });
    rows
}

pub(super) struct View {
    pub rows: Vec<Row>,
    pub budget_hint: Option<String>,
}

pub(super) fn adaptive(
    summary: &ReportSummary,
    subfolder: Option<&str>,
    budget: usize,
    order: RowOrder,
) -> View {
    let root = subfolder.unwrap_or(ROOT);
    let mut tree = Tree {
        summary,
        children: BTreeMap::new(),
    };
    for (path, folder) in &summary.folders {
        if path == ROOT || folder.recursive.source.files == 0 || below(path, root).is_none() {
            continue;
        }
        tree.children.entry(parent(path)).or_default().push(path);
    }
    let mut rows: Vec<_> = tree
        .children(root)
        .iter()
        .map(|path| tree.folder(path))
        .collect();
    if summary.folders.contains_key(root) {
        rows.extend(tree.direct(root));
    }
    let initial = rows.len();
    let limit = budget.max(initial);
    loop {
        let mut candidates: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r.kind == RowKind::Folder)
            .collect();
        candidates.sort_by(|(_, a), (_, b)| a.code_order(b));
        let next = candidates.into_iter().find_map(|(index, row)| {
            tree.split(row, limit - rows.len() + 1)
                .map(|children| (index, children))
        });
        match next {
            Some((index, children)) => {
                rows.remove(index);
                rows.extend(children);
            }
            None => break,
        }
    }
    let budget_hint = rows
        .iter()
        .min_by(|a, b| a.code_order(b))
        .filter(|row| row.code() > 0)
        .and_then(|row| match row.kind {
            RowKind::Folder if !tree.children(&row.path).is_empty() => Some(format!(
                "{} has more folder detail; increase --folder-rows.",
                row.path
            )),
            RowKind::Other(_) => Some(format!(
                "More child folders of {} can be shown; increase --folder-rows.",
                row.path
            )),
            RowKind::Folder | RowKind::Direct => None,
        });
    rows.sort_by(|a, b| {
        let name = a.path_order(b).then_with(|| a.kind.cmp(&b.kind));
        match order {
            RowOrder::Code => a.code_order(b),
            RowOrder::Name => name,
        }
    });
    View { rows, budget_hint }
}
