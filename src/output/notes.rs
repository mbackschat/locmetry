//! Notes and findings shared by the terminal and Markdown reports.
use super::*;

/// Introduces the meanings below a category table.
pub(super) const MEANINGS_INTRO: &str = "Category meanings declared in the policy:";

/// The declared meanings of the categories `keys` show, as `purpose:category`
/// keys in row order; each name once, since one meaning covers its purposes.
pub(super) fn shown_meanings<'m, 'k>(
    meanings: &'m BTreeMap<String, String>,
    keys: impl Iterator<Item = &'k str>,
) -> Vec<(&'m str, &'m str)> {
    let mut shown: Vec<(&str, &str)> = Vec::new();
    for key in keys {
        let name = key.split_once(':').map_or(key, |(_, name)| name);
        if let Some((name, meaning)) = meanings.get_key_value(name) {
            if !shown.iter().any(|(seen, _)| *seen == name) {
                shown.push((name, meaning));
            }
        }
    }
    shown
}

/// The terminal legend of `shown` meanings, names padded to one column.
pub(super) fn meanings_note(shown: &[(&str, &str)]) -> Option<String> {
    let width = shown.iter().map(|(name, _)| name.chars().count()).max()?;
    let mut note = String::from(MEANINGS_INTRO);
    for (name, meaning) in shown {
        write!(note, "\n   {name:<width$}  {meaning}").unwrap();
    }
    Some(note)
}

pub(super) enum MemberNotes {
    All,
    Unclaimed,
    Unavailable,
}

pub(super) fn members_note(members: &[MemberView], kind: MemberNotes) -> Option<String> {
    let parts: Vec<String> = members
        .iter()
        .filter_map(|member| {
            let mut findings = Vec::new();
            if !matches!(kind, MemberNotes::Unavailable) && member.unclaimed_files > 0 {
                findings.push(format!(
                    "{} ({})",
                    counted(member.unclaimed_files, "unclaimed file"),
                    counted(member.unclaimed_code, "code line")
                ));
            }
            if !matches!(kind, MemberNotes::Unclaimed) && member.unread_files > 0 {
                findings.push(counted(member.unread_files as u64, "unreadable file"));
            }
            if !matches!(kind, MemberNotes::Unclaimed) && member.parse_failed_files > 0 {
                findings.push(counted(
                    member.parse_failed_files as u64,
                    "file with a parsing failure",
                ));
            }
            (!findings.is_empty())
                .then(|| format!("{} has {}", quoted(&member.name), findings.join(" and ")))
        })
        .collect();
    let label = match kind {
        MemberNotes::All => "Members to review",
        MemberNotes::Unclaimed => "Unclaimed source by member",
        MemberNotes::Unavailable => "Unavailable counts",
    };
    (!parts.is_empty()).then(|| format!("{label}: {}.", parts.join(", ")))
}

/// Code each build system classified, in one line.
pub(super) fn build_modules_note(modules: &[ModuleShare], order: RowOrder) -> Option<String> {
    // A system whose layouts claimed nothing does not explain production;
    // JSON and the Markdown module list keep it.
    let systems: Vec<_> = build_systems(modules)
        .into_iter()
        .filter(|(_, system)| system.code > 0)
        .map(|(id, system)| (id.to_string(), system))
        .collect();
    if systems.is_empty() {
        return None;
    }
    let systems: Vec<String> = ordered(systems, |system| system.code, order)
        .into_iter()
        .map(|(id, system)| {
            format!(
                "{} in {} {} {}",
                counted(system.code, "code line"),
                grouped(system.modules),
                quoted(&id),
                plural(system.modules, "module")
            )
        })
        .collect();
    Some(format!(
        "Production from build modules: {}.",
        systems.join(", ")
    ))
}

/// Label of a folder share; `.` holds the files directly in the scan root.
pub(super) fn folder_label(folder: &str) -> &str {
    match folder {
        "." => "(root files)",
        _ => folder,
    }
}

/// Where the unclaimed source lives, as an aligned folder list, so a reader
/// can inspect it instead of only learning its size.
pub(super) fn unclaimed_note(folders: &[FolderShare], theme: Theme) -> String {
    let shown = &folders[..folders.len().min(FOLDER_LINES)];
    let rows: Vec<[String; 5]> = shown
        .iter()
        .map(|share| {
            [
                plain(folder_label(&share.folder)),
                grouped(share.code),
                plural(share.code, "code line"),
                grouped(share.files),
                plural(share.files, "file"),
            ]
        })
        .collect();
    let [label, code, unit, files, _] = column_widths(&rows);
    let mut note = format!(
        "{} {} in {}, here:",
        theme.paint(Role::Information, "Unclaimed source:"),
        counted(folders.iter().map(|share| share.code).sum(), "code line"),
        counted(folders.iter().map(|share| share.files).sum(), "file")
    );
    for [folder, lines, lines_unit, count, count_unit] in &rows {
        write!(
            note,
            "\n   {folder:<label$}  {lines:>code$} {lines_unit:<unit$}  {count:>files$} {count_unit}"
        )
        .unwrap();
    }
    if folders.len() > shown.len() {
        let rest = &folders[shown.len()..];
        write!(
            note,
            "\n   and {} more {} ({})",
            rest.len(),
            plural(rest.len() as u64, "folder"),
            counted(rest.iter().map(|share| share.code).sum(), "code line")
        )
        .unwrap();
    }
    note + &format!(
        "\n To inspect or refine purpose classification, use {}. {} shows each file's rule.",
        quoted("--guide configure"),
        quoted("--files --output -")
    )
}

/// The largest module-claimed files as an aligned list; like the unclaimed
/// list, it owns its line breaks and sanitizes paths before padding.
pub(super) fn module_files_note(files: &[ModuleFile]) -> String {
    let rows: Vec<[String; 4]> = files
        .iter()
        .map(|file| {
            [
                plain(&file.path),
                grouped(file.code),
                plural(file.code, "code line"),
                format!(
                    "{} ({})",
                    plain(module_label(&file.module)),
                    plain(&file.detector)
                ),
            ]
        })
        .collect();
    let [path, code, unit, _] = column_widths(&rows);
    let mut note = String::from(
        "Largest files counted as Production only by a build layout. If one is generated or vendored, exclude or reclassify it in '.locmetryrc':",
    );
    for [file, lines, lines_unit, module] in &rows {
        write!(
            note,
            "\n   {file:<path$}  {lines:>code$} {lines_unit:<unit$}  {module}"
        )
        .unwrap();
    }
    note
}

/// Folders listed in the terminal note; the JSON headline lists all of them.
pub(super) const FOLDER_LINES: usize = 8;

/// Unrecognized extensions, most files first, ties by name.
pub(super) fn extensions_by_files(coverage: &Coverage) -> Vec<(&String, &usize)> {
    let mut extensions: Vec<_> = coverage.unrecognized_extensions.iter().collect();
    extensions.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    extensions
}

/// Points at Git submodules, often other teams' code, so a reader decides
/// their scope instead of counting them unawares.
pub(super) fn submodules_note(submodules: &[SubmoduleShare]) -> Option<String> {
    if submodules.is_empty() {
        return None;
    }
    let shares: Vec<String> = submodules
        .iter()
        .map(|share| format!("{} ({})", share.path, counted(share.code, "code line")))
        .collect();
    Some(format!(
        "Git submodules: {}. If they are not this project's code, exclude them with '--exclude' or 'scope.exclude'.",
        shares.join(", ")
    ))
}

/// Qualifies a policy note measured in a subfolder only.
pub(super) fn subtree(subfolder: Option<&str>) -> &'static str {
    match subfolder {
        Some(_) => " in this subtree",
        None => "",
    }
}

/// Names configuration entries that decided no file, which usually means a
/// glob that matches nothing or an entry an earlier stage preempts.
pub(super) fn unused_entries_note(entries: &[String], subfolder: Option<&str>) -> Option<String> {
    (!entries.is_empty()).then(|| {
        format!(
            "Entries that decided no file{}: {}.",
            subtree(subfolder),
            entries.join(", ")
        )
    })
}

/// Names, per entry, the `match` or `except` globs that did nothing, which an
/// entry that still decides files would otherwise hide: a mistyped glob in a
/// partly working entry. Entries that decided no file have their own note.
pub(super) fn unused_globs_notes(
    entries: &[crate::PolicyEntry],
    unused_entries: &[String],
    subfolder: Option<&str>,
) -> Vec<String> {
    let partly_used = || entries.iter().filter(|e| !unused_entries.contains(&e.id));
    let listed = |globs: fn(&crate::PolicyEntry) -> &Vec<String>| -> Vec<String> {
        partly_used()
            .filter(|entry| !globs(entry).is_empty())
            .map(|entry| format!("{} ({})", entry.id, globs(entry).join(", ")))
            .collect()
    };
    [
        (
            "Patterns that decided no file",
            listed(|e| &e.unused_patterns),
        ),
        (
            "Exceptions that left out no file",
            listed(|e| &e.unused_exceptions),
        ),
    ]
    .into_iter()
    .filter(|(_, found)| !found.is_empty())
    .map(|(label, found)| format!("{label}{}: {}.", subtree(subfolder), found.join(", ")))
    .collect()
}

/// Names entries whose matching files an earlier entry of the policy decided.
/// Built-in conventions are left out: their precedence is the stage order, and
/// `except` on an entry cannot change it. The configure guide recommends
/// `except` over relying on entry order (docs/guides/configure.md).
pub(super) fn preempted_entries_note(entries: &[crate::PolicyEntry]) -> Option<String> {
    let found: Vec<String> = entries
        .iter()
        // A mapping has no `except`; its first match decides.
        .filter(|entry| entry.stage != crate::EntryStage::Mapping)
        .flat_map(|entry| {
            entry
                .preempted
                .iter()
                .filter(|by| entries.iter().any(|earlier| earlier.id == by.by))
                .map(move |by| format!("{} ({} to {})", entry.id, counted(by.files, "file"), by.by))
        })
        .collect();
    (!found.is_empty()).then(|| {
        format!(
            "Entries that lost matching files to an earlier entry: {}. An {} list on the earlier entry states such a split without relying on order.",
            found.join(", "),
            quoted("except")
        )
    })
}

/// Names language mappings that matched files the counter recognizes, which
/// they cannot change: a mapping of `*.spec` to Plain Text counted nothing,
/// because the counter reads such files as RPM Specfile, and nothing said so
/// (docs/research/agents/feedback-2026-10-25-b.md).
pub(super) fn recognized_mappings_note(entries: &[crate::PolicyEntry]) -> Option<String> {
    let found: Vec<String> = entries
        .iter()
        .filter(|entry| entry.stage == crate::EntryStage::Mapping)
        .flat_map(|entry| {
            entry
                .preempted
                .iter()
                .filter(|by| by.by == crate::RECOGNITION)
                .map(move |by| format!("{} ({})", entry.id, counted(by.files, "file")))
        })
        .collect();
    (!found.is_empty()).then(|| {
        format!(
            "Language mappings that matched files the counter already recognizes: {}. A mapping applies only to unrecognized files; a {} entry changes the language of a recognized file.",
            found.join(", "),
            quoted("[[languages.corrections]]")
        )
    })
}

/// Names language rows whose code is all embedded in files of other
/// languages, such as a script in HTML: a row of 0 files otherwise reads as an
/// error, and agents guessed wrong causes for one
/// (docs/research/agents/experiments-2026-10-25.md).
pub(super) fn embedded_only_legend<'a>(
    rows: impl IntoIterator<Item = (&'a str, &'a Counts)>,
) -> Option<String> {
    let names: Vec<&str> = rows
        .into_iter()
        .filter(|(_, counts)| counts.files == 0 && counts.lines > 0)
        .map(|(language, _)| language)
        .collect();
    (!names.is_empty()).then(|| {
        format!(
            "Rows with 0 files count code embedded in files of other languages, such as scripts in HTML: {}.",
            names.join(", ")
        )
    })
}

/// Names entries whose files a later stage would give another purpose, such
/// as an override turning module production into tests. Such moves are often
/// intended, as for test runners classified as tooling, so the note asks for
/// review instead of warning (docs/proposals/archive/policy-entry-review.md).
pub(super) fn purpose_changes_note(entries: &[crate::PolicyEntry]) -> Option<String> {
    let changes: Vec<String> = entries
        .iter()
        .flat_map(|entry| {
            let to = entry.purpose?;
            Some(
                entry
                    .purpose_changes()
                    .into_iter()
                    .map(move |(from, code)| {
                        format!(
                            "{} ({} to {}, {})",
                            entry.id,
                            purpose_title(from),
                            purpose_title(to),
                            counted(code, "code line")
                        )
                    }),
            )
        })
        .flatten()
        .collect();
    (!changes.is_empty()).then(|| {
        format!(
            "Entries that changed the purpose a later stage would give: {}.",
            changes.join(", ")
        )
    })
}

/// Scoped non-use cannot establish whether project exclusions need repair.
pub(super) fn unused_exclusions_note(
    patterns: &[String],
    subfolder: Option<&str>,
) -> Option<String> {
    (!patterns.is_empty()).then(|| {
        format!(
            "Exclusions that excluded nothing{}: {}. {}.",
            subtree(subfolder),
            patterns.join(", "),
            match subfolder {
                Some(_) => "Check policy use with an unrestricted measurement",
                None =>
                    "Exclude a folder as '<folder>/**'. An ignore file may already hide the path",
            }
        )
    })
}

/// Inline Rust test code inside the table's rows, one legend line per
/// language and purpose: it qualifies their Prod. and Test/prod figures, so
/// it appears with the rows, and filters that hide them drop it.
pub(super) fn inline_tests_legend(rows: &[FileRecord]) -> Vec<String> {
    let mut found: BTreeMap<(&str, Purpose), (u64, u64)> = BTreeMap::new();
    for row in rows.iter().filter(|row| row.inline_tests.code > 0) {
        let (files, code) = found
            .entry((&row.language, row.classification.purpose))
            .or_default();
        *files += 1;
        *code += row.inline_tests.code;
    }
    found
        .into_iter()
        .map(|((language, purpose), (files, code))| {
            format!(
                "{} in {} includes {} of inline #[cfg(test)] tests in {}. {} names them.",
                purpose_title(purpose),
                quoted(language),
                counted(code, "code line"),
                counted(files, "file"),
                quoted("--files --output -")
            )
        })
        .collect()
}

/// Code of inline Rust test items that each purpose's source counts, with
/// `attribute` spelled for the format; `None` without any, so a module that
/// only declares `mod tests;` adds no note.
pub(super) fn inline_tests_note(summary: &Summary, attribute: &str) -> Option<String> {
    let purposes: Vec<String> = summary
        .inline_tests
        .iter()
        .map(|(purpose, counts)| {
            format!(
                "{purpose} includes {} in Rust {attribute} items ({})",
                counted(counts.code, "code line"),
                counted(counts.files, "file")
            )
        })
        .collect();
    (!purposes.is_empty()).then(|| format!("Inline tests: {}.", purposes.join(", ")))
}
