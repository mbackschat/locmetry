use super::*;
use std::fmt::Write;

fn safe(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('|', "&#124;")
        .replace(['\r', '\n', '\t'], " ")
}
fn grid(output: &mut String, title: &str, rows: Vec<(String, CountChange)>, markdown: bool) {
    if rows.is_empty() {
        return;
    }
    if markdown {
        writeln!(output,"\n## {title}\n\n| Source code | Before | After | Net change |\n| --- | ---: | ---: | ---: |").unwrap();
    } else {
        writeln!(
            output,
            "\n{title}\n{:<40} {:>12} {:>12} {:>12}",
            "Source code", "Before", "After", "Net change"
        )
        .unwrap();
    }
    for (label, c) in rows {
        if markdown {
            writeln!(
                output,
                "| {} | {} | {} | {:+} |",
                safe(&label),
                c.before.code,
                c.after.code,
                c.delta.code
            )
            .unwrap();
        } else {
            writeln!(
                output,
                "{:<40} {:>12} {:>12} {:>+12}",
                safe(&label),
                c.before.code,
                c.after.code,
                c.delta.code
            )
            .unwrap();
        }
    }
}
fn render(c: &Comparison, markdown: bool) -> String {
    let mut out = String::new();
    writeln!(
        out,
        "{}Net change: {} → {}",
        if markdown { "# " } else { "" },
        safe(&c.before.project),
        safe(&c.after.project)
    )
    .unwrap();
    writeln!(out,"\nAfter minus before, over recorded counts. Unavailable lines are not estimated; presence does not prove physical creation or deletion.").unwrap();
    if markdown {
        out.push_str("\n## Totals\n\n| Measure | Before | After | Net change |\n| --- | ---: | ---: | ---: |\n");
    } else {
        out.push_str(
            "\nTotals\nMeasure                                  Before        After   Net change\n",
        );
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
                writeln!(out, "| {label} {metric} | {b} | {a} | {d:+} |").unwrap();
            } else {
                writeln!(
                    out,
                    "{:<40} {b:>12} {a:>12} {d:>+12}",
                    format!("{label} {metric}")
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
    if !c.findings.is_empty() {
        writeln!(out, "\n{}Findings", if markdown { "## " } else { "" }).unwrap();
        for f in &c.findings {
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
                f.code(),
                context.side,
                context
                    .member
                    .as_ref()
                    .map_or(String::new(), |m| format!(", {}", safe(m))),
                safe(&entity),
                safe(&context.message)
            )
            .unwrap();
        }
    } else {
        out.push_str("\nNo recorded differences in measurement settings. Reports do not fingerprint source, ignore files or executable builds; this does not prove source-only growth.\n");
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
    render(c, false)
}
pub fn render_markdown(c: &Comparison) -> String {
    render(c, true)
}
