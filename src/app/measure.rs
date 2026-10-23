//! Preparing and scanning roots, and resolving workspace members.
use super::*;

/// One root measured as a regular run measures it.
pub(crate) struct Measurement {
    pub(crate) root: PathBuf,
    pub(crate) name: String,
    pub(crate) config_path: Option<PathBuf>,
    pub(crate) policy: Policy,
    pub(crate) coverage: Coverage,
    pub(crate) inaccurate: bool,
    pub(crate) rows: Vec<FileRecord>,
    pub(crate) modules: Vec<ModuleShare>,
    pub(crate) submodules: Vec<SubmoduleShare>,
    pub(crate) unused_entries: Vec<String>,
    pub(crate) unused_exclusions: Vec<String>,
}

/// A root with its loaded policy, ready to scan; preparing first lets output
/// paths be checked before anything is scanned.
pub(crate) struct Prepared {
    pub(crate) root: PathBuf,
    pub(crate) name: String,
    pub(crate) config_path: Option<PathBuf>,
    pub(crate) policy: Policy,
}

/// Loads the policy for `root`; `workspace_path` names a workspace file,
/// which, like the configurations, is excluded when it lies inside the root.
pub(crate) fn prepare(
    root: &Path,
    config: Option<PathBuf>,
    user_path: Option<&Path>,
    workspace_path: Option<&Path>,
    overrides: &ScopeOverrides,
) -> Result<Prepared> {
    let config_path = config.or_else(|| {
        let discovered = root.join(CONFIG_FILE);
        discovered.is_file().then_some(discovered)
    });
    let mut policy = Policy::load_layers(config_path.as_deref(), user_path)?;
    policy.override_scope(overrides)?;
    let root = locmetry::scan_root(root)?;
    for path in [config_path.as_deref(), user_path, workspace_path]
        .into_iter()
        .flatten()
    {
        policy.exclude_configuration(&root, path)?;
    }
    let root_text = root.to_str().context("Scan root must be valid UTF-8")?;
    let name = root.file_name().map_or_else(
        || root_text.to_owned(),
        |name| name.to_string_lossy().into(),
    );
    Ok(Prepared {
        root,
        name,
        config_path,
        policy,
    })
}

pub(crate) fn measure(prepared: Prepared, subfolder: Option<&str>) -> Result<Measurement> {
    let Prepared {
        root,
        name,
        config_path,
        mut policy,
    } = prepared;
    let scan = match subfolder {
        Some(folder) => locmetry::scan_subfolder(&root, &policy, folder)?,
        None => scan(&root, &policy)?,
    };
    policy.adopt_modules(&scan.modules)?;
    let rows = scan.records(&policy)?;
    let mut modules = module_shares(policy.modules(), &rows);
    if subfolder.is_some() {
        modules.retain(|module| module.files > 0);
    }
    // A correction that decides no selected file contradicts the counter for
    // nothing, so it is reported with the unused classification entries.
    let mut unused = unused_entries(policy.entry_ids(), &rows);
    let applied: std::collections::BTreeSet<&str> =
        scan.corrections.values().map(String::as_str).collect();
    unused.extend(
        policy
            .correction_ids()
            .filter(|id| !applied.contains(id))
            .map(String::from),
    );
    Ok(Measurement {
        modules,
        submodules: submodule_shares(&git_submodules(&root), &rows),
        unused_entries: unused,
        unused_exclusions: scan.unused_exclusions,
        coverage: scan.coverage,
        inaccurate: scan.inaccurate,
        rows,
        root,
        name,
        config_path,
        policy,
    })
}

/// Items every list contains, in the order of the first list.
pub(crate) fn common<'a>(mut lists: impl Iterator<Item = &'a Vec<String>>) -> Vec<String> {
    let first = lists.next().cloned().unwrap_or_default();
    lists.fold(first, |kept, list| {
        kept.into_iter()
            .filter(|item| list.contains(item))
            .collect()
    })
}

/// A workspace member: an independent root measured with its own policy, or
/// a `folder` of a project root measured with the project's policy, as
/// `--subfolder` measures it.
pub(crate) struct WorkspaceMember {
    pub(crate) name: String,
    /// The root or folder as the user typed it, for errors.
    pub(crate) given: String,
    pub(crate) root: PathBuf,
    pub(crate) folder: Option<String>,
}

/// Canonical members sorted by name, after the checks that make a merge
/// well defined: unique names, no location twice, and no member inside another.
pub(crate) fn workspace_members(
    definition: locmetry::WorkspaceDefinition,
) -> Result<Vec<WorkspaceMember>> {
    let mut members = match definition {
        locmetry::WorkspaceDefinition::Roots(roots) => roots
            .iter()
            .map(|given| {
                let given = given.display().to_string();
                let root = dunce::canonicalize(&given)
                    .with_context(|| format!("Workspace member root '{given}' does not exist"))?;
                anyhow::ensure!(
                    root.is_dir(),
                    "Workspace member root '{given}' must be a directory"
                );
                let name = root
                    .file_name()
                    .and_then(|name| name.to_str())
                    .with_context(|| format!("Workspace member '{given}' has no folder name"))?
                    .to_owned();
                Ok(WorkspaceMember {
                    name,
                    given,
                    root,
                    folder: None,
                })
            })
            .collect::<Result<Vec<_>>>()?,
        locmetry::WorkspaceDefinition::Folders { project, folders } => {
            let root = dunce::canonicalize(&project).with_context(|| {
                format!("Workspace project '{}' does not exist", project.display())
            })?;
            folders
                .iter()
                .map(|given| {
                    let folder = locmetry::subfolder_path(&root, Path::new(given))
                        .with_context(|| format!("Workspace folder '{given}'"))?
                        .with_context(|| {
                            format!(
                                "Workspace folder '{given}' names the project root, not a folder"
                            )
                        })?;
                    let name = folder
                        .rsplit_once('/')
                        .map_or(folder.as_str(), |(_, last)| last)
                        .to_owned();
                    Ok(WorkspaceMember {
                        name,
                        given: given.clone(),
                        root: root.clone(),
                        folder: Some(folder),
                    })
                })
                .collect::<Result<Vec<_>>>()?
        }
    };
    members.sort_by(|a, b| (&a.name, &a.root, &a.folder).cmp(&(&b.name, &b.root, &b.folder)));
    let location = |member: &WorkspaceMember| match &member.folder {
        Some(folder) => locmetry::filesystem_path(&member.root, folder),
        None => member.root.clone(),
    };
    for pair in members.windows(2) {
        let [member, other] = pair else {
            unreachable!("windows of two")
        };
        if location(member) == location(other) {
            anyhow::ensure!(
                member.given != other.given,
                "Workspace root '{}' is given twice",
                member.given
            );
            anyhow::bail!(
                "Workspace roots '{}' and '{}' name the same folder",
                member.given,
                other.given
            );
        }
        anyhow::ensure!(
            member.name != other.name,
            "Workspace members '{}' and '{}' share the name '{}'",
            member.given,
            other.given,
            member.name
        );
    }
    for member in &members {
        if let Some(outer) = members.iter().find(|outer| {
            location(outer) != location(member) && location(member).starts_with(location(outer))
        }) {
            anyhow::bail!(
                "Workspace member '{}' lies inside member '{}'",
                member.given,
                outer.given
            );
        }
    }
    Ok(members)
}
