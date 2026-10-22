# Development Plan

[PRODUCT](PRODUCT.md) owns supported requirements. This file indexes open work and decision gates; it is not a checkpoint log. Read the relevant linked design only when working on that item. Deferred items do not authorize implementation.

## Windows Support — Follow-up

- [ ] Publish a Scoop manifest, updated per release like the Homebrew tap, once a release ships the Windows zip.
- [ ] Generate PowerShell tab completion, together with the other shells; deferred by the owner on 2026-10-21.
- Accept UTF-16 configuration and workspace files, and add PowerShell equivalents of the `jq` recipes, only when Windows users ask for them. Windows on ARM stays excluded while x86_64 emulation serves it. The [Windows support record](proposals/archive/windows-support.md) owns the rationale.

## CI Maintenance — Follow-up

- [ ] Update the action versions behind the Node.js runtime-deprecation warnings observed in the v0.3.0 CI/release runs. Treat this as a separate tested maintenance change; the [published-release record](plans/archive/development-plan-2026-10-16.md#minor-release-v030--published) preserves the original evidence.

## Comparison Presence Findings — Follow-up

- [ ] Decide the [comparison presence summary](proposals/comparison-presence-summary.md) proposal, which bounds human presence findings for large folder/member/package restructures and keeps complete JSON evidence. Coverage summaries and source-code row selection are already implemented; [their checkpoint](plans/archive/measurement-quality-and-net-change.md#comparison-readability--2026-10-16) records the separation of scope.

## Known Display Follow-up

- Folder-partition convergence remains deferred: adaptive folder detail and JSON `unclassified_folders` intentionally use different policies. Revisit only if a workflow demonstrates confusing divergence, with explicit compatibility review. The [adaptive-folder decision](proposals/archive/adaptive-folder-detail.md#existing-partition-and-feasibility) owns the rationale.

## Ideas — Not Scheduled

- [Analytics cookbooks](plans/analytics-cookbooks.md): removed 2026-10-21; recreate them, refocused on questions native output cannot answer, only on the owner's decision. The design names the recovery commit and every removed file and pin.
- [Conditional measurement/comparison capabilities](plans/measurement-quality-and-net-change.md#conditional-follow-ups-and-preserved-rationales): consumer-driven candidates and their triggers, including [per-file comparison](plans/measurement-quality-and-net-change.md#per-file-comparison), generated/minified evidence and duplicate evidence. Ancestor discovery remains outside accepted scope.
- Revisit native file comparison only when recurring usage justifies expanding the CLI. The optional [file-transition helper](../scripts/compare-locmetry-files.sh) ships with releases and Homebrew, while its `jq` prerequisite remains a user-installed choice; avoiding that prerequisite alone does not activate native implementation.
- JSON finding grammar, such as “1 unrecognized files”, remains an optional separate cleanup. The comparison-readability changes intentionally preserved JSON message text.
- [Ineffective-option compatibility policy](proposals/archive/cli-option-clarity.md#deferred-compatibility-decision): blanket rejection remains deferred. Reopen only with a separate compatibility assessment of reusable option sets, terminal-to-pipe transitions and existing script expectations.

- [File-detail option naming](proposals/archive/cli-option-clarity.md#optional-and-excluded-scope): retain `--files`; revisit `--include-files` only if task evidence shows recurring confusion with input selection.

## Proposals — Under Discussion

- [Comparison presence summary](proposals/comparison-presence-summary.md): draft of 2026-10-22, awaiting the owner's decision.

The [proposal index](proposals/README.md) distinguishes active designs from historical decisions.

## Historical Evidence

- [CLI option clarity and applicability](proposals/archive/cli-option-clarity.md): reviewed naming and output-view contract, owner amendment and verification.
- [Windows support](proposals/archive/windows-support.md): reviewed platform contract and its verification.

- [Complete CLI help coverage](plans/archive/full-help-coverage.md): implementation scope and experiment evidence; [help design](help-design.md) owns the current feature inventory and contributor guidance.
- [Default report output decision](proposals/archive/default-report-output-proposal-redacted.md): reviewed design and acceptance evidence.
- [Development and release snapshot through 2026-10-16](plans/archive/development-plan-2026-10-16.md): completed work, published releases and their verification links.
- [Measurement quality and subsequent implementation checkpoints](plans/archive/measurement-quality-and-net-change.md): S1–S5, reviews, accepted decisions and completion evidence.
- [Test/production ratio design](plans/archive/test-production-ratios.md): research behind neutral, unbanded ratios.
- [Research index](research/README.md): dated findings and source provenance.

Contributor workflow and check selection belong in [CLAUDE.md](../CLAUDE.md). Update this index only for material scope/status changes; do not duplicate historical completion narratives here.
