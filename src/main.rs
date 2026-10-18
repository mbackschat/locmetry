mod guides;

use anyhow::{Context, Result};
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};
use guides::Guide;
use locmetry::contract::{self, MeasurementReport};
use locmetry::style::{plain, Role, Theme};
use locmetry::{
    breakdown, build_report, build_systems, git_submodules, module_shares, render_markdown,
    render_table_with_options, report_text, review, scan, scope_policy, submodule_shares,
    unclassified_folders, unused_entries, user_defaults_file, validate_output, write_report,
    BarKind, Bars, Coverage, FileRecord, Findings, MemberView, Module, ModuleShare, Policy,
    Purpose, RowOrder, ScopeOverrides, SubmoduleShare, TableOptions, BACKEND, CONFIG_FILE,
    DATA_LANGUAGES, DOCUMENTATION_LANGUAGES,
};
use std::collections::BTreeMap;
use std::{
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
    process::ExitCode,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
enum Format {
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
enum ExplainFormat {
    /// Human-readable tables and evidence.
    Table,
    /// Complete JSON explanation.
    Json,
    /// Markdown tables and evidence.
    Markdown,
}

#[derive(Clone, Copy, ValueEnum)]
enum PurposeArg {
    Production,
    Test,
    Unclassified,
}

impl PurposeArg {
    fn purpose(self) -> Purpose {
        match self {
            PurposeArg::Production => Purpose::Production,
            PurposeArg::Test => Purpose::Test,
            PurposeArg::Unclassified => Purpose::Unclassified,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum BarArg {
    /// Each row's Code split by purpose, scaled to the largest row.
    Purpose,
    /// Each row's Code against its Comments, across the full bar.
    Comments,
}

impl BarArg {
    fn kind(self) -> BarKind {
        match self {
            BarArg::Purpose => BarKind::Purpose,
            BarArg::Comments => BarKind::Comments,
        }
    }
}

#[derive(Clone, Copy, ValueEnum)]
enum Sort {
    /// Largest code first; ties alphabetical.
    Code,
    /// Alphabetical.
    Name,
}

#[derive(Clone, Copy, ValueEnum)]
enum Schema {
    /// The project configuration, `.locmetryrc`.
    Config,
    /// Personal defaults in ~/.config/locmetry/config.toml.
    UserConfig,
    /// The JSON headline and full report.
    Report,
    /// Net-change comparison headline and full document.
    Comparison,
    /// Selection, parser and classification evidence for one path.
    Explanation,
    /// A workspace file for `locmetry workspace --file`.
    Workspace,
}

/// Select the format when no explicit format was supplied.
///
/// Explicit exports default to complete JSON so saved baselines remain comparable;
/// ordinary pipes retain compact agent output (docs/PRODUCT.md, Output Routing).
fn default_format(stdout_is_terminal: bool, output_given: bool) -> Format {
    match (output_given, stdout_is_terminal) {
        (true, _) => Format::Json,
        (false, false) => Format::Headline,
        (false, true) => Format::Table,
    }
}

// A macro rather than a const because `concat!` accepts only literals.
macro_rules! root_after_help {
    () => {
        "Run locmetry for source counts by language. Add --folders for the folder table, or --verbose to inspect classification, scope and coverage notes. For optional project configuration, use locmetry --guide configure.\n\nCommand names are reserved. To measure a directory named compare, use ./compare or --root compare."
    };
}

#[derive(Parser)]
#[command(
    version,
    about = "Measure source code by language, folder, and purpose: production, test, or unclassified. Defaults to a table on a terminal, a JSON headline in pipes. --output FILE saves only to that file, defaulting to full JSON; --format overrides the format.",
    after_help = root_after_help!(),
    // The README "Common commands" table mirrors these examples; tests/readme.rs enforces it.
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
  Count production source only, with its comment density:
    locmetry --purpose production
  Inspect the configuration contract:
    locmetry --schema config

For workflows and saved-report recipes, use locmetry --guide TOPIC with a topic listed under --guide."
    )
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    /// Directory to measure, instead of --root [default: the current directory].
    #[arg(value_name = "DIR", conflicts_with = "root")]
    directory: Option<PathBuf>,
    /// Measure this root-relative folder and descendants, retaining project configuration and module context.
    #[arg(long, value_name = "DIR")]
    subfolder: Option<PathBuf>,
    /// Add the folder table to single-root table output.
    #[arg(long)]
    folders: bool,
    /// Show the folder table and split the largest folders within an N-row budget; no top-level folder's code is merged away. Table format only.
    #[arg(long, value_name = "N", allow_negative_numbers = true)]
    folder_rows: Option<std::num::NonZeroUsize>,
    #[command(flatten)]
    root: RootArgs,
    #[command(flatten)]
    shared: SharedArgs,
    /// Print a JSON Schema and exit: of a configuration file, which explains
    /// every setting and the classification order, of the JSON report, or of
    /// a workspace file, a net-change comparison, or a path explanation.
    /// This is a standalone query; do not combine it with measurement options.
    #[arg(long, value_enum)]
    schema: Option<Schema>,
    /// Read a task guide. Use --help for syntax, --guide for workflows and
    /// caveats, and --schema for machine-readable contracts. Standalone query.
    #[arg(long, value_enum)]
    guide: Option<Guide>,
}

#[derive(Subcommand)]
enum Command {
    /// Measure several repositories as one virtual monorepo: each root with its
    /// own .locmetryrc, merged with paths prefixed by the root's folder name.
    /// Defaults to a table on a terminal, a JSON headline in pipes.
    /// --output FILE saves only to that file, defaulting to full JSON; --format overrides the format.
    #[command(
        after_help = "Add --verbose to inspect classification, scope and coverage notes. For optional configuration, use locmetry --guide configure; for results, use locmetry --guide reports."
    )]
    Workspace(WorkspaceArgs),
    /// Compare two saved full reports: net change is after minus before.
    /// Defaults to a table on a terminal, a JSON headline in pipes.
    /// --output FILE saves only to that file, defaulting to full JSON; --format overrides the format.
    #[command(after_help = "For baselines and interpretation, use locmetry --guide compare.")]
    Compare(CompareArgs),
    /// Explain selection and classification of one existing root-relative file.
    /// JSON prints the complete explanation; there is no separate headline.
    #[command(
        after_help = "For selection and classification diagnosis, use locmetry --guide selection."
    )]
    Explain(ExplainArgs),
}

#[derive(clap::Args)]
struct ExplainArgs {
    /// Root-relative file, or MEMBER/PATH with --workspace.
    path: PathBuf,
    #[command(flatten)]
    root: RootArgs,
    /// Workspace file; use the selected member's own policy.
    #[arg(long, value_name = "FILE", conflicts_with_all = ["root", "config", "exclude", "hidden", "respect_ignore_files"])]
    workspace: Option<PathBuf>,
    /// Ignore personal defaults.
    #[arg(long)]
    no_user_config: bool,
    /// Stdout format [default: table on a terminal, json otherwise].
    #[arg(long, value_enum)]
    format: Option<ExplainFormat>,
}

#[derive(clap::Args)]
struct CompareArgs {
    /// Full JSON measurement before the change.
    before: PathBuf,
    /// Full JSON measurement after the change.
    after: PathBuf,
    /// Result format [default: json with --output; otherwise table on a terminal, headline in pipes].
    #[arg(long, value_enum)]
    format: Option<Format>,
    /// Write only to FILE; - selects stdout. Defaults to full JSON; --format overrides it.
    #[arg(long, value_name = "FILE")]
    output: Option<PathBuf>,
}

/// Settings of a single scan root, which a workspace leaves to each member's
/// own configuration (docs/PRODUCT.md, Workspace Mode).
#[derive(clap::Args)]
struct RootArgs {
    /// Scan root [default: the current directory].
    #[arg(long)]
    root: Option<PathBuf>,
    /// TOML configuration [default: .locmetryrc in the scan root, else built-in settings].
    #[arg(long)]
    config: Option<PathBuf>,
    /// Add a root-relative exclusion glob; repeatable.
    #[arg(long, value_name = "GLOB")]
    exclude: Vec<String>,
    /// Override whether hidden files and directories are measured.
    #[arg(long, value_name = "BOOL")]
    hidden: Option<bool>,
    /// Override whether ignore files limit the selection.
    #[arg(long, value_name = "BOOL")]
    respect_ignore_files: Option<bool>,
}

impl RootArgs {
    fn overrides(&self) -> ScopeOverrides {
        ScopeOverrides {
            hidden: self.hidden,
            respect_ignore_files: self.respect_ignore_files,
            exclude: self.exclude.clone(),
        }
    }
}

#[derive(clap::Args)]
struct WorkspaceArgs {
    /// Member root; repeatable. Its folder name prefixes its paths in the report.
    #[arg(long = "root", value_name = "DIR", required_unless_present = "file")]
    roots: Vec<PathBuf>,
    /// Read the member roots from a TOML workspace file instead; its roots are
    /// relative to the file (see --schema workspace).
    #[arg(long, value_name = "FILE", conflicts_with = "roots")]
    file: Option<PathBuf>,
    #[command(flatten)]
    shared: SharedArgs,
}

/// Settings that both modes accept: user defaults and the report's form.
#[derive(clap::Args)]
struct SharedArgs {
    /// Ignore user defaults in ~/.config/locmetry/config.toml.
    #[arg(long)]
    no_user_config: bool,
    /// Write only to FILE, outside the scan root; - selects stdout.
    /// Defaults to full JSON; --format overrides it. File extensions do not select a format.
    /// Add --print table to also print the overview from the same run.
    #[arg(long, value_name = "FILE")]
    output: Option<PathBuf>,
    /// With --output FILE, also print the same measurement to stdout in FORMAT,
    /// such as table for an overview or headline for compact JSON.
    #[arg(long, value_enum, value_name = "FORMAT")]
    print: Option<Format>,
    /// Include per-file records; requires full JSON (--format json or --output FILE).
    #[arg(long)]
    files: bool,
    /// Add a table of source code per purpose and category to the terminal table.
    #[arg(long)]
    categories: bool,
    /// Include classification, scope and coverage notes in table output.
    #[arg(long)]
    verbose: bool,
    /// Chart each table row beside the table when the width leaves room,
    /// otherwise below it: its share of Code by purpose, or with --bars=comments
    /// its Code against Comments. A terminal shows purpose bars automatically
    /// where they fit; COLUMNS overrides its width. Table format only.
    #[arg(
        long,
        value_enum,
        value_name = "KIND",
        num_args = 0..=1,
        require_equals = true,
        default_missing_value = "purpose"
    )]
    bars: Option<BarArg>,
    /// Omit the share bars a terminal shows automatically.
    #[arg(long, conflicts_with = "bars")]
    no_bars: bool,
    /// Count only these purposes' source in tables, such as production to drop
    /// tests; comma-separated. Table format only.
    #[arg(long, value_enum, value_delimiter = ',', value_name = "PURPOSES")]
    purpose: Vec<PurposeArg>,
    /// Result format [default: json with --output; otherwise table on a terminal, headline in pipes].
    #[arg(long, value_enum)]
    format: Option<Format>,
    /// Row order of table and Markdown output; the Markdown folder table keeps
    /// its path hierarchy, and the review list stays largest first.
    #[arg(long, value_enum, default_value = "code")]
    sort: Sort,
    /// Deepest Markdown folder level, relative to --subfolder when selected; JSON lists all.
    #[arg(long, default_value_t = 2, value_name = "N")]
    depth: usize,
    #[arg(skip)]
    depth_explicit: bool,
}

/// Whether human terminal output may use styling: only for a terminal, unless
/// `NO_COLOR` is set to anything nonempty (https://no-color.org) or the
/// terminal is `dumb`.
fn emphasis(stdout_is_terminal: bool, no_color: Option<String>, term: Option<String>) -> bool {
    stdout_is_terminal
        && no_color.is_none_or(|value| value.is_empty())
        && term.as_deref() != Some("dumb")
}

fn terminal_theme(is_terminal: bool) -> Theme {
    Theme::terminal(emphasis(
        is_terminal,
        std::env::var("NO_COLOR").ok(),
        std::env::var("TERM").ok(),
    ))
}

/// Result of a run that produced a report.
enum Outcome {
    Complete,
    /// Counts are unavailable; the message names read and parsing failures.
    Partial(String),
}

/// One root measured as a regular run measures it.
struct Measurement {
    root: PathBuf,
    name: String,
    config_path: Option<PathBuf>,
    policy: Policy,
    coverage: Coverage,
    inaccurate: bool,
    rows: Vec<FileRecord>,
    modules: Vec<ModuleShare>,
    submodules: Vec<SubmoduleShare>,
    unused_entries: Vec<String>,
    unused_exclusions: Vec<String>,
}

/// A root with its loaded policy, ready to scan; preparing first lets output
/// paths be checked before anything is scanned.
struct Prepared {
    root: PathBuf,
    name: String,
    config_path: Option<PathBuf>,
    policy: Policy,
}

/// Loads the policy for `root`; `workspace_path` names a workspace file,
/// which, like the configurations, is excluded when it lies inside the root.
fn prepare(
    root: &Path,
    config: Option<PathBuf>,
    user_path: Option<&Path>,
    workspace_path: Option<&Path>,
    overrides: &ScopeOverrides,
) -> Result<Prepared> {
    let config_path = config.or_else(|| {
        let discovered = root.join(CONFIG_FILE);
        discovered.is_file().then_some(discovered)
    });
    let mut policy = Policy::load_layers(config_path.as_deref(), user_path)?;
    policy.override_scope(overrides)?;
    let root = root.canonicalize().context("Scan root does not exist")?;
    for path in [config_path.as_deref(), user_path, workspace_path]
        .into_iter()
        .flatten()
    {
        policy.exclude_configuration(&root, path)?;
    }
    let root_text = root.to_str().context("Scan root must be valid UTF-8")?;
    let name = root.file_name().map_or_else(
        || root_text.to_owned(),
        |name| name.to_string_lossy().into(),
    );
    Ok(Prepared {
        root,
        name,
        config_path,
        policy,
    })
}

fn measure(prepared: Prepared, subfolder: Option<&str>) -> Result<Measurement> {
    let Prepared {
        root,
        name,
        config_path,
        mut policy,
    } = prepared;
    let scan = match subfolder {
        Some(folder) => locmetry::scan_subfolder(&root, &policy, folder)?,
        None => scan(&root, &policy)?,
    };
    policy.adopt_modules(&scan.modules)?;
    let rows = scan.records(&policy)?;
    let mut modules = module_shares(policy.modules(), &rows);
    if subfolder.is_some() {
        modules.retain(|module| module.files > 0);
    }
    Ok(Measurement {
        modules,
        submodules: submodule_shares(&git_submodules(&root), &rows),
        unused_entries: unused_entries(policy.entry_ids(), &rows),
        unused_exclusions: scan.unused_exclusions,
        coverage: scan.coverage,
        inaccurate: scan.inaccurate,
        rows,
        root,
        name,
        config_path,
        policy,
    })
}

/// What a report is built from: one measurement, or a workspace merge with
/// paths prefixed by member name.
struct Results {
    project: String,
    rows: Vec<FileRecord>,
    coverage: Coverage,
    inaccurate: bool,
    modules: Vec<Module>,
    module_shares: Vec<ModuleShare>,
    submodules: Vec<SubmoduleShare>,
    /// Configuration findings as human notes name them.
    unused_entries: Vec<String>,
    unused_exclusions: Vec<String>,
    /// Workspace members in name order; empty for a single root.
    members: Vec<MemberView>,
}

/// The result destination and format, checked before measuring so a
/// misuse fails fast.
struct Destination<'a> {
    path: Option<&'a Path>,
    format: Format,
    /// A second view of the same measurement for stdout, beside a saved file.
    print: Option<Format>,
    folders: bool,
    folder_rows: Option<usize>,
}

fn destination(shared: &SharedArgs) -> Result<Destination<'_>> {
    let mut destination = output_destination(&shared.output, shared.format);
    anyhow::ensure!(
        !shared.files || destination.format == Format::Json,
        "--files adds per-file records only to full JSON; use --format json (or --output FILE without a conflicting --format)"
    );
    anyhow::ensure!(
        shared.print.is_none() || destination.path.is_some(),
        "--print requires --output FILE; without a saved file, stdout already receives the result"
    );
    destination.print = shared.print;
    Ok(destination)
}

fn output_destination(output: &Option<PathBuf>, format: Option<Format>) -> Destination<'_> {
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
    fn theme(&self) -> Theme {
        terminal_theme(self.path.is_none() && io::stdout().is_terminal())
    }

    /// Whether the saved result or the printed view uses `format`, so the
    /// options shaping that format take effect.
    fn shows(&self, format: Format) -> bool {
        self.format == format || self.print == Some(format)
    }

    fn write(&self, text: &str) -> Result<()> {
        match self.path {
            Some(path) => write_report(path, text),
            None => Ok(io::stdout().lock().write_all(text.as_bytes())?),
        }
    }
}

/// Columns available to share bars: `COLUMNS` when set, as for other terminal
/// programs, then the width of a stdout terminal, else a conventional 80.
fn bar_columns(stdout: bool) -> usize {
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

fn warn_ineffective_options(shared: &SharedArgs, destination: &Destination) {
    if !destination.shows(Format::Table) {
        for (given, option) in [
            (shared.verbose, "--verbose"),
            (shared.bars.is_some(), "--bars"),
            (!shared.purpose.is_empty(), "--purpose"),
            (shared.no_bars, "--no-bars"),
            (shared.categories, "--categories"),
        ] {
            if given {
                stderr_message(
                    Role::Warning,
                    format_args!("{option} has no effect here; it applies only to table output"),
                );
            }
        }
    }
    if shared.depth_explicit && !destination.shows(Format::Markdown) {
        stderr_message(
            Role::Warning,
            "--depth has no effect here; it applies only to Markdown output",
        );
    }
}

fn user_defaults(no_user_config: bool) -> Option<PathBuf> {
    user_defaults_file().filter(|path| !no_user_config && path.is_file())
}

fn run(cli: Cli) -> Result<Outcome> {
    if let Some(guide) = cli.guide {
        let (path, text) = guide.document();
        let mut stdout = io::stdout().lock();
        writeln!(
            stdout,
            "Source: {path}; locmetry {}\n",
            env!("CARGO_PKG_VERSION")
        )?;
        stdout.write_all(text.as_bytes())?;
        return Ok(Outcome::Complete);
    }
    if let Some(schema) = cli.schema {
        let schema = match schema {
            Schema::Config => locmetry::config_schema(),
            Schema::UserConfig => locmetry::user_config_schema(),
            Schema::Report => locmetry::report_schema(),
            Schema::Comparison => locmetry::compare::schema(),
            Schema::Explanation => locmetry::explain::schema(),
            Schema::Workspace => locmetry::workspace_schema(),
        };
        println!("{}", serde_json::to_string_pretty(&schema)?);
        return Ok(Outcome::Complete);
    }
    match cli.command {
        Some(Command::Workspace(workspace)) => run_workspace(&workspace),
        Some(Command::Compare(args)) => run_compare(&args),
        Some(Command::Explain(args)) => run_explain(&args),
        None => {
            let mut root = cli.root;
            root.root = root.root.or(cli.directory);
            run_root(
                &root,
                &cli.shared,
                cli.subfolder.as_deref(),
                cli.folders,
                cli.folder_rows.map(|n| n.get()),
            )
        }
    }
}

fn run_explain(args: &ExplainArgs) -> Result<Outcome> {
    let user_path = user_defaults(args.no_user_config);
    let workspace = args
        .workspace
        .as_deref()
        .map(Path::canonicalize)
        .transpose()
        .context("Cannot read workspace file")?;
    let (root, target, member) = match &workspace {
        Some(file) => {
            let members = workspace_members(&locmetry::workspace_roots(file)?)?;
            let mut components = args.path.components();
            let name = match components.next() {
                Some(std::path::Component::Normal(name)) => {
                    name.to_str().context("Member name must be UTF-8")?
                }
                _ => anyhow::bail!("Workspace query must be MEMBER/PATH"),
            };
            let (_, root) = members
                .into_iter()
                .find(|(member, _)| member == name)
                .context("Unknown workspace member")?;
            (root, components.as_path().to_owned(), Some(name.to_owned()))
        }
        None => (
            args.root.root.clone().unwrap_or_else(|| PathBuf::from(".")),
            args.path.clone(),
            None,
        ),
    };
    let overrides = args.root.overrides();
    let mut prepared = prepare(
        &root,
        args.root.config.clone(),
        user_path.as_deref(),
        workspace.as_deref(),
        &overrides,
    )?;
    let (path, result) = locmetry::explain::inspect(&prepared.root, &target, &mut prepared.policy)?;
    let mut explanation = locmetry::explain::Explanation {
        document_type: locmetry::explain::DocumentType::Explanation,
        schema_version: contract::Version::V1,
        root: prepared.root.to_string_lossy().into(),
        path,
        member: None,
        config_path: prepared.config_path.map(|p| p.to_string_lossy().into()),
        user_config_path: user_path.map(|p| p.to_string_lossy().into()),
        scope: scope_policy(&prepared.policy),
        result,
    };
    if let Some(member) = member {
        explanation.qualify(member);
    }
    match args
        .format
        .unwrap_or_else(|| match io::stdout().is_terminal() {
            true => ExplainFormat::Table,
            false => ExplainFormat::Json,
        }) {
        ExplainFormat::Json => println!("{}", serde_json::to_string_pretty(&explanation)?),
        ExplainFormat::Table => print!(
            "{}",
            explanation.render_with_theme(false, terminal_theme(io::stdout().is_terminal()))
        ),
        ExplainFormat::Markdown => print!("{}", explanation.render(true)),
    }
    Ok(if explanation.partial() {
        Outcome::Partial(format!("Counts unavailable for {}", explanation.path))
    } else {
        Outcome::Complete
    })
}

fn run_compare(args: &CompareArgs) -> Result<Outcome> {
    let destination = output_destination(&args.output, args.format);
    let before = locmetry::compare::read_input(&args.before)?;
    let after = locmetry::compare::read_input(&args.after)?;
    if let Some(path) = destination.path {
        locmetry::compare::validate_destination(path, [&args.before, &args.after])?;
    }
    let result = locmetry::compare::compare(
        before,
        after,
        args.before.to_string_lossy().into(),
        args.after.to_string_lossy().into(),
    )?;
    let partial = result.partial();
    let text = match destination.format {
        Format::Json => report_text(&serde_json::to_value(&result)?)?,
        Format::Headline => format!("{}\n", serde_json::to_string(&result.headline())?),
        Format::Table => locmetry::compare::render_table_with_theme(&result, destination.theme()),
        Format::Markdown => locmetry::compare::render_markdown(&result),
    };
    destination.write(&text)?;
    Ok(if partial {
        Outcome::Partial(
            "Net change uses partial input measurements; inspect coverage findings.".into(),
        )
    } else {
        Outcome::Complete
    })
}

fn run_root(
    args: &RootArgs,
    shared: &SharedArgs,
    subfolder: Option<&Path>,
    folders: bool,
    folder_rows: Option<usize>,
) -> Result<Outcome> {
    let root = args.root.as_deref().unwrap_or(Path::new("."));
    let mut destination = destination(shared)?;
    if folder_rows.is_some() {
        anyhow::ensure!(
            destination.shows(Format::Table),
            "--folder-rows requires table output; add --format table or --print table"
        );
    }
    destination.folders = folders;
    destination.folder_rows = folder_rows;
    if folders && !destination.shows(Format::Table) {
        stderr_message(
            Role::Warning,
            "--folders has no effect here; it applies only to table output",
        );
    }
    warn_ineffective_options(shared, &destination);
    let user_path = user_defaults(shared.no_user_config);
    let overrides = args.overrides();
    let measured_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let prepared = prepare(
        root,
        args.config.clone(),
        user_path.as_deref(),
        None,
        &overrides,
    )?;
    if let Some(path) = destination.path {
        validate_output(path, &prepared.root, prepared.config_path.as_deref())?;
    }
    let subfolder = subfolder
        .map(|path| locmetry::subfolder_path(&prepared.root, path))
        .transpose()?
        .flatten();
    if let Some(folder) = &subfolder {
        let inactive = prepared.root.join(folder).join(CONFIG_FILE);
        if inactive.is_file()
            && prepared
                .config_path
                .as_ref()
                .and_then(|p| p.canonicalize().ok())
                != inactive.canonicalize().ok()
        {
            let project_config = prepared
                .config_path
                .as_ref()
                .map(std::fs::canonicalize)
                .transpose()?;
            stderr_message(Role::Warning, format_args!(
                "{} is inactive; project configuration: {}. --subfolder retains project policy.",
                inactive.display(),
                project_config.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "none (built-in settings)".into())
            ));
        }
    }
    let measurement = measure(prepared, subfolder.as_deref())?;
    let mut scope = scope_policy(&measurement.policy);
    scope.subfolder = subfolder;

    let location = Location::Root {
        root: measurement.root,
        scope,
        config_path: measurement.config_path,
        user_path,
        config: Box::new(measurement.policy.snapshot().clone()),
    };
    let results = Results {
        project: measurement.name,
        modules: measurement.policy.modules().to_vec(),
        module_shares: measurement.modules,
        submodules: measurement.submodules,
        unused_entries: measurement.unused_entries,
        unused_exclusions: measurement.unused_exclusions,
        coverage: measurement.coverage,
        inaccurate: measurement.inaccurate,
        rows: measurement.rows,
        members: Vec::new(),
    };
    report(shared, &destination, results, location, measured_at)
}

/// Canonical member roots sorted by name, after the checks that make a merge
/// well defined: unique names, no root twice, and no member inside another.
fn workspace_members(roots: &[PathBuf]) -> Result<Vec<(String, PathBuf)>> {
    let mut members = roots
        .iter()
        .map(|given| {
            let root = given.canonicalize().with_context(|| {
                format!("Workspace member {}: root does not exist", given.display())
            })?;
            let name = root
                .file_name()
                .and_then(|name| name.to_str())
                .with_context(|| {
                    format!("Workspace member {} has no folder name", given.display())
                })?
                .to_owned();
            Ok((name, root))
        })
        .collect::<Result<Vec<_>>>()?;
    members.sort();
    for pair in members.windows(2) {
        let [(name, root), (other_name, other)] = pair else {
            unreachable!("windows of two")
        };
        anyhow::ensure!(
            root != other,
            "Workspace root {} is given twice",
            root.display()
        );
        anyhow::ensure!(
            name != other_name,
            "Workspace members {} and {} share the name {name}",
            root.display(),
            other.display()
        );
    }
    for (_, root) in &members {
        if let Some((_, outer)) = members
            .iter()
            .find(|(_, outer)| outer != root && root.starts_with(outer))
        {
            anyhow::bail!(
                "Workspace member {} lies inside member {}",
                root.display(),
                outer.display()
            );
        }
    }
    Ok(members)
}
fn run_workspace(args: &WorkspaceArgs) -> Result<Outcome> {
    let shared = &args.shared;
    let destination = destination(shared)?;
    warn_ineffective_options(shared, &destination);
    let workspace_path = args
        .file
        .as_deref()
        .map(|file| {
            file.canonicalize()
                .with_context(|| format!("Cannot read workspace file {}", file.display()))
        })
        .transpose()?;
    let roots = match &workspace_path {
        Some(file) => locmetry::workspace_roots(file)?,
        None => args.roots.clone(),
    };
    let members = workspace_members(&roots)?;
    if let Some(path) = destination.path {
        for (_, root) in &members {
            validate_output(path, root, workspace_path.as_deref())?;
        }
    }
    let user_path = user_defaults(shared.no_user_config);
    let measured_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    // Members run in parallel; collecting in name order keeps the merge and
    // the reported error deterministic whichever thread finishes first.
    let measurements = std::thread::scope(|threads| {
        let handles: Vec<_> = members
            .iter()
            .map(|(name, root)| {
                let (user_path, workspace_path) = (user_path.as_deref(), workspace_path.as_deref());
                let handle = threads.spawn(move || {
                    let overrides = ScopeOverrides::default();
                    measure(
                        prepare(root, None, user_path, workspace_path, &overrides)?,
                        None,
                    )
                });
                (name, handle)
            })
            .collect();
        handles
            .into_iter()
            .map(|(name, handle)| {
                handle
                    .join()
                    .map_err(|_| anyhow::anyhow!("Workspace member {name} panicked"))?
                    .with_context(|| format!("Workspace member {name}"))
            })
            .collect::<Result<Vec<_>>>()
    })?;
    let mut results = Results {
        project: members
            .iter()
            .map(|(name, _)| name.as_str())
            .collect::<Vec<_>>()
            .join(" + "),
        rows: Vec::new(),
        coverage: Coverage::default(),
        inaccurate: false,
        modules: Vec::new(),
        module_shares: Vec::new(),
        submodules: Vec::new(),
        unused_entries: Vec::new(),
        unused_exclusions: Vec::new(),
        members: Vec::new(),
    };
    let mut records = Vec::new();
    for measurement in measurements {
        let name = &measurement.name;
        let prefixed = |path: &str| locmetry::qualify(name, path);
        records.push(contract::Member {
            name: name.clone(),
            root: measurement.root.to_string_lossy().into(),
            config_path: measurement
                .config_path
                .as_ref()
                .map(|p| p.to_string_lossy().into()),
            scope: scope_policy(&measurement.policy),
            unused_entries: measurement.unused_entries.clone(),
            unused_exclusions: measurement.unused_exclusions.clone(),
            coverage: (&measurement.coverage).into(),
            config: Some(measurement.policy.snapshot().clone()),
        });
        let unclaimed = measurement.rows.iter().filter(|row| row.needs_review());
        results.members.push(MemberView {
            name: name.clone(),
            unclaimed_files: unclaimed.clone().count() as u64,
            unclaimed_code: unclaimed.map(|row| row.counts.code).sum(),
            unread_files: measurement.coverage.unread.len(),
            parse_failed_files: measurement.coverage.parse_failures.len(),
        });
        results
            .rows
            .extend(measurement.rows.into_iter().map(|mut row| {
                row.path = prefixed(&row.path);
                row
            }));
        results.coverage.merge(&measurement.coverage, prefixed);
        results.inaccurate |= measurement.inaccurate;
        results
            .modules
            .extend(measurement.policy.modules().iter().map(|module| Module {
                path: prefixed(&module.path),
                ..module.clone()
            }));
        results
            .module_shares
            .extend(measurement.modules.into_iter().map(|mut share| {
                share.path = prefixed(&share.path);
                share
            }));
        results
            .submodules
            .extend(measurement.submodules.into_iter().map(|mut share| {
                share.path = prefixed(&share.path);
                share
            }));
        results.unused_entries.extend(
            measurement
                .unused_entries
                .iter()
                .map(|id| format!("{id} ({name})")),
        );
        results.unused_exclusions.extend(
            measurement
                .unused_exclusions
                .iter()
                .map(|pattern| format!("{pattern} ({name})")),
        );
    }
    // Prefixes keep each member's records together but can reorder them
    // against plain path order (`a-b/` sorts before `a/`), so sort once more.
    results.rows.sort_by(|a, b| a.path.cmp(&b.path));
    results.coverage.unread.sort();
    results
        .coverage
        .parse_failures
        .sort_by(|a, b| a.path.cmp(&b.path));
    results.coverage.unrecognized.sort();
    let location = Location::Workspace {
        members: records,
        workspace_path,
        user_path,
    };
    report(shared, &destination, results, location, measured_at)
}

enum Location {
    Root {
        root: PathBuf,
        scope: contract::Selection,
        config_path: Option<PathBuf>,
        user_path: Option<PathBuf>,
        config: Box<locmetry::EffectiveConfig>,
    },
    Workspace {
        members: Vec<contract::Member>,
        workspace_path: Option<PathBuf>,
        user_path: Option<PathBuf>,
    },
}

/// Bars: forced by `--bars`, refused by `--no-bars`, and otherwise purpose
/// bars only on a terminal and only where they fit, so files and pipes stay stable.
fn bar_mode(bars: Option<BarKind>, no_bars: bool, terminal: bool, columns: usize) -> Bars {
    match (bars, no_bars, terminal) {
        (Some(kind), _, _) => Bars::Always(columns, kind),
        (None, true, _) | (None, false, false) => Bars::Off,
        (None, false, true) => Bars::Fit(columns),
    }
}

/// The purposes tables count, in `Purpose::ALL` order; all without a selection.
fn selected_purposes(selection: &[PurposeArg]) -> Vec<Purpose> {
    Purpose::ALL
        .into_iter()
        .filter(|purpose| {
            selection.is_empty() || selection.iter().any(|arg| arg.purpose() == *purpose)
        })
        .collect()
}

/// The footer line naming what a human report measured: the root and any
/// subfolder, or every workspace root. Markdown sets the paths as code.
fn measured_roots(location: &Location, markdown: bool) -> String {
    let path = |path: &str| match markdown {
        true => format!("`{}`", locmetry::style::plain(path)),
        false => locmetry::style::plain(path),
    };
    match location {
        Location::Root { root, scope, .. } => {
            let root = path(&root.to_string_lossy());
            match &scope.subfolder {
                Some(subfolder) => format!("Root: {root}, subfolder {}", path(subfolder)),
                None => format!("Root: {root}"),
            }
        }
        Location::Workspace { members, .. } => {
            let roots: Vec<_> = members.iter().map(|member| path(&member.root)).collect();
            format!("Roots: {}", roots.join(", "))
        }
    }
}

/// Writes the report, headline, or human output for `results`.
///
/// `location` holds the mode's own keys: a single root's root, scope, and
/// configuration, or a workspace's members. Configuration findings enter the
/// summary only for a single root, because a workspace's entry IDs and
/// exclusion patterns are relative to one member.
fn report(
    shared: &SharedArgs,
    destination: &Destination,
    results: Results,
    location: Location,
    measured_at: u64,
) -> Result<Outcome> {
    // Workspace members record their own unused entries; only a single root
    // reports them in the summary.
    let summary_findings = matches!(location, Location::Root { .. });
    let Results {
        project,
        rows,
        coverage,
        inaccurate,
        modules,
        module_shares,
        submodules,
        unused_entries,
        unused_exclusions,
        members,
    } = results;
    let format = destination.format;
    let summary = build_report(&rows);
    let subfolder = match &location {
        Location::Root { scope, .. } => scope.subfolder.as_deref(),
        Location::Workspace { .. } => None,
    };
    let findings = Findings {
        subfolder,
        coverage: &coverage,
        modules: &module_shares,
        submodules: &submodules,
        unused_entries: &unused_entries,
        unused_exclusions: &unused_exclusions,
        members: &members,
    };
    let order = match shared.sort {
        Sort::Code => RowOrder::Code,
        Sort::Name => RowOrder::Name,
    };
    let purposes = selected_purposes(&shared.purpose);
    // Table and Markdown views; `stdout` decides styling and automatic bars.
    let human = |format: Format, stdout: bool| {
        let terminal = stdout && io::stdout().is_terminal();
        let mut human = match format {
            Format::Table => render_table_with_options(
                &summary,
                &rows,
                &findings,
                TableOptions {
                    categories: shared.categories,
                    folders: destination.folders,
                    verbose: shared.verbose,
                    order,
                    theme: terminal_theme(terminal),
                    folder_rows: destination.folder_rows,
                    bars: bar_mode(
                        shared.bars.map(BarArg::kind),
                        shared.no_bars,
                        terminal,
                        bar_columns(stdout),
                    ),
                    purposes: purposes.clone(),
                },
            ),
            Format::Markdown => {
                render_markdown(&project, &summary, &rows, &findings, shared.depth, order)
            }
            Format::Json | Format::Headline => return String::new(),
        };
        if inaccurate {
            let theme = match format {
                Format::Markdown => Theme::MARKDOWN,
                _ => terminal_theme(terminal),
            };
            human.push_str(&format!(
                "\n{}; inspect the JSON metadata.\n",
                theme.paint(Role::Warning, "Backend reported incomplete counts")
            ));
        }
        // `Measured at:` stays the documented final line; the roots precede it.
        let separator = match format {
            Format::Markdown => "\n\n",
            _ => "\n",
        };
        let mut context = measured_roots(&location, format == Format::Markdown);
        // Only tables filter by purpose, and only a strict subset filters, as
        // the table renderer decides; the footer keeps that selection visible.
        if format == Format::Table && purposes.len() < Purpose::ALL.len() {
            let names: Vec<String> = purposes.iter().map(Purpose::to_string).collect();
            context += &format!("; purposes: {}", names.join(", "));
        }
        human.push_str(&format!(
            "\n{context}{separator}Measured at: {}\n",
            locmetry::measurement_time(measured_at)
        ));
        human
    };
    let saved_human = human(format, destination.path.is_none());
    let printed = destination.print.map(|print| (print, human(print, true)));
    // The document is built only for JSON views; a headline omits the review,
    // breakdown and file records, so only full JSON derives them.
    let document = |full: bool| {
        let mut member_totals = BTreeMap::new();
        for member in &members {
            member_totals.insert(
                member.name.clone(),
                summary
                    .folders
                    .get(&member.name)
                    .map(|f| f.recursive.clone())
                    .unwrap_or_default(),
            );
        }
        let summary_document = contract::ReportSummary {
            top_folders: summary.top_folders(),
            totals: summary.totals,
            members: member_totals,
            unclassified_folders: Some(unclassified_folders(&rows)),
            build_modules: build_systems(&module_shares)
                .into_iter()
                .map(|(k, v)| (k.into(), v))
                .collect(),
            submodules,
            unused_entries: if summary_findings {
                unused_entries
            } else {
                Vec::new()
            },
            unused_exclusions: if summary_findings {
                unused_exclusions
            } else {
                Vec::new()
            },
        };
        let mut metadata = contract::Metadata {
            measured_at_unix_seconds: measured_at,
            locmetry_version: env!("CARGO_PKG_VERSION").into(),
            backend: BACKEND.into(),
            backend_inaccurate: inaccurate,
            backend_configuration: locmetry::BACKEND_CONFIGURATION.into(),
            kinds: Some(contract::Kinds {
                default: locmetry::Kind::Source,
                data: DATA_LANGUAGES.iter().map(|l| l.name().into()).collect(),
                documentation: DOCUMENTATION_LANGUAGES
                    .iter()
                    .map(|l| l.name().into())
                    .collect(),
            }),
            coverage: contract::Coverage::from(&coverage),
            modules: Some(
                modules
                    .iter()
                    .map(|m| contract::Module {
                        path: m.path.clone(),
                        detector: m.detector.into(),
                    })
                    .collect(),
            ),
            root: None,
            scope: None,
            config_path: None,
            config: None,
            members: None,
            workspace_path: None,
            user_config_path: None,
        };
        match location {
            Location::Root {
                root,
                scope,
                config_path,
                user_path,
                config,
            } => {
                metadata.root = Some(root.to_string_lossy().into());
                metadata.scope = Some(scope);
                metadata.config_path = Some(config_path.map(|p| p.to_string_lossy().into()));
                metadata.config = Some(*config);
                metadata.user_config_path = Some(user_path.map(|p| p.to_string_lossy().into()));
            }
            Location::Workspace {
                members,
                workspace_path,
                user_path,
            } => {
                metadata.members = Some(members);
                metadata.workspace_path = workspace_path.map(|p| p.to_string_lossy().into());
                metadata.user_config_path = Some(user_path.map(|p| p.to_string_lossy().into()));
            }
        }
        MeasurementReport {
            schema_version: contract::Version::V1,
            project,
            metadata,
            summary: summary_document,
            review: full.then(|| review(&rows)),
            breakdown: full.then(|| breakdown(&rows)),
            files: (full && shared.files).then(|| rows.iter().map(contract::File::from).collect()),
            report: None,
        }
    };
    let shows = |format| destination.shows(format);
    let document =
        (shows(Format::Json) || shows(Format::Headline)).then(|| document(shows(Format::Json)));
    let text = |format: Format, human: String| -> Result<String> {
        let document = || document.as_ref().expect("built for JSON views");
        Ok(match format {
            Format::Json => report_text(&serde_json::to_value(document())?)?,
            Format::Headline => format!("{}\n", serde_json::to_string(&document().headline())?),
            Format::Table | Format::Markdown => human,
        })
    };
    destination.write(&text(format, saved_human)?)?;
    if let Some((print, human)) = printed {
        io::stdout()
            .lock()
            .write_all(text(print, human)?.as_bytes())?;
    }
    if inaccurate {
        let mut failures: Vec<_> = coverage
            .unread
            .iter()
            .map(|path| format!("{path}: unreadable"))
            .chain(
                coverage
                    .parse_failures
                    .iter()
                    .map(|failure| failure.to_string()),
            )
            .take(10)
            .collect();
        let count = coverage.unread.len() + coverage.parse_failures.len();
        if count > failures.len() {
            failures.push(format!("and {} more", count - failures.len()));
        }
        if failures.is_empty() {
            failures.push("backend did not identify affected paths".into());
        }
        return Ok(Outcome::Partial(format!(
            "Backend reported incomplete counts; output contains partial results. Unavailable files: {}",
            failures.join(", ")
        )));
    }
    Ok(Outcome::Complete)
}

fn validate_query(
    command: &mut clap::Command,
    matches: &clap::ArgMatches,
    query: &str,
) -> clap::error::Result<()> {
    if let Some(name) = matches.subcommand_name() {
        return Err(command.error(
            clap::error::ErrorKind::ArgumentConflict,
            format!("--{query} is a standalone query and cannot be combined with '{name}'"),
        ));
    }
    // Only concrete, explicit arguments matter: flattened groups inherit sources,
    // and default sort/depth values must not invalidate standalone queries.
    let conflict = command
        .get_arguments()
        .find(|arg| {
            arg.get_id().as_str() != query
                && matches.value_source(arg.get_id().as_str())
                    == Some(clap::parser::ValueSource::CommandLine)
        })
        .map(|arg| match arg.get_long() {
            Some(long) => format!("--{long}"),
            None => "DIR".to_owned(),
        });
    if let Some(argument) = conflict {
        return Err(command.error(
            clap::error::ErrorKind::ArgumentConflict,
            format!("--{query} is a standalone query and cannot be combined with {argument}"),
        ));
    }
    Ok(())
}

fn parse_cli() -> clap::error::Result<Cli> {
    let mut command = Cli::command();
    let matches = command.try_get_matches_from_mut(std::env::args_os())?;
    for query in ["schema", "guide"] {
        if matches.value_source(query) == Some(clap::parser::ValueSource::CommandLine) {
            validate_query(&mut command, &matches, query)?;
        }
    }
    // Clap's args_conflicts_with_subcommands disables command recognition after
    // root flags, allowing a reserved name to become DIR. Validate after parsing
    // instead so command precedence is independent of preceding root options.
    if let Some(name) = matches.subcommand_name() {
        if matches.contains_id("folder_rows") {
            return Err(command.error(
                clap::error::ErrorKind::ArgumentConflict,
                "--folder-rows is only available for single-root measurement, not subcommands",
            ));
        }
        if matches.args_present() {
            return Err(command.error(
                clap::error::ErrorKind::ArgumentConflict,
                format!("Root measurement arguments cannot be combined with the '{name}' subcommand; put that command's options after its name"),
            ));
        }
    }
    let mut cli = Cli::from_arg_matches(&matches)?;
    let shared_matches = match &mut cli.command {
        Some(Command::Workspace(args)) => Some((
            &mut args.shared,
            matches
                .subcommand_matches("workspace")
                .expect("parsed workspace"),
        )),
        None => Some((&mut cli.shared, &matches)),
        _ => None,
    };
    if let Some((shared, matches)) = shared_matches {
        shared.depth_explicit =
            matches.value_source("depth") == Some(clap::parser::ValueSource::CommandLine);
    }
    Ok(cli)
}

fn stderr_message(role: Role, message: impl std::fmt::Display) {
    eprintln!(
        "{} {}",
        terminal_theme(io::stderr().is_terminal()).paint(role, "locmetry:"),
        plain(&message.to_string())
    );
}

fn main() -> ExitCode {
    // clap exits usage errors with 2, which is reserved for partial reports.
    let cli = match parse_cli() {
        Ok(cli) => cli,
        Err(error) => {
            let _ = error.print();
            return if error.use_stderr() {
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            };
        }
    };
    match run(cli) {
        Ok(Outcome::Complete) => ExitCode::SUCCESS,
        Ok(Outcome::Partial(message)) => {
            stderr_message(Role::Warning, message);
            ExitCode::from(2)
        }
        Err(error) => {
            stderr_message(Role::Error, format_args!("{error:#}"));
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_help_layout(args: &[&str]) {
        for width in [60, 80, 100] {
            let error = Cli::command()
                .term_width(width)
                .color(clap::ColorChoice::Never)
                .try_get_matches_from(args)
                .unwrap_err();
            assert_eq!(error.kind(), clap::error::ErrorKind::DisplayHelp);
            let help = error.to_string();
            for line in help.lines() {
                assert!(
                    line.chars().count() <= width,
                    "{args:?}, width {width}: overlong help line {line:?}"
                );
            }
            for section in ["Commands:\n", "Arguments:\n", "Options:\n"] {
                if let Some((_, content)) = help.split_once(section) {
                    let content = content.split("\n\n").next().unwrap();
                    assert!(
                        content.lines().all(|line| line.starts_with("  ")),
                        "{args:?}, width {width}: unindented help in {section}{content}"
                    );
                }
            }
        }
    }

    #[test]
    fn root_help_wraps_and_indents_descriptions() {
        for flag in ["-h", "--help"] {
            assert_help_layout(&["locmetry", flag]);
        }
    }

    #[test]
    fn subcommand_help_wraps_and_indents_descriptions() {
        for command in ["workspace", "compare", "explain"] {
            for flag in ["-h", "--help"] {
                assert_help_layout(&["locmetry", command, flag]);
            }
        }
    }

    /// Bold is for people at a terminal: redirected output stays plain
    /// text, and `NO_COLOR` (no-color.org) or `TERM=dumb` turns it off.
    #[test]
    fn bars_are_automatic_only_on_a_terminal_unless_requested_or_refused() {
        let comments = Some(BarKind::Comments);
        assert_eq!(bar_mode(None, false, true, 150), Bars::Fit(150));
        assert_eq!(bar_mode(None, false, false, 150), Bars::Off);
        assert_eq!(
            bar_mode(comments, false, false, 150),
            Bars::Always(150, BarKind::Comments)
        );
        assert_eq!(
            bar_mode(comments, false, true, 150),
            Bars::Always(150, BarKind::Comments)
        );
        assert_eq!(bar_mode(None, true, true, 150), Bars::Off);
    }

    #[test]
    fn emphasis_needs_a_capable_terminal() {
        assert!(emphasis(true, None, Some("xterm-256color".into())));
        assert!(emphasis(true, Some(String::new()), None));
        assert!(!emphasis(false, None, None));
        assert!(!emphasis(true, Some("1".into()), None));
        assert!(!emphasis(true, None, Some("dumb".into())));
    }

    #[test]
    fn explicit_exports_default_to_full_json_and_ordinary_pipes_to_headlines() {
        assert_eq!(default_format(true, false), Format::Table);
        assert_eq!(default_format(false, false), Format::Headline);
        assert_eq!(default_format(true, true), Format::Json);
        assert_eq!(default_format(false, true), Format::Json);
    }
}
