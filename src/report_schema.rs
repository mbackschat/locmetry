//! JSON Schema of the report, for agents and scripts that read it.
//!
//! The report is assembled as JSON rather than from one typed struct, so this
//! schema is written by hand; tests validate real headlines and full reports
//! against it with unknown keys rejected, which keeps it from drifting.
use serde_json::{json, Value};

/// JSON Schema of both the headline (stdout) and the full report
/// (`--output`); keys only the full report carries are optional.
pub fn report_schema() -> Value {
    let strict = |properties: Value, required: &[&str], description: &str| {
        json!({
            "type": "object",
            "additionalProperties": false,
            "properties": properties,
            "required": required,
            "description": description,
        })
    };
    let counts_map = |description: &str| {
        json!({
            "type": "object",
            "additionalProperties": {"$ref": "#/$defs/Counts"},
            "description": description,
        })
    };
    let strings = |description: &str| json!({"type": "array", "items": {"type": "string"}, "description": description});
    let line_counts = strict(
        json!({
            "code": {"type": "integer", "description": "Lines with code."},
            "comments": {"type": "integer", "description": "Comment lines, including documentation embedded in source."},
            "blanks": {"type": "integer", "description": "Blank lines."},
            "lines": {"type": "integer", "description": "All lines: code, comments, and blanks."},
        }),
        &["code", "comments", "blanks", "lines"],
        "Line counts of one file.",
    );
    let mut counts = line_counts.clone();
    counts["properties"]["files"] = json!({"type": "integer", "description": "Files counted."});
    counts["required"] = json!(["files", "code", "comments", "blanks", "lines"]);
    counts["description"] = json!("Line counts summed over files.");

    let metadata = strict(
        json!({
            "root": {"type": "string", "description": "Canonical scan root; symbolic links are resolved."},
            "measured_at_unix_seconds": {"type": "integer"},
            "locmetry_version": {"type": "string"},
            "backend": {"type": "string", "description": "The embedded counter and its version."},
            "backend_inaccurate": {"type": "boolean", "description": "Whether some files could not be read, so counts are partial; the exit code is then 2."},
            "backend_configuration": {"type": "string"},
            "scope": strict(
                json!({
                    "hidden": {"type": "boolean"},
                    "respect_ignore_files": {"type": "boolean"},
                    "ignore_files": strings("Ignore files that limited the selection."),
                    "symlinks": {"type": "string"},
                    "exclude": strings("Effective exclusion globs: defaults, personal defaults, the project's, the command line's, and the loaded configuration file itself."),
                }),
                &["hidden", "respect_ignore_files", "ignore_files", "symlinks", "exclude"],
                "Effective file selection.",
            ),
            "kinds": strict(
                json!({
                    "default": {"type": "string", "description": "Kind of every language not listed below."},
                    "data": strings("Languages counted as data."),
                    "documentation": strings("Languages counted as documentation."),
                }),
                &["default", "data", "documentation"],
                "Full report only: which languages count as which content kind.",
            ),
            "coverage": strict(
                json!({
                    "selected_files": {"type": "integer", "description": "Files in scope."},
                    "counted_files": {"type": "integer", "description": "Selected files with counts, including mapped ones."},
                    "mapped_files": {"type": "integer", "description": "Files counted through a language mapping."},
                    "unrecognized_files": {"type": "integer", "description": "Selected files no language recognizes; their lines are missing, not zero."},
                    "unrecognized_extensions": {"type": "object", "additionalProperties": {"type": "integer"}, "description": "Unrecognized files per extension; `(none)` without one."},
                    "unrecognized": strings("Full report only: paths of unrecognized files; a `[[languages.mappings]]` entry can count them."),
                    "unread": strings("Recognized files that could not be read."),
                }),
                &["selected_files", "counted_files", "mapped_files", "unrecognized_files", "unrecognized_extensions", "unread"],
                "Which selected files were counted.",
            ),
            "modules": {
                "type": "array",
                "items": strict(
                    json!({
                        "path": {"type": "string", "description": "Module folder; `.` is the scan root."},
                        "detector": {"type": "string", "description": "Build system, such as `gradle` or `npm`."},
                    }),
                    &["path", "detector"],
                    "A folder whose build file declares a source layout.",
                ),
                "description": "Full report only: every adopted build module; empty when `classification.builtin_modules` is false.",
            },
            "config_path": {"type": ["string", "null"], "description": "Loaded project configuration, or null for built-in settings."},
            "user_config_path": {"type": ["string", "null"], "description": "Loaded personal defaults, or null."},
            "config": {
                "type": "object",
                "description": "Full report only: the effective configuration after merging personal defaults, shaped as `locmetry --schema config` describes, plus the applied `builtin_conventions` and `builtin_language_mappings`.",
            },
        }),
        &[
            "root",
            "measured_at_unix_seconds",
            "locmetry_version",
            "backend",
            "backend_inaccurate",
            "backend_configuration",
            "scope",
            "coverage",
            "config_path",
            "user_config_path",
        ],
        "How the measurement was made.",
    );

    let summary = strict(
        json!({
            "all_counted": {"$ref": "#/$defs/Counts", "description": "Every counted file of every kind."},
            "source": {"$ref": "#/$defs/Counts", "description": "Source files only; every other summary below is restricted to source."},
            "by_kind_purpose": counts_map("Counts keyed `kind:purpose`, such as `data:test`, over every kind."),
            "source_purpose": counts_map("Source counts keyed by purpose: `production`, `test`, `unclassified`."),
            "source_categories": counts_map("Source counts keyed `purpose:category`, such as `production:tooling`; filter `production:implementation` for product code alone."),
            "source_languages": counts_map("Source counts keyed by language."),
            "top_folders": {
                "type": "object",
                "additionalProperties": {
                    "type": "object",
                    "additionalProperties": {"type": "integer"},
                    "description": "Source code lines per purpose.",
                },
                "description": "Each top-level folder's whole subtree; `.` holds the files directly in the root. Together they cover every file once.",
            },
            "build_modules": {
                "type": "object",
                "additionalProperties": strict(
                    json!({
                        "modules": {"type": "integer"},
                        "code": {"type": "integer", "description": "Source code lines the modules' layouts classified as production; files that an earlier stage decided, such as build scripts, are not included."},
                    }),
                    &["modules", "code"],
                    "One build system.",
                ),
                "description": "Present when build modules were adopted; keyed by build system.",
            },
            "unclassified_folders": {
                "type": "array",
                "items": strict(
                    json!({
                        "folder": {"type": "string", "description": "Root-relative folder; `.` holds files directly in the root."},
                        "files": {"type": "integer"},
                        "code": {"type": "integer"},
                    }),
                    &["folder", "files", "code"],
                    "Unclaimed source below one folder.",
                ),
                "description": "Non-overlapping folders that together hold all unclaimed source, largest first; empty when every file is decided. These are the folders a configuration still needs entries for.",
            },
        }),
        &["all_counted", "top_folders", "unclassified_folders"],
        "Totals; zero counts and empty maps are omitted, so a missing entry means zero.",
    );

    let purpose = json!({"enum": ["production", "test", "unclassified"]});
    let kind = json!({"enum": ["source", "data", "documentation", "other"]});
    let classification = |extra: Value| {
        let mut properties = json!({
            "path": {"type": "string", "description": "Root-relative path."},
            "language": {"type": "string"},
            "category": {"type": "string"},
            "rule": {"type": "string", "description": "What decided the classification: an entry ID, `builtin:<convention>`, `builtin:<detector>-module`, or `fallback`."},
        });
        properties
            .as_object_mut()
            .unwrap()
            .extend(extra.as_object().unwrap().clone());
        properties
    };
    let review_entry = strict(
        classification(json!({
            "code": {"type": "integer"},
            "lines": {"type": "integer"},
        })),
        &["path", "language", "category", "rule", "code", "lines"],
        "An unclaimed source file.",
    );
    let mut breakdown_row = counts.clone();
    breakdown_row["properties"]
        .as_object_mut()
        .unwrap()
        .extend(
            json!({
                "folder": {"type": "string", "description": "The folder holding the files directly; `.` is the root."},
                "kind": kind,
                "purpose": purpose,
                "category": {"type": "string"},
                "language": {"type": "string", "description": "Embedded languages get rows of their own, with `files` 0."},
            })
            .as_object()
            .unwrap()
            .clone(),
        );
    breakdown_row["required"] = json!([
        "folder", "kind", "purpose", "category", "language", "files", "code", "comments", "blanks",
        "lines"
    ]);
    breakdown_row["description"] =
        json!("Counts of one folder, kind, purpose, category, and language; rows are additive under any grouping.");
    let file_record = strict(
        classification(json!({
            "kind": kind,
            "purpose": purpose,
            "language_mapping": {"type": "string", "description": "Mapping that supplied the language of a file the counter does not recognize; absent otherwise."},
            "counts": line_counts,
            "languages": {
                "type": "object",
                "additionalProperties": line_counts,
                "description": "Per-language counts; present only for files with embedded languages, such as code blocks in Markdown.",
            },
        })),
        &[
            "path", "language", "kind", "purpose", "category", "rule", "counts",
        ],
        "One counted file.",
    );

    json!({
        "$schema": "https://json-schema.org/draft/2020-12/schema",
        "title": "Report",
        "description": "Locmetry report. Stdout prints the headline: everything except `review`, `breakdown`, `files`, `metadata.config`, `metadata.kinds`, `metadata.modules`, and `metadata.coverage.unrecognized`. `--output FILE` writes the full report and the headline names it as `report`; `--output -` prints the full report instead; `--files` adds `files`. Purposes and categories follow `locmetry --schema config`.",
        "type": "object",
        "additionalProperties": false,
        "properties": {
            "schema_version": {"const": "1"},
            "project": {"type": "string", "description": "Name of the scan root folder."},
            "report": {"type": "string", "description": "Headline only: the full report written by `--output FILE`."},
            "metadata": metadata,
            "summary": summary,
            "review": {"type": "array", "items": review_entry, "description": "Full report only: unclaimed source files, largest first."},
            "breakdown": {"type": "array", "items": breakdown_row, "description": "Full report only: additive rows for any grouping, such as purpose per folder at every depth."},
            "files": {"type": "array", "items": file_record, "description": "With `--files`: one record per counted file."},
        },
        "required": ["schema_version", "project", "metadata", "summary"],
        "$defs": {"Counts": counts},
    })
}
