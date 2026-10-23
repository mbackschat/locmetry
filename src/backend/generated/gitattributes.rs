//! `linguist-generated` declarations in the project's `.gitattributes` files,
//! with Git's matching and precedence (gitattributes(5)): later lines and
//! deeper files win, and the attribute's state follows GitHub Linguist,
//! where any value but `false` means generated (`lazy_blob.rb`,
//! `boolean_attribute`).
use globset::{GlobBuilder, GlobMatcher};
use std::collections::BTreeSet;
use std::path::Path;

const ATTRIBUTE: &str = "linguist-generated";
const FILE: &str = ".gitattributes";

/// What the deciding line says about a file.
#[derive(Debug, PartialEq, Eq)]
pub(super) enum Declaration {
    /// Generated, with the file and line that say so.
    Generated(String),
    /// Explicitly not generated: `-linguist-generated` or `=false`.
    NotGenerated,
}

#[derive(Clone, Copy)]
enum State {
    Generated,
    NotGenerated,
    /// `!linguist-generated`, which undoes earlier lines and files.
    Unspecified,
}

struct Line {
    matcher: GlobMatcher,
    state: State,
    text: String,
}

struct AttributesFile {
    /// Root-relative directory, empty for the scan root.
    directory: String,
    path: String,
    lines: Vec<Line>,
}

pub(super) struct Attributes {
    /// Shallowest first, so deeper files are consulted later and win.
    files: Vec<AttributesFile>,
}

impl Attributes {
    /// The `.gitattributes` files in the directories that hold `files` and
    /// their ancestors, read from disk whatever the selection, as Git reads
    /// them. An unreadable file declares nothing.
    pub(super) fn load(root: &Path, files: &BTreeSet<String>) -> Self {
        let mut directories = BTreeSet::from([String::new()]);
        for file in files {
            let mut path = file.as_str();
            while let Some((parent, _)) = path.rsplit_once('/') {
                directories.insert(parent.to_string());
                path = parent;
            }
        }
        let mut found: Vec<AttributesFile> = directories
            .into_iter()
            .filter_map(|directory| {
                let path = join(&directory, FILE);
                let text = std::fs::read_to_string(root.join(&path)).ok()?;
                let lines = text
                    .lines()
                    .filter_map(|line| parse(&directory, line))
                    .collect();
                Some(AttributesFile {
                    directory,
                    path,
                    lines,
                })
            })
            .collect();
        found.sort_by_key(|file| {
            file.directory
                .split('/')
                .filter(|part| !part.is_empty())
                .count()
        });
        Self { files: found }
    }

    /// The declaration of the last matching line in the deepest file that
    /// has one; `None` when no line decides or the last one unspecifies.
    pub(super) fn declaration(&self, path: &str) -> Option<Declaration> {
        let mut decided: Option<(State, &AttributesFile, &Line)> = None;
        for file in &self.files {
            if !(file.directory.is_empty() || path.starts_with(&format!("{}/", file.directory))) {
                continue;
            }
            for line in &file.lines {
                if line.matcher.is_match(path) {
                    decided = Some((line.state, file, line));
                }
            }
        }
        match decided? {
            (State::Generated, file, line) => Some(Declaration::Generated(format!(
                "{}: {}",
                file.path, line.text
            ))),
            (State::NotGenerated, ..) => Some(Declaration::NotGenerated),
            (State::Unspecified, ..) => None,
        }
    }
}

/// The attribute line for [`ATTRIBUTE`], with its pattern anchored as Git
/// anchors it: relative to the file's directory when it contains a slash,
/// at any depth below it otherwise. Lines Git would not apply to files,
/// such as negated, quoted or directory patterns, and macro definitions,
/// are skipped.
fn parse(directory: &str, line: &str) -> Option<Line> {
    let mut fields = line.split_whitespace();
    let pattern = fields.next()?;
    if pattern.starts_with(['#', '!', '"', '[']) || pattern.ends_with('/') {
        return None;
    }
    let (state, token) = fields
        .rev()
        .find_map(|field| Some((state(field)?, field)))?;
    let relative = match pattern.strip_prefix('/') {
        Some(anchored) => anchored.to_string(),
        None if pattern.contains('/') => pattern.to_string(),
        None => format!("**/{pattern}"),
    };
    let matcher = GlobBuilder::new(&join(directory, &relative))
        .literal_separator(true)
        .backslash_escape(true)
        .build()
        .ok()?
        .compile_matcher();
    Some(Line {
        matcher,
        state,
        text: format!("{pattern} {token}"),
    })
}

fn state(field: &str) -> Option<State> {
    match field {
        ATTRIBUTE => Some(State::Generated),
        _ if field.strip_prefix('-') == Some(ATTRIBUTE) => Some(State::NotGenerated),
        _ if field.strip_prefix('!') == Some(ATTRIBUTE) => Some(State::Unspecified),
        _ => match field.split_once('=') {
            Some((ATTRIBUTE, "false")) => Some(State::NotGenerated),
            Some((ATTRIBUTE, _)) => Some(State::Generated),
            _ => None,
        },
    }
}

fn join(directory: &str, name: &str) -> String {
    if directory.is_empty() {
        name.to_string()
    } else {
        format!("{directory}/{name}")
    }
}
