//! Personal defaults, which widen exclusions and language mappings for every project.
use super::*;

/// Locmetry personal defaults: TOML in `$XDG_CONFIG_HOME/locmetry/config.toml`, else `~/.config/locmetry/config.toml`, or `%APPDATA%\locmetry\config.toml` on Windows, applied to every project.
///
/// They may only narrow scope and teach languages, never classify by purpose or category. Personal mappings can still change recognition and content kind. `--no-user-config` ignores them, and reports name the file as `metadata.user_config_path`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "UserDefaults")]
pub(super) struct UserDefaults {
    /// Contract version; must be `1`.
    #[schemars(extend("const" = 1))]
    pub(super) version: u32,
    #[serde(default)]
    pub(super) scope: UserScope,
    #[serde(default)]
    pub(super) languages: UserLanguages,
}

/// Exclusions for every project.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct UserScope {
    /// Root-relative globs of files to leave out, added after the default exclusions and before the project's.
    #[serde(default)]
    #[schemars(extend("items" = {"type": "string"}))]
    pub(super) exclude: Vec<String>,
}

/// Language mappings for every project.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub(super) struct UserLanguages {
    /// Applied after the project's mappings and before the built-in ones; IDs must differ from the project's.
    #[serde(default)]
    pub(super) mappings: Vec<MappingConfig>,
}

impl UserDefaults {
    pub(super) fn load(path: &Path) -> Result<Self> {
        let input = std::fs::read_to_string(path)
            .with_context(|| format!("Cannot read user defaults '{}'", path.display()))?;
        let defaults: Self = parse_toml(&input).with_context(|| {
            format!(
                "Invalid user defaults '{}' (they may set only 'version', 'scope.exclude' and '[[languages.mappings]]')",
                path.display()
            )
        })?;
        ensure!(
            defaults.version == 1,
            "Unsupported version {} in user defaults '{}'",
            defaults.version,
            path.display()
        );
        Ok(defaults)
    }

    /// Adds user exclusions before the project's and user mappings after the
    /// project's, so the project's own mappings take precedence.
    pub(super) fn merge_into(self, config: &mut ConfigFile, path: &Path) -> Result<()> {
        for mapping in &self.languages.mappings {
            ensure!(
                config
                    .languages
                    .mappings
                    .iter()
                    .all(|project| project.id != mapping.id),
                "Language mapping '{}' in user defaults '{}' is also defined by the project",
                mapping.id,
                path.display()
            );
        }
        config.scope.exclude.splice(0..0, self.scope.exclude);
        config.languages.mappings.extend(self.languages.mappings);
        Ok(())
    }
}

/// The user defaults file: `$XDG_CONFIG_HOME/locmetry/config.toml`, else
/// `~/.config/locmetry/config.toml`, or `%APPDATA%\locmetry\config.toml` on
/// Windows (docs/PRODUCT.md).
pub fn user_defaults_file() -> Option<PathBuf> {
    defaults_file(|name| std::env::var_os(name))
}

/// A relative `XDG_CONFIG_HOME` is ignored, as the XDG Base Directory
/// specification requires. Windows sets no `HOME` by default, and Tokei, cloc
/// and uv keep user configuration in `%APPDATA%` there
/// (docs/proposals/archive/windows-support.md).
fn defaults_file(var: impl Fn(&str) -> Option<std::ffi::OsString>) -> Option<PathBuf> {
    let absolute = |name: &str| {
        var(name)
            .map(PathBuf::from)
            .filter(|path| path.is_absolute())
    };
    let base = absolute("XDG_CONFIG_HOME").or_else(|| {
        if cfg!(windows) {
            absolute("APPDATA")
        } else {
            var("HOME").map(|home| PathBuf::from(home).join(".config"))
        }
    })?;
    Some(base.join("locmetry").join("config.toml"))
}

#[cfg(test)]
mod tests;
