//! Command-line interface: arguments, help text and parsing.
use super::*;

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub(crate) enum Format {
    /// Human-readable tables and evidence.
    Table,
    /// Complete JSON document; measurement reports can be compared later.
    Json,
    /// Compact JSON for agents and scripts; not a comparison-ready measurement.
    Headline,
    /// Markdown tables and evidence to save or paste into reviews.
    Markdown,
}

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum ExplainFormat {
    /// Human-readable tables and evidence.
    Table,
    /// Complete JSON explanation.
    Json,
    /// Markdown tables and evidence.
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
/// Explicit exports default to complete JSON so saved baselines remain comparable;
/// ordinary pipes retain compact agent output (docs/PRODUCT.md, Output Routing).
pub(crate) fn default_format(stdout_is_terminal: bool, output_given: bool) -> Format {
    match (output_given, stdout_is_terminal) {
        (true, _) => Format::Json,
        (false, false) => Format::Headline,
        (false, true) => Format::Table,
    }
}

// A macro rather than a const because `concat!` accepts only literals.
macro_rules! root_after_help {
    () => {
        "Run locmetry for source counts by language. Add --show-folders for the folder table, or --verbose to inspect classification, scope and coverage notes. For optional project configuration, use locmetry --guide configure.\n\nCommand names are reserved. To measure a directory named compare, use ./compare or --root compare."
    };
}

#[derive(Parser)]
#[command(
    version,
    about = "Measure source code by language, folder, and purpose: production, test, or other. Defaults to a table on a terminal, a JSON headline in pipes. --output FILE saves only to that file, defaulting to full JSON; --format overrides the format.",
    after_help = root_after_help!(),
    // The README "Common commands" table mirrors these examples; tests/integration/readme.rs enforces it.
    after_long_help = concat!(
        root_after_help!(),
        "\n\nExamples:
  Measure another project:
    locmetry --root ../service
  Inspect scope, classification and coverage notes:
    locmetry --verbose
  Measure a subfolder with project policy and module context:
    locmetry --subfolder web/src
  Explain a file's classification or missing counts:
    locmetry explain web/src/orders/Order.test.ts
  Save full JSON with per-file decisions outside the root:
    locmetry --files --output ../report.json
  Save full JSON and print the table from one scan, as for agents:
    locmetry --output ../report.json --print table
  Save a Markdown report:
    locmetry --format markdown --output ../report.md
  Measure repositories together, each with its own policy:
    locmetry workspace --root ../web --root ../api
  Show production source in the table, with its comment density:
    locmetry --format table --table-purpose production
  Inspect the configuration contract:
    locmetry --schema config
  Read full JSON on stdout:
    locmetry --root service --format json --files
  Measure selected source, including ignored files, except vendor:
    locmetry --root service --subfolder src --exclude 'src/vendor/**' --hidden true --respect-ignore-files false --format json
  Show adaptive folder detail and production categories:
    locmetry --root service --format table --folder-rows 30 --table-purpose production --show-categories --show-bars=comments --sort name
  Save a sorted Markdown tree to depth 3:
    locmetry --root service --format markdown --markdown-depth 3 --sort name --output report.md
  Measure an explicit workspace definition:
    locmetry workspace --workspace-file repos.toml --files --output workspace.json --print table
  Compare two saved full reports:
    locmetry compare before.json after.json --format table

  For a production-rule change covered by the example below, inspect existing
  configuration and measure current decisions, then adapt the paths and category
  and run the verification command. Reading a guide or schema first is optional
  for this route. Use --guide configure or --schema config for exclusions,
  overrides, intentional Other, or semantics not shown here. Verify the reported
  per-file decisions and unresolved review items before concluding.

  Classify maintained scripts in service/.locmetryrc (TOML):
    version = 1

    [[rules]]
    id = \"maintained-scripts\"
    match = [\"scripts/**\"]
    purpose = \"production\"
    category = \"tooling\"
  Rule patterns are root-relative; built-in test/build classification precedes rules.
  Verify the policy with per-file decisions and a human overview:
    locmetry --root service --files --output report.json --print table --verbose

Read results:
  Lines = Code + Comments + Blanks. Default tables describe source; JSON
  summary.source and summary.all_counted distinguish source from data/documentation.
  Test/prod = Test Code / Production Code, excluding Other; it is not test coverage.
  Files are classified whole; a legend names Rust #[cfg(test)] code in production.
  Other remains counted. Unclaimed source appears in review; a clear review list
  does not prove correct classification. Unavailable counts are not zero.
  Use --verbose for coverage, inferred build modules, submodule hints and unused
  policy. Full JSON has breakdown, review and metadata.coverage; --files adds files.

Choose scope and views:
  --subfolder retains the root's policy and module context; --root on that folder
  starts a different measurement. Configuration is not discovered in ancestors.
  Quote root-relative exclusion globs. --table-purpose and --table-language filter tables, not JSON.
  Tables and Markdown count language variants within their family, such as TSX within
  TypeScript, and name each fold; --split-languages separates them. JSON keeps backend names.
  --folder-rows needs --format table or --print table; its budget retains all top-level folders.
  --markdown-depth limits only Markdown's Folder breakdown; other evidence can name deeper paths.
  Folder parent rows include descendants.
  --no-bars removes charts; --no-color or NO_COLOR disables styling; COLUMNS controls width.

Broader policy:
  Project overrides precede test/build conventions; ordinary rules follow them.
  Use exclusions for scope, intentional Other for counted historical material,
  and language mappings for unrecognized extensions. Build modules infer conventional
  layouts from descriptor names without executing builds; submodule hints are not
  ownership decisions. --schema config gives precedence, conventions and settings.
  Personal defaults only add scope exclusions and language mappings. Inspect
  --schema user-config; use --no-user-config for reproducible runs without them.

Automation:
  --output FILE saves full JSON by default; --files adds per-file decisions and
  --print table displays that same scan. Save outside every measured root.
  Exit 0 means completed, not necessarily complete language recognition.
  Exit 2 writes a partial report with unavailable/inaccurate counts; retain that
  status. Exit 1 is a command failure, with no usable report promised. Diagnostics
  go to stderr. Preserve status before querying JSON or running a pipeline.

For routine capture-and-compare tasks, start with locmetry compare --help and execute
its recipes. Extra guide/schema reading is optional for that route; inspect summary
and findings before drawing conclusions.
For workflows and saved-report recipes, use locmetry --guide TOPIC when you need
detail beyond these examples; topic names are listed under --guide."
    )
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub(crate) command: Option<Command>,
    /// Directory to measure, instead of --root [default: the current directory].
    #[arg(value_name = "DIR", conflicts_with = "root")]
    pub(crate) directory: Option<PathBuf>,
    /// Measure this root-relative folder and descendants, retaining project configuration and module context.
    #[arg(long, value_name = "DIR", help_heading = "Selection and policy")]
    pub(crate) subfolder: Option<PathBuf>,
    #[command(flatten)]
    pub(crate) root: RootArgs,
    #[command(flatten)]
    pub(crate) shared: SharedArgs,
    /// Add the folder table to single-root table output.
    #[arg(long = "show-folders", help_heading = "Presentation")]
    pub(crate) folders: bool,
    /// Show the folder table and split the largest folders within an N-row budget; no top-level folder's code is merged away. Table format only.
    #[arg(
        long,
        value_name = "N",
        allow_negative_numbers = true,
        help_heading = "Presentation"
    )]
    pub(crate) folder_rows: Option<std::num::NonZeroUsize>,
    /// Print a JSON Schema and exit: of a configuration file, which explains
    /// every setting and the classification order, of the JSON report, or of
    /// a workspace file, a net-change comparison, or a path explanation.
    /// This is a standalone query; do not combine it with measurement options.
    #[arg(long, value_enum, value_name = "TOPIC", help_heading = "Reference")]
    pub(crate) schema: Option<Schema>,
    /// Read a task guide. Use --help for syntax, --guide for workflows and
    /// caveats, and --schema for machine-readable contracts. Standalone query.
    #[arg(long, value_enum, value_name = "TOPIC", help_heading = "Reference")]
    pub(crate) guide: Option<Guide>,
    #[command(flatten)]
    pub(crate) styling: Styling,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    /// Measure several repositories or one project's folders as one virtual
    /// monorepo: members keep their repository's .locmetryrc, and paths are
    /// prefixed by the member's folder name.
    /// Defaults to a table on a terminal, a JSON headline in pipes.
    /// --output FILE saves only to that file, defaulting to full JSON; --format overrides the format.
    #[command(
        after_help = "Add --verbose to inspect classification, scope and coverage notes. For optional configuration, use locmetry --guide configure; for results, use locmetry --guide reports.",
        after_long_help = "Examples:
  Measure roots directly, keeping each member's policy:
    locmetry workspace --root web --root api --files --output workspace.json --print table
  Measure a workspace file:
    locmetry workspace --workspace-file repos.toml --files --output workspace.json --print table
  Export repository, category and language detail:
    locmetry workspace --workspace-file repos.toml --format markdown --markdown-depth 3 --sort name --output workspace.md
  Explain a member file:
    locmetry explain web/src/index.ts --workspace-file repos.toml --format json

repos.toml:
  version = 1
  roots = [\"web\", \"api\"]

Paths are relative to the workspace file; it is used only when --workspace-file names it.
Members need unique folder names and non-overlapping roots. Each scan uses
its own policy: each member's .locmetryrc and the personal defaults unless
--no-user-config is given. Set exclusions and overrides in member configuration;
workspace-wide --config/--exclude/--hidden/--respect-ignore-files are unsupported.
For one repository's components, a file can name project and folders instead of
roots: each folder is measured like --subfolder under the project's .locmetryrc.
Save outputs outside every member. JSON paths are member-prefixed; summary.members
holds per-repository totals. Workspace Test/prod is sum(Test Code) / sum(Production Code),
not an average of member ratios. --verbose exposes member coverage and policy findings.

Save before.json and after.json using the same workspace command around a change,
then run locmetry compare before.json after.json --format table. Member names match
snapshots; an absent member means it left the measured workspace, not that its files
were deleted. Use compare --member NAME to inspect one member and its subfolder changes.
See locmetry compare --help for full comparison recipes and limitations.
Use locmetry --schema workspace for the file contract, locmetry --guide configure
for broader policy, and locmetry --guide reports for interpretation."
    )]
    Workspace(WorkspaceArgs),
    /// Compare two saved full reports: net change is after minus before.
    /// Defaults to a table on a terminal, a JSON headline in pipes.
    /// --output FILE saves only to that file, defaulting to full JSON; --format overrides the format.
    #[command(
        after_help = "For baselines and interpretation, use locmetry --guide compare.",
        after_long_help = "For two full reports of the same mode and subfolder scope, execute the recipes below.
Inspect summary and findings before drawing conclusions. Additional guide/schema
reading is optional for this route; use the references below for uncovered semantics.

Examples:
  Capture a full baseline before changing files or policy:
    locmetry --root service --output before.json
  Capture the new state after the intended change:
    locmetry --root service --output after.json
  Inspect net change as a table:
    locmetry compare before.json after.json --format table
  Save the complete comparison:
    locmetry compare before.json after.json --output change.json
  Save a Markdown comparison:
    locmetry compare before.json after.json --format markdown --output change.md
  Capture a workspace baseline instead:
    locmetry workspace --workspace-file repos.toml --output before.json
  Capture the workspace after the intended change:
    locmetry workspace --workspace-file repos.toml --output after.json
  Inspect one workspace member and its immediate subfolders:
    locmetry compare before.json after.json --member api --format table

For --member, use two workspace reports and an exact member name present in either report.
The example assumes repos.toml includes api; unknown names and single-root inputs fail.
Check Member: NAME and Source total: grids and Test/prod cover only that member.
The folder table shows changed immediate subfolders; Markdown/JSON retain deeper detail.
JSON retains the member selector and member-prefixed paths. Input metadata, report-level
findings and partial status still cover the whole workspace inputs. Nested-folder
selection is unsupported. Nothing is rescanned.

Inputs must be full measurement JSON; headlines cannot be baselines. --files is
optional for aggregate comparison. Capture both inputs before comparing; rescanning
now cannot recover an earlier state. Compare two single roots or two workspaces,
with matching subfolder restrictions; use the same workspace form for after.json.
Measurement outputs must stay outside their roots. Comparison output must not
replace either input. --output is silent; use --format json for full JSON on stdout.
There is no --print option for compare.

Read summary.source/all_counted for counts; full JSON also has members, folders,
packages and findings. Human grids omit unchanged rows but totals include them.
Net change is after minus before, not Git additions/deletions, churn or per-file diff.
A departing workspace member does not prove source deletion. Recursive folder totals
include descendants. Inspect findings for changed policy, coverage and presence;
changed counts alone do not prove source-only growth. Exit 2 retains partial input
counts, exit 1 rejects invalid input, and exit 0 does not prove identical settings.
Use locmetry --schema comparison for fields or locmetry --guide compare for detail."
    )]
    Compare(CompareArgs),
    /// Explain selection and classification of existing root-relative files.
    /// JSON prints the complete explanation; there is no separate headline.
    #[command(
        after_help = "For selection and classification diagnosis, use locmetry --guide selection.",
        after_long_help = "Examples:
  Inspect a project-relative file:
    locmetry explain src/main.rs --root service
  Read a complete JSON explanation:
    locmetry explain src/main.rs --root service --format json
  Check several files, one JSON line each:
    locmetry explain src/main.rs tests/smoke.rs --root service --format json
  Diagnose a member file:
    locmetry explain web/src/index.ts --workspace-file repos.toml --format json
  Check an exclusion under a proposed scope:
    locmetry explain src/main.rs --root service --exclude 'src/**' --format json

PATH names an existing, exact-spelling root-relative file; workspace paths use
MEMBER/PATH while policy patterns remain relative to that member. Workspace mode
uses the member's own configuration and rejects single-root policy flags.
Several PATHs are explained in order; JSON then prints JSON Lines, one complete
explanation per line, and any target with unavailable counts makes the exit 2.
This inspects the current filesystem, not a saved report or missing/deleted file.
Redirect stdout to save JSON; explain has no --output option and no headline format.

Read result.state first: counted includes file counts and decision stage/patterns;
excluded names a scope exclusion; not_selected names an established ignore/hidden
cause or an unknown cause; unrecognized has no parser or counts. unread and
parse_failed mean unavailable counts (exit 2). symlink stops without following it.
Counted Other/unclaimed source is still counted. Zero is meaningful only with known
counts. Other successful states exit 0; invalid paths/configuration fail with exit 1.
Use locmetry --schema explanation for the tagged result, or locmetry --guide selection
for broader diagnosis. --schema and --guide are root-level standalone queries."
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
    /// Write only to FILE; - selects stdout. Defaults to full JSON; --format overrides it.
    #[arg(long, value_name = "FILE", help_heading = "Output and content")]
    pub(crate) output: Option<PathBuf>,
    #[command(flatten)]
    pub(crate) view: ViewArgs,
}

/// Views of tables and Markdown that measurement and comparison share.
#[derive(Clone, Copy, clap::Args)]
pub(crate) struct ViewArgs {
    /// Print only the tables, without titles, legends, notes, findings or the
    /// closing location and date lines. Table format only.
    #[arg(long = "tables-only", help_heading = "Presentation")]
    pub(crate) tables_only: bool,
    /// Show language variants such as TSX, JSX and C++ Header as rows of their own.
    /// By default tables and Markdown count them in their family's row, such as
    /// TypeScript, and name each fold. JSON keeps every backend language.
    #[arg(long = "split-languages", help_heading = "Presentation")]
    pub(crate) split_languages: bool,
}

impl ViewArgs {
    pub(crate) fn variants(self) -> LanguageVariants {
        match self.split_languages {
            true => LanguageVariants::Split,
            false => LanguageVariants::Folded,
        }
    }

    pub(crate) fn warn_ineffective(self, destination: &Destination, styling: Styling) {
        if self.tables_only {
            destination.warn_ineffective("--tables-only", &[Format::Table], styling);
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
    /// Member root; repeatable. Its folder name prefixes its paths in the report.
    #[arg(
        long = "root",
        value_name = "DIR",
        required_unless_present = "file",
        help_heading = "Selection and policy"
    )]
    pub(crate) roots: Vec<PathBuf>,
    /// Read the member roots from a TOML workspace file instead; its roots are
    /// relative to the file (see --schema workspace).
    #[arg(
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
    /// Write only to FILE, outside the scan root; - selects stdout.
    /// Defaults to full JSON; --format overrides it. File extensions do not select a format.
    /// Add --print table to also print the overview from the same run.
    #[arg(long, value_name = "FILE", help_heading = "Output and content")]
    pub(crate) output: Option<PathBuf>,
    /// With --output FILE, also print the same measurement to stdout in FORMAT,
    /// such as table for an overview or headline for compact JSON.
    #[arg(
        long,
        value_enum,
        value_name = "FORMAT",
        help_heading = "Output and content"
    )]
    pub(crate) print: Option<Format>,
    /// Include per-file records in full JSON, saved or printed; headline omits them.
    #[arg(long, help_heading = "Output and content")]
    pub(crate) files: bool,
    /// Add a source purpose/category table to saved or printed table output.
    #[arg(long = "show-categories", help_heading = "Presentation")]
    pub(crate) categories: bool,
    /// Include classification, scope and coverage notes in table output.
    #[arg(long, conflicts_with = "tables_only", help_heading = "Presentation")]
    pub(crate) verbose: bool,
    /// Add charts to saved or printed tables, beside them when they fit, otherwise below.
    /// Chart code share by purpose or comment density; selecting one table purpose
    /// uses comment density even with =purpose. Terminals show fitting bars automatically;
    /// COLUMNS overrides the width. Table format only.
    #[arg(
        long = "show-bars",
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
    /// Show only these purposes' source in tables; comma-separated.
    /// JSON and Markdown keep the complete measurement. Table format only.
    #[arg(
        long = "table-purpose",
        value_enum,
        value_delimiter = ',',
        value_name = "PURPOSES",
        help_heading = "Presentation"
    )]
    pub(crate) purpose: Vec<PurposeArg>,
    /// Show only these source languages in tables; comma-separated, in any case.
    /// A variant such as TSX selects its family's row unless --split-languages.
    /// JSON and Markdown keep the complete measurement. Table format only.
    #[arg(
        long = "table-language",
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
    /// Row order of table and Markdown output; the Markdown folder table keeps
    /// its path hierarchy, and the review list stays largest first.
    #[arg(
        long,
        value_enum,
        default_value = "code",
        help_heading = "Presentation"
    )]
    pub(crate) sort: Sort,
    /// Folder-breakdown depth in Markdown; 0 leaves no data rows. Relative to --subfolder;
    /// workspace members occupy level 1. Other evidence and JSON retain deeper paths.
    #[arg(
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
    /// Disable terminal colors, bold and dim text, including help and diagnostics.
    #[arg(long, global = true, help_heading = "Presentation")]
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
