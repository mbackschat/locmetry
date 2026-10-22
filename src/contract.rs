//! Saved measurement contract shared by serialization, reading and JSON Schema.
use crate::{Counts, Kind, Purpose, Summary};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeMap;

mod validation;

/// Report contract version, independent of the executable version and
/// shared by measurement, comparison and explanation documents.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum Version {
    /// Explicit document identity and full/headline projection.
    #[serde(rename = "3")]
    V3,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DocumentType {
    Measurement,
}

/// The requested projection; omitted file records still mean detail was not requested.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Detail {
    Full,
    Headline,
}

/// Locmetry measurement: --format json emits the full report; --format headline emits compact JSON.
/// Full reports add review, breakdown,
/// kinds, modules, unrecognized paths and effective policy; --files adds file detail.
/// Missing optional detail is not measured zero. Aggregate summary omissions alone mean zero.
/// Duplicate raw keys in dynamic JSON maps retain the last value; overwritten keys cannot be validated.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(example = headline_example(), transform = measurement_detail)]
pub struct MeasurementReport {
    pub document_type: DocumentType,
    pub detail: Detail,
    pub schema_version: Version,
    /// Scan-root name, or workspace member names joined for display; not verified identity.
    pub project: String,
    pub metadata: Metadata,
    pub summary: ReportSummary,
    /// Full report only: unclaimed source, largest first. Empty is recorded no findings.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "Vec<crate::ReviewEntry>")]
    pub review: Option<Vec<crate::ReviewEntry>>,
    /// Full report only: additive rows by direct parent folder, kind, purpose, category and language.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "Vec<crate::BreakdownRow>")]
    pub breakdown: Option<Vec<crate::BreakdownRow>>,
    /// Only with --files: one record per counted physical file; absence means detail unavailable.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "Vec<File>")]
    pub files: Option<Vec<File>>,
}

/// Counting context. Root and workspace fields are mutually exclusive.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(transform = metadata_modes)]
pub struct Metadata {
    pub measured_at_unix_seconds: u64,
    pub locmetry_version: String,
    /// Embedded counter name and version; Locmetry version also matters for local corrections.
    pub backend: String,
    /// True means partial counts and measurement exit 2, even without identified failure paths. Historical false does not certify notebook correctness.
    pub backend_inaccurate: bool,
    pub backend_configuration: String,
    /// Loaded personal-default path or null; historical absence means not recorded. Location is provenance, not identity.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub user_config_path: Option<Option<String>>,
    pub coverage: Coverage,
    /// Full report only: content-kind interpretation of language contributions.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "Kinds")]
    pub kinds: Option<Kinds>,
    /// Full report only, introduced in v0.1.2. Missing is unavailable package evidence. With subfolder selection this retains whole-project classification context, including modules without measured contributions.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "Vec<Module>")]
    pub modules: Option<Vec<Module>>,
    /// Single-root only: canonical root location; not a source fingerprint.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "String")]
    pub root: Option<String>,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "Selection")]
    pub scope: Option<Selection>,
    /// Single-root only: loaded configuration path or null for built-in settings.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    pub config_path: Option<Option<String>>,
    /// Full single-root report only: effective policy, not current filesystem configuration.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "crate::config::EffectiveConfig")]
    pub config: Option<crate::config::EffectiveConfig>,
    /// Present only in workspace mode, including members with zero counted files; name order.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "Vec<Member>", length(min = 1))]
    pub members: Option<Vec<Member>>,
    /// Workspace file path when explicitly supplied; absent for repeatable --root.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "String")]
    pub workspace_path: Option<String>,
}

fn measurement_detail(schema: &mut schemars::Schema) {
    schema.insert(
        "allOf".into(),
        serde_json::json!([{
            "if": {"properties": {"detail": {"const": "full"}}},
            "then": {
                "required": ["review", "breakdown"],
                "properties": {"metadata": {
                    "required": ["kinds", "modules"],
                    "properties": {
                        "coverage": {"required": ["unrecognized"]},
                        "members": {"items": {"required": ["config"]}}
                    },
                    "if": {"required": ["root"]},
                    "then": {"required": ["config"]}
                }}
            },
            "else": {
                "properties": {
                    "review": false, "breakdown": false, "files": false,
                    "metadata": {"properties": {
                        "kinds": false, "modules": false, "config": false,
                        "coverage": {"properties": {"unrecognized": false}},
                        "members": {"items": {"properties": {"config": false}}}
                    }}
                }
            }
        }]),
    );
}

fn metadata_modes(schema: &mut schemars::Schema) {
    schema.insert("oneOf".into(), serde_json::json!([
        {"required":["root","scope","config_path"],"not":{"anyOf":[{"required":["members"]},{"required":["workspace_path"]}]}},
        {"required":["members"],"not":{"anyOf":[{"required":["root"]},{"required":["scope"]},{"required":["config_path"]},{"required":["config"]}]}}
    ]));
}

/// Presence without null for optional report evidence; absence must survive decoding.
pub(crate) fn present<'de, D: Deserializer<'de>, T: Deserialize<'de>>(
    d: D,
) -> Result<Option<T>, D::Error> {
    T::deserialize(d).map(Some)
}

/// Effective selection receipt. It records settings, not the contents of ignore files.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    /// Optional literal project-relative measured directory, without a trailing slash. Absence means no subfolder restriction. Module context remains project-wide; counts and coverage concern only this folder and descendants. In a workspace member, the project folder the member measures; the member's `root` is then the project.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "String", extend("pattern" = r"^[^/]+(?:/[^/]+)*$"))]
    pub subfolder: Option<String>,
    pub hidden: bool,
    pub respect_ignore_files: bool,
    /// Ignore-file mechanisms enabled, not a complete inventory of participating paths.
    pub ignore_files: Vec<String>,
    pub symlinks: String,
    /// Union of effective root-relative exclusions; order does not affect selection.
    pub exclude: Vec<String>,
}

/// Selected files partition into counted, unrecognized, unread and recorded parse failures.
/// Unavailable lines are not zero. Older reports cannot reveal silent notebook failures.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Coverage {
    pub selected_files: u64,
    pub counted_files: u64,
    /// Subset of counted files whose parser came from a language mapping.
    pub mapped_files: u64,
    pub unrecognized_files: u64,
    pub unrecognized_extensions: BTreeMap<String, u64>,
    /// Full report only: root-relative paths, member-prefixed in a workspace.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "Vec<String>")]
    pub unrecognized: Option<Vec<String>>,
    pub unread: Vec<String>,
    /// Always emitted now, including empty. Historical absence means not recorded, never known zero.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "Vec<crate::ParseFailure>")]
    pub parse_failures: Option<Vec<crate::ParseFailure>>,
}

impl From<&crate::Coverage> for MemberCoverage {
    fn from(c: &crate::Coverage) -> Self {
        Self {
            selected_files: c.selected_files as u64,
            counted_files: c.counted_files as u64,
            mapped_files: c.mapped_files as u64,
            unrecognized_files: c.unrecognized_files as u64,
            unread_files: c.unread.len() as u64,
            parse_failed_files: Some(c.parse_failures.len() as u64),
        }
    }
}

impl From<&crate::Coverage> for Coverage {
    fn from(c: &crate::Coverage) -> Self {
        Self {
            selected_files: c.selected_files as u64,
            counted_files: c.counted_files as u64,
            mapped_files: c.mapped_files as u64,
            unrecognized_files: c.unrecognized_files as u64,
            unrecognized_extensions: c
                .unrecognized_extensions
                .iter()
                .map(|(k, v)| (k.clone(), *v as u64))
                .collect(),
            unrecognized: Some(c.unrecognized.clone()),
            unread: c.unread.clone(),
            parse_failures: Some(c.parse_failures.clone()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Kinds {
    pub default: Kind,
    pub data: Vec<String>,
    pub documentation: Vec<String>,
}

/// Descriptor-triggered layout inference, not manifest parsing or exclusive package ownership.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Module {
    /// Root-relative folder; `.` means the root; workspace paths include member names.
    pub path: String,
    pub detector: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Member {
    pub name: String,
    pub root: String,
    pub config_path: Option<String>,
    pub scope: Selection,
    /// Entries of this member's own policy deciding no measured file. Empty for folder members, whose shared project policy reports its findings once in `summary.unused_entries`.
    pub unused_entries: Vec<String>,
    /// Exclusions of this member's own policy leaving out no observed candidate. Empty for folder members, whose shared project policy reports them once in `summary.unused_exclusions`.
    pub unused_exclusions: Vec<String>,
    pub coverage: MemberCoverage,
    /// Full report only: this member's independently loaded policy.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "crate::config::EffectiveConfig")]
    pub config: Option<crate::config::EffectiveConfig>,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MemberCoverage {
    pub selected_files: u64,
    pub counted_files: u64,
    pub mapped_files: u64,
    pub unrecognized_files: u64,
    pub unread_files: u64,
    /// Always emitted now; historical absence means failures were not recorded.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "u64")]
    pub parse_failed_files: Option<u64>,
}

/// Aggregate omissions mean zero; historical missing unclaimed-folder evidence stays unknown. Language file counts are participation counts and may overlap;
/// all other totals and breakdown file counts count physical files once.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ReportSummary {
    #[serde(flatten)]
    pub totals: Summary,
    /// Source code by purpose, over disjoint top-level subtrees plus `.` for root files.
    #[schemars(schema_with = "crate::schema::folder_purpose_map")]
    pub top_folders: BTreeMap<String, BTreeMap<Purpose, u64>>,
    /// Recorded unclaimed folders; historical absence means not recorded, while [] records none.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "Vec<crate::FolderShare>")]
    pub unclaimed_folders: Option<Vec<crate::FolderShare>>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub members: BTreeMap<String, Summary>,
    /// Without subfolder selection: adopted modules. With it: only modules classifying at least one measured source file; code is their measured contribution. Context-only modules remain in metadata.modules.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub build_modules: BTreeMap<String, crate::BuildSystemShare>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub submodules: Vec<crate::SubmoduleShare>,
    /// The five largest source files that build-module layouts classified as production, by code and then path. Nothing else reviews these files, so exclude or reclassify a generated or vendored one. [] records none; absence means an earlier producer did not record the list.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "Vec<crate::ModuleFile>")]
    pub largest_module_files: Option<Vec<crate::ModuleFile>>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Entries deciding no measured file; with subfolder selection, or in a workspace of project folders, this says nothing about use elsewhere. Policy cleanup requires an unrestricted measurement. Absent in a workspace of independent roots, whose members report their own.
    pub unused_entries: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    /// Exclusions leaving out no observed candidate in the measured selection. With subfolder selection, or in a workspace of project folders, matches elsewhere do not count as use; policy cleanup requires an unrestricted measurement. Absent in a workspace of independent roots.
    pub unused_exclusions: Vec<String>,
}

/// Line categories are disjoint; lines = code + comments + blanks, using checked arithmetic.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct LineCounts {
    /// Lines with code.
    pub code: u64,
    /// Comment lines, including documentation embedded in source.
    pub comments: u64,
    pub blanks: u64,
    pub lines: u64,
}

impl From<Counts> for LineCounts {
    fn from(c: Counts) -> Self {
        Self {
            code: c.code,
            comments: c.comments,
            blanks: c.blanks,
            lines: c.lines,
        }
    }
}
impl From<LineCounts> for Counts {
    fn from(c: LineCounts) -> Self {
        Self {
            files: 1,
            code: c.code,
            comments: c.comments,
            blanks: c.blanks,
            lines: c.lines,
        }
    }
}

/// One physical counted file. Embedded contributions inherit host kind and purpose.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct File {
    pub path: String,
    pub language: String,
    pub kind: Kind,
    pub purpose: Purpose,
    pub category: String,
    /// Entry ID, builtin convention/detector ID, or fallback.
    pub rule: String,
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "String")]
    pub language_mapping: Option<String>,
    pub counts: LineCounts,
    /// Only for embedded languages. Counts sum to the file; host participation includes zero counts.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "BTreeMap<String, LineCounts>")]
    pub languages: Option<BTreeMap<String, LineCounts>>,
    /// Only for source files of a purpose other than test with inline Rust `#[cfg(test)]` items: the lines those items span, which `counts` includes.
    #[serde(
        default,
        skip_serializing_if = "Option::is_none",
        deserialize_with = "present"
    )]
    #[schemars(with = "LineCounts")]
    pub inline_tests: Option<LineCounts>,
}
impl From<&crate::FileRecord> for File {
    fn from(f: &crate::FileRecord) -> Self {
        Self {
            path: f.path.clone(),
            language: f.language.clone(),
            kind: f.kind,
            purpose: f.classification.purpose,
            category: f.classification.category.clone(),
            rule: f.classification.rule.clone(),
            language_mapping: f.language_mapping.clone(),
            counts: f.counts.into(),
            languages: (f.languages.len() > 1).then(|| {
                f.languages
                    .iter()
                    .map(|(k, v)| (k.clone(), (*v).into()))
                    .collect()
            }),
            inline_tests: (f.inline_tests.lines > 0).then(|| f.inline_tests.into()),
        }
    }
}

impl MeasurementReport {
    /// Compact projection; coverage failures remain visible while unavailable detail is omitted.
    /// It borrows the report, so the omitted detail, such as per-file records, is never copied.
    pub fn headline(&self) -> Self {
        Self {
            document_type: self.document_type,
            detail: Detail::Headline,
            schema_version: self.schema_version.clone(),
            project: self.project.clone(),
            metadata: self.metadata.clone().headline(),
            summary: self.summary.clone(),
            review: None,
            breakdown: None,
            files: None,
        }
    }
}

impl Metadata {
    /// Shared compact metadata projection for measurement and comparison headlines.
    pub fn headline(mut self) -> Self {
        self.config = None;
        self.modules = None;
        self.kinds = None;
        self.coverage.unrecognized = None;
        for member in self.members.iter_mut().flatten() {
            member.config = None;
        }
        self
    }
}

pub(crate) fn headline_example() -> serde_json::Value {
    serde_json::json!({
        "document_type":"measurement", "detail":"headline",
        "schema_version":"3", "project":"empty-example",
        "metadata":{
            "root":"/example", "scope":{"hidden":true,"respect_ignore_files":true,"ignore_files":[".ignore"],"symlinks":"not followed","exclude":[]},
            "config_path":null, "user_config_path":null,
            "measured_at_unix_seconds":0, "locmetry_version":env!("CARGO_PKG_VERSION"), "backend":crate::BACKEND,
            "backend_configuration":crate::BACKEND_CONFIGURATION, "backend_inaccurate":false,
            "coverage":{"selected_files":0,"counted_files":0,"mapped_files":0,"unrecognized_files":0,"unrecognized_extensions":{},"unread":[],"parse_failures":[]}
        },
        "summary":{"all_counted":{"files":0,"code":0,"comments":0,"blanks":0,"lines":0},"top_folders":{},"unclaimed_folders":[]}
    })
}
