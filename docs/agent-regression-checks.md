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

After the owner acknowledges the resolved model and effort (see [Models](#models)), give a fresh MEDIUM agent the two revisions and this section. Unlike an agent-run subject, the reviewer may read the repository.

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

Use the benchmark's frozen tasks, runners ([six-task](../scripts/agent-explorability.py), [full-help](../scripts/full-help-study.py)) and [replay contract](plans/agent-explorability-benchmark.md#isolation-fixtures-and-replay-contract): blind fresh sessions, identical model, effort, budget and environment in both arms, balanced AB and BA order. Apply the [full-help study's gates](research/agent-explorability-full-help-study.md) unchanged: no loss of correctness, per-task median calls at most the control's plus one, median uncached input at most 110% of the control's, and across all runs at most the control's total calls and 105% of its uncached input. A correction run of a single task allows its total calls one call above the control's, matching the per-task median gate, because a strict total failed on a one-call swing in eight sessions ([2026-10-22 correction](research/agent-regression-run-2026-10-22.md#correction); decided by the owner the same day). Each protocol freezes its gates, so this applies from the next protocol frozen. A task whose prompt or oracle must change for the candidate becomes a new revision in both arms.

Uncached input stays the primary cost gate. From the [guide sections](proposals/guide-sections.md) protocol on, each run also reports the context each session built: the summed increases of per-request input after the first request, which provider cache hits do not affect. A request smaller than its predecessor, as after compaction, adds nothing and is counted. It measures whole-session growth, not the cost of one document, and is a diagnostic, not a gate; the runner records it in each session summary. Because one provider cache miss can decide a four-pair median of uncached input, as the [2026-10-24 correction](research/agent-regression-run-2026-10-24.md#correction-run-shorter-guide) showed, a protocol may freeze one confirmation sample of four more balanced pairs per task. It names the activation rule, the tasks, whether the samples are pooled and the decision rule before launch; `full-help-study.py --run-confirmation TASK …` runs it once, only after the primary sample. This reduces the sensitivity to cache misses without moving the gate, but cannot rule out that they decide an outcome.

The runner drives Codex today. Results from different models are separate campaigns and are never pooled. Freeze and run a campaign below `/tmp`, because the explorer sandbox lets the launcher record calls only there; the runner refuses other destinations. Afterwards move the campaign into Git-ignored `docs/research/local/agent-explorability/` with `mv`, or with `rsync -aH` across disks: pairs share their binaries as hard links, which `cp -R` turns back into copies. Publish a dated research note with the design, results and limits.

## Models

Every agent in a check, whether desk reviewer, subject or blind answer reviewer, runs on the current MEDIUM tier of its CLI, resolved through the owner's tier mapping. The mapping lives outside the repository; when it defines no MEDIUM for that CLI, stop and ask the owner rather than choosing a model.

- A new protocol records the resolved model and effort. The runner reads them only from the frozen protocol, so a later mapping change takes effect with the next protocol.
- Freezing checks Codex's own catalog (`codex debug models`): it refuses a model or effort Codex does not offer and names newer models of the same family. When it names one, ask the owner to update the mapping before launching.
- The owner acknowledges the model and effort before every agent run, including reviews. Launch with `--run-frozen --acknowledge MODEL:EFFORT`; the runner refuses any other value and records the acknowledgement and catalog entry in `started.json`.

## Help-Size Guard

`cargo test --locked --test integration help::help_stays_within_its_reading_budget` fails when the root or a subcommand help text exceeds its byte budget at `COLUMNS=100`. The budgets are the measured sizes plus about 3%, so factual edits pass while real growth stops for a decision. Size is a stop, not an efficiency measure: raise a budget only after a desk review of the growth, at the level its findings justify, and lower it when help shrinks.

## Ledger

One row per check, newest first. The outcome links the evidence.

| Date | Candidate | Control | Level | Outcome |
| --- | --- | --- | --- | --- |
| 2026-10-24 | `e15e402` (`bd820b7` plus the policy entry audit, `except` lists, refinements and the category reference) | `bd820b7` | Targeted agent run, tasks 3, 8 and the new category-naming task 12, `gpt-6.1-sol` at `xhigh` | Correctness 24/24; tasks 3 and 8 and the totals pass every cost gate; task 12 median uncached input +11.2% fails the 10% bound, spent reading the category guide, while all candidate runs named its seven files alike with reference names against four different name sets in the control; [agent run](research/agent-regression-run-2026-10-24.md) |
| 2026-10-24 | `15b9c56` (`e15e402` with a 6.5 KB category guide that keeps owner-requested and finer custom names) | `bd820b7` | Correction of task 12 on a new frozen protocol, `gpt-6.1-sol` at `xhigh` | Correctness 8/8 and calls pass; uncached input fails for task 12 (+61% median) and in total (+17%), mostly provider prompt-cache misses, while the context the sessions built grew 16%; all candidate runs again chose the same seven reference names; [correction run](research/agent-regression-run-2026-10-24.md#correction-run-shorter-guide) |
| 2026-10-23 | `bd820b7` (`be32810` plus the language list shared in the schemas) | `3bec0a3` | Targeted agent run, tasks 3 and 8, `gpt-6.1-sol` at `xhigh` | Correctness 16/16; all cost gates pass, median uncached input +4.0% and −0.9%; three secondary flags from agents' own checking; [agent run](research/agent-regression-run-2026-10-23.md) |
| 2026-10-23 | `be32810` (`3bec0a3` plus framework layouts, module credit, language corrections, generated-code markers, the Go layout, `.locmetryrc` exclusion and the BASH fold) | `3bec0a3` | Desk review, `sonnet` at `xhigh` | No correctness regression; help +0.3%, guides +3.3%, schemas +28.1%, 41% of it a duplicated language list; targeted run of tasks 3 and 8 recommended for the cost gates; [desk review](research/agent-regression-desk-review-2026-10-23.md) |
| 2026-10-22 | `3bec0a3` (`9a46ad3` plus `--table-category`, one-line `-h` summaries and the verification-tooling guidance) | `9a46ad3` | Desk review, `sonnet` at `xhigh` | No regression; help −15.8%, `-h` −40%; no `-h` call in 310 earlier sessions, so no agent run needed; three wording gaps fixed after review; [desk review](research/agent-regression-desk-review-help-split.md) |
| 2026-10-22 | `9a46ad3` (`99bf4d0` plus flags-only notes, named roots, severity labels and member vocabulary) | `99bf4d0` | Desk review, `sonnet` at `xhigh` | No regression: exit statuses and JSON identical apart from one schema description; no agent run needed; stale "repository" prose fixed after review; [desk review](research/agent-regression-desk-review-0.9.0.md) |
| 2026-10-22 | `99bf4d0` (`142645e` plus human output wording) | `142645e` | Desk review, `sonnet` at `xhigh` | Help and schemas byte-identical; no broken flag or exit status; JSON changes only in message texts; quoted whole commands omit the invocation's scope (A2, since fixed by quoting flags only); [desk review](research/agent-regression-desk-review-2026-10-22.md) |
| 2026-10-22 | `142645e` (`81e7445` plus comparison `detail`) | `c9098e2` | Targeted agent run, task 4 correction, `gpt-6.1-sol` at `xhigh` | Correctness 4/4; per-task gates pass; total calls 29 against 28 fails by one, adopted with the residual recorded; [correction](research/agent-regression-run-2026-10-22.md#correction) |
| 2026-10-22 | `81e7445` (0.8.0 plus inline-test legend and help fixes) | `c9098e2` | Targeted agent run, tasks 3, 4 and 8, `gpt-6.1-sol` at `xhigh` | Correctness 24/24; task 4 median calls 7 → 9.5 because comparisons lacked `detail`; cost gates failed; [agent run](research/agent-regression-run-2026-10-22.md) |
| 2026-10-21 | `dc2bf33` (0.8.0 plus inline-test legend) | `v0.7.0`, itself unchecked | Desk review, `sonnet` at `xhigh` | No broken flag, example or route; help +13%, guides +27%; targeted run of tasks 3, 4 and 8 recommended; [desk review](research/agent-regression-desk-review-2026-10-21.md) |
| 2026-10-20 | Full-feature help with comparison routing (before 0.7.0) | Adopted configuration routing `c17fece` | Full campaign, `gpt-6.1-sol` at `xhigh` | Adopted after correction; [full-help study](research/agent-explorability-full-help-study.md) |

## Trend

Each row is one task in one paired check: medians of the control and candidate runs, and their ratio. Only ratios within a campaign are valid; absolute counts from different campaigns are not comparable. A task's series continues while consecutive checks measure it on the same model and effort and each control is the previous candidate; "Since" names the series' first control, and the cumulative columns multiply the step ratios since then. A new model or effort starts a new series. With four runs per arm, ratios are diagnostic, not statistically established.

`python3 scripts/agent-regression-trend.py update` regenerates the table from the portable results in [research](research/README.md); CI fails when it is stale.

<!-- agent-regression-trend:start -->

| Date | Model | Control → candidate | Task | Passes | Median calls | Median uncached input | Since | Calls since | Uncached since |
| --- | --- | --- | ---: | --- | --- | --- | --- | ---: | ---: |
| 2026-10-20 | gpt-6.1-sol `xhigh` | `c17fece` → `c9098e2` | 1 | 4/4 → 4/4 | 2 → 2 (×1.00) | 20,068 → 20,805 (×1.04) | `c17fece` | ×1.00 | ×1.04 |
| 2026-10-20 | gpt-6.1-sol `xhigh` | `c17fece` → `c9098e2` | 4 | 4/4 → 4/4 | 7.5 → 7 (×0.93) | 44,296 → 34,253 (×0.77) | `c17fece` | ×0.93 | ×0.77 |
| 2026-10-20 | gpt-6.1-sol `xhigh` | `c17fece` → `c9098e2` | 5 | 4/4 → 4/4 | 5.5 → 6 (×1.09) | 41,948 → 39,574 (×0.94) | `c17fece` | ×1.09 | ×0.94 |
| 2026-10-20 | gpt-6.1-sol `xhigh` | `c17fece` → `c9098e2` | 7 | 3/4 → 4/4 | 5.5 → 5 (×0.91) | 43,706 → 47,774 (×1.09) | `c17fece` | ×0.91 | ×1.09 |
| 2026-10-20 | gpt-6.1-sol `xhigh` | `c17fece` → `6fdaef3` | 1 | 4/4 → 4/4 | 3 → 2 (×0.67) | 20,802 → 20,714 (×1.00) | `c17fece` | ×0.67 | ×1.00 |
| 2026-10-20 | gpt-6.1-sol `xhigh` | `c17fece` → `6fdaef3` | 2 | 4/4 → 4/4 | 7.5 → 7 (×0.93) | 35,099 → 37,620 (×1.07) | `c17fece` | ×0.93 | ×1.07 |
| 2026-10-20 | gpt-6.1-sol `xhigh` | `c17fece` → `6fdaef3` | 3 | 4/4 → 4/4 | 6.5 → 6 (×0.92) | 32,249 → 31,532 (×0.98) | `c17fece` | ×0.92 | ×0.98 |
| 2026-10-20 | gpt-6.1-sol `xhigh` | `c17fece` → `6fdaef3` | 4 | 4/4 → 4/4 | 6.5 → 8 (×1.23) | 38,992 → 34,919 (×0.90) | `c17fece` | ×1.23 | ×0.90 |
| 2026-10-20 | gpt-6.1-sol `xhigh` | `c17fece` → `6fdaef3` | 5 | 4/4 → 4/4 | 7 → 6 (×0.86) | 60,008 → 37,458 (×0.62) | `c17fece` | ×0.86 | ×0.62 |
| 2026-10-20 | gpt-6.1-sol `xhigh` | `c17fece` → `6fdaef3` | 6 | 4/4 → 4/4 | 4 → 4 (×1.00) | 29,273 → 27,486 (×0.94) | `c17fece` | ×1.00 | ×0.94 |
| 2026-10-20 | gpt-6.1-sol `xhigh` | `c17fece` → `6fdaef3` | 7 | 4/4 → 4/4 | 5.5 → 5 (×0.91) | 50,830 → 47,100 (×0.93) | `c17fece` | ×0.91 | ×0.93 |
| 2026-10-20 | gpt-6.1-sol `xhigh` | `c17fece` → `6fdaef3` | 9 | 4/4 → 4/4 | 5.5 → 6 (×1.09) | 31,352 → 28,942 (×0.92) | `c17fece` | ×1.09 | ×0.92 |
| 2026-10-20 | gpt-6.1-sol `xhigh` | `c17fece` → `6fdaef3` | 10 | 4/4 → 4/4 | 7.5 → 7 (×0.93) | 35,114 → 35,127 (×1.00) | `c17fece` | ×0.93 | ×1.00 |
| 2026-10-20 | gpt-6.1-sol `xhigh` | `c17fece` → `6fdaef3` | 11 | 4/4 → 4/4 | 7.5 → 8 (×1.07) | 38,581 → 38,420 (×1.00) | `c17fece` | ×1.07 | ×1.00 |
| 2026-10-22 | gpt-6.1-sol `xhigh` | `c9098e2` → `142645e` | 4 | 3/4 → 4/4 | 7 → 7.5 (×1.07) | 38,064 → 32,830 (×0.86) | `c17fece` | ×1.00 | ×0.67 |
| 2026-10-22 | gpt-6.1-sol `xhigh` | `c9098e2` → `81e7445` | 3 | 4/4 → 4/4 | 6.5 → 6 (×0.92) | 32,572 → 32,976 (×1.01) | `c9098e2` | ×0.92 | ×1.01 |
| 2026-10-22 | gpt-6.1-sol `xhigh` | `c9098e2` → `81e7445` | 4 | 4/4 → 4/4 | 7 → 9.5 (×1.36) | 36,792 → 39,608 (×1.08) | `c17fece` | ×1.27 | ×0.83 |
| 2026-10-22 | gpt-6.1-sol `xhigh` | `c9098e2` → `81e7445` | 8 | 4/4 → 4/4 | 5.5 → 6 (×1.09) | 29,886 → 32,705 (×1.09) | `c9098e2` | ×1.09 | ×1.09 |
| 2026-10-23 | gpt-6.1-sol `xhigh` | `3bec0a3` → `bd820b7` | 3 | 4/4 → 4/4 | 5.5 → 6 (×1.09) | 33,191 → 34,534 (×1.04) | `3bec0a3` | ×1.09 | ×1.04 |
| 2026-10-23 | gpt-6.1-sol `xhigh` | `3bec0a3` → `bd820b7` | 8 | 4/4 → 4/4 | 7 → 7 (×1.00) | 30,876 → 30,604 (×0.99) | `3bec0a3` | ×1.00 | ×0.99 |
| 2026-10-24 | gpt-6.1-sol `xhigh` | `bd820b7` → `15b9c56` | 12 | 4/4 → 4/4 | 7 → 6.5 (×0.93) | 28,388 → 45,576 (×1.61) | `bd820b7` | ×0.93 | ×1.61 |
| 2026-10-24 | gpt-6.1-sol `xhigh` | `bd820b7` → `e15e402` | 3 | 4/4 → 4/4 | 6 → 5.5 (×0.92) | 32,155 → 32,405 (×1.01) | `3bec0a3` | ×1.00 | ×1.05 |
| 2026-10-24 | gpt-6.1-sol `xhigh` | `bd820b7` → `e15e402` | 8 | 4/4 → 4/4 | 6 → 6 (×1.00) | 35,316 → 32,560 (×0.92) | `3bec0a3` | ×1.00 | ×0.91 |
| 2026-10-24 | gpt-6.1-sol `xhigh` | `bd820b7` → `e15e402` | 12 | 4/4 → 4/4 | 6.5 → 6.5 (×1.00) | 31,658 → 35,194 (×1.11) | `bd820b7` | ×1.00 | ×1.11 |

<!-- agent-regression-trend:end -->
