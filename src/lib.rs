//! Scope, classification, and reporting around the embedded Tokei counter.
mod backend;
pub mod compare;
mod config;
pub mod contract;
pub mod explain;
mod modules;
mod output;
mod paths;
mod report;
mod report_schema;
mod schema;
pub mod style;

pub use backend::{
    file_records, git_submodules, language_family, language_kind, scan, scan_root, scan_subfolder,
    scope_policy, source_language, subfolder_path, Coverage, ParseFailure, ParseFailureReason,
    Scan, DATA_LANGUAGES, DOCUMENTATION_LANGUAGES,
};
pub use config::{
    config_schema, merged_policy_entries, unused_entries, user_config_schema, user_defaults_file,
    workspace_definition, workspace_schema, Classification, Displacement, EffectiveConfig,
    EntryStage, Policy, PolicyEntry, Preemption, Purpose, Scope, ScopeOverrides,
    WorkspaceDefinition, CONFIG_FILE, RECOGNITION,
};
pub use modules::{detect as detect_modules, detector_ids as module_detector_ids, Module};
pub use output::{
    measurement_time, render_markdown, render_table_with_options, report_text, validate_output,
    write_report, BarKind, Bars, CategoryPattern, Findings, Fold, LanguageVariants, MemberView,
    RowOrder, TableOptions,
};
pub use paths::{filesystem_path, member_path, within};
pub use report::{
    breakdown, build_report, build_systems, largest_module_files, module_shares, review,
    submodule_shares, unclaimed_folders, BreakdownRow, BuildSystemShare, Counts, FileRecord,
    FolderShare, Kind, ModuleFile, ModuleShare, ReportSummary, ReviewEntry, SubmoduleShare,
    Summary,
};
pub use report_schema::report_schema;

pub const BACKEND: &str = "tokei 15.0.0+locmetry.30 (embedded)";
pub const BACKEND_CONFIGURATION: &str =
    "Explicit defaults; ambient Tokei configuration is not loaded";
