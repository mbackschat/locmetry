# Measurement Quality and Net Change Work Plan

Status: sequence accepted by the owner on 2026-10-15; implementation not started. The accepted order is notebook correctness, documentation corrections, shared report/schema contracts, net change, then selection explanations. This file makes those recommendations executable and retains their rationale. Detailed public-interface choices remain recommendations until resolved at the relevant stage; accepting the sequence does not implement or publish a feature.

[Development plan](../PLAN.md) indexes this work. [PRODUCT.md](../PRODUCT.md) remains the supported contract. The [net-change proposal](../proposals/net-change.md) owns the proposed comparison design; this plan owns sequencing, tasks, review checkpoints and completion evidence. The [capability comparison](../research/tool-capability-comparison.md), [configuration/documentation audit](../research/configuration-and-documentation.md), and [lessons and gaps](../research/tool-lessons-and-gaps.md) preserve the research evidence.

## Evidence and Decision Discipline

The research baseline is Locmetry commit `bece839731d78092e997b5b8b959801324e64b26`, scc 4.1.0, cloc 2.10, Tokei 15.0.0, gocloc 0.7.0 and pygount 3.2.0. Peer behavior was established by documentation and source inspection, not execution. Notebook defects were reproduced with a fresh Locmetry build. Binary-content observations used an existing binary with unverified build identity; the `.gitmodules` limitation was source-inspected. Recommendations are judgments drawn from this evidence, not demonstrated demand, performance rankings or quantified return on investment.

Update each stage when work lands: status, dated decision changes, relevant commit, failing-before/passing-after evidence, compatibility impact and remaining limitations. Keep historical research findings intact and link their resolution from this plan. Do not mark a stage complete because its implementation compiles or its totals reconcile; use its stated independent acceptance checks.

Resolve ordinary implementation details within the accepted scope. Record material changes to public semantics in the proposal or product contract, with the reason and evidence; seek an owner decision when expanding scope. An existing recommendation is the starting judgment, not a requirement to ask the same question again. Do not carry an unresolved contract assumption silently into implementation.

## Sequence and Live Status

| Stage | Work | Depends on | Status and completion evidence |
| --- | --- | --- | --- |
| S1a | Correct notebook normalization | None | Not started. Fresh-build reproduction exists; no fix. |
| S1b | Expose structured-document parse failure | S1a backend findings | Not started. Fresh-build reproduction exists; error contract needs resolution. |
| S2 | Correct documentation and executable examples | S1 public behavior for final wording | Not started. Audit identifies affected sections. |
| S3 | Unify typed report and schema contracts | S1, S2 | Not started. Preserve valid measurement version-1 output. |
| S4 | Implement aggregate and topology net change | S3 | Not started. Detailed design remains in the proposal. |
| S5 | Add a focused selection-explanation workflow | S4 | Not started. Public interface requires a bounded design. |

Documentation changes needed by S1 land with S1; S2 addresses the wider audit. The same rule applies to every later stage. The sequence is not permission to defer documentation until the next stage.

## S1a — Correct Notebook Normalization

**Rationale.** A valid notebook containing one Python line reports two; a separate two-line notebook reports four. Tokei includes code-cell counts in both parent statistics and language blobs. Locmetry treats them as disjoint. This violates the existing count-once guarantee. Arithmetic reconciliation cannot detect duplication repeated consistently through files and aggregates, so independent expected-count fixtures are essential.

**Provenance.** [Fresh-build reproduction](../research/tool-lessons-and-gaps.md#notebooks-break-two-existing-measurement-guarantees), [Locmetry backend](../../src/backend.rs), and [Tokei notebook aggregation](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/src/language/language_type.rs#L279).

- [ ] Add public synthetic tests that fail on the one-line and two-line fixtures before changing code.
- [ ] Establish expected parent/embedded contributions for empty notebooks, multiple cells, Markdown/raw cells and unsupported kernels; document unsupported cases rather than guessing totals.
- [ ] Correct normalization at the backend boundary, preserving physical-file identity, language contributions, host kind/purpose and additive breakdown semantics.
- [ ] Verify recognized and explicitly mapped notebook paths use the same normalization where mapping is supported.
- [ ] Guard ordinary source and other embedded-language cases against regressions; retain independent count oracles as well as aggregate equations.

**Completion check.** The original fixtures report exactly one and two code lines; file, language, folder and workspace views obey their documented counting semantics. Relevant regression tests fail before the change and pass after it; required checks and the private measurement regression pass, or an unavailable private environment is explicitly recorded as outstanding evidence.

**Documentation in this stage.** Update [measurement semantics](../measurement.md#counting), the [product measurement model](../PRODUCT.md#measurement-model) where needed, and any affected README example. Describe extracted notebook-cell counts versus serialized JSON lines. Record the backend correction and its tested limits; do not claim complete notebook correctness from two fixtures.

**Review checkpoint.** Confirm the representation invariant fixes the class of overlap rather than only Jupyter display totals. Retain Tokei unless evidence shows the embedded API cannot support a sound boundary.

## S1b — Expose Unavailable Parsing

**Rationale.** Malformed JSON and valid JSON with the wrong notebook structure both produce zero-count records, exit 0 and `backend_inaccurate = false`. Tokei converts parse failure into default statistics, so I/O coverage cannot see it. Subtracting such a zero later would invent a change. File access, successful parsing and successful counting are separate facts.

**Provenance.** [Reproduced coverage failure](../research/tool-lessons-and-gaps.md#notebooks-break-two-existing-measurement-guarantees), [Tokei fallback](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/src/language/language_type.rs#L67), and pygount's [explicit processing states](https://github.com/roskakori/pygount/blob/v3.2.0/pygount/analysis.py#L74).

- [ ] Add failing tests for malformed JSON and wrong-shaped JSON, alongside a valid empty notebook that must remain a known zero.
- [ ] Decide and document the public parse-failure representation before changing serialized output. Recommendation: a distinct structured failure reason, partial-measurement signaling and exit 2, consistent with existing incomplete-measurement handling; do not falsely label a readable file as an I/O read failure.
- [ ] Make failure observable at the backend boundary and propagate it through single-root/workspace coverage and all output formats.
- [ ] Keep unavailable measurement separate from classification and heuristic findings: a generated file can still be counted production source.
- [ ] Assess report-version compatibility and amend the comparison proposal's coverage rules to consume the chosen failure representation. Avoid a generic processing-state framework beyond the required contract.

**Completion check.** Neither invalid fixture is certified as a successfully measured zero; the valid empty fixture remains zero. JSON, terminal and Markdown communicate the same limitation, workspace aggregation retains it, diagnostics preserve machine-readable stdout, and the selected exit contract is tested. Required checks and private measurement regression follow the same evidence rule as S1a.

**Documentation in this stage.** Update [coverage guarantees](../PRODUCT.md#reporting-and-coverage), [measurement limits](../measurement.md#limits), README coverage/exit guidance, executable report-schema descriptions and the [proposal's missing-evidence semantics](../proposals/net-change.md#comparability-and-missing-evidence).

**Review checkpoint.** Can an agent distinguish a known zero from failed parsing using documented fields alone? Record any remaining structured-container failure modes that the chosen solution cannot observe.

## S2 — Correct Documentation and Guard Meaningful Examples

**Rationale.** Documentation defines users' expectations and agents' workflows. A correct executable accompanied by misleading scope or CI examples still produces incorrect decisions. Peer documentation also drifts from help and serializers, so adding prose alone is insufficient. Correct claims now and exercise examples where defaults, missing values or failures change their meaning.

**Provenance.** The [location-by-location audit](../research/configuration-and-documentation.md#documentation-deserves-the-same-consistency-checks-as-code) links current Locmetry sections and peer evidence. Tokei's [README](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/README.md) and [CLI](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/src/cli.rs), and pygount's [JSON reference](https://github.com/roskakori/pygount/blob/v3.2.0/docs/json.md) and [serializer](https://github.com/roskakori/pygount/blob/v3.2.0/pygount/write.py), illustrate concrete drift.

- [ ] Qualify universal counting claims with selection, recognition, decoding/parsing and coverage boundaries; describe Tokei reuse plus local corrections.
- [ ] State when per-file records require `--files`; align headline/full-report descriptions with emitted fields, including module metadata.
- [ ] Distinguish self-contained executables from explicitly static Linux builds.
- [ ] Qualify reproducibility with versions, personal settings, ignore context and evidence absent from reports. Keep classification trial claims tied to their population and task.
- [ ] Describe modules as descriptor-triggered layout inference; distinguish whole-file purpose from semantic build membership.
- [ ] Update competitor comparisons to acknowledge diagnostics, configuration, analytics and integrations. Describe the combined Locmetry contract without unsupported uniqueness or accuracy claims.
- [ ] Make CI recipes preserve failure/partial exit status and define missing/zero-production behavior. Validate successful, zero-production and failed/partial examples with synthetic inputs.
- [ ] Explain configuration discovery and path anchors with examples: explicit replacement, personal settings, first-match order, exclusion union and workspace member policy. Check examples against actual parser/help behavior.
- [ ] Keep net change visibly proposed until S4 lands; add task-oriented documentation links where readers need them.

**Completion check.** Each audit item has a correction or a documented disposition with evidence. CLI flags and schema examples agree with the executable; CI examples demonstrably retain failure status and avoid division by zero. Test behavior-sensitive examples rather than snapshotting prose merely to detect edits.

**Documentation in this stage.** [README](../../README.md), [PRODUCT.md](../PRODUCT.md), [measurement semantics](../measurement.md), [integration/distribution](../tokei-integration.md), and schema descriptions as applicable. Preserve one owner for each guarantee and link supporting detail.

**Review checkpoint.** Can a new user or agent choose scope, understand missing counts and use a report safely without reading source? Record residual gaps explicitly before S3.

## S3 — Unify Typed Report and Schema Contracts

**Rationale.** Configuration schemas derive from typed models, while measurement JSON and its handwritten schema have separate owners. A comparison schema must not introduce another independently maintained solution. The current report schema permits negative counts and leaves some effective-policy structure unconstrained. A focused public-contract refactor makes saved reports a reliable input boundary while preserving valid existing output.

**Provenance.** [Configuration schema code](../../src/config/schema.rs), [report schema](../../src/report_schema.rs), [schema audit](../research/configuration-and-documentation.md#machine-interfaces-must-preserve-policy-and-uncertainty), and the proposal's [one contract mechanism](../proposals/net-change.md#one-contract-mechanism). Pygount's [independent JSON format version](https://github.com/roskakori/pygount/blob/v3.2.0/docs/json.md) is useful compatibility precedent.

- [ ] Inventory actual version-1 root/workspace and headline/full variants, including optional/null values, omitted zeros, file detail, effective policy and S1 failure semantics.
- [ ] Add compatibility fixtures and rejection tests before replacing manual construction: valid existing reports, negative counts, invalid variants, duplicate identities and conflicting totals.
- [ ] Give serialization, deserialization, structural schema and field descriptions shared typed public owners through the existing `schemars` mechanism. Share definitions only when shape and meaning agree.
- [ ] Preserve valid version-1 output and document validation tightening; explicitly decide any incompatible S1 representation change rather than hiding it in this refactor.
- [ ] Centralize headline projection. Keep internal aggregation types separate where their semantics differ; do not create a generic schema/plugin framework.
- [ ] Add semantic validation for arithmetic, identity uniqueness, coverage and totals supported by available detail. Keep comparison-only requirements, such as full-input eligibility, distinguishable from ordinary report validity.
- [ ] Describe nonnegative ranges, missing versus zero, language-participation file counts, additive embedded rows, and effective-policy structures in self-contained schemas with valid examples.

**Completion check.** Existing valid measurement exports retain their documented shape; actual root/workspace outputs validate against generated schemas; malformed structural and semantic cases fail clearly. Config parsing, report serialization and schema generation agree without copies of field contracts. Tests, formatting and lint pass; run the private regression if measurement behavior changes.

**Documentation in this stage.** Update executable schema descriptions, [report contract](../PRODUCT.md#reporting-and-coverage), README automation examples and [proposal compatibility requirements](../proposals/net-change.md#accepted-inputs-and-identity). Document narrower validation and supported versions where report consumers look for them.

**Review checkpoint.** Can an agent learn structure and semantics from the executable schemas alone? Check that the refactor has not silently broadened supported inputs or changed counts.

## S4 — Implement Aggregate and Topology Net Change

**Rationale.** Comparing saved measurements directly serves the owner's workflow and works without source checkouts. A useful result preserves before/after counts and explains workspace composition, package detection and coverage changes. Removing a root from configuration must not look like proven source deletion; a package descriptor can disappear while its folder and LOC remain. Zero aggregate change can conceal significant arrivals, departures or reclassification.

**Provenance.** [Net-change proposal](../proposals/net-change.md), [comparison research](../research/net-change-tools.md), cloc's [source comparison](https://github.com/AlDanial/cloc/blob/v2.10/cloc#L3190), and scc's [history analysis](https://github.com/boyter/scc/blob/v4.1.0/processor/history.go). The peers demonstrate adjacent capabilities, with different semantics.

- [ ] Review the proposal against S1/S3 outcomes and record decisions on its remaining public-interface recommendations. Recommended initial scope: aggregate/topology comparison; optional per-file comparison deferred to its own decision.
- [ ] Add failing tests for full-report eligibility, direction, version/mode incompatibility, arithmetic overflow and destination aliases before implementing the relevant behavior.
- [ ] Implement `after - before` using checked arithmetic and one comparison model; never reload source, current config or ignore files.
- [ ] Match the union of recorded identities and represent member/folder/package presence explicitly. Use existing modules as package evidence; package rows must not double-add their folder counts.
- [ ] Separate changes in matched roots from contributions of entering/leaving roots; preserve zero-count member changes and disjoint nonempty workspaces.
- [ ] Emit stable structured findings for recorded policy, counter, identity, module, coverage and presence differences, including S1 parse failures. Absence of findings is not proof of source-only change.
- [ ] Expose the self-contained comparison schema through the shared generation mechanism, with semantic descriptions and examples.
- [ ] Render terminal, Markdown and JSON from the same result, preserving findings and consistent totals in headline/full forms.
- [ ] Verify self-comparison, reversal, ordering independence, zero-net replacements, reclassification, nested-folder non-additivity, descriptor removal, coverage-only paths and incomplete measurements.

**Completion check.** Satisfy the proposal's [acceptance criteria](../proposals/net-change.md#acceptance-criteria-and-decisions), including compatibility with eligible old full exports and no conversion of unavailable counts into known zero. All required formats agree. Tests, formatting and lint pass; run private regression if shared measurement behavior changes. No Git execution, rename detection, empty-directory inventory, percentage metrics or history store is required.

**Documentation in this stage.** Move shipped behavior into [PRODUCT.md](../PRODUCT.md); add README recipes for single-root/workspace comparisons, arrivals/departures and partial coverage; update schema descriptions and [measurement limits](../measurement.md). Mark the proposal implemented only to the extent delivered, retaining explicit optional/excluded scope. Resolve stale product exclusions in the same change.

**Review checkpoint.** Can a reader distinguish recorded LOC changes, configuration changes and workspace membership changes? Assess whether aggregate/topology output answers the original workflow before deciding on optional per-file detail.

## S5 — Add a Focused Selection Explanation

**Rationale.** This is the strongest optional usability recommendation because it helps establish trustworthy scope. Existing rule IDs and review findings provide ingredients; a path-focused explanation can connect why a file was excluded, unrecognized, assigned a parser or purpose, or associated with a module convention. The original Handlebars investigation demonstrates the need to understand the counted input. Peer diagnostics establish an interface precedent, not measured demand.

**Provenance.** cloc's [found/counted/ignored and filter diagnostics](https://github.com/AlDanial/cloc/blob/v2.10/cloc#L2310), [Locmetry findings](../../src/report.rs), and the [research recommendation](../research/tool-lessons-and-gaps.md#evidence-can-improve-scope-without-silently-changing-totals).

- [ ] Write a bounded public contract using synthetic debugging cases: ignored path, project exclusion, unrecognized extension, mapped parser, first-match classification, module inference and parse failure.
- [ ] Prefer a targeted path workflow if it answers those cases without inventorying every pruned file; decide the command and machine output only after that assessment.
- [ ] Preserve shared configuration discovery, matching and counting behavior. Explain the actual decision rather than reimplementing it in a diagnostic path.
- [ ] Expose known reasons and missing evidence, with stable machine semantics where needed. Do not infer a cause the traversal never recorded.
- [ ] Add failing behavior tests before implementation, including single-root/workspace path bases, personal defaults and rule ordering.

**Completion check.** The documented workflow answers each agreed debugging case, with a reason consistent with ordinary measurement and no silently different scope policy. Required checks pass; assess private regression if shared measurement paths change.

**Documentation in this stage.** Add a task-focused troubleshooting guide if the workflow outgrows README usage, link it from the README, and update the product/schema contract for any public interface. Include complete examples using the executable's real output.

**Review checkpoint.** Assess against the agreed debugging tasks. If available evidence cannot support the proposed explanation, narrow the promise or revise the design explicitly before shipping it.

## Conditional Follow-ups and Preserved Rationales

These are recorded for later assessment, not added to the accepted implementation sequence. Promote an item only with its trigger, scope, acceptance criteria and documentation impact recorded here or in a linked plan.

| Item | Recommendation and rationale | Trigger, evidence and next assessment |
| --- | --- | --- |
| Generated/minified evidence | Report suspicion with reasons before considering automatic exclusion. Generated files can be maintained or intentionally included; markers and line-length thresholds are heuristics. | A real scope-debugging need. Define inspected bytes/lines, thresholds, false positives and policy provenance. Learn from [pygount reasons](https://github.com/roskakori/pygount/blob/v3.2.0/pygount/analysis.py#L375) and [scc heuristics](https://github.com/boyter/scc/blob/v4.1.0/processor/workers.go). |
| Duplicate evidence | Preserve physical totals; optionally identify equal-content groups. Equal bytes can belong to different purposes or members. A unique-content metric needs a separate definition and deterministic representative choice. | A consumer needs duplication evidence. Compare [cloc default deduplication](https://github.com/AlDanial/cloc/blob/v2.10/cloc#L1450), [pygount handling](https://github.com/roskakori/pygount/blob/v3.2.0/pygount/analysis.py#L385) and scc's opt-in exclusion before choosing semantics. |
| Binary/encoding policy | Establish current-source fixtures before changing defaults. A NUL heuristic can reject text encodings; unknown language is not known source code. | Reproduce the weaker existing-binary observation and audit decoding/container boundaries. Keep exclusions and unavailable states explicit. [Qualified finding](../research/tool-lessons-and-gaps.md#evidence-can-improve-scope-without-silently-changing-totals). |
| `.gitmodules` syntax | Separate correctness follow-up: reproduce quoted/escaped paths and inline comments, then improve parsing or state the supported subset. Do not build further behavior on an unverified parser assumption. | The source-inspected weakness needs independent fixtures. [Finding](../research/tool-lessons-and-gaps.md#shared-contracts-and-accurate-examples-come-next), [current reader](../../src/backend.rs). |
| Language/help discovery | Consider a compact language list and task examples generated from existing definitions. Avoid a second drifting registry. | A demonstrated agent/user discovery problem; compare [Tokei CLI](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/src/cli.rs). |
| Git history analytics | Defer: hotspots, coupling and authorship introduce history-window and identity semantics beyond saved measurements. | A concrete history workflow that existing tools cannot serve adequately. [scc history](https://github.com/boyter/scc/blob/v4.1.0/processor/history.go). |
| Complexity, ULOC/DRYness and cost estimates | Defer: heuristic/formula metrics require independent definitions and validation, and do not establish effort or quality. | A named decision requiring a metric and an agreed validation corpus. [scc metrics](https://github.com/boyter/scc/blob/v4.1.0/processor/workers.go), [LOCOMO](https://github.com/boyter/scc/blob/v4.1.0/processor/locomo.go). |
| MCP | Defer while CLI/JSON/schema meet consumers' needs. Any new interface must share policy and result semantics; scc's MCP bypasses CLI config discovery in the inspected source. | A client cannot adequately use the existing boundary. [scc entry](https://github.com/boyter/scc/blob/v4.1.0/main.go), [MCP handlers](https://github.com/boyter/scc/blob/v4.1.0/mcp.go). |
| HTML, CSV, SQL and OpenMetrics | Consumer-driven: each format needs a user, compatibility expectations and evidence that existing output is insufficient. | A stakeholder-sharing or named analytics/monitoring integration. [scc formats](https://github.com/boyter/scc/blob/v4.1.0/processor/formatters.go). |
| Streaming and resource limits | Measurement-driven: streaming changes ordering/detail tradeoffs; silent truncation or size exclusion would change scope. | Reproducible memory, latency or oversized-file problems with a representative corpus. [Capability comparison](../research/tool-capability-comparison.md#selection-and-grouping-determine-which-totals-matter). |
| Archive/remote input, custom lexers, more platforms | Defer until an unsupported input or deployment requirement justifies parsing, authority and maintenance costs. | A representative use case and a feasible test/release matrix. [Input/deployment comparison](../research/tool-capability-comparison.md). |
| Backend replacement | Retain Tokei and strengthen independent adapter checks. Neither performance nor correctness was ranked by an equal-scope benchmark in this research. | A demonstrated architectural limit or representative evidence that an alternative materially improves correctness/deployment. [Integration rationale](../tokei-integration.md), [Tokei Rust API](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/src/lib.rs). |

## Completion Evidence and Ongoing Review

For each implementation checkpoint, record the tested behavior and commit in the stage's status row, update its checkboxes, and preserve material rationale changes with a date. Record an unavailable check as unavailable, never as passed. The [repository guidelines](../../CLAUDE.md) own commands and commit rules; [measurement semantics](../measurement.md#private-regression) owns private validation instructions. Tests and examples must use public synthetic fixtures; private evidence remains in the ignored research directory.

Review documentation in every checkpoint: README for workflows, PRODUCT for shipped guarantees, measurement semantics for counting/coverage, schema descriptions for agents, and a focused new guide when existing documents would become overloaded. Update this plan with verified progress in the same commit. Link rather than duplicate authoritative contracts; update the proposal when design changes and the product contract when behavior ships. No release or push is implied by completion of this plan.

Decision record, 2026-10-15: the owner accepted the proposed sequence and requested an actionable plan retaining rationale, source provenance and ongoing assessment, with documentation maintained throughout. This checkpoint records that plan only; all implementation stages remain unstarted.
