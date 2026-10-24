//! What each project entry decided in one measurement, reported as
//! `summary.policy_entries` (docs/proposals/archive/policy-entry-review.md).
//! `unused_entries` flags only an entry that decides nothing; these counts
//! also expose an entry that decides fewer files than intended, loses files
//! to an earlier stage, or replaces another stage's decision.
use super::{Classification, Entry, Policy, Purpose};
use crate::FileRecord;
use globset::GlobSet;
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
    fn new(patterns: &[String], exceptions: &[String]) -> Self {
        Self {
            patterns: vec![false; patterns.len()],
            exceptions: vec![false; exceptions.len()],
            ..Self::default()
        }
    }

    /// Records the `except` globs that leave out a file the entry matches,
    /// and whether any did.
    fn excepted(&mut self, except: &GlobSet, path: &str) -> bool {
        let excepted = except.matches(path);
        for &i in &excepted {
            self.exceptions[i] = true;
        }
        !excepted.is_empty()
    }

    /// Records a file the entry decided, with the decision it replaced.
    fn decided(
        &mut self,
        matcher: &GlobSet,
        path: &str,
        code: u64,
        replaced: Option<Classification>,
    ) {
        self.files += 1;
        self.code += code;
        for i in matcher.matches(path) {
            self.patterns[i] = true;
        }
        if let Some(replaced) = replaced {
            let counts = self
                .displaced
                .entry(format!("{}:{}", replaced.purpose, replaced.category))
                .or_default();
            counts.0 += 1;
            counts.1 += code;
        }
    }

    fn preempted_by(&mut self, by: &str) {
        *self.preempted.entry(by.to_owned()).or_default() += 1;
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
        let classifying: Vec<(&Entry, EntryStage)> = self
            .overrides
            .iter()
            .map(|entry| (entry, EntryStage::Override))
            .chain(self.rules.iter().map(|entry| (entry, EntryStage::Rule)))
            .collect();
        let mut tallies = tallies(classifying.iter().map(|(entry, _)| *entry));
        for row in rows {
            let (path, kind) = (row.path.as_str(), row.kind);
            for ((entry, _), tally) in classifying.iter().zip(&mut tallies) {
                if !entry.has_kind(kind)
                    || !entry.matcher.is_match(path)
                    || tally.excepted(&entry.except, path)
                {
                    continue;
                }
                if entry.id == row.classification.rule {
                    let replaced = self.decision_without(path, kind, &entry.id);
                    tally.decided(&entry.matcher, path, row.counts.code, replaced);
                } else {
                    tally.preempted_by(&row.classification.rule);
                }
            }
        }
        let mut entries: Vec<PolicyEntry> = tallies
            .into_iter()
            .zip(&classifying)
            .map(|(tally, (entry, stage))| finish(tally, entry, *stage, entry.purpose))
            .collect();
        entries.extend(self.refinement_entries(rows));
        entries.extend(self.correction_entries(rows, corrections));
        entries
    }

    fn refinement_entries(&self, rows: &[FileRecord]) -> Vec<PolicyEntry> {
        let mut tallies = tallies(&self.refinements);
        for row in rows.iter().filter(|row| !row.classification.is_fallback()) {
            let (path, kind) = (row.path.as_str(), row.kind);
            for (refinement, tally) in self.refinements.iter().zip(&mut tallies) {
                if refinement.purpose != row.classification.purpose
                    || !refinement.has_kind(kind)
                    || !refinement.matcher.is_match(path)
                    || tally.excepted(&refinement.except, path)
                {
                    continue;
                }
                if row.refinement.as_ref() == Some(&refinement.id) {
                    let replaced = self.classify(path, kind);
                    tally.decided(&refinement.matcher, path, row.counts.code, Some(replaced));
                } else {
                    // An earlier refinement, or a generated-code marker that
                    // refinements leave alone.
                    tally.preempted_by(row.refinement.as_ref().unwrap_or(&row.classification.rule));
                }
            }
        }
        tallies
            .into_iter()
            .zip(&self.refinements)
            .map(|(tally, refinement)| {
                finish(
                    tally,
                    refinement,
                    EntryStage::Refinement,
                    refinement.purpose,
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
        let mut tallies: Vec<Tally> = self
            .corrections
            .iter()
            .map(|correction| Tally::new(&correction.patterns, &[]))
            .collect();
        for (path, decider) in corrections {
            for (correction, tally) in self.corrections.iter().zip(&mut tallies) {
                if !correction.matcher.is_match(path) {
                    continue;
                }
                if &correction.id == decider {
                    let code = code.get(path.as_str()).copied().unwrap_or_default();
                    tally.decided(&correction.matcher, path, code, None);
                } else {
                    tally.preempted_by(decider);
                }
            }
        }
        tallies
            .into_iter()
            .zip(&self.corrections)
            .map(|(tally, correction)| {
                tally.entry(
                    &correction.id,
                    EntryStage::Correction,
                    None,
                    &correction.patterns,
                    &[],
                )
            })
            .collect()
    }
}

fn tallies<'a>(entries: impl IntoIterator<Item = &'a Entry>) -> Vec<Tally> {
    entries
        .into_iter()
        .map(|entry| Tally::new(&entry.patterns, &entry.exceptions))
        .collect()
}

fn finish(tally: Tally, entry: &Entry, stage: EntryStage, purpose: Purpose) -> PolicyEntry {
    tally.entry(
        &entry.id,
        stage,
        Some(purpose),
        &entry.patterns,
        &entry.exceptions,
    )
}

/// IDs of the entries that decided no file: a glob that matches nothing, or
/// an entry an earlier stage always preempts, such as a rule behind a test
/// convention. A correction that decides no selected file contradicts the
/// counter for nothing, so it is listed with them.
pub fn unused_entries(entries: &[PolicyEntry]) -> Vec<String> {
    entries
        .iter()
        .filter(|entry| entry.files == 0)
        .map(|entry| entry.id.clone())
        .collect()
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
