//! The version-1 TOML contract of `.locmetryrc`, and the effective configuration reports record.
use super::*;

/// Which files are measured. Scope applies before classification, so an excluded file appears nowhere in the report.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "Scope")]
pub struct ScopeSettings {
    /// Whether hidden files and directories are measured; `--hidden` overrides it.
    pub hidden: bool,
    /// Whether `.ignore`, `.tokeignore`, and, inside a Git work tree, `.gitignore` files limit the selection; `--respect-ignore-files` overrides it.
    pub respect_ignore_files: bool,
    /// Whether the default exclusions apply; `false` is the only way to drop them.
    #[schemars(transform = schema::describe_default_excludes)]
    pub default_excludes: bool,
    /// Globs of files to leave out, added after the default and personal exclusions; `--exclude` adds more. Use it for code that is not the project's own, such as vendored libraries, Git submodules (reports list them as `summary.submodules` until excluded), or copied reference material; exclude a folder as `<folder>/**`. The project's own code stays measured and gets an entry instead.
    #[schemars(extend("items" = {"type": "string", "description": "Root-relative glob."}))]
    pub exclude: Vec<String>,
}

impl Default for ScopeSettings {
    fn default() -> Self {
        Self {
            hidden: true,
            respect_ignore_files: true,
            default_excludes: true,
            exclude: Vec::new(),
        }
    }
}

/// Explicit CLI settings, which take precedence over project settings.
#[derive(Debug, Default)]
pub struct ScopeOverrides {
    pub hidden: Option<bool>,
    pub respect_ignore_files: Option<bool>,
    /// Added to the configured exclusions.
    pub exclude: Vec<String>,
}

/// The counting engine.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub(super) enum Backend {
    #[default]
    Tokei,
}

/// Switches for the built-in classification stages and the fallback.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "Classification")]
pub(super) struct ClassificationSettings {
    /// Whether built-in test conventions classify files, after `overrides` and before `rules`. A test convention also wins over a `rules` entry that matches the same file, so tests inside production trees stay tests; an `overrides` entry is the way to beat one.
    #[schemars(transform = schema::describe_test_conventions)]
    pub(super) builtin_tests: bool,
    /// Whether built-in build conventions classify files, right after the test conventions: build scripts are `production` with category `build-tooling`, and build-tool wrappers are `other` with category `third-party-wrapper`.
    #[schemars(transform = schema::describe_build_conventions)]
    pub(super) builtin_build: bool,
    /// Whether build modules classify files that no override, convention, or rule matched. A build module is a folder where a selected descriptor triggers conventional source-layout inference; build systems call it a module, project, package, or crate, and it is unrelated to modules or packages inside a language. The build system's default production layout in that folder is `production` with category `implementation`. The innermost build module containing a file decides, and build files in one folder combine their layouts. Only file names are read, never the build file's content, so a source directory that a build file reconfigures needs a rule. Set `false` when build files mislead and rules classify the production source; a rule alone corrects a single folder. The headline counts, per build system, the modules credited with production source their layout covers, whichever stage decided it, and that code as `summary.build_modules`; full JSON from `--format json` or the default `--output FILE` export lists every module as `metadata.modules`.
    #[schemars(transform = schema::describe_detectors)]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::contract::present"
    )]
    #[schemars(with = "bool", extend("default" = true))]
    pub(super) builtin_modules: Option<bool>,
    /// Whether generated-code markers refine build-module claims: module source that carries a documented marker is `production` with category `generated` instead of `implementation`, recorded with the marker as its `rule`. Overrides, conventions and rules decide first, and purpose, totals and module credit stay the same.
    #[schemars(transform = schema::describe_generated_markers)]
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "crate::contract::present"
    )]
    #[schemars(with = "bool", extend("default" = true))]
    pub(super) builtin_generated: Option<bool>,
    /// Purpose of files that no stage matched; their category is `unclaimed` and their `rule` is `fallback`. Keep `other` so unclaimed code is reported for review instead of silently counted.
    pub(super) fallback: Purpose,
}

impl Default for ClassificationSettings {
    fn default() -> Self {
        Self {
            builtin_tests: true,
            builtin_build: true,
            builtin_modules: None,
            builtin_generated: None,
            fallback: Purpose::Other,
        }
    }
}

/// A classification entry: files matching it get its purpose and category.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "Entry")]
pub(super) struct EntryConfig {
    /// Unique across `overrides` and `rules`; must not be `fallback` or start with `builtin:`. Files this entry decides record the ID as their `rule`.
    #[schemars(length(min = 1))]
    pub(super) id: String,
    /// Root-relative globs; the entry matches a file when any of them matches.
    #[serde(rename = "match")]
    #[schemars(length(min = 1), extend("items" = {"type": "string"}))]
    pub(super) patterns: Vec<String>,
    /// Root-relative globs of files the entry leaves out although `match` matches them; they fall through to later entries and stages, as if the entry were absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1), extend("items" = {"type": "string"}))]
    pub(super) except: Option<Vec<String>>,
    /// Restricts the entry to files of these content kinds; absent means every kind.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(length(min = 1))]
    pub(super) kind: Option<Vec<Kind>>,
    pub(super) purpose: Purpose,
    /// Free-form subcategory reported with the purpose, such as `implementation`, `tooling`, `generated`, `example`, or `test-support`; defaults to the purpose.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) category: Option<String>,
    /// Why the entry exists, for reviewers; recorded in the report's effective configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) reason: Option<String>,
}

/// How files that the counter does not recognize are counted.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(default, deny_unknown_fields)]
#[schemars(rename = "Languages")]
pub(super) struct LanguageSettings {
    /// Whether the built-in language mappings apply, after `mappings`.
    #[schemars(transform = schema::describe_language_mappings)]
    pub(super) builtin: bool,
    /// Ordered mappings; the first matching one counts the file. They apply only to files the counter does not recognize, so they never change a recognized file's language. Reports list unrecognized files as `metadata.coverage.unrecognized`.
    pub(super) mappings: Vec<MappingConfig>,
    /// Ordered corrections for files the counter recognized wrongly, such as ProGuard rules read as Prolog; the first matching one decides, before the counter recognizes the file, and mappings no longer apply to it. Reports name the correction as each counted file's `language_correction`. Only the project configuration may correct languages.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) corrections: Vec<CorrectionConfig>,
}

impl Default for LanguageSettings {
    fn default() -> Self {
        Self {
            builtin: true,
            mappings: Vec::new(),
            corrections: Vec::new(),
        }
    }
}

/// Counts matching unrecognized files as a known language.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "LanguageMapping")]
pub(super) struct MappingConfig {
    /// Unique among mappings; reports name it for files it counted.
    #[schemars(length(min = 1))]
    pub(super) id: String,
    /// Root-relative globs; the mapping matches a file when any of them matches.
    #[serde(rename = "match")]
    #[schemars(length(min = 1), extend("items" = {"type": "string"}))]
    pub(super) patterns: Vec<String>,
    /// Language whose parser counts the file.
    #[schemars(schema_with = "schema::language_names")]
    pub(super) language: String,
}

#[derive(Debug)]
pub(super) struct LanguageMapping {
    pub(super) id: String,
    pub(super) matcher: GlobSet,
    pub(super) language: LanguageType,
}

/// Locmetry project configuration: TOML in `.locmetryrc` at the scan root, or the file `--config` names.
///
/// Every measured file gets one purpose (`production`, `test`, or `other`) and a category from the first stage that matches: (1) `overrides`, in order; (2) built-in test conventions, then built-in build conventions; (3) `rules`, in order; (4) build modules, conventional source layouts inferred from descriptors; (5) the fallback. Scope exclusions apply before all stages. The stage order is fixed, so where `[[overrides]]` and `[[rules]]` appear in the file does not matter; only the order within each list decides between entries that match the same file. A broad rule never swallows conventional tests or build scripts, and an override beats the conventions; but a rule does take source a build module would claim, so match only the folders a rule is about. Each file records the deciding stage in its `rule` field: an entry ID, `builtin:<convention>`, `builtin:<detector>-module`, or `fallback`.
///
/// This file is the only one that classifies: `--config` replaces `.locmetryrc`, while user defaults (`--schema user-config`) and command-line flags only add exclusions and language mappings or switch scope settings. The same files and configuration always give the same classification, independent of the order in which files are found.
///
/// Globs are root-relative, `/`-separated, and case-sensitive: `*` stays within a path segment, `**` spans directories, `?` matches one character, `[...]` a character class, `{a,b}` alternatives, and a backslash escapes. `*` also matches names that start with a dot. Negation, absolute paths, and basename-only matching are not supported, so `*.rs` matches only at the root; use `**/*.rs`. Globs match files, not folders: `docs/**` covers a folder, while `docs` matches only a file of that name.
///
/// To create or evaluate configuration, start with `locmetry --guide configure` and measure the unrestricted project before editing. Inspect `summary.submodules`, `summary.unclaimed_folders`, `summary.largest_module_files` and representative already-classified files: module layouts claim production without review, so generated or vendored code hides there. Names suggesting tests, fixtures or examples are prompts to inspect evidence, not classification verdicts. For source needing a decision, either add an evidence-based entry, record an intentional `purpose = "other"` entry with a `reason`, or leave it unclaimed and report an open question to the owner. Do not add entries merely to remove findings; correct built-in decisions need no redundant rules. Measure the unrestricted project again (without `--subfolder`) with `--config <file>` and review `summary.unused_entries` and `summary.unused_exclusions` when present; omission means no such findings. Review coverage separately. `--files --output -` shows each file's `purpose`, `category`, and `rule`. An entry looks like this:
///
/// ```toml
/// version = 1
///
/// [[rules]]
/// id = "tooling"
/// match = ["scripts/**"]
/// purpose = "production"
/// category = "tooling"
/// reason = "Maintained release scripts"
/// ```
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "Configuration")]
pub(super) struct ConfigFile {
    /// Contract version; must be `1`.
    #[schemars(extend("const" = 1))]
    pub(super) version: u32,
    #[serde(default)]
    pub(super) backend: Backend,
    #[serde(default)]
    pub(super) scope: ScopeSettings,
    #[serde(default)]
    pub(super) classification: ClassificationSettings,
    #[serde(default)]
    pub(super) languages: LanguageSettings,
    /// Optional list of the categories entries may use, each mapped to its meaning, such as `proof = "Machine-checked theorems behind the assurance claim"`. When present, an entry's `category` must be listed here or be built in: a purpose name, `implementation`, or a built-in convention's category. Declaring categories catches typos and keeps names consistent; reports record the meanings in the effective configuration.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub(super) categories: BTreeMap<String, String>,
    /// Entries checked first, before the built-in conventions; the first matching entry decides. Use them only for decisions that must beat a convention, such as a test-support package whose build scripts are test support, or archived code with test file names that must stay `other`.
    #[serde(default)]
    pub(super) overrides: Vec<EntryConfig>,
    /// Entries checked after the built-in conventions and before build modules; the first matching entry decides. Use them for ordinary project knowledge: maintained tooling or examples as `production` with a category, unmaintained prototypes as `other`, or source no build file declares; conventional tests, build scripts, and build-tool wrappers inside a matched tree keep their built-in classification, but build-module source does not.
    #[serde(default)]
    pub(super) rules: Vec<EntryConfig>,
}

/// Effective project policy with the exact built-in sets applied by the producer.
/// Configuration field semantics share the TOML parser's types; no current config is loaded when reading this receipt.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct EffectiveConfig {
    #[serde(flatten)]
    pub(super) configuration: ConfigFile,
    /// Ordered conventions applied by this measurement, not today's compiled-in set.
    pub(super) builtin_conventions: Vec<EntryConfig>,
    /// Ordered mappings applied after user/project mappings.
    pub(super) builtin_language_mappings: Vec<MappingConfig>,
    /// Generated-code markers applied by this measurement; reports of earlier producers lack the field.
    #[serde(default)]
    pub(super) builtin_generated_markers: Vec<String>,
}

impl EffectiveConfig {
    pub(crate) fn validate(&self) -> Result<()> {
        let config = &self.configuration;
        ensure!(
            config.version == 1,
            "Unsupported effective configuration version {}",
            config.version
        );
        check_entry_ids(config)?;
        for correction in &config.languages.corrections {
            LanguageCorrection::from_config(correction)?;
        }
        check_ids(
            config
                .languages
                .mappings
                .iter()
                .map(|mapping| mapping.id.as_str()),
        )?;
        check_categories(config)?;
        for entry in config
            .overrides
            .iter()
            .chain(&config.rules)
            .chain(&self.builtin_conventions)
        {
            ensure!(
                !entry.patterns.is_empty(),
                "Effective classification entry needs match patterns"
            );
            ensure!(
                entry.kind.as_ref().is_none_or(|kinds| !kinds.is_empty()),
                "Effective classification kind list must not be empty"
            );
            glob_set(entry.patterns.iter().map(String::as_str))?;
        }
        for mapping in config
            .languages
            .mappings
            .iter()
            .chain(&self.builtin_language_mappings)
        {
            ensure!(
                !mapping.patterns.is_empty(),
                "Effective language mapping needs match patterns"
            );
            ensure!(
                LanguageType::from_name(&mapping.language).is_some(),
                "Unknown effective mapping language {}",
                mapping.language
            );
            glob_set(mapping.patterns.iter().map(String::as_str))?;
        }
        Ok(())
    }
}
