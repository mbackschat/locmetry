# Counting Tools and Classification

Research date: 2026-10-15. This document contains general findings only; project-specific evidence is kept in Git-ignored local documentation.

## Reuse Tokei

Use the Tokei Rust library as the counting backend. Its per-file code, comment, blank, and embedded-language statistics support Locmetry’s scope, classification, and folder aggregation. The initial CLI-wrapper experiment established feasibility; the selected [embedded integration](../tokei-integration.md) supports a single executable with minimal Homebrew dependencies. [Tokei documentation](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/README.md)

Tokei already reads `tokei.toml` or `.tokeirc`. It merges configuration-directory, home-directory, and working-directory settings; local discovery follows the process working directory. Exclusions use `.tokeignore`, `.ignore`, `.gitignore`, or `--exclude`; the inspected TOML configuration does not have an exclusion-list field. Configuration alone therefore does not justify a separate tool. [Configuration source](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/src/config.rs), [exclusion documentation](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/README.md#excluding-folders)

## Alternative Tools

| Tool | Useful capabilities | Relevance |
| --- | --- | --- |
| [Tokei](https://github.com/XAMPPRocky/tokei) | CLI JSON, Rust API, file reports, embedded-language counts | Initial counting backend |
| [scc](https://github.com/boyter/scc/blob/v4.1.0/README.md) | `.sccconfig`, JSON, Go API, pattern-based reporting categories | Strong alternative; borrow explicit precedence and configurable categorization |
| [cloc](https://github.com/AlDanial/cloc) | Extensive filtering, configuration, per-file output, report summation | Useful comparison tool and filtering reference |
| [gocloc](https://github.com/hhatto/gocloc) | Compact Go API and JSON file reports | Potential adapter if Go integration becomes relevant |

No comparative performance benchmark was conducted. Parsing, ignore behavior, and error handling differ between engines; adapters must not imply identical counts merely by normalizing field names.

## Classification Requirements

Ownership/scope comes before purpose: upstream source in a reference corpus is not project implementation merely because a counter recognizes its language. Exclude it through explicit project policy. Keep test data distinguishable from test source; serialized snapshots, models, and configuration can dominate raw code columns.

Use conventional test paths as defaults, with explicit overrides for test-only helper packages, type assertions, and framework source sets. For example, Kotlin multiplatform uses source sets such as `commonTest`, while Go uses `_test.go`; generic `/test/` matching is insufficient. Mixed files remain a limit of whole-file classification. [Kotlin source sets](https://kotlinlang.org/docs/multiplatform/multiplatform-discover-project.html), [Go testing](https://pkg.go.dev/testing), [Rust test organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)

Recommend ordered, root-relative project rules with visible rule provenance and an unclassified fallback. Scope/evidence rules may need to outrank filename heuristics. A generic directory named `fixtures` can contain supported example data as well as test helpers, so its purpose needs evidence. Maintained hidden tooling and source directories named like outputs require exceptions to blanket exclusions.

Keep language, purpose, and optional category separate. Production can mean maintained non-test source, but tooling, examples, benchmarks, and third-party bootstrap files should remain visible as distinct categories. This definition must accompany totals rather than imply all included code ships to users.

Classification cannot recover a file the backend does not recognize. Known-source extensions may need a supported language override or an explicit coverage limitation; the PoC does not silently equate absent records with zero source lines.

## Aggregation and Output

Aggregate each original file once. Tokei reports embedded language data inside file statistics and through aggregate views; summing both duplicates counts. A wrapper must define how host and embedded languages contribute while keeping physical-file counts separate from per-language counts. [Tokei statistics API](https://docs.rs/tokei/15.0.0/tokei/struct.CodeStats.html)

Expose direct folder counts and recursive totals separately. A recursive parent equals its direct files plus its immediate children's recursive totals; all displayed recursive rows are not additive.

Recommend a versioned JSON contract containing backend identity, scan scope, relevant policy, file classification provenance, and available error/inaccuracy signals. Human tables should use the same measurements. Record whether ambient backend settings were inherited; the CLI does not automatically provide a hermetic scan. Do not promise an exhaustive manifest of skipped or unreadable files without a supporting backend API.

## Local Evidence Boundary

The executable PoC and synthetic tests may be shared. Measured-project names, paths, counts, rules, and analyses must remain under Git-ignored `docs/research/local/`. Before committing documentation, verify that no local measurement artifacts or identifying references enter Git's candidate file set.
