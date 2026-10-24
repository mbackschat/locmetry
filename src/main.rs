mod app;
mod guides;

use anyhow::Result;
use app::*;
use std::process::ExitCode;

fn run(cli: Cli) -> Result<Outcome> {
    let styling = cli.styling;
    if let Some(guide) = cli.guide {
        return run_guide(guide, &cli.section);
    }
    if let Some(schema) = cli.schema {
        return run_schema(schema, &cli.section);
    }
    match cli.command {
        Some(Command::Workspace(workspace)) => run_workspace(&workspace, styling),
        Some(Command::Compare(args)) => run_compare(&args, styling),
        Some(Command::Explain(args)) => run_explain(&args, styling),
        None => {
            let mut root = cli.root;
            root.root = root.root.or(cli.directory);
            run_root(
                &root,
                &cli.shared,
                cli.subfolder.as_deref(),
                cli.folders,
                cli.folder_rows.map(|n| n.get()),
                styling,
            )
        }
    }
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
    let styling = cli.styling;
    match run(cli) {
        Ok(Outcome::Complete) => ExitCode::SUCCESS,
        Ok(Outcome::Partial(message)) => {
            styling.stderr_message(Severity::Warning, message);
            ExitCode::from(2)
        }
        Err(error) => {
            styling.stderr_message(Severity::Error, format_args!("{error:#}"));
            ExitCode::FAILURE
        }
    }
}
