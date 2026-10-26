# Agent Explorability: Six-Task Guide-Removal Study

Study conducted 2026-10-20 after the owner authorized autonomous experiment design, recording, and a final assessment. The owner requested that recommendations be double-checked. This record extends the [first Task 1 pilot](explorability-task1-pilot.md); the [working benchmark](../../plans/agent-explorability-benchmark.md) retains the earlier discussion and historical evidence.

## Finding

Keep guides as optional reference and do not add guide navigation. Across 36 task runs, guides were not a prerequisite for completing any of the six workflows, but removing them did not produce a consistent resource improvement and one guide-free answer made a material coverage error. In six separate discovery runs, all agents identified the four main workflows with valid commands; guide removal reduced CLI invocations but increased uncached input by 16.7% and left slightly less explicit capability coverage. These results support a practical interface decision, not a general reliability guarantee.

## Design Fixed Before Trials

The study uses 36 new, independent MEDIUM sessions: six tasks × two conditions × three repetitions. The earlier pilot is not included in the sample. Condition A retains guides; B removes guide access and references using the already verified treatment. Both native binaries are byte-for-byte the binaries identified in the first pilot. The actual model is `gpt-6.1-sol`, effort `xhigh`, throughout.

Each pair receives identical prompts, fixture contents, tools, and budgets. Pair order alternates by task and repetition, yielding nine A-first and nine B-first pairs. Up to three pairs run concurrently; the two conditions within a pair run sequentially. Cost comparisons preserve uncached input, cached input, output, tool calls, CLI invocations, and timing separately. This is a diagnostic study, not a powered estimate of general agent reliability or an evaluation of every flag.

Task definitions were frozen before launching the campaign. The machine-readable campaign manifest records their hashes, the harness hash, binary hashes, actual model/effort, and execution order. The per-task stop thresholds remain 600 seconds, 40 agent-facing calls, 60 CLI invocations, and 100,000 uncached input tokens, sampled by the observer. All assertions about existing fixture counts and A/B measurement equivalence were checked before trials. No prompt or treatment changes are permitted within this campaign; a necessary revision must produce a separate recorded sample.

| Task | Frozen definition | Required outcome and principal error risks |
| --- | --- | --- |
| 1 — Report understanding | [Task 1](../../../tests/fixtures/agent-explorability/task1.json) | Correct source/all-counted figures, purpose Code, Code-based Test/prod, and unknown unrecognized counts. Identical task to the earlier pilot. |
| 2 — JSON automation | [Task 2](../../../tests/fixtures/agent-explorability/task2.json) | A replayable collector saves full per-file JSON and displays a human view from one measurement. Complete, partial, and invalid-configuration probes must preserve statuses 0, 2, and 1. Distinguish an empty file's measured zero from a parse failure's unknown counts and a headline from a baseline. |
| 3 — Configuration | [Task 3](../../../tests/fixtures/agent-explorability/task3.json) | Classify tools and intentional Other, exclude vendor, preserve inference/tests, and leave unresolved scratch counted and in review. Reject blanket classification or exclusion to eliminate uncertainty. |
| 4 — Comparison | [Task 4](../../../tests/fixtures/agent-explorability/task4.json) | Preserve the baseline, apply an exact one-line mutation, produce valid full reports and +1 Code/+1 Line comparison, reject a supplied headline, and explain net change and unrecoverable omitted historical detail. |
| 5 — Workspace | [Task 5](../../../tests/fixtures/agent-explorability/task5.json) | Reconcile file/direct-root measurements and member policies. Compute a deliberately nontrivial aggregate ratio: Test 6 / Production 4 = 1.50. Explain a -5 Code departure as changed measurement membership, not proof of deleted source. |
| 6 — Explanation | [Task 6](../../../tests/fixtures/agent-explorability/task6.json) | Save six complete explanations; distinguish counted, excluded, unrecognized, and counted-but-unclaimed; identify module/rule evidence and member-relative workspace patterns. |

All cases use synthetic data. Existing project files are immutable except the one explicitly requested source mutation in Task 4 and newly created configuration in Task 3. Each task starts independently, with any necessary preexisting policy supplied as fixture data. Task 2 introduces a deterministic invalid-notebook parse failure; Task 5 introduces unequal member ratios and a departing member, extending the earlier exploratory scenarios.

## Scoring and Interpretation Rules

Correctness remains the gate. A corrected mistake can pass while its recovery cost remains visible. Missing required work is incomplete; a delivered materially wrong artifact or interpretation is a critical failure. Deliberately requested invalid inputs are negative tests rather than accidental failures. Do not discard failed runs when reporting aggregate resource use.

[Artifact checks](../../../scripts/score_agent_explorability.py) validate saved data, remeasure configurations, rerun comparisons, and replay collector scripts on complete, partial, and invalid inputs. Evaluator probes are separated from explorer costs using the recorded completion boundary. Structured checks are not a semantic grader: final claims still need independent review. Missing telemetry, contamination, unsupported model configuration, and harness failures invalidate a trial rather than silently becoming successful executions.

The decision will distinguish necessity from efficiency. Successful guide-free runs establish feasibility for the tested cases. They do not establish that removing guides improves the interface. A guide-removal recommendation needs preserved correctness and a consistent resource advantage; conflicting tool/token results or task-specific regressions must remain visible. Conversely, retaining optional guides does not require claiming they are indispensable.

## Harness Changes

The [runner](../../../scripts/agent-explorability.py) now accepts a frozen task definition, permitted fixture mutations, generated baseline inputs, and prebuilt A/B binaries. It captures final session logs and counterbalances 18 pairs with bounded concurrency. The prior telemetry reader could silently skip a malformed completed JSONL line, potentially undercounting calls; it now tolerates only an unfinished final line. [Regression tests](../../../scripts/test_agent_explorability.py) demonstrated the failure before the fix and verify task-specific seeding. No production CLI behavior changed.

## Six-Task Results

All 36 runs completed within their budgets. Every delivered artifact passed executable checks. Independent MEDIUM reviewers assessed twelve shuffled answers each, with condition and cost labels concealed; commands in the answers could reveal guide access, so this was not perfect blinding. The lead evaluator checked the flagged interpretation against the original answer and explanation evidence. Final outcomes were **18/18 passes with guides and 17/18 without guides**, with no incomplete runs and one critical interpretation failure. No trials were excluded or replaced.

The failure occurred in guide-free Task 6, repetition 3. After correctly identifying `assets/opaque.xyz` as selected but unrecognized, the answer concluded: “No command failures or unavailable counts remain.” The file's counts remained unknown. All six explanation artifacts were correct; this was an unsupported final coverage claim, not a measurement failure. The frozen rule grades volunteered material misinformation even when artifacts pass. One occurrence does not establish that guide removal caused the error or that guides reliably prevent it.

| Task | Passes A / B | Median agent calls A / B | Median CLI invocations A / B | Median uncached input A / B | Median output tokens A / B |
| --- | --- | --- | --- | --- | --- |
| Report understanding | 3 / 3 | 2 / 3 | 3 / 5 | 20,148 / 17,523 | 1,067 / 1,099 |
| JSON automation | 3 / 3 | 6 / 8 | 10 / 12 | 36,455 / 38,340 | 5,188 / 6,929 |
| Configuration | 3 / 3 | 7 / 6 | 12 / 9 | 34,666 / 33,899 | 4,804 / 3,743 |
| Comparison | 3 / 3 | 8 / 7 | 9 / 9 | 38,301 / 36,575 | 4,340 / 4,418 |
| Workspace | 3 / 3 | 6 / 7 | 11 / 10 | 43,011 / 47,104 | 3,765 / 3,248 |
| Explanation | 3 / 2 | 5 / 4 | 13 / 14 | 28,799 / 30,518 | 2,588 / 2,502 |

A retains guides; B removes them. All-run totals include the critical failure and all verification/recovery work. Each arm has eighteen fresh sessions, so sums are directly comparable within this campaign. Output includes reasoning; reasoning must not be added again. Uncached input includes general agent/framework instructions as well as task and tool content.

| Resource | A: guides | B: no guides | B relative to A |
| --- | ---: | ---: | ---: |
| Agent-facing calls | 103 | 104 | +1.0% |
| Shell executions | 242 | 232 | -4.1% |
| Locmetry invocations | 174 | 173 | -0.6% |
| Uncached input tokens | 623,422 | 654,270 | +4.9% |
| Cached input tokens | 3,615,104 | 3,638,784 | +0.7% |
| Output tokens, including reasoning | 65,165 | 65,056 | -0.2% |
| Sum of run durations, seconds | 1,220.78 | 1,229.03 | +0.7% |

The aggregate is close, and the task differences matter more than a one-call difference. Removing guides used fewer agent calls in seven pairs, tied two, and used more in nine. It used fewer uncached input tokens in eight pairs and more in ten. Guide-free configuration used fewer CLI invocations in two pairs and fewer output tokens in all three; guide-free JSON automation used more output tokens in two pairs. Guide-free report understanding saved fresh input in all three pairs but used more CLI invocations in two and tied one. There is no consistent resource advantage for wholesale guide removal.

Every explorer started with `--help`. All eighteen A runs read at least one guide: 32 guide calls in total, including repeats. B made 42 schema calls compared with A's 22. Thus removing guides often shifted discovery toward schemas; it did not eliminate the need to discover semantics. Nonzero CLI statuses in the six-task campaign were deliberate partial/error probes, including the supplied invalid headline, rather than unsupported-option guesses. Shell/Python corrections remain included in the resource totals.

## Separate Initial-Discovery Track

A further six fresh sessions use [Task 7](../../../tests/fixtures/agent-explorability/task7.json): three repetitions per condition, with a frozen thirteen-family capability inventory, up to 800 words, and no project measurement or mutation. This track was specified before its own trials and is not pooled with the six-task sample. It tests capability discovery rather than task execution. The inventory covers measurement, workspace, comparison, explanation, output, scope, configuration, folders, purpose views, sorting, charts, schemas, and output defaults. Each family earns 1 for full correct coverage, 0.5 for partial correct coverage, and 0 for absent or wrong coverage. Minimal workflow commands and materially false claims are checked separately; guide availability is excluded from shared coverage.

The same binaries, model, effort, isolation, and per-run budgets were reused. Two discovery pairs ran concurrently; each pair's arms ran sequentially, with A first in repetitions 1 and 3 and B first in repetition 2. Timing remains diagnostic. All six started with `--help`, stayed within their budgets, preserved fixtures, and issued only help/version/schema/guide queries. All four minimal workflow commands were valid in every answer, and independent review found no critical errors.

| Discovery measure | A: guides | B: no guides |
| --- | ---: | ---: |
| Coverage scores by repetition, out of 13 | 12.5, 13, 13 | 12.5, 12, 12 |
| Agent-facing calls, total | 15 | 17 |
| CLI invocations, total | 51 | 43 |
| Uncached input tokens, total | 118,633 | 138,475 |
| Cached input tokens, total | 544,256 | 612,480 |
| Output tokens, total | 9,215 | 10,692 |
| Median duration, seconds | 60.49 | 72.61 |

The coverage difference concerns omissions, not false functionality: four answers left workspace comparison implicit, and two guide-free answers named `--folder-rows` without explaining its adaptive budget. Even granting full credit for implicit workspace support, the decision stays the same: coverage is high in both conditions, and removing guides does not consistently reduce resource use. A made 18 guide and 18 schema calls; B made 28 schema calls. B used 15.7% fewer CLI invocations but 13.3% more agent-facing calls, 16.7% more uncached input, and 16.0% more output tokens.

This broad discovery request elicits a detailed tour, not a measure of how much one `--help` call alone teaches an agent. Main workflows were visible in help and correctly mapped in all six runs, but the complete tour still took a median five agent calls in either condition. The experiment gives no basis for adding another navigation layer.

## Interpretation and Recommendation

The six-task evidence supports retaining guides as optional reference and leaving guide navigation unchanged. Guide-free execution is feasible for all six workflows, but not perfectly reliable in this sample. Removing guides produced essentially the same aggregate call count, slightly more fresh input, and one incorrect final interpretation. This does not support deleting the guides or claiming they are indispensable. The most promising narrower efficiency question is configuration discovery, where guide removal reduced the median CLI count and output tokens, while fresh input remained similar; any revised help or schema presentation should be tested as a new condition, not assumed to improve the interface.

Three repetitions per task are diagnostic evidence, not a powered reliability estimate. This study covers one actual model/effort configuration, one time window, and deliberately small synthetic fixtures. Explicit prompts request several semantic distinctions, so results do not establish that agents spontaneously volunteer every caveat in ordinary requests. Complex personal-policy interactions, a second independent fixture family, and long-session knowledge reuse remain untested. First-verified-success time and exhaustive phase-by-phase token attribution were not measured; reported time and usage run through final delivery.

## Next Variable: Information Placement

Follow-up: the [configuration-recipe study](explorability-help-recipe-study.md) records the subsequently authorized test of the owner's usage-example proposal. The proposal below preserves the reasoning before that experiment; consult the follow-up for its result.

After seeing interim results, the owner asked whether another variable would be more useful than guide availability. The recommended next experiment keeps guides and schemas available in both conditions and varies how much task-relevant information the first help response supplies. For configuration, the control is current help; a candidate treatment puts a minimal valid configuration recipe and its essential constraints beside the configuration entry point. This is an untested hypothesis, not a product change or an established improvement.

The retrieval traces support testing this narrower question: all six configuration trials requested the configuration schema, and four requested it twice. The emitted schema is 28,268 bytes in A and 28,224 in B; pipelines can filter that output before it reaches the model, so those byte sizes are not token-cost estimates. A short recipe could remove an initial syntax lookup, but might leave later schema queries necessary for precedence and classification evidence. The current experiment does not establish that a missing recipe caused those queries.

Freeze the help treatment before testing it on configuration and an independently constructed configuration variant. Include unrelated report-understanding trials to measure the extra reading imposed by longer root help. Preserve task correctness as the gate, then compare all-run tool calls, uncached/cached input, output, schema/guide retrievals, and time. A treatment earns adoption only if its task benefit outweighs that shared help cost without introducing errors. Do not fill help with fixture-specific answers or simultaneously add a new guide-navigation interface.

## Variation Across Repetitions

| Task | Agent-call range A / B | Uncached-input range A / B |
| --- | --- | --- |
| 1 | 2–3 / 3–3 | 19,902–20,359 / 17,051–17,977 |
| 2 | 6–7 / 6–8 | 30,982–37,887 / 34,929–55,211 |
| 3 | 6–7 / 5–7 | 31,185–35,014 / 32,401–34,990 |
| 4 | 6–8 / 6–9 | 26,336–74,843 / 33,747–52,134 |
| 5 | 6–7 / 7–7 | 40,547–45,126 / 40,708–68,158 |
| 6 | 5–6 / 3–5 | 28,696–31,165 / 30,058–32,947 |

The per-run results (removed 2026-10-26; `git show c8ad807:docs/research/agents/explorability-six-task-results.json`) preserve every task/discovery metric, outcome, guide/schema route, coverage subscore, and frozen campaign hash. In particular, comparison and workspace input costs vary substantially; neither one favorable run nor the small aggregate difference is a stable cost forecast.

## Evidence and Replay

Final session token totals were reconciled with the CLI's completion event rather than summed across cumulative updates. Agent tool counts were independently recounted from session calls, and CLI counts were checked against the pre-grading invocation boundary. All contexts reported the requested model/effort and no repository instructions. Task 2 evaluator replay calls are preserved separately and excluded from explorer costs. Review agents and evaluator work are also excluded: these are costs of assessing the interface, not costs incurred by an explorer using it.

Run only the native binary preparation and parity-check block from the [first pilot](explorability-task1-pilot.md#evidence-and-replay), retaining its `experiment_root` variable, then execute the six-task runner against fresh scratch paths. The fixtures and runner are versioned; no API key, private source checkout, or hosted artifact service is required by Locmetry. The experimental runner itself requires an authenticated Codex CLI and the recorded model/effort.

```sh
python3 scripts/agent-explorability.py campaign /tmp/locmetry-study-replay \
  --binaries "$experiment_root/A/target/debug/locmetry" "$experiment_root/B/target/debug/locmetry" \
  --model gpt-6.1-sol --effort xhigh
python3 scripts/score_agent_explorability.py /tmp/locmetry-study-replay
```

Artifact checks deliberately leave semantic interpretation pending. Independently grade final answers against their frozen prompts and oracles before assigning final outcomes. A process exiting successfully is not a benchmark pass. To add or revise a case, follow the [revision contract](../../plans/agent-explorability-benchmark.md#revising-or-adding-a-use-case); preserve old results and rerun both conditions for the changed case.

For an individual discovery pair or a new case, prepare with `--task`, then run each fresh arm. `run-02` is A and `run-01` is B; reverse their launch order for the next pair. The study's archived orchestration records its exact three-pair schedule.

```sh
python3 scripts/agent-explorability.py prepare /tmp/locmetry-discovery-replay \
  --task tests/fixtures/agent-explorability/task7.json \
  --binaries "$experiment_root/A/target/debug/locmetry" "$experiment_root/B/target/debug/locmetry"
python3 scripts/agent-explorability.py run /tmp/locmetry-discovery-replay run-02 --model gpt-6.1-sol --effort xhigh
python3 scripts/agent-explorability.py run /tmp/locmetry-discovery-replay run-01 --model gpt-6.1-sol --effort xhigh
```

Raw evidence is retained under Git-ignored `docs/research/local/agent-explorability/2026-10-20-six-task-study-v1/` and `docs/research/local/agent-explorability/2026-10-20-discovery-v1/`: frozen manifests/tasks/harnesses, treatment patches, prompts, CLI streams, session telemetry, answers, delivered artifacts, grader checks, and independent reviews. Native binaries are identified by hash instead of duplicating build directories. Raw session logs contain framework/account context and are deliberately not published. The public results and frozen task definitions remain portable across machines; the ignored raw archive must be copied separately if its full provenance is needed elsewhere.

Verification completed: all 36 artifact sets checked, all 42 final answers independently reviewed, final token/tool counters reconciled, an independent reporting audit confirmed numerical claims and scoring sensitivity, four Python harness/grader regression tests passed, and documentation indexes/links and Git whitespace checks passed. The earlier telemetry corruption and task-seeding regressions were demonstrated before the harness changes. Production Rust source and embedded guides remain unchanged.
