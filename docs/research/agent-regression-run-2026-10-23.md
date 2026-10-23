# Agent Regression Run: Schema Growth

Evaluated 2026-10-23 as the targeted agent run that the [desk review](agent-regression-desk-review-2026-10-23.md) recommended, under the [agent regression checks](../agent-regression-checks.md#agent-runs). Control A is `3bec0a3`, the last ledger candidate, built from `git archive` for this run (SHA-256 `b1fc44c9…`). Candidate B is `bd820b7`: the accepted proposals and follow-ups, with the language-name list shared in the schemas after the desk review's finding D1 (`b4ab875c…`). Subjects were fresh Codex sessions on `gpt-6.1-sol` at `xhigh`, the owner's Codex MEDIUM, acknowledged by the owner before launch; Codex 0.163.0 listed the model with that effort and no newer model in its family. Answers were graded blind by one fresh Claude Code `sonnet` agent at `xhigh`.

The [frozen protocol](../../tests/fixtures/agent-explorability/regression-2026-10-23-study.json) ran tasks 3 and 8, the configuration tasks that read `--schema config` in every recorded earlier session, with four balanced AB/BA pairs each: 16 sessions. Gates are those of the full-help study: no loss of correctness; per task, median calls at most the control's plus one and median uncached input at most 110%; across all runs, total calls at most the control's and uncached input at most 105%.

## Verdict

The candidate passes every gate. All 16 answers and all 16 artifact checks pass in both arms, and the telemetry audit passed all 249 integrity checks with status admissible. The desk review's replay estimate of +18% to +19% uncached input for these tasks did not materialise once the language list was shared: the medians moved by +4.0% and −0.9%. No release blocker remains from the 2026-10-23 desk review.

## Results

| Task | Passes A → B | Median calls A → B (range) | Median uncached input A → B (range) |
| --- | --- | --- | --- |
| 3, configure a web project | 4/4 → 4/4 | 5.5 → 6 (5–8, 5–7) | 33,191 → 34,535 (28,944–72,582, 31,632–39,016) |
| 8, configure a Rust project | 4/4 → 4/4 | 7 → 7 (5–8, 5–8) | 30,876 → 30,604 (23,047–63,108, 28,661–37,852) |
| All runs | 8/8 → 8/8 | 51 → 51 total | 315,814 → 267,438 total |

The bounds were 6.5 and 36,510 for task 3, 8 and 33,963 for task 8, 51 total calls and 331,605 total uncached input. Paired call differences were +1, −2, +2, −1 (task 3) and +1, +3, −1, −3 (task 8). The lower candidate total comes mostly from two control runs, one per task, that used 33,566 and 34,447 more uncached tokens than their paired candidate runs; it is not evidence that the candidate is cheaper. The [portable results](agent-regression-2026-10-23-results.json) hold the per-run metrics.

Every agent in both arms read `--schema config` exactly twice and `--guide configure` once or twice, so the shared language list reached every candidate session: the configuration schema the candidate printed was 32,893 bytes against the control's 28,857, about 8 KB more per session across the two reads.

## Secondary Flags

Three medians exceeded the ×1.2 diagnostic flag; they do not fail the run, and no blanket efficiency claim is made.

| Task | Metric | Control | Candidate |
| --- | --- | ---: | ---: |
| 3 | shell executions | 10.5 | 13.5 |
| 8 | Locmetry invocations | 7 | 9 |
| 8 | harness seconds | 118 | 160 |

The task-8 difference is in the agents' own checking: three of four candidate runs called `explain --help` and one or two `explain` invocations more, against one of four control runs; `explain --help` is byte-identical in both revisions, so no changed text routed them there. One candidate run of task 3 combined `--no-user-config` with `--schema`, which both revisions reject with the same message apart from quoting, and recovered on the next call. The shell flag in task 3 counts the agents' own shell commands around Locmetry, such as reading the saved report.

## Limits

Four pairs per task make the ratios diagnostic, not statistically established. Only tasks 3 and 8 were measured; the desk review records why other tasks need no run, and task 2 (reads the `report` schema) and task 7 (reads every schema) were optional and not run. Prebuilt arm source revisions are operator declarations; the audit verifies binary hashes, not reproducible builds. Raw sessions and the campaign are kept in Git-ignored `docs/research/local/agent-explorability/2026-10-23-regression-schema-v1/`.
