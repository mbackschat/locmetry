# Agent Regression Run: Category Reference

Evaluated 2026-10-24 as a targeted agent run requested by the owner, focused on creating `.locmetryrc`, under the [agent regression checks](../agent-regression-checks.md#agent-runs). Control A is `bd820b7`, the last ledger candidate, built from `git archive` for this run (SHA-256 `c448807f…`). Candidate B is `e15e402`: the [policy entry review](../proposals/archive/policy-entry-review.md) with `except` lists and refinements, and the [category reference](../proposals/archive/category-reference.md) embedded as `--guide categories` (`b4ed3244…`). Subjects were fresh Codex sessions on `gpt-6.1-sol` at `xhigh`, the owner's Codex MEDIUM, acknowledged by the owner before launch; Codex 0.164.0 listed the model with that effort and no newer model in its family. Answers were graded blind by one fresh Claude Code `sonnet` agent at `xhigh`.

The [frozen protocol](../../tests/fixtures/agent-explorability/regression-2026-10-24-study.json) ran three configuration tasks with four balanced AB/BA pairs each, 24 sessions. Tasks [3](../../tests/fixtures/agent-explorability/task3.json) and [8](../../tests/fixtures/agent-explorability/task8.json) are the configuration tasks of the [2026-10-23 run](agent-regression-run-2026-10-23.md); their prompts name the categories, such as `examples` and `operations`. [Task 12](../../tests/fixtures/agent-explorability/task12.json) is new: a Rust crate whose unit tests, binary-running tests, shared helpers, benchmark and three scripts need categories the agent names itself. It gates on purposes, counts and distinct categories for the kinds the prompt asks to tell apart; the names are a diagnostic, kept from the blind review. Its oracle was counted by hand and confirmed with a policy solved in each arm, overrides in the control and refinements in the candidate. Gates are those of the full-help study: no loss of correctness; per task, median calls at most the control's plus one and median uncached input at most 110%; across all runs, total calls at most the control's and uncached input at most 105%.

## Verdict

Correctness holds: all 24 answers and all 24 artifact checks pass in both arms, and the telemetry audit passed 358 of 359 checks with status admissible, the remaining one being the cost gate below. Tasks 3 and 8 and the overall totals pass every cost gate. Task 12 fails one gate: its median uncached input rose from 31,659 to 35,195 tokens, 11.2% against a 10% bound. The candidate's agents read the categories guide, and three of four read it twice; the control has no such guide.

What the candidate bought with it is the naming the reference was built for: all four candidate runs gave the seven files the same names, all of them reference names, while the four control runs produced four different name sets, sharing only `test-support`. Every candidate run expressed the test kinds as refinements; every control run needed three overrides, the only stage it has for relabeling conventional tests.

## Results

| Task | Passes A → B | Median calls A → B (range) | Median uncached input A → B (range) |
| --- | --- | --- | --- |
| 3, configure a web project | 4/4 → 4/4 | 6 → 5.5 (5–6, 5–6) | 32,155 → 32,405 (31,036–36,073, 30,501–33,860) |
| 8, configure a Rust project | 4/4 → 4/4 | 6 → 6 (5–7, 6–7) | 35,316 → 32,560 (32,034–59,092, 21,787–34,899) |
| 12, name categories in a Rust project | 4/4 → 4/4 | 6.5 → 6.5 (6–8, 5–8) | 31,659 → 35,195 (15,594–37,709, 32,829–51,242) |
| All runs | 12/12 → 12/12 | 74 → 73 total | 409,797 → 405,436 total |

The bounds were 7 and 35,371 for task 3, 7 and 38,848 for task 8, 7.5 and 34,824 for task 12, 74 total calls and 430,287 total uncached input. Paired uncached-input differences in task 12 were −1,583, +8,655, −4,880 and +35,648; the last pair also holds the lowest control run of the campaign, 15,594. The [portable results](agent-regression-2026-10-24-results.json) hold the per-run metrics and the naming diagnostic.

Tasks 3 and 8 did not read the new guide: both arms requested one guide, `configure`, per session, and every agent kept the category names its prompt gave. Their cost moved within the noise of earlier runs.

## Category Names in Task 12

| File | Control names (runs) | Candidate names (runs) |
| --- | --- | --- |
| `src/parse/tests.rs` | `unit-tests` (3), `unit-test` (1) | `unit` (4) |
| `tests/cli.rs` | `integration-tests` (3), `integration-test` (1) | `integration` (4) |
| `tests/common/mod.rs` | `test-support` (4) | `test-support` (4) |
| `benches/throughput.rs` | `benchmarks` (3), `benchmark` (1) | `performance` (4) |
| `scripts/release.sh` | `release-tooling` (3), `release-automation` (1) | `distribution` (4) |
| `scripts/dev-setup.sh` | `developer-setup` (3), `developer-tooling` (1) | `tooling` (4) |
| `ci/run-tests.sh` | `ci-tooling` (3), `ci-automation` (1) | `verification-tooling` (4) |

Even within one model and one prompt, the control's names varied in number and wording. Several repeat the purpose (`unit-tests`, `integration-test`), and others are names the reference lists as replaced (`benchmark`, `developer-tooling`, `ci-tooling`).

## Secondary Flags

| Task | Metric | Control | Candidate |
| --- | --- | ---: | ---: |
| 12 | Locmetry invocations | 6.5 | 8.5 |
| 12 | guide retrievals | 1 | 3 |
| 12 | schema retrievals | 1 | 2 |

The repeated retrievals have one mechanism. Agents fetched the configure guide, the categories guide, the configuration schema and the fixture in one parallel tool call; the combined output exceeded the harness's display cap ("truncated output, original token count 17,081" in one session), so the agent fetched the categories guide again, often with a projection of the schema. The 10 KB guide appeared to push that batch over the cap; the [correction run](#correction-run-shorter-guide) found repeated reads with a 6.5 KB guide too. The configuration schema also grew from 32,893 to 36,481 bytes with the refinement and audit descriptions, read once or twice per session in both arms.

## Correction Run: Shorter Guide

To test whether the guide's length caused task 12's repeated retrievals, the guide was shortened from 10 KB to 6.5 KB and task 12 rerun on a [new frozen protocol](../../tests/fixtures/agent-explorability/regression-2026-10-24-correction-study.json): four AB/BA pairs, the same control, candidate `15b9c56` (`271cfb74…`), the same model and effort, acknowledged by the owner before launch, and one fresh blind reviewer as above. Its gates are those of the first run, with the overall call bound raised by one as for a single-task correction. The first run's trials are not pooled with it.

Correctness holds again: all eight answers pass the blind review and all eight artifact checks, and the telemetry audit passed 140 of 142 checks with status admissible. The naming result repeats: all four candidate runs chose the same seven reference names, while the control produced four different name sets; this time it named the tests alike (`unit-tests`, `integration-tests`, `test-support`, `benchmarks`) and varied only on the scripts. Calls pass, 27 → 26 in total, but both uncached-input gates fail by more than before: task 12's median rose from 28,388 to 45,576.5 tokens against a bound of 31,227, and the total from 149,227 to 174,092 against 156,688. Paired differences were −20,033, +4,670, +21,733 and +18,495. Shortening did not stop the repeated reads: the candidate retrieved the categories guide twice in two runs, once batched with the configure guide and the schema into 59.5 KB of output.

Per-request token counts in the session logs separate two causes. The context each session built for itself, the sum of the increases of per-request input after the first request, grew from a median of 21,176 to 24,557 tokens, +16.0%; recomputed for the first run's task 12, from 23,527 to 26,653, +13.3%. (An earlier version of this note subtracted the first request's cached input instead, which varies between sessions with cross-session cache hits, and reported +10.5% and +7.8%; an independent review of the [guide sections proposal](../proposals/guide-sections.md) found the flaw on 2026-10-24.) This is whole-session context growth, not the guide's isolated cost: it includes the guide itself, a configure guide and schema that grew with refinements, repeated reads and everything else the sessions did differently. The rest of the gate failure is prompt-cache misses at the provider, requests billed as uncached although their prefix had been sent before: three candidate sessions missed 11,000 to 14,000 tokens each, and one control session, the control's maximum of 65,316, missed about 26,000. Misses depend on the provider, not on the binary, and four pairs cannot average them out. This decomposition is a diagnostic; the frozen gates stand as failed. The [portable results](agent-regression-2026-10-24-correction-results.json) hold the per-run metrics and the naming diagnostic; raw sessions are kept in Git-ignored `docs/research/local/agent-explorability/2026-10-24-regression-categories-correction-v1/`.

## Limits

Four pairs per task make the ratios diagnostic, not statistically established. Task 12 is new, so it has no earlier series; its naming diagnostic shows determinism for one fixture and one model, not for every project. The blind reviewer saw the names each answer chose, which differ visibly between arms in task 12, so blinding to the condition was partial for that task; names were not graded. Before launch, the operator ran each arm binary of `task12-rep1` once with `--guide categories` from the pair folder to confirm the arm mapping, and the launcher logged both calls; they precede the launch marker, no explorer saw them, and every CLI and guide figure above excludes them. Prebuilt arm source revisions are operator declarations; the audit verifies binary hashes, not reproducible builds. Raw sessions and the campaign are kept in Git-ignored `docs/research/local/agent-explorability/2026-10-24-regression-categories-v1/`.
