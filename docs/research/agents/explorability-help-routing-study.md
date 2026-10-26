# Agent Explorability: Execution-First Help Routing

Study started 2026-10-20 after the owner authorized testing whether help can make the direct execution path clear enough to avoid unnecessary reference reading. The [previous help-example experiment](explorability-help-recipe-study.md) found that every configuration explorer still read the guide and schema. This study tests explicit conditional routing; it does not assume those references caused the earlier behavior.

## Finding

Follow-up: the [bounded replication](explorability-routing-replication.md#finding) subsequently passed both adverse cost checks and supports narrow adoption of this unchanged candidate. The original results and recommendation below remain historical evidence.

Explicit conditional routing improved the covered task: all four C agents skipped both configuration references, compared with none in either control, while delivering correct results. Median agent calls fell from 7.5 in current help and 6.5 with the example alone to 5; median uncached input fell by 37.4% and 34.7%, respectively. All 36 sessions passed artifact checks and independent interpretation review.

Keep this direction as a supported candidate, but do not adopt this exact root-help treatment yet. Broader configuration needed one more median call and 8.0% more median uncached input than current help, failing the frozen guard. Unrelated report understanding passed its main guard, but its cached-input median exceeded the example-only control by 21.2%, also crossing the secondary threshold. Aggregate savings do not erase these per-task failures.

## Frozen Design

The [protocol](../../../tests/fixtures/agent-explorability/help-routing-study.json) fixes 36 new MEDIUM sessions: three tasks × three conditions × four repetitions. Actual model and effort are `gpt-6.1-sol` / `xhigh`. Earlier runs are excluded. A is current help; B is the previous [configuration example](../../../tests/fixtures/agent-explorability/help-recipe.txt); C is exactly B plus the [routing paragraph](../../../tests/fixtures/agent-explorability/help-routing.txt) immediately before that example. Guides, schemas, short help, subcommand help and all measurement behavior remain unchanged. C−B isolates the complete routing paragraph; C−A compares the complete candidate with current help.

The paragraph tells an agent whose change is covered to inspect existing configuration, measure current decisions, adapt the example and verify the result. It explicitly makes initial guide/schema reading optional for that route, while directing exclusions, overrides, intentional Other and uncovered semantics to references. Baseline measurement preserves the existing configure workflow's evidence requirement. This is an explicit exception to guide-first advice for a covered task; the guide itself is unchanged. No new guide-navigation interface is introduced.

| Task | Role | Oracle |
| --- | --- | --- |
| [9: simple rule](../../../tests/fixtures/agent-explorability/task9.json) | Primary efficiency/mechanism test; the example covers its configuration syntax. | Source Files 3, Lines 5, Code 4; Production/Test/Other Code 2/1/1; only `notes/probe.py` unresolved. |
| [8: broader configuration](../../../tests/fixtures/agent-explorability/task8.json) | Guard against bypassing references when the recipe is insufficient. | Preserve inferred Production and conventional tests, classify operations and intentional Other, exclude vendor, retain unresolved scratch; exact saved decisions and counts. |
| [1: report understanding](../../../tests/fixtures/agent-explorability/task1.json) | Unrelated-task correctness and help-reading cost guard. | Source/all-counted totals, Code-based Test/prod, non-source contributions and unavailable unrecognized counts. |

The prompts, fixtures and oracles are unchanged. Correctness depends on artifacts and delivered interpretation, never on whether a guide was read. Each triad uses identical task inputs across its three fresh sessions. Arms execute sequentially within a triad; up to three triads run concurrently. Every arm permutation appears twice across the twelve triads. Within-task order balance is imperfect: Task 1 places B first twice and last twice, never in the middle; the other task/arm combinations occupy each position once or twice. This corrects the preregistration prose's erroneous once-or-twice claim without changing the exact frozen schedule. No outcome-dependent reordering occurred.

The unchanged per-run limits are 600 seconds, 40 agent-facing tool calls, 60 CLI invocations and 100,000 uncached input tokens. Isolated scratch sessions start with binary access and synthetic files only, without repository instructions, prior sessions, source access or expected answers. Actual model/effort, fixture integrity, prompts, source/binary hashes and final counters are audited. Independent reviewers receive shuffled final answers, prompts, oracles and artifact checks without condition/resource labels; answers can reveal their route, so blinding is imperfect.

## Decision Rules Before Execution

Every C run must pass artifact and semantic grading, with no lower pass count on any task than either A or B. Recovered mistakes can pass; all valid failures and recovery costs remain in resource totals.

For Task 9, the same efficiency branch must pass against both A and B: either at least one fewer median agent call with median uncached input no more than 5% higher and at least three of four paired call wins against each comparator; or at least 10% less median uncached input with median calls no higher and at least three of four paired token wins against each comparator. The mechanism signal requires at least three of four C runs to skip both `--guide configure` and `--schema config`, compared with at most one of four B runs. A retrieval change supports the proposed explanation but does not establish causal mediation or identify which sentence caused it.

For Tasks 8 and 1, C median agent calls must not rise and median uncached input must not exceed A by more than 5%. Reference reading in Task 8 is appropriate when needed; it is not a scoring requirement. Report cached input, output including reasoning, shell executions, CLI invocations, guide/schema retrievals and final-delivery duration. On any task, a C median more than 20% above either comparator for any of these secondary cost metrics prevents unqualified adoption. Correctness, efficiency, retrieval mechanism and secondary gates are reported separately.

These are practical diagnostic thresholds, not statistical significance tests. A successful covered-task result would support a narrow workflow claim, not all configuration or all CLI features. Production adoption is a separate decision; this experiment changes only temporary native binaries. An independent MEDIUM preflight checked the design before freezing it, leading to baseline measurement, explicit intentional Other routing, a common efficiency branch across comparators, the exact schedule and a numeric secondary-cost threshold.

## Verification and Replay

The [runner](../../../scripts/agent-routing-study.py) records protocol, harness, runner, grader, task, recipe, routing and binary hashes before sessions launch. The shared harness adds a third isolated arm; `run-02` is A, `run-01` B and `run-03` C. Red/green regressions verify that C differs from B only by routing text, each binary maps to the intended arm, other product sources remain unchanged, and the frozen schedule runs each arm exactly once per triad. The [parity verifier](../../../scripts/verify_help_recipe.py) checks exact help additions, all guides/schemas, four report formats, independent counts and executable recipe behavior.

Preflight verification passed ten Python regressions, exact B/C help parity, unchanged guides/schemas and report formats, the executable example, and both configuration oracles. A/B binary hashes and product source inputs match the previous experiment. C was built once with locked offline dependencies in 38.83 seconds, retaining the log.

Build A, B and C sequentially in a new scratch location, or reuse A/B only after verifying their recorded hashes and unchanged source inputs. Build logs should be retained. The runner requires an authenticated Codex CLI with the recorded model/effort; Locmetry itself gains no runtime dependency. Offline builds require cached dependencies.

```sh
experiment_root=/tmp/locmetry-routing-replay-build
python3 scripts/agent-explorability.py prepare "$experiment_root" --treatment help-routing
cargo build --locked --offline --manifest-path "$experiment_root/A/Cargo.toml" > "$experiment_root/build-A.log" 2>&1
cargo build --locked --offline --manifest-path "$experiment_root/B/Cargo.toml" > "$experiment_root/build-B.log" 2>&1
cargo build --locked --offline --manifest-path "$experiment_root/C/Cargo.toml" > "$experiment_root/build-C.log" 2>&1
python3 scripts/verify_help_recipe.py "$experiment_root/A/target/debug/locmetry" "$experiment_root/B/target/debug/locmetry"
python3 scripts/verify_help_recipe.py "$experiment_root/B/target/debug/locmetry" "$experiment_root/C/target/debug/locmetry" --routing-only
python3 scripts/agent-routing-study.py /tmp/locmetry-routing-replay \
  --binaries "$experiment_root/A/target/debug/locmetry" "$experiment_root/B/target/debug/locmetry" "$experiment_root/C/target/debug/locmetry"
python3 scripts/score_agent_explorability.py /tmp/locmetry-routing-replay
```

Executable artifact checks leave interpretation pending. Independently grade all final answers before assigning outcomes; process success alone is insufficient. Reconcile cumulative final session counters with completion events, count agent calls independently, and exclude evaluator probes using the recorded completion boundary. Evaluator/reviewer work is not explorer usage. Preserve all cases and their costs, not only successful or favorable repetitions. A changed model, task, treatment or source revision is a new campaign; follow the [benchmark revision contract](../../plans/agent-explorability-benchmark.md#revising-or-adding-a-use-case).

## Results

All 36 sessions completed within budget: **12/12 passes in each condition**, with no incomplete or critical-failure runs. Three independent MEDIUM reviewers each graded twelve shuffled answers for one task, with condition/resource labels hidden. Recorded model/effort, context isolation, prompt/task/source/binary hashes, actual arm order and final counters were reconciled. No previous sessions were pooled; no new runs were excluded or replaced. The routing paragraph appeared in the returned tool content of 12 of twelve C sessions. Artifact and semantic outcomes are recorded separately in the per-run results (removed 2026-10-26; `git show c8ad807:docs/research/agents/explorability-help-routing-results.json`).

| Task | Median agent calls A / B / C | Median uncached input A / B / C | Median CLI calls A / B / C |
| --- | --- | --- | --- |
| Simple rule | 7.5 / 6.5 / 5 | 30,981 / 29,710 / 19,388.5 | 6.5 / 6 / 3 |
| Broader configuration | 6 / 8 / 7 | 33,984 / 35,944.5 / 36,696 | 10 / 9.5 / 7.5 |
| Report understanding | 3 / 2.5 / 3 | 20,600.5 / 20,818 / 20,719.5 | 4 / 3.5 / 4 |

A is current help; B adds the example; C adds the routing paragraph to B. Uncached input includes general agent/framework context as well as task and tool content. One agent call can launch multiple shell commands and multiple Locmetry invocations. Output includes reasoning and must not have reasoning added a second time.

| Frozen gate | Result |
| --- | --- |
| Correctness | Pass |
| Simple-task efficiency versus both controls | Pass |
| Retrieval mechanism | Pass |
| Broader-task cost guard | Fail |
| Unrelated-task cost guard | Pass |
| Secondary-cost guard | Fail |
| All adoption gates | Fail |

| Simple-task comparator | C call wins, out of 4 | C uncached-input wins, out of 4 | Call branch | Token branch |
| --- | ---: | ---: | --- | --- |
| A | 4 | 4 | True | True |
| B | 3 | 4 | True | True |

| Task / arm | Configure guide calls | Config schema calls | Sessions skipping both, out of 4 |
| --- | ---: | ---: | ---: |
| 9 / A | 4 | 4 | 0 |
| 9 / B | 4 | 4 | 0 |
| 9 / C | 0 | 0 | 4 |
| 8 / A | 4 | 4 | 0 |
| 8 / B | 4 | 5 | 0 |
| 8 / C | 4 | 5 | 0 |

Retrieval counts include repeats and the whole session, not only pre-edit discovery. A retrieval change is a process observation; correctness remains independently graded.

| Resource, twelve sessions per arm | A | B | C |
| --- | ---: | ---: | ---: |
| Agent-facing calls | 66 | 69 | 59 |
| Shell executions | 137 | 126 | 100 |
| Locmetry invocations | 80 | 76 | 59 |
| Uncached input tokens | 375,539 | 356,129 | 317,044 |
| Cached input tokens | 2,265,472 | 2,444,160 | 1,884,032 |
| Output tokens including reasoning | 31,149 | 34,831 | 31,389 |
| Sum of run durations, seconds | 599.6 | 650.8 | 592.1 |
| Guide retrievals | 16 | 15 | 10 |
| Schema retrievals | 8 | 9 | 5 |

The following C medians exceed a comparator by more than the frozen 20% secondary-cost threshold:

| Task | Comparator | Metric | C increase |
| --- | --- | --- | ---: |
| 1 | B | cached | 21.2% |

## Interpretation and Decision

The experiment supports a narrower, useful conclusion: when help provides both a working example and an explicit conditional path, agents can avoid reference reading on a covered task. All four C simple-task answers were correct without either reference, and every paired comparison used less uncached input than A or B. All four broader-task C agents still consulted both references, so this was selective use rather than a general refusal to read documentation.

Across all twelve sessions per condition, C used 59 agent calls versus A's 66 and B's 69, and 317,044 uncached input tokens versus 375,539 and 356,129. Those are favorable aggregate results, but the frozen adoption rule also protects other workflows. Broader configuration fails that guard even if the cached-token threshold is set aside; the adoption decision does not hinge on one cache fluctuation. C improves its median call count over B on the broader case (7 versus 8) but still exceeds current help (6).

Retain current production help and optional guides. Preserve explicit execution-first routing as the empirically supported part of this candidate, with no claim that this exact wording is ready for general rollout. Any revision needs to retain the covered-task gain while checking the broader-task cost; use fresh controls and a new frozen definition. No additional campaign or product change is started by this conclusion.

## Evidence and Limits

Raw evidence is retained under Git-ignored `docs/research/local/agent-explorability/2026-10-20-help-routing-v1/`: frozen protocol, task definitions, harness and runner, native build/parity logs, manifests, CLI streams, session telemetry, answers, artifacts, executable checks, independent reviews and analysis scripts. Raw sessions can contain framework/account context and are not published. Public definitions and results are portable; copy the ignored archive separately when full provenance is needed elsewhere. Native binaries are identified by hash rather than copying build targets into the archive.

The experiment is restricted to one model/effort, four repetitions per case, small synthetic fixtures and explicit evidence requests. It tests the complete routing paragraph immediately before this particular configuration example. It does not isolate individual sentences, measure human usability, or establish effects for other commands, existing complex policies, or long sessions. Timing reflects concurrent fresh sessions through final delivery; it is not time to first verified success. Cache behavior, within-task order imbalance and stochastic agent trajectories limit the stability of token forecasts. In particular, the Task 1 B-position imbalance adds uncertainty to its cached-input comparison. No monetary saving or statistical significance is inferred.

Verification completed: ten Python regressions, exact native B/C parity and configuration oracles, all 36 executable artifact checks, all 36 independent final-answer reviews, and independent recomputation of metrics and gates. The audit reconciled raw final token/tool/shell/CLI counters, sequential arm order and peak concurrency of three. It caught missing guide/schema retrieval counts in the first secondary-gate evaluator; these were added to implement the already frozen rule, including positive-versus-zero handling. No protocol or trial changed. Documentation indexes/links and Git whitespace checks passed. Production Rust sources and embedded guides remain unchanged.
