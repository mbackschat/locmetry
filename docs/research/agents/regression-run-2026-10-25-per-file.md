# Agent Regression Run, Per-File Comparison, 2026-10-25

A targeted check of [per-file comparison](../../proposals/archive/per-file-comparison.md) (`compare --files`) before its release, as PLAN then required for a new flag and report section. The question: does a native per-file view make agents reviewing a policy change cheaper or more correct, where every agent in the [field-sized experiment](feedback-2026-10-25-b.md#experiment-2-a-field-sized-project) and the [workspace round](feedback-2026-10-25.md#findings-as-verified) wrote its own join of two `--files` reports?

## Design

- Control: `bc4ffe0` (0.11.1 plus the empty-glob fix and documentation), the build agents used in the field-sized experiment; it was not itself checked since `97ff0de`, and both tasks are new. Candidate: the per-file comparison working tree on `0fa0b4c`, committed as `85b5bdd` on the unmerged branch `per-file-comparison`, with help shortened afterwards (`compare --help` 5,327 bytes in the run, 5,140 committed; the control's is 4,733).
- Fixture: the deterministic Gradle-shaped project of the field-sized experiment, 354 counted files. Each arm's own binary saved both reports with `--files`, so reader and reports always match.
- Task 1, single root: the policy change refines tests by class-name suffix and fixes a benchmark glob, 237 files change category. The agent lists each changed file as `PATH`, before and after `purpose:category` in `transitions.txt`, and summarizes the code that moved.
- Task 2, workspace: the same project as member `rules` plus a small `scripts` member. Between captures the policies change, a TSV mapping and a member mapping make three unrecognized files countable, and one law test grows by four lines: 240 changed files. The agent lists each file whose classification or counts changed, or that only one report counts, using `uncounted` for a side that does not count it.
- Oracle: the exact set of lines, computed from the reports' file records. Prompts name neither `compare` nor `--files` and allow only Locmetry's own output as documentation.
- Subjects: Codex `gpt-6.1-sol` at `xhigh`, four sessions per arm and task, in fresh homes with repository documents excluded, launched in balanced pairs; every session shares one instruction fingerprint and none carried repository or global instructions. Sonnet at `xhigh`, two sessions per arm on task 2, as a cross-model probe. The sessions ran with the runner's subject home and context audit, but from an ad hoc launcher rather than a frozen protocol, so these results are not in the trend table and have no confirmation sample.

## Results

| Task | Model | Correct (control → candidate) | Calls, total | Median uncached input | Median context growth | Candidates using `--files` |
| --- | --- | --- | --- | --- | --- | --- |
| 1, single root | Codex | 4/4 → 4/4 | 19 → 19 | 19,841 → 24,724 (+24.6%) | 16,206 → 19,263 (+18.9%) | 4/4 |
| 2, workspace | Codex | 4/4 → 4/4 | 20 → 21 | 27,457 → 31,400 (+14.4%) | 24,621 → 29,896 (+21.4%) | 3/4 |
| 2, workspace | Sonnet | 2/2 → 2/2 | 15 → 15 | cost $0.235 → $0.180 per session (−23%) | cache writes 38,797 → 26,832 tokens (−31%) | 2/2 |

Every session was correct in both arms, including the three state changes, so no join failed. Calls are level. Codex's uncached input fails the 110% bound in both tasks, and context growth rose with it, so provider cache misses do not explain the rise.

## How the Agents Worked

Codex agents in both arms first inspected the raw reports and joined them in one Python call. Candidate agents then also ran `compare --files --format json`, printed its `summary`, `findings` and `file_summary`, and asserted that the native list equalled their own; one wrote its file from the native list after that assertion. The extra context is this cross-check: the help growth, about 150 tokens, is small next to a rise of 3,000–5,000. One candidate on task 2 never called Locmetry.

One Sonnet candidate wrote its file with `jq` directly from the saved `compare --files` output, without its own join; the other joined the reports itself after reading the native table. Both Sonnet controls wrote Python joins.

## Verdict

The native view does not replace the join for Codex: a join of two file lists is one cheap, correct call, so agents keep it and use `compare --files` as a check, which costs more. Sonnet's two-session sample points the other way. Correctness and calls do not separate the arms on these tasks.

The candidate raised the help-size budgets for the option's text, which is not the cost driver. Releasing the feature would have been a decision about value for reviewers, humans included, rather than about agent efficiency, which this run does not show for Codex; see [agent-facing design lesson 9](../../agent-lessons.md#reports-and-vocabulary).

Decision, 2026-10-26, under the owner's delegation: rejected, because agent efficiency is the priority and the feature fails Codex's cost gate without improving correctness or calls. It was never merged; the implementation stays on the branch `per-file-comparison`, and [REJECTED](../../REJECTED.md#tested-and-rejected) keeps the reopening condition.

## Limits

- Four Codex pairs per task and two Sonnet pairs; ratios are diagnostic, not established.
- Only one task shape: writing a per-file list. A review that needs only the transition table, or reports too large to print, was not tested.
- The launcher is not the frozen runner, so the protocol has no frozen gates or confirmation sample.
- Evidence, prompts, oracles and per-session summaries stay with the operator; the binaries' SHA-256 prefixes are `603876a040747adb` (control) and `9503617120e0da8a` (candidate).
