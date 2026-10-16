use super::RowOrder;
use crate::{
    report::{below, collapse_folder, Summary},
    Purpose, ReportSummary,
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
    pub purposes: BTreeMap<Purpose, u64>,
}

impl Row {
    fn new(path: &str, kind: RowKind, summary: &Summary) -> Self {
        Self {
            path: path.into(),
            kind,
            purposes: summary
                .source_purpose
                .iter()
                .map(|(p, c)| (*p, c.code))
                .collect(),
        }
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
        let path = collapse_folder(path.into(), |path| match self.children(path) {
            [child] if self.summary.folders[path].direct.source.files == 0 => Some((*child).into()),
            _ => None,
        });
        Row::new(
            &path,
            RowKind::Folder,
            &self.summary.folders[&path].recursive,
        )
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
        let mut direct: Vec<_> = self.direct(&row.path).into_iter().collect();
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
        children.sort_by(|a, b| b.code().cmp(&a.code()).then_with(|| a.path.cmp(&b.path)));
        if direct.len() + children.len() > slots {
            let named = slots.checked_sub(direct.len() + 1)?;
            if named < 2 {
                return None;
            }
            let omitted = children.split_off(named);
            let mut remainder = Row {
                path: row.path.clone(),
                kind: RowKind::Other(omitted.len()),
                purposes: BTreeMap::new(),
            };
            for child in omitted {
                for (purpose, code) in child.purposes {
                    *remainder.purposes.entry(purpose).or_default() += code;
                }
            }
            direct.push(remainder);
        }
        // The remainder describes immediate children; collapse only the named
        // rows after allocation so its scope stays stable (docs/PRODUCT.md#adaptive-folder-detail).
        direct.extend(children.into_iter().map(|child| self.folder(&child.path)));
        Some(direct)
    }
}

pub(super) struct View {
    pub rows: Vec<Row>,
    pub initial_budget_exhausted: bool,
}

pub(super) fn adaptive(
    summary: &ReportSummary,
    subfolder: Option<&str>,
    budget: usize,
    order: RowOrder,
) -> View {
    let root = subfolder.unwrap_or(".");
    let mut tree = Tree {
        summary,
        children: BTreeMap::new(),
    };
    for (path, folder) in &summary.folders {
        if path == "." || folder.recursive.source.files == 0 {
            continue;
        }
        let base = if root == "." { "" } else { root };
        if below(path, base).is_none() {
            continue;
        }
        let parent = path.rsplit_once('/').map_or(".", |(parent, _)| parent);
        tree.children.entry(parent).or_default().push(path);
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
    let initial_budget_exhausted = initial >= budget
        && rows.iter().any(|row| {
            row.kind == RowKind::Folder && row.code() > 0 && !tree.children(&row.path).is_empty()
        });
    loop {
        let mut candidates: Vec<_> = rows
            .iter()
            .enumerate()
            .filter(|(_, r)| r.kind == RowKind::Folder)
            .collect();
        candidates
            .sort_by(|(_, a), (_, b)| b.code().cmp(&a.code()).then_with(|| a.path.cmp(&b.path)));
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
    rows.sort_by(|a, b| {
        let name = a.path.cmp(&b.path).then_with(|| a.kind.cmp(&b.kind));
        match order {
            RowOrder::Code => b.code().cmp(&a.code()).then(name),
            RowOrder::Name => name,
        }
    });
    View {
        rows,
        initial_budget_exhausted,
    }
}
