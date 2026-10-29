# Research

Research records carry their own evaluation dates. Repository-facing findings describe tool capabilities and general design implications without identifying private measured projects.

The [archived implementation record](../plans/archive/measurement-quality-and-net-change.md) preserves how the recommendations were implemented and verified. [PLAN](../PLAN.md) indexes remaining work and deferred designs; these reports retain the dated research evidence.

- [Counting tools and classification](loc-counting-tools.md): Tokei reuse, alternatives, configuration requirements, and the evidence behind the built-in conventions.
- [Counting backends beyond Tokei](counting-backends.md): tree-sitter, hand-written lexers and a Tokei fork compared with Tokei on probes and 14 public repositories for accuracy, speed and size.
- [Tokei fork step 2 measurement](tokei-fork-step-2.md): raw strings and comments with repeated delimiters in Rust, Swift, C#, C++, Lua and OCaml compared with tree-sitter on 22 public repositories, and speed against upstream.
- [Tokei fork step 3 measurement](tokei-fork-step-3.md): interpolation holes, regular expression literals and shell quoting in ten languages compared with tree-sitter on 34 public repositories, and speed against upstream.
- [Tokei fork step 4 measurement](tokei-fork-step-4.md): heredocs in shell, Ruby, PHP and Perl compared with tree-sitter on 37 public repositories, and speed against upstream.
- [Tokei fork step 5 measurement](tokei-fork-step-5.md): quotes only inside markup tags, JSX elements and YAML scalars in HTML, XML, YAML, JavaScript and TSX compared with tree-sitter on 41 public repositories, and speed against upstream.
- [Tokei fork step 5 resolution](tokei-fork-step-5-resolution.md): every remaining disagreement with tree-sitter on 40 public repositories corrected in the fork or decided file by file with Babel, Hermes, the Swift toolchain, bash, PyYAML, parse5 or minimal patches, and speed against upstream.
- [Tokei fork step 1 measurement](tokei-fork-step-1.md): the fork's character-literal and missing-declaration fixes compared with tree-sitter on 17 public repositories, every remaining disagreement explained, and speed against upstream.
- [Compare complete counter capabilities](tool-capability-comparison.md): major feature families across Locmetry, scc, cloc, Tokei, gocloc and pygount, with pinned source evidence and measurement differences.
- [Review configuration and documentation contracts](configuration-and-documentation.md): discovery, precedence, paths, failures, schemas, agent interfaces and concrete documentation drift.
- [Read historical findings and recommendations](tool-lessons-and-gaps.md): reproduced notebook defects, documentation/schema priorities, optional ideas and deliberate scope boundaries.
- [Learn from comparable net-change tools](net-change-tools.md): saved-report comparison, source diffs, history analysis, and bounded lessons for topology, coverage, and schema contracts.
- [Trials on real projects](real-project-trials.md): blind agent configuration trials, their scores, and what they changed.
- [Agent studies](agents/README.md): how agents use Locmetry: explorability studies, regression checks of interface changes, feedback rounds on real projects and the experiments that verified them.
- [Integration and distribution](../tokei-integration.md): embedded Rust backend and minimal dependency model.
- [Measurement semantics and limits](../measurement.md): selection and counting details as implemented.

Sensitive project analyses, candidate rules, raw measurements, and breakdowns of private projects belong exclusively in `docs/research/local/`. That directory is Git-ignored and is not part of the shared documentation.

- [Report analytics assessment](report-analytics.md): DuckDB, pandas, Polars and simpler alternatives; schema-3 and Showboat decisions.
- [Policy test drives](policy-test-drives/README.md): policies written for public GitHub projects, what configuration could not express, and the experiments behind the 2026-10-23 proposals.
- [Generated-code markers](generated-code-markers.md): documented markers, `.gitattributes` declarations and heuristics measured for coverage and false claims on applications and generators' own repositories.
- [Prior art for purpose classification](classification-prior-art.md): Linguist, go-enry, gengo and framework detectors; none infers a production and test split, and Linguist's generated-file checks are the starting evidence for generated code.
- [Go layout](go-layout.md): the Go packages `./...` builds, measured on six Go repositories; vendored dependencies and test inputs no longer count as production.
- [Category vocabulary](category-vocabulary.md): the category names 24 policies chose, how they drift, and four fresh GitHub projects measured to test whether one reference vocabulary covers them.
- [Framework layouts out of the box](framework-layouts.md): starters of the most used web frameworks measured without configuration, and an emulation of the proposed detectors.
