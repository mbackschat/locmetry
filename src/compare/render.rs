use super::*;
use crate::style::{Role, Theme};
use std::fmt::Write;

mod notes;

fn label(value: &str, markdown: bool) -> String {
    match markdown {
        true => crate::output::cell(value),
        false => crate::output::plain(value),
    }
}
fn grid(
    output: &mut String,
    title: &str,
    rows: Vec<(String, CountChange)>,
    markdown: bool,
    theme: Theme,
) {
    if rows.is_empty() {
        return;
    }
    if markdown {
        writeln!(output,"\n## {title}\n\n| Source code | Before | After | Net change |\n| --- | ---: | ---: | ---: |").unwrap();
    } else {
        writeln!(
            output,
            "\n{}\n{}",
            theme.paint(Role::Heading, title),
            theme.paint(
                Role::Heading,
                &format!(
                    "{:<40} {:>12} {:>12} {:>12}",
                    "Source code", "Before", "After", "Net change"
                )
            )
        )
        .unwrap();
    }
    for (label, c) in rows {
        if markdown {
            writeln!(
                output,
                "| {} | {} | {} | {} |",
                self::label(&label, markdown),
                c.before.code,
                c.after.code,
                theme.delta(c.delta.code, 0)
            )
            .unwrap();
        } else {
            writeln!(
                output,
                "{:<40} {:>12} {:>12} {}",
                self::label(&label, markdown),
                c.before.code,
                c.after.code,
                theme.delta(c.delta.code, 12)
            )
            .unwrap();
        }
    }
}
fn render(c: &Comparison, markdown: bool, theme: Theme) -> String {
    let mut out = String::new();
    let title = format!(
        "Net change: {} → {}",
        label(&c.before.project, markdown),
        label(&c.after.project, markdown)
    );
    match markdown {
        true => writeln!(out, "# {title}"),
        false => writeln!(out, "{}", theme.paint(Role::Heading, &title)),
    }
    .unwrap();
    if let Some(folder) = c
        .before
        .metadata
        .scope
        .as_ref()
        .and_then(|s| s.subfolder.as_deref())
    {
        writeln!(
            out,
            "Subfolder: {} (project-relative; measured contributions only)",
            label(folder, markdown)
        )
        .unwrap();
    }
    writeln!(out,"\n{}", theme.paint(Role::Muted, "After minus before, over recorded counts. Unavailable lines are not estimated; presence does not prove physical creation or deletion.")).unwrap();
    if markdown {
        out.push_str("\n## Totals\n\n| Measure | Before | After | Net change |\n| --- | ---: | ---: | ---: |\n");
    } else {
        writeln!(
            out,
            "\n{}\n{}",
            theme.paint(Role::Heading, "Totals"),
            theme.paint(
                Role::Heading,
                &format!(
                    "{:<40} {:>12} {:>12} {:>12}",
                    "Measure", "Before", "After", "Net change"
                )
            )
        )
        .unwrap();
    }
    for (label, change) in [
        ("All counted", c.summary.dimensions.all_counted),
        ("Source", c.summary.dimensions.source),
    ] {
        for (metric, b, a, d) in [
            (
                "files",
                change.before.files,
                change.after.files,
                change.delta.files,
            ),
            (
                "code",
                change.before.code,
                change.after.code,
                change.delta.code,
            ),
            (
                "comments",
                change.before.comments,
                change.after.comments,
                change.delta.comments,
            ),
            (
                "blanks",
                change.before.blanks,
                change.after.blanks,
                change.delta.blanks,
            ),
            (
                "lines",
                change.before.lines,
                change.after.lines,
                change.delta.lines,
            ),
        ] {
            if markdown {
                writeln!(
                    out,
                    "| {label} {metric} | {b} | {a} | {} |",
                    theme.delta(d, 0)
                )
                .unwrap();
            } else {
                writeln!(
                    out,
                    "{} {b:>12} {a:>12} {}",
                    theme.paint(
                        Role::Heading,
                        &format!("{:<40}", format!("{label} {metric}"))
                    ),
                    theme.delta(d, 12)
                )
                .unwrap();
            }
        }
    }
    grid(
        &mut out,
        "Purpose",
        c.summary
            .dimensions
            .source_purpose
            .iter()
            .filter(|(_, v)| v.changed())
            .map(|(k, v)| (k.to_string(), *v))
            .collect(),
        markdown,
        theme,
    );
    grid(
        &mut out,
        "Languages",
        c.summary
            .dimensions
            .source_languages
            .iter()
            .filter(|(_, v)| v.changed())
            .map(|(k, v)| (k.clone(), *v))
            .collect(),
        markdown,
        theme,
    );
    if let Some(members) = &c.members {
        grid(
            &mut out,
            "Workspace members",
            members
                .iter()
                .filter(|(_, v)| {
                    v.presence != Presence::Both
                        || v.dimensions.source.changed()
                        || v.dimensions.all_counted.changed()
                })
                .map(|(k, v)| {
                    (
                        format!("{k} ({})", presence_name(v.presence)),
                        v.dimensions.source,
                    )
                })
                .collect(),
            markdown,
            theme,
        );
    }
    if let Some(composition) = &c.summary.composition {
        writeln!(
            out,
            "\nWorkspace source code: matched {:+}, entering +{}, leaving -{}; total {:+}.",
            composition.source.matched.code,
            composition.source.entering.code,
            composition.source.leaving.code,
            c.summary.dimensions.source.delta.code
        )
        .unwrap();
        writeln!(
            out,
            "Workspace all-counted code: matched {:+}, entering +{}, leaving -{}; total {:+}.",
            composition.all_counted.matched.code,
            composition.all_counted.entering.code,
            composition.all_counted.leaving.code,
            c.summary.dimensions.all_counted.delta.code
        )
        .unwrap();
    }
    if let Some(folders) = &c.folders {
        let mut folders: Vec<_> = folders.iter().collect();
        folders.sort_by(|(a, _), (b, _)| a.split('/').cmp(b.split('/')));
        grid(
            &mut out,
            "Folders (recursive; do not sum parents and children)",
            folders
                .into_iter()
                .filter(|(path, v)| {
                    (markdown
                        || (c.members.is_none() && (path.as_str() == "." || !path.contains('/'))))
                        && (v.presence != Presence::Both
                            || v.recursive.source.changed()
                            || v.recursive.all_counted.changed())
                })
                .map(|(k, v)| {
                    (
                        format!("{k} ({})", presence_name(v.presence)),
                        v.recursive.source,
                    )
                })
                .collect(),
            markdown,
            theme,
        );
    }
    writeln!(
        out,
        "\nPresence: members -{} +{}; folders -{} +{}; packages -{} +{}.",
        c.presence_summary.members.before_only,
        c.presence_summary.members.after_only,
        c.presence_summary.folders.before_only,
        c.presence_summary.folders.after_only,
        c.presence_summary.packages.before_only,
        c.presence_summary.packages.after_only
    )
    .unwrap();
    let notes = notes::measurement_notes(c);
    if !notes.is_empty() {
        match markdown {
            true => writeln!(out, "\n## Measurement notes\n"),
            false => writeln!(out, "\n{}", theme.paint(Role::Heading, "Measurement notes")),
        }
        .unwrap();
        for (role, note) in notes {
            writeln!(out, "- {}", theme.paint(role, &label(&note, markdown))).unwrap();
        }
    }
    let findings: Vec<_> = c
        .findings
        .iter()
        .filter(|f| {
            !matches!(
                f,
                Finding::CoverageGap { .. } | Finding::CounterChanged { .. }
            )
        })
        .collect();
    if !findings.is_empty() {
        writeln!(out, "\n{}Findings", if markdown { "## " } else { "" }).unwrap();
        for f in findings {
            let context = f.context();
            let entity = match f {
                Finding::PresenceChanged { entity, .. } => match entity {
                    Entity::Member { name } => format!("member {name}: "),
                    Entity::Folder { path } => format!("folder {path}: "),
                    Entity::Package { path, detector } => format!("package {path} ({detector}): "),
                },
                _ => String::new(),
            };
            writeln!(
                out,
                "\n- {} ({:?}{}): {}{}",
                theme.paint(Role::Information, f.code()),
                context.side,
                context
                    .member
                    .as_ref()
                    .map_or(String::new(), |m| format!(", {}", label(m, markdown))),
                label(&entity, markdown),
                label(&context.message, markdown)
            )
            .unwrap();
        }
    }
    if !c.findings.iter().any(|f| {
        matches!(
            f,
            Finding::CounterChanged { .. }
                | Finding::PolicyChanged { .. }
                | Finding::ModulesChanged { .. }
                | Finding::IdentityChanged { .. }
        )
    }) {
        writeln!(out, "\n{}", theme.paint(Role::Muted, "No recorded differences in measurement settings. Reports do not fingerprint source, ignore files or executable builds; this does not prove source-only growth.")).unwrap();
    }
    out
}
fn presence_name(p: Presence) -> &'static str {
    match p {
        Presence::Both => "both",
        Presence::BeforeOnly => "before only",
        Presence::AfterOnly => "after only",
    }
}
pub fn render_table(c: &Comparison) -> String {
    render_table_with_theme(c, Theme::PLAIN)
}
pub fn render_markdown(c: &Comparison) -> String {
    render(c, true, Theme::MARKDOWN)
}

pub fn render_table_with_theme(c: &Comparison, theme: Theme) -> String {
    render(c, false, theme)
}
