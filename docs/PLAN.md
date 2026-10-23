# Development Plan

[PRODUCT](PRODUCT.md) owns supported requirements. This file indexes open work and decision gates; it is not a checkpoint log. Read the relevant linked design only when working on that item. Deferred and unscheduled items live in [DEFERRED](DEFERRED.md) and do not authorize implementation.

## Windows Support — Follow-up

- [ ] Publish the Scoop manifest from [packaging/scoop](../packaging/scoop) in a bucket repository and update it per release like the Homebrew tap; 0.9.0 ships the Windows zip, and `render-manifest.sh 0.9.0` renders a valid manifest. Add the update step to the release handback in `scripts/release.py` once the bucket exists.

## Proposals — Under Discussion

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
- [Test/production ratio design](plans/archive/test-production-ratios.md): research behind neutral, unbanded ratios.
- [Research index](research/README.md): dated findings and source provenance.

Contributor workflow and check selection belong in [CLAUDE.md](../CLAUDE.md). Update this index only for material scope/status changes; do not duplicate historical completion narratives here.
