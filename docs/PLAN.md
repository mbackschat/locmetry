# Development Plan

[PRODUCT](PRODUCT.md) owns supported requirements. This file indexes open work and decision gates; it is not a checkpoint log. Read the relevant linked design only when working on that item. Deferred and unscheduled items live in [DEFERRED](DEFERRED.md) and do not authorize implementation.

## Next

1. **Per-file comparison**, activated by the owner on 2026-10-25 as the next item: native per-file transitions in `compare`, so a policy review no longer needs its own join of two `--files` reports. Every agent session reviewing a workspace policy change wrote one ([feedback round](research/agent-feedback-2026-10-25.md#findings-as-verified)), and the file-transition helper accepts single-root reports only. Start from the [retained design](plans/measurement-quality-and-net-change.md#per-file-comparison), which adds `compare --files`.
   - Proposal first, settling the human summary of transitions, matching workspace members, and paths counted or known on one side only.
   - Implement test-first, with workspace reports covered.
   - Run a targeted [agent regression check](agent-regression-checks.md) with a per-file review task before the release, as a new flag and report section require.
   - Release it as 0.12.0. Another release that changes commands, reports or configuration goes ahead of it only by the owner's decision.

## Accepted — Awaiting Verification

None at present.

The [proposal index](proposals/README.md) distinguishes active designs from historical decisions.

## Historical Evidence

- Accepted proposals implemented on 2026-10-23, in the owner's order: [framework layouts](proposals/archive/framework-layouts.md), [module credit by layout coverage](proposals/archive/module-credit-by-coverage.md), [language corrections](proposals/archive/language-corrections.md) and [generated-code markers](proposals/archive/generated-code-markers.md); each archived record links its verification.
- [CLI option clarity and applicability](proposals/archive/cli-option-clarity.md): reviewed naming and output-view contract, owner amendment and verification.
- [Windows support](proposals/archive/windows-support.md): reviewed platform contract and its verification.

- [Complete CLI help coverage](plans/archive/full-help-coverage.md): implementation scope and experiment evidence; [help design](help-design.md) owns the current feature inventory and contributor guidance.
- [Default report output decision](proposals/archive/default-report-output-proposal-redacted.md): reviewed design and acceptance evidence.
- [Development and release snapshot through 2026-10-16](plans/archive/development-plan-2026-10-16.md): completed work, published releases and their verification links.
- [Measurement quality and subsequent implementation checkpoints](plans/archive/measurement-quality-and-net-change.md): S1–S5, reviews, accepted decisions and completion evidence.
- [Manual page, completions and reference topic](proposals/archive/manual-and-reference.md): `--generate`, the man page, shell completions and the reference guide, implemented 2026-10-24 and verified 2026-10-25; the record links the shell checks and the desk review.
- [Guide sections and task index](proposals/archive/guide-sections.md): `--section`, the task index and direct help routes, adopted 2026-10-24 with the category reference after three agent runs; the record links the runs and the decision.
- [Category reference](proposals/archive/category-reference.md): the embedded category guide, its guards and the aligned model policies, implemented 2026-10-24.
- [Single-format options select their format](proposals/archive/single-format-options.md): table, Markdown and file-detail options choose their format without `--format` or `--output`, decided 2026-10-25 from the categorization feedback and [experiments](research/agent-experiments-2026-10-25.md).
- [Reviewable policy entries and category refinements](proposals/archive/policy-entry-review.md): `except`, per-entry audit and refinements, implemented 2026-10-24 from three projects' test-categorization feedback.
- [Test/production ratio design](plans/archive/test-production-ratios.md): research behind neutral, unbanded ratios.
- [Research index](research/README.md): dated findings and source provenance.

Contributor workflow and check selection belong in [CLAUDE.md](../CLAUDE.md). Update this index only for material scope/status changes; do not duplicate historical completion narratives here.
