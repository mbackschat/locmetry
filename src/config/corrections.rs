//! Language corrections: project entries that replace the counter's language
//! for files it recognized wrongly (docs/PRODUCT.md, `languages.corrections`).
//! They exist only in the project configuration, because which language a
//! file is written in is knowledge about one project.
use super::{glob_set, schema};
use anyhow::{Context, Result};
use globset::GlobSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokei::LanguageType;

/// The reserved `language` that leaves a corrected file selected but uncounted.
pub const UNRECOGNIZED: &str = "unrecognized";

/// Replaces the counter's language for matching files.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(rename = "LanguageCorrection")]
pub(super) struct CorrectionConfig {
    /// Unique among overrides, rules and corrections; reports name it for files it decided, and list it in `summary.unused_entries` when it decides none.
    #[schemars(length(min = 1))]
    pub(super) id: String,
    /// Root-relative globs; the correction matches a file when any of them matches.
    #[serde(rename = "match")]
    #[schemars(length(min = 1), extend("items" = {"type": "string"}))]
    pub(super) patterns: Vec<String>,
    /// A counter language, whose parser and content kind then apply, or `unrecognized`, which leaves the file selected, uncounted and listed in `metadata.coverage.unrecognized`; use it when no counter language shares the file's comment syntax.
    #[schemars(schema_with = "schema::correction_languages")]
    pub(super) language: String,
    /// Why the counter's language is wrong, for reviewers; `explain` shows it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(super) reason: Option<String>,
}

/// What a correction makes of a file.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CorrectedLanguage {
    Counted(LanguageType),
    Unrecognized,
}

#[derive(Debug)]
pub(super) struct LanguageCorrection {
    pub(super) id: String,
    pub(super) matcher: GlobSet,
    /// The globs `matcher` compiles, by its match indices.
    pub(super) patterns: Vec<String>,
    pub(super) language: CorrectedLanguage,
    pub(super) reason: Option<String>,
}

impl CorrectedLanguage {
    pub(super) fn parse(name: &str) -> Option<Self> {
        match name {
            UNRECOGNIZED => Some(Self::Unrecognized),
            name => LanguageType::from_name(name).map(Self::Counted),
        }
    }
}

impl LanguageCorrection {
    pub(super) fn from_config(config: &CorrectionConfig) -> Result<Self> {
        anyhow::ensure!(
            !config.patterns.is_empty(),
            "Language correction {} needs a nonempty match array",
            config.id
        );
        Ok(Self {
            id: config.id.clone(),
            matcher: glob_set(config.patterns.iter().map(String::as_str))
                .with_context(|| format!("Invalid match in language correction '{}'", config.id))?,
            patterns: config.patterns.clone(),
            language: CorrectedLanguage::parse(&config.language).with_context(|| {
                format!(
                    "Unknown language '{}' in language correction '{}'; name a counter language or '{UNRECOGNIZED}'",
                    config.language, config.id
                )
            })?,
            reason: config.reason.clone(),
        })
    }
}
