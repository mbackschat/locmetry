//! Scope, classification, and reporting around the embedded Tokei counter.
mod backend;
mod config;
mod output;
mod report;

pub use backend::{
    file_records, language_kind, scan, scope_policy, Coverage, Scan, DATA_LANGUAGES,
    DOCUMENTATION_LANGUAGES,
};
pub use config::{Classification, Profile, Purpose};
pub use output::{render_markdown, validate_output, write_reports};
pub use report::{build_report, Counts, FileRecord, Kind, ReportSummary};

pub const BACKEND: &str = "tokei 15.0.0 (embedded)";
