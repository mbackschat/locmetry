# Development Plan

[PRODUCT.md](PRODUCT.md) owns requirements, including optional and excluded scope; this file tracks remaining work and its completion evidence. Implemented work is recorded in Git history. Unchecked items are planned work, not supported features.

## Measurement Quality and Net Change — Complete

Sequence accepted 2026-10-15. The [actionable work plan](plans/measurement-quality-and-net-change.md) owns detailed tasks, retained rationale, source provenance, acceptance checks, review checkpoints and live stage status. Implement in this order: notebook correctness, documentation corrections, shared report/schema contracts, net change, then selection explanations. S1 notebook correctness/failure reporting, S2 documentation corrections, S3 typed report/schema contracts, S4 net change and S5 selection explanations are complete. The public suite, formatting, Clippy, executable README recipe, schema/documented examples and private replay pass; release publication is tracked below. Optional follow-ups remain conditional in that plan.

The detailed plan records stage decisions and verified completion. Update affected documentation with each implementation checkpoint; S2 is the wider documentation audit, not a reason to delay documentation for S1. [PRODUCT.md](PRODUCT.md) continues to describe supported behavior; the [historical net-change decision record](proposals/archive/net-change.md) preserves S4 rationale and acceptance criteria. Conditional design work belongs in the detailed plan.

External review of `4ca626a` is [recorded with dispositions](plans/measurement-quality-and-net-change.md#external-review-disposition--2026-10-15). The review established a shared notebook decoder with end-to-end fixtures; Tokei retains cell-content counting. The plan now distinguishes technical prerequisites from work order and records historical-report compatibility, missing failure evidence and review boundaries. S1 subsequently implemented those corrections; its tests and compatibility evidence are recorded in the detailed plan.

The [follow-up review corrections](plans/measurement-quality-and-net-change.md#follow-up-review-corrections--2026-10-15) address missing-path diagnostics, configuration defaults, measurement help and false positives in the README workflow guard. Release publication is tracked below.

## Human Output Hierarchy — Complete

Implemented muted legends/separators, bold structure, signed-change colors and warning/error emphasis through one [semantic palette](../src/style.rs). Measurement, workspace, comparison and explanation output share the presentation roles. Markdown uses portable emphasis; redirected output and JSON receive no styling escapes. The [presentation contract](PRODUCT.md#human-presentation) owns stream behavior. Custom theme configuration remains outside scope; the palette provides a small implementation boundary without new dependencies.

Verification: renderer guards preserve plain-text content and alignment after removing styling, check root/workspace legends, signed changes and unavailable explanations, and preserve Markdown without ANSI. The initial separator and Markdown-legend checks failed before implementation. The full public suite and subsequent focused renderer tests pass, as do Clippy and executable README workflows. A local pseudo-terminal probe verifies real terminal styling, `NO_COLOR`, `TERM=dumb`, pipe parity, JSON and independent stderr warning/error styling. No measurement semantics changed; private counting replay was not rerun.

## Positional Scan Directory — Complete

Implemented 2026-10-16: `locmetry DIR` selects the same root as `--root DIR`, including root configuration discovery. Command names retain precedence independently of directory existence; `./compare` or `--root compare` measures a directory named like a command. [Project configuration](PRODUCT.md#project-configuration) owns the selection contract. Root arguments before subcommands remain rejected, and scripts can use `--root` to avoid reserved names.

Verification: positional-root equivalence and command-precedence regression tests failed before their fixes and pass afterward. Coverage includes relative/absolute paths, spaces, configuration discovery, explicit command-name paths and conflicting roots. The full public suite, formatting, Clippy and executable README workflows pass. No counting behavior changed; private replay was not rerun.

## Minor Release v0.3.0 — Published

Authorized 2026-10-16 after final external review of the adaptive-folder fixes: publish this verified batch as v0.3.0. [Release notes](releases/0.3.0.md) cover adaptive Folder detail and terminal safety. Neutral language ratios and presentation-option warnings were excluded from this release; the ratio extension remains on hold.

- [x] Set crate/lockfile identity to 0.3.0 and replace unreleased usage wording in README and PRODUCT.
- [x] Verify local public tests, formatting, Clippy, executable README workflows and the release build.
- [x] Push the reviewed changes and confirm CI on every supported platform.
- [x] Publish archives and checksums and verify downloaded artifacts.
- [x] Update and test the Homebrew tap.

Published 2026-10-16: [v0.3.0](https://github.com/mbackschat/locmetry/releases/tag/v0.3.0) points to `a3cefb4`. [CI](https://github.com/mbackschat/locmetry/actions/runs/38908613526) and the [release workflow](https://github.com/mbackschat/locmetry/actions/runs/38909015689) pass on all three supported targets. All downloaded archives match `SHA256SUMS` and carry licenses/notices; the downloaded macOS binary passes independent source/purpose counts, both adaptive-hint outcomes and terminal-safety probes with an empty environment. The Linux release jobs verify static linkage and empty-environment execution. [Tap commit 39fb4c3](https://github.com/mbackschat/homebrew-tap/commit/39fb4c3) updates all three platform URLs/checksums; Homebrew style, the macOS upgrade to 0.3.0 and the formula test pass. A Linux Homebrew installation was not repeated.

## Minor Release v0.2.0 — Published

Authorized by the owner on 2026-10-16: push the reviewed implementation, update GitHub description/topics, and publish the next minor release. Version `0.2.0` replaces the development identity `0.2.0-dev.1`; the v0.1.6 tag stays unchanged. [Release notes](releases/0.2.0.md) describe new commands, subfolder scope, counting/coverage corrections and compatibility boundaries.

- [x] Prepare the final version, stable installation examples and release notes.
- [x] Verify the full local public suite, formatting, Clippy and executable README workflows; keep producer-version and stable-install examples aligned.
- [x] Verify the local release build and GitHub CI on all supported targets.
- [x] Publish v0.2.0 archives and checksums, verify the downloaded macOS executable, and update the Homebrew tap.
- [x] Update the GitHub repository description and topics for measurement, comparison and explanation capabilities.

Published 2026-10-16: [v0.2.0](https://github.com/mbackschat/locmetry/releases/tag/v0.2.0) points to `22e31eb`. [CI](https://github.com/mbackschat/locmetry/actions/runs/38825551746) and the [release workflow](https://github.com/mbackschat/locmetry/actions/runs/38825752736) pass for all three platforms, including license generation, empty-environment checks and static Linux linkage. All three downloaded archives match `SHA256SUMS` and include notices/licenses; the downloaded macOS executable passes independently expected source/purpose counts with an empty environment. [Tap commit e8ac495](https://github.com/mbackschat/homebrew-tap/commit/e8ac4950d45f2f03a0078095cebb033173eaab76) updates all platform URLs/checksums; Homebrew style, macOS upgrade and the formula test pass. Linux package execution was checked by the release workflow; a Linux Homebrew installation was not rerun for this release.

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

## Adaptive Folder Detail — Complete, v0.3.0

Implemented after owner approval on 2026-10-16: `--folder-rows N` gives the single-root terminal Folder table deeper additive detail, with a required positive budget and no default. It preserves initial sibling contributions, collapses single-child chains, splits the largest fitting folder and uses direct-files/remainder rows without overlap. [PRODUCT](PRODUCT.md#adaptive-folder-detail) owns the supported contract; the [checkpoint](plans/measurement-quality-and-net-change.md#adaptive-folder-detail--2026-10-16) records red/green evidence and validation. The option is introduced in v0.3.0.

The full public suite, formatting, Clippy, executable README workflows and private regression pass locally on macOS. Default output, JSON and the unclassified-folder partition remain unchanged. These implementation checks preceded publication; the v0.3.0 release checkpoint records cross-platform CI and distribution verification. Large-tree rendering cost remains unverified. The neutral language-ratio extension stays on hold.

## CI Maintenance — Follow-up

The v0.3.0 CI/release runs report Node.js 20 deprecation warnings for `actions/checkout@v4`, `actions/upload-artifact@v4` and `actions/download-artifact@v4`, which GitHub ran under Node.js 24. All jobs passed. Update these action versions as a separate tested maintenance change; no workflow changes were bundled into this release.

## Known Display Follow-up

- Output routing — implemented and verified as `0.4.0-dev.1`: one destination, explicit formats applied to files, full `json`, compact `headline`, silent exports and ineffective-option warnings. [PRODUCT](PRODUCT.md#output-routing) owns the contract, [v0.4.0 notes](releases/0.4.0.md) describe changed commands, and the [checkpoint](plans/measurement-quality-and-net-change.md#output-routing-redesign) tracks verification. Implementation is authorized; publication is not.

- Help wrapping — corrected in development after v0.3.0: enable Clap's terminal-aware help layout so command and option descriptions retain continuation indentation. [Verification](plans/measurement-quality-and-net-change.md#help-wrapping) records root/subcommand checks; [PRODUCT](PRODUCT.md#human-presentation) owns the behavior.

- Adaptive detail hint — approved and implemented 2026-10-16 with the narrower largest-row trigger: name the largest positive-code row when it is an expandable folder or a remainder. Smaller expandable rows alone do not trigger it, avoiding a routine hint on nearly every nested project. No suggested budget or code-share threshold is added. [PRODUCT](PRODUCT.md#adaptive-folder-detail) owns the contract; [verification](plans/measurement-quality-and-net-change.md#largest-row-budget-hint) records the acceptance checks.

- Terminal output safety — reopened by follow-up review for parse-failure notes, submodule notes and partial-results stderr; corrected 2026-10-16 at the shared note and diagnostic rendering boundaries. Table cells and unclaimed labels retain their earlier guards; JSON evidence remains unchanged. [Verification](plans/measurement-quality-and-net-change.md#terminal-note-and-diagnostic-safety) records the expanded coverage.

- Folder-partition convergence — deferred decision: the [adaptive-folder decision record](proposals/archive/adaptive-folder-detail.md#existing-partition-and-feasibility) differs intentionally from `unclassified_folders`, whose results appear in JSON and agent workflows. The adaptive feature preserves that behavior; share only narrow containment/chain-collapse semantics where practical. Consider policy convergence separately if using both views demonstrates confusing divergence, with explicit compatibility review rather than an incidental renderer refactor.

- Presentation-option consistency — included in the [output routing checkpoint](plans/measurement-quality-and-net-change.md#output-routing-redesign). Warn for explicit ineffective `--categories`/`--depth`, preserve success and output, and use clap’s `ValueSource` to keep defaulted depth silent. `--files` requests data and therefore requires full JSON; `--folder-rows` remains strict. This replaces the earlier error recommendation for harmless presentation flags.

## Proposals — Under Discussion

- [On-demand guides in the binary](proposals/embedded-guides.md) — proposed for external review on 2026-10-16. Revised after external review: four embedded task guides through `--guide TOPIC`, no index or skill, explicit configuration-review outcomes and reuse of troubleshooting. Root help must name the starting command; the [self-audit](plans/measurement-quality-and-net-change.md#embedded-guidance-self-audit--2026-10-16) maps discovery and workflow promises to checks. A shared standalone-query rule requires a separate preceding schema-validation change. [Review disposition](plans/measurement-quality-and-net-change.md#embedded-guidance-review--2026-10-16) records recommendations and evidence; neither change is approved for implementation.

The [proposal index and retirement procedure](proposals/README.md) distinguish active designs from archived decision records.

## Completed Design Decisions

- [Adaptive folder detail — historical decision record](proposals/archive/adaptive-folder-detail.md): implemented for v0.3.0; retained rationale and acceptance criteria. [PRODUCT](PRODUCT.md#adaptive-folder-detail) owns behavior.

- [Subtree measurement — historical decision record](proposals/archive/subtree-measurement.md): retained design rationale and acceptance criteria; [PRODUCT](PRODUCT.md#subfolder-measurement) owns the implemented contract.

- [Net change between measurement reports — historical decision record](proposals/archive/net-change.md): aggregate/topology comparison implemented as `0.2.0-dev.1`; retained for S4 acceptance evidence and design rationale. [PRODUCT](PRODUCT.md#net-change) owns supported behavior. The [conditional per-file follow-up](plans/measurement-quality-and-net-change.md#per-file-comparison) owns the deferred design and consumer-demand trigger.

## Ideas — Not Scheduled

- [ ] [Test to production ratio presentation](plans/measurement-quality-and-net-change.md#test-to-production-ratio-presentation--awaiting-further-direction): review incorporated 2026-10-16; implementation remains on hold. Retain folder/workspace ratios and add neutral language ratios with a total row and cross-language caveat. Keep gray undefined/zero values; use no magnitude-dependent color bands or highlighting. Published ranges are descriptive (Khorikov: `1–3` for heavily tested enterprise projects); the reviewed evidence establishes no validated threshold. The detailed plan preserves corrected source context, retrieval limitations, the rationale for neutral presentation and pending acceptance checks.

The previously deferred configuration-skill idea is superseded by the owner’s 2026-10-16 preference for guidance inside the binary, without an additional skill. The [embedded-guides proposal](proposals/embedded-guides.md) preserves the procedural needs and proposes evaluating their benefit; historical classification trials do not establish that the new guides improve decisions.

## Working Rules

Contributor commands and conventions are in [CLAUDE.md](../CLAUDE.md). Run the [private regression](measurement.md#private-regression) when measurement behavior changes, and keep this plan current as work lands.
