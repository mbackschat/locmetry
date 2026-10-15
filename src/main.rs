use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use locmetry::{
    build_report, file_records, render_markdown, scan, scope_policy, validate_output,
    write_reports, Profile, Scan, BACKEND, DATA_LANGUAGES, DOCUMENTATION_LANGUAGES,
};
use serde_json::json;
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
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
    /// JSON/regex PoC profile (the proposed TOML product format is separate).
    #[arg(long)]
    config: PathBuf,
    /// Override the profile's scan root.
    #[arg(long)]
    root: Option<PathBuf>,
    /// Write .json, .md, and .tokei.json reports at this prefix, outside the scan root.
    #[arg(long)]
    output: Option<PathBuf>,
    /// Stdout format; with --output, JSON stdout is a compact summary.
    #[arg(long, value_enum, default_value = "json")]
    format: Format,
}

fn run(args: Args) -> Result<()> {
    let profile =
        Profile::from_json(&fs::read_to_string(&args.config).context("Cannot read profile")?)?;
    let root = args
        .root
        .as_deref()
        .unwrap_or(profile.root())
        .canonicalize()
        .context("Scan root does not exist")?;
    let root_text = root.to_str().context("Scan root must be valid UTF-8")?;
    if let Some(output) = &args.output {
        validate_output(output, &root, &args.config)?;
    }
    let measured_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let Scan { payload, coverage } = scan(&root, &profile)?;
    let inaccurate = payload
        .as_object()
        .context("Invalid backend payload")?
        .values()
        .any(|group| group["inaccurate"].as_bool().unwrap_or(false));
    let rows = file_records(payload.clone(), &profile)?;
    let summary = build_report(&rows);
    let mut markdown = render_markdown(profile.project(), &summary, &rows, &coverage);
    if inaccurate {
        markdown.push_str("\nBackend reported incomplete counts; inspect the JSON metadata.\n");
    }
    let document = json!({
        "schema_version":"poc-3", "project":profile.project(),
        "metadata":{
            "root":root_text, "measured_at_unix_seconds":measured_at,
            "locmetry_version":env!("CARGO_PKG_VERSION"), "backend":BACKEND,
            "backend_inaccurate":inaccurate,
            "backend_configuration":"Explicit defaults; ambient Tokei configuration is not loaded",
            "scope":scope_policy(&profile),
            "kinds":{
                "default":"source",
                "data/config":DATA_LANGUAGES.iter().map(|l| l.name()).collect::<Vec<_>>(),
                "documentation":DOCUMENTATION_LANGUAGES.iter().map(|l| l.name()).collect::<Vec<_>>()
            },
            "coverage":coverage, "config":profile.snapshot()?
        },
        "summary":summary, "files":rows
    });
    if let Some(output) = &args.output {
        write_reports(output, &root, &args.config, &document, &payload, &markdown)?;
    }
    let mut stdout = io::stdout().lock();
    match args.format {
        Format::Json => {
            let value = match args.output {
                Some(_) => {
                    json!({"project":profile.project(),"source":summary.totals.source,"purpose":summary.totals.source_purpose,"backend_inaccurate":inaccurate})
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
