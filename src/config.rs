use anyhow::{ensure, Context, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{collections::BTreeSet, fmt, path::PathBuf};

/// Provenance of files that no rule matched.
const FALLBACK: &str = "fallback";

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "lowercase")]
pub enum Purpose {
    Production,
    Test,
    Unclassified,
}

impl Purpose {
    pub const ALL: [Self; 3] = [Self::Production, Self::Test, Self::Unclassified];
}

impl fmt::Display for Purpose {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Production => "production",
            Self::Test => "test",
            Self::Unclassified => "unclassified",
        })
    }
}

#[derive(Debug, Serialize)]
pub struct Classification {
    pub purpose: Purpose,
    pub category: String,
    pub rule: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Rule {
    id: String,
    pattern: String,
    purpose: Purpose,
    category: Option<String>,
    reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct ProfileData {
    project: String,
    root: PathBuf,
    #[serde(default)]
    exclude_prefixes: Vec<String>,
    rules: Vec<Rule>,
    #[serde(default)]
    notes: Vec<String>,
    #[serde(default)]
    examples: Vec<Value>,
}

#[derive(Debug)]
pub struct Profile {
    data: ProfileData,
    patterns: Vec<Regex>,
    prefixes: Vec<String>,
}

impl Profile {
    pub fn from_json(input: &str) -> Result<Self> {
        let data: ProfileData = serde_json::from_str(input).context("Invalid JSON profile")?;
        let mut ids = BTreeSet::new();
        let patterns = data
            .rules
            .iter()
            .map(|rule| {
                ensure!(
                    !rule.id.is_empty() && ids.insert(&rule.id),
                    "Rule IDs must be nonempty and unique"
                );
                ensure!(
                    rule.id != FALLBACK,
                    "Rule ID {FALLBACK} is reserved for unmatched files"
                );
                Regex::new(&rule.pattern)
                    .with_context(|| format!("Invalid regex in rule {}", rule.id))
            })
            .collect::<Result<Vec<_>>>()?;
        let prefixes = data
            .exclude_prefixes
            .iter()
            .map(|prefix| {
                let prefix = prefix.trim_end_matches('/');
                ensure!(
                    !prefix.is_empty()
                        && !prefix.contains(['?', '*', '[', ']', '\\', '!', '{', '}', ':'])
                        && prefix
                            .split('/')
                            .all(|part| !matches!(part, "" | "." | "..")),
                    "Exclusion must be a literal root-relative directory prefix: {prefix}"
                );
                Ok(prefix.to_owned())
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(Self {
            data,
            patterns,
            prefixes,
        })
    }

    pub fn classify(&self, path: &str) -> Classification {
        for (rule, pattern) in self.data.rules.iter().zip(&self.patterns) {
            if pattern.is_match(path) {
                return Classification {
                    purpose: rule.purpose,
                    category: rule
                        .category
                        .clone()
                        .unwrap_or_else(|| rule.purpose.to_string()),
                    rule: rule.id.clone(),
                };
            }
        }
        Classification {
            purpose: Purpose::Unclassified,
            category: "unmapped".into(),
            rule: FALLBACK.into(),
        }
    }

    pub fn is_excluded(&self, path: &str) -> bool {
        self.prefixes.iter().any(|prefix| {
            path.strip_prefix(prefix)
                .is_some_and(|rest| rest.starts_with('/'))
        })
    }

    pub fn project(&self) -> &str {
        &self.data.project
    }
    pub fn root(&self) -> &std::path::Path {
        &self.data.root
    }
    pub fn prefixes(&self) -> &[String] {
        &self.prefixes
    }
    pub fn snapshot(&self) -> Result<Value> {
        Ok(serde_json::to_value(&self.data)?)
    }
}
