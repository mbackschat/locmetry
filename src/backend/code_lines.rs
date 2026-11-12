//! The lines of a counted file that the counter classifies as code, which a
//! refinement's `content` expression is tested against.
use super::{decoded_text, ParseFailureReason};
use crate::config::MatchedLine;
use crate::paths::filesystem_path;
use std::path::Path;
use tokei::{LanguageType, LineClass};

/// The code lines of `name`, counted in `language`, with their 1-based line
/// numbers; `None` when the file cannot be read or decoded, or when its lines
/// are counted elsewhere, as a Jupyter notebook's or a Markdown file's with
/// fenced languages.
pub(super) fn code_lines(root: &Path, name: &str, language: &str) -> Option<Vec<MatchedLine>> {
    let language = LanguageType::from_name(language)?;
    let text = decoded_text(
        &filesystem_path(root, name),
        ParseFailureReason::InvalidEncoding,
        "source",
    )
    .ok()?;
    let classes = language.line_classes(text.as_bytes(), &tokei::Config::default())?;
    Some(
        text.lines()
            .zip(classes)
            .enumerate()
            .filter(|(_, (_, class))| *class == LineClass::Code)
            .map(|(index, (line, _))| MatchedLine {
                number: index + 1,
                text: line.to_owned(),
            })
            .collect(),
    )
}
