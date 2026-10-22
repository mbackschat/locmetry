//! Workspace files: the roots or project folders one workspace measurement merges.
use super::*;

/// Locmetry workspace: TOML naming what `locmetry workspace --workspace-file` measures as one virtual monorepo.
///
/// Name either independent repositories with `roots`, each measured with its own `.locmetryrc`, or folders of one repository with `project` and `folders`, all measured with the project's `.locmetryrc` as `--subfolder` would measure them. Each member becomes a top-level folder of the report, named after its root's or folder's last segment. The file is read only when `--workspace-file` names it, never discovered, and a member that contains it does not measure it.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(
    rename = "Workspace",
    extend("oneOf" = [{"required": ["roots"]}, {"required": ["project", "folders"]}])
)]
pub(crate) struct WorkspaceFile {
    /// Contract version; must be `1`.
    #[schemars(extend("const" = 1))]
    pub(super) version: u32,
    /// Independent member roots, each relative to the folder of this file or absolute; folder names must be unique, and no root may lie inside another. Excludes `project` and `folders`.
    #[schemars(length(min = 1), extend("items" = {"type": "string"}))]
    pub(super) roots: Option<Vec<PathBuf>>,
    /// One repository whose `.locmetryrc` decides every member, relative to the folder of this file or absolute. Requires `folders`.
    #[schemars(extend("type" = "string"))]
    pub(super) project: Option<PathBuf>,
    /// Literal project-relative member folders, measured as `--subfolder` would measure them; their last segments must be unique, and no folder may lie inside another. Files outside them are not measured. Requires `project`.
    #[schemars(length(min = 1))]
    pub(super) folders: Option<Vec<String>>,
}

/// What a workspace file names: independent roots, or folders of one project.
#[derive(Debug, PartialEq, Eq)]
pub enum WorkspaceDefinition {
    Roots(Vec<PathBuf>),
    Folders {
        project: PathBuf,
        folders: Vec<String>,
    },
}

/// The members a workspace file names, with paths resolved against its folder.
pub fn workspace_definition(path: &Path) -> Result<WorkspaceDefinition> {
    let input = std::fs::read_to_string(path)
        .with_context(|| format!("Cannot read workspace file '{}'", path.display()))?;
    let file: WorkspaceFile = parse_toml(&input)
        .with_context(|| format!("Invalid workspace file '{}'", path.display()))?;
    ensure!(
        file.version == 1,
        "Unsupported version {} in workspace file '{}'",
        file.version,
        path.display()
    );
    let folder = path.parent().unwrap_or(Path::new("."));
    match (file.roots, file.project, file.folders) {
        (Some(roots), None, None) => {
            ensure!(
                !roots.is_empty(),
                "Workspace file '{}' lists no roots",
                path.display()
            );
            Ok(WorkspaceDefinition::Roots(
                roots.iter().map(|root| folder.join(root)).collect(),
            ))
        }
        (None, Some(project), Some(folders)) => {
            ensure!(
                !folders.is_empty(),
                "Workspace file '{}' lists no folders",
                path.display()
            );
            Ok(WorkspaceDefinition::Folders {
                project: folder.join(project),
                folders,
            })
        }
        (Some(_), _, _) => bail!(
            "Workspace file '{}' combines roots with project or folders. Name either independent roots or one project's folders.",
            path.display()
        ),
        (None, Some(_), None) => bail!(
            "Workspace file '{}' names a project but no folders",
            path.display()
        ),
        (None, None, _) => bail!(
            "Workspace file '{}' needs roots, or a project with folders",
            path.display()
        ),
    }
}
