//! Classification entries: overrides, rules and refinements compiled from
//! the configuration, and the checks every entry list must pass.
use super::*;
use crate::output::purpose_title;

#[derive(Debug)]
pub(super) struct Entry {
    pub(super) id: String,
    pub(super) matcher: GlobSet,
    /// The globs `matcher` compiles, by its match indices.
    pub(super) patterns: Vec<String>,
    /// Paths the entry leaves out although `matcher` matches them.
    pub(super) except: GlobSet,
    /// The globs `except` compiles, by its match indices.
    pub(super) exceptions: Vec<String>,
    pub(super) kinds: Option<Vec<Kind>>,
    pub(super) purpose: Purpose,
    pub(super) category: String,
    pub(super) reason: Option<String>,
}

impl Entry {
    pub(super) fn matches(&self, path: &str, kind: Kind) -> bool {
        self.matcher.is_match(path) && !self.except.is_match(path) && self.has_kind(kind)
    }

    /// The entry's `match` globs that match `path`.
    pub(super) fn matched_patterns(&self, path: &str) -> Vec<String> {
        self.matcher
            .matches(path)
            .into_iter()
            .map(|i| self.patterns[i].clone())
            .collect()
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
            entry.reason.clone(),
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
            refinement.reason.clone(),
        )
    }

    pub(super) fn compile(
        id: &str,
        patterns: &[String],
        except: Option<&[String]>,
        kinds: Option<Vec<Kind>>,
        purpose: Purpose,
        category: String,
        reason: Option<String>,
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
                .with_context(|| format!("In 'match' of entry '{id}'"))?,
            patterns: patterns.to_vec(),
            except: glob_set(except.into_iter().flatten().map(String::as_str))
                .with_context(|| format!("In 'except' of entry '{id}'"))?,
            exceptions: except.unwrap_or_default().to_vec(),
            kinds,
            purpose,
            category,
            reason,
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
        .filter_map(|entry| Some((&entry.id, entry.purpose, entry.category.as_ref()?)))
        .chain(
            config
                .refinements
                .iter()
                .map(|refinement| (&refinement.id, refinement.purpose, &refinement.category)),
        );
    for (id, purpose, category) in categories {
        if config.categories.contains_key(category) || builtin.contains(category) {
            continue;
        }
        let reference = reference_use(purpose, category).map_or(String::new(), |use_for| {
            let (first, rest) = use_for.split_at(use_for.chars().next().map_or(0, char::len_utf8));
            format!(
                " The reference in '--guide categories' uses '{category}' in {} for {}{}. Declare it in [categories] to use it.",
                purpose_title(purpose),
                first.to_lowercase(),
                rest
            )
        });
        bail!(
            "Entry '{id}' uses category '{category}', which [categories] does not declare.{reference} Declared: {}. Built in: {}.",
            config.categories.keys().cloned().collect::<Vec<_>>().join(", "),
            builtin.into_iter().collect::<Vec<_>>().join(", ")
        );
    }
    Ok(())
}

/// The category reference, whose vocabulary table owns the reference names;
/// the binary embeds the same file as `--guide categories`.
const CATEGORY_GUIDE: &str = include_str!("../../docs/guides/categories.md");

/// What the reference uses `name` for under `purpose`, read from the guide's
/// marked vocabulary table, which tests/integration/categories.rs also parses.
fn reference_use(purpose: Purpose, name: &str) -> Option<&'static str> {
    let (_, table) = CATEGORY_GUIDE.split_once("<!-- category-vocabulary -->\n")?;
    let (purpose, name) = (purpose.to_string(), format!("`{name}`"));
    table
        .lines()
        .take_while(|line| line.starts_with('|'))
        .skip(2)
        .find_map(|row| {
            let cells: Vec<_> = row.trim_matches('|').split('|').map(str::trim).collect();
            (cells.len() > 2 && cells[0] == purpose && cells[1] == name).then(|| cells[2])
        })
}

/// Rejects empty, duplicate, and reserved entry IDs across all stages.
/// Overrides, rules, refinements and the project's language mappings and
/// corrections share one ID space, because `summary.unused_entries` lists them
/// together; personal mappings stay out of that list and this check.
pub(super) fn check_entry_ids(config: &ConfigFile, project_mappings: usize) -> Result<()> {
    check_ids(
        config
            .overrides
            .iter()
            .chain(&config.rules)
            .map(|entry| entry.id.as_str())
            .chain(config.refinements.iter().map(|r| r.id.as_str()))
            .chain(
                config.languages.mappings[..project_mappings]
                    .iter()
                    .map(|m| m.id.as_str()),
            )
            .chain(config.languages.corrections.iter().map(|c| c.id.as_str())),
    )
}

pub(super) fn check_ids<'a>(ids: impl IntoIterator<Item = &'a str>) -> Result<()> {
    let mut seen = BTreeSet::new();
    for id in ids {
        ensure!(!id.is_empty(), "An entry has an empty ID.");
        ensure!(
            seen.insert(id),
            "Entry ID '{id}' is used more than once. IDs are unique across overrides, rules, refinements and language mappings and corrections."
        );
        ensure!(
            id != FALLBACK && !id.starts_with(BUILTIN),
            "Entry ID '{id}' is reserved. '{FALLBACK}' and IDs starting with '{BUILTIN}' name Locmetry's own stages."
        );
    }
    Ok(())
}
