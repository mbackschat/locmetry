//! Focused evidence for one existing path, using ordinary selection and counting.
use crate::{backend, contract, Policy, ScopeOverrides};
use anyhow::{ensure, Context, Result};
use schemars::JsonSchema;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Explanation,
}

/// One path's selection, parser and classification evidence. Exit 2 means the target's counts are unavailable; exit 0 includes excluded, unrecognized and filtered paths. This is not a saved measurement input for compare.
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(example = example())]
pub struct Explanation {
    pub document_type: DocumentType,
    pub schema_version: contract::Version,
    /// Canonical scan root; the selected member's root in workspace mode.
    pub root: String,
    /// Root-relative path, prefixed with member name in workspace mode.
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<String>,
    /// Loaded project configuration, or null for built-in settings.
    pub config_path: Option<String>,
    /// Loaded personal defaults, or null when absent or disabled.
    pub user_config_path: Option<String>,
    /// Effective selection; exclusion patterns remain relative to the member's own root.
    pub scope: contract::Selection,
    pub result: PathResult,
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
    pub module: Option<contract::Module>,
}

#[derive(Serialize, JsonSchema)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub enum PathResult {
    /// Successfully counted, including known zero. Parser and optional mapping ID are in file.
    Counted {
        file: Box<contract::File>,
        decision: Decision,
    },
    /// The ordinary walker rejected this file or pruned its ancestor with the listed scope globs.
    Excluded { at: String, patterns: Vec<String> },
    /// The ordinary walker did not select the file. No winning ignore filename or line is claimed.
    NotSelected { reason: FilterReason },
    /// Selected, but neither Tokei nor a fallback mapping recognized a language.
    Unrecognized,
    /// Selected and recognized but unreadable; no known-zero count is supplied.
    Unread,
    /// Selected but the structured container could not supply counts.
    ParseFailed { failure: crate::ParseFailure },
    /// This path or an ancestor is a symlink. Measurement does not follow symlinks; no claim about other filters is made.
    Symlink { at: String },
}

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum FilterReason {
    /// Disabling only respect_ignore_files makes the target selected; the winning ignore rule is unavailable.
    IgnorePolicy,
    /// Enabling only hidden makes the target selected; no other setting was changed.
    HiddenPolicy,
    /// No target exclusion trace or single-setting probe established a cause, including when filters mask one another.
    ReasonNotRecorded,
}

pub fn inspect(root: &Path, path: &Path, policy: &mut Policy) -> Result<(String, PathResult)> {
    let root = root.canonicalize().context("Scan root does not exist")?;
    ensure!(root.is_dir(), "Scan root must be a directory");
    let root = root.as_path();
    let path = backend::relative_path(path)?;
    let mut current = root.to_owned();
    let mut relative = std::path::PathBuf::new();
    for component in Path::new(&path).components() {
        current.push(component);
        relative.push(component);
        let metadata = std::fs::symlink_metadata(&current)
            .context("Explanation target does not exist or cannot be inspected")?;
        if metadata.file_type().is_symlink() {
            return Ok((
                path,
                PathResult::Symlink {
                    at: backend::relative_path(&relative)?,
                },
            ));
        }
    }
    ensure!(current.is_file(), "Explanation target must be a file");
    let (scan, exclusion) = backend::scan_selection(root, policy, Some(&path))?;
    policy.adopt_modules(&scan.modules)?;
    let result = if let Some(file) = scan.records(policy)?.into_iter().next() {
        let decision = policy.decision(&path, file.kind);
        PathResult::Counted {
            file: Box::new(contract::File::from(&file)),
            decision,
        }
    } else if let Some(failure) = scan.coverage.parse_failures.into_iter().next() {
        PathResult::ParseFailed { failure }
    } else if !scan.coverage.unread.is_empty() {
        PathResult::Unread
    } else if !scan.coverage.unrecognized.is_empty() {
        PathResult::Unrecognized
    } else if let Some(exclusion) = exclusion {
        PathResult::Excluded {
            at: exclusion.at,
            patterns: exclusion.patterns,
        }
    } else {
        let selected_with = |overrides| -> Result<bool> {
            let scope = policy.scope().with_overrides(&overrides)?;
            Ok(backend::select_paths(root, &scope, None)?
                .paths
                .contains(&root.join(&path)))
        };
        let reason = if policy.scope().respect_ignore_files()
            && selected_with(ScopeOverrides {
                respect_ignore_files: Some(false),
                ..Default::default()
            })? {
            FilterReason::IgnorePolicy
        } else if !policy.scope().hidden()
            && selected_with(ScopeOverrides {
                hidden: Some(true),
                ..Default::default()
            })?
        {
            FilterReason::HiddenPolicy
        } else {
            FilterReason::ReasonNotRecorded
        };
        PathResult::NotSelected { reason }
    };
    Ok((path, result))
}

impl Explanation {
    pub fn partial(&self) -> bool {
        matches!(
            self.result,
            PathResult::Unread | PathResult::ParseFailed { .. }
        )
    }

    pub fn qualify(&mut self, member: String) {
        let prefix = |path: &str| {
            if path == "." {
                member.clone()
            } else {
                format!("{member}/{path}")
            }
        };
        self.path = prefix(&self.path);
        match &mut self.result {
            PathResult::Counted { file, decision } => {
                file.path = prefix(&file.path);
                if let Some(module) = &mut decision.module {
                    module.path = prefix(&module.path);
                }
            }
            PathResult::ParseFailed { failure } => failure.path = prefix(&failure.path),
            PathResult::Excluded { at, .. } | PathResult::Symlink { at } => *at = prefix(at),
            _ => {}
        }
        self.member = Some(member);
    }

    pub fn render(&self, markdown: bool) -> String {
        let mut rows = vec![("Path", self.path.clone()), ("Root", self.root.clone())];
        rows.push((
            "Project config",
            self.config_path
                .clone()
                .unwrap_or_else(|| "built-in settings".into()),
        ));
        rows.push((
            "Personal config",
            self.user_config_path
                .clone()
                .unwrap_or_else(|| "none".into()),
        ));
        match &self.result {
            PathResult::Counted { file, decision } => {
                rows.extend([
                    ("State", "counted".into()),
                    ("Language", file.language.clone()),
                    ("Kind", file.kind.to_string()),
                    ("Purpose", file.purpose.to_string()),
                    ("Category", file.category.clone()),
                    ("Rule", file.rule.clone()),
                    ("Stage", decision.stage.to_string()),
                    (
                        "Counts",
                        format!(
                            "{} code, {} comments, {} blanks, {} lines",
                            file.counts.code,
                            file.counts.comments,
                            file.counts.blanks,
                            file.counts.lines
                        ),
                    ),
                ]);
                if let Some(mapping) = &file.language_mapping {
                    rows.push(("Language mapping", mapping.clone()));
                }
                if !decision.matched_patterns.is_empty() {
                    rows.push(("Matching patterns", decision.matched_patterns.join(", ")));
                }
                if let Some(reason) = &decision.reason {
                    rows.push(("Reason", reason.clone()));
                }
                if let Some(module) = &decision.module {
                    rows.push(("Module", format!("{} ({})", module.path, module.detector)));
                }
            }
            PathResult::Excluded { at, patterns } => rows.extend([
                ("State", "excluded".into()),
                ("Excluded at", at.clone()),
                ("Patterns", patterns.join(", ")),
            ]),
            PathResult::NotSelected { reason } => {
                let detail = match reason {
                    FilterReason::IgnorePolicy => "ignore_policy: disabling ignore filtering selects this path; winning ignore rule not recorded",
                    FilterReason::HiddenPolicy => "hidden_policy: enabling hidden paths selects this path",
                    FilterReason::ReasonNotRecorded => "reason_not_recorded: available evidence does not establish a cause",
                };
                rows.extend([("State", "not_selected".into()), ("Reason", detail.into())]);
            }
            PathResult::Unrecognized => {
                rows.push(("State", "unrecognized: no parser or mapping".into()));
            }
            PathResult::Unread => {
                rows.push(("State", "unread: counts unavailable".into()));
            }
            PathResult::ParseFailed { failure } => rows.extend([
                ("State", "parse_failed: counts unavailable".into()),
                ("Reason", format!("{}: {}", failure.reason, failure.message)),
            ]),
            PathResult::Symlink { at } => rows.extend([
                ("State", "symlink: not followed".into()),
                ("Symlink at", at.clone()),
            ]),
        }
        let mut text = if markdown {
            "# Path explanation\n\n| Evidence | Value |\n| --- | --- |\n"
        } else {
            "Path explanation\n"
        }
        .to_owned();
        for (key, value) in rows {
            let value = value.replace(['\r', '\n', '\t'], " ");
            if markdown {
                let value = value
                    .replace('&', "&amp;")
                    .replace('<', "&lt;")
                    .replace('>', "&gt;")
                    .replace('|', "&#124;")
                    .replace('`', "&#96;");
                text.push_str(&format!("| {key} | {value} |\n"));
            } else {
                text.push_str(&format!("{key}: {value}\n"));
            }
        }
        text
    }
}

pub fn schema() -> serde_json::Value {
    crate::schema::json::<Explanation>()
}

fn example() -> Explanation {
    Explanation {
        document_type: DocumentType::Explanation,
        schema_version: contract::Version::V1,
        root: "/synthetic/project".into(),
        path: "unknown.custom".into(),
        member: None,
        config_path: None,
        user_config_path: None,
        scope: backend::scope_policy(&Policy::defaults()),
        result: PathResult::Unrecognized,
    }
}
