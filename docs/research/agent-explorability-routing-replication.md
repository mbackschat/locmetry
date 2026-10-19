# Agent Explorability: Routing Cost Replication

Started 2026-10-20. The owner asked to finish the uncertainty left by the [routing study](agent-explorability-help-routing-study.md). Its four broader-task repetitions failed the practical cost guard, but that did not establish a repeatable penalty. The assistant's earlier “isn't ready to ship” wording was too categorical. This replication fixes the sample and decision rule before execution, keeps the candidate unchanged, and ends with a bounded recommendation rather than extending trials until a favorable result appears.

## Frozen Decision Process

The three arms remain A current help, B the previous example alone, and C that example plus conditional execution-first routing. Reuse the exact native binary hashes from the prior study after verifying unchanged product source inputs. All prompts, task fixtures, oracles, guide/schema content, model/effort (`gpt-6.1-sol` / `xhigh`) and per-run budgets remain unchanged. No prior sessions are pooled with the new sample.

Stage 1 uses [the broader-configuration protocol](../../tests/fixtures/agent-explorability/help-routing-replication-broad.json): Task 8, twelve fresh triads, thirty-six sessions. Each of the six arm permutations appears twice in two complete blocks. Each arm therefore occupies each position four times. Arms run sequentially inside a triad, with at most three triads active. The exact order is frozen in the protocol.

Stage 2 uses [the report-understanding protocol](../../tests/fixtures/agent-explorability/help-routing-replication-report.json): Task 1, six fresh triads, eighteen sessions, every permutation once and each arm in each position twice. It runs only if Stage 1 is admissible and passes correctness and both resource guards. Its purpose is to check the prior cached-input regression with balanced positions. Its entire schedule and definition are frozen before Stage 1; Stage 1 records its hash. If Stage 1 fails, Stage 2 is not run because adoption is already blocked by the bounded decision rule.

For each stage:

- Every C session must pass executable artifact checks and independent interpretation review, with no lower pass count than A or B. Valid failures, incomplete work and recovery costs stay in the sample. A budget-limited C run fails correctness.
- The primary guard is unchanged: C median agent-facing calls must not exceed A, and C median uncached input must not exceed A by more than 5%.
- The secondary guard is unchanged: no C median for shell executions, CLI invocations, cached input, output including reasoning, final-delivery duration, guide calls or schema calls may exceed either A or B by more than 20%. Positive C against a zero comparator fails without attempting division by zero.
- Invalid context, unsupported actual model/effort or missing/corrupt telemetry makes the affected stage inadmissible. Do not silently replace trials. Retain the issue and stop the adoption recommendation; ordinary task failures are not instrumentation exclusions.

For each failed primary metric in Stage 1, call its penalty direction consistent only if C is strictly greater than A in at least nine of twelve pairs; ties do not count. A failed primary guard without that directional consistency is mixed/inconclusive evidence, not established harm. A secondary-only failure is a resource-guard failure, not a consistent primary penalty. Stage 2 reports its six paired directions descriptively; the nine-of-twelve criterion does not apply.

After the fixed Stage 1 sample and grading, stop if any gate fails; otherwise execute the fixed Stage 2 sample and stop after grading it. Both stages passing would support a reasonable narrow adoption recommendation for the unchanged candidate. Either stage failing means retain current help. There is no additional trial, wording revision or automatic production installation in this protocol. These are practical decision thresholds, not statistical significance tests or proof of equivalence/absence of harm. The simple-task benefit still rests on the original four repetitions; this study tests the adverse guards only.

An independent MEDIUM preflight checked this bounded design and required exact schedules for both stages, per-metric strict paired directions, inherited failure handling, zero-comparator handling and explicit limits on what a successful replication would establish. All were incorporated before launch.

## Verification and Replay

The [runner](../../scripts/agent-routing-study.py) now accepts an explicit protocol file while retaining its original default. A red/green regression checks that a supplied protocol controls the selected task, arm order and recorded protocol hash. Eleven Python tests passed. Native product inputs are unchanged, so the prior native parity and oracle checks remain applicable; binary/source hashes are rechecked before use, without rebuilding or rerunning unrelated Rust suites.

Run against fresh scratch directories and the three native binaries prepared by the [previous study's replay instructions](agent-explorability-help-routing-study.md#verification-and-replay):

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

Pending fixed-sample execution, independent grading and telemetry audit. Both protocols and the stopping rule remain frozen.
