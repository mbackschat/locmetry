//! Scope, classification, and reporting around the embedded Tokei counter.
mod backend;
pub mod compare;
mod config;
pub mod contract;
pub mod explain;
mod modules;
mod output;
mod report;
mod report_schema;
mod schema;
pub mod style;

pub use backend::{
    file_records, git_submodules, language_kind, scan, scan_subfolder, scope_policy,
    subfolder_path, Coverage, ParseFailure, ParseFailureReason, Scan, DATA_LANGUAGES,
    DOCUMENTATION_LANGUAGES,
};
pub use config::{
    config_schema, user_config_schema, user_defaults_file, workspace_roots, workspace_schema,
    Classification, EffectiveConfig, Policy, Purpose, Scope, ScopeOverrides, CONFIG_FILE,
};
pub use modules::{detect as detect_modules, Module, ModuleDetector, DETECTORS};
pub use output::{
    measurement_time, render_markdown, render_table, render_table_with_options,
    render_table_with_theme, report_text, validate_output, write_report, BarKind, Bars, Findings,
    MemberView, RowOrder, TableOptions,
};
pub use report::{
    breakdown, build_report, build_systems, module_shares, review, submodule_shares,
    unclassified_folders, unused_entries, BreakdownRow, BuildSystemShare, Counts, FileRecord,
    FolderShare, Kind, ModuleShare, ReportSummary, ReviewEntry, SubmoduleShare, Summary,
};
pub use report_schema::report_schema;

pub const BACKEND: &str = "tokei 15.0.0 (embedded)";
