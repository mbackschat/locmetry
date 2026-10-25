//! The terminal report: tables with optional bars, measurement notes and findings.
use super::*;

/// Presentation choices for terminal measurement tables; they do not change saved evidence.
pub struct TableOptions {
    pub categories: bool,
    pub folders: bool,
    pub verbose: bool,
    pub order: RowOrder,
    pub theme: Theme,
    pub folder_rows: Option<usize>,
    pub bars: Bars,
    /// Purposes whose source the tables count, in `Purpose::ALL` order.
    pub purposes: Vec<Purpose>,
    /// Categories whose source the tables count; empty counts all.
    pub category_selection: Vec<CategoryPattern>,
    /// Source languages the tables count, by display name; empty counts all.
    pub languages: Vec<String>,
    /// Omits every line outside the tables: legends, bar keys and notes.
    pub tables_only: bool,
    /// How the rows show language variants; selected names follow them.
    pub variants: LanguageVariants,
}

/// Render terminal tables, optionally partitioning folders within a row budget.
pub fn render_table_with_options(
    summary: &ReportSummary,
    rows: &[FileRecord],
    findings: &Findings,
    options: TableOptions,
) -> String {
    let TableOptions {
        categories,
        folders: show_folders,
        verbose,
        order,
        theme,
        folder_rows,
        bars,
        purposes,
        category_selection,
        languages,
        tables_only,
        variants,
    } = options;
    // Variants fold first, so a selected variant name selects its family's row.
    let mut selected_languages: Vec<String> = Vec::new();
    for language in languages.iter().map(|language| variants.row_name(language)) {
        if !selected_languages.contains(&language) {
            selected_languages.push(language);
        }
    }
    let folds = variants.source_folds(rows, &purposes, &category_selection, &selected_languages);
    let folded = variants.folded(rows);
    let (summary, rows) = match &folded {
        Some((rows, summary)) => (summary, &rows[..]),
        None => (summary, rows),
    };
    // A selection counts only that source, so every table, total and bar is
    // rebuilt from the selected records.
    let selection = TableSelection::of(rows, &purposes, &category_selection, &selected_languages);
    let selected_summary;
    let (summary, rows) = match &selection.rows {
        Some(selected) => {
            selected_summary = crate::build_report(selected);
            (&selected_summary, &selected[..])
        }
        None => (summary, rows),
    };
    // A category selection within one purpose shows that purpose alone, as
    // a purpose selection does; the purpose note still names only those the
    // purpose option left out.
    let shown: Vec<Purpose> = purposes
        .iter()
        .copied()
        .filter(|purpose| {
            category_selection.is_empty()
                || category_selection
                    .iter()
                    .any(|pattern| pattern.names(*purpose))
        })
        .collect();
    let columns = PurposeColumns {
        shown: &shown,
        sliced: selection.sliced,
    };
    let languages = ordered(
        source_languages(rows).into_iter().collect(),
        |(counts, _)| counts.code,
        order,
    );
    let codes: Vec<RowCode> = languages.iter().map(|(_, (_, code))| *code).collect();
    let body: Vec<Vec<String>> = languages
        .iter()
        .zip(&codes)
        .map(|((language, (counts, _)), code)| {
            let mut row = vec![language.clone()];
            row.extend(count_cells(counts));
            row.extend(columns.cells(*code));
            row
        })
        .collect();
    let total_code = RowCode::of_totals(&summary.totals);
    let mut total = vec!["Total".to_string()];
    total.extend(count_cells(&summary.totals.source));
    total.extend(columns.cells(total_code));
    let mut output = String::new();
    let legend = |text: &str| match tables_only {
        true => String::new(),
        false => format!(" {}\n", theme.paint(Role::Muted, text)),
    };
    // With bars, a table returns what follows its legends: an inline bar key
    // or the chart below it.
    let table = |table_columns: &Columns,
                 body: &[Vec<String>],
                 total: Option<Vec<String>>,
                 rows: &[RowCode]| {
        let rows = BarRows {
            rows,
            total: total_code,
            columns: &columns,
            bars,
            key: !tables_only,
        };
        bar_table(table_columns, body, total, &rows, theme)
    };
    let (language_table, after) = table(
        &Columns::with_purposes(&LANGUAGE_COLUMNS, &columns),
        &body,
        Some(total),
        &codes,
    );
    output += &language_table.top_rule(theme);
    output += &language_table.text;
    let mut legends = vec![columns.legend(Some("Lines = Code + Comments + Blanks"), None)];
    if columns.ratio() == Ratio::TestPerProduction {
        legends.push(LANGUAGE_RATIO_NOTE.into());
    }
    legends.extend(inline_tests_legend(rows));
    legends.extend(folded_note(&folds));
    legends.extend(embedded_only_legend(
        languages
            .iter()
            .map(|(language, (counts, _))| (language.as_str(), counts)),
    ));
    legends.extend(selection.notes(&purposes));
    for text in legends {
        output += &legend(&text);
    }
    output += &after;

    // A workspace's top-level folders are its members, so member rows,
    // which add the test ratio and share, take the folder rows' place.
    if !findings.members.is_empty() {
        let (body, codes, total) = member_rows(summary, findings.members, order, &columns);
        let (text, after) = table(&member_columns(&columns), &body, Some(total), &codes);
        output += "\n";
        output += &text.top_rule(theme);
        output += &text.text;
        output += &legend(&columns.legend(None, Some("Share = Code / workspace Code")));
        output += &after;
    }
    if findings.members.is_empty() && (show_folders || folder_rows.is_some()) {
        let adaptive =
            folder_rows.map(|budget| folders::adaptive(summary, findings.subfolder, budget, order));
        let standard = folders::standard(summary, findings.subfolder, order);
        let folders = adaptive.as_ref().map_or(&standard, |view| &view.rows);
        if !folders.is_empty() {
            let codes: Vec<RowCode> = folders
                .iter()
                .map(|folder| RowCode::of(&folder.purposes, folder.counts.comments))
                .collect();
            let body: Vec<Vec<String>> = folders
                .iter()
                .zip(&codes)
                .map(|(folder, code)| {
                    let mut row = vec![
                        folder.label(),
                        grouped(folder.counts.lines),
                        grouped(folder.counts.code),
                    ];
                    row.extend(columns.cells(*code));
                    row
                })
                .collect();
            let mut total = vec![
                "Total".to_string(),
                grouped(summary.totals.source.lines),
                grouped(summary.totals.source.code),
            ];
            total.extend(columns.cells(total_code));
            let (text, after) = table(
                &Columns::with_purposes(&code_columns("Folder"), &columns),
                &body,
                Some(total),
                &codes,
            );
            output += "\n";
            output += &text.top_rule(theme);
            output += &text.text;
            output += &legend(&columns.legend(None, None));
            if let Some(view) = &adaptive {
                output += &legend("Adaptive folder detail: rows cover measured source once.");
                if let Some(hint) = &view.budget_hint {
                    output += &legend(&plain(hint));
                }
            }
            output += &after;
        }
    }
    if categories {
        let categories = ordered(
            source_categories(rows).into_iter().collect(),
            |(counts, _)| counts.code,
            order,
        );
        let codes: Vec<RowCode> = categories.iter().map(|(_, (_, code))| *code).collect();
        let meanings = match verbose {
            true => meanings_note(&shown_meanings(
                findings.category_meanings,
                categories.iter().map(|(category, _)| category.as_str()),
            )),
            false => None,
        };
        let body: Vec<Vec<String>> = categories
            .into_iter()
            .map(|(category, (counts, _))| {
                vec![
                    category,
                    grouped(counts.files),
                    grouped(counts.lines),
                    grouped(counts.code),
                ]
            })
            .collect();
        if !body.is_empty() {
            // The language table above already totals the rows.
            let (text, after) = table(
                &Columns::of(&count_columns("Category")[..4]),
                &body,
                None,
                &codes,
            );
            output += "\n";
            output += &text.top_rule(theme);
            output += &text.text;
            output += &legend(CATEGORY_LEGEND);
            if let Some(meanings) = &meanings {
                output += &legend(meanings);
            }
            output += &after;
        }
    }

    if tables_only {
        return output;
    }
    let mut notes = Vec::new();
    if verbose {
        // Per kind: its counts and the code of languages embedded in its
        // files, such as fenced snippets, which never reach the source tables.
        let mut other_kinds = BTreeMap::<Kind, (Counts, u64)>::new();
        for row in rows.iter().filter(|row| row.kind != Kind::Source) {
            let (counts, embedded) = other_kinds.entry(row.kind).or_default();
            counts.add(&row.counts);
            *embedded += row
                .languages
                .iter()
                .filter(|(language, _)| **language != row.language)
                .map(|(_, counts)| counts.code)
                .sum::<u64>();
        }
        if !other_kinds.is_empty() {
            let kinds: Vec<String> = other_kinds
                .iter()
                .map(|(kind, (counts, embedded))| {
                    let snippets = match embedded {
                        0 => String::new(),
                        code => format!(
                            ", including {} in embedded snippets",
                            counted(*code, "code line")
                        ),
                    };
                    format!(
                        "{} ({}{snippets})",
                        counted(counts.files, &format!("{kind} file")),
                        counted(counts.lines, "line")
                    )
                })
                .collect();
            notes.push(TerminalNote::Text(format!(
                "Not source code: {}. {} lists each file's kind.",
                kinds.join(", "),
                quoted("--files --output -")
            )));
        }
        notes.extend(build_modules_note(findings.modules, order).map(TerminalNote::Text));
        if !findings.module_files.is_empty() {
            notes.push(TerminalNote::ModuleFiles(findings.module_files.to_vec()));
        }
        notes.extend(submodules_note(findings.submodules).map(TerminalNote::Text));
        let folders = unclaimed_folders(rows);
        if !folders.is_empty() {
            notes.push(TerminalNote::Unclaimed(folders));
        }
        notes.extend(
            unused_entries_note(findings.unused_entries, findings.subfolder)
                .map(TerminalNote::Text),
        );
        notes.extend(
            unused_globs_notes(
                findings.policy_entries,
                findings.unused_entries,
                findings.subfolder,
            )
            .into_iter()
            .map(TerminalNote::Text),
        );
        notes.extend(
            unused_exclusions_note(findings.unused_exclusions, findings.subfolder)
                .map(TerminalNote::Text),
        );
        notes.extend(preempted_entries_note(findings.policy_entries).map(TerminalNote::Text));
        notes.extend(purpose_changes_note(findings.policy_entries).map(TerminalNote::Text));
        notes
            .extend(members_note(findings.members, MemberNotes::Unclaimed).map(TerminalNote::Text));
    }
    notes.extend(members_note(findings.members, MemberNotes::Unavailable).map(TerminalNote::Text));
    let coverage = findings.coverage;
    let mut gaps = Vec::new();
    if verbose && coverage.unrecognized_files > 0 {
        let extensions: Vec<String> = extensions_by_files(coverage)
            .into_iter()
            .take(5)
            .map(|(extension, files)| match extension.as_str() {
                "(none)" => format!("no extension: {}", grouped(*files as u64)),
                _ => format!("{extension}: {}", grouped(*files as u64)),
            })
            .collect();
        notes.push(TerminalNote::Unrecognized(format!(
            "{} ({}). Their lines are unknown, not zero.",
            grouped(coverage.unrecognized_files as u64),
            extensions.join(", ")
        )));
    }
    if !coverage.unread.is_empty() {
        gaps.push(counted(coverage.unread.len() as u64, "unreadable file"));
    }
    if !coverage.parse_failures.is_empty() {
        gaps.push(counted(
            coverage.parse_failures.len() as u64,
            "file with a parsing failure",
        ));
        if verbose {
            for failure in coverage.parse_failures.iter().take(25) {
                notes.push(TerminalNote::Text(failure.to_string()));
            }
        }
    }
    if !gaps.is_empty() {
        notes.push(TerminalNote::CoverageGap(format!(
            "{}. Their lines are unknown, not zero.",
            gaps.join(", ")
        )));
    }
    if !notes.is_empty() {
        output += "\n";
        for note in notes {
            writeln!(output, " {}", note.render(theme)).unwrap();
        }
    }
    output
}

pub(super) enum TerminalNote {
    Text(String),
    Unclaimed(Vec<FolderShare>),
    ModuleFiles(Vec<ModuleFile>),
    CoverageGap(String),
    Unrecognized(String),
}

impl TerminalNote {
    pub(super) fn render(self, theme: Theme) -> String {
        match self {
            Self::Text(text) => plain(&text),
            // This list owns its line breaks and sanitizes labels before padding.
            Self::Unclaimed(folders) => unclaimed_note(&folders, theme),
            Self::ModuleFiles(files) => module_files_note(&files),
            Self::Unrecognized(text) => theme.paint(
                Role::Muted,
                &format!("Unrecognized files: {}", plain(&text)),
            ),
            Self::CoverageGap(text) => format!(
                "{} {}",
                theme.paint(Role::Warning, "Not counted:"),
                plain(&text)
            ),
        }
    }
}
