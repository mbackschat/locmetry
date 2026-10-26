# Rejected Designs

Designs decided against, each with the reason and the evidence that settled it. Unlike [DEFERRED](DEFERRED.md), whose items wait for a condition, an item here is reopened only by new evidence that answers its reason, or by the owner's decision. [PRODUCT](PRODUCT.md) owns supported requirements.

## Tested and Rejected

- Per-file comparison, `compare --files`: rejected on 2026-10-26 under the owner's delegation and never merged. Given it, Codex agents kept their own one-call join of the two reports and cross-checked it, failing the uncached-input gate in both tasks of the [agent run](research/agents/regression-run-2026-10-25-per-file.md) at level correctness and calls. The [proposal](proposals/archive/per-file-comparison.md) records the design; the implementation stays on the branch `per-file-comparison`. Reopen only with evidence that agents would drop their own join, such as reports too large to join in one call or joins that get answers wrong, or when people rather than agents need the per-file review. The optional [file-transition helper](../scripts/compare-locmetry-files.sh) ships with releases and Homebrew for single roots.
- A root-help route to the reports-guide section on saving and printing tables: rejected on 2026-10-26 under the owner's delegation. Agents followed it, but the task's uncached input did not fall while calls rose and three cost gates failed ([second route](research/agents/regression-run-2026-10-26.md#second-route-saving-tables)); the candidate stays on the branch `table-routes`. A route pays only where the document it replaces dominates the task's reading.
- Compact JSON for `--schema`: not pursued on 2026-10-26. It would save about a fifth of the configuration schema's tokens, but configuration is occasional, so the saving did not justify an agent campaign ([where the cost goes](research/agents/regression-run-2026-10-26.md#where-the-cost-goes)); the implementation stays on the branch `compact-schemas`.

## Removed

- [Analytics cookbooks](plans/analytics-cookbooks.md): removed 2026-10-21; recreate them, refocused on questions native output cannot answer, only on the owner's decision. The design names the recovery commit and every removed file and pin.

## Out of Scope by Decision

- Ratio checks that fail the exit status, such as `support <= theory`: ratios are neutral and unbanded by the [test/production ratio design](plans/archive/test-production-ratios.md), so a threshold belongs to the caller; `jq -e` over the JSON gates.
- A counter of Locmetry's own, including a mapping that counts every line as code for a DSL without a matching counter: out of scope while Tokei counts. Language mappings and corrections choose among Tokei's counters.
- Ancestor configuration discovery: outside accepted scope, as the [measurement plan](plans/measurement-quality-and-net-change.md#conditional-follow-ups-and-preserved-rationales) records; configuration is read from the scan root only.
- A `catch_all` flag that silences the verbose note on entries that lost files to an earlier entry: it would hide duplicated globs instead of removing them. [DEFERRED](DEFERRED.md#configuration-and-categories) keeps entry references in `except`, which would remove them.
- Windows on ARM builds, excluded while x86_64 emulation serves it; the [Windows support record](proposals/archive/windows-support.md) owns the rationale.
