//! Notes and findings shared by the terminal and Markdown reports.
use super::*;

pub(super) enum MemberNotes {
    All,
    Unclaimed,
    Unavailable,
}

pub(super) fn repositories_note(members: &[MemberView], kind: MemberNotes) -> Option<String> {
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
            (!findings.is_empty()).then(|| format!("{}: {}", member.name, findings.join(", ")))
        })
        .collect();
    let label = match kind {
        MemberNotes::All => "Repositories to review",
        MemberNotes::Unclaimed => "Unclaimed source by repository",
        MemberNotes::Unavailable => "Unavailable counts",
    };
    (!parts.is_empty()).then(|| format!("{label}: {}.", parts.join("; ")))
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
                "{id} {} in {}",
                counted(system.code, "code line"),
                counted(system.modules, "module")
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
    note + "\n To inspect or refine purpose classification, use locmetry --guide configure; --files --output - shows each file's rule."
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
                    "{} {}",
                    plain(&file.detector),
                    plain(module_label(&file.module))
                ),
            ]
        })
        .collect();
    let [path, code, unit, _] = column_widths(&rows);
    let mut note = String::from(
        "Largest module-claimed production; generated or vendored code here needs an entry:",
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

/// Label of a module folder; `.` is the scan root.
pub(super) fn module_label(path: &str) -> &str {
    match path {
        "." => "(scan root)",
        path => path,
    }
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
        "Git submodules: {}; exclude them with --exclude or scope.exclude if they are not this project's code.",
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

/// Scoped non-use cannot establish whether project exclusions need repair.
pub(super) fn unused_exclusions_note(
    patterns: &[String],
    subfolder: Option<&str>,
) -> Option<String> {
    (!patterns.is_empty()).then(|| {
        format!(
            "Exclusions that excluded nothing{}: {}; {}.",
            subtree(subfolder),
            patterns.join(", "),
            match subfolder {
                Some(_) => "check policy use with an unrestricted measurement",
                None =>
                    "exclude a folder as <folder>/**, and an ignore file may already hide the path",
            }
        )
    })
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
    (!purposes.is_empty()).then(|| format!("Inline tests: {}.", purposes.join("; ")))
}
