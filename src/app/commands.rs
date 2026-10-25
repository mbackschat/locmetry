//! The measurement, workspace, comparison and explanation commands.
use super::*;

pub(crate) fn run_explain(args: &ExplainArgs, styling: Styling) -> Result<Outcome> {
    let user_path = user_defaults(args.no_user_config);
    let workspace = args
        .workspace
        .as_deref()
        .map(|file| {
            dunce::canonicalize(file)
                .with_context(|| format!("Cannot read workspace file '{}'", file.display()))
        })
        .transpose()?;
    let members = args
        .workspace
        .as_deref()
        .map(|file| workspace_members(locmetry::workspace_definition(file)?))
        .transpose()?;
    // Every target is inspected before printing, so a failure leaves no
    // partial output behind.
    let explanations = args
        .paths
        .iter()
        .map(|path| {
            let workspace = workspace.as_deref().zip(members.as_deref());
            explain_path(args, workspace, user_path.as_deref(), path)
        })
        .collect::<Result<Vec<_>>>()?;
    let format = args
        .format
        .unwrap_or_else(|| match io::stdout().is_terminal() {
            true => ExplainFormat::Table,
            false => ExplainFormat::Json,
        });
    for (index, explanation) in explanations.iter().enumerate() {
        let separator = if index > 0 { "\n" } else { "" };
        match format {
            // One path keeps the pretty document; several are JSON Lines.
            ExplainFormat::Json if explanations.len() == 1 => {
                println!("{}", serde_json::to_string_pretty(explanation)?)
            }
            ExplainFormat::Json => println!("{}", serde_json::to_string(explanation)?),
            ExplainFormat::Table => print!(
                "{separator}{}",
                explanation.render_with_theme(false, styling.theme(io::stdout().is_terminal()))
            ),
            ExplainFormat::Markdown => print!("{separator}{}", explanation.render(true)),
        }
    }
    let partial: Vec<&str> = explanations
        .iter()
        .filter(|explanation| explanation.partial())
        .map(|explanation| explanation.path.as_str())
        .collect();
    Ok(match partial.is_empty() {
        true => Outcome::Complete,
        false => Outcome::Partial(format!("Counts unavailable: {}", partial.join(", "))),
    })
}

/// One target's explanation, under its member's policy in a workspace given
/// as its file and resolved members.
pub(crate) fn explain_path(
    args: &ExplainArgs,
    workspace: Option<(&Path, &[WorkspaceMember])>,
    user_path: Option<&Path>,
    path: &Path,
) -> Result<locmetry::explain::Explanation> {
    let (root, target, member) = match workspace {
        Some((_, members)) => {
            let mut components = path.components();
            let name = match components.next() {
                Some(std::path::Component::Normal(name)) => {
                    name.to_str().context("Member name must be UTF-8")?
                }
                _ => anyhow::bail!("Workspace query must be MEMBER/PATH"),
            };
            let member = members
                .iter()
                .find(|member| member.name == name)
                .with_context(|| format!("Unknown workspace member '{name}'"))?;
            let target = match &member.folder {
                Some(folder) => Path::new(folder).join(components.as_path()),
                None => components.as_path().to_owned(),
            };
            (
                member.root.clone(),
                target,
                Some((member.name.clone(), member.folder.as_deref())),
            )
        }
        None => (
            args.root.root.clone().unwrap_or_else(|| PathBuf::from(".")),
            path.to_owned(),
            None,
        ),
    };
    let overrides = args.root.overrides();
    let mut prepared = prepare(
        &root,
        args.root.config.clone(),
        user_path,
        workspace.map(|(file, _)| file),
        &overrides,
    )?;
    let explained = locmetry::explain::inspect(&prepared.root, &target, &mut prepared.policy);
    // A member-relative path in the error would not be what the user typed.
    let (path, result) = match member {
        Some(_) => explained.with_context(|| format!("Cannot explain '{}'", path.display()))?,
        None => explained?,
    };
    let mut explanation = locmetry::explain::Explanation {
        document_type: locmetry::explain::DocumentType::Explanation,
        schema_version: locmetry::explain::Version::V3,
        root: prepared.root.to_string_lossy().into(),
        path,
        member: None,
        config_path: prepared.config_path.map(|p| p.to_string_lossy().into()),
        user_config_path: user_path.map(|p| p.to_string_lossy().into()),
        scope: scope_policy(&prepared.policy),
        result,
    };
    if let Some((member, folder)) = member {
        explanation.qualify(member, folder);
    }
    Ok(explanation)
}

pub(crate) fn run_compare(args: &CompareArgs, styling: Styling) -> Result<Outcome> {
    let destination = output_destination(
        &args.output,
        args.format,
        &args.view.single_format_options(),
    );
    args.view.warn_ineffective(&destination, styling);
    let before = locmetry::compare::read_input(&args.before)?;
    let after = locmetry::compare::read_input(&args.after)?;
    if let Some(path) = destination.path {
        locmetry::compare::validate_destination(path, [&args.before, &args.after])?;
    }
    let result = locmetry::compare::compare(
        before,
        after,
        args.before.to_string_lossy().into(),
        args.after.to_string_lossy().into(),
    )?;
    let result = match &args.member {
        Some(member) => result.select_member(member)?,
        None => result,
    };
    let partial = result.partial();
    let variants = args.view.variants();
    let text = match destination.format {
        Format::Json => report_text(&serde_json::to_value(&result)?)?,
        Format::Headline => format!("{}\n", serde_json::to_string(&result.headline())?),
        Format::Table => {
            let theme = styling.theme(destination.path.is_none() && io::stdout().is_terminal());
            match args.view.tables_only {
                true => locmetry::compare::render_tables(&result, theme, variants),
                false => locmetry::compare::render_table_with_theme(&result, theme, variants),
            }
        }
        Format::Markdown => locmetry::compare::render_markdown(&result, variants),
    };
    destination.write(&text)?;
    Ok(if partial {
        Outcome::Partial(
            "Net change uses partial input measurements. Inspect the coverage findings.".into(),
        )
    } else {
        Outcome::Complete
    })
}

pub(crate) fn run_root(
    args: &RootArgs,
    shared: &SharedArgs,
    subfolder: Option<&Path>,
    folders: bool,
    folder_rows: Option<usize>,
    styling: Styling,
) -> Result<Outcome> {
    let root = args.root.as_deref().unwrap_or(Path::new("."));
    let mut destination = destination(
        shared,
        &[
            ("--show-folders", folders, Format::Table),
            ("--folder-rows", folder_rows.is_some(), Format::Table),
        ],
    )?;
    if folder_rows.is_some() {
        anyhow::ensure!(
            destination.shows(Format::Table),
            "'--folder-rows' requires table output. {}",
            destination.format_remedy(Format::Table)
        );
    }
    destination.folders = folders;
    destination.folder_rows = folder_rows;
    if folders {
        destination.warn_ineffective("--show-folders", &[Format::Table], styling);
    }
    warn_ineffective_options(shared, &destination, styling);
    let user_path = user_defaults(shared.no_user_config);
    let overrides = args.overrides();
    let measured_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    let prepared = prepare(
        root,
        args.config.clone(),
        user_path.as_deref(),
        None,
        &overrides,
    )?;
    if let Some(path) = destination.path {
        validate_output(path, &prepared.root, prepared.config_path.as_deref())?;
    }
    let subfolder = subfolder
        .map(|path| locmetry::subfolder_path(&prepared.root, path))
        .transpose()?
        .flatten();
    if let Some(folder) = &subfolder {
        let inactive = locmetry::filesystem_path(&prepared.root, folder).join(CONFIG_FILE);
        if inactive.is_file()
            && prepared
                .config_path
                .as_ref()
                .and_then(|p| dunce::canonicalize(p).ok())
                != dunce::canonicalize(&inactive).ok()
        {
            let project_config = prepared
                .config_path
                .as_ref()
                .map(dunce::canonicalize)
                .transpose()?;
            styling.stderr_message(Severity::Warning, format_args!(
                "'{}' is inactive. Project configuration: {}. '--subfolder' retains project policy.",
                inactive.display(),
                project_config.as_ref().map(|p| p.display().to_string()).unwrap_or_else(|| "none (built-in settings)".into())
            ));
        }
    }
    let measurement = measure(prepared, subfolder.as_deref())?;
    let mut scope = scope_policy(&measurement.policy);
    scope.subfolder = subfolder;

    let location = Location::Root {
        root: measurement.root,
        scope,
        config_path: measurement.config_path,
        user_path,
        config: Box::new(measurement.policy.snapshot().clone()),
    };
    let results = Results {
        project: measurement.name,
        modules: measurement.policy.modules().to_vec(),
        module_shares: measurement.modules,
        submodules: measurement.submodules,
        unused_entries: measurement.unused_entries,
        unused_exclusions: measurement.unused_exclusions,
        policy_entries: measurement.policy_entries,
        coverage: measurement.coverage,
        inaccurate: measurement.inaccurate,
        rows: measurement.rows,
        members: Vec::new(),
    };
    report(
        shared,
        &destination,
        results,
        location,
        measured_at,
        styling,
    )
}

pub(crate) fn run_workspace(args: &WorkspaceArgs, styling: Styling) -> Result<Outcome> {
    let shared = &args.shared;
    let destination = destination(shared, &[])?;
    warn_ineffective_options(shared, &destination, styling);
    let workspace_path = args
        .file
        .as_deref()
        .map(|file| {
            dunce::canonicalize(file)
                .with_context(|| format!("Cannot read workspace file '{}'", file.display()))
        })
        .transpose()?;
    // The file is read under the name the user typed, so errors about it and
    // the roots it lists name paths the user can recognise.
    let definition = match &args.file {
        Some(file) => locmetry::workspace_definition(file)?,
        None => locmetry::WorkspaceDefinition::Roots(args.roots.clone()),
    };
    let members = workspace_members(definition)?;
    // Folder members share their project's policy, so its findings are the
    // workspace's own rather than one member's.
    let shared_policy = members.iter().all(|member| member.folder.is_some());
    if let Some(path) = destination.path {
        for member in &members {
            validate_output(path, &member.root, workspace_path.as_deref())?;
        }
    }
    let user_path = user_defaults(shared.no_user_config);
    let measured_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
    // Members run in parallel; collecting in name order keeps the merge and
    // the reported error deterministic whichever thread finishes first.
    let measurements = std::thread::scope(|threads| {
        let handles: Vec<_> = members
            .iter()
            .map(|member| {
                let (user_path, workspace_path) = (user_path.as_deref(), workspace_path.as_deref());
                let handle = threads.spawn(move || {
                    let overrides = ScopeOverrides::default();
                    measure(
                        prepare(&member.root, None, user_path, workspace_path, &overrides)?,
                        member.folder.as_deref(),
                    )
                });
                (&member.name, handle)
            })
            .collect();
        handles
            .into_iter()
            .map(|(name, handle)| {
                handle
                    .join()
                    .map_err(|_| anyhow::anyhow!("Workspace member '{name}' panicked"))?
                    .with_context(|| format!("Workspace member '{name}'"))
            })
            .collect::<Result<Vec<_>>>()
    })?;
    let mut results = Results {
        project: members
            .iter()
            .map(|member| member.name.as_str())
            .collect::<Vec<_>>()
            .join(" + "),
        rows: Vec::new(),
        coverage: Coverage::default(),
        inaccurate: false,
        modules: Vec::new(),
        module_shares: Vec::new(),
        submodules: Vec::new(),
        unused_entries: Vec::new(),
        unused_exclusions: Vec::new(),
        policy_entries: Vec::new(),
        members: Vec::new(),
    };
    // Unused in every member folder means unused by the shared policy; an
    // independent member's findings name it instead.
    if shared_policy {
        results.unused_exclusions = common(measurements.iter().map(|m| &m.unused_exclusions));
        results.policy_entries = locmetry::merged_policy_entries(
            measurements
                .iter()
                .map(|m| m.policy_entries.clone())
                .collect(),
        );
        results.unused_entries = locmetry::unused_entries(&results.policy_entries);
    }
    let mut records = Vec::new();
    for (member, measurement) in members.iter().zip(measurements) {
        let name = &member.name;
        let folder = member.folder.as_deref();
        let prefixed = |path: &str| locmetry::member_path(name, folder, path);
        let mut scope = scope_policy(&measurement.policy);
        scope.subfolder = member.folder.clone();
        // A shared policy's findings are reported once, for the workspace.
        records.push(contract::Member {
            name: name.clone(),
            root: measurement.root.to_string_lossy().into(),
            config_path: measurement
                .config_path
                .as_ref()
                .map(|p| p.to_string_lossy().into()),
            scope,
            unused_entries: report::kept_if(!shared_policy, &measurement.unused_entries),
            unused_exclusions: report::kept_if(!shared_policy, &measurement.unused_exclusions),
            policy_entries: report::kept_if(!shared_policy, &measurement.policy_entries),
            coverage: (&measurement.coverage).into(),
            config: Some(measurement.policy.snapshot().clone()),
        });
        let unclaimed = measurement.rows.iter().filter(|row| row.needs_review());
        results.members.push(MemberView {
            name: name.clone(),
            unclaimed_files: unclaimed.clone().count() as u64,
            unclaimed_code: unclaimed.map(|row| row.counts.code).sum(),
            unread_files: measurement.coverage.unread.len(),
            parse_failed_files: measurement.coverage.parse_failures.len(),
        });
        results
            .rows
            .extend(measurement.rows.into_iter().map(|mut row| {
                row.path = prefixed(&row.path);
                if let Some(module) = &mut row.module {
                    module.path = prefixed(&module.path);
                }
                row
            }));
        results.coverage.merge(&measurement.coverage, prefixed);
        results.inaccurate |= measurement.inaccurate;
        // A folder member keeps the modules inside it, and enclosing project
        // modules only where they classified its files.
        let relevant = |module: &Module| match folder {
            None => true,
            Some(folder) => {
                locmetry::within(&module.path, folder)
                    || measurement
                        .modules
                        .iter()
                        .any(|share| share.path == module.path)
            }
        };
        results.modules.extend(
            measurement
                .policy
                .modules()
                .iter()
                .filter(|module| relevant(module))
                .map(|module| Module {
                    path: prefixed(&module.path),
                    ..module.clone()
                }),
        );
        results
            .module_shares
            .extend(measurement.modules.into_iter().map(|mut share| {
                share.path = prefixed(&share.path);
                share
            }));
        results
            .submodules
            .extend(measurement.submodules.into_iter().map(|mut share| {
                share.path = prefixed(&share.path);
                share
            }));
        if !shared_policy {
            results.unused_entries.extend(
                measurement
                    .unused_entries
                    .iter()
                    .map(|id| format!("{id} ({name})")),
            );
            results.unused_exclusions.extend(
                measurement
                    .unused_exclusions
                    .iter()
                    .map(|pattern| format!("{pattern} ({name})")),
            );
            results
                .policy_entries
                .extend(
                    measurement
                        .policy_entries
                        .iter()
                        .map(|entry| locmetry::PolicyEntry {
                            id: format!("{} ({name})", entry.id),
                            ..entry.clone()
                        }),
                );
        }
    }
    // Prefixes keep each member's records together but can reorder them
    // against plain path order (`a-b/` sorts before `a/`), so sort once more.
    results.rows.sort_by(|a, b| a.path.cmp(&b.path));
    results.coverage.unread.sort();
    results
        .coverage
        .parse_failures
        .sort_by(|a, b| a.path.cmp(&b.path));
    results.coverage.unrecognized.sort();
    let location = Location::Workspace {
        members: records,
        workspace_path,
        user_path,
        shared_policy,
    };
    report(
        shared,
        &destination,
        results,
        location,
        measured_at,
        styling,
    )
}
