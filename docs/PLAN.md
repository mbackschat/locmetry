# Development Plan

[PRODUCT.md](PRODUCT.md) owns requirements, including optional and excluded scope; this file tracks remaining work and its completion evidence. Implemented work is recorded in Git history. Unchecked items are planned work, not supported features.

## Measurement Quality and Net Change — Complete

Sequence accepted 2026-10-15. The [actionable work plan](plans/measurement-quality-and-net-change.md) owns detailed tasks, retained rationale, source provenance, acceptance checks, review checkpoints and live stage status. Implement in this order: notebook correctness, documentation corrections, shared report/schema contracts, net change, then selection explanations. S1 notebook correctness/failure reporting, S2 documentation corrections, S3 typed report/schema contracts, S4 net change and S5 selection explanations are complete. The public suite, formatting, Clippy, executable README recipe, schema/documented examples and private replay pass; implementation checkpoints have not been pushed or released. Optional follow-ups remain conditional in that plan.

The detailed plan records stage decisions and verified completion. Update affected documentation with each implementation checkpoint; S2 is the wider documentation audit, not a reason to delay documentation for S1. [PRODUCT.md](PRODUCT.md) continues to describe supported behavior; the [historical net-change decision record](proposals/net-change.md) preserves S4 rationale and acceptance criteria. Conditional design work belongs in the detailed plan.

External review of `4ca626a` is [recorded with dispositions](plans/measurement-quality-and-net-change.md#external-review-disposition--2026-10-15). The review established a shared notebook decoder with end-to-end fixtures; Tokei retains cell-content counting. The plan now distinguishes technical prerequisites from work order and records historical-report compatibility, missing failure evidence and review boundaries. S1 subsequently implemented those corrections; its tests and compatibility evidence are recorded in the detailed plan.

The [follow-up review corrections](plans/measurement-quality-and-net-change.md#follow-up-review-corrections--2026-10-15) address missing-path diagnostics, configuration defaults, measurement help and false positives in the README workflow guard. Release preparation remains separate.

## Human Output Hierarchy — Complete

Implemented muted legends/separators, bold structure, signed-change colors and warning/error emphasis through one [semantic palette](../src/style.rs). Measurement, workspace, comparison and explanation output share the presentation roles. Markdown uses portable emphasis; redirected output and JSON receive no styling escapes. The [presentation contract](PRODUCT.md#human-presentation) owns stream behavior. Custom theme configuration remains outside scope; the palette provides a small implementation boundary without new dependencies.

Verification: renderer guards preserve plain-text content and alignment after removing styling, check root/workspace legends, signed changes and unavailable explanations, and preserve Markdown without ANSI. The initial separator and Markdown-legend checks failed before implementation. The full public suite and subsequent focused renderer tests pass, as do Clippy and executable README workflows. A local pseudo-terminal probe verifies real terminal styling, `NO_COLOR`, `TERM=dumb`, pipe parity, JSON and independent stderr warning/error styling. No measurement semantics changed; private counting replay was not rerun.

## Positional Scan Directory — Complete

Implemented 2026-10-16: `locmetry DIR` selects the same root as `--root DIR`, including root configuration discovery. Command names retain precedence independently of directory existence; `./compare` or `--root compare` measures a directory named like a command. [Project configuration](PRODUCT.md#project-configuration) owns the selection contract. Root arguments before subcommands remain rejected, and scripts can use `--root` to avoid reserved names.

Verification: positional-root equivalence and command-precedence regression tests failed before their fixes and pass afterward. Coverage includes relative/absolute paths, spaces, configuration discovery, explicit command-name paths and conflicting roots. The full public suite, formatting, Clippy and executable README workflows pass. No counting behavior changed; private replay was not rerun.

## Development Release — Pending

The S1–S5 changes and review corrections identify as `0.2.0-dev.1`, distinct from the unchanged v0.1.6 tag. They are unreleased; no push, tag, release or tap update is authorized by this plan.

- [ ] When a release is requested, choose the final version, rerun release checks, publish the new binaries and update the tap and README stable installation together. Include the notebook counting/coverage changes, saved-report compatibility, net change and explanation commands in release notes.

## Correctness Follow-ups — Complete

Implemented 2026-10-16: distinguish Handlebars access failures (`unread`) from readable but undecodable content (`parse_failures`, reason `invalid_encoding`). Handlebars and notebooks share the BOM-aware decoding boundary and failure type, preserving notebook reason codes. Submodule hints now parse named sections, quoted/escaped values, inline comments and continuations, instead of collecting every literal `path =` line. [Measurement semantics](measurement.md#counting) and [submodule hint semantics](measurement.md#git-submodule-hints) own the supported boundaries; includes are not followed and malformed entries remain best-effort omissions.

Verification: encoding and submodule fixtures failed before the corrections. Detected/mapped templates, BOM preservation, unreadable files, explanation schema/results, quoted submodule report attribution and malformed/unrelated entries now pass. Git's own parser independently confirmed the valid synthetic submodule fixture. Full public tests, Clippy, formatting, README workflows and the private frozen-count replay pass; the private replay guards existing records, while new scanning fixtures establish the fixes. These follow-ups do not activate the [conditional follow-ups](plans/measurement-quality-and-net-change.md#conditional-follow-ups-and-preserved-rationales).

## Release for macOS and Linux — Initial Distribution Complete

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

## Subfolder Measurement — Complete

Approved and implemented 2026-10-16: `--subfolder DIR` restricts a single-root measurement while preserving project policy, module context and path identities. Shared selection, spelling checks and diagnostic probes also serve `explain`; reports record the restriction, comparisons require matching restrictions, and human folder views start at the selected folder. [PRODUCT](PRODUCT.md#subfolder-measurement) owns the contract. Ancestor discovery and workspace restrictions remain excluded; per-file comparison and other conditional features remain deferred.

Verification: initial command and saved-receipt guards failed before implementation. Sixteen focused tests now cover record/breakdown parity, independent counts, context/coverage separation, filtered and empty directories, path identity, zero-code modules, scoped findings, restricted comparisons, output protection, descriptor removal, schema validation and human views. The full public suite, formatting, Clippy, private replay and executable README troubleshooting workflows pass. Private replay guards existing measurement behavior; synthetic scanning cases establish the feature. Verification was local on macOS, not Linux CI. [Detailed evidence](plans/measurement-quality-and-net-change.md#subfolder-implementation--2026-10-16) records limits and documentation ownership. The external implementation review of `799b65f` confirmed the counting, selection and comparison design. Its diagnostic and documentation corrections are recorded in the [review follow-up](plans/measurement-quality-and-net-change.md#subfolder-review-follow-up--2026-10-16); the following refactor checkpoint shares module ownership between summary generation and saved-report validation.

## Known Display Follow-up

- Terminal note escaping: the unclaimed-source folder notes can print control characters embedded in path names. The new subfolder title uses the shared plain-label sanitizer, but older note formatting needs its own focused correction and regression guard. JSON path identities must remain unchanged.

## Completed Design Decisions

- [Subtree measurement — historical decision record](proposals/subtree-measurement.md): retained design rationale and acceptance criteria; [PRODUCT](PRODUCT.md#subfolder-measurement) owns the implemented contract.

- [Net change between measurement reports — historical decision record](proposals/net-change.md): aggregate/topology comparison implemented as `0.2.0-dev.1`; retained for S4 acceptance evidence and design rationale. [PRODUCT](PRODUCT.md#net-change) owns supported behavior. The [conditional per-file follow-up](plans/measurement-quality-and-net-change.md#per-file-comparison) owns the deferred design and consumer-demand trigger.

## Ideas — Not Scheduled

- [ ] A configuration skill for coding agents: decided 2026-10-15 to record, not build yet. Blind test drives showed agents already reach 99%+ agreement with human-reviewed configurations from `--help` and `--schema` alone, so the skill would add procedure rather than knowledge: keep the repository read-only until a human approves, require evidence and a `reason` for every entry, declare `[categories]`, resolve `summary.submodules` and `summary.unused_entries`, and end with the judgment calls a human must make. Acceptance: the same blind trials scored with and without the skill show a gain, not only more ceremony.

## Working Rules

Contributor commands and conventions are in [CLAUDE.md](../CLAUDE.md). Run the [private regression](measurement.md#private-regression) when measurement behavior changes, and keep this plan current as work lands.
