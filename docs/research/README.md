# Research

Research records carry their own evaluation dates. Repository-facing findings describe tool capabilities and general design implications without identifying private measured projects.

The [archived implementation record](../plans/archive/measurement-quality-and-net-change.md) preserves how the recommendations were implemented and verified. [PLAN](../PLAN.md) indexes remaining work and deferred designs; these reports retain the dated research evidence.

- [Counting tools and classification](loc-counting-tools.md): Tokei reuse, alternatives, configuration requirements, and the evidence behind the built-in conventions.
- [Compare complete counter capabilities](tool-capability-comparison.md): major feature families across Locmetry, scc, cloc, Tokei, gocloc and pygount, with pinned source evidence and measurement differences.
- [Review configuration and documentation contracts](configuration-and-documentation.md): discovery, precedence, paths, failures, schemas, agent interfaces and concrete documentation drift.
- [Read historical findings and recommendations](tool-lessons-and-gaps.md): reproduced notebook defects, documentation/schema priorities, optional ideas and deliberate scope boundaries.
- [Learn from comparable net-change tools](net-change-tools.md): saved-report comparison, source diffs, history analysis, and bounded lessons for topology, coverage, and schema contracts.
- [Trials on real projects](real-project-trials.md): blind agent configuration trials, their scores, and what they changed.
- [First agent-explorability pilot](agent-explorability-task1-pilot.md): paired MEDIUM report-understanding runs with and without guides, verified outcomes, token/tool metrics, and replay instructions.
- [Six-task guide-removal study](agent-explorability-six-task-study.md): repeated MEDIUM trials across six workflows plus initial capability discovery, with correctness, token/tool costs, and a guide-retention recommendation.
- [Routing cost replication](agent-explorability-routing-replication.md): both balanced cost checks pass across 54 fresh sessions, supporting narrow adoption of the unchanged candidate.
- [Execution-first help routing](agent-explorability-help-routing-study.md): three-condition results: direct routing reduces covered-task reference reading and cost, but broader-task guard fails.
- [Configuration recipe in help](agent-explorability-help-recipe-study.md): results of the owner's usage-example proposal: two configuration layouts, unrelated-task cost, and a separate simple-rule probe.
- [Integration and distribution](../tokei-integration.md): embedded Rust backend and minimal dependency model.
- [Measurement semantics and limits](../measurement.md): selection and counting details as implemented.

Sensitive project analyses, candidate rules, raw measurements, and breakdowns of private projects belong exclusively in `docs/research/local/`. That directory is Git-ignored and is not part of the shared documentation.

- [Full-feature help study](agent-explorability-full-help-study.md): ten-task evaluation, bounded comparison-routing correction, adoption decision and retained resource tradeoffs.
- [Agent regression desk review, 0.7.0 to 0.8.0](agent-regression-desk-review-2026-10-21.md): no broken flag, example or route; reading-cost growth and two risks on measured tasks.
- [Agent regression run, schema growth](agent-regression-run-2026-10-23.md): tasks 3 and 8 against `3bec0a3`; correctness 16/16 and every cost gate passes once the language list is shared.
- [Agent regression desk review, accepted proposals and follow-ups](agent-regression-desk-review-2026-10-23.md): no correctness regression; help flat but schemas +28%, mostly a duplicated language list, so tasks 3 and 8 need a cost run.
- [Agent regression desk review, help split and category filter](agent-regression-desk-review-help-split.md): no regression; in 310 earlier sessions agents never called `-h`, so the shorter `-h` needs no agent run.
- [Agent regression desk review, 0.9.0](agent-regression-desk-review-0.9.0.md): no regression and no agent run needed; workspace output says member, notes quote flags only, stderr names severity.
- [Agent regression desk review, human output wording](agent-regression-desk-review-2026-10-22.md): help and schemas unchanged, JSON changes only in message texts; quoted whole commands omit the invocation's scope.
- [Agent regression run, 0.8.0](agent-regression-run-2026-10-22.md): tasks 3, 4 and 8 against `c9098e2`; a comparison-cost regression traced to the missing comparison `detail`, its correction and the recorded one-call residual.

- [Report analytics assessment](report-analytics.md): DuckDB, pandas, Polars and simpler alternatives; schema-3 and Showboat decisions.
- [Policy test drives](policy-test-drives/README.md): policies written for public GitHub projects, what configuration could not express, and the experiments behind the 2026-10-23 proposals.
- [Generated-code markers](generated-code-markers.md): documented markers, `.gitattributes` declarations and heuristics measured for coverage and false claims on applications and generators' own repositories.
- [Prior art for purpose classification](classification-prior-art.md): Linguist, go-enry, gengo and framework detectors; none infers a production and test split, and Linguist's generated-file checks are the starting evidence for generated code.
- [Go layout](go-layout.md): the Go packages `./...` builds, measured on six Go repositories; vendored dependencies and test inputs no longer count as production.
- [Framework layouts out of the box](framework-layouts.md): starters of the most used web frameworks measured without configuration, and an emulation of the proposed detectors.
