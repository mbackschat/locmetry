use super::*;
use crate::style::{Role, Theme};
use std::fmt::Write;

mod notes;

fn label(value: &str, markdown: bool) -> String {
    match markdown {
        true => crate::output::cell(value),
        false => crate::style::plain(value),
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
    let headers = [
        "Source",
        "Lines before",
        "Lines after",
        "Lines change",
        "Code before",
        "Code after",
        "Code change",
    ];
    let cells = |name: &str, c: &CountChange| {
        [
            label(name, markdown),
            c.before.lines.to_string(),
            c.after.lines.to_string(),
            format!("{:+}", c.delta.lines),
            c.before.code.to_string(),
            c.after.code.to_string(),
            format!("{:+}", c.delta.code),
        ]
    };
    let widths = rows
        .iter()
        .fold(headers.map(str::len), |mut widths, (name, c)| {
            for (width, value) in widths.iter_mut().zip(cells(name, c)) {
                *width = (*width).max(value.chars().count());
            }
            widths
        });
    let text_row = |values: &[String; 7]| {
        values
            .iter()
            .zip(widths)
            .enumerate()
            .map(|(index, (value, width))| match index {
                0 => format!("{value:<width$}"),
                _ => format!("{value:>width$}"),
            })
            .collect::<Vec<_>>()
    };
    if markdown {
        writeln!(
            output,
            "\n## {title}\n\n| {} |\n| --- |{}\n",
            headers.join(" | "),
            " ---: |".repeat(6)
        )
        .unwrap();
    } else {
        writeln!(
            output,
            "\n{}\n{}",
            theme.paint(Role::Heading, title),
            theme.paint(
                Role::Heading,
                &text_row(&headers.map(String::from)).join("  ")
            )
        )
        .unwrap();
    }
    for (name, c) in rows {
        let values = cells(&name, &c);
        let mut rendered = match markdown {
            true => values.to_vec(),
            false => text_row(&values),
        };
        for (index, delta) in [(3, c.delta.lines), (6, c.delta.code)] {
            rendered[index] = theme.delta(delta, if markdown { 0 } else { widths[index] });
        }
        match markdown {
            true => writeln!(output, "| {} |", rendered.join(" | ")),
            false => writeln!(output, "{}", rendered.join("  ")),
        }
        .unwrap();
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
    out.push('\n');
    for note in [
        "Net change = after − before, from the two saved reports.",
        "Unreadable files count as 0; a folder seen in only one report may be newly visible, not newly created.",
    ] {
        writeln!(out, "{}", theme.paint(Role::Muted, note)).unwrap();
    }
    let totals: Vec<_> = [
        ("All counted", c.summary.dimensions.all_counted),
        ("Source", c.summary.dimensions.source),
    ]
    .into_iter()
    .flat_map(|(label, change)| {
        [
            (
                "files",
                change.before.files,
                change.after.files,
                change.delta.files,
            ),
            (
                "lines",
                change.before.lines,
                change.after.lines,
                change.delta.lines,
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
        ]
        .map(|(metric, before, after, delta)| (format!("{label} {metric}"), before, after, delta))
    })
    .collect();
    let headers = ["Measure", "Before", "After", "Net change"];
    let [measure_width, before_width, after_width, delta_width] = totals.iter().fold(
        headers.map(str::len),
        |mut widths, (name, before, after, delta)| {
            for (width, required) in widths.iter_mut().zip([
                name.len(),
                before.to_string().len(),
                after.to_string().len(),
                format!("{delta:+}").len(),
            ]) {
                *width = (*width).max(required);
            }
            widths
        },
    );
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
                    "{:<measure_width$}  {:>before_width$}  {:>after_width$}  {:>delta_width$}",
                    headers[0], headers[1], headers[2], headers[3]
                )
            )
        )
        .unwrap();
    }
    for (name, before, after, delta) in totals {
        if markdown {
            writeln!(
                out,
                "| {name} | {before} | {after} | {} |",
                theme.delta(delta, 0)
            )
            .unwrap();
        } else {
            writeln!(
                out,
                "{}  {before:>before_width$}  {after:>after_width$}  {}",
                theme.paint(Role::Heading, &format!("{name:<measure_width$}")),
                theme.delta(delta, delta_width)
            )
            .unwrap();
        }
    }
    grid(
        &mut out,
        "Purpose",
        c.summary
            .dimensions
            .source_purpose
            .iter()
            .filter(|(_, v)| v.delta.code != 0 || v.delta.lines != 0)
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
            .filter(|(_, v)| v.delta.code != 0 || v.delta.lines != 0)
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
                        || v.dimensions.source.delta.code != 0
                        || v.dimensions.source.delta.lines != 0
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
            "\nWorkspace source lines: matched {:+}, entering +{}, leaving -{}; total {:+}.",
            composition.source.matched.lines,
            composition.source.entering.lines,
            composition.source.leaving.lines,
            c.summary.dimensions.source.delta.lines
        )
        .unwrap();
        writeln!(
            out,
            "Workspace all-counted lines: matched {:+}, entering +{}, leaving -{}; total {:+}.",
            composition.all_counted.matched.lines,
            composition.all_counted.entering.lines,
            composition.all_counted.leaving.lines,
            c.summary.dimensions.all_counted.delta.lines
        )
        .unwrap();
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
                            || v.recursive.source.delta.code != 0
                            || v.recursive.source.delta.lines != 0)
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
    let summary = &c.presence_summary;
    let presence: Vec<_> = [
        (summary.members.before_only, "member", "before"),
        (summary.members.after_only, "member", "after"),
        (summary.folders.before_only, "folder", "before"),
        (summary.folders.after_only, "folder", "after"),
        (summary.packages.before_only, "package", "before"),
        (summary.packages.after_only, "package", "after"),
    ]
    .into_iter()
    .filter(|(count, _, _)| *count > 0)
    .map(|(count, noun, side)| format!("{} only {side}", crate::output::counted(count, noun)))
    .collect();
    if !presence.is_empty() {
        writeln!(out, "\nOnly in one report: {}.", presence.join("; ")).unwrap();
    }
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
            writeln!(
                out,
                "\n- {} ({:?}{}): {}",
                theme.paint(Role::Information, f.code()),
                context.side,
                context
                    .member
                    .as_ref()
                    .map_or(String::new(), |m| format!(", {}", label(m, markdown))),
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
    writeln!(
        out,
        "\nBefore measured at: {}\nAfter measured at: {}",
        crate::measurement_time(c.before.metadata.measured_at_unix_seconds),
        crate::measurement_time(c.after.metadata.measured_at_unix_seconds)
    )
    .unwrap();
    out
}
fn presence_name(p: Presence) -> &'static str {
    match p {
        Presence::Both => "both",
        Presence::BeforeOnly => "before only",
        Presence::AfterOnly => "after only",
    }
}
pub fn render_markdown(c: &Comparison) -> String {
    render(c, true, Theme::MARKDOWN)
}

pub fn render_table_with_theme(c: &Comparison, theme: Theme) -> String {
    render(c, false, theme)
}
