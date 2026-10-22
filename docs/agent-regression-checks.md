# Agent Regression Checks

Locmetry's help, guides, flags and report fields are an interface for agents as much as for people. This document says when and how to check, on demand, that a change has not made ordinary agent use less correct or more expensive, and records each check. [CLI help design](help-design.md) owns what good help is; the [agent explorability benchmark](plans/agent-explorability-benchmark.md) owns the experiment contract, harness and historical studies these checks reuse.

## Levels

Choose the cheapest level that can answer the question, and escalate only on its findings.

| Level | When | Cost | Establishes |
| --- | --- | --- | --- |
| Desk review | Before a release whose help, guides, flags, defaults or report fields changed since the last checked version | One MEDIUM agent | Stale routes and flag names, broken or missing examples, help growth, features agents cannot discover, interpretation changes in default output |
| Targeted agent run | A desk-review finding that only behavior can settle, or an owner request | About 4–8 fresh sessions per affected task | Correctness and cost of the affected tasks against the control |
| Full campaign | A change of the MEDIUM model, or before 1.0 | 40–80 fresh sessions | The whole interface across the frozen tasks |

A desk review cannot show that calls or tokens stayed level; only agent runs measure efficiency. It can show that nothing agents rely on broke, and it decides whether an agent run is worth its cost.

## Control

Compare against the most recent version in the [ledger](#ledger), never against an unchecked one, so "since when" has an answer. Name both sides by release tag or commit and keep them fixed for the whole check.

## Desk Review

Give a fresh MEDIUM agent, resolved through the owner's tier mapping, the two revisions and this section. Unlike an agent-run subject, the reviewer may read the repository.

1. Build each revision into its own scratch directory from `git archive`, without changing the working tree.
2. Capture from both executables, with `NO_COLOR=1`, `COLUMNS=100` and an empty `XDG_CONFIG_HOME`: `-h` and `--help` of the root and every subcommand, every `--guide` topic, every `--schema` name, and the default table, `--verbose` table and JSON headline of a synthetic fixture.
3. Check against the [feature inventory](help-design.md#current-feature-inventory) and [placement rules](help-design.md#what-belongs-where):
   - Every flag, value and command that help, guides, examples or error remedies name exists in the candidate.
   - Each new user-visible feature has an inventory entry and a route from root help, and each removed or renamed one has left no reference.
   - Help examples still run and still answer their task; [executable-example tests](help-design.md#adding-an-example) cover the current ones.
   - Reading cost: the size in lines and bytes of each help text and guide, and what an agent must now read before ordinary use.
   - Default output: any new line, note or changed figure that alters how an agent interprets results.
4. Report each finding with its evidence, a severity (regression, risk or improvement) and whether a targeted agent run should settle it.

## Agent Runs

Use the benchmark's frozen tasks, runners ([six-task](../scripts/agent-explorability.py), [full-help](../scripts/full-help-study.py)) and [replay contract](plans/agent-explorability-benchmark.md#isolation-fixtures-and-replay-contract): blind fresh sessions, identical model, effort, budget and environment in both arms, balanced AB and BA order. Apply the [full-help study's gates](research/agent-explorability-full-help-study.md) unchanged: no loss of correctness, per-task median calls at most the control's plus one, median uncached input at most 110% of the control's, and across all runs at most the control's total calls and 105% of its uncached input. A task whose prompt or oracle must change for the candidate becomes a new revision in both arms.

Resolve MEDIUM through the owner's tier mapping for the CLI that runs the subjects, and record the actual model and effort. The runner drives Codex today. Results from different models are separate campaigns and are never pooled. Freeze and run a campaign below `/tmp`, because the explorer sandbox lets the launcher record calls only there; the runner refuses other destinations. Afterwards archive the raw sessions in Git-ignored `docs/research/local/agent-explorability/` and publish a dated research note with the design, results and limits.

## Help-Size Guard

`cargo test --locked --test integration help::help_stays_within_its_reading_budget` fails when the root or a subcommand help text exceeds its byte budget at `COLUMNS=100`. The budgets are the measured sizes plus about 3%, so factual edits pass while real growth stops for a decision. Size is a stop, not an efficiency measure: raise a budget only after a desk review of the growth, at the level its findings justify, and lower it when help shrinks.

## Ledger

One row per check, newest first. The outcome links the evidence.

| Date | Candidate | Control | Level | Outcome |
| --- | --- | --- | --- | --- |
| 2026-10-21 | `dc2bf33` (0.8.0 plus inline-test legend) | `v0.7.0`, itself unchecked | Desk review, `sonnet` at `xhigh` | No broken flag, example or route; help +13%, guides +27%; targeted run of tasks 3, 4 and 8 recommended; [desk review](research/agent-regression-desk-review-2026-10-21.md) |
| 2026-10-20 | Full-feature help with comparison routing (before 0.7.0) | Adopted configuration routing `c17fece` | Full campaign, `gpt-6.1-sol` at `xhigh` | Adopted after correction; [full-help study](research/agent-explorability-full-help-study.md) |
