# Net Change Between Measurement Reports

<!-- doc-status
{
  "status": "implemented",
  "date": "2026-10-15",
  "summary": "Aggregate and topology comparison; per-file comparison remains deferred.",
  "evidence": "[Completion evidence](../../plans/archive/measurement-quality-and-net-change.md#completion-evidence-and-ongoing-review)"
}
-->

Status: historical decision record — implemented. Aggregate and topology net change was implemented on 2026-10-15 as `0.2.0-dev.1` and released in v0.2.0. Archived on 2026-10-16 under the [retirement procedure](../README.md#retirement-procedure). [PRODUCT](../../PRODUCT.md#net-change) owns the supported contract; the [work plan](../../plans/archive/measurement-quality-and-net-change.md#completion-evidence-and-ongoing-review) records verification.

The sections below preserve the reviewed design and acceptance criteria. Their imperative wording, examples and stated limitations are historical evidence, not current guarantees. Do not update this body to track behavior. The deferred per-file design belongs to the [conditional follow-up](../../plans/measurement-quality-and-net-change.md#per-file-comparison); this record carries no live scope.

## Purpose and Scope

Compare two saved Locmetry measurements and explain how their recorded counts differ. A net change is always `after − before`. It cannot establish Git line additions/deletions, editing effort, or whether a changed count was caused by source edits rather than measurement policy.

| Original scope | S4 design record |
| --- | --- |
| Required | Compare full JSON reports for two single roots or two workspaces; counts by purpose, kind, category, language, and folder; workspace member totals and membership changes; package detection changes; JSON, terminal tables, and Markdown; a discoverable, described comparison schema; explicit measurement and coverage findings. |
| Deferred at S4 | Per-file comparison was not implemented. Its design and activation trigger now live in the plan's [conditional follow-up](../../plans/measurement-quality-and-net-change.md#per-file-comparison). |
| Excluded | Git invocation, checkouts, source-content diffs, churn, automatic commit identification, rename detection, matching a single-root report to a workspace report, arbitrary member remapping, history storage, charts, and a generic schema/plugin framework. |

The comparison reads only the two report files. It neither rescans their roots nor reloads current configuration or ignore files. Source checkouts need not exist. Reports can be named after Git revisions by the caller; their filenames do not constitute verified Git provenance.

## Command and Output

```sh
locmetry compare before.json after.json
locmetry compare before.json after.json --format markdown
locmetry compare before.json after.json --output net-change.json
locmetry --schema comparison
```

Arguments determine direction regardless of timestamps. The terminal defaults to a table; redirected stdout defaults to JSON. `--format table|json|markdown` and `--output FILE|-` follow the existing measurement command's conventions: JSON stdout is a compact headline, `--output FILE` writes the full comparison, and `--output -` prints the full comparison and cannot be combined with a human format. The comparison output must not overwrite either input, including aliases through symlinks or hard links. No scan-root destination restriction is needed because comparison does not scan.

All formats say **net change**. Human tables show before, after, and signed change columns. The terminal shows source totals by purpose and language, then top-level folders or workspace members; Markdown also shows the folder hierarchy. Findings and membership changes appear even when their code delta is zero. Unchanged detail rows are hidden in human output; JSON retains the complete union of dimension keys. No percentage changes or ratio deltas are required initially.

Exit code `0` means a comparison without recorded measurement failures or backend inaccuracy; recorded policy differences do not cause failure. Exit code `2` means a comparison was produced from an input marked inaccurate or containing unreadable files or the decode/counting failures introduced by S1. Exit code `1` means no comparison was produced because an input, mode, version, arithmetic operation, or destination was invalid. Unrecognized files and missing historical failure evidence generate coverage findings but do not alone cause exit code `2`; missing evidence is not an observed failure or proof of completeness. Diagnostics go to stderr. Every output exposes findings or their explicit presence summary; the full comparison retains all individual findings.

## Accepted Inputs and Identity

Accept full measurement reports with supported report schema version `"1"`, including existing reports without file records. Reject headlines, comparison documents, unsupported versions, malformed counts, duplicate identities in array rows, and conflicting summary/detail totals. Duplicate raw JSON keys in dynamic maps are outside this rejection guarantee: the existing deserializer retains the last value before schema/semantic validation. Document and test that boundary; a duplicate-aware reader is not part of this proposal. For version 1, full-report evidence is the presence of `review`, `breakdown`, `metadata.kinds`, `metadata.modules`, `metadata.coverage.unrecognized`, and the effective `config` at the root or in every workspace member; empty arrays and objects remain valid where the report permits them. Do not add a new mandatory discriminator to existing measurements. Comparisons have their own discriminator and independent schema version.

The existing report schema accepts both headlines and full reports, so satisfying it alone does not establish comparison eligibility. Older exports missing required evidence must fail with the missing fields named; neither a matching schema-version string nor an empty replacement section can recover that evidence. Existing full exports satisfying these requirements remain accepted regardless of their recorded Locmetry version.

Version-1 shapes evolved across releases. Source inspection establishes that `v0.1.0` and `v0.1.1` exports lack `metadata.modules`, introduced in `v0.1.2`; those earlier reports are deliberately ineligible for comparison, not retroactively invalid measurement reports. The [S3 compatibility inventory](../../report-compatibility.md) records field introductions across `v0.1.0`–`v0.1.6` and S1, with synthetic reports generated by each tagged binary and separate validity/eligibility tests. A version string alone does not establish either full-report detail or failure-observation capability.

Missing aggregate keys mean zero only where the measurement contract explicitly permits omission. Missing required report sections, missing file-detail capability, and unrecognized/unreadable files do not mean zero. Input validation must preserve those distinctions. Validate counts, line equations, coverage accounting, and the totals recoverable from the available records; do not invent detail absent from an aggregate-only report.

S1's `metadata.coverage.parse_failures` collection is always emitted by new measurements, including when empty, and optional in the version-1 schema to retain historical inputs. Preserve absent evidence separately from a recorded empty collection during deserialization and comparison. Eligible older reports without that field produce `coverage_gap` findings on the affected side/member; do not manufacture an empty list or infer correctness from their successful exit or `backend_inaccurate = false`. S1 records `{path, reason, message}` and member `parse_failed_files`; S3 preserves this shape in the shared typed contract. Older strict schemas may reject the added field; backward readability does not imply forward compatibility with those schemas.

| Entity | Identity and presence evidence |
| --- | --- |
| Single root | The caller pairs the reports. Absolute root paths may differ. Preserve both project names and paths; differing names generate an identity finding rather than automatic rejection. |
| Workspace root | Match by the existing member `name`. Use the union of `metadata.members`, including members with no counted files. A name is a matching key, not proof of repository identity. |
| Folder | Member name, where applicable, plus the root-relative folder path. Use folders and ancestors evidenced by breakdown rows, coverage paths, or recorded packages; include `.` for each root. |
| Package | A build module already recorded in `metadata.modules`, identified by member, path, and detector. Multiple detectors at one path remain distinct. Ordinary package-like directories still appear as folders when no detector recognizes them. |
| File, if enabled | Member name, where applicable, plus root-relative path. Match by path only. |

Absolute checkout locations, report filenames, timestamps, and configuration file locations are provenance, not entity matching keys. Workspace members are not paired by position. A renamed member, folder, or package path appears as one departure and one arrival. Workspaces with disjoint member sets are valid comparisons.

## Packages, Folders, and Roots Entering or Leaving

Every entity row carries `presence`, one of `both`, `before_only`, or `after_only`. These describe appearance in the reports, not proven creation or deletion on disk. A root removed from a workspace is labelled **removed from the workspace measurement**. Files or folders missing after an exclusion change are not labelled deleted source.

Workspace totals include entering and leaving roots. For each count, the result must reconcile as:

```text
total net change
  = net change within matched roots
  + after counts of entering roots
  - before counts of leaving roots
```

Show those three contributions separately in the workspace summary. A missing member contributes zero to that side's workspace aggregate because it was not included; its `presence` still records why. A retained root with unreadable files is a different case: its recorded counts are partial, not evidence of removal. Root membership changes must be shown even for zero-count roots. Current workspace measurements require at least one root; comparing entirely disjoint nonempty workspaces is supported, but producing a measurement with no workspace roots is outside this change.

For example, these synthetic source-code counts give a total net change of `+10`, comprising `+20` within matched roots, `+30` from an entering root, and `−40` from a leaving root:

| Member | Presence | Before | After | Net change |
| --- | --- | ---: | ---: | ---: |
| `api` | `both` | 100 | 120 | +20 |
| `legacy` | `before_only` | 40 | 0 | −40 |
| `worker` | `after_only` | 0 | 30 | +30 |
| Total | | 140 | 150 | +10 |

Folders expose both direct counts and counts including descendants, with separate changes for `all_counted` and `source`. Parent totals and child totals must not be summed together. A folder entering or leaving the report remains visible even with zero measured source code, such as a documentation-only folder. Coverage-only folders carry the finding that their contents have missing counts.

Package rows report detection presence and refer to the corresponding folder's counts. Those counts include descendants and tests; they are not an exclusive package total or the amount a detector classified as production. Removing a counted build descriptor removes its own data counts and can change the purpose of retained source even when source LOC remains unchanged. Disabling module detection or changing detectors is surfaced as a configuration/counting-context difference. Package additions/removals therefore do not independently add or subtract code from the aggregate.

The descriptor-removal guard uses a three-line `package.json` and two nonblank JavaScript statements in `src/index.js`, without classification overrides. After removing only the descriptor, `all_counted` code/lines change from 5 to 2 and files from 2 to 1, while source code/lines remain 2 and source files remain 1. Production source code changes from 2 to 0 and unclassified source code from 0 to 2. The npm package is `before_only`; its root folder and source subfolder remain `both`. Its presence change is not an extra subtraction.

Empty directories and directories containing only excluded files leave no evidence in current reports. Their physical creation/deletion cannot be detected by this feature. Supporting that would require an additional directory inventory in measurement reports and is outside this proposal. An empty workspace member is still detectable through its member metadata.

## Count and JSON Contract

Use a separate comparison document with `document_type: "comparison"` and `schema_version: "1"`. Preserve existing dimension names and their meanings. Do not repurpose a measurement report to hold negative counts.

The shared numeric contract is illustrated completely below; these are public data shapes, not a proposed internal aggregation framework:

```rust
struct Counts {
    files: u64,
    code: u64,
    comments: u64,
    blanks: u64,
    lines: u64,
}

struct SignedCounts {
    files: i64,
    code: i64,
    comments: i64,
    blanks: i64,
    lines: i64,
}

struct CountChange {
    before: Counts,
    after: Counts,
    delta: SignedCounts,
}
```

Counts remain exact JSON integers. Arithmetic must be checked: if a sum exceeds `u64`, or a resulting delta cannot be represented by `i64`, fail with an explanatory error rather than wrapping, clamping, or converting to floating point. `lines = code + comments + blanks` holds for before, after, and delta.

| Comparison field | Contents |
| --- | --- |
| `document_type`, `schema_version` | Constants `"comparison"` and `"1"`. Measurement and comparison versions are independent. |
| `before`, `after` | Input filename, project name, and the complete measurement metadata, reusing the typed measurement metadata contract. These retain member identities, times, versions, effective configuration, and coverage evidence. |
| `summary` | `CountChange` values for `all_counted` and `source`, and maps of them for `by_kind_purpose`, `source_purpose`, `source_categories`, and `source_languages`. For workspaces, also the three member-set contributions defined above, for both `all_counted` and `source`. |
| `members` | Workspace only: rows keyed by member name with `presence` and the same summary dimensions. Single-root comparisons omit this field. |
| `folders` | Rows keyed by member/path with `presence`, direct and recursive `all_counted` and `source` changes. |
| `packages` | Rows with explicit `folder` (member-prefixed where applicable), `detector` and `presence`; that tuple is the identity. Folder counts are referenced by identity, not duplicated as an additional additive total. |
| `presence_summary` | Fixed keys `members`, `folders`, and `packages`, each holding `before_only: u64` and `after_only: u64` entity counts. Always present in headline and full output, including zeros; single-root member counts are zero. These are presence counts, not LOC contributions. |
| `breakdown` | Union of the measurement breakdown keys `(folder, kind, purpose, category, language)` with `CountChange` values; workspace folder paths retain their member prefix. |
| `findings` | Structured findings with a stable code, human-readable message, before/after applicability, optional member, and affected metadata/report paths. Agents branch on codes rather than parsing prose. |
| `files` | Proposed extension, not part of the implemented S4 output; design moved to the plan's [per-file follow-up](../../plans/measurement-quality-and-net-change.md#per-file-comparison). |
| `report` | Headline only when `--output FILE` was supplied: the destination of the full comparison. |

The recommended headline projection omits `folders`, `packages`, `breakdown`, `files`, and individual `presence_changed` findings, and projects input metadata to the same compact metadata view used by measurement headlines. It retains summary, member changes, other findings, and `presence_summary`, so zero-code arrivals/departures stay visible without one headline finding per changed folder/package. Only this topology-summary portion is fixed-size; member rows and coverage findings still scale with their evidence. The full comparison retains all rows, including unchanged ones, and every individual finding. S4 adopts this projection and defines its schema with the typed result. Maps and row arrays have deterministic identity ordering, and findings sort by code, member, side, and path. Human folder views retain the hierarchy.

The finding codes are `counter_changed`, `policy_changed`, `modules_changed`, `identity_changed`, `coverage_gap`, and `presence_changed`. Each finding has `side: "before" | "after" | "both"`, `message: string`, `paths: string[]` identifying fields in that side's report, and optional `member: string`. Coverage findings optionally identify a direct `folder` with unavailable counts, whose recursive parents are affected too. Presence findings additionally identify the entity kind (`member`, `folder`, or `package`), its identity, and its presence state. These discriminated variants belong in the schema, including their descriptions; a missing optional member means the finding applies at report level, not that a member's identity is unknown.

Line counts in `summary.source_languages` are additive, but its file counts count language participation: a file embedding another source language can appear under more than one language. In contrast, breakdown rows assign zero files to embedded-language contributions and are additive across all columns. Compare each representation using its documented meaning; never sum language participation counts to reconstruct physical file totals. This distinction is already present in [the measurement model](../../../src/report.rs).

## Comparability and Missing Evidence

Emit findings for differing recorded Locmetry/backend versions, backend configuration, content-kind mappings, effective configuration or scope, adopted module sets, member sets, project names, and coverage limitations. Match policy evidence per retained workspace member. Treat exclusion lists as sets but retain the order of classification rules and language mappings, where first match matters. Ignore relocation and timestamps when comparing policy. Differences in descriptive configuration text may be reported conservatively; a difference is not a claim that it caused the count change.

Unrecognized, unreadable or failed-decoding/counting paths on either side always produce coverage findings, even if the lists match. Their line counts remain unknown. Missing historical failure evidence also produces a finding, even when comparing a report to itself. Aggregate deltas compare the recorded measured subsets and must be labelled accordingly when coverage is incomplete or failure observation was not recorded. They must never be presented as an estimate of the missing lines.

No finding does not prove equivalent measurement conditions. Current reports do not retain ignore-file contents, source-content hashes, Git revisions, or an executable fingerprint. Identical version strings can identify different development builds. Describe the result as **no recorded differences in measurement settings**, never as proven source-only growth. These evidence limits belong in schema descriptions and human output; adding fingerprints or Git metadata is not required for this feature.

## One Contract Mechanism

S3 now generates configuration and measurement schemas through shared Draft 2020-12 settings. [Measurement contract types](../../../src/contract.rs) own report serialization, reading and schema descriptions; effective policy reuses the configuration parser types. Comparison extends this mechanism rather than introducing another independently maintained schema. [Compatibility](../../report-compatibility.md) records the shipped refactor and its limits.

Use typed public contracts for configuration, measurement reports, and comparisons, with one schema-generation path based on the existing `schemars` mechanism. Serialization, deserialization, and structural schemas use those same contracts. Public types describe the serialized representation; internal aggregation structures can remain separate. Share definitions only when both shape and meaning agree, including line counts, aggregate counts, classification enums, metadata, and the effective configuration with its recorded built-in sets. The TOML-specific rejection of `null` must not be applied to JSON report schemas.

The measurement contract refactor preserves valid version-1 output: field names, enum spellings, optional/null distinctions, omitted zero totals, per-file count shape, embedded-language representation, headline/full projection, and mutually exclusive root/member metadata. The former handwritten schema admitted negative counts and lacked some semantic constraints; preserving valid output does not require accepting malformed reports it happened to admit. Narrower validation must be documented explicitly. The accepted-input requirements above determine which existing exports contain enough evidence for comparison.

`locmetry --schema comparison` emits a self-contained JSON Schema Draft 2020-12 with local definitions, without requiring network resolution or reading a second schema. Every schema is generated through the common mechanism, with field and variant descriptions owned by the public types and dynamic built-in lists taken from their executable definitions. The comparison schema covers headline/full forms and explains how to distinguish them. Descriptions state net-change direction, count ranges, defaults and omissions, matching rules, presence states, additive/non-additive dimensions, comparability limits, findings, and exit semantics, with a valid example.

JSON Schema validates structure and numeric bounds. Shared semantic validation enforces arithmetic, unique array-row identities, compatible modes, full-report requirements, and reconciliation. It is tested in S3 and exposed to users through `compare` input errors in S4; no public `validate` command is introduced. Generated schemas do not replace those checks or detect raw duplicate map keys already collapsed by deserialization. One comparison result supplies all three renderers; renderers do not independently match entities, subtract counts, or decide findings. One headline projection supplies both serialization and its structural contract.

## Optional File Detail

Deferred at S4. The design is retained in the work plan's [per-file comparison follow-up](../../plans/measurement-quality-and-net-change.md#per-file-comparison), with a consumer-demand trigger. This heading remains for historical links; it defines no active scope.

## Acceptance Criteria and Decisions

- Identity: comparing a report to itself produces zero deltas and `both` presence; existing coverage limitations still appear as findings. Reversing inputs reverses every delta and presence state when both directions fit the signed numeric range. Input row ordering does not affect results.
- Reconciliation: both snapshots and their deltas satisfy line and aggregate equations, including workspace member arrivals/departures, nested folders, and embedded-language file-count semantics. Reclassification can transfer counts between buckets while total LOC remains unchanged.
- Topology: new/removed packages and folders, empty members, disjoint workspace sets, zero-code membership changes, and descriptor removal with retained source are all represented without inferring physical deletion. Partial or unrecognized coverage is never silently converted into known zero LOC.
- Contracts: eligible historical full reports remain consumable; per-release fixtures distinguish valid measurement reports from comparison-eligible ones. Missing failure evidence is distinct from recorded empty evidence. Actual measurement and comparison outputs validate against generated schemas. Invalid negative counts, overflow, unsupported versions, headlines supplied as full inputs, duplicate array rows, and inconsistent aggregates fail clearly; dynamic-map duplicate-key behavior is tested and documented separately.
- Agent use: schema-only examples explain direction, presence, missing evidence, file-count overlap, and findings; all example documents validate. JSON, text, and Markdown agree on totals and findings, and headline presence summaries reconcile with all full-report presence rows, including large zero-net restructures.

Decisions adopted for S4: require aggregate and topology comparison first; keep per-file detail optional; use existing build modules as the package definition; produce comparisons with explicit findings for changed measurement conditions; include the focused measurement-contract refactor so all schemas use one mechanism. Physical empty-folder tracking, verified Git identity, and full measurement reproducibility require evidence absent from existing reports and remain excluded.
