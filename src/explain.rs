//! Focused evidence for one existing path, using ordinary selection and counting.
use crate::style::{Role, Theme};
use crate::{backend, contract, output, Policy};
use anyhow::{ensure, Result};
use schemars::JsonSchema;
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Explanation,
}

pub use crate::contract::Version;

/// One path's selection, parser and classification evidence. Exit 2 means the target's counts are unavailable; exit 0 includes counted, excluded, unrecognized, filtered and symlink paths. This is not a saved measurement input for compare.
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(example = example())]
pub struct Explanation {
    pub document_type: DocumentType,
    pub schema_version: Version,
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

pub use crate::backend::FilterReason;

pub fn inspect(root: &Path, path: &Path, policy: &mut Policy) -> Result<(String, PathResult)> {
    let root = backend::scan_root(root)?;
    let root = root.as_path();
    ensure!(
        path.components()
            .any(|part| part != std::path::Component::CurDir),
        "Explanation target '{}' must be a file",
        path.display()
    );
    let path = backend::relative_path(path)?;
    if let Some(at) = backend::inspect_path(root, &path, "Explanation")? {
        return Ok((path, PathResult::Symlink { at }));
    }
    ensure!(
        root.join(&path).is_file(),
        "Explanation target '{path}' must be a file"
    );
    let (scan, exclusion) =
        backend::scan_selection(root, policy, Some(backend::Target::File(&path)))?;
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
        let reason = backend::filter_reason(root, policy.scope(), backend::Target::File(&path))?;
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

    /// Names paths as a workspace report does; a folder member's paths are
    /// relative to its project `folder`.
    pub fn qualify(&mut self, member: String, folder: Option<&str>) {
        let prefix = |path: &str| crate::paths::member_path(&member, folder, path);
        self.path = prefix(&self.path);
        match &mut self.result {
            PathResult::Counted { file, decision } => {
                file.path = prefix(&file.path);
                for module in [&mut decision.module, &mut file.module]
                    .into_iter()
                    .flatten()
                {
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
                            "{} code, {}, {}, {}",
                            output::grouped(file.counts.code),
                            output::counted(file.counts.comments, "comment"),
                            output::counted(file.counts.blanks, "blank"),
                            output::counted(file.counts.lines, "line")
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
                let label = |module: &crate::contract::Module| {
                    format!(
                        "{} ({})",
                        output::module_label(&module.path),
                        module.detector
                    )
                };
                match (&decision.module, &file.module) {
                    (Some(module), _) => rows.push(("Module", label(module))),
                    // An entry decided the file, but the module keeps its credit.
                    (None, Some(module)) => rows.push((
                        "Module credit",
                        format!("{}, whose layout covers this file", label(module)),
                    )),
                    (None, None) => {}
                }
            }
            PathResult::Excluded { at, patterns } => rows.extend([
                ("State", "excluded".into()),
                ("Excluded at", at.clone()),
                ("Patterns", patterns.join(", ")),
            ]),
            PathResult::NotSelected { reason } => {
                let (code, detail) = match reason {
                    FilterReason::IgnorePolicy => ("ignore_policy", "Disabling ignore filtering selects this path. The winning ignore rule is not recorded."),
                    FilterReason::HiddenPolicy => ("hidden_policy", "Enabling hidden paths selects this path."),
                    FilterReason::ReasonNotRecorded => ("reason_not_recorded", "The available evidence does not establish a cause."),
                };
                rows.extend([
                    ("State", "not_selected".into()),
                    ("Reason", code.into()),
                    ("Detail", detail.into()),
                ]);
            }
            PathResult::Unrecognized => {
                rows.push((
                    "State",
                    "unrecognized (no parser or language mapping)".into(),
                ));
            }
            PathResult::Unread => {
                rows.push(("State", "unread (counts unavailable)".into()));
            }
            PathResult::ParseFailed { failure } => rows.extend([
                ("State", "parse_failed (counts unavailable)".into()),
                ("Reason", failure.reason.to_string()),
                ("Detail", failure.message.clone()),
            ]),
            PathResult::Symlink { at } => rows.extend([
                ("State", "symlink (not followed)".into()),
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
                    crate::style::plain(&value)
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
        schema_version: Version::V3,
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
        assert!(styled.contains("\x1b[33mState\x1b[0m: unread (counts unavailable)"));
        assert_eq!(
            styled
                .replace("\x1b[1m", "")
                .replace("\x1b[33m", "")
                .replace("\x1b[0m", ""),
            explanation.render(false)
        );
        let markdown = explanation.render(true);
        assert!(markdown.contains("| **State** | unread (counts unavailable) |"));
        assert!(!markdown.contains('\x1b'));
    }
}
