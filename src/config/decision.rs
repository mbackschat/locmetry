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
    /// The first code line that matched the refinement's `content`, cut to 200 characters with a trailing ellipsis; absent for a refinement without one.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched_line: Option<MatchedLine>,
}

/// A code line of a file, with its 1-based line number. A line longer than
/// [`MatchedLine::CAP`] characters is cut there and ends in an ellipsis.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MatchedLine {
    pub number: usize,
    pub text: String,
}

impl MatchedLine {
    /// The most characters of a line that are kept.
    pub const CAP: usize = 200;

    fn new(number: usize, line: &str) -> Self {
        let text = match line.char_indices().nth(Self::CAP) {
            Some((end, _)) => format!("{}\u{2026}", &line[..end]),
            None => line.to_owned(),
        };
        Self { number, text }
    }
}

/// A counted file's text with the class the counter gave each of its lines.
pub(crate) struct CodeText {
    text: String,
    classes: Vec<tokei::LineClass>,
}

impl CodeText {
    pub(crate) fn new(text: String, classes: Vec<tokei::LineClass>) -> Self {
        Self { text, classes }
    }

    /// The first code line `pattern` matches, by its whole text.
    fn first_match(&self, pattern: &regex::Regex) -> Option<MatchedLine> {
        self.text
            .lines()
            .zip(&self.classes)
            .enumerate()
            .find(|(_, (line, class))| **class == tokei::LineClass::Code && pattern.is_match(line))
            .map(|(index, (line, _))| MatchedLine::new(index + 1, line))
    }
}

/// The code of one file, read the first time a content refinement asks, and
/// at most once.
pub(crate) struct CodeLines<'a> {
    read: &'a dyn Fn() -> Option<CodeText>,
    text: std::cell::OnceCell<Option<CodeText>>,
}

impl<'a> CodeLines<'a> {
    pub(crate) fn new(read: &'a dyn Fn() -> Option<CodeText>) -> Self {
        Self {
            read,
            text: std::cell::OnceCell::new(),
        }
    }

    /// The first code line `pattern` matches; none when the file's lines
    /// cannot be classified.
    pub(super) fn first_match(&self, pattern: &regex::Regex) -> Option<MatchedLine> {
        self.text
            .get_or_init(self.read)
            .as_ref()?
            .first_match(pattern)
    }
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
