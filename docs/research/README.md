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

Sensitive project analyses, candidate rules, raw measurements, and breakdowns belong exclusively in `docs/research/local/`. That directory is Git-ignored and is not part of the shared documentation.

- [Full-feature help study](agent-explorability-full-help-study.md): ten-task evaluation, bounded comparison-routing correction, adoption decision and retained resource tradeoffs.
- [Agent regression desk review, 0.7.0 to 0.8.0](agent-regression-desk-review-2026-10-21.md): no broken flag, example or route; reading-cost growth and two risks on measured tasks.

- [Report analytics assessment](report-analytics.md): DuckDB, pandas, Polars and simpler alternatives; schema-3 and Showboat decisions.
