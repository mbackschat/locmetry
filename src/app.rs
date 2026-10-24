//! The executable's commands, built on the library; `main.rs` parses the
//! command line and dispatches to them.
mod cli;
mod commands;
mod destination;
mod measure;
mod parse;
mod reference;
mod report;
#[cfg(test)]
mod tests;

pub(crate) use cli::*;
pub(crate) use commands::*;
use destination::*;
use measure::*;
pub(crate) use parse::*;
pub(crate) use reference::*;
use report::*;

use crate::guides::Guide;
use anyhow::{Context, Result};
use clap::{CommandFactory, FromArgMatches, Parser, Subcommand, ValueEnum};
use locmetry::contract::{self, MeasurementReport};
use locmetry::style::{plain, Role, Theme};
use locmetry::{
    breakdown, build_report, build_systems, git_submodules, largest_module_files, module_shares,
    render_markdown, render_table_with_options, report_text, review, scan, scope_policy,
    submodule_shares, unclaimed_folders, unused_entries, user_defaults_file, validate_output,
    write_report, BarKind, Bars, Coverage, FileRecord, Findings, LanguageVariants, MemberView,
    Module, ModuleShare, Policy, Purpose, RowOrder, ScopeOverrides, SubmoduleShare, TableOptions,
    BACKEND, CONFIG_FILE, DATA_LANGUAGES, DOCUMENTATION_LANGUAGES,
};
use std::{
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

/// Result of a run that produced a report.
pub(crate) enum Outcome {
    Complete,
    /// Counts are unavailable; the message names read and parsing failures.
    Partial(String),
}

pub(crate) fn user_defaults(no_user_config: bool) -> Option<PathBuf> {
    user_defaults_file().filter(|path| !no_user_config && path.is_file())
}
