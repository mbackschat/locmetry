//! Compiled selection: globs and the exclusions that decide which files are measured.
use super::*;

/// Escapes glob metacharacters so `path` matches only itself.
pub(crate) fn literal_glob(path: &str) -> String {
    path.chars().fold(String::new(), |mut literal, character| {
        if "\\*?[]{}!".contains(character) {
            literal.push('\\');
        }
        literal.push(character);
        literal
    })
}

/// Compiles a root-relative glob with the separator semantics of docs/PRODUCT.md.
///
/// globset lets `*` cross `/` and makes `\` platform-dependent by default
/// (globset 0.4 `GlobBuilder` docs), so both options are set explicitly.
pub(super) fn glob(pattern: &str) -> Result<Glob> {
    ensure!(
        !pattern.starts_with('!'),
        "Pattern '{pattern}' uses negation, which is not supported"
    );
    ensure!(
        !pattern.starts_with('/')
            && pattern
                .split('/')
                .all(|part| !matches!(part, "" | "." | "..")),
        "Pattern '{pattern}' must be a relative path glob without empty, '.' or '..' segments"
    );
    GlobBuilder::new(pattern)
        .literal_separator(true)
        .backslash_escape(true)
        .build()
        .with_context(|| format!("Invalid glob '{pattern}'"))
}

pub(super) fn glob_set<'a>(patterns: impl IntoIterator<Item = &'a str>) -> Result<GlobSet> {
    let mut set = GlobSetBuilder::new();
    for pattern in patterns {
        set.add(glob(pattern)?);
    }
    Ok(set.build()?)
}

/// Compiled selection policy.
#[derive(Clone, Debug)]
pub struct Scope {
    pub(super) settings: ScopeSettings,
    pub(super) excludes: GlobSet,
    /// Directories whose every descendant is excluded, from `<dir>/**` patterns.
    pub(super) pruned_directories: GlobSet,
    /// Index into [`Scope::exclusions`] of each `pruned_directories` pattern.
    pub(super) pruning_patterns: Vec<usize>,
}

impl Scope {
    pub(super) fn new(settings: ScopeSettings) -> Result<Self> {
        let patterns = Self::patterns(&settings);
        let excludes =
            glob_set(patterns.iter().map(String::as_str)).context("Invalid scope exclusion")?;
        let (pruning_patterns, directories): (Vec<usize>, Vec<&str>) = patterns
            .iter()
            .enumerate()
            .filter_map(|(index, pattern)| Some((index, pattern.strip_suffix("/**")?)))
            .unzip();
        Ok(Self {
            settings,
            excludes,
            pruned_directories: glob_set(directories)?,
            pruning_patterns,
        })
    }

    pub(crate) fn with_overrides(&self, overrides: &ScopeOverrides) -> Result<Self> {
        let mut settings = self.settings.clone();
        if let Some(hidden) = overrides.hidden {
            settings.hidden = hidden;
        }
        if let Some(respect) = overrides.respect_ignore_files {
            settings.respect_ignore_files = respect;
        }
        settings.exclude.extend(overrides.exclude.iter().cloned());
        Self::new(settings)
    }

    pub(super) fn patterns(settings: &ScopeSettings) -> Vec<String> {
        let defaults = DEFAULT_EXCLUDES
            .iter()
            .filter(|_| settings.default_excludes)
            .map(|pattern| pattern.to_string());
        defaults.chain(settings.exclude.iter().cloned()).collect()
    }

    pub fn hidden(&self) -> bool {
        self.settings.hidden
    }

    pub fn respect_ignore_files(&self) -> bool {
        self.settings.respect_ignore_files
    }

    /// Effective exclusion patterns, defaults first.
    pub fn exclusions(&self) -> Vec<String> {
        Self::patterns(&self.settings)
    }

    pub fn is_excluded(&self, path: &str) -> bool {
        self.excludes.is_match(path)
    }

    /// Indices into [`Scope::exclusions`] of the patterns that leave out the
    /// file or directory at `path`; only `<dir>/**` patterns prune a directory,
    /// so a bare folder name excludes nothing.
    pub fn excluded_by(&self, path: &str, directory: bool) -> Vec<usize> {
        match directory {
            true => self
                .pruned_directories
                .matches(path)
                .into_iter()
                .map(|index| self.pruning_patterns[index])
                .collect(),
            false => self.excludes.matches(path),
        }
    }

    /// How many of [`Scope::exclusions`] are default exclusions, which come first.
    pub(super) fn default_count(&self) -> usize {
        match self.settings.default_excludes {
            true => DEFAULT_EXCLUDES.len(),
            false => 0,
        }
    }
}
