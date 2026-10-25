# Agent Feedback on Categorizing Tests with 0.11.1, 2026-10-25

The owner asked agents in three projects to improve their test categorization with Locmetry 0.11.1 ("I would like to have a better categorization (via purposes). Especially the tests …"), then whether Locmetry could be improved: a12-rulekit, bpmn-lean-experiment and a private Lean project. These are the same three projects as the [0.11.0 round](experiments-2026-10-25.md), so requests repeated here were made after that round's changes shipped. The feedback is leads; each claim was checked against 0.11.1 before anything was recorded, and the claims that only agent behavior can settle were tested in the experiments below. Private projects' names, paths and counts stay in the Git-ignored local folder.

## Findings, as Verified

| Finding | Verification | Result |
| --- | --- | --- |
| A language mapping that never applies is silent (private Lean project: `*.spec` mapped to Plain Text, already read as RPM Specfile) | Reproduced: no `--verbose` note, no `summary.unused_entries` entry, and mappings were absent from `summary.policy_entries` | Fixed: project mappings are audited, files the counter recognizes are `preempted` by `builtin:recognition`, and `--verbose` names the mapping with the correction remedy |
| The configure guide's list of what `summary.policy_entries` covers | It named overrides, rules and corrections, omitting refinements | Fixed with the mapping audit |
| Refinements are hard to discover (a12-rulekit) | `--help` names them only through the section ID `refine-categories`, at line 226 of 279 | Not reproduced: in [experiment 1](#experiment-1-discovering-refinements) all 12 agents found and used refinements |
| Categories follow tags or declarations, not names (a12-rulekit: JUnit `@Tag`; bpmn-lean-experiment: theorems among definitions) | Name globs needed four hand-kept exceptions in a12-rulekit; in [experiment 2](#experiment-2-a-field-sized-project) every agent read test sources for tags, and four of seven still labelled untagged unit tests by name | Evidence for content predicates on refinements; bpmn-lean-experiment adds a design, a category attribute in `.gitattributes` that the project's own tooling writes |
| `except` lists repeat other entries' globs (bpmn-lean-experiment, private Lean project) | The verbose note on entries that lost files to an earlier entry recommends `except` even for a deliberate catch-all, and copies of the same globs drift | New [DEFERRED](../../DEFERRED.md#configuration-and-categories) item |
| A refinement filter on the deciding stage (a12-rulekit, bpmn-lean-experiment) | Without it, a12-rulekit rebuilt the JVM test-class convention in `except` | Second request for the [DEFERRED](../../DEFERRED.md#configuration-and-categories) item |
| `preempted` and `displaced` give counts only; an unused pattern gives no reason (a12-rulekit) | Reproduced: a pattern for a symbolic link that selection does not follow is listed as unused without a cause | [DEFERRED](../../DEFERRED.md#configuration-and-categories) |
| Categories of data and documentation are missing from tables | All three projects tallied them from `--files` JSON | Evidence for the [DEFERRED](../../DEFERRED.md#configuration-and-categories) item |
| Plain data counts as documentation (a12-rulekit: `.api` dumps; private Lean project: TSV tables) | Mapping to Plain Text makes such files documentation; in experiment 2, three of seven agents named this as their reason to leave a TSV unmapped and its override dead | Evidence for the deferred display name on mappings |
| A stale report was read after a failed command (bpmn-lean-experiment) | `--tables-only --verbose` fails with exit 1; with stderr merged into `sed`, the error vanished and the previous saved report was read | Evidence for [lesson 1](../../agent-lessons.md#output-and-options); the conflict itself is principled |
| A per-file comparison was written by hand on a single root (bpmn-lean-experiment) | The configure guide links the transition recipe; the agent did not reach it. In experiment 2, every agent joined two `--files` reports in a script | Evidence for the [per-file comparison](../../PLAN.md#next) |
| "Purposes are fixed; use categories" appears only in a guide section (a12-rulekit, private Lean project) | `--help` never says it; configure guide line 32 does | Not acted on: experiment 1 found the distinction anyway |
| `explain` with several paths prints JSON Lines undocumented (private Lean project) | `explain --help` documents it twice | No change |
| Mapping and correction need a guide note (private Lean project) | The configure guide explains both; the missing piece was the signal, now fixed | No further change |
| The categories guide lists source gates under `verification-tooling` (a12-rulekit) | It did not say that a gate written as a JUnit class is a test by convention | Fixed in the categories guide |
| Unclaimed documentation and data are silent (private Lean project) | `review` lists source only, by design; documentation reaches the fallback as a matter of course, so listing it would flood the review | No change to `review`; a completeness count is [DEFERRED](../../DEFERRED.md#configuration-and-categories) |
| `--strict`, hypothetical paths in `explain`, `--git-tracked` (private Lean project) | `jq -e` already gates on JSON findings; `explain` of a missing path fails; Git-tracked selection would add a Git dependency the project excludes | Not pursued |

Confirmed working in 0.11.1: the error for an undeclared reference category, which states the reference meaning; `summary.policy_entries`, praised in all three projects; the verbose policy notes, `explain` and measuring a draft with `--config`.

## Experiment 1: Discovering Refinements

Whether 0.11.1's help leads an agent to refinements. A synthetic Gradle project with 21 tests that built-in conventions decide; a `laws/` folder mixing law, differential and agreement tests that only class names separate; guard, benchmark, helper and `integrationTest` files. The prompt was the owner's field prompt naming those kinds, with "learn about locmetry only from its own output". Each session ran headless (`claude -p`) in a neutral temporary path with only the binary on `PATH`, at commit `bc4ffe0` (0.11.1 help). Two policy variants: rules only, with a `perf` rule a convention preempts; and rules with an override that already sets a test category, to tempt copying overrides.

| Variant | Agents | Found refinements | Used them for tests | Groups and helpers separated, purpose totals unchanged | Median calls |
| --- | --- | --- | --- | --- | --- |
| Dead rule | 3 Sonnet `xhigh`, 3 Opus `high` | 6/6, from the 2nd to 5th result | 6/6 | 6/6 | 9.5 |
| Override template | 3 Sonnet `xhigh`, 3 Opus `high` | 6/6, from the 2nd to 4th result | 6/6, none copied the override | 6/6 | 10.5 |

Opus followed the help route to `--section refine-categories` in all six sessions; Sonnet read the whole configure guide instead and still found refinements. The field miss therefore depends on something these fixtures lack, such as repository size or a truncated read of help.

## Experiment 2: A Field-Sized Project

Where agents spend effort on a project shaped like a12-rulekit: three Gradle modules, 359 files, about 180 test files whose kinds are JUnit tags, mostly matching name suffixes spread across folders. Untagged unit tests in the `laws` and `guard` packages carry `LawsTest` and `GuardTest` names, and product classes such as `GuardrailEvaluator` contain "Guard"; JMH benchmarks in `src/jmh`, which no convention claims; `testFixtures`; `.api` dumps and a TSV the counter does not recognize; a policy of rules and one override, with a case-mismatched benchmark rule. Same prompt, binary `bc4ffe0`. Codex sessions ran `gpt-6.1-sol` at `xhigh` in fresh homes through the runner's credential handling, with the binary behind a logging wrapper; the context audit found no repository or global instructions.

| Agents | Median calls | Median uncached input | Read test sources for tags | Labelled untagged unit tests by name | Joined two `--files` reports in a script | Left the TSV unmapped and its override dead |
| --- | --- | --- | --- | --- | --- | --- |
| 4 Codex | 12 | 51,190 | 4/4 | 3/4 | 4/4 | 4/4 |
| 3 Sonnet `xhigh` | 19 | 87,781 | 3/3 | 1/3 | 3/3 | 3/3 |

Every agent found refinements, claimed the benchmarks, separated the helpers and left product classes named like guards as unit tests. Help, guides and schemas made up 18 to 48 KB of tool output per Codex session and 65 to 106 KB per Sonnet session, which read whole guides where help routes to sections.

## Limits

Feedback came from one session per project in a turn that invited criticism. The experiments are informal: Claude and Codex agents launched from this repository's session, outside the frozen protocol of the [agent regression checks](../../agent-regression-checks.md), with three or four sessions per arm. Fixtures are synthetic, and experiment 2's oracle is the fixture's tags. Transcripts, final policies and scorers are kept outside the repository with the session's scratch evidence.
