//! The `--generate` query: the man page and shell completion scripts, built
//! from the clap definitions (docs/proposals/archive/manual-and-reference.md).
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
    let mut command = match shell {
        Shell::Zsh => without_root_positional(),
        _ => Cli::command(),
    };
    let mut script = Vec::new();
    clap_complete::generate(shell, &mut command, "locmetry", &mut script);
    match shell {
        Shell::Zsh => {
            root_options_after_directory(String::from_utf8(script).expect("UTF-8")).into_bytes()
        }
        _ => script,
    }
}

/// The command tree for zsh, without the root's `[DIR]`.
///
/// clap_complete 4.6.11 gives zsh the root's positionals before the command
/// slot, so zsh took `compare` in `locmetry compare --m` as the directory and
/// offered root options such as `--markdown-depth` (manual completion check,
/// 2026-10-24). Command names are reserved and never a directory, so without
/// the positional the first word is the command; bash and fish find commands
/// by name and keep it. Clap cannot remove an argument, so the root is
/// rebuilt from its built form, in which help and version are explicit
/// arguments; the version argument needs the version.
fn without_root_positional() -> clap::Command {
    let mut built = Cli::command();
    built.build();
    clap::Command::new("locmetry")
        .version(env!("CARGO_PKG_VERSION"))
        .disable_help_flag(true)
        .disable_version_flag(true)
        .disable_help_subcommand(true)
        .args(
            built
                .get_arguments()
                .filter(|arg| !arg.is_positional())
                .cloned(),
        )
        .subcommands(built.get_subcommands().cloned())
}

/// Adds a default branch to the zsh dispatch on the first word: a word that
/// names no command is the directory, after which the root's options apply.
/// zsh also dispatches with an empty first word while it completes that
/// word, where a match-all branch repeated the root's completions and rang
/// the bell (zsh 5.9, 2026-10-24), so the branch requires a word.
///
/// The branch repeats the root's option specs, which open the script's first
/// `_arguments` call and end where its command slot begins, and closes the
/// last `case` of the `_locmetry` function, the dispatch, which encloses the
/// nested one of the `help` command. tests/integration/generate.rs checks the
/// branch against `--help`.
fn root_options_after_directory(script: String) -> String {
    let specs: Vec<&str> = script
        .lines()
        .skip_while(|line| !line.trim_start().starts_with("_arguments "))
        .skip(1)
        .take_while(|line| !line.starts_with("\":: :_locmetry_commands\""))
        .collect();
    let branch = format!(
        "(?*)\n_arguments \"${{_arguments_options[@]}}\" : \\\n{}\n&& ret=0\n;;\n",
        specs.join("\n")
    );
    let function = script.find("\n}\n").expect("the _locmetry function ends");
    let dispatch_end = script[..function]
        .rfind("\n        esac")
        .expect("the dispatch closes")
        + 1;
    let mut patched = script;
    patched.insert_str(dispatch_end, &branch);
    patched
}
