use anyhow::{Context, Result};
use clap::{Parser, Subcommand, ValueEnum};
use locmetry::{
    breakdown, build_report, build_systems, git_submodules, module_shares, render_markdown,
    render_table, report_text, review, scan, scope_policy, submodule_shares, unclassified_folders,
    unused_entries, user_defaults_file, validate_output, write_report, Coverage, FileRecord,
    Findings, Module, ModuleShare, Policy, RowOrder, ScopeOverrides, SubmoduleShare, BACKEND,
    CONFIG_FILE, DATA_LANGUAGES, DOCUMENTATION_LANGUAGES,
};
use serde_json::{json, Value};
use std::{
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
    process::ExitCode,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
enum Format {
    /// Source code per language and folder, split by purpose, for people.
    Table,
    /// The compact headline for agents and scripts.
    Json,
    /// A full human report to save or paste into reviews.
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
    about = "Measure source code by language, folder, and purpose: production, test, or unclassified",
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
    /// every setting and the classification order, or of the JSON report.
    #[arg(long, value_enum)]
    schema: Option<Schema>,
}

#[derive(Subcommand)]
enum Command {
    /// Measure several repositories as one virtual monorepo: each root with its
    /// own .locmetryrc, merged with paths prefixed by the root's folder name.
    Workspace(WorkspaceArgs),
}

/// Settings of a single scan root, which a workspace leaves to each member's
/// own configuration (PRODUCT.md, Workspace Mode).
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
    #[arg(long = "root", value_name = "DIR", required = true)]
    roots: Vec<PathBuf>,
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

/// Result of a run that produced a report.
enum Outcome {
    Complete,
    /// Counts are missing for unreadable files; the message names them.
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

fn measure(
    root: &Path,
    config: Option<PathBuf>,
    user_path: Option<&Path>,
    overrides: &ScopeOverrides,
) -> Result<Measurement> {
    let config_path = config.or_else(|| {
        let discovered = root.join(CONFIG_FILE);
        discovered.is_file().then_some(discovered)
    });
    let mut policy = Policy::load_layers(config_path.as_deref(), user_path)?;
    policy.override_scope(overrides)?;
    let root = root.canonicalize().context("Scan root does not exist")?;
    for path in [config_path.as_deref(), user_path].into_iter().flatten() {
        policy.exclude_configuration(&root, path)?;
    }
    let root_text = root.to_str().context("Scan root must be valid UTF-8")?;
    let name = root.file_name().map_or_else(
        || root_text.to_owned(),
        |name| name.to_string_lossy().into(),
    );
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
}

/// The output destination and stdout format, checked before measuring so a
/// misuse fails fast.
struct Destination<'a> {
    report_path: Option<&'a Path>,
    report_to_stdout: bool,
    format: Format,
}

fn destination(shared: &SharedArgs) -> Result<Destination<'_>> {
    let report_path = shared
        .output
        .as_deref()
        .filter(|path| *path != Path::new("-"));
    let report_to_stdout = shared.output.is_some() && report_path.is_none();
    let format = shared
        .format
        .unwrap_or_else(|| default_format(io::stdout().is_terminal(), report_to_stdout));
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

fn user_defaults(shared: &SharedArgs) -> Option<PathBuf> {
    user_defaults_file().filter(|path| !shared.no_user_config && path.is_file())
}

fn run(cli: Cli) -> Result<Outcome> {
    if let Some(schema) = cli.schema {
        let schema = match schema {
            Schema::Config => locmetry::config_schema(),
            Schema::UserConfig => locmetry::user_config_schema(),
            Schema::Report => locmetry::report_schema(),
        };
        println!("{}", serde_json::to_string_pretty(&schema)?);
        return Ok(Outcome::Complete);
    }
    match cli.command {
        Some(Command::Workspace(workspace)) => run_workspace(&workspace),
        None => run_root(&cli.root, &cli.shared),
    }
}

fn run_root(args: &RootArgs, shared: &SharedArgs) -> Result<Outcome> {
    let root = args.root.as_deref().unwrap_or(Path::new("."));
    let destination = destination(shared)?;
    let user_path = user_defaults(shared);
    let overrides = ScopeOverrides {
        hidden: args.hidden,
        respect_ignore_files: args.respect_ignore_files,
        exclude: args.exclude.clone(),
    };
    let measured_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let measurement = measure(root, args.config.clone(), user_path.as_deref(), &overrides)?;
    if let Some(path) = destination.report_path {
        validate_output(path, &measurement.root, measurement.config_path.as_deref())?;
    }
    let metadata = json!({
        "root": measurement.root,
        "scope": scope_policy(&measurement.policy),
        "config_path": measurement.config_path,
        "user_config_path": user_path,
        "config": measurement.policy.snapshot(),
    });
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
    };
    report(shared, &destination, results, metadata, measured_at, true)
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
    let members = workspace_members(&args.roots)?;
    if let Some(path) = destination.report_path {
        for (_, root) in &members {
            validate_output(path, root, None)?;
        }
    }
    let user_path = user_defaults(shared);
    let measured_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    // Members run in parallel; collecting in name order keeps the merge and
    // the reported error deterministic whichever thread finishes first.
    let measurements = std::thread::scope(|threads| {
        let handles: Vec<_> = members
            .iter()
            .map(|(name, root)| {
                let user_path = user_path.as_deref();
                let handle = threads
                    .spawn(move || measure(root, None, user_path, &ScopeOverrides::default()));
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
    };
    let mut records = Vec::new();
    for measurement in measurements {
        let name = &measurement.name;
        let prefixed = |path: &str| match path {
            "." => name.clone(),
            _ => format!("{name}/{path}"),
        };
        records.push(json!({
            "name": name,
            "root": measurement.root,
            "config_path": measurement.config_path,
            "scope": scope_policy(&measurement.policy),
            "unused_entries": measurement.unused_entries,
            "unused_exclusions": measurement.unused_exclusions,
            "config": measurement.policy.snapshot(),
        }));
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
    results.coverage.unrecognized.sort();
    let metadata = json!({"user_config_path": user_path, "members": records});
    report(shared, &destination, results, metadata, measured_at, false)
}

/// Writes the report, headline, or human output for `results`.
///
/// `metadata` holds the mode's own keys: a single root's root, scope, and
/// configuration, or a workspace's members. Configuration findings enter the
/// summary only for a single root, because a workspace's entry IDs and
/// exclusion patterns are relative to one member.
fn report(
    shared: &SharedArgs,
    destination: &Destination,
    results: Results,
    metadata: Value,
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
    } = results;
    let format = destination.format;
    let summary = build_report(&rows);
    let findings = Findings {
        coverage: &coverage,
        modules: &module_shares,
        submodules: &submodules,
        unused_entries: &unused_entries,
        unused_exclusions: &unused_exclusions,
    };
    let order = match shared.sort {
        Sort::Code => RowOrder::Code,
        Sort::Name => RowOrder::Name,
    };
    let mut human = match format {
        Format::Table => render_table(
            &project,
            &summary,
            &rows,
            &findings,
            shared.categories,
            order,
        ),
        Format::Markdown => {
            render_markdown(&project, &summary, &rows, &findings, shared.depth, order)
        }
        Format::Json => String::new(),
    };
    if inaccurate && format != Format::Json {
        human.push_str("\nBackend reported incomplete counts; inspect the JSON metadata.\n");
    }
    let mut summary_json = serde_json::to_value(&summary.totals)?;
    summary_json["top_folders"] = serde_json::to_value(summary.top_folders())?;
    if !module_shares.is_empty() {
        summary_json["build_modules"] = serde_json::to_value(build_systems(&module_shares))?;
    }
    if !submodules.is_empty() {
        summary_json["submodules"] = serde_json::to_value(&submodules)?;
    }
    if summary_findings && !unused_entries.is_empty() {
        summary_json["unused_entries"] = serde_json::to_value(&unused_entries)?;
    }
    if summary_findings && !unused_exclusions.is_empty() {
        summary_json["unused_exclusions"] = serde_json::to_value(&unused_exclusions)?;
    }
    // Always present, even empty: it is the documented next step for configuration.
    summary_json["unclassified_folders"] = serde_json::to_value(unclassified_folders(&rows))?;
    let mut document = json!({
        "schema_version":"1", "project":project,
        "metadata":{
            "measured_at_unix_seconds":measured_at,
            "locmetry_version":env!("CARGO_PKG_VERSION"), "backend":BACKEND,
            "backend_inaccurate":inaccurate,
            "backend_configuration":"Explicit defaults; ambient Tokei configuration is not loaded",
            "kinds":{
                "default":"source",
                "data":DATA_LANGUAGES.iter().map(|l| l.name()).collect::<Vec<_>>(),
                "documentation":DOCUMENTATION_LANGUAGES.iter().map(|l| l.name()).collect::<Vec<_>>()
            },
            "coverage":coverage, "modules":modules,
        },
        "summary":summary_json, "review":review(&rows), "breakdown":breakdown(&rows)
    });
    let fields = metadata.as_object().context("Invalid metadata")?;
    for (key, value) in fields {
        document["metadata"][key] = value.clone();
    }
    if shared.files {
        document["files"] = serde_json::to_value(&rows)?;
    }
    if let Some(path) = destination.report_path {
        write_report(path, &report_text(&document)?)?;
    }
    let mut stdout = io::stdout().lock();
    match format {
        Format::Json if destination.report_to_stdout => {
            write!(stdout, "{}", report_text(&document)?)?
        }
        Format::Json => {
            serde_json::to_writer(&mut stdout, &headline(document, destination.report_path)?)?;
            writeln!(stdout)?;
        }
        Format::Table | Format::Markdown => write!(stdout, "{human}")?,
    }
    if inaccurate {
        let mut unread = coverage.unread.iter().take(10).cloned().collect::<Vec<_>>();
        if coverage.unread.len() > unread.len() {
            unread.push(format!("and {} more", coverage.unread.len() - unread.len()));
        }
        if unread.is_empty() {
            unread.push("none identified".into());
        }
        return Ok(Outcome::Partial(format!(
            "Backend reported incomplete counts; output contains partial results. Unreadable files: {}",
            unread.join(", ")
        )));
    }
    Ok(Outcome::Complete)
}

/// The report without its tables, policy echo, or path lists: what an agent
/// reads into its context, so it is compact. The full report stays in the
/// report file or `--output -`.
fn headline(mut document: Value, report: Option<&Path>) -> Result<Value> {
    let object = document.as_object_mut().context("Invalid report")?;
    for table in ["review", "breakdown", "files"] {
        object.remove(table);
    }
    let metadata = document["metadata"]
        .as_object_mut()
        .context("Invalid metadata")?;
    metadata.remove("config");
    metadata.remove("modules");
    for member in metadata
        .get_mut("members")
        .and_then(Value::as_array_mut)
        .into_iter()
        .flatten()
    {
        member
            .as_object_mut()
            .context("Invalid member")?
            .remove("config");
    }
    metadata.remove("kinds");
    metadata["coverage"]
        .as_object_mut()
        .context("Invalid coverage")?
        .remove("unrecognized");
    if let Some(path) = report {
        document["report"] = json!(path);
    }
    Ok(document)
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
            eprintln!("locmetry: {message}");
            ExitCode::from(2)
        }
        Err(error) => {
            eprintln!("locmetry: {error:#}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn terminals_default_to_the_table_and_pipes_to_json() {
        assert_eq!(default_format(true, false), Format::Table);
        assert_eq!(default_format(false, false), Format::Json);
        assert_eq!(default_format(true, true), Format::Json);
        assert_eq!(default_format(false, true), Format::Json);
    }
}
