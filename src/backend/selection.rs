//! Path selection: the scope walk, its targets, and why a path was left out.
use super::*;

pub(crate) struct Selection {
    pub paths: Vec<PathBuf>,
    pub target_selected: bool,
    pub(super) applied: BTreeSet<usize>,
    pub exclusion: Option<Exclusion>,
}

pub(crate) struct Exclusion {
    pub at: String,
    pub patterns: Vec<String>,
}

#[derive(Clone, Copy)]
pub(crate) enum Target<'a> {
    File(&'a str),
    Subfolder(&'a str),
}

impl<'a> Target<'a> {
    pub(super) fn path(self) -> &'a str {
        match self {
            Self::File(path) | Self::Subfolder(path) => path,
        }
    }
    pub(super) fn contains(self, path: &str) -> bool {
        match self {
            Self::File(target) => target == path,
            Self::Subfolder(target) => crate::paths::below(path, target).is_some(),
        }
    }
}

/// Check on-disk spelling before selection; aliases must not become false scope diagnoses.
/// Project path identity follows directory entries, including on alias-tolerant APFS
/// (docs/PRODUCT.md, Path Explanations and Subfolder Measurement).
pub(crate) fn inspect_path(root: &Path, path: &str, label: &str) -> Result<Option<String>> {
    let mut current = root.to_owned();
    let mut relative = PathBuf::new();
    for component in Path::new(path).components() {
        let exact = std::fs::read_dir(&current)
            .with_context(|| format!("Cannot inspect the spelling of {label} target '{path}'"))?
            .collect::<std::io::Result<Vec<_>>>()?
            .iter()
            .any(|entry| entry.file_name() == component.as_os_str());
        current.push(component);
        let metadata = std::fs::symlink_metadata(&current).with_context(|| {
            format!("{label} target '{path}' does not exist or cannot be inspected")
        })?;
        ensure!(
            exact,
            "{label} target must use the exact spelling of directory entries: {path}"
        );
        relative.push(component);
        if metadata.file_type().is_symlink() {
            return Ok(Some(relative_path(&relative)?));
        }
    }
    Ok(None)
}

/// Resolve one literal directory without changing the project's path base.
pub fn subfolder_path(root: &Path, given: &Path) -> Result<Option<String>> {
    ensure!(!given.as_os_str().is_empty(), "Subfolder must not be empty");
    ensure!(
        given
            .components()
            .all(|c| matches!(c, Component::Normal(_) | Component::CurDir)),
        "Subfolder '{}' must be root-relative without parent components",
        given.display()
    );
    if given.components().all(|c| c == Component::CurDir) {
        return Ok(None);
    }
    let path = relative_path(given)?;
    ensure!(
        inspect_path(root, &path, "Subfolder")?.is_none(),
        "Subfolder '{path}' must not pass through a symlink"
    );
    ensure!(
        root.join(&path).is_dir(),
        "Subfolder '{path}' must be a directory"
    );
    Ok(Some(path))
}

#[derive(Serialize, schemars::JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FilterReason {
    /// Disabling only ignore filtering selects the target; no winning ignore-file rule is claimed.
    IgnorePolicy,
    /// Enabling only hidden paths selects the target.
    HiddenPolicy,
    /// No single-setting probe established a cause, including overlapping filters.
    ReasonNotRecorded,
}

pub(crate) fn filter_reason(
    root: &Path,
    scope: &crate::Scope,
    target: Target<'_>,
) -> Result<FilterReason> {
    let selected_with = |overrides| -> Result<bool> {
        let scope = scope.with_overrides(&overrides)?;
        Ok(select_paths(root, &scope, Some(target))?.target_selected)
    };
    if scope.respect_ignore_files()
        && selected_with(crate::ScopeOverrides {
            respect_ignore_files: Some(false),
            ..Default::default()
        })?
    {
        Ok(FilterReason::IgnorePolicy)
    } else if !scope.hidden()
        && selected_with(crate::ScopeOverrides {
            hidden: Some(true),
            ..Default::default()
        })?
    {
        Ok(FilterReason::HiddenPolicy)
    } else {
        Ok(FilterReason::ReasonNotRecorded)
    }
}

pub(crate) fn select_paths(
    root: &Path,
    scope: &crate::Scope,
    target: Option<Target<'_>>,
) -> Result<Selection> {
    let root = root.to_owned();
    // Tokei v15 src/utils/fs.rs anchors overrides to CWD and uses gitignore
    // syntax; select scope here so root-relative globs keep docs/PRODUCT.md semantics.
    let scope = scope.clone();
    let mut walker = WalkBuilder::new(&root);
    walker
        .standard_filters(scope.respect_ignore_files())
        .hidden(!scope.hidden());
    if scope.respect_ignore_files() {
        walker.add_custom_ignore_filename(TOOL_IGNORE_FILE);
    }
    let walk_root = root.clone();
    let applied = Arc::new(Mutex::new(BTreeSet::new()));
    let recorder = Arc::clone(&applied);
    let exclusion = Arc::new(Mutex::new(None));
    let trace = Arc::clone(&exclusion);
    let target_path = target.map(|t| t.path().to_owned());
    let subfolder = match target {
        Some(Target::Subfolder(path)) => Some(path.to_owned()),
        _ => None,
    };
    walker.filter_entry(move |entry| {
        let Ok(path) = entry
            .path()
            .strip_prefix(&walk_root)
            .map_err(anyhow::Error::from)
            .and_then(relative_path)
        else {
            return true; // The root itself; non-UTF-8 names fail later with context.
        };
        let directory = entry.file_type().is_some_and(|kind| kind.is_dir());
        let excluded_by = scope.excluded_by(&path, directory);
        let selected = excluded_by.is_empty();
        if !selected
            && target_path.as_ref().is_some_and(|target| {
                target == &path || (directory && crate::paths::below(target, &path).is_some())
            })
        {
            let patterns = scope.exclusions();
            *trace.lock().expect("single-threaded traversal") = Some(Exclusion {
                at: path.clone(),
                patterns: excluded_by
                    .iter()
                    .map(|&index| patterns[index].clone())
                    .collect(),
            });
        }
        if subfolder.as_ref().is_none_or(|folder| {
            path == *folder
                || Target::Subfolder(folder).contains(&path)
                || (directory && Target::Subfolder(&path).contains(folder))
        }) {
            recorder
                .lock()
                .expect("the walk records exclusions on one thread")
                .extend(excluded_by);
        }
        selected
    });
    let mut paths = Vec::new();
    let mut target_selected = false;
    for entry in walker.build() {
        let entry = entry.context("Cannot traverse scan root")?;
        if target.is_some_and(|target| entry.path() == root.join(target.path())) {
            target_selected = true;
        }
        if entry.file_type().is_some_and(|kind| kind.is_file()) {
            paths.push(entry.into_path());
        }
    }
    let applied = std::mem::take(&mut *applied.lock().expect("the walk has finished"));
    let exclusion = exclusion.lock().expect("the walk has finished").take();
    Ok(Selection {
        paths,
        target_selected,
        applied,
        exclusion,
    })
}
