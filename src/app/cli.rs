//! Command-line interface: arguments, help text and parsing.
use super::*;

#[macro_use]
mod long_help;

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub(crate) enum Format {
    /// Tables and evidence.
    Table,
    /// Complete JSON; measurement reports can be compared later.
    Json,
    /// Compact JSON for agents and scripts; not a comparison-ready measurement.
    Headline,
    /// Markdown tables and evidence for reviews.
    Markdown,
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum ExplainFormat {
    /// Tables and evidence.
    Table,
    /// Complete JSON explanation.
    Json,
    /// Markdown tables and evidence for reviews.
    Markdown,
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum PurposeArg {
    Production,
    Test,
    Other,
}

impl PurposeArg {
    pub(crate) fn purpose(self) -> Purpose {
        match self {
            PurposeArg::Production => Purpose::Production,
            PurposeArg::Test => Purpose::Test,
            PurposeArg::Other => Purpose::Other,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum BarArg {
    /// Each row's Code split by purpose, scaled to the largest row.
    Purpose,
    /// Comment density, scaled so the densest row fills the bar.
    Comments,
}

impl BarArg {
    pub(crate) fn kind(self) -> BarKind {
        match self {
            BarArg::Purpose => BarKind::Purpose,
            BarArg::Comments => BarKind::Comments,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Sort {
    /// Largest code first; ties alphabetical.
    Code,
    /// Alphabetical.
    Name,
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Schema {
    /// The project configuration, `.locmetryrc`.
    Config,
    // The file differs on Windows (src/config/user.rs); help names the local one.
    #[cfg_attr(
        not(windows),
        doc = "Personal defaults in ~/.config/locmetry/config.toml."
    )]
    #[cfg_attr(windows, doc = r"Personal defaults in %APPDATA%\locmetry\config.toml.")]
    UserConfig,
    /// The JSON headline and full report.
    Report,
    /// Net-change comparison headline and full document.
    Comparison,
    /// Selection, parser and classification evidence for one path.
    Explanation,
    /// A workspace file for `locmetry workspace --workspace-file`.
    Workspace,
}

/// Select the format when no explicit format was supplied.
///
/// Explicit exports default to complete JSON so saved baselines remain comparable.
/// Otherwise an option that shapes only one format selects it, because naming
/// it states the view wanted: agents, which run without a terminal, otherwise
/// got a headline and a warning their output filters hid (feedback of three
/// projects, 2026-10-25). Ordinary pipes retain compact agent output
/// (docs/PRODUCT.md, Output Routing).
pub(crate) fn default_format(
    stdout_is_terminal: bool,
    output_given: bool,
    implied: Option<Format>,
) -> Format {
    match (output_given, implied, stdout_is_terminal) {
        (true, _, _) => Format::Json,
        (false, Some(format), _) => format,
        (false, None, false) => Format::Headline,
        (false, None, true) => Format::Table,
    }
}

/// The one format that every given option of `options` applies to; `None`
/// when none is given or they apply to different formats.
pub(crate) fn implied_format(options: &[(&str, bool, Format)]) -> Option<Format> {
    let mut formats = options
        .iter()
        .filter(|(_, given, _)| *given)
        .map(|(_, _, format)| *format);
    let first = formats.next()?;
    formats.all(|format| format == first).then_some(first)
}

// A macro rather than a const because `concat!` accepts only literals.
macro_rules! root_after_help {
    () => {
        "Run locmetry for source counts by language. Add --show-folders for the folder table, or --verbose to inspect classification, scope and coverage notes. To interpret counts and coverage, read locmetry --guide reports --section interpret-the-numbers --section unavailable-is-not-zero. For optional project configuration, use locmetry --guide configure.\n\nCommand names are reserved. To measure a directory named compare, use ./compare or --root compare."
    };
}

#[derive(Parser)]
#[command(
    version,
    about = "Measure source code by language, folder, and purpose: production, test, or other. Defaults to a table on a terminal, a JSON headline in pipes. An option of one format selects it. --output FILE saves only to that file, defaulting to full JSON; --format overrides the format.",
    after_help = root_after_help!(),
    after_long_help = concat!(root_after_help!(), root_examples!())
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
    /// Directory to measure, instead of --root [default: the current directory].
    #[arg(value_name = "DIR", conflicts_with = "root")]
    pub(crate) directory: Option<PathBuf>,
    /// Measure only this root-relative folder, under project policy
    #[arg(
        long_help = long_help::SUBFOLDER,
        long,
        value_name = "DIR",
        help_heading = "Selection and policy"
    )]
    pub(crate) subfolder: Option<PathBuf>,
    #[command(flatten)]
    pub(crate) root: RootArgs,
    #[command(flatten)]
    pub(crate) shared: SharedArgs,
    /// Add the folder table to single-root table output.
    #[arg(long = "show-folders", help_heading = "Presentation")]
    pub(crate) folders: bool,
    /// Show the folder table within an N-row budget
    #[arg(
        long_help = long_help::FOLDER_ROWS,
        long,
        value_name = "N",
        allow_negative_numbers = true,
        help_heading = "Presentation"
    )]
    pub(crate) folder_rows: Option<std::num::NonZeroUsize>,
    /// Print a JSON Schema and exit; standalone query.
    #[arg(long, value_enum, value_name = "TOPIC", help_heading = "Reference")]
    pub(crate) schema: Option<Schema>,
    /// Print a task guide and exit; standalone query
    #[arg(
        long_help = long_help::GUIDE,
        long,
        value_enum,
        value_name = "TOPIC",
        help_heading = "Reference"
    )]
    pub(crate) guide: Option<Guide>,
    /// Print only these sections of the guide or schema
    #[arg(
        long_help = long_help::SECTION,
        long,
        value_name = "ID",
        action = clap::ArgAction::Append,
        hide_short_help = true,
        help_heading = "Reference"
    )]
    pub(crate) section: Vec<String>,
    /// Set from the parsing command's `--generate` (see `generate_arg`).
    #[arg(skip)]
    pub(crate) generate: Option<Generate>,
    #[command(flatten)]
    pub(crate) styling: Styling,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Measure several repositories or one project's folders as one virtual monorepo.
    ///
    /// Defaults to a table on a terminal, a JSON headline in pipes. An option of one format selects it.
    /// --output FILE saves only to that file, defaulting to full JSON; --format overrides the format.
    #[command(
        after_help = "Add --verbose to inspect classification, scope and coverage notes. For optional configuration, use locmetry --guide configure; for results, use locmetry --guide reports.",
        after_long_help = long_help::WORKSPACE_EXAMPLES
    )]
    Workspace(WorkspaceArgs),
    /// Compare two saved full reports: net change is after minus before.
    ///
    /// Defaults to a table on a terminal, a JSON headline in pipes. An option of one format selects it.
    /// --output FILE saves only to that file, defaulting to full JSON; --format overrides the format.
    #[command(
        after_help = "For baselines and interpretation, use locmetry --guide compare.",
        after_long_help = long_help::COMPARE_EXAMPLES
    )]
    Compare(CompareArgs),
    /// Explain selection and classification of existing root-relative files.
    ///
    /// JSON prints the complete explanation; there is no separate headline.
    #[command(
        after_help = "For selection and classification diagnosis, use locmetry --guide selection.",
        after_long_help = long_help::EXPLAIN_EXAMPLES
    )]
    Explain(ExplainArgs),
}

#[derive(clap::Args)]
pub(crate) struct ExplainArgs {
    /// Root-relative files, or MEMBER/PATH with --workspace-file; several print JSON Lines.
    #[arg(value_name = "PATH", required = true)]
    pub(crate) paths: Vec<PathBuf>,
    #[command(flatten)]
    pub(crate) root: RootArgs,
    /// Workspace file; use the selected member's own policy.
    #[arg(long = "workspace-file", value_name = "FILE", conflicts_with_all = ["root", "config", "exclude", "hidden", "respect_ignore_files"], help_heading = "Selection and policy")]
    pub(crate) workspace: Option<PathBuf>,
    /// Ignore personal defaults.
    #[arg(long, help_heading = "Selection and policy")]
    pub(crate) no_user_config: bool,
    /// Stdout format [default: table on a terminal, json otherwise].
    #[arg(
        long,
        value_enum,
        value_name = "FORMAT",
        help_heading = "Output and content"
    )]
    pub(crate) format: Option<ExplainFormat>,
}

#[derive(clap::Args)]
pub(crate) struct CompareArgs {
    /// Full JSON measurement before the change.
    pub(crate) before: PathBuf,
    /// Full JSON measurement after the change.
    pub(crate) after: PathBuf,
    /// Focus on one exact workspace member name; accepts presence on either side.
    #[arg(long, value_name = "NAME", help_heading = "Selection and policy")]
    pub(crate) member: Option<String>,
    /// Result format [default: json with --output; otherwise table on a terminal, headline in pipes].
    #[arg(
        long,
        value_enum,
        value_name = "FORMAT",
        help_heading = "Output and content"
    )]
    pub(crate) format: Option<Format>,
    /// Write only to FILE; - selects stdout
    #[arg(
        long_help = long_help::COMPARE_OUTPUT,
        long,
        value_name = "FILE",
        help_heading = "Output and content"
    )]
    pub(crate) output: Option<PathBuf>,
    #[command(flatten)]
    pub(crate) view: ViewArgs,
}

/// Views of tables and Markdown that measurement and comparison share.
#[derive(Clone, Copy, clap::Args)]
pub(crate) struct ViewArgs {
    /// Print only the tables
    #[arg(
        long_help = long_help::TABLES_ONLY,
        long = "tables-only",
        help_heading = "Presentation"
    )]
    pub(crate) tables_only: bool,
    /// Show language variants such as TSX as rows of their own
    #[arg(
        long_help = long_help::SPLIT_LANGUAGES,
        long = "split-languages",
        help_heading = "Presentation"
    )]
    pub(crate) split_languages: bool,
}

impl ViewArgs {
    pub(crate) fn variants(self) -> LanguageVariants {
        match self.split_languages {
            true => LanguageVariants::Split,
            false => LanguageVariants::Folded,
        }
    }

    /// Options of this group that apply to a single format.
    pub(crate) fn single_format_options(self) -> [(&'static str, bool, Format); 1] {
        [("--tables-only", self.tables_only, Format::Table)]
    }

    pub(crate) fn warn_ineffective(self, destination: &Destination, styling: Styling) {
        for (option, given, format) in self.single_format_options() {
            if given {
                destination.warn_ineffective(option, &[format], styling);
            }
        }
        if self.split_languages {
            let formats = [Format::Table, Format::Markdown];
            destination.warn_ineffective("--split-languages", &formats, styling);
        }
    }
}

/// Settings of a single scan root, which a workspace leaves to each member's
/// own configuration (docs/PRODUCT.md, Workspace Mode).
#[derive(clap::Args)]
pub(crate) struct RootArgs {
    /// Scan root [default: the current directory].
    #[arg(long, value_name = "DIR", help_heading = "Selection and policy")]
    pub(crate) root: Option<PathBuf>,
    /// TOML configuration [default: .locmetryrc in the scan root, else built-in settings].
    #[arg(long, value_name = "FILE", help_heading = "Selection and policy")]
    pub(crate) config: Option<PathBuf>,
    /// Add a root-relative exclusion glob; repeatable.
    #[arg(long, value_name = "GLOB", help_heading = "Selection and policy")]
    pub(crate) exclude: Vec<String>,
    /// Override whether hidden files and directories are measured.
    #[arg(long, value_name = "BOOL", help_heading = "Selection and policy")]
    pub(crate) hidden: Option<bool>,
    /// Override whether ignore files limit the selection.
    #[arg(long, value_name = "BOOL", help_heading = "Selection and policy")]
    pub(crate) respect_ignore_files: Option<bool>,
}

impl RootArgs {
    pub(crate) fn overrides(&self) -> ScopeOverrides {
        ScopeOverrides {
            hidden: self.hidden,
            respect_ignore_files: self.respect_ignore_files,
            exclude: self.exclude.clone(),
        }
    }
}

#[derive(clap::Args)]
pub(crate) struct WorkspaceArgs {
    /// Member root; repeatable
    #[arg(
        long_help = long_help::WORKSPACE_ROOT,
        long = "root",
        value_name = "DIR",
        required_unless_present = "file",
        help_heading = "Selection and policy"
    )]
    pub(crate) roots: Vec<PathBuf>,
    /// Read the member roots from a TOML workspace file
    #[arg(
        long_help = long_help::WORKSPACE_FILE,
        long = "workspace-file",
        value_name = "FILE",
        conflicts_with = "roots",
        help_heading = "Selection and policy"
    )]
    pub(crate) file: Option<PathBuf>,
    #[command(flatten)]
    pub(crate) shared: SharedArgs,
}

/// Settings that both modes accept: user defaults and the report's form.
#[derive(clap::Args)]
pub(crate) struct SharedArgs {
    #[cfg_attr(
        not(windows),
        doc = "Ignore user defaults in ~/.config/locmetry/config.toml."
    )]
    #[cfg_attr(
        windows,
        doc = r"Ignore user defaults in %APPDATA%\locmetry\config.toml."
    )]
    #[arg(long, help_heading = "Selection and policy")]
    pub(crate) no_user_config: bool,
    /// Write only to FILE, outside the scan root; - selects stdout
    #[arg(
        long_help = long_help::OUTPUT,
        long,
        value_name = "FILE",
        help_heading = "Output and content"
    )]
    pub(crate) output: Option<PathBuf>,
    /// With --output FILE, also print the measurement in FORMAT
    #[arg(
        long_help = long_help::PRINT,
        long,
        value_enum,
        value_name = "FORMAT",
        help_heading = "Output and content"
    )]
    pub(crate) print: Option<Format>,
    /// Include per-file records in full JSON
    #[arg(
        long_help = long_help::FILES,
        long,
        help_heading = "Output and content"
    )]
    pub(crate) files: bool,
    /// Add the source purpose/category table
    #[arg(
        long_help = long_help::SHOW_CATEGORIES,
        long = "show-categories",
        help_heading = "Presentation"
    )]
    pub(crate) categories: bool,
    /// Include classification, scope and coverage notes in table output.
    #[arg(long, conflicts_with = "tables_only", help_heading = "Presentation")]
    pub(crate) verbose: bool,
    /// Add share charts to tables, beside them when they fit
    #[arg(long_help = long_help::SHOW_BARS, long = "show-bars",
        value_enum,
        value_name = "KIND",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "purpose",
        help_heading = "Presentation"
    )]
    pub(crate) bars: Option<BarArg>,
    /// Omit the share bars a terminal shows automatically.
    #[arg(long, conflicts_with = "bars", help_heading = "Presentation")]
    pub(crate) no_bars: bool,
    #[command(flatten)]
    pub(crate) view: ViewArgs,
    /// Show only these purposes in tables; comma-separated
    #[arg(
        long_help = long_help::TABLE_PURPOSE,
        long = "table-purpose",
        value_enum,
        value_delimiter = ',',
        value_name = "PURPOSES",
        help_heading = "Presentation"
    )]
    pub(crate) purpose: Vec<PurposeArg>,
    /// Show only these categories in tables; comma-separated PURPOSE[:CATEGORY]
    #[arg(
        long = "table-category",
        long_help = long_help::TABLE_CATEGORY,
        value_delimiter = ',',
        value_name = "CATEGORIES",
        value_parser = locmetry::CategoryPattern::parse,
        help_heading = "Presentation"
    )]
    pub(crate) table_categories: Vec<locmetry::CategoryPattern>,
    /// Show only these source languages in tables; comma-separated
    #[arg(long_help = long_help::TABLE_LANGUAGE, long = "table-language",
        value_delimiter = ',',
        value_name = "LANGUAGES",
        value_parser = locmetry::source_language,
        help_heading = "Presentation"
    )]
    pub(crate) languages: Vec<&'static str>,
    /// Result format [default: json with --output; otherwise table on a terminal, headline in pipes].
    #[arg(
        long,
        value_enum,
        value_name = "FORMAT",
        help_heading = "Output and content"
    )]
    pub(crate) format: Option<Format>,
    /// Row order of table and Markdown output
    #[arg(
        long_help = long_help::SORT,
        long,
        value_enum,
        default_value = "code",
        help_heading = "Presentation"
    )]
    pub(crate) sort: Sort,
    /// Folder-breakdown depth in Markdown
    #[arg(
        long_help = long_help::MARKDOWN_DEPTH,
        long = "markdown-depth",
        default_value_t = 2,
        value_name = "N",
        help_heading = "Presentation"
    )]
    pub(crate) depth: usize,
    #[arg(skip)]
    pub(crate) depth_explicit: bool,
}

/// Whether human terminal output may use styling: only for a terminal, unless
/// `NO_COLOR` is set to anything nonempty (https://no-color.org), the
/// terminal is `dumb`, or a Windows console refused virtual-terminal
/// processing (`console_escapes`, which is `None` elsewhere).
pub(crate) fn emphasis(
    stdout_is_terminal: bool,
    no_color: Option<String>,
    term: Option<String>,
    console_escapes: Option<bool>,
) -> bool {
    stdout_is_terminal
        && no_color.is_none_or(|value| value.is_empty())
        && term.as_deref() != Some("dumb")
        && console_escapes != Some(false)
}

/// Asks a Windows console once to interpret escape sequences; Windows
/// PowerShell 5.1's console host prints them literally unless enabled
/// (https://learn.microsoft.com/windows/console/console-virtual-terminal-sequences).
fn console_escapes() -> Option<bool> {
    static ENABLED: std::sync::OnceLock<Option<bool>> = std::sync::OnceLock::new();
    *ENABLED.get_or_init(anstyle_query::windows::enable_ansi_colors)
}

#[derive(Clone, Copy, clap::Args)]
pub(crate) struct Styling {
    /// Disable terminal colors and styling
    #[arg(
        long_help = long_help::NO_COLOR,
        long,
        global = true,
        help_heading = "Presentation"
    )]
    pub(crate) no_color: bool,
}

/// How serious a stderr message is. Its label keeps severity readable without
/// color, as clap's own `error:` prefix already does for usage errors.
#[derive(Clone, Copy)]
pub(crate) enum Severity {
    Warning,
    Error,
}

impl Severity {
    fn role(self) -> Role {
        match self {
            Self::Warning => Role::Warning,
            Self::Error => Role::Error,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::Warning => "warning:",
            Self::Error => "error:",
        }
    }
}

impl Styling {
    pub(crate) fn theme(self, is_terminal: bool) -> Theme {
        Theme::terminal(
            !self.no_color
                && emphasis(
                    is_terminal,
                    std::env::var("NO_COLOR").ok(),
                    std::env::var("TERM").ok(),
                    console_escapes(),
                ),
        )
    }

    pub(crate) fn stderr_message(self, severity: Severity, message: impl std::fmt::Display) {
        eprintln!(
            "{} {}",
            self.theme(io::stderr().is_terminal())
                .paint(severity.role(), severity.label()),
            plain(&message.to_string())
        );
    }
}
