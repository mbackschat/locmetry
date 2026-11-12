//! The lines of a counted file that the counter classifies as code, which a
//! refinement's `content` expression is tested against.
use super::{decoded_text, ParseFailureReason};
use crate::config::CodeText;
use crate::paths::filesystem_path;
use std::path::Path;
use tokei::LanguageType;

/// The text of `name`, counted in `language` with `config`, and the class of
/// each line, as the counter splits them (at line feeds only); `None` when the
/// file cannot be read or decoded, or when its lines are counted elsewhere, as
/// a Jupyter notebook's or a Markdown file's with fenced languages.
pub(super) fn code_text(
    root: &Path,
    name: &str,
    language: &str,
    config: &tokei::Config,
) -> Option<CodeText> {
    let language = LanguageType::from_name(language)?;
    let text = decoded_text(
        &filesystem_path(root, name),
        ParseFailureReason::InvalidEncoding,
        "source",
    )
    .ok()?;
    let classes = language.line_classes(text.as_bytes(), config)?;
    Some(CodeText::new(text, classes))
}
