//! What each project entry decided in one measurement, reported as
//! `summary.policy_entries` (docs/proposals/policy-entry-review.md).
//! `unused_entries` flags only an entry that decides nothing; these counts
//! also expose an entry that decides fewer files than intended, loses files
//! to an earlier stage, or replaces another stage's decision.
use super::{Entry, Policy, Purpose};
use crate::FileRecord;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};

/// The list of the configuration an entry belongs to.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EntryStage {
    Override,
    Rule,
    Refinement,
    Correction,
}

/// What one project entry decided among the measured files.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PolicyEntry {
    pub id: String,
    pub stage: EntryStage,
    /// The purpose an override or rule assigns, or the decided purpose a refinement applies to; absent for language corrections, which decide a language.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<Purpose>,
    /// Files the entry decided, of every kind. A correction to `unrecognized` decides files that count no lines.
    pub files: u64,
    pub code: u64,
    /// Files the entry matches that an earlier entry, built-in convention, refinement or generated-code marker decided, by its ID, largest first.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub preempted: Vec<Preemption>,
    /// The entry's `match` globs that decided no file.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unused_patterns: Vec<String>,
    /// The entry's `except` globs that left out no file the entry otherwise matches.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unused_exceptions: Vec<String>,
    /// The decided files by the `purpose:category` that later stages would give them without the entry, before generated-code markers, largest code first; for a refinement, the decision it recategorized. Files that would reach the fallback are left out: claiming unclaimed code replaces no decision.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub displaced: Vec<Displacement>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Preemption {
    /// The deciding entry ID or `builtin:<convention>`.
    pub by: String,
    pub files: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Displacement {
    /// The replaced `purpose:category`.
    pub decision: String,
    pub files: u64,
    pub code: u64,
}

impl PolicyEntry {
    /// Code moved to this entry's purpose from each other purpose, in
    /// `Purpose::ALL` order; empty when every replaced decision kept it.
    pub fn purpose_changes(&self) -> Vec<(Purpose, u64)> {
        let mut moved = BTreeMap::new();
        for displaced in &self.displaced {
            let from = displaced.decision.split(':').next().and_then(|name| {
                Purpose::ALL
                    .into_iter()
                    .find(|purpose| purpose.to_string() == name)
            });
            if let Some(from) = from.filter(|from| Some(*from) != self.purpose) {
                *moved.entry(from).or_default() += displaced.code;
            }
        }
        moved.into_iter().collect()
    }
}

/// Counts of one entry while files are audited.
#[derive(Default)]
struct Tally {
    files: u64,
    code: u64,
    preempted: BTreeMap<String, u64>,
    patterns: Vec<bool>,
    exceptions: Vec<bool>,
    displaced: BTreeMap<String, (u64, u64)>,
}

impl Tally {
    fn new(patterns: usize, exceptions: usize) -> Self {
        Self {
            patterns: vec![false; patterns],
            exceptions: vec![false; exceptions],
            ..Self::default()
        }
    }

    fn entry(
        self,
        id: &str,
        stage: EntryStage,
        purpose: Option<Purpose>,
        patterns: &[String],
        exceptions: &[String],
    ) -> PolicyEntry {
        let unused = |used: &[bool], globs: &[String]| {
            used.iter()
                .zip(globs)
                .filter(|(used, _)| !**used)
                .map(|(_, glob)| glob.clone())
                .collect()
        };
        PolicyEntry {
            id: id.into(),
            stage,
            purpose,
            files: self.files,
            code: self.code,
            preempted: preemptions(self.preempted),
            unused_patterns: unused(&self.patterns, patterns),
            unused_exceptions: unused(&self.exceptions, exceptions),
            displaced: displacements(self.displaced),
        }
    }
}

/// Preempted files by deciding rule, largest first.
fn preemptions(by_rule: BTreeMap<String, u64>) -> Vec<Preemption> {
    let mut preempted: Vec<_> = by_rule
        .into_iter()
        .map(|(by, files)| Preemption { by, files })
        .collect();
    preempted.sort_by(|a, b| b.files.cmp(&a.files).then(a.by.cmp(&b.by)));
    preempted
}

/// Replaced decisions with their files and code, largest code first.
fn displacements(by_decision: BTreeMap<String, (u64, u64)>) -> Vec<Displacement> {
    let mut displaced: Vec<_> = by_decision
        .into_iter()
        .map(|(decision, (files, code))| Displacement {
            decision,
            files,
            code,
        })
        .collect();
    displaced.sort_by(|a, b| b.code.cmp(&a.code).then(a.decision.cmp(&b.decision)));
    displaced
}

impl Policy {
    /// The project's overrides, rules, refinements and language corrections,
    /// in that order, with what each decided among `rows`; `corrections` maps every
    /// selected file a correction decided, counted or not, to its ID.
    pub fn policy_entries(
        &self,
        rows: &[FileRecord],
        corrections: &BTreeMap<String, String>,
    ) -> Vec<PolicyEntry> {
        let configuration = &self.snapshot.configuration;
        let configs = configuration.overrides.iter().chain(&configuration.rules);
        let classifying: Vec<(&Entry, EntryStage)> = self
            .overrides
            .iter()
            .map(|entry| (entry, EntryStage::Override))
            .chain(self.rules.iter().map(|entry| (entry, EntryStage::Rule)))
            .collect();
        let mut tallies: Vec<Tally> = configs
            .clone()
            .map(|config| {
                Tally::new(
                    config.patterns.len(),
                    config.except.iter().flatten().count(),
                )
            })
            .collect();
        for row in rows {
            let (path, kind) = (row.path.as_str(), row.kind);
            for ((entry, _), tally) in classifying.iter().zip(&mut tallies) {
                if !entry.has_kind(kind) || !entry.matcher.is_match(path) {
                    continue;
                }
                let excepted = entry.except.matches(path);
                if !excepted.is_empty() {
                    excepted
                        .into_iter()
                        .for_each(|i| tally.exceptions[i] = true);
                } else if entry.id == row.classification.rule {
                    tally.files += 1;
                    tally.code += row.counts.code;
                    for i in entry.matcher.matches(path) {
                        tally.patterns[i] = true;
                    }
                    if let Some(replaced) = self.decision_without(path, kind, &entry.id) {
                        let decision = format!("{}:{}", replaced.purpose, replaced.category);
                        let counts = tally.displaced.entry(decision).or_default();
                        counts.0 += 1;
                        counts.1 += row.counts.code;
                    }
                } else {
                    *tally
                        .preempted
                        .entry(row.classification.rule.clone())
                        .or_default() += 1;
                }
            }
        }
        let mut entries: Vec<PolicyEntry> = tallies
            .into_iter()
            .zip(configs)
            .zip(&classifying)
            .map(|((tally, config), (entry, stage))| {
                tally.entry(
                    &config.id,
                    *stage,
                    Some(entry.purpose),
                    &config.patterns,
                    config.except.as_deref().unwrap_or_default(),
                )
            })
            .collect();
        entries.extend(self.refinement_entries(rows));
        entries.extend(self.correction_entries(rows, corrections));
        entries
    }

    fn refinement_entries(&self, rows: &[FileRecord]) -> Vec<PolicyEntry> {
        let configs = &self.snapshot.configuration.refinements;
        let mut tallies: Vec<Tally> = configs
            .iter()
            .map(|config| {
                Tally::new(
                    config.patterns.len(),
                    config.except.iter().flatten().count(),
                )
            })
            .collect();
        for row in rows.iter().filter(|row| !row.classification.is_fallback()) {
            let (path, kind) = (row.path.as_str(), row.kind);
            for (refinement, tally) in self.refinements.iter().zip(&mut tallies) {
                if refinement.purpose != row.classification.purpose
                    || !refinement.has_kind(kind)
                    || !refinement.matcher.is_match(path)
                {
                    continue;
                }
                let excepted = refinement.except.matches(path);
                if !excepted.is_empty() {
                    excepted
                        .into_iter()
                        .for_each(|i| tally.exceptions[i] = true);
                } else if row.refinement.as_ref() == Some(&refinement.id) {
                    tally.files += 1;
                    tally.code += row.counts.code;
                    for i in refinement.matcher.matches(path) {
                        tally.patterns[i] = true;
                    }
                    let decided = self.classify(path, kind);
                    let decision = format!("{}:{}", decided.purpose, decided.category);
                    let counts = tally.displaced.entry(decision).or_default();
                    counts.0 += 1;
                    counts.1 += row.counts.code;
                } else {
                    // An earlier refinement, or a generated-code marker that
                    // refinements leave alone.
                    let by = row.refinement.as_ref().unwrap_or(&row.classification.rule);
                    *tally.preempted.entry(by.clone()).or_default() += 1;
                }
            }
        }
        tallies
            .into_iter()
            .zip(configs)
            .map(|(tally, config)| {
                tally.entry(
                    &config.id,
                    EntryStage::Refinement,
                    Some(config.purpose),
                    &config.patterns,
                    config.except.as_deref().unwrap_or_default(),
                )
            })
            .collect()
    }

    fn correction_entries(
        &self,
        rows: &[FileRecord],
        corrections: &BTreeMap<String, String>,
    ) -> Vec<PolicyEntry> {
        let code: HashMap<&str, u64> = rows
            .iter()
            .filter(|row| row.language_correction.is_some())
            .map(|row| (row.path.as_str(), row.counts.code))
            .collect();
        let configs = &self.snapshot.configuration.languages.corrections;
        let mut tallies: Vec<Tally> = configs
            .iter()
            .map(|config| Tally::new(config.patterns.len(), 0))
            .collect();
        for (path, decider) in corrections {
            for (correction, tally) in self.corrections.iter().zip(&mut tallies) {
                if !correction.matcher.is_match(path) {
                    continue;
                }
                if &correction.id == decider {
                    tally.files += 1;
                    tally.code += code.get(path.as_str()).copied().unwrap_or_default();
                    for i in correction.matcher.matches(path) {
                        tally.patterns[i] = true;
                    }
                } else {
                    *tally.preempted.entry(decider.clone()).or_default() += 1;
                }
            }
        }
        tallies
            .into_iter()
            .zip(configs)
            .map(|(tally, config)| {
                tally.entry(
                    &config.id,
                    EntryStage::Correction,
                    None,
                    &config.patterns,
                    &[],
                )
            })
            .collect()
    }
}

/// The entries of one policy measured in several folders, as one list: counts
/// add up, and a glob is unused only when it is unused in every folder.
pub fn merged_policy_entries(lists: Vec<Vec<PolicyEntry>>) -> Vec<PolicyEntry> {
    let mut lists = lists.into_iter();
    let first = lists.next().unwrap_or_default();
    lists.fold(first, |merged, list| {
        merged
            .into_iter()
            .zip(list)
            .map(|(mut entry, other)| {
                entry.files += other.files;
                entry.code += other.code;
                let mut preempted = BTreeMap::new();
                for item in entry.preempted.into_iter().chain(other.preempted) {
                    *preempted.entry(item.by).or_default() += item.files;
                }
                entry.preempted = preemptions(preempted);
                let mut displaced: BTreeMap<String, (u64, u64)> = BTreeMap::new();
                for item in entry.displaced.into_iter().chain(other.displaced) {
                    let counts = displaced.entry(item.decision).or_default();
                    counts.0 += item.files;
                    counts.1 += item.code;
                }
                entry.displaced = displacements(displaced);
                entry
                    .unused_patterns
                    .retain(|glob| other.unused_patterns.contains(glob));
                entry
                    .unused_exceptions
                    .retain(|glob| other.unused_exceptions.contains(glob));
                entry
            })
            .collect()
    })
}
