# Agent Explorability: Execution-First Help Routing

Study started 2026-10-20 after the owner authorized testing whether help can make the direct execution path clear enough to avoid unnecessary reference reading. The [previous help-example experiment](agent-explorability-help-recipe-study.md) found that every configuration explorer still read the guide and schema. This study tests explicit conditional routing; it does not assume those references caused the earlier behavior.

## Frozen Design

The [protocol](../../tests/fixtures/agent-explorability/help-routing-study.json) fixes 36 new MEDIUM sessions: three tasks × three conditions × four repetitions. Actual model and effort are `gpt-6.1-sol` / `xhigh`. Earlier runs are excluded. A is current help; B is the previous [configuration example](../../tests/fixtures/agent-explorability/help-recipe.txt); C is exactly B plus the [routing paragraph](../../tests/fixtures/agent-explorability/help-routing.txt) immediately before that example. Guides, schemas, short help, subcommand help and all measurement behavior remain unchanged. C−B isolates the complete routing paragraph; C−A compares the complete candidate with current help.

The paragraph tells an agent whose change is covered to inspect existing configuration, measure current decisions, adapt the example and verify the result. It explicitly makes initial guide/schema reading optional for that route, while directing exclusions, overrides, intentional Other and uncovered semantics to references. Baseline measurement preserves the existing configure workflow's evidence requirement. This is an explicit exception to guide-first advice for a covered task; the guide itself is unchanged. No new guide-navigation interface is introduced.

| Task | Role | Oracle |
| --- | --- | --- |
| [9: simple rule](../../tests/fixtures/agent-explorability/task9.json) | Primary efficiency/mechanism test; the example covers its configuration syntax. | Source Files 3, Lines 5, Code 4; Production/Test/Other Code 2/1/1; only `notes/probe.py` unresolved. |
| [8: broader configuration](../../tests/fixtures/agent-explorability/task8.json) | Guard against bypassing references when the recipe is insufficient. | Preserve inferred Production and conventional tests, classify operations and intentional Other, exclude vendor, retain unresolved scratch; exact saved decisions and counts. |
| [1: report understanding](../../tests/fixtures/agent-explorability/task1.json) | Unrelated-task correctness and help-reading cost guard. | Source/all-counted totals, Code-based Test/prod, non-source contributions and unavailable unrecognized counts. |

The prompts, fixtures and oracles are unchanged. Correctness depends on artifacts and delivered interpretation, never on whether a guide was read. Each triad uses identical task inputs across its three fresh sessions. Arms execute sequentially within a triad; up to three triads run concurrently. Every arm permutation appears twice across the twelve triads. Within a task, each arm occupies each position once or twice; four repetitions cannot fully balance three positions. The exact schedule is frozen in the protocol, with no outcome-dependent reordering.

The unchanged per-run limits are 600 seconds, 40 agent-facing tool calls, 60 CLI invocations and 100,000 uncached input tokens. Isolated scratch sessions start with binary access and synthetic files only, without repository instructions, prior sessions, source access or expected answers. Actual model/effort, fixture integrity, prompts, source/binary hashes and final counters are audited. Independent reviewers receive shuffled final answers, prompts, oracles and artifact checks without condition/resource labels; answers can reveal their route, so blinding is imperfect.

## Decision Rules Before Execution

Every C run must pass artifact and semantic grading, with no lower pass count on any task than either A or B. Recovered mistakes can pass; all valid failures and recovery costs remain in resource totals.

For Task 9, the same efficiency branch must pass against both A and B: either at least one fewer median agent call with median uncached input no more than 5% higher and at least three of four paired call wins against each comparator; or at least 10% less median uncached input with median calls no higher and at least three of four paired token wins against each comparator. The mechanism signal requires at least three of four C runs to skip both `--guide configure` and `--schema config`, compared with at most one of four B runs. A retrieval change supports the proposed explanation but does not establish causal mediation or identify which sentence caused it.

For Tasks 8 and 1, C median agent calls must not rise and median uncached input must not exceed A by more than 5%. Reference reading in Task 8 is appropriate when needed; it is not a scoring requirement. Report cached input, output including reasoning, shell executions, CLI invocations, guide/schema retrievals and final-delivery duration. On any task, a C median more than 20% above either comparator for any of these secondary cost metrics prevents unqualified adoption. Correctness, efficiency, retrieval mechanism and secondary gates are reported separately.

These are practical diagnostic thresholds, not statistical significance tests. A successful covered-task result would support a narrow workflow claim, not all configuration or all CLI features. Production adoption is a separate decision; this experiment changes only temporary native binaries. An independent MEDIUM preflight checked the design before freezing it, leading to baseline measurement, explicit intentional Other routing, a common efficiency branch across comparators, the exact schedule and a numeric secondary-cost threshold.

## Verification and Replay

The [runner](../../scripts/agent-routing-study.py) records protocol, harness, runner, grader, task, recipe, routing and binary hashes before sessions launch. The shared harness adds a third isolated arm; `run-02` is A, `run-01` B and `run-03` C. Red/green regressions verify that C differs from B only by routing text, each binary maps to the intended arm, other product sources remain unchanged, and the frozen schedule runs each arm exactly once per triad. The [parity verifier](../../scripts/verify_help_recipe.py) checks exact help additions, all guides/schemas, four report formats, independent counts and executable recipe behavior.

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

Executable artifact checks leave interpretation pending. Independently grade all final answers before assigning outcomes; process success alone is insufficient. Reconcile cumulative final session counters with completion events, count agent calls independently, and exclude evaluator probes using the recorded completion boundary. Evaluator/reviewer work is not explorer usage. Preserve all cases and their costs, not only successful or favorable repetitions. A changed model, task, treatment or source revision is a new campaign; follow the [benchmark revision contract](../plans/agent-explorability-benchmark.md#revising-or-adding-a-use-case).

## Results

Pending execution, grading and telemetry reconciliation. The protocol above must remain frozen regardless of intermediate outcomes.
