//! Measurement policy: which files are selected and how each one is classified.
//!
//! The input is the version-1 TOML contract in docs/PRODUCT.md.
use crate::{modules::Module, paths::ancestors, Kind};
use anyhow::{anyhow, bail, ensure, Context, Result};
use globset::{Glob, GlobBuilder, GlobSet, GlobSetBuilder};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap},
    fmt,
    path::{Path, PathBuf},
};
use tokei::LanguageType;

mod audit;
mod builtin;
mod corrections;
mod decision;
mod diagnostics;
mod entry;
mod exclusions;
mod file;
mod generated;
mod schema;
mod scope;
mod user;
mod workspace;
pub use audit::{
    merged_policy_entries, unused_entries, Displacement, EntryStage, PolicyEntry, Preemption,
    RECOGNITION,
};
use builtin::{
    Convention, BUILTIN_BUILD_V1, BUILTIN_LANGUAGES_V1, BUILTIN_TESTS_V1, DEFAULT_EXCLUDES,
};
pub use corrections::{CorrectedLanguage, UNRECOGNIZED};
use corrections::{CorrectionConfig, LanguageCorrection};
pub(crate) use decision::Resolution;
pub use decision::{Decision, RefinementEvidence, Stage};
use entry::{check_categories, check_entry_ids, check_ids, Entry};
pub use file::*;
use generated::GENERATED_CATEGORY;
pub use generated::{GeneratedEvidence, GeneratedMarker};
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
                let source = input.lines().nth(line - 1);
                anyhow!(
                    "line {line}, column {column}: {}",
                    diagnostics::toml_message(message, source)
                )
            }
            None => anyhow!("{}", diagnostics::toml_message(message, None)),
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
    /// Tests and code that exists only to support them, such as fixtures, stand-in services, and test utilities. Programs that run, prepare, or score tests, such as test runners, graders, and CI gates, are production with a category such as `verification-tooling`.
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
        self.purpose == Purpose::Other && self.is_fallback()
    }

    /// Whether no stage decided this file, whatever the fallback purpose.
    pub fn is_fallback(&self) -> bool {
        self.rule == FALLBACK
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

/// Scope and classification for one measurement.
#[derive(Debug)]
pub struct Policy {
    scope: Scope,
    overrides: Vec<Entry>,
    /// Built-in test conventions, then built-in build conventions.
    conventions: Vec<Entry>,
    rules: Vec<Entry>,
    /// Category-only entries applied to decided files, see [`Policy::refine`].
    refinements: Vec<Entry>,
    /// Whether build descriptors may declare production source.
    detect_modules: bool,
    /// Whether generated-code markers refine module claims.
    generated: bool,
    /// Modules adopted from the scan; empty when detection is disabled.
    modules: Vec<Module>,
    /// One production entry per adopted module; applied after rules and
    /// before the fallback.
    module_entries: Vec<(Module, Entry)>,
    /// Indices into `module_entries` by module folder, so a file's innermost
    /// module is found by walking its folders rather than every module.
    module_folders: HashMap<String, Vec<usize>>,
    fallback: Purpose,
    /// Literal exclusions of the policy files inside the scan root: the
    /// project configuration, user defaults, or a workspace file.
    configuration_exclusions: BTreeSet<String>,
    /// Project mappings first, then personal and built-in ones.
    languages: Vec<LanguageMapping>,
    /// How many leading `languages` the project configuration defines. Only
    /// these are audited: personal mappings apply to every project, so one
    /// that matches nothing here is expected.
    project_mappings: usize,
    /// Project corrections, applied before the counter recognizes a file.
    corrections: Vec<LanguageCorrection>,
    snapshot: EffectiveConfig,
}

impl Policy {
    /// Built-in settings used when no configuration file exists.
    pub fn defaults() -> Self {
        Self::from_toml("version = 1").expect("built-in configuration is valid")
    }

    pub fn from_toml(input: &str) -> Result<Self> {
        let config: ConfigFile = parse_toml(input).context("Invalid TOML configuration")?;
        let project_mappings = config.languages.mappings.len();
        Self::from_config(config, project_mappings)
    }

    fn from_config(mut config: ConfigFile, project_mappings: usize) -> Result<Self> {
        // Defaults belong to a new measurement, not to historical receipt decoding.
        let detect_modules = *config.classification.builtin_modules.get_or_insert(true);
        let generated = *config.classification.builtin_generated.get_or_insert(true);
        ensure!(
            config.version == 1,
            "Unsupported configuration version {}. Locmetry {} reads version 1. A newer release may read this file.",
            config.version,
            env!("CARGO_PKG_VERSION")
        );
        check_entry_ids(&config, project_mappings)?;
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
                except: None,
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
                    patterns: convention.patterns.iter().map(|s| s.to_string()).collect(),
                    except: GlobSet::empty(),
                    exceptions: Vec::new(),
                    kinds: None,
                    purpose: convention.purpose,
                    category: convention.category.into(),
                    reason: None,
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
                        || format!("In 'match' of language mapping '{}'", mapping.id),
                    )?,
                    patterns: mapping.patterns.clone(),
                    language: LanguageType::from_name(&mapping.language).with_context(|| {
                        format!(
                            "Language mapping '{}': {}",
                            mapping.id,
                            diagnostics::unknown_language(&mapping.language)
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
                patterns: patterns.iter().map(|s| s.to_string()).collect(),
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
            builtin_generated_markers: GeneratedMarker::V1
                .iter()
                .filter(|_| generated)
                .map(|marker| marker.rule())
                .collect(),
        };
        let fallback = config.classification.fallback;
        let corrections = config
            .languages
            .corrections
            .iter()
            .map(LanguageCorrection::from_config)
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            scope: Scope::new(config.scope.clone())?,
            overrides: entries(&config.overrides)?,
            conventions,
            rules: entries(&config.rules)?,
            refinements: config
                .refinements
                .iter()
                .map(Entry::from_refinement)
                .collect::<Result<_>>()?,
            detect_modules,
            generated,
            modules: Vec::new(),
            module_entries: Vec::new(),
            module_folders: HashMap::new(),
            fallback,
            configuration_exclusions: BTreeSet::new(),
            languages,
            project_mappings,
            corrections,
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
        let project_mappings = config.languages.mappings.len();
        if let Some(path) = user {
            UserDefaults::load(path)?.merge_into(&mut config, path)?;
        }
        let sources: Vec<String> = [project, user]
            .into_iter()
            .flatten()
            .map(|path| format!("'{}'", path.display()))
            .collect();
        Self::from_config(config, project_mappings)
            .with_context(|| format!("Invalid configuration {}", sources.join(" with ")))
    }

    pub fn override_scope(&mut self, overrides: &ScopeOverrides) -> Result<()> {
        self.scope = self.scope.with_overrides(overrides)?;
        Ok(())
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
                    patterns: module.sources.clone(),
                    except: glob_set(module.excluded.iter().map(String::as_str))?,
                    exceptions: module.excluded.clone(),
                    kinds: None,
                    purpose: Purpose::Production,
                    category: MODULE_CATEGORY.into(),
                    reason: None,
                };
                Ok((module.clone(), entry))
            })
            .collect::<Result<_>>()?;
        self.module_folders = HashMap::new();
        for (index, module) in modules.iter().enumerate() {
            self.module_folders
                .entry(module.path.clone())
                .or_default()
                .push(index);
        }
        self.modules = modules.to_vec();
        Ok(())
    }

    /// The first refinement for a file decided as `decided`. Files that
    /// reached the fallback stay `unclaimed` for review, and files a
    /// generated-code marker refined keep that documented evidence.
    fn refinement_for(&self, path: &str, kind: Kind, decided: &Classification) -> Option<&Entry> {
        // Runs for every counted file, so the common case without refinements
        // returns before comparing marker provenance.
        let marked = || {
            let id = decided.rule.strip_prefix(BUILTIN);
            GeneratedMarker::V1.iter().any(|m| id == Some(m.id()))
        };
        if self.refinements.is_empty() || decided.is_fallback() || marked() {
            return None;
        }
        self.refinements.iter().find(|refinement| {
            refinement.purpose == decided.purpose && refinement.matches(path, kind)
        })
    }

    /// Replaces the category of a decided file with its first refinement's,
    /// recording the refinement; purpose and `rule` stay.
    pub(crate) fn refine(&self, row: &mut crate::FileRecord) {
        if let Some(refinement) = self.refinement_for(&row.path, row.kind, &row.classification) {
            row.classification.category = refinement.category.clone();
            row.refinement = Some(refinement.id.clone());
        }
    }

    /// Modules whose source this policy classifies.
    pub fn modules(&self) -> &[Module] {
        &self.modules
    }

    pub fn classify(&self, path: &str, kind: Kind) -> Classification {
        self.classification(self.deciding_entry(path, kind))
    }

    fn classification(&self, decided: Option<(Stage, &Entry, Option<&Module>)>) -> Classification {
        decided
            .map(|(_, entry, _)| entry.classification())
            .unwrap_or_else(|| Classification {
                purpose: self.fallback,
                category: UNCLAIMED.into(),
                rule: FALLBACK.into(),
            })
    }

    /// Classification, module credit and marker eligibility of a counted
    /// file from one first-match decision.
    ///
    /// The credited module is the innermost module whose layout covers
    /// production source, whichever stage decided it, so an entry that keeps
    /// `production` and only names a category keeps the module's credit
    /// (docs/PRODUCT.md, Module Detection). A marker refines only source a
    /// module layout decided, so overrides, conventions and rules keep their
    /// files (docs/PRODUCT.md, Matching and Precedence).
    pub(crate) fn resolve(&self, path: &str, kind: Kind) -> Resolution {
        let decided = self.deciding_entry(path, kind);
        let by_module = matches!(decided, Some((Stage::Module, ..)));
        let classification = self.classification(decided);
        let module = (kind == Kind::Source && classification.purpose == Purpose::Production)
            .then(|| match decided {
                Some((Stage::Module, _, module)) => module,
                _ => self.module_entry(path, kind).map(|(module, _)| module),
            })
            .flatten()
            .map(crate::contract::Module::from);
        Resolution {
            classification,
            module,
            markable: self.generated && kind == Kind::Source && by_module,
        }
    }

    fn deciding_entry(&self, path: &str, kind: Kind) -> Option<(Stage, &Entry, Option<&Module>)> {
        self.first_match(path, kind, None)
    }

    /// The classification later stages give a file when the project entry
    /// `skipped` is absent; `None` when the file would reach the fallback.
    fn decision_without(&self, path: &str, kind: Kind, skipped: &str) -> Option<Classification> {
        self.first_match(path, kind, Some(skipped))
            .map(|(_, entry, _)| entry.classification())
    }

    fn first_match(
        &self,
        path: &str,
        kind: Kind,
        skipped: Option<&str>,
    ) -> Option<(Stage, &Entry, Option<&Module>)> {
        for (stage, entries) in [
            (Stage::Override, &self.overrides),
            (Stage::Convention, &self.conventions),
            (Stage::Rule, &self.rules),
        ] {
            if let Some(entry) = entries
                .iter()
                .find(|entry| Some(entry.id.as_str()) != skipped && entry.matches(path, kind))
            {
                return Some((stage, entry, None));
            }
        }
        self.module_entry(path, kind)
            .map(|(module, entry)| (Stage::Module, entry, Some(module)))
    }

    /// Whether a scan must look for generated-code markers: they refine only
    /// module claims, so without modules nothing reads file content for them.
    pub(crate) fn reads_generated_markers(&self) -> bool {
        self.generated && self.detect_modules
    }

    pub(crate) fn decision(
        &self,
        path: &str,
        kind: Kind,
        evidence: Option<&GeneratedEvidence>,
    ) -> Decision {
        let Some((stage, entry, module)) = self.deciding_entry(path, kind) else {
            return Decision {
                stage: Stage::Fallback,
                matched_patterns: Vec::new(),
                reason: None,
                module: None,
                generated: None,
                refinement: None,
            };
        };
        let generated = evidence
            .filter(|_| self.generated && kind == Kind::Source && stage == Stage::Module)
            .cloned();
        let refinement = match generated {
            Some(_) => None,
            None => self.refinement_for(path, kind, &entry.classification()),
        }
        .map(|refinement| RefinementEvidence {
            id: refinement.id.clone(),
            matched_patterns: refinement.matched_patterns(path),
            reason: refinement.reason.clone(),
        });
        Decision {
            stage,
            matched_patterns: entry.matched_patterns(path),
            reason: entry.reason.clone(),
            module: module.map(crate::contract::Module::from),
            generated,
            refinement,
        }
    }

    /// Classification by the innermost module containing `path`, as build
    /// tools treat nested projects: an outer module's layout never claims a
    /// nested module's files, while build systems sharing the innermost
    /// directory combine their layouts.
    fn module_entry(&self, path: &str, kind: Kind) -> Option<&(Module, Entry)> {
        let innermost = ancestors(path).find_map(|folder| self.module_folders.get(folder))?;
        innermost
            .iter()
            .map(|&index| &self.module_entries[index])
            .find(|(_, entry)| entry.matches(path, kind))
    }

    pub fn is_excluded(&self, path: &str) -> bool {
        self.scope.is_excluded(path)
    }

    /// The first correction matching `path`, with its ID; it replaces the
    /// counter's recognition, and mappings no longer apply.
    pub fn corrected_language(&self, path: &str) -> Option<(&str, CorrectedLanguage)> {
        self.corrections
            .iter()
            .find(|correction| correction.matcher.is_match(path))
            .map(|correction| (correction.id.as_str(), correction.language))
    }

    /// The stated reason of correction `id`, when supplied.
    pub fn correction_reason(&self, id: &str) -> Option<&str> {
        self.corrections
            .iter()
            .find(|c| c.id == id)
            .and_then(|c| c.reason.as_deref())
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
