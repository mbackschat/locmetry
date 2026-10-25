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
    /// What each project entry decided; in a workspace of independent roots,
    /// IDs name their member as human notes do.
    pub(crate) policy_entries: Vec<locmetry::PolicyEntry>,
    /// Declared category meanings the human reports show beside the rows.
    pub(crate) category_meanings: BTreeMap<String, String>,
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

impl Location {
    /// Whether configuration findings belong in the summary: for a single
    /// root or a shared project policy, while independent workspace members
    /// record their own.
    fn summary_findings(&self) -> bool {
        matches!(
            self,
            Location::Root { .. }
                | Location::Workspace {
                    shared_policy: true,
                    ..
                }
        )
    }
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
                "Project: {} (folders: {})",
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
    let summary = build_report(&results.rows);
    if destination.shows(Format::Table) {
        for entry in &shared.table_categories {
            if !entry.matches_any(&results.rows) {
                styling.stderr_message(
                    Severity::Warning,
                    format_args!(
                        "'--table-category' entry '{entry}' matches no category in this measurement. '--show-categories' lists the measured ones."
                    ),
                );
            }
        }
    }
    let module_files = largest_module_files(&results.modules, &results.rows);
    let human = HumanView {
        shared,
        destination,
        styling,
        results: &results,
        summary: &summary,
        module_files: &module_files,
        location: &location,
        measured_at,
    };
    let saved_human = human.render(destination.format, destination.path.is_none());
    let printed = destination
        .print
        .map(|print| (print, human.render(print, true)));
    let partial = results
        .inaccurate
        .then(|| partial_counts(&results.coverage));
    // The document is built only for JSON views; a headline omits the review,
    // breakdown and file records, so only full JSON derives them.
    let shows = |format| destination.shows(format);
    let document = (shows(Format::Json) || shows(Format::Headline)).then(|| {
        let detail = match shows(Format::Json) {
            true => contract::Detail::Full,
            false => contract::Detail::Headline,
        };
        measurement_document(
            results,
            summary,
            module_files,
            location,
            measured_at,
            detail,
            shared.files,
        )
    });
    let text = |format: Format, human: String| -> Result<String> {
        let document = || document.as_ref().expect("built for JSON views");
        Ok(match format {
            Format::Json => report_text(&serde_json::to_value(document())?)?,
            Format::Headline => format!("{}\n", serde_json::to_string(&document().headline())?),
            Format::Table | Format::Markdown => human,
        })
    };
    destination.write(&text(destination.format, saved_human)?)?;
    if let Some((print, human)) = printed {
        io::stdout()
            .lock()
            .write_all(text(print, human)?.as_bytes())?;
    }
    Ok(partial.map_or(Outcome::Complete, Outcome::Partial))
}

/// Everything the table and Markdown views of one measurement read.
struct HumanView<'a> {
    shared: &'a SharedArgs,
    destination: &'a Destination<'a>,
    styling: Styling,
    results: &'a Results,
    summary: &'a locmetry::ReportSummary,
    module_files: &'a [locmetry::ModuleFile],
    location: &'a Location,
    measured_at: u64,
}

impl HumanView<'_> {
    /// The view in `format`, empty for JSON; `stdout` decides styling and
    /// automatic bars.
    fn render(&self, format: Format, stdout: bool) -> String {
        let terminal = stdout && io::stdout().is_terminal();
        let mut human = match format {
            Format::Table => self.table(terminal, stdout),
            Format::Markdown => render_markdown(
                &self.results.project,
                self.summary,
                &self.results.rows,
                &self.findings(),
                self.shared.depth,
                self.order(),
                self.shared.view.variants(),
            ),
            Format::Json | Format::Headline => return String::new(),
        };
        // Partial counts still warn on stderr and exit 2.
        if format == Format::Table && self.shared.view.tables_only {
            return human;
        }
        if self.results.inaccurate {
            let theme = match format {
                Format::Markdown => Theme::MARKDOWN,
                _ => self.styling.theme(terminal),
            };
            human.push_str(&format!(
                "\n{}. Inspect the JSON metadata.\n",
                theme.paint(Role::Warning, "Backend reported incomplete counts")
            ));
        }
        // `Measured at:` stays the documented final line; the roots precede it.
        let separator = match format {
            Format::Markdown => "\n\n",
            _ => "\n",
        };
        let context = measured_roots(self.location, format == Format::Markdown);
        human.push_str(&format!(
            "\n{context}{separator}Measured at: {}\n",
            locmetry::measurement_time(self.measured_at)
        ));
        human
    }

    fn table(&self, terminal: bool, stdout: bool) -> String {
        let shared = self.shared;
        render_table_with_options(
            self.summary,
            &self.results.rows,
            &self.findings(),
            TableOptions {
                categories: shared.categories,
                folders: self.destination.folders,
                verbose: shared.verbose,
                order: self.order(),
                theme: self.styling.theme(terminal),
                folder_rows: self.destination.folder_rows,
                bars: bar_mode(
                    shared.bars.map(BarArg::kind),
                    shared.no_bars,
                    terminal,
                    bar_columns(stdout),
                ),
                purposes: selected_purposes(&shared.purpose),
                category_selection: shared.table_categories.clone(),
                languages: shared.languages.iter().map(|l| l.to_string()).collect(),
                tables_only: shared.view.tables_only,
                variants: shared.view.variants(),
            },
        )
    }

    fn findings(&self) -> Findings<'_> {
        let results = self.results;
        Findings {
            subfolder: match self.location {
                Location::Root { scope, .. } => scope.subfolder.as_deref(),
                Location::Workspace { .. } => None,
            },
            coverage: &results.coverage,
            modules: &results.module_shares,
            module_files: self.module_files,
            submodules: &results.submodules,
            unused_entries: &results.unused_entries,
            unused_exclusions: &results.unused_exclusions,
            policy_entries: &results.policy_entries,
            category_meanings: &results.category_meanings,
            members: &results.members,
        }
    }

    fn order(&self) -> RowOrder {
        match self.shared.sort {
            Sort::Code => RowOrder::Code,
            Sort::Name => RowOrder::Name,
        }
    }
}

/// The JSON report at `detail`; per-file records need `files` and full detail.
fn measurement_document(
    results: Results,
    summary: locmetry::ReportSummary,
    module_files: Vec<locmetry::ModuleFile>,
    location: Location,
    measured_at: u64,
    detail: contract::Detail,
    files: bool,
) -> MeasurementReport {
    let full = matches!(detail, contract::Detail::Full);
    let rows = &results.rows;
    let review = full.then(|| review(rows));
    let breakdown = full.then(|| breakdown(rows));
    let files = (full && files).then(|| rows.iter().map(contract::File::from).collect());
    let summary = summary_document(&results, summary, module_files, location.summary_findings());
    let metadata = metadata(&results, location, measured_at);
    MeasurementReport {
        document_type: contract::DocumentType::Measurement,
        detail,
        schema_version: contract::Version::V3,
        project: results.project,
        metadata,
        summary,
        review,
        breakdown,
        files,
    }
}

/// The report summary; configuration findings enter it only when
/// `findings` holds, as [`Location::summary_findings`] decides.
fn summary_document(
    results: &Results,
    summary: locmetry::ReportSummary,
    module_files: Vec<locmetry::ModuleFile>,
    findings: bool,
) -> contract::ReportSummary {
    let members = results
        .members
        .iter()
        .map(|member| {
            let totals = summary
                .folders
                .get(&member.name)
                .map(|folder| folder.recursive.clone())
                .unwrap_or_default();
            (member.name.clone(), totals)
        })
        .collect();
    contract::ReportSummary {
        top_folders: summary.top_folders(),
        totals: summary.totals,
        members,
        unclaimed_folders: Some(unclaimed_folders(&results.rows)),
        build_modules: build_systems(&results.module_shares)
            .into_iter()
            .map(|(k, v)| (k.into(), v))
            .collect(),
        submodules: results.submodules.clone(),
        largest_module_files: Some(module_files),
        unused_entries: kept_if(findings, &results.unused_entries),
        unused_exclusions: kept_if(findings, &results.unused_exclusions),
        policy_entries: kept_if(findings, &results.policy_entries),
    }
}

/// Report metadata, with the keys of a single root or of a workspace.
fn metadata(results: &Results, location: Location, measured_at: u64) -> contract::Metadata {
    let mut metadata = contract::Metadata {
        measured_at_unix_seconds: measured_at,
        locmetry_version: env!("CARGO_PKG_VERSION").into(),
        backend: BACKEND.into(),
        backend_inaccurate: results.inaccurate,
        backend_configuration: locmetry::BACKEND_CONFIGURATION.into(),
        kinds: Some(contract::Kinds {
            default: locmetry::Kind::Source,
            data: DATA_LANGUAGES.iter().map(|l| l.name().into()).collect(),
            documentation: DOCUMENTATION_LANGUAGES
                .iter()
                .map(|l| l.name().into())
                .collect(),
        }),
        coverage: contract::Coverage::from(&results.coverage),
        modules: Some(results.modules.iter().map(contract::Module::from).collect()),
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
    metadata
}

/// The partial-count message: up to ten unavailable files, then how many more.
fn partial_counts(coverage: &Coverage) -> String {
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
    format!(
        "Backend reported incomplete counts, so the output holds partial results. Unavailable files: {}",
        failures.join(", ")
    )
}

/// `list` when `keep` holds, else nothing: findings appear only where requested and owned.
pub(crate) fn kept_if<T: Clone>(keep: bool, list: &[T]) -> Vec<T> {
    match keep {
        true => list.to_vec(),
        false => Vec::new(),
    }
}
