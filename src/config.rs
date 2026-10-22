//! Measurement policy: which files are selected and how each one is classified.
//!
//! The input is the version-1 TOML contract in docs/PRODUCT.md.
use crate::{backend::relative_path, modules::Module, Kind};
use anyhow::{anyhow, bail, ensure, Context, Result};
use globset::{Glob, GlobBuilder, GlobSet, GlobSetBuilder};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    path::{Path, PathBuf},
};
use tokei::LanguageType;

mod builtin;
mod file;
mod schema;
mod scope;
mod user;
mod workspace;
use builtin::{
    Convention, BUILTIN_BUILD_V1, BUILTIN_LANGUAGES_V1, BUILTIN_TESTS_V1, DEFAULT_EXCLUDES,
};
pub use file::*;
pub use schema::{config_schema, user_config_schema, workspace_schema};
pub use scope::*;
pub use user::*;
pub use workspace::*;

/// Configuration file discovered in the scan root.
pub const CONFIG_FILE: &str = ".locmetryrc";

/// Provenance of files that no entry matched.
const FALLBACK: &str = "fallback";
/// Category of files no stage decided, whatever the fallback purpose.
const UNCLAIMED: &str = "unclaimed";

/// Parses TOML, reporting a failure as one line: `line L, column C: message`.
///
/// The parser's own report is a multi-line source snippet, and stderr
/// sanitizing turns newlines into spaces, which garbled it into one line.
fn parse_toml<T: serde::de::DeserializeOwned>(input: &str) -> Result<T> {
    toml::from_str(input).map_err(|error| {
        let message = error.message();
        match error.span() {
            Some(span) => {
                let before = &input[..span.start];
                let line = before.matches('\n').count() + 1;
                let column = before.rsplit('\n').next().unwrap_or(before).chars().count() + 1;
                anyhow!(
                    "line {line}, column {column}: {}",
                    crate::output::single_quoted(message)
                )
            }
            None => anyhow!("{}", crate::output::single_quoted(message)),
        }
    })
}

/// Category of source that a build module classifies.
const MODULE_CATEGORY: &str = "implementation";

/// Provenance prefix of built-in test and build conventions.
const BUILTIN: &str = "builtin:";

/// What a file is for, decided by whether code is maintained rather than whether it ships; `category` then says what kind of code it is, so `source_categories` separates the product from its tooling.
#[derive(Clone, Copy, Debug, Serialize, JsonSchema, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Purpose {
    /// Maintained non-test source, not necessarily shipped runtime code: the product itself and the tooling, build logic, examples, and demos a project keeps working. Tell these apart with `category`, such as `implementation`, `tooling`, `build-tooling`, or `example`, not with another purpose.
    Production,
    /// Tests and code that exists only to support them, such as fixtures, stand-in services, and test utilities. Programs that run, prepare, or score tests, such as test runners, graders, and CI gates, are production with a category such as `tooling`.
    Test,
    /// Source that counts as neither production nor test. Code no stage claims lands here with category `unclaimed` and is listed for review as `summary.unclaimed_folders`; code an entry or convention assigns here on purpose, such as abandoned prototypes, archived experiments, copied snippets, or build-tool wrappers, is counted but not listed. Maintained code that is not the product, such as tooling or examples, is production with its own category.
    Other,
}

impl Purpose {
    pub const ALL: [Self; 3] = [Self::Production, Self::Test, Self::Other];

    /// Position in [`Self::ALL`].
    pub fn index(self) -> usize {
        match self {
            Self::Production => 0,
            Self::Test => 1,
            Self::Other => 2,
        }
    }
}

impl fmt::Display for Purpose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Production => "production",
            Self::Test => "test",
            Self::Other => "other",
        })
    }
}

/// Deserialized by name so the purpose retired in 0.5 names its replacement
/// instead of failing as an unknown variant.
impl<'de> Deserialize<'de> for Purpose {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        use serde::de::Error;
        let name = std::borrow::Cow::<str>::deserialize(deserializer)?;
        Self::ALL
            .into_iter()
            .find(|purpose| purpose.to_string() == name)
            .ok_or_else(|| match name.as_ref() {
                "unclassified" => D::Error::custom(
                    "purpose 'unclassified' was renamed to 'other' in locmetry 0.5",
                ),
                _ => D::Error::unknown_variant(&name, &["production", "test", "other"]),
            })
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct Classification {
    pub purpose: Purpose,
    pub category: String,
    pub rule: String,
}

impl Classification {
    /// Whether no stage decided this file, so it still needs a decision.
    /// Code an entry or convention assigns `other` on purpose, such as a
    /// parked prototype or a build wrapper, is decided.
    pub fn is_unclaimed(&self) -> bool {
        self.purpose == Purpose::Other && self.rule == FALLBACK
    }
}

/// Provenance of files a build module of `detector` classifies.
pub(crate) fn module_rule(detector: &str) -> String {
    format!("{BUILTIN}{detector}-module")
}

/// The detector of a [`module_rule`], if `rule` is one.
pub(crate) fn module_detector(rule: &str) -> Option<&str> {
    rule.strip_prefix(BUILTIN)?.strip_suffix("-module")
}

#[derive(Debug)]
struct Entry {
    id: String,
    matcher: GlobSet,
    kinds: Option<Vec<Kind>>,
    purpose: Purpose,
    category: String,
}

impl Entry {
    fn matches(&self, path: &str, kind: Kind) -> bool {
        self.matcher.is_match(path)
            && self
                .kinds
                .as_ref()
                .is_none_or(|kinds| kinds.contains(&kind))
    }

    fn classification(&self) -> Classification {
        Classification {
            purpose: self.purpose,
            category: self.category.clone(),
            rule: self.id.clone(),
        }
    }

    fn from_config(entry: &EntryConfig) -> Result<Self> {
        ensure!(
            !entry.patterns.is_empty(),
            "Entry {} needs a nonempty match array",
            entry.id
        );
        ensure!(
            entry.kind.as_ref().is_none_or(|kinds| !kinds.is_empty()),
            "Entry {} needs a nonempty kind array when kind is present",
            entry.id
        );
        Ok(Self {
            id: entry.id.clone(),
            matcher: glob_set(entry.patterns.iter().map(String::as_str))
                .with_context(|| format!("Invalid match in entry {}", entry.id))?,
            kinds: entry.kind.clone(),
            purpose: entry.purpose,
            category: entry
                .category
                .clone()
                .unwrap_or_else(|| entry.purpose.to_string()),
        })
    }
}

/// Rejects entry categories that a declared `[categories]` list does not
/// name; without a list, categories stay free-form.
fn check_categories(config: &ConfigFile) -> Result<()> {
    if config.categories.is_empty() {
        return Ok(());
    }
    let builtin: BTreeSet<String> = Purpose::ALL
        .iter()
        .map(Purpose::to_string)
        .chain(std::iter::once(MODULE_CATEGORY.to_string()))
        .chain(
            BUILTIN_TESTS_V1
                .iter()
                .chain(BUILTIN_BUILD_V1)
                .map(|convention| convention.category.to_string()),
        )
        .collect();
    for entry in config.overrides.iter().chain(&config.rules) {
        let Some(category) = &entry.category else {
            continue;
        };
        ensure!(
            config.categories.contains_key(category) || builtin.contains(category),
            "Entry '{}' uses category '{category}', which [categories] does not declare. Declared: {}. Built in: {}.",
            entry.id,
            config.categories.keys().cloned().collect::<Vec<_>>().join(", "),
            builtin.into_iter().collect::<Vec<_>>().join(", ")
        );
    }
    Ok(())
}

/// Rejects empty, duplicate, and reserved entry IDs across all stages.
fn check_ids<'a>(ids: impl IntoIterator<Item = &'a str>) -> Result<()> {
    let mut seen = BTreeSet::new();
    for id in ids {
        ensure!(
            !id.is_empty() && seen.insert(id),
            "Entry ID '{id}' is empty or not unique"
        );
        ensure!(
            id != FALLBACK && !id.starts_with(BUILTIN),
            "Entry ID '{id}' is reserved for '{FALLBACK}' or '{BUILTIN}' provenance"
        );
    }
    Ok(())
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Stage {
    Override,
    Convention,
    Rule,
    Module,
    Fallback,
}

impl std::fmt::Display for Stage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Override => "override",
            Self::Convention => "convention",
            Self::Rule => "rule",
            Self::Module => "module",
            Self::Fallback => "fallback",
        })
    }
}

/// Evidence from the same first-match decision as measurement: overrides, conventions, rules, innermost module, fallback. Later matching entries do not decide the file.
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Decision {
    pub stage: Stage,
    /// Matching globs of the winning entry, relative to the member's root. Empty for fallback.
    pub matched_patterns: Vec<String>,
    /// The winning user entry's stated reason, when supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    /// Winning descriptor-inferred module; this is layout inference, not semantic build membership.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub module: Option<crate::contract::Module>,
}

/// Scope and classification for one measurement.
#[derive(Debug)]
pub struct Policy {
    scope: Scope,
    overrides: Vec<Entry>,
    /// Built-in test conventions, then built-in build conventions.
    conventions: Vec<Entry>,
    rules: Vec<Entry>,
    /// Whether build descriptors may declare production source.
    detect_modules: bool,
    /// Modules adopted from the scan; empty when detection is disabled.
    modules: Vec<Module>,
    /// One production entry per adopted module; applied after rules and
    /// before the fallback.
    module_entries: Vec<(Module, Entry)>,
    fallback: Purpose,
    /// Literal exclusions of the policy files inside the scan root: the
    /// project configuration, user defaults, or a workspace file.
    configuration_exclusions: BTreeSet<String>,
    /// Project mappings first, then built-in ones.
    languages: Vec<LanguageMapping>,
    snapshot: EffectiveConfig,
}

impl Policy {
    /// Built-in settings used when no configuration file exists.
    pub fn defaults() -> Self {
        Self::from_toml("version = 1").expect("built-in configuration is valid")
    }

    pub fn from_toml(input: &str) -> Result<Self> {
        Self::from_config(parse_toml(input).context("Invalid TOML configuration")?)
    }

    fn from_config(mut config: ConfigFile) -> Result<Self> {
        // Defaults belong to a new measurement, not to historical receipt decoding.
        let detect_modules = *config.classification.builtin_modules.get_or_insert(true);
        ensure!(
            config.version == 1,
            "Unsupported configuration version {}",
            config.version
        );
        check_ids(
            config
                .overrides
                .iter()
                .chain(&config.rules)
                .map(|entry| entry.id.as_str()),
        )?;
        check_categories(&config)?;
        let entries = |list: &[EntryConfig]| {
            list.iter()
                .map(Entry::from_config)
                .collect::<Result<Vec<_>>>()
        };
        let conventions = [
            (config.classification.builtin_tests, BUILTIN_TESTS_V1),
            (config.classification.builtin_build, BUILTIN_BUILD_V1),
        ]
        .into_iter()
        .filter(|(enabled, _)| *enabled)
        .flat_map(|(_, set)| set.iter())
        .collect::<Vec<_>>();
        let convention_snapshot = conventions
            .iter()
            .map(|convention| EntryConfig {
                id: format!("{BUILTIN}{}", convention.id),
                patterns: convention.patterns.iter().map(|s| s.to_string()).collect(),
                purpose: convention.purpose,
                category: Some(convention.category.into()),
                kind: None,
                reason: None,
            })
            .collect();
        let conventions = conventions
            .into_iter()
            .map(|convention| {
                Ok(Entry {
                    id: format!("{BUILTIN}{}", convention.id),
                    matcher: glob_set(convention.patterns.iter().copied())?,
                    kinds: None,
                    purpose: convention.purpose,
                    category: convention.category.into(),
                })
            })
            .collect::<Result<Vec<_>>>()?;
        check_ids(
            config
                .languages
                .mappings
                .iter()
                .map(|mapping| mapping.id.as_str()),
        )?;
        let mut languages = config
            .languages
            .mappings
            .iter()
            .map(|mapping| {
                ensure!(
                    !mapping.patterns.is_empty(),
                    "Language mapping {} needs a nonempty match array",
                    mapping.id
                );
                Ok(LanguageMapping {
                    id: mapping.id.clone(),
                    matcher: glob_set(mapping.patterns.iter().map(String::as_str)).with_context(
                        || format!("Invalid match in language mapping '{}'", mapping.id),
                    )?,
                    language: LanguageType::from_name(&mapping.language).with_context(|| {
                        format!(
                            "Unknown language '{}' in language mapping '{}'",
                            mapping.language, mapping.id
                        )
                    })?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let builtin_languages = BUILTIN_LANGUAGES_V1
            .iter()
            .filter(|_| config.languages.builtin)
            .collect::<Vec<_>>();
        for (id, patterns, language) in &builtin_languages {
            languages.push(LanguageMapping {
                id: format!("{BUILTIN}{id}"),
                matcher: glob_set(patterns.iter().copied())?,
                language: *language,
            });
        }
        let snapshot = EffectiveConfig {
            configuration: config.clone(),
            builtin_language_mappings: builtin_languages
                .iter()
                .map(|(id, patterns, language)| MappingConfig {
                    id: format!("{BUILTIN}{id}"),
                    patterns: patterns.iter().map(|s| s.to_string()).collect(),
                    language: language.name().into(),
                })
                .collect(),
            builtin_conventions: convention_snapshot,
        };
        let fallback = config.classification.fallback;
        Ok(Self {
            scope: Scope::new(config.scope.clone())?,
            overrides: entries(&config.overrides)?,
            conventions,
            rules: entries(&config.rules)?,
            detect_modules,
            modules: Vec::new(),
            module_entries: Vec::new(),
            fallback,
            configuration_exclusions: BTreeSet::new(),
            languages,
            snapshot,
        })
    }

    /// Loads an explicit or discovered TOML configuration file.
    pub fn load(path: &Path) -> Result<Self> {
        Self::load_layers(Some(path), None)
    }

    /// Builds the policy from an optional project configuration merged with
    /// optional user defaults; without either, built-in settings apply.
    pub fn load_layers(project: Option<&Path>, user: Option<&Path>) -> Result<Self> {
        let mut config: ConfigFile = match project {
            Some(path) => {
                // JSON/regex profiles were retired after strict TOML translations
                // reproduced every reviewed classification (docs/PLAN.md section 4).
                ensure!(
                    path.extension().is_none_or(|extension| extension != "json"),
                    "'{}' looks like a retired JSON/regex profile. Translate it to TOML (docs/PRODUCT.md).",
                    path.display()
                );
                let input = std::fs::read_to_string(path)
                    .with_context(|| format!("Cannot read configuration '{}'", path.display()))?;
                parse_toml(&input)
                    .with_context(|| format!("Invalid configuration '{}'", path.display()))?
            }
            None => toml::from_str("version = 1").expect("built-in configuration is valid"),
        };
        if let Some(path) = user {
            UserDefaults::load(path)?.merge_into(&mut config, path)?;
        }
        let sources: Vec<String> = [project, user]
            .into_iter()
            .flatten()
            .map(|path| format!("'{}'", path.display()))
            .collect();
        Self::from_config(config)
            .with_context(|| format!("Invalid configuration {}", sources.join(" with ")))
    }

    pub fn override_scope(&mut self, overrides: &ScopeOverrides) -> Result<()> {
        self.scope = self.scope.with_overrides(overrides)?;
        Ok(())
    }

    /// Excludes the loaded configuration file when it lies inside the
    /// canonical scan `root`.
    ///
    /// The configuration is measurement policy, not project content. Its path
    /// is escaped so glob metacharacters in its name match only itself.
    pub fn exclude_configuration(&mut self, root: &Path, config: &Path) -> Result<()> {
        let config = dunce::canonicalize(config)
            .with_context(|| format!("Cannot resolve configuration '{}'", config.display()))?;
        let Ok(relative) = config.strip_prefix(root) else {
            return Ok(());
        };
        let pattern = literal_glob(&relative_path(relative)?);
        self.configuration_exclusions.insert(pattern.clone());
        self.override_scope(&ScopeOverrides {
            exclude: vec![pattern],
            ..Default::default()
        })
    }

    /// Configured exclusions that left out no file or folder, given the
    /// indices into the effective exclusions that a scan applied.
    ///
    /// Default exclusions and the configuration's own path are omitted: they
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

    /// Adopts the modules a scan detected, unless `builtin_modules = false`.
    ///
    /// Their production source is classified after the project's rules, so
    /// project knowledge wins and only files that would otherwise reach the
    /// fallback change (docs/PRODUCT.md, Matching and Precedence).
    pub fn adopt_modules(&mut self, modules: &[Module]) -> Result<()> {
        if !self.detect_modules {
            return Ok(());
        }
        self.module_entries = modules
            .iter()
            .map(|module| {
                let entry = Entry {
                    id: module_rule(module.detector),
                    matcher: glob_set(module.sources.iter().map(String::as_str))?,
                    kinds: None,
                    purpose: Purpose::Production,
                    category: MODULE_CATEGORY.into(),
                };
                Ok((module.clone(), entry))
            })
            .collect::<Result<_>>()?;
        self.modules = modules.to_vec();
        Ok(())
    }

    /// IDs of the project's overrides and rules, in order.
    pub fn entry_ids(&self) -> impl Iterator<Item = &str> {
        self.overrides
            .iter()
            .chain(&self.rules)
            .map(|entry| entry.id.as_str())
    }

    /// Modules whose source this policy classifies.
    pub fn modules(&self) -> &[Module] {
        &self.modules
    }

    pub fn classify(&self, path: &str, kind: Kind) -> Classification {
        self.deciding_entry(path, kind)
            .map(|(_, entry, _)| entry.classification())
            .unwrap_or_else(|| Classification {
                purpose: self.fallback,
                category: UNCLAIMED.into(),
                rule: FALLBACK.into(),
            })
    }

    fn deciding_entry(&self, path: &str, kind: Kind) -> Option<(Stage, &Entry, Option<&Module>)> {
        for (stage, entries) in [
            (Stage::Override, &self.overrides),
            (Stage::Convention, &self.conventions),
            (Stage::Rule, &self.rules),
        ] {
            if let Some(entry) = entries.iter().find(|entry| entry.matches(path, kind)) {
                return Some((stage, entry, None));
            }
        }
        self.module_entry(path, kind)
            .map(|(module, entry)| (Stage::Module, entry, Some(module)))
    }

    pub(crate) fn decision(&self, path: &str, kind: Kind) -> Decision {
        let Some((stage, entry, module)) = self.deciding_entry(path, kind) else {
            return Decision {
                stage: Stage::Fallback,
                matched_patterns: Vec::new(),
                reason: None,
                module: None,
            };
        };
        let config = self
            .snapshot
            .configuration
            .overrides
            .iter()
            .chain(&self.snapshot.configuration.rules)
            .chain(&self.snapshot.builtin_conventions)
            .find(|config| config.id == entry.id);
        let patterns = match module {
            Some(module) => &module.sources,
            None => &config.expect("compiled entry has configuration").patterns,
        };
        Decision {
            stage,
            matched_patterns: entry
                .matcher
                .matches(path)
                .iter()
                .map(|&i| patterns[i].clone())
                .collect(),
            reason: config.and_then(|config| config.reason.clone()),
            module: module.map(|module| crate::contract::Module {
                path: module.path.clone(),
                detector: module.detector.into(),
            }),
        }
    }

    /// Classification by the innermost module containing `path`, as build
    /// tools treat nested projects: an outer module's layout never claims a
    /// nested module's files, while build systems sharing the innermost
    /// directory combine their layouts.
    fn module_entry(&self, path: &str, kind: Kind) -> Option<&(Module, Entry)> {
        let innermost = self
            .module_entries
            .iter()
            .map(|(module, _)| module)
            .filter(|module| module.contains(path))
            .max_by_key(|module| module.depth())?
            .path
            .as_str();
        self.module_entries
            .iter()
            .filter(|(module, _)| module.path == innermost)
            .find(|(_, entry)| entry.matches(path, kind))
    }

    pub fn is_excluded(&self, path: &str) -> bool {
        self.scope.is_excluded(path)
    }

    /// Language and mapping ID for a file the backend does not recognize.
    pub fn mapped_language(&self, path: &str) -> Option<(&str, LanguageType)> {
        self.languages
            .iter()
            .find(|mapping| mapping.matcher.is_match(path))
            .map(|mapping| (mapping.id.as_str(), mapping.language))
    }

    pub fn scope(&self) -> &Scope {
        &self.scope
    }

    pub fn snapshot(&self) -> &EffectiveConfig {
        &self.snapshot
    }
}
