# Agent Regression Run: Printed Tables and Configuration Repair

Evaluated 2026-10-25 as a targeted agent run under the [agent regression checks](../agent-regression-checks.md#agent-runs), after the changes that followed the categorization feedback. Control A is `361f8cf`, the last ledger candidate (SHA-256 `973b290c…`). Candidate B is `97ff0de` (`3fcf7f49…`): the [single-format option rule](../proposals/archive/single-format-options.md), configuration errors that name the likely fix, the legend for language rows of embedded code, the category grid in comparisons and the verbose policy notes. Subjects were fresh Codex sessions in fresh homes on `gpt-6.1-sol` at `xhigh`, acknowledged by the owner before launch; Codex 0.164.0 listed no newer model in that family. Answers were graded blind by fresh Claude Code `sonnet` agents at `xhigh`, asked for case-specific reasons from the start.

The [frozen protocol](../../tests/fixtures/agent-explorability/regression-2026-10-25-study.json) ran two new tasks with four balanced AB/BA pairs each, 16 sessions, and froze a confirmation sample of four more pairs per task. [Task 13](../../tests/fixtures/agent-explorability/task13.json) asks for the category table and the test-only folder table of a small project, saved as printed. [Task 14](../../tests/fixtures/agent-explorability/task14.json) asks for five broken configurations to be repaired to their stated intent: a mappings table under the wrong name, a single `[rules]` table, a lowercase language name, a configuration written for version 2, and `exclude` in an entry. All 16 primary sessions share one context fingerprint. The [informal experiments](agent-experiments-2026-10-25.md) of the same day motivated both tasks.

## Verdict

The candidate passes on the pooled sample, as the protocol's decision rule requires. On the primary sample one gate failed: task 13's median uncached input rose from 13,059 to 18,460 tokens against a bound of 14,365, while every calls gate, the totals and correctness passed, and the context the sessions built stayed flat, 9,331 against 9,443 tokens. The confirmation rule therefore activated for task 13. Over its eight pairs the median uncached input fell from 15,173 to 13,567 tokens; over all 24 sessions, total calls fell from 64 to 59 and uncached input fell 7.7%. No gate fails on the pooled sample.

Correctness: all 12 candidate answers pass, and 11 of 12 control answers. One control session of task 14 stopped at the 600-second wall limit without an answer; its five repaired configurations had already loaded. The telemetry audit of the primary sample passed 249 of 252 checks with status admissible; the other three are that session's two budget checks and the task-13 cost gate.

## Results

| Task | Sample | Passes A → B | Median calls A → B | Median uncached input A → B (range) | Median context growth A → B |
| --- | --- | --- | --- | --- | --- |
| 13, print two tables | Primary, 4 pairs | 4/4 → 4/4 | 4.5 → 4.5 | 13,059 → 18,460 (10,893–22,297, 11,136–29,907) | 9,331 → 9,443 |
| 13, print two tables | Pooled, 8 pairs | 8/8 → 8/8 | 5 → 4.5 | 15,173 → 13,567 (10,893–26,233, 11,136–29,907) | 9,638 → 9,405 |
| 14, repair five configurations | Primary, 4 pairs | 3/4 → 4/4 | 6.5 → 5.5 | 31,789 → 25,916 (21,802–35,322, 22,024–30,507) | 26,258 → 24,051 |
| All runs | Primary | 7/8 → 8/8 | 44 → 42 total | 180,009 → 182,324 total | 144,737 → 137,299 total |
| All runs | Pooled | 11/12 → 12/12 | 64 → 59 total | 255,600 → 235,951 total | 185,222 → 174,619 total |

The [portable results](agent-regression-2026-10-25-results.json) hold the per-run metrics, the pooled analysis the trend reads, and the primary analysis.

## How the Agents Worked

Task 13 did not exercise the single-format option rule. Every one of the 16 sessions in both arms read `--help` and the reports guide first and then passed `--format table` itself; no measurement command in either arm omitted it. The arms ran the same commands, so the task shows that the rule costs Codex nothing, not that it helps it. The informal experiment, in which two of three Claude agents first omitted `--format` and received the headline, remains the only evidence of the benefit, as do the original incidents.

In task 14, both arms first ran every project and met five failures, then read `--help` and the configuration schema. All four candidate sessions still read the whole schema and then sections of it, as every control session did. The hints did not replace reference reading for Codex, unlike the informal repair experiment, where both candidate agents worked from the messages alone. Uncached input was lower in three of four pairs and calls in two, with one equal.

Nine sessions waited 371 to 531 seconds for the model's first response, before running any command, so the arm could not cause it: all four task-13 candidate sessions of repetitions 3, 4, 7 and 8, one candidate and all four control sessions of task 14. They raise task 13's median seconds from 68 to 306, the only secondary flag, and caused the control's wall-limit stop.

## Limits

Four or eight pairs per task make the ratios diagnostic, not statistically established. Task 13 measured an agent that already passes `--format`, so this run cannot confirm the rule's benefit, only the absence of a cost. Tasks 13 and 14 are new and start their trend series here. The telemetry audit covers the primary sample; the confirmation sessions were graded by the artifact checks and a second blind review. The blind reviewers saw answers and artifact verdicts, not the produced files. Prebuilt arm source revisions are operator declarations; the audit verifies binary hashes, not reproducible builds. Raw sessions and the campaign are kept in Git-ignored `docs/research/local/agent-explorability/2026-10-25-regression-v1/`.
