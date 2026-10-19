# Agent Explorability: First Task 1 Pilot

Evaluated on 2026-10-20 after the owner requested the first experiment using the recommended approach. This is one paired MEDIUM-agent pilot, not a completed benchmark campaign. The [working specification](../plans/agent-explorability-benchmark.md) owns remaining task-oracle and experiment decisions.

## Finding

Both agents correctly completed report understanding. Guides were not necessary for success on this fixture. Removing them did not demonstrate an overall efficiency benefit: the guide-free agent used fewer uncached input tokens but more tool calls, CLI invocations, output tokens, and wall time. The current-interface agent read one relevant guide and then answered from one measurement.

This contradicts a simple inference from the broad 2026-10-19 exploration that extensive guide reading necessarily characterizes ordinary use. It does not establish that guides reduce work in general, because there is only one run per condition and agent choices vary.

Recommendation after checking both answers, reports, and telemetry: leave the product interface unchanged. Repeat this task with the execution order reversed before drawing an efficiency conclusion, and review the next task's oracle before expanding the campaign. No new guide navigation is supported by this result either.

## Frozen Pilot Inputs

Pilot revision: `task1-pilot-v1`. The [task definition](../../tests/fixtures/agent-explorability/task1.json) contains exact fixture contents, prompt, expected values, provisional grading rule, and limits. The [harness](../../scripts/agent-explorability.py) prepares isolated source copies and task directories, verifies the treatment, invokes the explorers, and captures telemetry. Product source files were not edited.

Both explorers were fresh outside-repository sessions using actual model `gpt-6.1-sol` with `xhigh` effort. The harness inspected the session context and confirmed the requested model/effort and absence of Locmetry repository instructions. General user/framework instructions remained identical in kind and contribute to token usage. Both agents received identical prompts and fixture contents, had empty isolated personal configuration, and used nonterminal output with `COLUMNS=100`, `NO_COLOR=1`, and `LC_ALL=C`. The command transcripts contained only local fixture inventory and permitted binary invocations; no product source or external documentation was consulted.

The requested answer was bounded to 250 words. Neither agent was asked to produce an assessment document or logging harness. The prompt explicitly requested Test/prod and its basis, removing the hidden-grading ambiguity in the earlier draft phrase “how Production and Test compare.”

The pilot used the recommended rule that a corrected mistake may pass with recovery cost recorded separately. This was provisional use for the authorized pilot; it does not freeze the remaining five task oracles or the whole campaign. No mistakes requiring recovery occurred in either run.

Limits were fixed before execution: 600 seconds wall time, 40 agent-facing calls, 60 Locmetry invocations, and 100,000 uncached input tokens. The observer samples thresholds approximately once per second and stops an over-limit run; these are observer stop thresholds, not exact provider-side token caps. Neither run approached them.

Condition B ran first as `run-01`; condition A followed as `run-02`. There was no initial help-only assessment turn and no shared conversation. Each agent chose its discovery steps in response to the task itself.

## Treatment and Verification

Both binaries were built offline with the same locked Rust dependencies and debug profile from source revision `449f0b7df0ce544d77ddafa2b47b22bd234b4b62`. Separate temporary source copies and target directories preserved the working binary and avoided Git worktrees.

| Condition | Treatment | Binary SHA-256 |
| --- | --- | --- |
| A — Current | Unmodified interface with embedded guides. | `57b8ead9bc90a6ef3cb0af9666dad452bb397081e7dab47d1c840cfc435ad179` |
| B — Without guides | Remove the guide option, dispatch, and guide references from help, the unclaimed-source note, and configuration-schema prose. | `e77c497cb5a33aded51a26a74ffc912e838270cae9a022f7918977e661711364` |

The treatment removes references as well as access; it does not leave broken advertised navigation. Removing the configuration schema's guide reference changes its description text but not its structure or remaining semantics. The harness permits exactly that prose difference. No substitute workflow guidance was added.

The red guard rejected the untreated binary when supplied as B because it still advertised guides. After applying the treatment, verification passed for root/short/subcommand help, rejection of all four guide queries in B, successful guide queries in A, all six schemas with only the declared prose exception, version parity, and Task 1 JSON/headline/table/Markdown output parity except removed guide text and measurement timestamps. Independently expected fixture counts and unknown-path evidence also passed. This is a bounded treatment check, not a claim to have regression-tested every feature of the experimental binary.

## Results

| Metric | A — Current | B — Without guides |
| --- | ---: | ---: |
| Outcome | Pass | Pass |
| Critical errors | 0 | 0 |
| Recovery calls | 0 | 0 |
| Agent-facing tool calls | 2 | 4 |
| Nested shell executions | 4 | 7 |
| Locmetry invocations | 3 | 5 |
| Discovery invocations | 2 | 2 |
| Uncached input tokens | 19,932 | 18,140 |
| Cached input tokens | 60,544 | 117,632 |
| Output tokens, including reasoning | 1,025 | 1,464 |
| Total tokens, including cached input | 81,501 | 137,236 |
| Observed run wall time | 29.17 s | 34.24 s |
| Summed executable time | 0.089 s | 0.068 s |

Reasoning tokens were subsets of output: 471 for A and 566 for B. All CLI invocations exited 0. The answers contained 199 and 224 whitespace-delimited words respectively, within the shared response bound. The observed run wall time includes process startup and the final answer; it is not a separately instrumented time-to-first-success metric.

Token usage was checked twice: the emitted completion summary and the final cumulative session telemetry agreed. Tool-call counts were also recounted from the session transcripts. These totals include explorer instructions and context reuse, but exclude the evaluator's preparation, builds, verification, and reporting. Raw total tokens are not a monetary cost estimate; cached input and fresh input must remain distinguishable.

### Observed Paths

A used:

```text
locmetry --help
locmetry --guide reports
locmetry --root projects/web --no-user-config --format json --files
```

It also listed the synthetic files once. The guide and full JSON supplied sufficient evidence for its answer.

B used:

```text
locmetry --help
locmetry --root projects/web --no-user-config --format json --files
locmetry explain --help
locmetry --root projects/web --no-user-config --format table --verbose --categories --no-bars
locmetry explain --root projects/web --no-user-config --format json assets/opaque.xyz
```

It listed the synthetic files twice, with the second inventory explicitly bypassing ignore rules. The additional human report and targeted explanation were verification, not failed-command recovery. Neither agent requested a schema.

### Oracle and Interpretation Checks

Both reported 6 source files, 11 source Lines, and 8 source Code; Production 2, Test 2, and Other 4 Code; Test/prod 1.00 using source Code; and overall counted totals of 9 files, 16 Lines, and 10 Code. Both correctly explained the two data files and README contribution, and identified `assets/opaque.xyz` as unrecognized with unknown rather than zero counts. Both distinguished counted-but-unclaimed source from uncounted content.

The evaluator checked the raw full reports against the independent fixture oracle and verified that all fixture paths and contents remained unchanged, with no extra project files. The final answers and command transcripts were reviewed for unsupported semantic claims. Interpretation grading was performed by the evaluator with condition labels visible, not by a blinded independent judge. No critical errors or missing required conclusions were found.

## Evidence and Replay

Raw CLI output, events, session telemetry, both final answers, grading records, source/treatment hashes, and snapshots of the exact harness and task are retained locally in Git-ignored `docs/research/local/agent-explorability/2026-10-20-task1-pilot-v1/`. This is outside temporary storage, but is not synchronized by Git. The public task, harness, source revision, treatment recipe, metrics, and findings in this record support rerunning the experiment on another machine. Preserve or transfer the ignored evidence separately if exact raw historical transcripts are needed.

Session IDs: A `01a15634-76e2-7dd2-a595-b3ec119c8ab8`; B `01a15633-d054-7380-a255-60d6da57490f`. The task-definition SHA-256 was `04ec9a85637c0bb74c8c6cee2ce3e77f265c552b9c9ac0ffafd1c491db3bf250`; harness SHA-256 was `76aa3956f47cd4105c06d8223e83ba0bdabaf0782e3cd68ebd764eb3c92d714b`. The private manifest records every copied source hash. Build-path differences can change debug-binary hashes on another machine; record the new hashes rather than silently treating them as identical binaries.

For a replay from the repository root, choose a fresh destination and use the same task/harness revision. Preparation refuses an existing destination; agent execution refuses to overwrite previous run events. The treatment transformation fails if the expected source fragments have drifted, requiring review before adapting it.

```sh
experiment_root="$(mktemp -d /tmp/locmetry-task1.XXXXXX)/campaign"
python3 scripts/agent-explorability.py prepare "$experiment_root"
cargo build --locked --offline --manifest-path "$experiment_root/A/Cargo.toml" --target-dir "$experiment_root/A/target" > "$experiment_root/build-A.log" 2>&1
cargo build --locked --offline --manifest-path "$experiment_root/B/Cargo.toml" --target-dir "$experiment_root/B/target" > "$experiment_root/build-B.log" 2>&1
python3 scripts/agent-explorability.py verify "$experiment_root/A/target/debug/locmetry" "$experiment_root/B/target/debug/locmetry" > "$experiment_root/verification.log" 2>&1
```

Require successful builds and a passing verification result before running either explorer. `--offline` assumes the locked dependencies are already cached; dependency setup is a harness prerequisite, not explorer work. Invoke the runner using the model selector and effort resolved from the owner's active MEDIUM mapping; the following reproduces the actual configuration used in this pilot:

```sh
python3 scripts/agent-explorability.py run "$experiment_root" run-01 --model gpt-6.1-sol --effort xhigh > "$experiment_root/run-01.log" 2>&1
python3 scripts/agent-explorability.py run "$experiment_root" run-02 --model gpt-6.1-sol --effort xhigh > "$experiment_root/run-02.log" 2>&1
```

The runner requires a working authenticated Codex CLI and reads its persisted session telemetry. Confirm both context audits, actual model settings, completion statuses, fixture integrity, CLI evidence, and delivered conclusions before scoring. A nonzero process status, missing telemetry/context evidence, limit stop, or contamination must be reported explicitly rather than counted as a successful valid trial. The per-run summary is instrumentation, not an automatic semantic grade. Subsequent repetitions should use a fresh campaign and may reverse the two run commands to counterbalance order; retain their actual timestamps.

## Remaining Limits

This pair cannot establish a general efficiency ranking, reliable timing difference, or guidance needs for configuration, JSON automation, comparisons, workspaces, or file diagnosis. The simple synthetic fixture and explicit prompt make the required concepts clear. No partial-report case was included. Keep this result separate from the earlier sequential six-task exploration and from any revised Task 1 prompt or oracle.
