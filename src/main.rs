use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use locmetry::{
    build_report, render_markdown, scan, scope_policy, validate_output, write_reports, Policy,
    ScopeOverrides, BACKEND, CONFIG_FILE, DATA_LANGUAGES, DOCUMENTATION_LANGUAGES,
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
    /// Write .json, .md, and .tokei.json reports at this prefix, outside the scan root.
    #[arg(long)]
    output: Option<PathBuf>,
    /// Stdout format; with --output, JSON stdout omits file records and folders
    /// and names the written report files.
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
    if let Some(output) = &args.output {
        validate_output(output, &root, config_path.as_deref())?;
    }
    let measured_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let scan = scan(&root, &policy)?;
    let (inaccurate, coverage) = (scan.inaccurate, &scan.coverage);
    let rows = scan.records(&policy)?;
    let summary = build_report(&rows);
    let mut markdown = render_markdown(&project, &summary, &rows, coverage, args.depth);
    if inaccurate {
        markdown.push_str("\nBackend reported incomplete counts; inspect the JSON metadata.\n");
    }
    let document = json!({
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
        "summary":summary, "files":rows
    });
    let written = match &args.output {
        Some(output) => Some(write_reports(
            output,
            &root,
            config_path.as_deref(),
            &document,
            &scan.payload,
            &markdown,
        )?),
        None => None,
    };
    let mut stdout = io::stdout().lock();
    match args.format {
        Format::Json => {
            let value = match written {
                // The full report can be megabytes; agents reading stdout get
                // the same document without records, plus where to find them.
                Some([json, markdown, tokei]) => {
                    let mut headline = document;
                    headline
                        .as_object_mut()
                        .context("Invalid report")?
                        .remove("files");
                    headline["summary"]
                        .as_object_mut()
                        .context("Invalid summary")?
                        .remove("folders");
                    headline["reports"] =
                        json!({"json": json, "markdown": markdown, "tokei": tokei});
                    headline
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
        return Ok(Outcome::Partial(format!(
            "Backend reported incomplete counts; output contains partial results. Unreadable files: {}",
            unread.join(", ")
        )));
    }
    Ok(Outcome::Complete)
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
