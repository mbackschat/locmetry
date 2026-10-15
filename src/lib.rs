//! Scope, classification, and reporting around the embedded Tokei counter.
mod backend;
mod config;
pub mod contract;
mod modules;
mod output;
mod report;
mod report_schema;
mod schema;

pub use backend::{
    file_records, git_submodules, language_kind, scan, scope_policy, Coverage, ParseFailure,
    ParseFailureReason, Scan, DATA_LANGUAGES, DOCUMENTATION_LANGUAGES,
};
pub use config::{
    config_schema, user_config_schema, user_defaults_file, workspace_roots, workspace_schema,
    Classification, EffectiveConfig, Policy, Purpose, Scope, ScopeOverrides, CONFIG_FILE,
};
pub use modules::{detect as detect_modules, Module, ModuleDetector, DETECTORS};
pub use output::{
    render_markdown, render_table, report_text, validate_output, write_report, Findings,
    MemberView, RowOrder,
};
pub use report::{
    breakdown, build_report, build_systems, module_shares, review, submodule_shares,
    unclassified_folders, unused_entries, BreakdownRow, BuildSystemShare, Counts, FileRecord,
    FolderShare, Kind, ModuleShare, ReportSummary, ReviewEntry, SubmoduleShare, Summary,
};
pub use report_schema::report_schema;

pub const BACKEND: &str = "tokei 15.0.0 (embedded)";
