//! The Markdown report.
use super::*;

/// Member rows with each row's code, and the total row: source lines,
/// the purpose columns and ratio, and share of the workspace's source code.
pub(super) fn member_rows(
    summary: &ReportSummary,
    members: &[MemberView],
    order: RowOrder,
    columns: &PurposeColumns,
) -> (Vec<Vec<String>>, Vec<RowCode>, Vec<String>) {
    let workspace = RowCode::of_totals(&summary.totals);
    let cells = |label: &str, totals: &Summary, code: RowCode| {
        let mut row = vec![
            label.to_string(),
            grouped(totals.source.lines),
            grouped(totals.source.code),
        ];
        row.extend(columns.cells(code));
        row.push(share_of(code.code(), workspace.code()));
        row
    };
    let (body, codes) = member_totals(summary, members, order)
        .into_iter()
        .map(|(name, totals)| {
            let code = RowCode::of_totals(totals);
            (cells(&name, totals, code), code)
        })
        .unzip();
    (body, codes, cells("Total", &summary.totals, workspace))
}

pub(super) fn member_columns(purposes: &PurposeColumns) -> Columns {
    let mut columns = Columns::with_purposes(&code_columns("Member"), purposes);
    columns
        .0
        .push(Column::new("Share", ColumnRole::Share).separated());
    columns
}

/// Each member's recursive totals, in table row order.
pub(super) fn member_totals<'a>(
    summary: &'a ReportSummary,
    members: &[MemberView],
    order: RowOrder,
) -> Vec<(String, &'a Summary)> {
    static EMPTY: std::sync::LazyLock<Summary> = std::sync::LazyLock::new(Summary::default);
    let rows = members
        .iter()
        .map(|member| {
            let totals = summary
                .folders
                .get(&member.name)
                .map_or(&*EMPTY, |folder| &folder.recursive);
            (member.name.clone(), totals)
        })
        .collect();
    ordered(rows, |totals| totals.source.code, order)
}

/// A Markdown grid of source lines and code per `key` and member, with totals;
/// `of` selects the map, such as categories or languages, from a summary.
pub(super) fn member_grid(
    output: &mut String,
    title: &str,
    label: &str,
    summary: &ReportSummary,
    members: &[MemberView],
    of: impl Fn(&Summary) -> &BTreeMap<String, Counts>,
    order: RowOrder,
) {
    let names: Vec<&str> = members.iter().map(|member| member.name.as_str()).collect();
    let mut headers = vec![label.to_owned()];
    for name in &names {
        headers.extend([
            format!("{} Lines", cell(name)),
            format!("{} Code", cell(name)),
        ]);
    }
    headers.extend(["Total Lines".into(), "Total Code".into()]);
    writeln!(
        output,
        "\n## {title}\n\n{}\n\n| {} |\n| --- |{}",
        Theme::MARKDOWN.paint(Role::Muted, "Each total is the sum of its member columns."),
        headers.join(" | "),
        " ---: |".repeat(headers.len() - 1)
    )
    .unwrap();
    let rows = of(&summary.totals)
        .iter()
        .map(|(key, counts)| (key.clone(), *counts))
        .collect();
    for (key, total) in ordered(rows, |counts| counts.code, order) {
        let mut cells = vec![cell(&key)];
        for name in &names {
            let counts = summary
                .folders
                .get(*name)
                .and_then(|folder| of(&folder.recursive).get(&key))
                .copied()
                .unwrap_or_default();
            cells.extend([grouped(counts.lines), grouped(counts.code)]);
        }
        cells.extend([grouped(total.lines), grouped(total.code)]);
        writeln!(output, "| {} |", cells.join(" | ")).unwrap();
    }
}

pub(super) fn table(
    output: &mut String,
    title: &str,
    label: &'static str,
    counts: &BTreeMap<String, Counts>,
    order: RowOrder,
) {
    let header = Columns::of(&count_columns(label)).markdown_header();
    writeln!(output, "\n## {title}\n\n{header}").unwrap();
    let rows = counts
        .iter()
        .map(|(label, counts)| (label.clone(), counts))
        .collect();
    for (label, counts) in ordered(rows, |counts| counts.code, order) {
        count_row(output, &label, counts);
    }
}

pub fn render_markdown(
    project: &str,
    summary: &ReportSummary,
    rows: &[FileRecord],
    findings: &Findings,
    depth: usize,
    order: RowOrder,
    variants: LanguageVariants,
) -> String {
    let folds = variants.source_folds(rows, &Purpose::ALL, &[], &[]);
    let folded = variants.folded(rows);
    let (summary, rows) = match &folded {
        Some((rows, summary)) => (summary, &rows[..]),
        None => (summary, rows),
    };
    let project = display_project(project, findings.subfolder);
    let mut output = format!("# {}: lines of code by purpose\n\n{}\n\n## Source purpose\n\n{}\n", cell(&project), Theme::MARKDOWN.paint(Role::Muted, "Production means maintained non-test source, including separately categorized tooling. Data, configuration, and documentation are counted apart from source. Lines = Code + Comments + Blanks in every table with these columns, and the purpose rows add up to all source."), Columns::of(&count_columns("Purpose")).markdown_header());
    for purpose in Purpose::ALL {
        count_row(
            &mut output,
            &purpose.to_string(),
            &summary.totals.purpose(purpose),
        );
    }
    if let Some(note) = inline_tests_note(&summary.totals, "`#[cfg(test)]`") {
        writeln!(output, "\n{}", Theme::MARKDOWN.paint(Role::Muted, &note)).unwrap();
    }
    if !findings.members.is_empty() {
        let (body, _, total) = member_rows(summary, findings.members, order, &ALL_PURPOSES);
        let escaped = |row: &Vec<String>| row.iter().map(|value| cell(value)).collect::<Vec<_>>();
        let body: Vec<_> = body.iter().map(escaped).collect();
        write!(
            output,
            "\n## Members\n\n{}",
            shared_table(
                &member_columns(&ALL_PURPOSES),
                &body,
                Some(&escaped(&total)),
                true,
                Theme::MARKDOWN
            )
        )
        .unwrap();
        writeln!(
            output,
            "\n{}",
            Theme::MARKDOWN.paint(
                Role::Muted,
                &ALL_PURPOSES.legend(None, Some("Share = Code / workspace Code"))
            )
        )
        .unwrap();
        member_grid(
            &mut output,
            "Source categories by member",
            "Category",
            summary,
            findings.members,
            |totals| &totals.source_categories,
            order,
        );
        member_grid(
            &mut output,
            "Source languages by member",
            "Language",
            summary,
            findings.members,
            |totals| &totals.source_languages,
            order,
        );
    }
    table(
        &mut output,
        "Source categories",
        "Category",
        &summary.totals.source_categories,
        order,
    );
    let non_source = summary
        .totals
        .by_kind_purpose
        .iter()
        .filter(|(key, _)| !key.starts_with("source:"))
        .map(|(key, value)| (key.clone(), *value))
        .collect();
    table(
        &mut output,
        "Non-source content",
        "Kind:purpose",
        &non_source,
        order,
    );
    let languages = Columns(
        LANGUAGE_COLUMNS
            .into_iter()
            .chain([ALL_PURPOSES.ratio_column()])
            .collect(),
    );
    let notes: Vec<String> = std::iter::once(LANGUAGE_RATIO_NOTE.to_string())
        .chain(folded_note(&folds))
        .map(|note| Theme::MARKDOWN.paint(Role::Muted, &note))
        .collect();
    writeln!(
        output,
        "\n## Source languages\n\n{}\n\n{}",
        notes.join("\n\n"),
        languages.markdown_header()
    )
    .unwrap();
    for (language, (counts, code)) in ordered(
        source_languages(rows).into_iter().collect(),
        |(counts, _)| counts.code,
        order,
    ) {
        writeln!(
            output,
            "| {} | {} | {} |",
            cell(&language),
            count_cells(&counts).join(" | "),
            ALL_PURPOSES.ratio_cell(code)
        )
        .unwrap();
    }
    writeln!(output, "\n## Folder breakdown\n\n{}\n\n{}", Theme::MARKDOWN.paint(Role::Muted, &format!("Recursive source code totals to depth {depth}. Parent rows include their children; do not sum all displayed rows. The report file's additive `breakdown` rows cover every depth. Multi-language files count once physically but contribute to each contained language. {}", ALL_PURPOSES.legend(None, None))), Columns::with_purposes(&code_columns("Folder"), &ALL_PURPOSES).markdown_header()).unwrap();
    // Compare by path components: as plain strings, `web-admin` would sort
    // between `web` and `web/src`, because `-` sorts before `/`.
    let mut folders = human_folders(summary, findings.subfolder, depth);
    folders.sort_by(|a, b| a.0.split('/').cmp(b.0.split('/')));
    for (folder, totals) in folders {
        if totals.source.files == 0 {
            continue;
        }
        writeln!(
            output,
            "| {} | {} | {} | {} |",
            cell(&folder),
            grouped(totals.source.lines),
            grouped(totals.source.code),
            ALL_PURPOSES.cells(RowCode::of_totals(totals)).join(" | ")
        )
        .unwrap();
    }
    render_build_modules(&mut output, findings.modules, findings.module_files, order);
    let unknown = review(rows);
    writeln!(
        output,
        "\n## Source requiring review\n\n{} of unclaimed source: no rule, convention, or build module claims it. Code an entry or convention assigns to Other on purpose is not listed.",
        counted(unknown.len() as u64, "file")
    )
    .unwrap();
    let folders = unclaimed_folders(rows);
    if !folders.is_empty() {
        output.push_str("\n| Folder | Files | Lines | Code |\n| --- | ---: | ---: | ---: |\n");
        for share in &folders {
            writeln!(
                output,
                "| {} | {} | {} | {} |",
                cell(folder_label(&share.folder)),
                grouped(share.files),
                grouped(
                    unknown
                        .iter()
                        .filter(|entry| crate::report::share_holds(&share.folder, &entry.path))
                        .map(|entry| entry.lines)
                        .sum()
                ),
                grouped(share.code)
            )
            .unwrap();
        }
    }
    writeln!(output, "\nLargest 25 files; the report file's `review` list names all of them with their rule.\n\n| Path | Lines | Code | Category |\n| --- | ---: | ---: | --- |").unwrap();
    for row in unknown.into_iter().take(25) {
        writeln!(
            output,
            "| {} | {} | {} | {} |",
            cell(&row.path),
            grouped(row.lines),
            grouped(row.code),
            cell(&row.category)
        )
        .unwrap();
    }
    render_coverage(&mut output, findings);
    output
}

/// Build systems and every module, so a reader can see which build files
/// decided production and spot a module whose layout claimed nothing.
pub(super) fn render_build_modules(
    output: &mut String,
    modules: &[ModuleShare],
    files: &[ModuleFile],
    order: RowOrder,
) {
    if modules.is_empty() {
        return;
    }
    output.push_str("\n## Build modules\n\nFolders whose build file declares a source layout; that layout counts as production where no override, convention, or rule decided first. Code is the production source each module's layout covers, whichever stage decided it; 0 means its layout covers none, a hint at a misleading build file.\n\n| Build system | Modules | Lines | Code |\n| --- | ---: | ---: | ---: |\n");
    let systems = build_systems(modules)
        .into_iter()
        .map(|(id, system)| (id.to_string(), system))
        .collect();
    for (id, system) in ordered(systems, |system| system.code, order) {
        writeln!(
            output,
            "| {id} | {} | {} | {} |",
            grouped(system.modules),
            grouped(
                modules
                    .iter()
                    .filter(|module| module.detector == id)
                    .map(|module| module.lines)
                    .sum()
            ),
            grouped(system.code)
        )
        .unwrap();
    }
    output.push_str("\n| Module | Build system | Lines | Code |\n| --- | --- | ---: | ---: |\n");
    for module in modules {
        writeln!(
            output,
            "| {} | {} | {} | {} |",
            cell(module_label(&module.path)),
            module.detector,
            grouped(module.lines),
            grouped(module.code)
        )
        .unwrap();
    }
    if files.is_empty() {
        return;
    }
    output.push_str("\nLargest files counted as Production only by a build layout. If one is generated or vendored, exclude or reclassify it in `.locmetryrc`.\n\n| Path | Lines | Code | Module | Build system |\n| --- | ---: | ---: | --- | --- |\n");
    for file in files {
        writeln!(
            output,
            "| {} | {} | {} | {} | {} |",
            cell(&file.path),
            grouped(file.lines),
            grouped(file.code),
            cell(module_label(&file.module)),
            cell(&file.detector)
        )
        .unwrap();
    }
}

pub(super) fn render_coverage(output: &mut String, findings: &Findings) {
    let coverage = findings.coverage;
    writeln!(output, "\n## Coverage\n\nFiles selected: {}; counted: {}, including {} through language mappings; not recognized: {}; recognized but unreadable: {}. Their lines are missing, not zero. Counts describe the working copy, including untracked files, not a commit.", grouped(coverage.selected_files as u64), grouped(coverage.counted_files as u64), grouped(coverage.mapped_files as u64), grouped(coverage.unrecognized_files as u64), grouped(coverage.unread.len() as u64)).unwrap();
    let extensions = extensions_by_files(coverage);
    if !extensions.is_empty() {
        output.push_str("\n| Unrecognized extension | Files |\n| --- | ---: |\n");
        for (extension, files) in extensions.into_iter().take(15) {
            writeln!(
                output,
                "| {} | {} |",
                cell(extension),
                grouped(*files as u64)
            )
            .unwrap();
        }
    }
    if !coverage.unread.is_empty() {
        output.push_str("\n| Unreadable file |\n| --- |\n");
        for path in coverage.unread.iter().take(25) {
            writeln!(output, "| {} |", cell(path)).unwrap();
        }
    }
    if !coverage.parse_failures.is_empty() {
        output.push_str(
            "\n| File with unavailable counts | Reason | Detail |\n| --- | --- | --- |\n",
        );
        for failure in coverage.parse_failures.iter().take(25) {
            writeln!(
                output,
                "| {} | {} | {} |",
                cell(&failure.path),
                cell(&failure.reason.to_string()),
                cell(&failure.message)
            )
            .unwrap();
        }
    }
    for note in submodules_note(findings.submodules)
        .into_iter()
        .chain(unused_entries_note(
            findings.unused_entries,
            findings.subfolder,
        ))
        .chain(unused_exclusions_note(
            findings.unused_exclusions,
            findings.subfolder,
        ))
        .chain(purpose_changes_note(findings.policy_entries))
        .chain(members_note(findings.members, MemberNotes::All))
    {
        writeln!(output, "\n{note}").unwrap();
    }
}
