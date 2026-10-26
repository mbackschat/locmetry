# Conditional Measurement and Comparison Follow-ups

<!-- doc-status
{
  "status": "historical",
  "date": "2026-10-26",
  "summary": "Retired list of conditional measurement and comparison capabilities; live candidates moved to DEFERRED, decisions to REJECTED.",
  "evidence": "[DEFERRED](../../DEFERRED.md#measurement-and-comparison), [REJECTED](../../REJECTED.md)"
}
-->

Status: historical record, retired 2026-10-26. Its live candidates and their triggers moved to [DEFERRED](../../DEFERRED.md#measurement-and-comparison), and per-file comparison and the retained backend moved to [REJECTED](../../REJECTED.md); the prior-art references and the retained per-file design below stay here. The text below describes the record at retirement, not current work.

The [archived implementation record](measurement-quality-and-net-change.md) preserves completed stages, reviews and source provenance. The [ratio presentation design](test-production-ratios.md) is implemented. The retained sections below preserve the conditional designs and activation conditions; resolved entries are identified explicitly.

## Conditional Follow-ups and Preserved Rationales

These are recorded for later assessment, not added to the accepted implementation sequence. Promote an item only with its trigger, scope, acceptance criteria and documentation impact recorded here or in a linked plan.

Subfolder measurement was approved and implemented on 2026-10-16 after separate design review; [completion evidence](measurement-quality-and-net-change.md#subfolder-implementation--2026-10-16) records the additional checkpoint. [PRODUCT](../../PRODUCT.md#subfolder-measurement) owns supported behavior and the [historical proposal](../../proposals/archive/subtree-measurement.md) preserves rationale. Ancestor discovery and the other candidates below remain outside accepted scope.

| Item | Recommendation and rationale | Trigger, evidence and next assessment |
| --- | --- | --- |
| Per-file comparison | Keep aggregate/topology comparison usable without file records; retain the [deferred design](#per-file-comparison) separately from the historical S4 record. | A consumer needs per-file deltas. Confirm the use case and approve the bounded design and acceptance checks below before implementation. |
| Generated/minified evidence | Report suspicion with reasons before considering automatic exclusion. Generated files can be maintained or intentionally included; markers and line-length thresholds are heuristics. | A real scope-debugging need. Define inspected bytes/lines, thresholds, false positives and policy provenance. Learn from [pygount reasons](https://github.com/roskakori/pygount/blob/v3.2.0/pygount/analysis.py#L375) and [scc heuristics](https://github.com/boyter/scc/blob/v4.1.0/processor/workers.go). |
| Duplicate evidence | Preserve physical totals; optionally identify equal-content groups. Equal bytes can belong to different purposes or members. A unique-content metric needs a separate definition and deterministic representative choice. | A consumer needs duplication evidence. Compare [cloc default deduplication](https://github.com/AlDanial/cloc/blob/v2.10/cloc#L1450), [pygount handling](https://github.com/roskakori/pygount/blob/v3.2.0/pygount/analysis.py#L385) and scc's opt-in exclusion before choosing semantics. |
| Binary/encoding policy | Establish current-source fixtures before changing defaults. A NUL heuristic can reject text encodings; unknown language is not known source code. | Reproduce the weaker existing-binary observation and audit decoding/container boundaries. Keep exclusions and unavailable states explicit. [Qualified finding](../../research/tool-lessons-and-gaps.md#evidence-can-improve-scope-without-silently-changing-totals). |
| `.gitmodules` syntax | Resolved 2026-10-16: named sections, quoted/escaped paths, comments and continuations have independent fixtures and a Git-parser cross-check. | [Supported boundary](../../measurement.md#git-submodule-hints); [historical finding](../../research/tool-lessons-and-gaps.md#shared-contracts-and-accurate-examples-come-next). Further syntax support needs a reproducing input. |
| Language/help discovery | No new feature scheduled: the configuration schema already enumerates accepted language names and aliases. The prior name-list recommendation was redundant. | Verified against executable schema on 2026-10-16 following external review. A “did you mean” error hint is a possible small diagnostic fix, not a new discovery command. |
| Git history analytics | Defer: hotspots, coupling and authorship introduce history-window and identity semantics beyond saved measurements. | A concrete history workflow that existing tools cannot serve adequately. [scc history](https://github.com/boyter/scc/blob/v4.1.0/processor/history.go). |
| Complexity, ULOC/DRYness and cost estimates | Defer: heuristic/formula metrics require independent definitions and validation, and do not establish effort or quality. | A named decision requiring a metric and an agreed validation corpus. [scc metrics](https://github.com/boyter/scc/blob/v4.1.0/processor/workers.go), [LOCOMO](https://github.com/boyter/scc/blob/v4.1.0/processor/locomo.go). |
| MCP | Defer while CLI/JSON/schema meet consumers' needs. Any new interface must share policy and result semantics; scc's MCP bypasses CLI config discovery in the inspected source. | A client cannot adequately use the existing boundary. [scc entry](https://github.com/boyter/scc/blob/v4.1.0/main.go), [MCP handlers](https://github.com/boyter/scc/blob/v4.1.0/mcp.go). |
| HTML, CSV, SQL and OpenMetrics | Consumer-driven: each format needs a user, compatibility expectations and evidence that existing output is insufficient. | A stakeholder-sharing or named analytics/monitoring integration. [scc formats](https://github.com/boyter/scc/blob/v4.1.0/processor/formatters.go). |
| Streaming and resource limits | Measurement-driven: streaming changes ordering/detail tradeoffs; silent truncation or size exclusion would change scope. | Reproducible memory, latency or oversized-file problems with a representative corpus. [Capability comparison](../../research/tool-capability-comparison.md#selection-and-grouping-determine-which-totals-matter). |
| Archive/remote input, custom lexers, more platforms | Defer until an unsupported input or deployment requirement justifies parsing, authority and maintenance costs. | A representative use case and a feasible test/release matrix. [Input/deployment comparison](../../research/tool-capability-comparison.md). |
| Backend replacement | Retain Tokei and strengthen independent adapter checks. Neither performance nor correctness was ranked by an equal-scope benchmark in this research. | A demonstrated architectural limit or representative evidence that an alternative materially improves correctness/deployment. [Integration rationale](../../tokei-integration.md), [Tokei Rust API](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/src/lib.rs). |

### Conditional Feature Guidance

Reviewed with the owner on 2026-10-16; these recommendations guide scope decisions and do not activate implementation. Prefer observable capabilities with clear evidence before heuristic scope changes.

| Candidate | Recommendation and rationale | Smallest useful scope and decision boundary |
| --- | --- | --- |
| Per-file comparison | Next comparison capability if the question is “which files explain this delta?”; the existing aggregate report cannot answer it. | Use the retained [per-file design](#per-file-comparison): require file records on both sides, match recorded paths, preserve unavailable evidence and classification changes, and reconcile contributions. No Git diff or rename inference. Confirm that full before/after exports with file records fit the intended workflow. |
| Generated/minified evidence | Defer until a representative scope problem; begin with explanations only because generated does not mean disposable. | Report the suspected property and the exact marker or threshold that triggered it. Keep totals and selection unchanged. Any future evidence collection must respect the measured subfolder, not expand to context files. Agree on sample inputs, false-positive checks and acceptable scan cost first. |
| Duplicate-file evidence | Defer until a consumer needs to locate copied content; repeated bytes can have legitimate distinct purposes. | Report exact-content groups while preserving physical totals. Compare bytes after hash matches, respect scan selection and keep failures explicit. Any future groups must cover measured files only, not context files. Do not add fuzzy clone detection or a unique-LOC metric without a separate contract. |

### Per-file Comparison

**Trigger:** a consumer needs per-file deltas that aggregate/topology comparison cannot answer. Met on 2026-10-25, when the owner made it the next item in PLAN; the design below became `compare --files`, which was [rejected](../../REJECTED.md#tested-and-rejected) on 2026-10-26 after its agent run. The [historical S4 decision record](../../proposals/archive/net-change.md#optional-file-detail) records why aggregate comparison shipped first; this section now owns the retained extension design.

Retained design for assessment, not supported behavior: keep file detail optional so the required comparison works with ordinary full exports. If included, `compare --files` requires both inputs to contain file records and fails clearly if either lacks them. Match by member name where applicable and root-relative path over the union of paths, including relevant unreadable/unrecognized coverage paths. Each side distinguishes counted, not present in the report, unreadable, and unrecognized; unavailable counts are `null`, not fabricated zero source measurements.

Report numeric per-file deltas only when both sides are counted. S1's failed-decoding/counting state must also be represented as unavailable if this optional extension is adopted. For paths counted on just one side, show that side's count and its contribution to the recorded totals, without claiming on-disk creation/deletion. Preserve before/after language, kind, purpose, category, rule, and language-mapping provenance. Classification-only changes remain visible with zero total line delta; they explain transfers between aggregate buckets but do not prove unchanged source content. Rename detection and content-change detection remain excluded.

**Acceptance and documentation impact if activated:** verify missing-file-record rejection, counted/unavailable states on both sides, path presence, classification-only changes, and contributions reconciling with recorded aggregate totals. Require matching recorded subfolder restrictions as aggregate comparison does. Define the optional file rows in the shared comparison types/schema; update PRODUCT and README in the implementing checkpoint. Aggregate-only inputs must remain usable without `--files`.
