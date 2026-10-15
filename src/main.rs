use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use locmetry::contract::{self, MeasurementReport};
use locmetry::style::{Role, Theme};
use locmetry::{
    breakdown, build_report, build_systems, git_submodules, module_shares, render_markdown,
    render_table_with_theme, report_text, review, scan, scope_policy, submodule_shares,
    unclassified_folders, unused_entries, user_defaults_file, validate_output, write_report,
    Coverage, FileRecord, Findings, MemberView, Module, ModuleShare, Policy, RowOrder,
    ScopeOverrides, SubmoduleShare, BACKEND, CONFIG_FILE, DATA_LANGUAGES, DOCUMENTATION_LANGUAGES,
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
    /// Structured JSON for agents and scripts; see the command description.
    Json,
    /// Markdown tables and evidence to save or paste into reviews.
    Markdown,
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

/// The stdout format when `--format` is absent.
///
/// People at a terminal get the table, while pipes keep the JSON headline that
/// agents and scripts parse; `--output -` always prints the JSON report.
fn default_format(stdout_is_terminal: bool, report_to_stdout: bool) -> Format {
    match (report_to_stdout, stdout_is_terminal) {
        (true, _) | (false, false) => Format::Json,
        (false, true) => Format::Table,
    }
}

#[derive(Parser)]
#[command(
    version,
    about = "Measure source code by language, folder, and purpose: production, test, or unclassified. JSON defaults to a compact headline; --output - prints the full report.",
    args_conflicts_with_subcommands = true
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    #[command(flatten)]
    root: RootArgs,
    #[command(flatten)]
    shared: SharedArgs,
    /// Print a JSON Schema and exit: of a configuration file, which explains
    /// every setting and the classification order, of the JSON report, or of
    /// a workspace file, a net-change comparison, or a path explanation.
    #[arg(long, value_enum)]
    schema: Option<Schema>,
}

#[derive(Subcommand)]
enum Command {
    /// Measure several repositories as one virtual monorepo: each root with its
    /// own .locmetryrc, merged with paths prefixed by the root's folder name.
    /// JSON defaults to a compact headline; --output - prints the full report.
    Workspace(WorkspaceArgs),
    /// Compare two saved full reports: net change is after minus before. JSON defaults
    /// to a headline; --output - prints the full comparison.
    Compare(CompareArgs),
    /// Explain selection and classification of one existing root-relative file.
    /// JSON prints the complete explanation; there is no separate headline.
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
    format: Option<Format>,
}

#[derive(clap::Args)]
struct CompareArgs {
    /// Full JSON measurement before the change.
    before: PathBuf,
    /// Full JSON measurement after the change.
    after: PathBuf,
    /// Stdout format [default: table on a terminal, json otherwise].
    #[arg(long, value_enum)]
    format: Option<Format>,
    /// Write full comparison to FILE; - prints full JSON to stdout.
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
    /// Write the full JSON report to this file, outside the scan root; `-`
    /// writes it to stdout instead of the headline.
    #[arg(long, value_name = "FILE")]
    output: Option<PathBuf>,
    /// Include per-file records in the report.
    #[arg(long)]
    files: bool,
    /// Add a table of source code per purpose and category to the terminal table.
    #[arg(long)]
    categories: bool,
    /// Stdout format [default: table on a terminal, json otherwise].
    #[arg(long, value_enum)]
    format: Option<Format>,
    /// Row order of table and Markdown output; the Markdown folder table keeps
    /// its path hierarchy, and the review list stays largest first.
    #[arg(long, value_enum, default_value = "code")]
    sort: Sort,
    /// Deepest folder level listed in the Markdown folder table; JSON lists all.
    #[arg(long, default_value_t = 2, value_name = "N")]
    depth: usize,
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

fn measure(prepared: Prepared) -> Result<Measurement> {
    let Prepared {
        root,
        name,
        config_path,
        mut policy,
    } = prepared;
    let scan = scan(&root, &policy)?;
    policy.adopt_modules(&scan.modules)?;
    let rows = scan.records(&policy)?;
    Ok(Measurement {
        modules: module_shares(policy.modules(), &rows),
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

/// The output destination and stdout format, checked before measuring so a
/// misuse fails fast.
struct Destination<'a> {
    report_path: Option<&'a Path>,
    report_to_stdout: bool,
    format: Format,
}

fn destination(shared: &SharedArgs) -> Result<Destination<'_>> {
    output_destination(&shared.output, shared.format)
}

fn output_destination(output: &Option<PathBuf>, format: Option<Format>) -> Result<Destination<'_>> {
    let report_path = output.as_deref().filter(|path| *path != Path::new("-"));
    let report_to_stdout = output.is_some() && report_path.is_none();
    let format =
        format.unwrap_or_else(|| default_format(io::stdout().is_terminal(), report_to_stdout));
    anyhow::ensure!(
        !report_to_stdout || format == Format::Json,
        "--output - writes JSON to stdout and cannot be combined with --format table or markdown"
    );
    Ok(Destination {
        report_path,
        report_to_stdout,
        format,
    })
}

fn user_defaults(no_user_config: bool) -> Option<PathBuf> {
    user_defaults_file().filter(|path| !no_user_config && path.is_file())
}

fn run(cli: Cli) -> Result<Outcome> {
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
        None => run_root(&cli.root, &cli.shared),
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
    let overrides = ScopeOverrides {
        hidden: args.root.hidden,
        respect_ignore_files: args.root.respect_ignore_files,
        exclude: args.root.exclude.clone(),
    };
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
        .unwrap_or_else(|| default_format(io::stdout().is_terminal(), false))
    {
        Format::Json => println!("{}", serde_json::to_string_pretty(&explanation)?),
        Format::Table => print!(
            "{}",
            explanation.render_with_theme(false, terminal_theme(io::stdout().is_terminal()))
        ),
        Format::Markdown => print!("{}", explanation.render(true)),
    }
    Ok(if explanation.partial() {
        Outcome::Partial(format!("Counts unavailable for {}", explanation.path))
    } else {
        Outcome::Complete
    })
}

fn run_compare(args: &CompareArgs) -> Result<Outcome> {
    let destination = output_destination(&args.output, args.format)?;
    let before = locmetry::compare::read_input(&args.before)?;
    let after = locmetry::compare::read_input(&args.after)?;
    if let Some(path) = destination.report_path {
        locmetry::compare::validate_destination(path, [&args.before, &args.after])?;
    }
    let result = locmetry::compare::compare(
        before,
        after,
        args.before.to_string_lossy().into(),
        args.after.to_string_lossy().into(),
    )?;
    let partial = result.partial();
    if let Some(path) = destination.report_path {
        write_report(path, &report_text(&serde_json::to_value(&result)?)?)?;
    }
    let mut stdout = io::stdout().lock();
    match destination.format {
        Format::Json if destination.report_to_stdout => {
            write!(stdout, "{}", report_text(&serde_json::to_value(&result)?)?)?
        }
        Format::Json => {
            serde_json::to_writer(
                &mut stdout,
                &result.headline(destination.report_path.map(|p| p.to_string_lossy().into())),
            )?;
            writeln!(stdout)?;
        }
        Format::Table => write!(
            stdout,
            "{}",
            locmetry::compare::render_table_with_theme(
                &result,
                terminal_theme(io::stdout().is_terminal())
            )
        )?,
        Format::Markdown => write!(stdout, "{}", locmetry::compare::render_markdown(&result))?,
    }
    Ok(if partial {
        Outcome::Partial(
            "Net change uses partial input measurements; inspect coverage findings.".into(),
        )
    } else {
        Outcome::Complete
    })
}

fn run_root(args: &RootArgs, shared: &SharedArgs) -> Result<Outcome> {
    let root = args.root.as_deref().unwrap_or(Path::new("."));
    let destination = destination(shared)?;
    let user_path = user_defaults(shared.no_user_config);
    let overrides = ScopeOverrides {
        hidden: args.hidden,
        respect_ignore_files: args.respect_ignore_files,
        exclude: args.exclude.clone(),
    };
    let measured_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let prepared = prepare(
        root,
        args.config.clone(),
        user_path.as_deref(),
        None,
        &overrides,
    )?;
    if let Some(path) = destination.report_path {
        validate_output(path, &prepared.root, prepared.config_path.as_deref())?;
    }
    let measurement = measure(prepared)?;
    let location = Location::Root {
        root: measurement.root,
        scope: scope_policy(&measurement.policy),
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
    report(shared, &destination, results, location, measured_at, true)
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
    if let Some(path) = destination.report_path {
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
                    measure(prepare(root, None, user_path, workspace_path, &overrides)?)
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
        let prefixed = |path: &str| match path {
            "." => name.clone(),
            _ => format!("{name}/{path}"),
        };
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
            coverage: contract::MemberCoverage {
                selected_files: measurement.coverage.selected_files as u64,
                counted_files: measurement.coverage.counted_files as u64,
                mapped_files: measurement.coverage.mapped_files as u64,
                unrecognized_files: measurement.coverage.unrecognized_files as u64,
                unread_files: measurement.coverage.unread.len() as u64,
                parse_failed_files: Some(measurement.coverage.parse_failures.len() as u64),
            },
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
    report(shared, &destination, results, location, measured_at, false)
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
    summary_findings: bool,
) -> Result<Outcome> {
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
    let findings = Findings {
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
    let mut human = match format {
        Format::Table => render_table_with_theme(
            &project,
            &summary,
            &rows,
            &findings,
            shared.categories,
            order,
            terminal_theme(io::stdout().is_terminal()),
        ),
        Format::Markdown => {
            render_markdown(&project, &summary, &rows, &findings, shared.depth, order)
        }
        Format::Json => String::new(),
    };
    if inaccurate && format != Format::Json {
        let theme = match format {
            Format::Markdown => Theme::MARKDOWN,
            _ => terminal_theme(io::stdout().is_terminal()),
        };
        human.push_str(&format!(
            "\n{}; inspect the JSON metadata.\n",
            theme.paint(Role::Warning, "Backend reported incomplete counts")
        ));
    }
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
        backend_configuration: "Explicit defaults; ambient Tokei configuration is not loaded"
            .into(),
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
    let document = MeasurementReport {
        schema_version: contract::Version::V1,
        project,
        metadata,
        summary: summary_document,
        review: Some(review(&rows)),
        breakdown: Some(breakdown(&rows)),
        files: shared
            .files
            .then(|| rows.iter().map(contract::File::from).collect()),
        report: None,
    };
    let serialized = serde_json::to_value(&document)?;
    if let Some(path) = destination.report_path {
        write_report(path, &report_text(&serialized)?)?;
    }
    let mut stdout = io::stdout().lock();
    match format {
        Format::Json if destination.report_to_stdout => {
            write!(stdout, "{}", report_text(&serialized)?)?
        }
        Format::Json => {
            serde_json::to_writer(
                &mut stdout,
                &document.headline(destination.report_path.map(|p| p.to_string_lossy().into())),
            )?;
            writeln!(stdout)?;
        }
        Format::Table | Format::Markdown => write!(stdout, "{human}")?,
    }
    if inaccurate {
        let mut failures: Vec<_> = coverage
            .unread
            .iter()
            .map(|path| format!("{path}: unreadable"))
            .chain(coverage.parse_failures.iter().map(|failure| {
                format!("{}: {} — {}", failure.path, failure.reason, failure.message)
            }))
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

fn main() -> ExitCode {
    // clap exits usage errors with 2, which is reserved for partial reports.
    let cli = match Cli::try_parse() {
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
            eprintln!(
                "{} {message}",
                terminal_theme(io::stderr().is_terminal()).paint(Role::Warning, "locmetry:")
            );
            ExitCode::from(2)
        }
        Err(error) => {
            eprintln!(
                "{} {error:#}",
                terminal_theme(io::stderr().is_terminal()).paint(Role::Error, "locmetry:")
            );
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Bold is for people at a terminal: redirected output stays plain
    /// text, and `NO_COLOR` (no-color.org) or `TERM=dumb` turns it off.
    #[test]
    fn emphasis_needs_a_capable_terminal() {
        assert!(emphasis(true, None, Some("xterm-256color".into())));
        assert!(emphasis(true, Some(String::new()), None));
        assert!(!emphasis(false, None, None));
        assert!(!emphasis(true, Some("1".into()), None));
        assert!(!emphasis(true, None, Some("dumb".into())));
    }

    #[test]
    fn terminals_default_to_the_table_and_pipes_to_json() {
        assert_eq!(default_format(true, false), Format::Table);
        assert_eq!(default_format(false, false), Format::Json);
        assert_eq!(default_format(true, true), Format::Json);
        assert_eq!(default_format(false, true), Format::Json);
    }
}
