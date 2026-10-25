# Agent Explorability: Routing Cost Replication

Started 2026-10-20. The owner asked to finish the uncertainty left by the [routing study](explorability-help-routing-study.md). Its four broader-task repetitions failed the practical cost guard, but that did not establish a repeatable penalty. The assistant's earlier “isn't ready to ship” wording was too categorical. This replication fixes the sample and decision rule before execution, keeps the candidate unchanged, and ends with a bounded recommendation rather than extending trials until a favorable result appears.

Subsequent adoption, authorized 2026-10-20: the tested wording is now in production root long help, with contributor rationale in the [help-design guidelines](../../help-design.md). The study's recommendation-only scope and results below remain historical evidence. Follow the [pre-adoption replay instructions](../../plans/agent-explorability-benchmark.md#replaying-pre-adoption-studies) when reproducing these treatments.

## Finding

Recommend adopting the unchanged example plus conditional execution-first routing in root help. Both preregistered stages passed correctness and every cost guard: 54 new sessions, all correct, with no excluded or replacement trials. The earlier broader-task cost increase and report cached-input threshold breach did not recur in these larger, balanced samples. This settles the bounded adoption recommendation; no further repetition or wording revision is scheduled.

Broader configuration used 6.5 median agent calls versus 7 with current help, and 32,801.5 median uncached input tokens versus 35,070.5, a 6.5% reduction. C used more calls in five of twelve pairs and more uncached input in three, so the fresh sample did not meet the frozen criterion for a consistent penalty. Report understanding used 2.5 median calls versus 3, with similar uncached input; C's cached-input median was only 0.8% above the example-only control, within the 20% guard.

The positive reason to adopt still comes from the [original covered-task result](explorability-help-routing-study.md#finding): explicit routing reduced reference retrievals and cost when the example was sufficient. This replication checks the adverse guards; it does not newly replicate that benefit or justify removing guides. Keep guides available and navigation unchanged. Production help remains unchanged because the frozen protocol ends at a recommendation, not automatic installation.

## Frozen Decision Process

The three arms remain A current help, B the previous example alone, and C that example plus conditional execution-first routing. Reuse the exact native binary hashes from the prior study after verifying unchanged product source inputs. All prompts, task fixtures, oracles, guide/schema content, model/effort (`gpt-6.1-sol` / `xhigh`) and per-run budgets remain unchanged. No prior sessions are pooled with the new sample.

Stage 1 uses [the broader-configuration protocol](../../../tests/fixtures/agent-explorability/help-routing-replication-broad.json): Task 8, twelve fresh triads, thirty-six sessions. Each of the six arm permutations appears twice in two complete blocks. Each arm therefore occupies each position four times. Arms run sequentially inside a triad, with at most three triads active. The exact order is frozen in the protocol.

Stage 2 uses [the report-understanding protocol](../../../tests/fixtures/agent-explorability/help-routing-replication-report.json): Task 1, six fresh triads, eighteen sessions, every permutation once and each arm in each position twice. It runs only if Stage 1 is admissible and passes correctness and both resource guards. Its purpose is to check the prior cached-input regression with balanced positions. Its entire schedule and definition are frozen before Stage 1; Stage 1 records its hash. If Stage 1 fails, Stage 2 is not run because adoption is already blocked by the bounded decision rule.

For each stage:

- Every C session must pass executable artifact checks and independent interpretation review, with no lower pass count than A or B. Valid failures, incomplete work and recovery costs stay in the sample. A budget-limited C run fails correctness.
- The primary guard is unchanged: C median agent-facing calls must not exceed A, and C median uncached input must not exceed A by more than 5%.
- The secondary guard is unchanged: no C median for shell executions, CLI invocations, cached input, output including reasoning, final-delivery duration, guide calls or schema calls may exceed either A or B by more than 20%. Positive C against a zero comparator fails without attempting division by zero.
- Invalid context, unsupported actual model/effort or missing/corrupt telemetry makes the affected stage inadmissible. Do not silently replace trials. Retain the issue and stop the adoption recommendation; ordinary task failures are not instrumentation exclusions.

For each failed primary metric in Stage 1, call its penalty direction consistent only if C is strictly greater than A in at least nine of twelve pairs; ties do not count. A failed primary guard without that directional consistency is mixed/inconclusive evidence, not established harm. A secondary-only failure is a resource-guard failure, not a consistent primary penalty. Stage 2 reports its six paired directions descriptively; the nine-of-twelve criterion does not apply.

After the fixed Stage 1 sample and grading, stop if any gate fails; otherwise execute the fixed Stage 2 sample and stop after grading it. Both stages passing would support a reasonable narrow adoption recommendation for the unchanged candidate. Either stage failing means retain current help. There is no additional trial, wording revision or automatic production installation in this protocol. These are practical decision thresholds, not statistical significance tests or proof of equivalence/absence of harm. The simple-task benefit still rests on the original four repetitions; this study tests the adverse guards only.

An independent MEDIUM preflight checked this bounded design and required exact schedules for both stages, per-metric strict paired directions, inherited failure handling, zero-comparator handling and explicit limits on what a successful replication would establish. All were incorporated before launch.

## Verification and Replay

The [runner](../../../scripts/agent-routing-study.py) now accepts an explicit protocol file while retaining its original default. A red/green regression checks that a supplied protocol controls the selected task, arm order and recorded protocol hash. Eleven Python tests passed. Native product inputs are unchanged, so the prior native parity and oracle checks remain applicable; binary/source hashes are rechecked before use, without rebuilding or rerunning unrelated Rust suites.

Run against fresh scratch directories and the three native binaries prepared by the [previous study's replay instructions](explorability-help-routing-study.md#verification-and-replay):

```sh
python3 scripts/agent-routing-study.py /tmp/locmetry-routing-replication-broad \
  --protocol tests/fixtures/agent-explorability/help-routing-replication-broad.json \
  --binaries "$experiment_root/A/target/debug/locmetry" "$experiment_root/B/target/debug/locmetry" "$experiment_root/C/target/debug/locmetry"
python3 scripts/score_agent_explorability.py /tmp/locmetry-routing-replication-broad
```

Independently grade final answers with condition/resource labels concealed, reconcile final cumulative counters and actual execution order, and apply the frozen gates. Only if all Stage 1 gates pass, run:

```sh
python3 scripts/agent-routing-study.py /tmp/locmetry-routing-replication-report \
  --protocol tests/fixtures/agent-explorability/help-routing-replication-report.json \
  --binaries "$experiment_root/A/target/debug/locmetry" "$experiment_root/B/target/debug/locmetry" "$experiment_root/C/target/debug/locmetry"
python3 scripts/score_agent_explorability.py /tmp/locmetry-routing-replication-report
```

The runner requires an authenticated Codex CLI with the specified actual model/effort; Locmetry gains no dependency. Artifact checks leave interpretation pending. Evaluator/reviewer work and replay probes are excluded from explorer costs. Retain uncached input, cached input, output, agent calls, shell executions, CLI invocations, reference retrievals and time separately. A changed model, source, prompt, oracle or treatment is a new study, not a replacement repetition.

## Results

### Stage 1: Broader configuration

36 fresh sessions completed; independent artifact-plus-interpretation passes A/B/C: 12/12 / 12/12 / 12/12. No valid trial was excluded or replaced.

| Median resource | A: current help | B: example | C: example + routing |
| --- | ---: | ---: | ---: |
| Agent-facing calls | 7 | 7 | 6.5 |
| Uncached input | 35,070.5 | 33,413 | 32,801.5 |
| Cached input | 257,152 | 249,984 | 229,760 |
| Output including reasoning | 3,727.5 | 4,062.5 | 3,995.5 |
| Shell executions | 16 | 14.5 | 12 |
| CLI invocations | 10 | 9.5 | 6.5 |
| Guide calls | 2 | 2 | 1 |
| Schema calls | 1 | 1 | 1 |
| Duration, seconds | 72.4 | 79.3 | 74.9 |

| Guard | Result |
| --- | --- |
| Correctness | Pass |
| Primary cost | Pass |
| Secondary cost | Pass |
| Entire stage | Pass |

| Comparator | Calls: C higher / tied / lower | Uncached input: C higher / tied / lower |
| --- | --- | --- |
| A | 5 / 0 / 7 | 3 / 0 / 9 |
| B | 3 / 2 / 7 | 5 / 0 / 7 |

Neither primary metric failed its guard. Neither had at least nine strict C>A pairs; there is no consistent primary penalty under the frozen diagnostic rule.

| Total resource | A | B | C |
| --- | ---: | ---: | ---: |
| Agent-facing calls | 84 | 84 | 77 |
| Uncached input | 438,127 | 412,109 | 393,136 |
| Cached input | 3,058,688 | 3,066,240 | 2,806,016 |
| Output including reasoning | 48,424 | 48,900 | 49,663 |
| Shell executions | 187 | 175 | 147 |
| CLI invocations | 123 | 112 | 86 |
| Guide calls | 23 | 26 | 16 |
| Schema calls | 13 | 17 | 12 |
| Duration, seconds | 909.6 | 965.9 | 935.5 |

### Stage 2: Report understanding

18 fresh sessions completed; independent artifact-plus-interpretation passes A/B/C: 6/6 / 6/6 / 6/6. No valid trial was excluded or replaced.

| Median resource | A: current help | B: example | C: example + routing |
| --- | ---: | ---: | ---: |
| Agent-facing calls | 3 | 2.5 | 2.5 |
| Uncached input | 20,809.5 | 20,760.5 | 20,589.5 |
| Cached input | 91,136 | 75,200 | 75,776 |
| Output including reasoning | 1,077.5 | 990 | 1,040.5 |
| Shell executions | 5.5 | 5 | 4.5 |
| CLI invocations | 4 | 3.5 | 3.5 |
| Guide calls | 1 | 1 | 1 |
| Schema calls | 0 | 0 | 0 |
| Duration, seconds | 25.2 | 24.2 | 24.7 |

| Guard | Result |
| --- | --- |
| Correctness | Pass |
| Primary cost | Pass |
| Secondary cost | Pass |
| Entire stage | Pass |

| Comparator | Calls: C higher / tied / lower | Uncached input: C higher / tied / lower |
| --- | --- | --- |
| A | 1 / 3 / 2 | 3 / 0 / 3 |
| B | 1 / 4 / 1 | 3 / 0 / 3 |

| Total resource | A | B | C |
| --- | ---: | ---: | ---: |
| Agent-facing calls | 16 | 15 | 15 |
| Uncached input | 123,519 | 144,456 | 123,191 |
| Cached input | 488,448 | 431,488 | 458,368 |
| Output including reasoning | 6,554 | 5,848 | 6,277 |
| Shell executions | 32 | 28 | 28 |
| CLI invocations | 22 | 21 | 21 |
| Guide calls | 6 | 6 | 6 |
| Schema calls | 0 | 0 | 0 |
| Duration, seconds | 167.3 | 141 | 147 |

The [machine-readable results](explorability-routing-replication-results.json) preserve all per-run costs, paired differences, medians, totals, ranges, reference retrievals, outcomes, frozen manifests and gate decisions. Agent calls, shell executions and CLI invocations are distinct units. Uncached input is input minus cached input; output includes reasoning and must not have reasoning added again. Time runs through final delivery, not first verified success. Evaluator/reviewer work is excluded.

## Evidence and Limits

Five fresh MEDIUM reviewers independently graded shuffled answer packets with condition/resource labels concealed: three packets of twelve broader-task answers and two of nine report answers. All passed. A separate reviewer reconciled raw session counters, completion boundaries, actual model/effort, isolation, budgets, fixture integrity, binary/source/guide hashes, scheduled order and peak concurrency of three. It independently recomputed all per-run metrics, totals, medians, ranges, paired directions and gate decisions. Stage 1's successful audit and grades preceded Stage 2's activation. Changing scratch paths were normalized when comparing context; substantive global instructions were identical and no repository instructions or previous answers were present.

The protocol-selection runner change passed eleven Python regressions with its failing-first result retained. Documentation indexes, local links/fragments and diff whitespace checks passed for the final records. Production Rust inputs were unchanged, so the existing native parity and oracle checks were reused without another Rust build or suite run.

Raw evidence is retained under Git-ignored `docs/research/local/agent-explorability/2026-10-20-routing-replication-v1/`, including frozen protocols/tasks/harnesses, manifests, CLI streams, sessions, answers, artifacts, independent reviews, telemetry audits and analysis scripts. Raw sessions can contain framework/account context and are not published. Public records are portable; copy the ignored archive separately for complete provenance on another machine.

This is a fixed diagnostic sample for one model/effort and the existing synthetic fixtures. It neither proves harm nor establishes general equivalence or absence of regression. The conditional second stage cannot retroactively alter the first stage or the original study. The prior simple-task benefit is not newly replicated here. Cache behavior, agent variability and the practical threshold choices remain relevant to interpretation. No monetary forecast or statistical significance is inferred.
