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

    let scope = strict(
        json!({
            "hidden": {"type": "boolean"},
            "respect_ignore_files": {"type": "boolean"},
            "ignore_files": strings("Ignore files that limited the selection."),
            "symlinks": {"type": "string"},
            "exclude": strings("Effective exclusion globs: defaults, personal defaults, the project's, the command line's, and the loaded configuration file itself."),
        }),
        &[
            "hidden",
            "respect_ignore_files",
            "ignore_files",
            "symlinks",
            "exclude",
        ],
        "Effective file selection.",
    );

    let mut metadata = strict(
        json!({
            "root": {"type": "string", "description": "Canonical scan root; symbolic links are resolved. Absent in workspace mode, where each member has its own."},
            "measured_at_unix_seconds": {"type": "integer"},
            "locmetry_version": {"type": "string"},
            "backend": {"type": "string", "description": "The embedded counter and its version."},
            "backend_inaccurate": {"type": "boolean", "description": "Whether some files could not be read or counted, so counts are partial; the exit code is then 2. Historical reports without parse_failures did not observe notebook decoding failures."},
            "backend_configuration": {"type": "string"},
            "scope": {"$ref": "#/$defs/Scope", "description": "Effective file selection; absent in workspace mode, where each member has its own."},
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
                    "parse_failures": {
                        "type": "array",
                        "description": "Selected readable files without counts due to invalid or unsupported notebook content. Always emitted by current measurements, including an empty array, in headlines and full reports. Optional only for historical version-1 reports: absence means not recorded, never known zero failures. These files are neither counted nor unread/unrecognized; any failure sets backend_inaccurate and exit 2.",
                        "items": strict(json!({
                            "path": {"type":"string", "description":"Root-relative path, prefixed by member in a workspace."},
                            "reason": {"type":"string", "enum":["invalid_notebook","unsupported_notebook_version","unsupported_notebook_language","unsupported_notebook_cell"], "description":"Stable reason code; unsupported content is not necessarily invalid nbformat."},
                            "message": {"type":"string", "description":"Human-readable detail; branch on reason rather than parsing this text."},
                        }), &["path","reason","message"], "One file with unavailable counts."),
                    },
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
            "config_path": {"type": ["string", "null"], "description": "Loaded project configuration, or null for built-in settings; absent in workspace mode."},
            "user_config_path": {"type": ["string", "null"], "description": "Loaded personal defaults, or null."},
            "config": {
                "type": "object",
                "description": "Full report only: the effective configuration after merging personal defaults, shaped as `locmetry --schema config` describes, plus the applied `builtin_conventions` and `builtin_language_mappings`. Absent in workspace mode.",
            },
            "workspace_path": {"type": "string", "description": "Workspace mode only: the canonical workspace file that `--file` named; absent when `--root` named the members."},
            "members": {
                "type": "array",
                "items": strict(
                    json!({
                        "name": {"type": "string", "description": "The member root's folder name, which prefixes its paths in this report."},
                        "root": {"type": "string", "description": "Canonical member root."},
                        "config_path": {"type": ["string", "null"], "description": "The member's `.locmetryrc`, or null for built-in settings."},
                        "scope": {"$ref": "#/$defs/Scope"},
                        "unused_entries": strings("The member's configuration entries that decided no file."),
                        "unused_exclusions": strings("The member's configured exclusions that left out nothing."),
                        "coverage": strict(
                            json!({
                                "selected_files": {"type": "integer"},
                                "counted_files": {"type": "integer"},
                                "mapped_files": {"type": "integer"},
                                "unrecognized_files": {"type": "integer"},
                                "unread_files": {"type": "integer"},
                                "parse_failed_files": {"type":"integer", "minimum":0, "description":"Member files listed in the workspace parse_failures array. Always emitted now, including zero; historical absence means not recorded."},
                            }),
                            &["selected_files", "counted_files", "mapped_files", "unrecognized_files", "unread_files"],
                            "Which of the member's selected files were counted; `metadata.coverage` lists the paths for the whole workspace.",
                        ),
                        "config": {"type": "object", "description": "Full report only: the member's effective configuration."},
                    }),
                    &["name", "root", "config_path", "scope", "unused_entries", "unused_exclusions", "coverage"],
                    "One repository of the workspace, measured with its own configuration.",
                ),
                "description": "Present only in workspace mode (`locmetry workspace`), in name order. Settings and configuration findings relative to one member live here instead of in `root`, `scope`, `config_path`, `config`, and `summary.unused_entries` or `summary.unused_exclusions`.",
            },
        }),
        &[
            "measured_at_unix_seconds",
            "locmetry_version",
            "backend",
            "backend_inaccurate",
            "backend_configuration",
            "coverage",
            "user_config_path",
        ],
        "How the measurement was made.",
    );
    // A regular run reports one root, a workspace its members; never both.
    metadata["oneOf"] = json!([
        {"required": ["root", "scope", "config_path"], "not": {"anyOf": [{"required": ["members"]}, {"required": ["workspace_path"]}]}},
        {"required": ["members"], "not": {"anyOf": [{"required": ["root"]}, {"required": ["scope"]}, {"required": ["config_path"]}]}},
    ]);

    // The summary's own totals, which `summary.members` repeats per member.
    let totals = json!({
        "all_counted": {"$ref": "#/$defs/Counts", "description": "Every counted file of every kind."},
        "source": {"$ref": "#/$defs/Counts", "description": "Source files only; every other summary below is restricted to source."},
        "by_kind_purpose": counts_map("Counts keyed `kind:purpose`, such as `data:test`, over every kind."),
        "source_purpose": counts_map("Source counts keyed by purpose: `production`, `test`, `unclassified`."),
        "source_categories": counts_map("Source counts keyed `purpose:category`, such as `production:tooling`; filter `production:implementation` for product code alone."),
        "source_languages": counts_map("Source counts keyed by language."),
    });
    let mut summary_properties = totals.clone();
    let summary_properties_rest = json!({
        "members": {
            "type": "object",
            "additionalProperties": {"$ref": "#/$defs/Totals"},
            "description": "Workspace mode only: each member's totals, keyed by member name in name order; they equal what a regular run of that member reports.",
        },
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
        "submodules": {
            "type": "array",
            "items": strict(
                json!({
                    "path": {"type": "string", "description": "Submodule path from `.gitmodules`."},
                    "files": {"type": "integer"},
                    "code": {"type": "integer", "description": "Source code lines measured inside it."},
                }),
                &["path", "files", "code"],
                "One Git submodule with measured source.",
            ),
            "description": "Present when Git submodules declared in the scan root's `.gitmodules` hold measured source. Submodules are often other teams' code: decide their scope, usually with `scope.exclude`; an excluded submodule is no longer listed.",
        },
        "unused_entries": {
            "type": "array",
            "items": {"type": "string"},
            "description": "Present when configuration entries decided no file: their globs match nothing, or an earlier stage, such as a test convention, always decides first. Check their `match` or move them to `overrides`.",
        },
        "unused_exclusions": {
            "type": "array",
            "items": {"type": "string"},
            "description": "Present when configured exclusions, from the project, user defaults, or `--exclude`, left out no file or folder: a folder written without `/**`, a misspelled path, or one that ignore files or `hidden = false` already hide. Default exclusions and the configuration file's own path are never listed.",
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
    });
    summary_properties
        .as_object_mut()
        .expect("an object")
        .extend(
            summary_properties_rest
                .as_object()
                .expect("an object")
                .clone(),
        );
    let summary = strict(
        summary_properties,
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
        "$defs": {
            "Counts": counts,
            "Scope": scope,
            "Totals": strict(totals, &["all_counted"], "Counts of one member, shaped like the summary's totals."),
        },
    })
}
