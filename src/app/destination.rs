//! Where results go and in which format, checked before anything is measured.
use super::*;

/// The result destination and format, checked before measuring so a
/// misuse fails fast.
pub(crate) struct Destination<'a> {
    pub(crate) path: Option<&'a Path>,
    pub(crate) format: Format,
    /// A second view of the same measurement for stdout, beside a saved file.
    pub(crate) print: Option<Format>,
    pub(crate) folders: bool,
    pub(crate) folder_rows: Option<usize>,
}

/// `extra` holds the command's own single-format options, such as the
/// root's folder table options, beside the shared ones.
pub(crate) fn destination<'a>(
    shared: &'a SharedArgs,
    extra: &[(&'static str, bool, Format)],
) -> Result<Destination<'a>> {
    // Per-file records exist only in full JSON, so '--files' selects it too,
    // although it fails rather than warns where full JSON is not shown.
    let files = [("--files", shared.files, Format::Json)];
    let options = [
        presentation_options(shared).as_slice(),
        &shared.view.single_format_options(),
        &files,
        extra,
    ]
    .concat();
    let mut destination = output_destination(&shared.output, shared.format, &options);
    anyhow::ensure!(
        shared.print.is_none() || destination.path.is_some(),
        "'--print' requires '--output FILE'. Without a saved file, stdout already receives the result."
    );
    destination.print = shared.print;
    anyhow::ensure!(
        !shared.files || destination.shows(Format::Json),
        "'--files' adds per-file records only to full JSON. {}",
        destination.format_remedy(Format::Json)
    );
    Ok(destination)
}

/// `options` are the given command's single-format options, which select
/// the format when neither `format` nor `output` does.
pub(crate) fn output_destination<'a>(
    output: &'a Option<PathBuf>,
    format: Option<Format>,
    options: &[(&'static str, bool, Format)],
) -> Destination<'a> {
    let path = output.as_deref().filter(|path| *path != Path::new("-"));
    let format = format.unwrap_or_else(|| {
        default_format(
            io::stdout().is_terminal(),
            output.is_some(),
            implied_format(options),
        )
    });
    Destination {
        path,
        format,
        print: None,
        folders: false,
        folder_rows: None,
    }
}

impl Destination<'_> {
    /// Whether the saved result or the printed view uses `format`, so the
    /// options shaping that format take effect.
    pub(crate) fn shows(&self, format: Format) -> bool {
        self.format == format || self.print == Some(format)
    }

    pub(crate) fn format_remedy(&self, format: Format) -> String {
        let name = match format {
            Format::Table => "table",
            Format::Json => "json",
            Format::Headline => "headline",
            Format::Markdown => "markdown",
        };
        match self.path {
            Some(_) => format!("Use '--print {name}' or '--format {name}'."),
            None => format!("Use '--format {name}'."),
        }
    }

    /// Warns that `option` cannot shape this result, unless it shows one of
    /// the `supported` formats, offering the first of them as the remedy.
    pub(crate) fn warn_ineffective(&self, option: &str, supported: &[Format], styling: Styling) {
        if supported.iter().any(|format| self.shows(*format)) {
            return;
        }
        let names: Vec<&str> = supported
            .iter()
            .map(|format| match format {
                Format::Table => "table",
                Format::Json => "JSON",
                Format::Headline => "headline",
                Format::Markdown => "Markdown",
            })
            .collect();
        styling.stderr_message(
            Severity::Warning,
            format_args!(
                "'{option}' has no effect here. It applies only to {} output. {}",
                names.join(" and "),
                self.format_remedy(supported[0])
            ),
        );
    }

    pub(crate) fn write(&self, text: &str) -> Result<()> {
        match self.path {
            Some(path) => write_report(path, text),
            None => Ok(io::stdout().lock().write_all(text.as_bytes())?),
        }
    }
}

/// Columns available to share bars: `COLUMNS` when set, as for other terminal
/// programs, then the width of a stdout terminal, else a conventional 80.
pub(crate) fn bar_columns(stdout: bool) -> usize {
    std::env::var("COLUMNS")
        .ok()
        .and_then(|columns| columns.parse().ok())
        .filter(|columns| *columns > 0)
        .or_else(|| {
            stdout
                .then(terminal_size::terminal_size)
                .flatten()
                .map(|(width, _)| usize::from(width.0))
        })
        .unwrap_or(80)
}

/// The shared presentation options that shape only one format, with whether
/// each was given.
fn presentation_options(shared: &SharedArgs) -> [(&'static str, bool, Format); 8] {
    [
        ("--verbose", shared.verbose, Format::Table),
        ("--show-bars", shared.bars.is_some(), Format::Table),
        ("--table-purpose", !shared.purpose.is_empty(), Format::Table),
        (
            "--table-category",
            !shared.table_categories.is_empty(),
            Format::Table,
        ),
        (
            "--table-language",
            !shared.languages.is_empty(),
            Format::Table,
        ),
        ("--no-bars", shared.no_bars, Format::Table),
        ("--show-categories", shared.categories, Format::Table),
        ("--markdown-depth", shared.depth_explicit, Format::Markdown),
    ]
}

pub(crate) fn warn_ineffective_options(
    shared: &SharedArgs,
    destination: &Destination,
    styling: Styling,
) {
    for (option, given, format) in presentation_options(shared) {
        if given {
            destination.warn_ineffective(option, &[format], styling);
        }
    }
    shared.view.warn_ineffective(destination, styling);
}
