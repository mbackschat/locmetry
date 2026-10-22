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

pub(crate) fn destination(shared: &SharedArgs) -> Result<Destination<'_>> {
    let mut destination = output_destination(&shared.output, shared.format);
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

pub(crate) fn output_destination(
    output: &Option<PathBuf>,
    format: Option<Format>,
) -> Destination<'_> {
    let path = output.as_deref().filter(|path| *path != Path::new("-"));
    let format =
        format.unwrap_or_else(|| default_format(io::stdout().is_terminal(), output.is_some()));
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
            Role::Warning,
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

pub(crate) fn warn_ineffective_options(
    shared: &SharedArgs,
    destination: &Destination,
    styling: Styling,
) {
    for (given, option) in [
        (shared.verbose, "--verbose"),
        (shared.bars.is_some(), "--show-bars"),
        (!shared.purpose.is_empty(), "--table-purpose"),
        (!shared.languages.is_empty(), "--table-language"),
        (shared.no_bars, "--no-bars"),
        (shared.categories, "--show-categories"),
    ] {
        if given {
            destination.warn_ineffective(option, &[Format::Table], styling);
        }
    }
    if shared.depth_explicit {
        destination.warn_ineffective("--markdown-depth", &[Format::Markdown], styling);
    }
    shared.view.warn_ineffective(destination, styling);
}
