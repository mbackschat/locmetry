use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use locmetry::{
    build_report, file_records, render_markdown, scan, scope_policy, validate_output,
    write_reports, Policy, Scan, ScopeOverrides, BACKEND, CONFIG_FILE, DATA_LANGUAGES,
    DOCUMENTATION_LANGUAGES,
};
use serde_json::json;
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
    /// Scan root [default: a legacy profile's root, else the current directory].
    #[arg(long)]
    root: Option<PathBuf>,
    /// Configuration file [default: .locmetryrc in the scan root, else built-in
    /// settings]. A .json file selects a provisional regex profile.
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
    /// Write .json, .md, and .tokei.json reports at this prefix, outside the scan root.
    #[arg(long)]
    output: Option<PathBuf>,
    /// Stdout format; with --output, JSON stdout is a compact summary.
    #[arg(long, value_enum, default_value = "json")]
    format: Format,
}

fn run(args: Args) -> Result<()> {
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
        .or(policy.root())
        .unwrap_or(Path::new("."))
        .canonicalize()
        .context("Scan root does not exist")?;
    let root_text = root.to_str().context("Scan root must be valid UTF-8")?;
    let project = match policy.project() {
        Some(project) => project.to_owned(),
        None => root.file_name().map_or_else(
            || root_text.to_owned(),
            |name| name.to_string_lossy().into(),
        ),
    };
    if let Some(output) = &args.output {
        validate_output(output, &root, config_path.as_deref())?;
    }
    let measured_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let Scan { payload, coverage } = scan(&root, &policy)?;
    let inaccurate = payload
        .as_object()
        .context("Invalid backend payload")?
        .values()
        .any(|group| group["inaccurate"].as_bool().unwrap_or(false));
    let rows = file_records(payload.clone(), &policy)?;
    let summary = build_report(&rows);
    let mut markdown = render_markdown(&project, &summary, &rows, &coverage);
    if inaccurate {
        markdown.push_str("\nBackend reported incomplete counts; inspect the JSON metadata.\n");
    }
    let document = json!({
        "schema_version":"poc-3", "project":project,
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
        "summary":summary, "files":rows
    });
    if let Some(output) = &args.output {
        write_reports(
            output,
            &root,
            config_path.as_deref(),
            &document,
            &payload,
            &markdown,
        )?;
    }
    let mut stdout = io::stdout().lock();
    match args.format {
        Format::Json => {
            let value = match args.output {
                Some(_) => {
                    json!({"project":project,"source":summary.totals.source,"purpose":summary.totals.source_purpose,"backend_inaccurate":inaccurate})
                }
                None => document,
            };
            serde_json::to_writer_pretty(&mut stdout, &value)?;
            writeln!(stdout)?;
        }
        Format::Markdown => write!(stdout, "{markdown}")?,
    }
    if inaccurate {
        let mut unread = coverage.unread.iter().take(10).cloned().collect::<Vec<_>>();
        if coverage.unread.len() > unread.len() {
            unread.push(format!("and {} more", coverage.unread.len() - unread.len()));
        }
        if unread.is_empty() {
            unread.push("none identified".into());
        }
        anyhow::bail!(
            "Backend reported incomplete counts; output contains partial results. Unreadable files: {}",
            unread.join(", ")
        );
    }
    Ok(())
}

fn main() -> ExitCode {
    match run(Args::parse()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("locmetry: {error:#}");
            ExitCode::FAILURE
        }
    }
}
