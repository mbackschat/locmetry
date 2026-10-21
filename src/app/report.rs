//! Building a report from measured results and writing each requested view.
use super::*;

/// What a report is built from: one measurement, or a workspace merge with
/// paths prefixed by member name.
pub(crate) struct Results {
    pub(crate) project: String,
    pub(crate) rows: Vec<FileRecord>,
    pub(crate) coverage: Coverage,
    pub(crate) inaccurate: bool,
    pub(crate) modules: Vec<Module>,
    pub(crate) module_shares: Vec<ModuleShare>,
    pub(crate) submodules: Vec<SubmoduleShare>,
    /// Configuration findings as human notes name them.
    pub(crate) unused_entries: Vec<String>,
    pub(crate) unused_exclusions: Vec<String>,
    /// Workspace members in name order; empty for a single root.
    pub(crate) members: Vec<MemberView>,
}

pub(crate) enum Location {
    Root {
        root: PathBuf,
        scope: contract::Selection,
        config_path: Option<PathBuf>,
        user_path: Option<PathBuf>,
        config: Box<locmetry::EffectiveConfig>,
    },
    Workspace {
        members: Vec<contract::Member>,
        workspace_path: Option<PathBuf>,
        user_path: Option<PathBuf>,
        /// Folder members of one project, whose shared policy's findings
        /// belong in the summary.
        shared_policy: bool,
    },
}

/// Bars: forced by `--show-bars`, refused by `--no-bars`, and otherwise purpose
/// bars only on a terminal and only where they fit, so files and pipes stay stable.
pub(crate) fn bar_mode(
    bars: Option<BarKind>,
    no_bars: bool,
    terminal: bool,
    columns: usize,
) -> Bars {
    match (bars, no_bars, terminal) {
        (Some(kind), _, _) => Bars::Always(columns, kind),
        (None, true, _) | (None, false, false) => Bars::Off,
        (None, false, true) => Bars::Fit(columns),
    }
}

/// The purposes tables count, in `Purpose::ALL` order; all without a selection.
pub(crate) fn selected_purposes(selection: &[PurposeArg]) -> Vec<Purpose> {
    Purpose::ALL
        .into_iter()
        .filter(|purpose| {
            selection.is_empty() || selection.iter().any(|arg| arg.purpose() == *purpose)
        })
        .collect()
}

/// The footer line naming what a human report measured: the root and any
/// subfolder, or every workspace root. Markdown sets the paths as code.
pub(crate) fn measured_roots(location: &Location, markdown: bool) -> String {
    let path = |path: &str| match markdown {
        true => format!("`{}`", locmetry::style::plain(path)),
        false => locmetry::style::plain(path),
    };
    match location {
        Location::Root { root, scope, .. } => {
            let root = path(&root.to_string_lossy());
            match &scope.subfolder {
                Some(subfolder) => format!("Root: {root}, subfolder {}", path(subfolder)),
                None => format!("Root: {root}"),
            }
        }
        Location::Workspace {
            members,
            shared_policy: true,
            ..
        } => {
            let folders: Vec<_> = members
                .iter()
                .filter_map(|member| member.scope.subfolder.as_deref())
                .map(path)
                .collect();
            format!(
                "Project: {}, folders {}",
                path(&members[0].root),
                folders.join(", ")
            )
        }
        Location::Workspace { members, .. } => {
            let roots: Vec<_> = members.iter().map(|member| path(&member.root)).collect();
            format!("Roots: {}", roots.join(", "))
        }
    }
}

/// Writes the report, headline, or human output for `results`.
///
/// `location` holds the mode's own keys: a single root's root, scope, and
/// configuration, or a workspace's members. Configuration findings enter the
/// summary for a single root or a shared project policy; an independent
/// member's entry IDs and exclusion patterns are relative to that member.
pub(crate) fn report(
    shared: &SharedArgs,
    destination: &Destination,
    results: Results,
    location: Location,
    measured_at: u64,
    styling: Styling,
) -> Result<Outcome> {
    // Independent workspace members record their own unused entries; a single
    // root or a shared project policy reports them in the summary.
    let summary_findings = matches!(
        location,
        Location::Root { .. }
            | Location::Workspace {
                shared_policy: true,
                ..
            }
    );
    let Results {
        project,
        rows,
        coverage,
        inaccurate,
        modules,
        module_shares,
        submodules,
        unused_entries,
        unused_exclusions,
        members,
    } = results;
    let format = destination.format;
    let summary = build_report(&rows);
    let module_files = largest_module_files(&modules, &rows);
    let subfolder = match &location {
        Location::Root { scope, .. } => scope.subfolder.as_deref(),
        Location::Workspace { .. } => None,
    };
    let findings = Findings {
        subfolder,
        coverage: &coverage,
        modules: &module_shares,
        module_files: &module_files,
        submodules: &submodules,
        unused_entries: &unused_entries,
        unused_exclusions: &unused_exclusions,
        members: &members,
    };
    let order = match shared.sort {
        Sort::Code => RowOrder::Code,
        Sort::Name => RowOrder::Name,
    };
    let purposes = selected_purposes(&shared.purpose);
    let variants = shared.view.variants();
    // Table and Markdown views; `stdout` decides styling and automatic bars.
    let human = |format: Format, stdout: bool| {
        let terminal = stdout && io::stdout().is_terminal();
        let mut human = match format {
            Format::Table => render_table_with_options(
                &summary,
                &rows,
                &findings,
                TableOptions {
                    categories: shared.categories,
                    folders: destination.folders,
                    verbose: shared.verbose,
                    order,
                    theme: styling.theme(terminal),
                    folder_rows: destination.folder_rows,
                    bars: bar_mode(
                        shared.bars.map(BarArg::kind),
                        shared.no_bars,
                        terminal,
                        bar_columns(stdout),
                    ),
                    purposes: purposes.clone(),
                    languages: shared.languages.iter().map(|l| l.to_string()).collect(),
                    tables_only: shared.view.tables_only,
                    variants,
                },
            ),
            Format::Markdown => render_markdown(
                &project,
                &summary,
                &rows,
                &findings,
                shared.depth,
                order,
                variants,
            ),
            Format::Json | Format::Headline => return String::new(),
        };
        // Partial counts still warn on stderr and exit 2.
        if format == Format::Table && shared.view.tables_only {
            return human;
        }
        if inaccurate {
            let theme = match format {
                Format::Markdown => Theme::MARKDOWN,
                _ => styling.theme(terminal),
            };
            human.push_str(&format!(
                "\n{}; inspect the JSON metadata.\n",
                theme.paint(Role::Warning, "Backend reported incomplete counts")
            ));
        }
        // `Measured at:` stays the documented final line; the roots precede it.
        let separator = match format {
            Format::Markdown => "\n\n",
            _ => "\n",
        };
        let context = measured_roots(&location, format == Format::Markdown);
        human.push_str(&format!(
            "\n{context}{separator}Measured at: {}\n",
            locmetry::measurement_time(measured_at)
        ));
        human
    };
    let saved_human = human(format, destination.path.is_none());
    let printed = destination.print.map(|print| (print, human(print, true)));
    // The document is built only for JSON views; a headline omits the review,
    // breakdown and file records, so only full JSON derives them.
    let document = |full: bool| {
        let mut member_totals = BTreeMap::new();
        for member in &members {
            member_totals.insert(
                member.name.clone(),
                summary
                    .folders
                    .get(&member.name)
                    .map(|f| f.recursive.clone())
                    .unwrap_or_default(),
            );
        }
        let summary_document = contract::ReportSummary {
            top_folders: summary.top_folders(),
            totals: summary.totals,
            members: member_totals,
            unclaimed_folders: Some(unclaimed_folders(&rows)),
            build_modules: build_systems(&module_shares)
                .into_iter()
                .map(|(k, v)| (k.into(), v))
                .collect(),
            submodules,
            largest_module_files: Some(module_files.clone()),
            unused_entries: if summary_findings {
                unused_entries
            } else {
                Vec::new()
            },
            unused_exclusions: if summary_findings {
                unused_exclusions
            } else {
                Vec::new()
            },
        };
        let mut metadata = contract::Metadata {
            measured_at_unix_seconds: measured_at,
            locmetry_version: env!("CARGO_PKG_VERSION").into(),
            backend: BACKEND.into(),
            backend_inaccurate: inaccurate,
            backend_configuration: locmetry::BACKEND_CONFIGURATION.into(),
            kinds: Some(contract::Kinds {
                default: locmetry::Kind::Source,
                data: DATA_LANGUAGES.iter().map(|l| l.name().into()).collect(),
                documentation: DOCUMENTATION_LANGUAGES
                    .iter()
                    .map(|l| l.name().into())
                    .collect(),
            }),
            coverage: contract::Coverage::from(&coverage),
            modules: Some(
                modules
                    .iter()
                    .map(|m| contract::Module {
                        path: m.path.clone(),
                        detector: m.detector.into(),
                    })
                    .collect(),
            ),
            root: None,
            scope: None,
            config_path: None,
            config: None,
            members: None,
            workspace_path: None,
            user_config_path: None,
        };
        match location {
            Location::Root {
                root,
                scope,
                config_path,
                user_path,
                config,
            } => {
                metadata.root = Some(root.to_string_lossy().into());
                metadata.scope = Some(scope);
                metadata.config_path = Some(config_path.map(|p| p.to_string_lossy().into()));
                metadata.config = Some(*config);
                metadata.user_config_path = Some(user_path.map(|p| p.to_string_lossy().into()));
            }
            Location::Workspace {
                members,
                workspace_path,
                user_path,
                ..
            } => {
                metadata.members = Some(members);
                metadata.workspace_path = workspace_path.map(|p| p.to_string_lossy().into());
                metadata.user_config_path = Some(user_path.map(|p| p.to_string_lossy().into()));
            }
        }
        MeasurementReport {
            document_type: contract::DocumentType::Measurement,
            detail: if full {
                contract::Detail::Full
            } else {
                contract::Detail::Headline
            },
            schema_version: contract::Version::V3,
            project,
            metadata,
            summary: summary_document,
            review: full.then(|| review(&rows)),
            breakdown: full.then(|| breakdown(&rows)),
            files: (full && shared.files).then(|| rows.iter().map(contract::File::from).collect()),
        }
    };
    let shows = |format| destination.shows(format);
    let document =
        (shows(Format::Json) || shows(Format::Headline)).then(|| document(shows(Format::Json)));
    let text = |format: Format, human: String| -> Result<String> {
        let document = || document.as_ref().expect("built for JSON views");
        Ok(match format {
            Format::Json => report_text(&serde_json::to_value(document())?)?,
            Format::Headline => format!("{}\n", serde_json::to_string(&document().headline())?),
            Format::Table | Format::Markdown => human,
        })
    };
    destination.write(&text(format, saved_human)?)?;
    if let Some((print, human)) = printed {
        io::stdout()
            .lock()
            .write_all(text(print, human)?.as_bytes())?;
    }
    if inaccurate {
        let mut failures: Vec<_> = coverage
            .unread
            .iter()
            .map(|path| format!("{path}: unreadable"))
            .chain(
                coverage
                    .parse_failures
                    .iter()
                    .map(|failure| failure.to_string()),
            )
            .take(10)
            .collect();
        let count = coverage.unread.len() + coverage.parse_failures.len();
        if count > failures.len() {
            failures.push(format!("and {} more", count - failures.len()));
        }
        if failures.is_empty() {
            failures.push("backend did not identify affected paths".into());
        }
        return Ok(Outcome::Partial(format!(
            "Backend reported incomplete counts; output contains partial results. Unavailable files: {}",
            failures.join(", ")
        )));
    }
    Ok(Outcome::Complete)
}
