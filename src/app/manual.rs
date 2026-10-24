//! The combined `locmetry(1)` man page. Every part comes from text that
//! `--help` or a guide already shows, so the page cannot drift from them
//! (docs/proposals/manual-and-reference.md, The man page).
use super::*;
use clap_mangen::roff::{bold, roman, Inline, Roff};
use clap_mangen::Man;

/// The reference guide sections the page renders, in order.
const REFERENCE_SECTIONS: [&str; 3] = ["exit-status", "environment", "files"];

/// The release date of this version, which the release command writes into
/// its changelog heading together with the version in Cargo.toml
/// (scripts/release.py), so every build names the date its version was
/// released; the page has no other date to name, and `mandoc -T lint`
/// reports a missing one.
fn release_date() -> &'static str {
    let heading = format!("## [{}] - ", env!("CARGO_PKG_VERSION"));
    let changelog = include_str!("../../CHANGELOG.md");
    changelog
        .lines()
        .find_map(|line| line.strip_prefix(heading.as_str()))
        .unwrap_or_default()
}

pub(crate) fn manual() -> Result<Vec<u8>> {
    let mut root = Cli::command()
        .disable_help_subcommand(true)
        .subcommand_value_name("COMMAND");
    root.build();
    let root = with_usage(root);
    let man = Man::new(root.clone())
        .title("LOCMETRY")
        .date(release_date())
        .manual("User Commands");
    let mut page = Page::default();
    page.man(|w| man.render_title(w))?;
    let mut name = Roff::new();
    name.control("SH", ["NAME"]);
    let about = root
        .get_about()
        .map(ToString::to_string)
        .unwrap_or_default();
    let summary = about.split(". ").next().unwrap_or_default();
    name.text([roman(format!("locmetry - {summary}"))]);
    page.roff(&name);
    page.man(|w| man.render_synopsis_section(w))?;
    page.man(|w| man.render_description_section(w))?;
    page.man(|w| man.render_options_section(w))?;

    let mut notes_section = Roff::new();
    notes_section.control("SH", ["EXAMPLES AND NOTES"]);
    notes(&mut notes_section, &after_help(&root));
    notes_section.control("SH", ["COMMANDS"]);
    page.roff(&notes_section);
    for command in root.get_subcommands() {
        let command = with_usage(command.clone());
        let man = Man::new(command.clone());
        let mut heading = Roff::new();
        heading.control("SS", [command.get_bin_name().unwrap_or_default()]);
        page.roff(&heading);
        for render in [
            Man::render_description_section,
            Man::render_synopsis_section,
            Man::render_options_section,
        ] {
            page.labeled(&command, |w| render(&man, w))?;
        }
        let mut examples = Roff::new();
        notes(&mut examples, &after_help(&command));
        page.roff(&examples);
    }

    let mut tail = Roff::new();
    let (_, reference) = Guide::Reference.document();
    for id in REFERENCE_SECTIONS {
        let text = crate::guides::section(reference, id).expect("the reference guide has it");
        let (heading, body) = text.split_once('\n').unwrap_or((text, ""));
        tail.control(
            "SH",
            [heading.trim_start_matches("## ").to_uppercase().as_str()],
        );
        markdown(&mut tail, body);
    }
    tail.control("SH", ["SEE ALSO"]);
    for guide in Guide::value_variants() {
        let topic = guide.to_possible_value().expect("every topic is listed");
        tail.control("TP", []);
        tail.text([bold(format!("locmetry --guide {}", topic.get_name()))]);
        if let Some(help) = topic.get_help() {
            tail.text([roman(help.to_string())]);
        }
    }
    page.roff(&tail);
    Ok(page.finish().into_bytes())
}

/// The synopsis `--help` prints, instead of clap_mangen's list of every
/// option without its value.
fn with_usage(mut command: clap::Command) -> clap::Command {
    let usage = command.render_usage().to_string();
    let usage = usage.trim_start_matches("Usage: ").to_owned();
    command.override_usage(usage)
}

/// The page as it is assembled from clap_mangen's section renderers and
/// Locmetry's own sections.
#[derive(Default)]
struct Page {
    text: String,
}

impl Page {
    /// Appends a rendering without its preamble, which every clap_mangen
    /// renderer and `Roff::render` repeat; `finish` adds it once.
    fn push(&mut self, rendered: String) {
        let preamble = Roff::new().render();
        self.text += rendered.strip_prefix(&preamble).unwrap_or(&rendered);
    }

    fn roff(&mut self, roff: &Roff) {
        self.push(roff.render());
    }

    fn man(&mut self, render: impl FnOnce(&mut Vec<u8>) -> io::Result<()>) -> Result<()> {
        let mut rendered = Vec::new();
        render(&mut rendered)?;
        self.push(String::from_utf8(rendered)?);
        Ok(())
    }

    /// Appends a command's section with its headings as bold labels, because
    /// the command is itself a subsection of COMMANDS. The renderer
    /// uppercases option headings; a label restores the heading's spelling.
    fn labeled(
        &mut self,
        command: &clap::Command,
        render: impl FnOnce(&mut Vec<u8>) -> io::Result<()>,
    ) -> Result<()> {
        let mut rendered = Vec::new();
        render(&mut rendered)?;
        let section: String = String::from_utf8(rendered)?
            .lines()
            .map(|line| match line.strip_prefix(".SH ") {
                Some("SYNOPSIS" | "DESCRIPTION") => ".PP".to_owned(),
                Some(heading) => {
                    let heading = heading.trim_matches('"');
                    let label = command
                        .get_arguments()
                        .filter_map(|arg| arg.get_help_heading())
                        .find(|own| own.to_uppercase() == heading)
                        .unwrap_or("Options");
                    format!(".PP\n.B \"{label}\"")
                }
                None => line.to_owned(),
            })
            .map(|line| line + "\n")
            .collect();
        self.push(section);
        Ok(())
    }

    /// The preamble once, and two repairs that keep `mandoc -T lint` free of
    /// warnings: a paragraph macro directly after a heading is redundant, and
    /// clap_mangen 0.3.3 separates help from possible values by two breaks
    /// around a blank line, which is one vertical space.
    fn finish(self) -> String {
        let text = self.text.replace(".br\n\n.br\n", ".sp\n");
        let mut page = Roff::new().render();
        let mut previous = "";
        for line in text.lines() {
            let heading = previous.starts_with(".SH ") || previous.starts_with(".SS ");
            if !(heading && line == ".PP") {
                page += line;
                page += "\n";
            }
            previous = line;
        }
        page
    }
}

fn after_help(command: &clap::Command) -> String {
    command
        .get_after_long_help()
        .or_else(|| command.get_after_help())
        .map(ToString::to_string)
        .unwrap_or_default()
}

/// Renders help notes as clap prints them: a line at the margin that ends in
/// a colon and starts a block is a heading, other lines at the margin or two
/// spaces in are prose to refill, and lines indented further are commands or
/// files to keep as they are.
fn notes(roff: &mut Roff, text: &str) {
    let mut prose: Vec<&str> = Vec::new();
    let mut literal: Vec<&str> = Vec::new();
    for line in text.lines() {
        let indent = line.len() - line.trim_start().len();
        if line.trim().is_empty() {
            if literal.is_empty() {
                paragraph(roff, &mut prose);
            } else {
                literal.push("");
            }
        } else if indent >= 4 {
            paragraph(roff, &mut prose);
            literal.push(&line[4..]);
        } else {
            block(roff, &mut literal);
            if indent == 0 && line.ends_with(':') && prose.is_empty() {
                roff.control("PP", []);
                roff.text([bold(line)]);
            } else {
                prose.push(line.trim());
            }
        }
    }
    paragraph(roff, &mut prose);
    block(roff, &mut literal);
}

fn paragraph(roff: &mut Roff, lines: &mut Vec<&str>) {
    if !lines.is_empty() {
        roff.control("PP", []);
        roff.text(inline(&lines.join(" ")));
        lines.clear();
    }
}

/// An indented block in no-fill mode, without its trailing blank lines.
fn block(roff: &mut Roff, lines: &mut Vec<&str>) {
    while lines.last() == Some(&"") {
        lines.pop();
    }
    if !lines.is_empty() {
        roff.control("RS", ["4"]);
        roff.control("nf", []);
        for line in lines.iter() {
            roff.text([roman(*line)]);
        }
        roff.control("fi", []);
        roff.control("RE", []);
        lines.clear();
    }
}

/// Renders the Markdown subset the reference guide keeps to
/// (tests/integration/reference.rs): paragraphs, flat bullet lists, inline
/// code in bold, code blocks, and links as their text.
fn markdown(roff: &mut Roff, text: &str) {
    let mut prose: Vec<&str> = Vec::new();
    let mut code: Option<Vec<&str>> = None;
    let mut bullet = false;
    let flush = |roff: &mut Roff, prose: &mut Vec<&str>, bullet: bool| {
        if !prose.is_empty() {
            match bullet {
                true => roff.control("IP", ["\\(bu", "2"]),
                false => roff.control("PP", []),
            };
            roff.text(inline(&prose.join(" ")));
            prose.clear();
        }
    };
    for line in text.lines() {
        if line.starts_with("```") {
            flush(roff, &mut prose, bullet);
            match code.take() {
                Some(mut lines) => block(roff, &mut lines),
                None => code = Some(Vec::new()),
            }
        } else if let Some(lines) = code.as_mut() {
            lines.push(line);
        } else if line.trim().is_empty() {
            flush(roff, &mut prose, bullet);
        } else if let Some(item) = line.strip_prefix("- ") {
            flush(roff, &mut prose, bullet);
            bullet = true;
            prose.push(item);
        } else {
            if prose.is_empty() {
                bullet = false;
            }
            prose.push(line);
        }
    }
    flush(roff, &mut prose, bullet);
}

/// Inline code in bold, a link as its text, everything else roman.
fn inline(text: &str) -> Vec<Inline> {
    text.split('`')
        .enumerate()
        .filter(|(_, part)| !part.is_empty())
        .map(|(index, part)| match index % 2 {
            1 => bold(part),
            _ => roman(link_text(part)),
        })
        .collect()
}

fn link_text(text: &str) -> String {
    let mut plain = String::new();
    let mut rest = text;
    while let Some(open) = rest.find('[') {
        let Some(close) = rest[open..].find("](").map(|at| open + at) else {
            break;
        };
        let Some(end) = rest[close..].find(')').map(|at| close + at) else {
            break;
        };
        plain.push_str(&rest[..open]);
        plain.push_str(&rest[open + 1..close]);
        rest = &rest[end + 1..];
    }
    plain.push_str(rest);
    plain
}
