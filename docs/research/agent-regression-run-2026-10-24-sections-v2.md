# Agent Regression Run: Guide Sections, Amendment 1

Evaluated 2026-10-24 as the rerun of the [guide sections proposal](../proposals/guide-sections.md) after its Amendment 1, under the [agent regression checks](../agent-regression-checks.md#agent-runs). Control A is `bd820b7` with the binary of the [first sections run](agent-regression-run-2026-10-24-sections.md) (SHA-256 `c448807f…`). Candidate B is `7c63005`: repeated `--section` in one command, case-insensitive ids, a configuration route in `--help` that names the sections directly, and a closing pointer to the task index (`fbeb88b8…`). Subjects were fresh Codex sessions on `gpt-6.1-sol` at `xhigh`, acknowledged by the owner before launch; Codex 0.164.0 listed no newer model in that family. Answers were graded blind by one fresh Claude Code `sonnet` agent at `xhigh`, asked for case-specific reasons from the start.

The [frozen protocol](../../tests/fixtures/agent-explorability/regression-2026-10-24-sections-amendment-study.json) repeats the first sections run's tasks, schedule, gates, control and confirmation rule; only the candidate and the session environment changed. For the first time, every session ran in a fresh home, as after a new Codex install, holding only a copy of the credentials: the earlier runs had given every session the operator's global instructions and 44 personal and plugin skills, about 37,000 characters. All 32 sessions show the same context fingerprint, the four bundled system skills and no injected instructions. Because the environment changed, ratios compare only within this run.

A first freeze of this protocol carried a mistyped candidate revision and was stopped within a minute, before any session completed; it was frozen again from scratch, and the aborted campaign is kept with the local evidence.

## Verdict

Correctness holds: all 32 answers and all 32 artifact checks pass, and the telemetry audit passed 469 of 472 checks with status admissible, the other three being the cost gates below. Three gates fail. Task 1's median calls rose from 2 to 4 against a bound of 3; task 12's median uncached input rose from 21,450 to 26,503 tokens against a bound of 23,594; and total calls rose from 77 to 88 against a bound of 77. The confirmation sample did not activate, because calls gates failed.

The candidate otherwise read less. Total uncached input fell 20.9%, from 405,406 to 320,555 tokens, and the context the sessions built fell 30.0%, from 323,297 to 226,174. Task 8 fell from 27,783 to 17,951 median uncached tokens, lower in all four pairs, the task the first sections run had failed; task 3 from 29,421 to 22,247. Total session time fell 9%. All four candidate task-12 runs again gave the seven files the seven reference names; the control produced three different name sets.

## Results

| Task | Passes A → B | Median calls A → B | Median uncached input A → B (range) | Median context growth A → B |
| --- | --- | --- | --- | --- |
| 1, measure and interpret a project | 4/4 → 4/4 | 2 → 4 | 13,128 → 11,873 (10,922–13,242, 11,717–13,845) | 10,528 → 9,259 |
| 3, configure a web project | 4/4 → 4/4 | 5.5 → 6 | 29,421 → 22,247 (24,860–40,888, 12,288–29,497) | 24,634 → 13,444 |
| 8, configure a Rust project | 4/4 → 4/4 | 5.5 → 6.5 | 27,783 → 17,951 (23,278–45,303, 13,529–27,661) | 24,169 → 14,192 |
| 12, name categories in a Rust project | 4/4 → 4/4 | 5.5 → 6.5 | 21,450 → 26,503 (19,424–43,926, 21,104–33,767) | 20,527 → 17,201 |
| All runs | 16/16 → 16/16 | 77 → 88 total | 405,406 → 320,555 total | 323,297 → 226,174 total |

The [portable results](agent-regression-2026-10-24-sections-v2-results.json) hold the per-run metrics, the diagnostic and the naming result.

## How the Candidate Read

Configuration sessions followed the help route exactly. Every candidate session in tasks 3 and 8 read `--guide configure --section decide-purpose-not-just-coverage`, two of them adding `refine-categories`, and `--schema config --section Entry --section Scope`: two commands, where every control session read the configure guide and the whole schema twice. None opened the index or read a schema whole; one task-8 run requested an unknown section to list the ids, then read three more. In task 12, every candidate read `refine-categories` and `choose-a-name` as two commands, as the route names them, and one or two schema commands, three or four reads where the control read the configure guide and the schema in one or two.

Task 1 is where the calls went. Every candidate run read `--help`, then `--guide index`, then two sections of the reports guide, where every control run read `--help` and the whole reports guide; in the first sections run, one of four candidates had opened the index. Amendment 1 changed the closing line of `--help` from "For workflows and recipes beyond these examples, `locmetry --guide index` names the guide section or schema definition to read" to "For other workflows and recipes, start with `locmetry --guide index`", which reads as an instruction to an agent with an unrelated task. Candidate runs also explained the opaque file, which no control run did, and two of them first placed `--root` before `explain`, which fails, and recovered with one more call. Task 1 accounts for 7 of the 11 extra calls.

Task 12's uncached-input failure is not reproduced by the diagnostic: its context growth fell 16%. Three of its four pairs cost more uncached input, by 1,680, 12,560 and 2,209 tokens, one cost 14,822 less; the candidate spent one more call per session on the categories guide and paid more in requests the provider did not serve from cache.

## Secondary Flags

| Task | Metric | Control | Candidate |
| --- | --- | ---: | ---: |
| 1 | shell executions | 3.5 | 7 |
| 1 | Locmetry invocations | 3 | 6 |
| 1 | cached input | 44,416 | 89,024 |
| 1 | output tokens | 871 | 1,242 |
| 1 | seconds | 31 | 44 |
| 1 | guide retrievals | 1 | 2 |
| 12 | guide retrievals | 1 | 2 |
| 12 | schema retrievals | 1 | 2 |

Retrievals count each command that reads a guide or schema, so section reads raise them by design.

## Limits

Four pairs per task make the ratios diagnostic, not statistically established. The context-growth diagnostic measures whole-session growth, not the cost of one document. The fresh home makes this run's absolute figures incomparable with earlier runs, so the improvement over the first sections run cannot be separated from the environment change; only the ratios within this run count. The cause named for task 1 rests on the reading pattern and the changed sentence, not on a run that varies it alone. The blind reviewer saw the category names and commands quoted in answers, which differ between arms, so blinding to the condition was partial; names were not graded. Prebuilt arm source revisions are operator declarations; the audit verifies binary hashes, not reproducible builds. Raw sessions and the campaign are kept in Git-ignored `docs/research/local/agent-explorability/2026-10-24-sections-v2/`.
