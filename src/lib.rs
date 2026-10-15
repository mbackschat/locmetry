//! Scope, classification, and reporting around the embedded Tokei counter.
mod backend;
mod config;
mod output;
mod report;

pub use backend::{
    file_records, language_kind, scan, scope_policy, Coverage, Scan, DATA_LANGUAGES,
    DOCUMENTATION_LANGUAGES,
};
pub use config::{Classification, Policy, Purpose, Scope, ScopeOverrides, CONFIG_FILE};
pub use output::{render_markdown, render_table, report_text, validate_output, write_report};
pub use report::{
    breakdown, build_report, review, BreakdownRow, Counts, FileRecord, Kind, ReportSummary,
    ReviewEntry,
};

pub const BACKEND: &str = "tokei 15.0.0 (embedded)";
