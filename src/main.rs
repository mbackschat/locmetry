use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use locmetry::{
    breakdown, build_report, render_markdown, report_text, review, scan, scope_policy,
    validate_output, write_report, Policy, ScopeOverrides, BACKEND, CONFIG_FILE, DATA_LANGUAGES,
    DOCUMENTATION_LANGUAGES,
};
use serde_json::{json, Value};
use std::{
    io::{self, Write},
    path::{Path, PathBuf},
    process::ExitCode,
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Clone, Copy, ValueEnum)]
enum Format {
    Json,
    Markdown,
}

#[derive(Parser)]
#[command(
    version,
    about = "Measure code with embedded Tokei and reviewed classification rules"
)]
struct Args {
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
    /// Write the full JSON report to this file, outside the scan root; `-`
    /// writes it to stdout instead of the headline.
    #[arg(long, value_name = "FILE")]
    output: Option<PathBuf>,
    /// Include per-file records in the report.
    #[arg(long)]
    files: bool,
    /// Stdout format: a JSON headline (totals, top folders, coverage) or Markdown.
    #[arg(long, value_enum, default_value = "json")]
    format: Format,
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

fn run(args: Args) -> Result<Outcome> {
    let config_path = match &args.config {
        Some(path) => Some(path.clone()),
        None => {
            let discovered = args
                .root
                .as_deref()
                .unwrap_or(Path::new("."))
                .join(CONFIG_FILE);
            discovered.is_file().then_some(discovered)
        }
    };
    let mut policy = match &config_path {
        Some(path) => Policy::load(path)?,
        None => Policy::defaults(),
    };
    policy.override_scope(&ScopeOverrides {
        hidden: args.hidden,
        respect_ignore_files: args.respect_ignore_files,
        exclude: args.exclude.clone(),
    })?;
    let root = args
        .root
        .as_deref()
        .unwrap_or(Path::new("."))
        .canonicalize()
        .context("Scan root does not exist")?;
    let root_text = root.to_str().context("Scan root must be valid UTF-8")?;
    let project = root.file_name().map_or_else(
        || root_text.to_owned(),
        |name| name.to_string_lossy().into(),
    );
    let report_path = args
        .output
        .as_deref()
        .filter(|path| *path != Path::new("-"));
    if let Some(path) = report_path {
        validate_output(path, &root, config_path.as_deref())?;
    }
    anyhow::ensure!(
        report_path.is_some() || args.output.is_none() || matches!(args.format, Format::Json),
        "--output - writes JSON to stdout and cannot be combined with --format markdown"
    );
    let measured_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let scan = scan(&root, &policy)?;
    let (inaccurate, coverage) = (scan.inaccurate, &scan.coverage);
    let rows = scan.records(&policy)?;
    let summary = build_report(&rows);
    let mut markdown = render_markdown(&project, &summary, &rows, coverage, args.depth);
    if inaccurate {
        markdown.push_str("\nBackend reported incomplete counts; inspect the JSON metadata.\n");
    }
    let mut summary_json = serde_json::to_value(&summary.totals)?;
    summary_json["top_folders"] = serde_json::to_value(summary.top_folders())?;
    let mut document = json!({
        "schema_version":"1", "project":project,
        "metadata":{
            "root":root_text, "measured_at_unix_seconds":measured_at,
            "locmetry_version":env!("CARGO_PKG_VERSION"), "backend":BACKEND,
            "backend_inaccurate":inaccurate,
            "backend_configuration":"Explicit defaults; ambient Tokei configuration is not loaded",
            "scope":scope_policy(&policy),
            "kinds":{
                "default":"source",
                "data":DATA_LANGUAGES.iter().map(|l| l.name()).collect::<Vec<_>>(),
                "documentation":DOCUMENTATION_LANGUAGES.iter().map(|l| l.name()).collect::<Vec<_>>()
            },
            "coverage":coverage, "config_path":config_path, "config":policy.snapshot()
        },
        "summary":summary_json, "review":review(&rows), "breakdown":breakdown(&rows)
    });
    if args.files {
        document["files"] = serde_json::to_value(&rows)?;
    }
    if let Some(path) = report_path {
        write_report(path, &report_text(&document)?)?;
    }
    let mut stdout = io::stdout().lock();
    match (args.format, &args.output) {
        (Format::Json, Some(_)) if report_path.is_none() => {
            write!(stdout, "{}", report_text(&document)?)?
        }
        (Format::Json, _) => {
            serde_json::to_writer(&mut stdout, &headline(document, report_path)?)?;
            writeln!(stdout)?;
        }
        (Format::Markdown, _) => write!(stdout, "{markdown}")?,
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
    match run(Args::parse()) {
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
