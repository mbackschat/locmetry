//! Net change compares recorded measurement evidence without reading source checkouts.
use crate::{
    contract::{Detail, MeasurementReport, Metadata},
    Counts, Kind, Purpose,
};
use anyhow::{Context, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

mod engine;
mod render;
pub use engine::{compare, read_input, validate_destination};
pub use render::{render_markdown, render_table_with_theme, render_tables};

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub enum DocumentType {
    #[serde(rename = "comparison")]
    Comparison,
}
pub use crate::contract::Version;

/// Signed net change, after minus before. Each value must fit i64; overflow fails.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SignedCounts {
    pub files: i64,
    pub code: i64,
    pub comments: i64,
    pub blanks: i64,
    /// Equals code + comments + blanks; net change is not Git additions/deletions or effort.
    pub lines: i64,
}

/// Recorded before/after counts and their checked after-minus-before difference.
/// Coverage findings qualify these measured subsets; missing lines are never estimated.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct CountChange {
    pub before: Counts,
    pub after: Counts,
    pub delta: SignedCounts,
}
impl CountChange {
    fn new(before: Counts, after: Counts) -> Result<Self> {
        let delta = |a: u64, b: u64| {
            i64::try_from(i128::from(b) - i128::from(a))
                .context("Net change exceeds signed 64-bit range")
        };
        Ok(Self {
            before,
            after,
            delta: SignedCounts {
                files: delta(before.files, after.files)?,
                code: delta(before.code, after.code)?,
                comments: delta(before.comments, after.comments)?,
                blanks: delta(before.blanks, after.blanks)?,
                lines: delta(before.lines, after.lines)?,
            },
        })
    }
}

/// Counts over the complete union of dimension keys; an absent aggregate bucket contributes zero.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Dimensions {
    pub all_counted: CountChange,
    pub source: CountChange,
    pub by_kind_purpose: BTreeMap<String, CountChange>,
    #[schemars(schema_with = "crate::schema::purpose_map::<CountChange>")]
    pub source_purpose: BTreeMap<Purpose, CountChange>,
    pub source_categories: BTreeMap<String, CountChange>,
    /// Lines add across languages, but file counts are overlapping language participation.
    pub source_languages: BTreeMap<String, CountChange>,
}

/// Workspace net change = matched + entering - leaving, for each count field.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Contribution {
    pub matched: SignedCounts,
    /// Positive after counts of roots entering the workspace measurement.
    pub entering: Counts,
    /// Positive before counts of roots leaving the workspace measurement; subtract these.
    pub leaving: Counts,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Composition {
    pub all_counted: Contribution,
    pub source: Contribution,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Summary {
    #[serde(flatten)]
    pub dimensions: Dimensions,
    /// Workspace only: distinguish retained-root changes from entering/leaving measurements.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub composition: Option<Composition>,
}

/// Presence in saved reports, not proof of physical creation/deletion or repository identity.
#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Presence {
    Both,
    /// No after evidence; a removed root was removed from the workspace measurement.
    BeforeOnly,
    /// No before evidence; counts enter the measurement, without asserting file creation.
    AfterOnly,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct MemberChange {
    pub presence: Presence,
    #[serde(flatten)]
    pub dimensions: Dimensions,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FolderCounts {
    pub all_counted: CountChange,
    pub source: CountChange,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FolderChange {
    pub presence: Presence,
    /// Files directly in this folder; disjoint across folders.
    pub direct: FolderCounts,
    /// Folder including descendants; parents and children must not be summed together.
    pub recursive: FolderCounts,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PackageChange {
    /// Key into folders. Its recursive counts include descendants/tests and are not exclusive package totals.
    pub folder: String,
    pub detector: String,
    /// Detection presence is not an additional LOC contribution.
    pub presence: Presence,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PresenceCounts {
    pub before_only: u64,
    pub after_only: u64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct PresenceSummary {
    pub members: PresenceCounts,
    pub folders: PresenceCounts,
    pub packages: PresenceCounts,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BreakdownChange {
    pub folder: String,
    pub kind: Kind,
    pub purpose: Purpose,
    pub category: String,
    pub language: String,
    /// Additive in every count column, including files=0 for embedded-language contributions.
    pub counts: CountChange,
}

#[derive(
    Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum Side {
    Before,
    After,
    Both,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct FindingContext {
    pub side: Side,
    pub message: String,
    /// Report-level when absent; member name when the finding concerns a workspace root.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<String>,
    /// JSON pointers identifying affected fields in the applicable input report(s).
    pub paths: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum Entity {
    Member { name: String },
    Folder { path: String },
    Package { path: String, detector: String },
}
impl Entity {
    /// The entity as a sentence names it, e.g. `Folder 'src/extra'`; quotes keep
    /// a path distinct from the words around it.
    pub(crate) fn named(&self) -> String {
        match self {
            Entity::Member { name } => format!("Member {}", crate::output::quoted(name)),
            Entity::Folder { path } => format!("Folder {}", crate::output::quoted(path)),
            Entity::Package { path, detector } => {
                format!("Package {} ({detector})", crate::output::quoted(path))
            }
        }
    }
}

/// Stable finding codes; consumers branch on code, not explanatory message text.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "code", rename_all = "snake_case", deny_unknown_fields)]
pub enum Finding {
    /// Recorded producer/counter versions or backend settings differ.
    CounterChanged {
        #[serde(flatten)]
        context: FindingContext,
    },
    /// Recorded policy differs; this does not prove it caused the net change.
    PolicyChanged {
        #[serde(flatten)]
        context: FindingContext,
    },
    /// Adopted build-module sets differ.
    ModulesChanged {
        #[serde(flatten)]
        context: FindingContext,
    },
    /// Project names differ; matching does not verify repository identity.
    IdentityChanged {
        #[serde(flatten)]
        context: FindingContext,
    },
    /// Unavailable counts or historical failure evidence; measured-subset differences are not estimates.
    CoverageGap {
        #[serde(flatten)]
        context: FindingContext,
        /// Direct folder containing unavailable paths, when known; recursive parents are affected too.
        #[serde(skip_serializing_if = "Option::is_none")]
        folder: Option<String>,
    },
    /// Report presence changed, including zero-count entities. Full output only; headline retains counts.
    PresenceChanged {
        #[serde(flatten)]
        context: FindingContext,
        entity: Entity,
        presence: Presence,
    },
}
impl Finding {
    fn context(&self) -> &FindingContext {
        match self {
            Self::CounterChanged { context }
            | Self::PolicyChanged { context }
            | Self::ModulesChanged { context }
            | Self::IdentityChanged { context }
            | Self::CoverageGap { context, .. }
            | Self::PresenceChanged { context, .. } => context,
        }
    }
    fn code(&self) -> &'static str {
        match self {
            Self::CounterChanged { .. } => "counter_changed",
            Self::PolicyChanged { .. } => "policy_changed",
            Self::ModulesChanged { .. } => "modules_changed",
            Self::IdentityChanged { .. } => "identity_changed",
            Self::CoverageGap { .. } => "coverage_gap",
            Self::PresenceChanged { .. } => "presence_changed",
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Input {
    /// Caller-supplied filename; not verified Git provenance.
    pub filename: String,
    pub project: String,
    /// Full measurement metadata in full comparison; shared compact projection in headline.
    pub metadata: Metadata,
}

/// Net change is after - before over recorded measurements, without rescanning or Git.
/// --format json selects full output; --format headline selects compact JSON.
/// Full output has folders, packages, breakdown and all findings; headlines omit those arrays/maps
/// and individual presence findings, retaining presence_summary and member rows.
/// Exit 2 means an input has observed failures/inaccuracy; historical missing evidence alone does not.
/// Input dynamic-map duplicate keys retain the last value; this contract does not detect overwritten keys.
/// No recorded settings differences is not proof of source-only growth: reports lack source/ignore/build fingerprints.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(example = comparison_example())]
pub struct Comparison {
    pub document_type: DocumentType,
    /// The projection, named as in measurements. Optional only because comparisons written
    /// by 0.8.0 lack it and schema 3 grows by optional fields (docs/report-compatibility.md).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub detail: Option<Detail>,
    pub schema_version: Version,
    pub before: Input,
    pub after: Input,
    /// Selected workspace member; absent means the complete comparison. Counts and detail
    /// are scoped; input metadata, report-level findings and partial status retain whole-input evidence.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member: Option<String>,
    pub summary: Summary,
    /// Workspace only, keyed by member name, including zero-count arrivals/departures.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<BTreeMap<String, MemberChange>>,
    /// Root-relative keys; workspace keys retain the member prefix, including the root as the member name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub folders: Option<BTreeMap<String, FolderChange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub packages: Option<Vec<PackageChange>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub breakdown: Option<Vec<BreakdownChange>>,
    /// Fixed entity/direction counts, always present including zeros; these are not LOC counts.
    pub presence_summary: PresenceSummary,
    pub findings: Vec<Finding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    /// Historical headline field through v0.3.0: destination of a separately saved comparison.
    /// Retained when reading historical documents; new CLI output omits it.
    pub report: Option<String>,
}
impl Comparison {
    pub fn headline(mut self) -> Self {
        self.detail = Some(Detail::Headline);
        self.report = None;
        self.folders = None;
        self.packages = None;
        self.breakdown = None;
        self.findings
            .retain(|f| !matches!(f, Finding::PresenceChanged { .. }));
        self.before.metadata = self.before.metadata.headline();
        self.after.metadata = self.after.metadata.headline();
        self
    }
    pub fn partial(&self) -> bool {
        [&self.before.metadata, &self.after.metadata]
            .into_iter()
            .any(|m| {
                m.backend_inaccurate
                    || !m.coverage.unread.is_empty()
                    || m.coverage
                        .parse_failures
                        .as_ref()
                        .is_some_and(|f| !f.is_empty())
            })
    }
}
pub fn schema() -> serde_json::Value {
    crate::schema::json::<Comparison>()
}

fn comparison_example() -> Comparison {
    let report: MeasurementReport = serde_json::from_value(crate::contract::headline_example())
        .expect("validated measurement example");
    let zero = crate::Summary::default();
    Comparison {
        document_type: DocumentType::Comparison,
        detail: Some(Detail::Headline),
        schema_version: Version::V3,
        before: Input {
            filename: "before.json".into(),
            project: report.project.clone(),
            metadata: report.metadata.clone(),
        },
        after: Input {
            filename: "after.json".into(),
            project: report.project,
            metadata: report.metadata,
        },
        summary: Summary {
            dimensions: engine::dimensions(&zero, &zero).expect("zero differences fit"),
            composition: None,
        },
        member: None,
        members: None,
        folders: None,
        packages: None,
        breakdown: None,
        presence_summary: PresenceSummary::default(),
        findings: Vec::new(),
        report: None,
    }
}
