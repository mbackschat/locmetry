//! Root-relative paths as reports record them: `/`-separated, with [`ROOT`]
//! for the measured root, and in workspace reports prefixed by their member.

/// The measured root's folder name.
pub(crate) const ROOT: &str = ".";

/// The folder holding `path`; [`ROOT`] for a top-level entry.
pub(crate) fn parent(path: &str) -> &str {
    path.rsplit_once('/').map_or(ROOT, |(parent, _)| parent)
}

/// Folders holding `path`, innermost first and ending with [`ROOT`]; the root
/// itself has none.
pub(crate) fn ancestors(path: &str) -> impl Iterator<Item = &str> {
    let mut next = (path != ROOT).then(|| parent(path));
    std::iter::from_fn(move || {
        let folder = next?;
        next = (folder != ROOT).then(|| parent(folder));
        Some(folder)
    })
}

/// `path` relative to `folder`, if it lies below it; every path lies below
/// [`ROOT`].
pub(crate) fn below<'p>(path: &'p str, folder: &str) -> Option<&'p str> {
    match folder {
        ROOT => Some(path),
        _ => path.strip_prefix(folder)?.strip_prefix('/'),
    }
}

/// Whether `path` is `folder` or lies below it.
pub(crate) fn within(path: &str, folder: &str) -> bool {
    path == folder || below(path, folder).is_some()
}

/// The workspace member a member-prefixed path belongs to.
pub(crate) fn member_of(path: &str) -> &str {
    path.split('/').next().unwrap_or(path)
}

/// `path` of `member` as a workspace report names it; the member's root is
/// the member itself.
pub fn qualify(member: &str, path: &str) -> String {
    match path {
        ROOT => member.to_string(),
        _ => format!("{member}/{path}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ancestors_walk_up_to_the_root() {
        assert_eq!(ancestors("a/b/c.rs").collect::<Vec<_>>(), ["a/b", "a", "."]);
        assert_eq!(ancestors("c.rs").collect::<Vec<_>>(), ["."]);
        assert_eq!(ancestors(ROOT).count(), 0);
    }

    /// A shared name prefix is not containment: `web-admin` is not below `web`.
    #[test]
    fn below_requires_a_whole_folder_prefix() {
        assert_eq!(below("web/src/a.ts", "web"), Some("src/a.ts"));
        assert_eq!(below("web-admin/a.ts", "web"), None);
        assert_eq!(below("web", "web"), None);
        assert_eq!(below("web/a.ts", ROOT), Some("web/a.ts"));
        assert!(within("web", "web") && within("web/a", "web") && !within("webx", "web"));
    }

    #[test]
    fn qualify_names_the_member_root_by_the_member() {
        assert_eq!(qualify("api", ROOT), "api");
        assert_eq!(qualify("api", "src/lib.rs"), "api/src/lib.rs");
        assert_eq!(member_of("api/src/lib.rs"), "api");
    }
}
