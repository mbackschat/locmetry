//! The stage and evidence that decided a file, as `explain` reports them.
use super::GeneratedEvidence;
use schemars::JsonSchema;
use serde::Serialize;

#[derive(Clone, Copy, PartialEq, Eq, Serialize, JsonSchema)]
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
    pub module: Option<crate::contract::Module>,
    /// The generated-code marker that refined the module's claim to category `generated`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated: Option<GeneratedEvidence>,
    /// The refinement that replaced the deciding stage's category.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refinement: Option<RefinementEvidence>,
}

/// A refinement that changed a decided file's category, as `explain` reports it.
#[derive(Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RefinementEvidence {
    pub id: String,
    /// Matching globs of the refinement, relative to the member's root.
    pub matched_patterns: Vec<String>,
    /// The refinement's stated reason, when supplied.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// One first-match decision about a counted file, see [`super::Policy::resolve`].
pub(crate) struct Resolution {
    pub(crate) classification: super::Classification,
    /// The module credited with production source.
    pub(crate) module: Option<crate::contract::Module>,
    /// A module layout decided this source file, so a generated-code marker
    /// may refine it.
    pub(crate) markable: bool,
}
