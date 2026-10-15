//! Focused evidence for one existing path, using ordinary selection and counting.
use crate::style::{Role, Theme};
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

/// One path's selection, parser and classification evidence. Exit 2 means the target's counts are unavailable; exit 0 includes counted, excluded, unrecognized, filtered and symlink paths. This is not a saved measurement input for compare.
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

pub use crate::config::{Decision, Stage};

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
    ensure!(
        path.components()
            .any(|part| part != std::path::Component::CurDir),
        "Explanation target must be a file"
    );
    let path = backend::relative_path(path)?;
    let mut current = root.to_owned();
    let mut relative = std::path::PathBuf::new();
    for component in Path::new(&path).components() {
        // APFS can resolve case/Unicode aliases that the walk never emits. The
        // root-relative contract uses directory-entry spelling (PRODUCT, Path Explanations).
        let exact = std::fs::read_dir(&current)
            .context("Cannot inspect explanation path spelling")?
            .collect::<std::io::Result<Vec<_>>>()?
            .iter()
            .any(|entry| entry.file_name() == component.as_os_str());
        current.push(component);
        let metadata = std::fs::symlink_metadata(&current)
            .context("Explanation target does not exist or cannot be inspected")?;
        ensure!(
            exact,
            "Explanation target must use the exact spelling of directory entries: {path}"
        );
        relative.push(component);
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
        self.render_with_theme(
            markdown,
            if markdown {
                Theme::MARKDOWN
            } else {
                Theme::PLAIN
            },
        )
    }

    pub fn render_with_theme(&self, markdown: bool, theme: Theme) -> String {
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
            "# Path explanation\n\n| Evidence | Value |\n| --- | --- |\n".to_owned()
        } else {
            format!("{}\n", theme.paint(Role::Heading, "Path explanation"))
        };
        for (key, value) in rows {
            let role = match key {
                "State" if self.partial() || matches!(self.result, PathResult::Unrecognized) => {
                    Role::Warning
                }
                _ => Role::Heading,
            };
            if markdown {
                let value = crate::output::cell(&value);
                text.push_str(&format!("| {} | {value} |\n", theme.paint(role, key)));
            } else {
                text.push_str(&format!(
                    "{}: {}\n",
                    theme.paint(role, key),
                    crate::output::plain(&value)
                ));
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

#[cfg(test)]
mod style_tests {
    use super::*;

    #[test]
    fn unavailable_state_is_prominent_and_plain_explanations_stay_unstyled() {
        let mut explanation = example();
        explanation.result = PathResult::Unread;
        let styled = explanation.render_with_theme(false, Theme::TERMINAL);
        assert!(styled.contains("\x1b[33mState\x1b[0m: unread: counts unavailable"));
        assert_eq!(
            styled
                .replace("\x1b[1m", "")
                .replace("\x1b[33m", "")
                .replace("\x1b[0m", ""),
            explanation.render(false)
        );
        let markdown = explanation.render(true);
        assert!(markdown.contains("| **State** | unread: counts unavailable |"));
        assert!(!markdown.contains('\x1b'));
    }
}
