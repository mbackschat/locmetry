# Development Plan

[PRODUCT](PRODUCT.md) owns supported requirements. This file indexes open work and decision gates; it is not a checkpoint log. Read the relevant linked design only when working on that item. Deferred items do not authorize implementation.

## Next Release — Awaiting Authorization

The [v0.4.0 development notes](releases/0.4.0.md) cover implemented output routing, embedded guides and comparison readability changes. Publication has not been authorized. When requested, apply the [release verification policy](../CLAUDE.md#testing-guidelines) and [distribution procedure](tokei-integration.md#distribution); prior local checks do not replace verification of the release candidate.

## CI Maintenance — Follow-up

- [ ] Update the action versions behind the Node.js runtime-deprecation warnings observed in the v0.3.0 CI/release runs. Treat this as a separate tested maintenance change; the [published-release record](plans/archive/development-plan-2026-10-16.md#minor-release-v030--published) preserves the original evidence.

## Comparison Presence Findings — Follow-up

- [ ] Evaluate a bounded human summary for large folder/member/package restructures, preserving complete JSON evidence. Coverage summaries and source-code row selection are already implemented; [their checkpoint](plans/archive/measurement-quality-and-net-change.md#comparison-readability--2026-10-16) records the separation of scope.

## Known Display Follow-up

- Folder-partition convergence remains deferred: adaptive folder detail and JSON `unclassified_folders` intentionally use different policies. Revisit only if a workflow demonstrates confusing divergence, with explicit compatibility review. The [adaptive-folder decision](proposals/archive/adaptive-folder-detail.md#existing-partition-and-feasibility) owns the rationale.

## Ideas — Not Scheduled

- [Neutral language test/production ratios](plans/test-production-ratios.md): implementation remains on hold. The focused design retains the reviewed scope, research limitations, cross-language caveat and pending acceptance checks.
- [Conditional measurement/comparison capabilities](plans/measurement-quality-and-net-change.md#conditional-follow-ups-and-preserved-rationales): consumer-driven candidates and their triggers, including [per-file comparison](plans/measurement-quality-and-net-change.md#per-file-comparison), generated/minified evidence and duplicate evidence. Ancestor discovery remains outside accepted scope.
- [Guide usability research](plans/archive/measurement-quality-and-net-change.md#embedded-guidance-implementation--2026-10-16): a comparative trial remains optional, not a runtime or release prerequisite; preserve the limits of the existing single-agent trial.
- JSON finding grammar, such as “1 unrecognized files”, remains an optional separate cleanup. The comparison-readability changes intentionally preserved JSON message text.

## Proposals — Under Discussion

None. The [proposal index](proposals/README.md) distinguishes active designs from historical decisions.

## Historical Evidence

- [Development and release snapshot through 2026-10-16](plans/archive/development-plan-2026-10-16.md): completed work, published releases and their verification links.
- [Measurement quality and subsequent implementation checkpoints](plans/archive/measurement-quality-and-net-change.md): S1–S5, reviews, accepted decisions and completion evidence.
- [Research index](research/README.md): dated findings and source provenance.

Contributor workflow and check selection belong in [CLAUDE.md](../CLAUDE.md). Update this index only for material scope/status changes; do not duplicate historical completion narratives here.
