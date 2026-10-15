# Development Plan

## Purpose and Status

Move the working Rust PoC to the configuration and reporting contract in [PRODUCT.md](PRODUCT.md), then prepare macOS and Linux distribution. This file tracks remaining work and completion evidence; the product document owns requirements, and [PoC usage](poc/README.md) describes current behavior. Unchecked items are planned work, not supported features.

## Completed Foundation

- [x] Rust executable with embedded Tokei and locked Cargo dependencies; no Python, Git, or separate counter executable required at runtime.
- [x] Ordered JSON/regex classification profiles with explicit exclusions, purpose/category provenance, and an unclassified fallback.
- [x] Per-file counting, embedded-language deduplication, direct/recursive folder totals, JSON, and Markdown output.
- [x] Synthetic integration tests, an empty-`PATH` runtime test, and opt-in private measurement regression checks.
- [x] Review hardening (2026-10-15): report suffixes are appended so dotted prefixes stay distinct, and outputs never overwrite the configuration; paths are split by the platform parser, so Unix names containing `:` or `\` stay literal; raw backend payloads contain only root-relative paths; content kinds use a reviewed Tokei language mapping with source as the default; JSON and Markdown report unrecognized and unreadable files; documentation embedded in source counts as host comments. The private regression still reproduces all saved measurements.

The [integration and packaging document](docs/tokei-integration.md) records the backend boundary. These foundations do not complete the product configuration or release work below.

## Required Work

### 1. TOML Configuration and Discovery — Done

Implemented 2026-10-15; evidence in [configuration tests](tests/config.rs) and [CLI tests](tests/cli.rs). JSON profiles remain loadable for migration (§4).

- [x] Parse and validate the version-1 TOML contract, including unknown keys, unsupported versions/backends, duplicate IDs, and invalid values.
- [x] Select the scan root independently of configuration; discover `.locmetryrc` only at that root. An explicit configuration path must not change the root.
- [x] Resolve built-in defaults, project settings, and explicit CLI settings in the agreed precedence order; record the effective policy.
- [x] Settle missing-config behavior (built-in defaults; recorded in [PRODUCT.md](PRODUCT.md)). Profiles stay loadable through `.json` configurations until the migration in §4.

**Completion evidence:** synthetic CLI tests cover discovery from another working directory, explicit configuration outside the scan root, precedence, and validation failures before scanning. The product's example configuration runs successfully.

### 2. Glob Matching and Configurable Scope — Done

Implemented 2026-10-15. Selection uses Locmetry's own globset matcher; directories are pruned only for `<dir>/**` exclusions, which exclude every descendant.

- [x] Implement the root-relative, case-sensitive glob contract from [PRODUCT.md](PRODUCT.md#matching-and-precedence), without inheriting different backend matching semantics.
- [x] Apply the default exclusions unless `scope.default_excludes = false`, and add `scope.exclude` to them; remove the PoC's unconditional Git exclusion where the effective policy permits inclusion.
- [x] Wire hidden-file and ignore-file settings through selection, with explicit exclusions effective even when ignore files are disabled.
- [x] Expose the necessary scope overrides through the CLI and document their precedence.

**Completion evidence:** boundary tests distinguish root-only and recursive patterns, zero-directory `**` matches, directory/file names, case, invalid patterns, default opt-out, hidden files, and ignore exceptions. Excluded files never enter counting results.

### 3. Classification Stages and Content Coverage — Done

Implemented 2026-10-15. Provenance needs no separate stage field: entry IDs are unique across stages, conventions use `builtin:`, and unmatched files use `fallback`.

- [x] Implement scope → first override → built-in tests → first ordinary rule → configured fallback.
- [x] Match entries by path and optional `kind`; record optional `reason` in the effective-policy snapshot.
- [x] Define and document versioned test conventions, covering common test directories, filename conventions, and framework source sets with synthetic examples. Keep ambiguous ownership configurable.
- [x] Report the matching stage and rule/convention identity, including fallback provenance; settle omitted-category behavior.
- [x] Review the language-to-kind mapping for the supported release scope. Keep unsupported or unmapped content visible and document coverage limits. Evidence: kind and coverage tests in [measurement tests](tests/measurement.rs).

**Completion evidence:** tests prove broad production rules cannot swallow conventional tests, explicit evidence overrides can beat test names, test data remains data, and embedded documentation examples stay outside source totals.

### 4. Reporting Contract and Profile Migration — Next

Current gap: JSON is provisional `poc-3`, Markdown folder depth is fixed at two, and JSON stdout changes shape when `--output` is supplied. Partial-result behavior exists but is not yet the settled product contract.

- [ ] Finalize versioned JSON fields, stdout behavior, and success/error/partial-result exit semantics; document them in the product contract.
- [ ] Add the folder-depth CLI option without changing complete JSON totals or making recursive rows additive.
- [ ] Test effective-policy metadata, classification provenance, incomplete results, empty scans, and agreement between human and JSON output.
- [ ] Translate reviewed private profiles to TOML/globs. Brace alternation and `kind` filters cover every reviewed regex rule (checked 2026-10-15); review any remaining semantic differences explicitly. Keep every real profile and comparison Git-ignored.
- [ ] Compare equivalent file inventories and rule decisions; explain intentional changes caused by new defaults before retiring the provisional interface.
- [ ] Update usage examples and contributor guidance to the implemented product interface.

**Completion evidence:** synthetic contract tests pass; private comparisons reconcile scope, classifications, and folder totals without silently accepting differences caused by changed working copies.

### 5. Release Readiness for macOS and Linux

Current gap: the dependency model is documented, but there is no complete formula, tap, or published release. Builds and tests have only run on macOS.

- [ ] Establish the project license, release location, tap ownership, and supported architectures for macOS and Linux.
- [ ] Build, test, and lint on Linux; embedded Tokei is cross-platform, so no counter executable is needed there either.
- [ ] Publish standalone Linux release binaries alongside the macOS ones, and provide Homebrew-on-Linux bottles from the same formula.
- [ ] Prepare release artifacts, checksums, and a formula with Rust as a build-only dependency and no runtime formula dependencies.
- [ ] Add a formula test that measures a synthetic project and checks JSON classification and totals.
- [ ] Verify installation and binary linkage on each supported target; retain measurement with an empty `PATH` as a runtime-dependency guard.

**Completion evidence:** release artifacts and the formula are reviewable, source/bottle installations work on every declared macOS and Linux target, the empty-`PATH` measurement passes on Linux, and publication is explicitly authorized before publishing.

## Open Contract Decisions

Record accepted decisions in [PRODUCT.md](PRODUCT.md) before implementing dependent behavior.

| Decision | Recommendation and rationale |
| --- | --- |
| Legacy JSON profiles | Prefer a reviewed one-time migration over permanent dual-format support. Preserve historical inputs privately so equivalence remains checkable. |
| JSON stdout with report files | Keep the same full JSON schema with and without `--output`; a stable shape is easier for agents to consume. |
| Language/extension mappings | Add a small built-in mapping for unrecognized extensions that are common in the reference projects, such as `.gradle` build scripts and `.jsonc` configuration, before offering user-configurable mappings. Coverage now lists these files, so absence is visible either way; a mapping makes them countable. |
| Partial counts | Retain explicitly marked partial output with a nonzero exit; agents must not mistake incomplete results for a complete measurement. |

## Optional and Excluded Scope

Optional after the required contract: additional counter adapters, custom content-kind overrides, and language/extension mappings where supported by the backend. None is a prerequisite for the initial Tokei-backed release.

Excluded initially: a new counting engine, generic plugin infrastructure, semantic splitting of tests and production within one file, and LLM-dependent normal measurement runs.

## Working Rules and Verification

Work through configuration, scope, classification, reporting, then packaging. Use red/green TDD and mark a task complete only when its acceptance evidence exists. Keep this plan current as work lands; change requirements in the product document rather than creating a second contract here.

Run `cargo test --locked`, `cargo fmt --all --check`, and `cargo clippy --locked --all-targets -- -D warnings` for implementation changes; build with `cargo build --release --locked`. Use the [private regression procedure](poc/README.md#private-migration-regression) when measurement behavior changes. Reference projects remain read-only; identities, paths, profiles, counts, and reports stay exclusively in Git-ignored `docs/research/local/`.
