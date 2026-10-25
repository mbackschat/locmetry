//! `--generate`: the man page and each completion script list, per command,
//! what that command's `--help` shows, as far as each format promises
//! (docs/proposals/archive/manual-and-reference.md, Guards). `--help` is the oracle,
//! parsed independently of the generators.
use crate::common;
use std::collections::BTreeMap;

const COMMANDS: [&str; 4] = ["", "workspace", "compare", "explain"];

/// An option's value name and possible values; one entry per spelling, so
/// `-h` and `--help` are two.
#[derive(Clone, Debug, Default, PartialEq)]
struct Spec {
    value: Option<String>,
    values: Vec<String>,
}

type Options = BTreeMap<String, Spec>;

fn run(args: &[&str]) -> String {
    common::stdout(
        common::locmetry()
            .env("COLUMNS", "300")
            .env("NO_COLOR", "1")
            .args(args.iter().filter(|arg| !arg.is_empty()))
            .output()
            .unwrap(),
    )
}

fn generate(kind: &str) -> String {
    run(&["--generate", kind])
}

/// The option and argument sections of a command's `--help`: from the usage
/// line to the first line at the margin that is prose or `Examples:`.
fn help_sections(command: &str) -> Vec<String> {
    let help = run(&[command, "--help"]);
    help.lines()
        .skip_while(|line| !line.starts_with("Usage:"))
        .skip(1)
        .take_while(|line| {
            line.starts_with(' ') || line.is_empty() || line.ends_with(':') && *line != "Examples:"
        })
        .map(str::to_owned)
        .collect()
}

/// The name and value name of an option header such as `-h, --help`,
/// `--root <DIR>` or `--show-bars[=<KIND>]`.
fn header(text: &str) -> (Vec<String>, Option<String>) {
    let text = text.split("  ").next().unwrap();
    let names = text
        .split(", ")
        .map(|name| name.split([' ', '[', '=']).next().unwrap().to_owned())
        .collect();
    let value = text
        .split_once('<')
        .map(|(_, rest)| rest[..rest.find('>').unwrap()].to_owned());
    (names, value)
}

fn help_options(command: &str) -> Options {
    let mut options = Options::new();
    let mut current: Vec<String> = Vec::new();
    let mut listing = false;
    for line in help_sections(command) {
        let text = line.trim_start();
        let indent = line.len() - text.len();
        if (indent == 2 || indent == 6) && text.starts_with('-') {
            let (names, value) = header(text);
            for name in &names {
                options.insert(
                    name.clone(),
                    Spec {
                        value: value.clone(),
                        values: Vec::new(),
                    },
                );
            }
            current = names;
            listing = false;
        } else if text == "Possible values:" {
            listing = true;
        } else if let Some(item) = text.strip_prefix("- ").filter(|_| listing) {
            let value = item.split(':').next().unwrap().to_owned();
            for name in &current {
                options.get_mut(name).unwrap().values.push(value.clone());
            }
        } else if let Some(list) = text.strip_prefix("[possible values: ") {
            for name in &current {
                options.get_mut(name).unwrap().values = list
                    .trim_end_matches(']')
                    .split(", ")
                    .map(String::from)
                    .collect();
            }
        } else if text.is_empty() {
            listing = false;
        } else if indent == 0 {
            current.clear();
        }
    }
    options
}

/// Each positional argument as `--help` writes it, such as `<BEFORE>` or
/// `[DIR]`, with the first line of its description.
fn help_positionals(command: &str) -> Vec<(String, String)> {
    let sections = help_sections(command);
    let mut lines = sections
        .iter()
        .skip_while(|line| *line != "Arguments:")
        .skip(1)
        .take_while(|line| !line.ends_with(':'));
    let mut found = Vec::new();
    while let Some(line) = lines.next() {
        let text = line.trim();
        if line.starts_with("  ") && !line.starts_with("   ") && !text.is_empty() {
            let syntax = text.trim_end_matches("...").to_owned();
            let description = lines.next().unwrap().trim().to_owned();
            found.push((syntax, description));
        }
    }
    found
}

/// Keeps only what a format promises: value names and possible values.
fn projected(options: &Options, value: bool, values: bool) -> Options {
    options
        .iter()
        .map(|(name, spec)| {
            let spec = Spec {
                value: spec.value.clone().filter(|_| value),
                values: if values {
                    spec.values.clone()
                } else {
                    Vec::new()
                },
            };
            (name.clone(), spec)
        })
        .collect()
}

/// Checks every command and the command-scoped negative case: `--member`
/// belongs to `compare` alone and `--show-folders` to the root alone.
fn agrees_with_help(kind: &str, scripts: BTreeMap<&str, Options>, value: bool, values: bool) {
    for command in COMMANDS {
        let expected = projected(&help_options(command), value, values);
        assert_eq!(scripts[command], expected, "{kind}: '{command}'");
    }
    assert!(scripts["compare"].contains_key("--member"), "{kind}");
    assert!(!scripts["workspace"].contains_key("--member"), "{kind}");
    assert!(scripts[""].contains_key("--show-folders"), "{kind}");
    assert!(!scripts["compare"].contains_key("--show-folders"), "{kind}");
}

/// The text from a line equal to `start` up to the next line for which
/// `end` holds.
fn block<'a>(text: &'a str, start: &str, end: impl Fn(&str) -> bool) -> Vec<&'a str> {
    text.lines()
        .skip_while(|line| line.trim() != start)
        .skip(1)
        .take_while(|line| !end(line))
        .collect()
}

fn scope_name(command: &str) -> String {
    match command {
        "" => "locmetry".to_owned(),
        name => format!("locmetry__subcmd__{name}"),
    }
}

#[test]
fn bash_completes_each_commands_options_and_possible_values() {
    let script = generate("complete-bash");
    let mut scripts = BTreeMap::new();
    for command in COMMANDS {
        let lines = block(&script, &format!("{})", scope_name(command)), |line| {
            line.starts_with("        ") && !line.starts_with("         ") && line.ends_with(')')
        });
        let mut options = Options::new();
        let opts = lines[0].trim().strip_prefix("opts=\"").unwrap();
        for name in opts.trim_end_matches('"').split(' ') {
            if name.starts_with('-') {
                options.insert(name.to_owned(), Spec::default());
            }
        }
        for pair in lines.windows(2) {
            let (case, reply) = (pair[0].trim(), pair[1].trim());
            if let (Some(name), Some(words)) = (
                case.strip_suffix(')'),
                reply.strip_prefix("COMPREPLY=($(compgen -W \""),
            ) {
                let words = &words[..words.find('"').unwrap()];
                options.get_mut(name).unwrap().values =
                    words.split(' ').map(String::from).collect();
            }
        }
        scripts.insert(command, options);
    }
    agrees_with_help("bash", scripts, false, true);
}

/// One zsh option spec, such as `'(--no-bars)--show-bars=[Help]::KIND:((a\:"A"))'`.
fn zsh_spec(spec: &str, options: &mut Options) {
    let spec = spec.strip_prefix('\'').unwrap().strip_suffix('\'').unwrap();
    let spec = spec.trim_start_matches('*');
    let spec = match spec.strip_prefix('(') {
        Some(rest) => &rest[rest.find(')').unwrap() + 1..],
        None => spec,
    };
    if !spec.starts_with('-') {
        return;
    }
    let name = spec.split(['=', '[']).next().unwrap().to_owned();
    // The help bracket ends at the first `]` that is not escaped.
    let bytes = spec.as_bytes();
    let close = (0..bytes.len())
        .find(|&at| bytes[at] == b']' && bytes[at - 1] != b'\\')
        .unwrap();
    let rest = spec[close + 1..].trim_start_matches(':');
    let mut parts = rest.splitn(2, ':');
    let value = parts.next().filter(|v| !v.is_empty()).map(String::from);
    let completion = parts.next().unwrap_or_default();
    let values = if let Some(described) = completion.strip_prefix("((") {
        described
            .split(['\n', ' '])
            .filter_map(|item| item.split_once("\\:").map(|(name, _)| name.to_owned()))
            .collect()
    } else if let Some(plain) = completion.strip_prefix('(') {
        plain[..plain.find(')').unwrap()]
            .split(' ')
            .map(String::from)
            .collect()
    } else {
        Vec::new()
    };
    options.insert(name, Spec { value, values });
}

/// zsh fills positional slots in order, so a root positional before the
/// command slot takes the command name as a directory and completes root
/// options after it: `locmetry compare --m` offered `--markdown-depth`
/// (manual completion check, zsh 5.9, 2026-10-24). A command name is
/// reserved and never a directory, so the command is the first slot.
#[test]
fn zsh_dispatches_on_the_first_word_as_the_command() {
    let script = generate("complete-zsh");
    let root = block(
        &script,
        "local context curcontext=\"$curcontext\" state line",
        |line| line.trim() == "&& ret=0",
    );
    let positionals: Vec<&str> = root
        .iter()
        .map(|line| line.trim())
        .filter(|line| line.starts_with("'::") || line.starts_with("\"::"))
        .collect();
    assert_eq!(positionals, [r#"":: :_locmetry_commands" \"#]);
    assert!(script.contains("        case $line[1] in\n"));
}

/// A first word that names no command is the directory, so the dispatch's
/// default branch completes the root's options after it, as before the
/// command became the first slot.
#[test]
fn zsh_completes_root_options_after_a_directory() {
    let script = generate("complete-zsh");
    let function = &script[..script.find("\n}\n").unwrap()];
    let default = &function[function.rfind("(?*)\n").expect("a default branch")..];
    assert_eq!(zsh_options(default.lines()), help_options(""));
}

/// The option specs among `lines`, some of which span several lines.
fn zsh_options<'a>(lines: impl IntoIterator<Item = &'a str>) -> Options {
    let mut options = Options::new();
    let mut spec = String::new();
    for line in lines {
        if line.starts_with('\'') {
            spec.clear();
        }
        spec.push_str(line);
        spec.push('\n');
        if line.ends_with("' \\") {
            zsh_spec(spec.trim_end().trim_end_matches(" \\"), &mut options);
            spec.clear();
        }
    }
    options
}

#[test]
fn zsh_completes_each_commands_options_value_names_and_possible_values() {
    let script = generate("complete-zsh");
    let mut scripts = BTreeMap::new();
    for command in COMMANDS {
        let lines = match command {
            "" => block(
                &script,
                "local context curcontext=\"$curcontext\" state line",
                |l| l.trim() == "&& ret=0",
            ),
            name => block(&script, &format!("({name})"), |l| l.trim() == "&& ret=0"),
        };
        scripts.insert(command, zsh_options(lines));
    }
    agrees_with_help("zsh", scripts, true, true);
}

#[test]
fn fish_completes_each_commands_options_and_possible_values() {
    let script = generate("complete-fish");
    let mut scripts: BTreeMap<&str, Options> = BTreeMap::new();
    for statement in script.split("\ncomplete -c locmetry ").skip(1) {
        let condition = statement.split('"').nth(1).unwrap();
        let Some(command) = COMMANDS.into_iter().find(|command| match *command {
            "" => condition == "__fish_locmetry_needs_command",
            name => condition == format!("__fish_locmetry_using_subcommand {name}"),
        }) else {
            continue;
        };
        let words: Vec<&str> = statement.split(' ').collect();
        let mut names = Vec::new();
        for pair in words.windows(2) {
            match pair[0] {
                "-s" => names.push(format!("-{}", pair[1])),
                "-l" => names.push(format!("--{}", pair[1])),
                _ => {}
            }
        }
        if names.is_empty() {
            continue;
        }
        let values: Vec<String> = statement
            .split_once(" -a \"")
            .map(|(_, list)| {
                list[..list.rfind('"').unwrap()]
                    .lines()
                    .map(|item| item.split("\\t").next().unwrap().to_owned())
                    .collect()
            })
            .unwrap_or_default();
        let options = scripts.entry(command).or_default();
        for name in names {
            let spec = Spec {
                value: None,
                values: values.clone(),
            };
            options.insert(name, spec);
        }
    }
    agrees_with_help("fish", scripts, false, true);
}

/// PowerShell completes names only, as clap_complete generates it; the
/// reference guide says so.
#[test]
fn powershell_completes_each_commands_options() {
    let script = generate("complete-powershell");
    let mut scripts = BTreeMap::new();
    for command in COMMANDS {
        let scope = match command {
            "" => "'locmetry' {".to_owned(),
            name => format!("'locmetry;{name}' {{"),
        };
        let lines = block(&script, &scope, |line| line.trim() == "break");
        let options = lines
            .iter()
            .filter(|line| line.contains("[CompletionResultType]::ParameterName"))
            .map(|line| {
                let name = line.split('\'').nth(1).unwrap().to_owned();
                (name, Spec::default())
            })
            .collect();
        scripts.insert(command, options);
    }
    agrees_with_help("powershell", scripts, false, false);
}

/// The page with roff font changes and escapes removed, so its text compares
/// with `--help`.
fn man_text() -> String {
    generate("man")
        .replace("\\fB", "")
        .replace("\\fI", "")
        .replace("\\fR", "")
        .replace("\\-", "-")
        .replace("\\*(Aq", "'")
}

/// The root's part of the page, before COMMANDS, or a command's subsection.
fn man_scope(page: &str, command: &str) -> Vec<String> {
    let lines: Vec<&str> = match command {
        "" => page
            .lines()
            .take_while(|line| *line != ".SH COMMANDS")
            .collect(),
        name => block(page, &format!(".SS \"locmetry {name}\""), |line| {
            line.starts_with(".SS ") || line.starts_with(".SH ")
        }),
    };
    lines.into_iter().map(String::from).collect()
}

#[test]
fn the_man_page_lists_each_commands_options_value_names_and_possible_values() {
    let page = man_text();
    let mut scripts = BTreeMap::new();
    for command in COMMANDS {
        let lines = man_scope(&page, command);
        let mut options = Options::new();
        let mut current = Vec::new();
        for pair in lines.windows(2) {
            if pair[0] == ".TP" && pair[1].starts_with('-') {
                let (names, value) = header(&pair[1]);
                for name in &names {
                    let spec = Spec {
                        value: value.clone(),
                        values: Vec::new(),
                    };
                    options.insert(name.clone(), spec);
                }
                current = names;
            } else if pair[0] == ".TP" {
                current.clear();
            } else if pair[0] == ".IP \\(bu 2" {
                let value = pair[1].split(':').next().unwrap().to_owned();
                for name in &current {
                    options.get_mut(name).unwrap().values.push(value.clone());
                }
            }
        }
        scripts.insert(command, options);
    }
    agrees_with_help("man", scripts, true, true);
}

/// `--help` lists positional arguments under their own heading, before the
/// options; clap_mangen 0.3.3 lists them among the options.
#[test]
fn the_man_page_lists_each_commands_positional_arguments_before_its_options() {
    let page = man_text();
    for command in COMMANDS {
        let lines = man_scope(&page, command);
        let positionals = help_positionals(command);
        assert!(command.is_empty() || !positionals.is_empty() || command == "workspace");
        let (arguments, options) = match command {
            "" => (".SH ARGUMENTS", ".SH OPTIONS"),
            _ => (".B \"Arguments\"", ".B \"Options\""),
        };
        let start = lines.iter().position(|line| line == arguments);
        if positionals.is_empty() {
            assert_eq!(start, None, "'{command}' has no positional arguments");
            continue;
        }
        let start = start.unwrap_or_else(|| panic!("'{command}' lacks {arguments}"));
        let heading = |line: &String| line.starts_with(".SH ") || line.starts_with(".B \"");
        let end = start + 1 + lines[start + 1..].iter().position(heading).unwrap();
        assert_eq!(
            lines[end], options,
            "'{command}' lists its options after its arguments"
        );
        for (syntax, description) in positionals {
            assert!(
                lines[start..end]
                    .windows(3)
                    .any(|w| w[0] == ".TP" && w[1] == syntax && w[2].starts_with(&description)),
                "'{command}' lacks {syntax} under {arguments}: {description}"
            );
        }
        assert!(
            !lines[end..]
                .windows(2)
                .any(|w| w[0] == ".TP" && (w[1].starts_with('<') || w[1].starts_with('['))),
            "'{command}' lists positional arguments among its options"
        );
    }
}

#[test]
fn the_man_page_ends_with_the_reference_sections_and_every_guide_topic() {
    let page = man_text();
    let headings: Vec<&str> = page
        .lines()
        .filter_map(|line| line.strip_prefix(".SH "))
        .collect();
    assert_eq!(
        headings[headings.len() - 4..],
        ["\"EXIT STATUS\"", "ENVIRONMENT", "FILES", "\"SEE ALSO\""]
    );
    let see_also = page.split(".SH \"SEE ALSO\"").nth(1).unwrap();
    let topics = &help_options("")["--guide"].values;
    assert!(topics.len() > 5, "{topics:?}");
    for topic in topics {
        assert!(
            see_also.contains(&format!("locmetry --guide {topic}\n")),
            "{topic}"
        );
    }
    assert!(!page.contains("--generate") && !page.contains("locmetry help"));
}

/// The page dates itself by its version's changelog heading, which the
/// release command writes with the version (scripts/release.py).
#[test]
fn the_man_page_names_its_versions_release_date() {
    let version = env!("CARGO_PKG_VERSION");
    let changelog = common::repository_file("CHANGELOG.md");
    let heading = format!("## [{version}] - ");
    let date = changelog
        .lines()
        .find_map(|line| line.strip_prefix(heading.as_str()))
        .unwrap();
    let title = format!(".TH LOCMETRY 1 {date} \"locmetry {version}\" \"User Commands\"");
    assert!(generate("man").lines().any(|line| line == title), "{title}");
}

#[test]
fn completion_scripts_leave_out_the_generation_query() {
    for shell in ["bash", "zsh", "fish", "powershell"] {
        assert!(!generate(&format!("complete-{shell}")).contains("generate"));
    }
}

/// groff hyphenates words at line ends, which split option names such as
/// `--workspace-file` into `--work-` and `space-file` (groff 1.23, 2026-10-24);
/// `.nh` after the title turns hyphenation off for the whole page.
#[test]
fn the_man_page_turns_off_hyphenation() {
    let page = generate("man");
    let mut lines = page.lines().skip_while(|line| !line.starts_with(".TH "));
    lines.next();
    assert_eq!(lines.next(), Some(".nh"));
}

/// clap_mangen 0.3.3 sets each possible value as an `.IP` paragraph, and
/// `.IP` keeps the paragraph gap, so every value list was double-spaced in
/// `man` (reported 2026-10-25) while `--help` lists them line by line.
#[test]
fn the_man_page_lists_possible_values_without_gaps_between_them() {
    let page = generate("man");
    let lines: Vec<&str> = page.lines().collect();
    let lists: Vec<usize> = (1..lines.len())
        .filter(|&at| lines[at - 1] == "\\fIPossible values:\\fR")
        .collect();
    assert!(!lists.is_empty());
    for at in lists {
        assert!(lines[at].starts_with(".RS "), "{}", lines[at]);
        assert_eq!(lines[at + 1], ".PD 0", "after {}", lines[at - 1]);
        let end = at + lines[at..].iter().position(|line| *line == ".RE").unwrap();
        assert_eq!(
            lines.get(end + 1),
            Some(&".PD"),
            "after the list at line {at}"
        );
    }
}
