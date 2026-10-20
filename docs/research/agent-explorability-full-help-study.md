# Full-Feature Help Study

The owner clarified that the desired result is a complete agent-facing help experience across supported features, rather than one successful configuration example. This study evaluates the combined interface against the adopted configuration-route baseline. It follows the [feature inventory](../plans/full-help-coverage.md#feature-and-use-case-inventory), [help-design guidelines](../help-design.md) and reusable [benchmark](../plans/agent-explorability-benchmark.md).

## Design Fixed Before Trials

The [protocol](../../tests/fixtures/agent-explorability/full-help-study.json) freezes ten tasks, four independent pairs per task, AB and BA twice each, and 80 sessions in total. Each session uses fresh MEDIUM (`gpt-6.1-sol`, `xhigh`) context, isolated synthetic files and personal defaults, and only the executable's own product documentation. No historical trial is pooled. Three pairs can run concurrently; the two arms within each pair run sequentially.

Control A is `c17fece1c4415f3b871ff9694ef6c30d70ae0074`: the adopted configuration example/routing plus a factual correction shared by both arms, changing workspace-ratio documentation from Lines to Code. Candidate B is `6fdaef3`: task-oriented root notes and additional measurement examples, plus workspace, compare and explain recipes and constraints. Guides, schemas, counting and report behavior remain the same. The candidate is one combined package; this experiment cannot attribute an effect to an individual sentence.

| Native binary | SHA-256 |
| --- | --- |
| A | `79f8d3db22667ab429457d834e1ec036fdf8d18575c8e927fab38af13e490eac` |
| B | `ca6e949b73fa6114f6c04458ca27879914e4c530adc8294c9d9b6f08dd4ed400` |

| Task | Frozen definition | Main oracle |
| --- | --- | --- |
| 1: report interpretation | [Task 1](../../tests/fixtures/agent-explorability/task1.json) | Source/all-counted totals, Code ratio, unknown size |
| 2: JSON and CI | [Task 2](../../tests/fixtures/agent-explorability/task2.json) | One scan, full per-file data, human output, correct complete/partial/failure status |
| 3: broader configuration | [Task 3](../../tests/fixtures/agent-explorability/task3.json) | Exclusion, maintained source, intentional Other, preserved inference and unresolved review |
| 4: saved comparison | [Task 4](../../tests/fixtures/agent-explorability/task4.json) | Captured full baselines, exact mutation/delta, rejected headline, net-change interpretation |
| 5: workspaces | [Task 5](../../tests/fixtures/agent-explorability/task5.json) | File/direct equivalence, member contributions, weighted Code ratio, departure interpretation |
| 6: explanation | [Task 6](../../tests/fixtures/agent-explorability/task6.json) | Selection and classification states, module/rule evidence, member-relative policy |
| 7: discovery | [Revised Task 7](../../tests/fixtures/agent-explorability/task7-full-help.json) | Unchanged 13-family coverage rubric, four valid workflows and four explicitly requested semantic families |
| 9: simple configuration | [Task 9](../../tests/fixtures/agent-explorability/task9.json) | Narrow production rule, preserved test/module decisions and unresolved source |
| 10: scope and views | [Task 10](../../tests/fixtures/agent-explorability/task10.json) | Retained root context, CLI exclusions, hidden/ignore behavior, filtered table and recursive Markdown |
| 11: advanced and personal policy | [Task 11](../../tests/fixtures/agent-explorability/task11.json) | Override precedence, project/personal mappings, opt-out effects and preserved existing policy |

The new scope and advanced-policy oracles were independently hand-calculated and checked against the control before launch. Their artifact grader passed live synthetic replays with 154 and 263 checks respectively. Eleven scorer regressions include rejected wrong counts, paths, decisions, policy changes and read-only discovery violations. Seventeen harness regressions cover isolated defaults, explicit prebuilt provenance, frozen inputs, balanced scheduling, concurrency and rejection of altered graders or repeated launches.

## Decision Rules

Every candidate run must pass executable artifacts and final-answer interpretation, with no critical error and no lower pass count than control on any task. Discovery additionally requires all 13 capability families, valid measurement/workspace/compare/explain commands, and all four requested semantic families. An omission is incomplete; material misinformation is critical. Recovered errors may pass but retain their costs.

For each task, candidate median agent calls must be no more than control plus one, and median uncached input no more than 110% of control. Across all runs, candidate total calls must not exceed control and total uncached input must not exceed 105%. Secondary per-task median increases above 20% in cached input, output including reasoning, shell/CLI/reference invocations or elapsed time require route analysis and prevent a blanket efficiency claim. Both-zero comparisons pass; positive candidate against zero control fails or flags without division.

Invalid model/context, corrupt telemetry or broken instrumentation makes affected evidence inadmissible. Valid failures, incomplete work and exhausted budgets stay in the sample and all-run costs. There are no replacement or added trials. A corrective experiment requires a new candidate and protocol. Four pairs per task provide practical diagnostic evidence, not statistical equivalence or proof of perfection.

## Implementation Verification

Before freezing the candidate, 91 focused help, guide, README, query, comparison, workspace and explanation tests passed (one explicit README-maintenance test ignored). Formatting, Clippy and executable documentation recipes passed. Twenty-one checks established byte/normalized-output parity for guides, schemas, measurement, workspace and explanation between native binaries. Tests execute commands recovered from the displayed help, including wrapped continuation lines; visible-option inventory guards require new options to receive a workflow mapping.

## Results

The frozen campaign is running. Correctness, cost gates, independent interpretation review and the final recommendation remain pending; the implemented candidate is not yet an experimentally established efficiency improvement.

## Replay and Evidence

Use the committed [runner](../../scripts/full-help-study.py), [artifact grader](../../scripts/score_full_help.py), task definitions and protocol. Build exact source revisions into separate scratch directories without changing the working tree, or supply the recorded native binaries. Freeze with `--prepare-only`, inspect the manifests, then launch with `--run-frozen`. The runner refuses a second launch of a started campaign. Source declarations for prebuilt arms are separate from preparation-repository hashes; neither is presented as a source tree copied from the other arm.

Raw sessions, account/framework context, delivered artifacts, blinded review packets and telemetry audits belong only in ignored `docs/research/local/agent-explorability/2026-10-20-full-help-v1/`. Public summaries must contain only portable synthetic results and provenance. Final cumulative usage is checked against native session/events; output already includes reasoning and must not be added twice. Evaluator probes are excluded by the recorded explorer-completion boundary.
