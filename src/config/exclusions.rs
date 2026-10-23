//! Exclusions of configuration files, which are policy rather than project
//! content, and the unused-exclusion finding, which must not report them.
use super::{literal_glob, Policy, ScopeOverrides, CONFIG_FILE};
use crate::backend::relative_path;
use anyhow::{Context, Result};
use std::{collections::BTreeSet, path::Path};

impl Policy {
    /// Excludes the loaded configuration file when it lies inside the
    /// canonical scan `root` and no exclusion covers it yet.
    ///
    /// The configuration is measurement policy, not project content. Its path
    /// is escaped so glob metacharacters in its name match only itself.
    pub fn exclude_configuration(&mut self, root: &Path, config: &Path) -> Result<()> {
        let config = dunce::canonicalize(config)
            .with_context(|| format!("Cannot resolve configuration '{}'", config.display()))?;
        let Ok(relative) = config.strip_prefix(root) else {
            return Ok(());
        };
        let relative = relative_path(relative)?;
        if self.is_excluded(&relative) {
            return Ok(());
        }
        self.exclude_policy(literal_glob(&relative))
    }

    /// Excludes every project configuration file, loaded or not: an unloaded
    /// one, such as a nested project's or one beside an explicit `--config`,
    /// is policy too, and counting it would make coverage depend on which
    /// file was loaded (docs/PRODUCT.md, Project Configuration).
    pub fn exclude_configuration_files(&mut self) -> Result<()> {
        self.exclude_policy(format!("**/{CONFIG_FILE}"))
    }

    fn exclude_policy(&mut self, pattern: String) -> Result<()> {
        self.configuration_exclusions.insert(pattern.clone());
        self.override_scope(&ScopeOverrides {
            exclude: vec![pattern],
            ..Default::default()
        })
    }

    /// Configured exclusions that left out no file or folder, given the
    /// indices into the effective exclusions that a scan applied.
    ///
    /// Default exclusions and those of configuration files are omitted: they
    /// routinely match nothing, as with a hidden `.locmetryrc` and
    /// `hidden = false`. A pattern repeated across layers counts once.
    pub fn unused_exclusions(&self, applied: &BTreeSet<usize>) -> Vec<String> {
        let patterns = self.scope.exclusions();
        let used: BTreeSet<&str> = applied
            .iter()
            .map(|&index| patterns[index].as_str())
            .collect();
        let mut unused: Vec<String> = Vec::new();
        for pattern in &patterns[self.scope.default_count()..] {
            if !used.contains(pattern.as_str())
                && !self.configuration_exclusions.contains(pattern)
                && !unused.contains(pattern)
            {
                unused.push(pattern.clone());
            }
        }
        unused
    }
}
