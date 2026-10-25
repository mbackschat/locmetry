# Agent Studies

How agents use Locmetry's help, guides, reports and errors. [Agent regression checks](../../agent-regression-checks.md) owns the method and the ledger of checks, [agent-facing design lessons](../../agent-lessons.md) distills what these records establish, and the [explorability benchmark](../../plans/agent-explorability-benchmark.md) owns the frozen tasks and harness. The `*-results.json` files are the portable results the [trend table](../../agent-regression-checks.md#trend) is generated from.

## Explorability Studies

- [First agent-explorability pilot](explorability-task1-pilot.md): paired MEDIUM report-understanding runs with and without guides, verified outcomes, token/tool metrics, and replay instructions.
- [Six-task guide-removal study](explorability-six-task-study.md): repeated MEDIUM trials across six workflows plus initial capability discovery, with correctness, token/tool costs, and a guide-retention recommendation.
- [Routing cost replication](explorability-routing-replication.md): both balanced cost checks pass across 54 fresh sessions, supporting narrow adoption of the unchanged candidate.
- [Execution-first help routing](explorability-help-routing-study.md): three-condition results: direct routing reduces covered-task reference reading and cost, but broader-task guard fails.
- [Configuration recipe in help](explorability-help-recipe-study.md): results of the owner's usage-example proposal: two configuration layouts, unrelated-task cost, and a separate simple-rule probe.
- [Full-feature help study](explorability-full-help-study.md): ten-task evaluation, bounded comparison-routing correction, adoption decision and retained resource tradeoffs.

## Regression Checks

- [Agent regression desk review, 0.7.0 to 0.8.0](regression-desk-review-2026-10-21.md): no broken flag, example or route; reading-cost growth and two risks on measured tasks.
- [Agent regression run, category reference](regression-run-2026-10-24.md): tasks 3, 8 and the new naming task 12 against `bd820b7`; correctness 24/24, identical reference names in every candidate run, and task 12's uncached input 1.2 points over its bound; a correction run with a shorter guide repeated the names and failed uncached input by more, mostly through provider cache misses.
- [Agent regression run, guide sections](regression-run-2026-10-24-sections.md): tasks 1, 3, 8 and 12 against `bd820b7`; correctness 32/32 and 11.8% less uncached input, but task 8's uncached input and total calls over their bounds.
- [Agent regression run, guide sections Amendment 1](regression-run-2026-10-24-sections-v2.md): the same tasks in fresh homes; correctness 32/32 and 20.9% less uncached input, but task 1's calls, task 12's uncached input and total calls over their bounds.
- [Agent regression run, printed tables and configuration repair](regression-run-2026-10-25.md): new tasks 13 and 14 against `361f8cf`; correctness 12/12 against 11/12, passing every gate on the pooled sample after task 13's confirmation; Codex passed `--format` itself, so the single-format rule was not exercised.
- [Agent regression run, schema growth](regression-run-2026-10-23.md): tasks 3 and 8 against `3bec0a3`; correctness 16/16 and every cost gate passes once the language list is shared.
- [Agent regression desk review, wrap width and destinations](regression-desk-review-2026-10-25-b.md): no regression and no agent run needed; all four earlier findings and the reports-guide destination sentence fixed and verified, while the terminal-precedence claim rests on dependency behavior no test locks.
- [Agent regression desk review, man page fixes](regression-desk-review-2026-10-25.md): no regression and no agent run needed; all six earlier findings fixed, while the reference guide gives help an 80-column fallback it does not have.
- [Agent regression desk review, man page and completions](regression-desk-review-2026-10-24.md): no regression and no agent run needed; the reference guide misstates where comparisons may be written and the user-level man page path macOS needs.
- [Agent regression desk review, accepted proposals and follow-ups](regression-desk-review-2026-10-23.md): no correctness regression; help flat but schemas +28%, mostly a duplicated language list, so tasks 3 and 8 need a cost run.
- [Agent regression desk review, help split and category filter](regression-desk-review-help-split.md): no regression; in 310 earlier sessions agents never called `-h`, so the shorter `-h` needs no agent run.
- [Agent regression desk review, 0.9.0](regression-desk-review-0.9.0.md): no regression and no agent run needed; workspace output says member, notes quote flags only, stderr names severity.
- [Agent regression desk review, human output wording](regression-desk-review-2026-10-22.md): help and schemas unchanged, JSON changes only in message texts; quoted whole commands omit the invocation's scope.
- [Agent regression run, 0.8.0](regression-run-2026-10-22.md): tasks 3, 4 and 8 against `c9098e2`; a comparison-cost regression traced to the missing comparison `detail`, its correction and the recorded one-call residual.

## Feedback Rounds and Experiments

- [Agent feedback on two workspaces, 2026-10-25](feedback-2026-10-25.md): Codex agents updated two workspaces' policies with 0.11.1 cold and reported on the tool; each finding verified, with the glob fix and DEFERRED evidence it led to.
- [Agent feedback on categorizing tests with 0.11.1, 2026-10-25](feedback-2026-10-25-b.md): the three test-categorizing projects again, each claim verified, the silent-mapping fix, and experiments on discovering refinements and on a field-sized project with Claude and Codex agents.
- [Experiments on the categorization feedback, 2026-10-25](experiments-2026-10-25.md): table options in pipes reproduced and an A/B of six fresh agents, configuration errors repaired by four fresh agents, the shape of content predicates, and a workaround for the content kind of mapped files.
