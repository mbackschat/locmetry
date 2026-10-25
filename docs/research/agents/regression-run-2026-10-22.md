# Agent Regression Run: 0.8.0

Evaluated 2026-10-22 as the targeted agent run that the [desk review](regression-desk-review-2026-10-21.md) recommended, under the [agent regression checks](../../agent-regression-checks.md#agent-runs). Control A is `c9098e2`, the last version an agent run checked, run from its frozen measured binary (SHA-256 `882fe74a…`). Candidate B is `81e7445`, release 0.8.0 plus the inline-test legend, retired-spelling tips and corrected command summaries (`00f26e75…`). Subjects were fresh Codex sessions on `gpt-6.1-sol` at `xhigh`, the owner's Codex MEDIUM, acknowledged by the owner before launch; Codex listed the model with that effort. Answers were graded blind by one fresh Claude Code `sonnet` agent at `xhigh`.

The [frozen protocol](../../../tests/fixtures/agent-explorability/regression-0.8-study.json) ran tasks 3, 4 and 8, the configuration and comparison tasks the desk review flagged, with four balanced AB/BA pairs each: 24 sessions. Gates are those of the full-help study: no loss of correctness; per task, median calls at most the control's plus one and median uncached input at most 110%; across all runs, total calls at most the control's and uncached input at most 105%.

## Verdict

Correctness holds: all 24 answers and all 48 artifact checks pass in both arms. Cost regressed on the comparison task, through a gap in the report contract rather than in help: measurement documents name their projection with `detail`, comparison documents did not. The [correction](#correction) adds `detail` to comparisons and removes the mechanism; its strict total-calls gate still fails by one call, which the owner accepted as recorded rather than rerunning.

## Results

| Task | Passes A → B | Median calls A → B (range) | Median uncached input A → B |
| --- | --- | --- | --- |
| 3, configure a web project | 4/4 → 4/4 | 6.5 → 6 (6–7, 5–7) | 32,573 → 32,976 |
| 4, compare before and after a change | 4/4 → 4/4 | 7 → **9.5** (6–8, 8–11) | 36,792 → 39,608 |
| 8, configure a Rust project | 4/4 → 4/4 | 5.5 → 6 (5–7, 5–7) | 29,887 → 32,705 |
| All runs | 12/12 → 12/12 | 77 → **86** total | 391,285 → **437,911** total |

Bold values exceed their gate: task 4's median calls (bound 8), total calls (bound 77) and total uncached input (bound 410,849). Every candidate run of task 4 made more calls than its paired control run. The telemetry audit passed all 362 integrity checks; its status is admissible with failed cost gates. The [portable results](regression-0.8-results.json) hold the per-run metrics.

The desk review's risk F1 did not materialise: in tasks 3 and 8 every agent in both arms ran `--verbose` once, and no answer acted on the largest-module note. Risk F2, the grown compare guide, played no part: no task-4 agent in either arm read a guide.

## Task 4 Mechanism

Both arms made exactly 7 Locmetry calls in every task-4 run; the extra calls were the agents' own verification scripts. Since schema 3, measurement documents begin with `document_type` and `detail: "full"`, while comparison documents carried `document_type` alone, although comparisons also have full and headline projections. In all four candidate runs and in no control run, the agent saw `detail` in `before.json`, read `change['detail']`, failed with `KeyError: 'detail'` and rewrote its verification. The control's reports carry neither field, so its agents never made the assumption.

## Correction

Commit `142645e` writes `detail: "full"` or `"headline"` on every comparison, optional in the schema because schema 3 grows by optional fields. A [separate protocol](../../../tests/fixtures/agent-explorability/regression-0.8-correction-study.json) reran task 4 alone with four balanced pairs against the same control; no earlier trial was pooled or replaced.

| Gate (task 4) | Control `c9098e2` | Candidate `142645e` | Bound | Result |
| --- | ---: | ---: | ---: | --- |
| Median calls | 7 (5–9) | 7.5 (6–8) | 8 | Pass |
| Median uncached input | 38,064 | 32,831 | 41,870 | Pass |
| Total calls | 28 | 29 | 28 | Fail by one |
| Total uncached input | 152,771 | 152,622 | 160,410 | Pass |

No session in either arm raised a script error; paired call differences were +1, −2, +3 and −1. Blind review passed all four candidate answers and three of four control answers: one control answer stated that the 20-byte append was 19 bytes, which the grading rule counts as a wrong delivered figure. The audit passed all 142 integrity checks; the [portable results](regression-0.8-correction-results.json) hold the details.

The owner adopted the correction with the residual recorded: no mechanism remains, the per-task gates pass, and further trials would require a new candidate under the frozen stopping rule.

## Instrumentation Failure

A first 24-session campaign was frozen inside the repository. Codex's workspace-write sandbox allows writes only below the run, `/tmp` and `$TMPDIR`, so the launcher could not record calls and every session failed on its first Locmetry call. That campaign is inadmissible and retained only locally; commit `3106564` makes the runner refuse such destinations, and the reported campaigns ran below `/tmp`.

## Limits

Four runs per arm make the ratios diagnostic, not statistically established; a total-calls gate without tolerance fails on a one-call swing in an eight-session run. The candidates were debug builds from `git archive` declared by the operator; binary hashes are verified, builds are not reproduced. One reviewer graded each packet. The telemetry audits are the comparison-routing audit adapted by the operator rather than independently written; one scope check was narrowed to accept a session's own absolute root, which an agent had copied from report metadata into a verification script. Tasks 3 and 8 were not rerun after the correction, whose change touches only comparison output.

## Open Point

Future protocols could allow the total-calls gate a tolerance of one call for correction runs of a single task, matching the per-task median gate. Frozen protocols keep their rules.

## Reproduce

Freeze each protocol with the [runner](../../../scripts/full-help-study.py) below `/tmp`, supplying the control's frozen binary and a candidate built from its commit, then launch with `--run-frozen --acknowledge gpt-6.1-sol:xhigh`, which runners after this study require. Grade artifacts with the [full-help grader](../../../scripts/score_full_help.py). Raw sessions, review packets, analysis and audit scripts remain in Git-ignored `docs/research/local/agent-explorability/`.
