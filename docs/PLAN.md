# Development Plan

[PRODUCT](PRODUCT.md) owns supported requirements. This file indexes open work and decision gates; it is not a checkpoint log. Read the relevant linked design only when working on that item. Deferred and unscheduled items live in [DEFERRED](DEFERRED.md) and do not authorize implementation; designs decided against live in [REJECTED](REJECTED.md).

## Next

None at present.

## Accepted — Awaiting Verification

None at present.

The [proposal index](proposals/README.md) distinguishes active designs from historical decisions.

## Historical Evidence

- Accepted proposals implemented on 2026-10-23, in the owner's order: [framework layouts](proposals/archive/framework-layouts.md), [module credit by layout coverage](proposals/archive/module-credit-by-coverage.md), [language corrections](proposals/archive/language-corrections.md) and [generated-code markers](proposals/archive/generated-code-markers.md); each archived record links its verification.
- [CLI option clarity and applicability](proposals/archive/cli-option-clarity.md): reviewed naming and output-view contract, owner amendment and verification.
- [Windows support](proposals/archive/windows-support.md): reviewed platform contract and its verification.
- [Correct counting in a Tokei fork](plans/archive/tokei-fork.md): the fork's six steps, completed 2026-10-31 with fork `15.0.0+locmetry.43`; [Tokei defects](tokei-defects.md) owns what it corrects.

- [Complete CLI help coverage](plans/archive/full-help-coverage.md): implementation scope and experiment evidence; [help design](help-design.md) owns the current feature inventory and contributor guidance.
- [Default report output decision](proposals/archive/default-report-output-proposal-redacted.md): reviewed design and acceptance evidence.
- [Development and release snapshot through 2026-10-16](plans/archive/development-plan-2026-10-16.md): completed work, published releases and their verification links.
- [Measurement quality and subsequent implementation checkpoints](plans/archive/measurement-quality-and-net-change.md): S1–S5, reviews, accepted decisions and completion evidence.
- [Manual page, completions and reference topic](proposals/archive/manual-and-reference.md): `--generate`, the man page, shell completions and the reference guide, implemented 2026-10-24 and verified 2026-10-25; the record links the shell checks and the desk review.
- [Guide sections and task index](proposals/archive/guide-sections.md): `--section`, the task index and direct help routes, adopted 2026-10-24 with the category reference after three agent runs; the record links the runs and the decision.
- [Category reference](proposals/archive/category-reference.md): the embedded category guide, its guards and the aligned model policies, implemented 2026-10-24.
- [Per-file comparison](proposals/archive/per-file-comparison.md): `compare --files`, rejected on 2026-10-26 after its [agent run](research/agents/regression-run-2026-10-25-per-file.md) and never merged.
- [Single-format options select their format](proposals/archive/single-format-options.md): table, Markdown and file-detail options choose their format without `--format` or `--output`, decided 2026-10-25 from the categorization feedback and [experiments](research/agents/experiments-2026-10-25.md).
- [Reviewable policy entries and category refinements](proposals/archive/policy-entry-review.md): `except`, per-entry audit and refinements, implemented 2026-10-24 from three projects' test-categorization feedback.
- [Test/production ratio design](plans/archive/test-production-ratios.md): research behind neutral, unbanded ratios.
- [Research index](research/README.md): dated findings and source provenance.

Contributor workflow and check selection belong in [CLAUDE.md](../CLAUDE.md). Update this index only for material scope/status changes; do not duplicate historical completion narratives here.
