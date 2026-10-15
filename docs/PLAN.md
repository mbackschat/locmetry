# Development Plan

[PRODUCT.md](PRODUCT.md) owns requirements, including optional and excluded scope; this file tracks remaining work and its completion evidence. Implemented work is recorded in Git history. Unchecked items are planned work, not supported features.

## Measurement Quality and Net Change — Complete

Sequence accepted 2026-10-15. The [actionable work plan](plans/measurement-quality-and-net-change.md) owns detailed tasks, retained rationale, source provenance, acceptance checks, review checkpoints and live stage status. Implement in this order: notebook correctness, documentation corrections, shared report/schema contracts, net change, then selection explanations. S1 notebook correctness/failure reporting, S2 documentation corrections, S3 typed report/schema contracts, S4 net change and S5 selection explanations are complete. The public suite, formatting, Clippy, executable README recipe, schema/documented examples and private replay pass; implementation checkpoints have not been pushed or released. Optional follow-ups remain conditional in that plan.

The detailed plan records stage decisions and verified completion. Update affected documentation with each implementation checkpoint; S2 is the wider documentation audit, not a reason to delay documentation for S1. [PRODUCT.md](PRODUCT.md) continues to describe supported behavior; the [net-change proposal](proposals/net-change.md) remains the detailed design reference.

External review of `4ca626a` is [recorded with dispositions](plans/measurement-quality-and-net-change.md#external-review-disposition--2026-10-15). The review established a shared notebook decoder with end-to-end fixtures; Tokei retains cell-content counting. The plan now distinguishes technical prerequisites from work order and records historical-report compatibility, missing failure evidence and review boundaries. S1 subsequently implemented those corrections; its tests and compatibility evidence are recorded in the detailed plan.

## Release for macOS and Linux — Next

Decided 2026-10-15: licence MIT (dual MIT OR Apache-2.0 up to v0.1.1), repository `github.com/mbackschat/locmetry`, an own tap `mbackschat/homebrew-tap` before any homebrew-core submission, and targets macOS arm64 plus Linux x86_64 and aarch64. [Tokei integration and distribution](tokei-integration.md#distribution) describes the release path.

- [x] Add licence files and Cargo metadata.
- [x] Format, lint, and test on all three targets in [CI](../.github/workflows/ci.yml).
- [x] Build release archives on tags with an empty-environment measurement and, on Linux, a static-linkage check in the [release workflow](../.github/workflows/release.yml).
- [x] Write the [formula](../packaging/homebrew/locmetry.rb): prebuilt release archives per platform, a source build only for `--HEAD`, and a test that measures a synthetic project with an empty `PATH`. Prebuilt archives follow the tap's existing convention and spare users a Rust toolchain (decided 2026-10-15).
- [x] Create the GitHub repository, push, and confirm CI passes on both Linux targets (2026-10-15).
- [x] Tag `v0.1.0` and verify the release archives and `SHA256SUMS`; the downloaded macOS archive passes its checksum and measures with an empty environment (2026-10-15).
- [x] Publish the rendered formula to the tap and add Homebrew installation to the README; `brew install`, `brew test`, and `brew linkage` pass on macOS arm64 (2026-10-15).
- [x] Verify `brew install` and `brew test` on Linux x86_64 and aarch64: both pass in the `homebrew/brew` container (Ubuntu 22.04), and `brew linkage` lists no libraries (2026-10-15). Homebrew there also installs its own gcc and glibc, because the host's are older than it requires for formulae without bottles; publishing bottles would avoid that.

**Completion evidence:** CI passes on every target, the release carries archives and checksums, `brew install` and `brew test` pass on macOS arm64 and Linux, and publication was explicitly authorized.

## Workspace Mode — Done

Decided 2026-10-15: a `workspace` subcommand with repeatable `--root` or a workspace file that measures each member with its own configuration in parallel and merges the results, as [PRODUCT.md](PRODUCT.md#workspace-mode) defines. Done 2026-10-15: a workspace over four real sibling repositories reproduced the sum of their individual runs exactly and took under a second.

- [x] Parse `locmetry workspace` and reject single-root flags and flags before the subcommand.
- [x] Validate members: at least one, unique names, no root twice, no nesting.
- [x] Measure members in parallel, fail with the member's name, and merge prefixed records in name order.
- [x] Report `metadata.members`, the merged summary, and member-qualified notes; extend `--schema report`.
- [x] Read members from `--file`, a TOML workspace file whose roots are relative to it, with `--schema workspace`.
- [x] Turn the proposal into contract text and document the mode in the README.

**Completion evidence:** tests cover flag rejection, member validation, a merge that equals the per-member reports summed with prefixed paths, determinism, and the report schema; a workspace over real sibling repositories, recorded in `docs/research/local/`, matches the sum of their individual runs.

## Per-Repository Breakdown — Done

Decided 2026-10-15: workspace reports break metrics down by member, as [PRODUCT.md](PRODUCT.md#per-repository-breakdown) defines. Done 2026-10-15: on four real sibling repositories each member's totals equal its individual run.

- [x] Report `summary.members` and per-member coverage counts, and extend `--schema report`.
- [x] Show repository rows with test ratio and share in the table, and repository grids in Markdown.
- [x] Note members with unclaimed, unrecognized, or unreadable files.
- [x] Turn the proposal into contract text and update the README.

**Completion evidence:** tests show each member's totals equal its regular run, and the table, grids, and notes on a synthetic workspace; a workspace over real sibling repositories, recorded in `docs/research/local/`, shows the same per-member figures as their individual runs.

## Handlebars Counting Correction — Done

Done 2026-10-15: reproduced Tokei 15.0.0's global-quote bug with independent synthetic templates. A local lexical counter now keeps prose apostrophes from swallowing later blank lines and comments, while preserving actual expression strings, HTML attributes, comments, and raw blocks. Recognized and mapped Handlebars files share the correction; language, kind, and purpose stay unchanged. See [measurement semantics](measurement.md#counting).

**Completion evidence:** regression tests failed before the fix and pass afterward, covering both recognized extensions, language mappings, quote and comment boundaries, raw blocks, BOM decoding, and unreadable-file coverage. The public suite, private regression guard, formatting, and Clippy pass. No release is part of this change.

## Design References

- [Net change between measurement reports](proposals/net-change.md): aggregate/topology comparison is implemented, with a shared generated schema and explicit coverage/settings findings. Optional per-file comparison remains deferred; [PRODUCT](PRODUCT.md#net-change) owns supported behavior.

## Ideas — Not Scheduled

- [ ] A configuration skill for coding agents: decided 2026-10-15 to record, not build yet. Blind test drives showed agents already reach 99%+ agreement with human-reviewed configurations from `--help` and `--schema` alone, so the skill would add procedure rather than knowledge: keep the repository read-only until a human approves, require evidence and a `reason` for every entry, declare `[categories]`, resolve `summary.submodules` and `summary.unused_entries`, and end with the judgment calls a human must make. Acceptance: the same blind trials scored with and without the skill show a gain, not only more ceremony.

## Working Rules

Contributor commands and conventions are in [CLAUDE.md](../CLAUDE.md). Run the [private regression](measurement.md#private-regression) when measurement behavior changes, and keep this plan current as work lands.
