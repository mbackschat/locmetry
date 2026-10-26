# Agent Regression Run, Report-Interpretation Route, 2026-10-26

Where do agents spend their effort, and does the largest daily cost have a cheap remedy? This run first measured the cost in recorded sessions, then tested one change against it with the frozen runner.

## Where the Cost Goes

A read-only pass over about 480 recorded Codex sessions of the explorability and regression campaigns bucketed every tool call by purpose and summed its output. Reading Locmetry's own documentation, `--help`, `--guide` and `--schema`, made up 55–93% of the tool output per task. Scripts over reports, the workaround behind most requests in [DEFERRED](../../DEFERRED.md), were a small share.

In the campaigns since guide sections shipped (2026-10-24 sections v1, v2 and correction, and 2026-10-25), two whole-document reads stood out:

- Every graded session interpreting a measurement (tasks 1 and 13) read the 19 KB reports guide, 35 of 40 whole. The five that read only the two interpretation sections, 3.4 KB, passed alike. Root help routed configuration to guide sections but named only the topic for reports.
- Configuration sessions read the whole configuration schema, about 9,100 tokens, in nearly every session, and agents batching it with help and guides got 51 of 87 such reads truncated. Compact JSON would have saved about a fifth of it. Configuration is occasional rather than daily, so this change was not pursued; the implementation stays on the unmerged branch `compact-schemas`.

## Design

- Control: `aca3fba`, `main` with 0.11.1 and the audited language mappings. Candidate: `b8222e4`, which adds one sentence to root help: "To interpret counts and coverage, read locmetry --guide reports --section interpret-the-numbers --section unavailable-is-not-zero."
- Tasks: 1, interpreting a measurement's counts and coverage, the daily case the route serves; and 13, saving two printed tables, a report task the route should not divert.
- Protocol `agent-regression-2026-10-26-report-routes-v1`, frozen with the [runner](../../../scripts/full-help-study.py): Codex `gpt-6.1-sol` at `xhigh`, four balanced pairs per task in fresh homes, gates and a confirmation sample frozen before launch. Artifacts were checked automatically, and a fresh Sonnet reviewer graded the anonymized answers blind; the operator checked the task 13 table files the reviewer could not see.
- The telemetry audit found the campaign admissible: one context fingerprint for every session and prompt-only product context.

## Results

| Task | Correct (control → candidate) | Median calls | Median uncached input | Median context growth |
| --- | --- | --- | --- | --- |
| 1 | 4/4 → 4/4 | 2 → 2 | 15,493 → 9,603 (−38%) | 10,653 → 7,108 (−33%) |
| 13 | 4/4 → 4/4 | 4 → 4 | 11,457 → 10,966 (−4%) | 9,395 → 9,091 (−3%) |
| All runs, totals | 8/8 → 8/8 | 23 → 26 | 110,004 → 97,294 (−11.6%) | 80,343 → 65,294 (−19%) |

All four task 1 candidates read the two routed sections, and all four controls read the whole guide. Task 13 candidates read the guide as their controls did; one read the index and then four sections, with six agent calls against its control's three but the same five Locmetry calls and less context.

## Verdict

Every correctness gate and every per-task gate passes. Total agent calls, 26 against 23, fail the all-run gate, which allows no increase; the three extra calls come from that one task 13 session, through a route to the index that both arms' help offers. Because a calls gate failed, the frozen activation rule did not run the confirmation sample.

Decision, 2026-10-26, under the owner's delegation: adopted with the calls residual recorded, as for earlier one-call residuals, because the daily interpretation case cost 38% less uncached input in every pair at equal calls and correctness, and total uncached input and context growth fell.

## Second Route: Saving Tables

With the interpretation route, three of four task 13 candidates still read the whole reports guide, while its section on content and destinations is 5 KB. A second sentence, "To save or print tables, read locmetry --guide reports --section choose-content-and-destination.", was tested the same way: protocol `agent-regression-2026-10-26-table-routes-v1`, control `93c64f2` (`main` with the first route), candidate `dd8c4b3` on the unmerged branch `table-routes`, task 13 as the covered task and task 1 as the guard. The audit found the campaign admissible, and the blind review passed all 16 answers.

| Task | Correct (control → candidate) | Median calls | Median uncached input | Median context growth |
| --- | --- | --- | --- | --- |
| 1 | 4/4 → 4/4 | 2 → 2 | 9,640 → 11,098 (+15%) | 7,191 → 7,576 (+5%) |
| 13 | 4/4 → 4/4 | 4 → 4.5 | 14,216 → 13,802 (−3%) | 8,558 → 5,765 (−33%) |
| All runs, totals | 8/8 → 8/8 | 24 → 26 | 97,397 → 103,538 (+6.3%) | 62,047 → 54,942 (−11%) |

All four task 13 candidates read only the routed section, so the context they built fell by a third, but their uncached input barely moved and two of them ran each table command twice, once to view and once to save. Task 1's rise in uncached input came with a 5% rise in context growth, the signature of a provider cache miss. Total calls, total uncached input and task 1's uncached input fail their gates, and the failed calls gate excluded the confirmation sample.

Decision, 2026-10-26, under the owner's delegation: rejected and not merged, because the benefit appears only in the diagnostic while three primary gates fail. A route pays when the document it replaces dominates the task's reading, as the whole guide did for interpretation; for saving tables the guide was a smaller share of a session that spends more on its own commands.

One candidate's first guide call failed: `--no-user-config --guide reports …` exits 1, because guides are standalone queries that reject even that isolation flag, and the agent retried without it.

## Limits

- Four pairs per task; the ratios are diagnostic, not statistically established, and no confirmation sample ran.
- The session mining classified calls by command text and attributed a batched call's output to its main purpose; it locates costs, it does not measure them exactly.
- Root help grew by 131 bytes for every reader; tasks that read help without interpreting reports were not remeasured beyond task 13.
- Raw sessions and the campaign are kept in Git-ignored `docs/research/local/agent-explorability/2026-10-26-report-routes-v1/`; the [portable results](regression-2026-10-26-report-routes-results.json) feed the trend table.
