//! Scope, classification, and reporting around the embedded Tokei counter.
mod backend;
mod config;
mod modules;
mod output;
mod report;

pub use backend::{
    file_records, language_kind, scan, scope_policy, Coverage, Scan, DATA_LANGUAGES,
    DOCUMENTATION_LANGUAGES,
};
pub use config::{
    config_schema, user_config_schema, user_defaults_file, Classification, Policy, Purpose, Scope,
    ScopeOverrides, CONFIG_FILE,
};
pub use modules::{detect as detect_modules, Module, ModuleDetector, DETECTORS};
pub use output::{
    render_markdown, render_table, report_text, validate_output, write_report, RowOrder,
};
pub use report::{
    breakdown, build_report, review, unclassified_folders, BreakdownRow, Counts, FileRecord,
    FolderShare, Kind, ReportSummary, ReviewEntry,
};

pub const BACKEND: &str = "tokei 15.0.0 (embedded)";
