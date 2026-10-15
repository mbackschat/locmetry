use super::ParseFailureReason;
use encoding_rs_io::DecodeReaderBytesBuilder;
use serde::Deserialize;
use std::{fs::File, io::Read, path::Path, str::FromStr};
use tokei::{Config, LanguageType, Report};

pub(super) enum Error {
    Read,
    Content(ParseFailureReason, String),
}

#[derive(Deserialize)]
struct Notebook {
    nbformat: u64,
    nbformat_minor: u64,
    metadata: Metadata,
    cells: Vec<Cell>,
}

#[derive(Deserialize)]
struct Metadata {
    language_info: Option<LanguageInfo>,
    kernelspec: Option<Kernel>,
}

#[derive(Deserialize)]
struct LanguageInfo {
    name: Option<String>,
    file_extension: Option<String>,
}

#[derive(Deserialize)]
struct Kernel {
    language: Option<String>,
}

#[derive(Deserialize)]
struct Cell {
    cell_type: CellType,
    source: Source,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum CellType {
    Code,
    Markdown,
    Raw,
    #[serde(other)]
    Unsupported,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum Source {
    Text(String),
    Fragments(Vec<String>),
}

impl Source {
    fn text(self) -> String {
        match self {
            Self::Text(text) => text,
            Self::Fragments(fragments) => fragments.concat(),
        }
    }
}

impl Metadata {
    fn language(&self) -> Result<LanguageType, Error> {
        let name = self
            .language_info
            .as_ref()
            .and_then(|info| info.name.as_deref())
            .or_else(|| {
                self.kernelspec
                    .as_ref()
                    .and_then(|kernel| kernel.language.as_deref())
            });
        let extension = self
            .language_info
            .as_ref()
            .and_then(|info| info.file_extension.as_deref());
        let language = match (name, extension) {
            (Some(name), _) => LanguageType::from_str(name).ok(),
            (None, Some(extension)) => {
                LanguageType::from_file_extension(extension.trim_start_matches('.'))
            }
            // Preserve Tokei 15's fallback only when the notebook declares no language.
            (None, None) => Some(LanguageType::Python),
        };
        match language {
            Some(LanguageType::Jupyter) | None => Err(Error::Content(
                ParseFailureReason::UnsupportedNotebookLanguage,
                "Notebook declares a language without a supported cell-text counter".into(),
            )),
            Some(language) => Ok(language),
        }
    }
}

/// Decode cells once because Tokei 15's private Jupyter parser both loses
/// failures and overlaps parent/blob totals (language_type.rs:67, 278).
/// String/array sources and raw cells follow nbformat v4.5, pinned in
/// docs/measurement.md; this checks countable structure, not all notebook metadata.
pub(super) fn parse(path: &Path, config: &Config) -> Result<Report, Error> {
    let mut text = String::new();
    DecodeReaderBytesBuilder::new()
        .build(File::open(path).map_err(|_| Error::Read)?)
        .read_to_string(&mut text)
        .map_err(|_| Error::Read)?;
    let notebook: Notebook = serde_json::from_str(&text).map_err(|error| {
        Error::Content(
            ParseFailureReason::InvalidNotebook,
            format!("Cannot decode notebook: {error}"),
        )
    })?;
    if notebook.nbformat != 4 || notebook.nbformat_minor > 5 {
        return Err(Error::Content(
            ParseFailureReason::UnsupportedNotebookVersion,
            format!(
                "Notebook version {}.{} is unsupported; expected 4.0 through 4.5",
                notebook.nbformat, notebook.nbformat_minor
            ),
        ));
    }
    let mut report = Report::new(path.to_owned());
    for cell in notebook.cells {
        let text = cell.source.text();
        match cell.cell_type {
            CellType::Code if !text.is_empty() => {
                let language = notebook.metadata.language()?;
                let stats = language.parse_from_str(&text, config);
                *report.stats.blobs.entry(language).or_default() += stats;
            }
            CellType::Code => {}
            CellType::Markdown | CellType::Raw => {
                for line in text.lines() {
                    if line.trim().is_empty() {
                        report.stats.blanks += 1;
                    } else {
                        report.stats.comments += 1;
                    }
                }
            }
            CellType::Unsupported => {
                return Err(Error::Content(
                    ParseFailureReason::UnsupportedNotebookCell,
                    "Notebook contains an unsupported cell type".into(),
                ))
            }
        }
    }
    Ok(report)
}
