# Research

Research conducted on 2026-10-15. Repository-facing findings describe tool capabilities and general design implications without identifying measured projects.

- [Counting tools and classification](loc-counting-tools.md): Tokei reuse, alternatives, configuration requirements, and the evidence behind the built-in conventions.
- [Compare complete counter capabilities](tool-capability-comparison.md): major feature families across Locmetry, scc, cloc, Tokei, gocloc and pygount, with pinned source evidence and measurement differences.
- [Review configuration and documentation contracts](configuration-and-documentation.md): discovery, precedence, paths, failures, schemas, agent interfaces and concrete documentation drift.
- [Discuss lessons and open gaps](tool-lessons-and-gaps.md): reproduced notebook defects, documentation/schema priorities, optional ideas and deliberate scope boundaries.
- [Learn from comparable net-change tools](net-change-tools.md): saved-report comparison, source diffs, history analysis, and bounded lessons for topology, coverage, and schema contracts.
- [Trials on real projects](real-project-trials.md): blind agent configuration trials, their scores, and what they changed.
- [Integration and distribution](../tokei-integration.md): embedded Rust backend and minimal dependency model.
- [Measurement semantics and limits](../measurement.md): selection and counting details as implemented.

Sensitive project analyses, candidate rules, raw measurements, and breakdowns belong exclusively in `docs/research/local/`. That directory is Git-ignored and is not part of the shared documentation.
