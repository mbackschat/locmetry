//! The `--generate` query: the man page and shell completion scripts, built
//! from the clap definitions (docs/proposals/manual-and-reference.md).
use super::*;
use clap_complete::Shell;

#[derive(Clone, Copy, ValueEnum)]
pub(crate) enum Generate {
    /// The locmetry(1) man page in roff.
    Man,
    /// A bash completion script.
    CompleteBash,
    /// A zsh completion script.
    CompleteZsh,
    /// A fish completion script.
    CompleteFish,
    /// A PowerShell completion script.
    CompletePowershell,
}

/// Prints the document whole, after it is complete, so a failure leaves
/// stdout empty.
pub(crate) fn run_generate(kind: Generate) -> Result<Outcome> {
    let document = match kind {
        Generate::Man => manual()?,
        Generate::CompleteBash => completions(Shell::Bash),
        Generate::CompleteZsh => completions(Shell::Zsh),
        Generate::CompleteFish => completions(Shell::Fish),
        Generate::CompletePowershell => completions(Shell::PowerShell),
    };
    io::stdout().lock().write_all(&document)?;
    Ok(Outcome::Complete)
}

/// The hidden `--generate` option. Only the parsing command has it, so the
/// documents generated from `Cli::command()` never list it: the generators
/// of clap_complete 4.6.11 emit hidden options.
pub(crate) fn generate_arg() -> clap::Arg {
    clap::Arg::new("generate")
        .long("generate")
        .value_name("KIND")
        .value_parser(clap::builder::EnumValueParser::<Generate>::new())
        .hide(true)
}

fn completions(shell: Shell) -> Vec<u8> {
    let mut script = Vec::new();
    clap_complete::generate(shell, &mut Cli::command(), "locmetry", &mut script);
    script
}
