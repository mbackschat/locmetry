//! Parsing the command line, with the checks clap cannot express.
use super::*;

pub(crate) fn validate_query(
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
                && !arg.is_global_set()
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

pub(crate) fn cli_command(args: &[std::ffi::OsString]) -> clap::Command {
    let mut command = Cli::command().styles(locmetry::style::Palette::ANSI.cli_styles());
    // Clap exits on help and parse errors before producing Cli, so apply the
    // global presentation switch first. `--` makes subsequent tokens positional.
    if args
        .iter()
        .skip(1)
        .take_while(|arg| *arg != "--")
        .any(|arg| arg == "--no-color")
    {
        command = command.color(clap::ColorChoice::Never);
    }
    command
}

pub(crate) fn parse_cli() -> clap::error::Result<Cli> {
    let args: Vec<_> = std::env::args_os().collect();
    let mut command = cli_command(&args);
    let matches = command.try_get_matches_from_mut(args)?;
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
        if command.get_arguments().any(|arg| {
            !arg.is_global_set()
                && matches.value_source(arg.get_id().as_str())
                    == Some(clap::parser::ValueSource::CommandLine)
        }) {
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
