# Agent Regression Run: Guide Sections

Evaluated 2026-10-24 as the acceptance run of the [guide sections proposal](../../proposals/archive/guide-sections.md) under the [agent regression checks](../../agent-regression-checks.md#agent-runs). Control A is `bd820b7`, the control of both [category-reference runs](regression-run-2026-10-24.md), with the same binary (SHA-256 `c448807f…`). Candidate B is `a0faf26`: the category reference plus `--section`, schema sections, `--guide index` and the help routes (`98c0fad1…`). Subjects were fresh Codex sessions on `gpt-6.1-sol` at `xhigh`, acknowledged by the owner before launch; Codex 0.164.0 listed no newer model in that family. Answers were graded blind by one fresh Claude Code `sonnet` agent at `xhigh`, which was asked to regrade case by case after its first pass gave generic reasons; no outcome changed.

The [frozen protocol](../../../tests/fixtures/agent-explorability/regression-2026-10-24-sections-study.json) ran tasks [1](../../../tests/fixtures/agent-explorability/task1.json), [3](../../../tests/fixtures/agent-explorability/task3.json), [8](../../../tests/fixtures/agent-explorability/task8.json) and [12](../../../tests/fixtures/agent-explorability/task12.json) with four balanced AB/BA pairs each, 32 sessions. Task 1 reads root help for an unrelated purpose, guarding the help additions; the others configure projects. The gates are those of the first category run. New in this protocol, each session reports the context it built, the summed increases of per-request input after the first request, as a diagnostic; and the protocol froze one confirmation sample, to run only if an uncached-input gate failed while every other gate passed and the diagnostic stayed within that gate's bound.

## Verdict

Correctness holds: all 32 answers and all 32 artifact checks pass, and the telemetry audit passed 467 of 469 checks with status admissible, the other two being the cost gates below. Two gates fail. Task 8's median uncached input rose from 30,167 to 34,208 tokens against a bound of 33,183, and total calls rose from 80 to 91 against a bound of 80. The confirmation sample did not activate, because a calls gate failed.

Everything else moved the intended way. Total uncached input fell 11.8%, from 526,216 to 464,361 tokens, and the context the sessions built fell 17.8%, from 306,513 to 251,971. Task 12 fell from 34,768 to 28,419 median uncached tokens, the task the category reference had failed twice; task 3 from 36,867 to 25,185. Total session time fell 7%. All four candidate task-12 runs again gave the seven files the same reference names; the control produced four different name sets.

## Results

| Task | Passes A → B | Median calls A → B | Median uncached input A → B (range) | Median context growth A → B |
| --- | --- | --- | --- | --- |
| 1, measure and interpret a project | 4/4 → 4/4 | 2 → 2 | 21,840 → 21,446 (21,767–35,812, 20,336–22,030) | 10,624 → 10,867 |
| 3, configure a web project | 4/4 → 4/4 | 6 → 7 | 36,867 → 25,185 (19,717–66,074, 22,975–37,905) | 21,187 → 14,139 |
| 8, configure a Rust project | 4/4 → 4/4 | 6 → 7 | 30,167 → 34,208 (29,520–34,925, 27,838–55,727) | 21,995 → 19,257 |
| 12, name categories in a Rust project | 4/4 → 4/4 | 5.5 → 6.5 | 34,768 → 28,419 (15,118–56,000, 22,443–36,592) | 20,461 → 19,070 |
| All runs | 16/16 → 16/16 | 80 → 91 total | 526,216 → 464,361 total | 306,513 → 251,971 total |

The per-task call bounds were the control's median plus one, which every task met. The [portable results](regression-2026-10-24-sections-results.json) hold the per-run metrics, the diagnostic and the naming result.

## How the Candidate Read

Every candidate configuration session used sections; no control session could. All four task-12 runs followed the help routes directly, reading `--guide categories --section choose-a-name` and `--guide configure --section refine-categories`, and two of them read no full schema. In tasks 3 and 8, seven of eight runs opened `--guide index` and then read three to five sections, typically `decide-purpose-not-just-coverage`, `Entry`, `scope` and `Scope`; two task-8 runs still read the whole configuration schema. No candidate configuration session read a whole guide other than the index. In task 1, every control run and three candidate runs read the whole reports guide; the fourth candidate run read the index and two of its sections.

That is where the calls went. The index is one more step, and sections were mostly requested one per command and often one after another, where the control fetched the configure guide and the schema in one batch. Per session the cost was about one call, within every per-task bound, but across 32 sessions it adds up to eleven calls over a total bound that allows none.

Task 8's uncached-input failure is not reproduced by the diagnostic: its context growth fell 12%. Its median rests on runs 1 and 3, 32,161 and 36,255 tokens, whose context growth was below their control pairs'; they paid more in requests the provider did not serve from cache. One candidate run attempted `--no-user-config --guide index`, which the standalone-query rule rejects, and recovered with one more call.

## Secondary Flags

| Task | Metric | Control | Candidate |
| --- | --- | ---: | ---: |
| 1 | output tokens | 843 | 1,136 |
| 1 | seconds | 32 | 40 |
| 3 | Locmetry invocations | 7 | 9 |
| 3 | guide retrievals | 1 | 2 |
| 3 | schema retrievals | 2 | 3 |
| 8 | shell executions | 12.5 | 16.5 |
| 8 | Locmetry invocations | 7 | 9 |
| 8 | guide retrievals | 1 | 3 |
| 12 | guide retrievals | 1 | 2 |

Retrievals count each section request, so more, smaller reads raise them by design. Task 1 read the same help in both arms; its output and time flags rest on four pairs with medians under 1,200 tokens and 45 seconds.

## Limits

Four pairs per task make the ratios diagnostic, not statistically established. The context-growth diagnostic measures whole-session growth, not the cost of any one document. The blind reviewer saw the category names and the commands quoted in answers, which differ between arms, so blinding to the condition was partial; names were not graded. Prebuilt arm source revisions are operator declarations; the audit verifies binary hashes, not reproducible builds. Raw sessions and the campaign are kept in Git-ignored `docs/research/local/agent-explorability/2026-10-24-sections-v1/`.
