//! Classification entries: overrides, rules and refinements compiled from
//! the configuration, and the checks every entry list must pass.
use super::*;

#[derive(Debug)]
pub(super) struct Entry {
    pub(super) id: String,
    pub(super) matcher: GlobSet,
    /// Paths the entry leaves out although `matcher` matches them.
    pub(super) except: GlobSet,
    pub(super) kinds: Option<Vec<Kind>>,
    pub(super) purpose: Purpose,
    pub(super) category: String,
}

impl Entry {
    pub(super) fn matches(&self, path: &str, kind: Kind) -> bool {
        self.matcher.is_match(path) && !self.except.is_match(path) && self.has_kind(kind)
    }

    pub(super) fn has_kind(&self, kind: Kind) -> bool {
        self.kinds
            .as_ref()
            .is_none_or(|kinds| kinds.contains(&kind))
    }

    pub(super) fn classification(&self) -> Classification {
        Classification {
            purpose: self.purpose,
            category: self.category.clone(),
            rule: self.id.clone(),
        }
    }

    pub(super) fn from_config(entry: &EntryConfig) -> Result<Self> {
        Self::compile(
            &entry.id,
            &entry.patterns,
            entry.except.as_deref(),
            entry.kind.clone(),
            entry.purpose,
            entry
                .category
                .clone()
                .unwrap_or_else(|| entry.purpose.to_string()),
        )
    }

    /// A refinement compiles like an entry, with `purpose` as the decided
    /// purpose it applies to rather than the one it assigns.
    pub(super) fn from_refinement(refinement: &RefinementConfig) -> Result<Self> {
        Self::compile(
            &refinement.id,
            &refinement.patterns,
            refinement.except.as_deref(),
            refinement.kind.clone(),
            refinement.purpose,
            refinement.category.clone(),
        )
    }

    pub(super) fn compile(
        id: &str,
        patterns: &[String],
        except: Option<&[String]>,
        kinds: Option<Vec<Kind>>,
        purpose: Purpose,
        category: String,
    ) -> Result<Self> {
        ensure!(
            !patterns.is_empty(),
            "Entry {id} needs a nonempty match array"
        );
        ensure!(
            kinds.as_ref().is_none_or(|kinds| !kinds.is_empty()),
            "Entry {id} needs a nonempty kind array when kind is present"
        );
        ensure!(
            except.is_none_or(|except| !except.is_empty()),
            "Entry {id} needs a nonempty except array when except is present"
        );
        Ok(Self {
            id: id.into(),
            matcher: glob_set(patterns.iter().map(String::as_str))
                .with_context(|| format!("Invalid match in entry {id}"))?,
            except: glob_set(except.into_iter().flatten().map(String::as_str))
                .with_context(|| format!("Invalid except in entry {id}"))?,
            kinds,
            purpose,
            category,
        })
    }
}

/// Rejects entry categories that a declared `[categories]` list does not
/// name; without a list, categories stay free-form.
pub(super) fn check_categories(config: &ConfigFile) -> Result<()> {
    if config.categories.is_empty() {
        return Ok(());
    }
    let builtin: BTreeSet<String> = Purpose::ALL
        .iter()
        .map(Purpose::to_string)
        .chain([MODULE_CATEGORY, GENERATED_CATEGORY].map(String::from))
        .chain(
            BUILTIN_TESTS_V1
                .iter()
                .chain(BUILTIN_BUILD_V1)
                .map(|convention| convention.category.to_string()),
        )
        .collect();
    let categories = config
        .overrides
        .iter()
        .chain(&config.rules)
        .filter_map(|entry| Some((&entry.id, entry.category.as_ref()?)))
        .chain(
            config
                .refinements
                .iter()
                .map(|refinement| (&refinement.id, &refinement.category)),
        );
    for (id, category) in categories {
        ensure!(
            config.categories.contains_key(category) || builtin.contains(category),
            "Entry '{id}' uses category '{category}', which [categories] does not declare. Declared: {}. Built in: {}.",
            config.categories.keys().cloned().collect::<Vec<_>>().join(", "),
            builtin.into_iter().collect::<Vec<_>>().join(", ")
        );
    }
    Ok(())
}

/// Rejects empty, duplicate, and reserved entry IDs across all stages.
/// Overrides, rules, refinements and language corrections share one ID
/// space, because `summary.unused_entries` lists them together.
pub(super) fn check_entry_ids(config: &ConfigFile) -> Result<()> {
    check_ids(
        config
            .overrides
            .iter()
            .chain(&config.rules)
            .map(|entry| entry.id.as_str())
            .chain(config.refinements.iter().map(|r| r.id.as_str()))
            .chain(config.languages.corrections.iter().map(|c| c.id.as_str())),
    )
}

pub(super) fn check_ids<'a>(ids: impl IntoIterator<Item = &'a str>) -> Result<()> {
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
