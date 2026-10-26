# Agent Explorability: Configuration Recipe in Help

Study started 2026-10-20. This directly revisits the owner's earlier proposal to put usage examples at the end of help, as man pages do. The assistant had argued against that proposal and subsequently recommended a similar experiment without crediting it; that earlier dismissal was too broad. The owner authorized this follow-up after the [guide-removal study](explorability-six-task-study.md). This record tests the example's effect rather than assuming it helps or hurts.

## Finding

Do not adopt this particular help addition. All 32 fresh MEDIUM sessions passed, but the primary experiment found no transferable efficiency gain, and the separate simple-rule probe also failed its gate. Every configuration explorer still retrieved the configure guide and configuration schema despite the added example. This supports keeping the current interface while preserving the owner's broader usage-example idea as a hypothesis for different designs. It does not establish that examples generally fail or that guides are necessary.

## Frozen Design

The [study definition](../../../tests/fixtures/agent-explorability/help-recipe-study.json) fixes two conditions: A is the current executable; B adds the exact [435-byte recipe](../../../tests/fixtures/agent-explorability/help-recipe.txt) at the end of root long help's examples, immediately before the existing guide-reference paragraph. Both retain all guides, schemas, options, and output behavior. The recipe shows a valid TOML production rule for maintained `scripts/**`, a custom category, root-relative matching and test/build precedence, followed by a per-file JSON export plus human overview command. It does not supply the full solution for either configuration case. Short help and subcommand help are unchanged.

The study runs 24 fresh MEDIUM sessions: three tasks × two conditions × four repetitions. Actual configuration is `gpt-6.1-sol` with `xhigh` effort. No previous sessions count toward this sample. Pairs alternate A-first/B-first, with two of each order per task. Up to three pairs run concurrently, arms sequentially within pairs. Task prompts, fixtures, treatment, budgets, runner, and grader hashes are frozen before launch.

| Task | Purpose | Oracle |
| --- | --- | --- |
| [Task 3](../../../tests/fixtures/agent-explorability/task3.json) | Original npm/Python configuration task; direct comparison with current help, using fresh controls. | Production tooling, intentional Other examples, vendor exclusion, preserved inference/tests, unresolved scratch retained for review. |
| [Task 8](../../../tests/fixtures/agent-explorability/task8.json) | Independently designed Cargo/Rust/shell layout; tests transfer beyond the fixture used to motivate the recipe. | Source Files 6, Lines 21, Code 16; Production/Test/Other Code 8/4/4. All-counted Files 9, Lines 30, Code 22. Only `scratch/probe.rs` remains review. |
| [Task 1](../../../tests/fixtures/agent-explorability/task1.json) | Unrelated report-understanding task; measures the reading cost of longer help. | Original source/all-counted figures, Code-based Test/prod, non-source interpretation, and unknown unrecognized counts. |

Task 8 was independently authored and validated before trial execution. Its author did not inspect the recipe; a broad search incidentally exposed only the generic verification-command heading after the layout had been selected. The fixture has different languages, files, counts, module inference, and no unrecognized asset. Its counts and exact per-file decisions were validated with both executables. The configuration tasks retain the same substantive difficulty; Task 8 explicitly requests all-counted figures as well, so comparisons are within each task, never directly between tasks.

The existing per-run budgets remain 600 seconds, 40 agent-facing tool calls, 60 CLI invocations, and 100,000 uncached input tokens. Each explorer starts outside the repository, with only binary access and synthetic inputs; repository instructions, previous sessions, and expected answers are excluded. Actual model/effort and context are audited. No production source or embedded guide is edited.

## Decision Rule Before Results

Correctness is the gate: B must have no incomplete or critical-failure runs and no lower pass count than A on any task. Recovered mistakes can pass, retaining their full cost. Artifact checks and independent final-answer review remain separate.

For each configuration case, a useful improvement means either at least one fewer median agent-facing call with median uncached input no more than 5% higher, or at least 10% lower median uncached input with median agent calls no higher. The improved metric must favor B in at least three of the four pairs for that case. For unrelated report understanding, median calls must not rise and median uncached input must not rise by more than 5%. These are practical diagnostic thresholds, not a statistical significance test. They deliberately require transfer and account for the cost imposed on other uses.

Always report cached input, output including reasoning, shell/CLI invocations, guide/schema retrievals, and completion time. A material regression in secondary costs prevents unqualified adoption. Failure to meet the gate means this particular recipe has no demonstrated adoption benefit; it does not reject usage examples as an approach. The experiment does not install the treatment into production, even if the result is favorable.

## Treatment and Harness Verification

The recipe was added only in a temporary source copy. The control binary is byte-identical to the first study's A binary. The treatment built successfully with locked offline dependencies. Root long help differs by exactly the recipe; short help, subcommand help, all four guides, all six schemas, version output, and four report formats were checked for parity. The displayed TOML and verification command were executed on both binaries, confirming the production category and retained conventional tests.

The runner gained an explicit treatment selector and configurable task/repetition selection. Red/green checks guard retaining guides and other source files, selected cases, repetition counts, and balanced launch order. Extending the artifact grader exposed a boundary defect: unsupported task IDs could silently skip task-specific assertions. The grader now rejects unsupported tasks and empty/non-object report artifacts; fault-injection checks cover incorrect per-file decisions and empty artifacts. Earlier recorded artifacts were nonempty and used supported tasks, so this does not alter their outcomes.

## Supplementary Simple-Rule Probe

The primary configuration cases need exclusions and intentional Other rules beyond the displayed example. A separate [Task 9](../../../tests/fixtures/agent-explorability/task9.json) tests a single production rule, custom category, retained conventional tests and unresolved review work; its required syntax is covered by the recipe. This limitation and the supplementary definition (removed 2026-10-26; `git show c8ad807:tests/fixtures/agent-explorability/help-recipe-simple-study.json`) were recorded before primary aggregate results were inspected, although individual primary runs and artifact results had been observed. Its eight new sessions are not pooled with the primary sample and cannot override the original adoption gate. Four pairs alternate launch order; at most two pairs run concurrently. Both frozen native binaries are reused.

The supplementary gate requires all B runs to pass and at least three of four paired wins on the improved metric: either one fewer median agent call with uncached input no more than 5% higher, or at least 10% lower median uncached input with calls no higher. The source oracle is Files 3, Lines 5, Code 4; Production/Test/Other Code 2/1/1, with only `notes/probe.py` unresolved. Both native binaries independently reproduce it.

## Primary Results

All 24 runs completed within budget. Executable artifact checks and two independent MEDIUM reviewers found **12/12 passes in A and 12/12 in B**, with no incomplete or critical-failure runs. Reviewers each received twelve shuffled answers with condition and resource labels hidden; wording could still reveal the treatment, so blinding was imperfect. No runs were excluded or replaced. Actual model/effort, isolated context, prompt/source/binary hashes, final token counters, and agent-call counts were reconciled against recorded sessions.

| Task | Passes A / B | Median agent calls A / B | Median CLI invocations A / B | Median uncached input A / B | Median output A / B |
| --- | --- | --- | --- | --- | --- |
| Report understanding | 4 / 4 | 3 / 2 | 4 / 3.5 | 20,812 / 20,389 | 841.5 / 1,008 |
| Original configuration | 4 / 4 | 6.5 / 6.5 | 9 / 10 | 33,512.5 / 36,937.5 | 4,415 / 3,305.5 |
| Independent configuration | 4 / 4 | 6 / 7 | 11.5 / 9.5 | 31,044.5 / 35,783.5 | 3,740 / 3,735.5 |

A is current help; B adds the example. Output includes reasoning and must not have reasoning added again. Uncached input includes the general agent/framework context as well as task and tool content. Agent calls, shell executions and CLI invocations are different units: one agent call can launch multiple shell commands.

**Both configuration cases fail the frozen efficiency gate.** Original configuration has unchanged median agent calls and 10.2% more median uncached input. Independent configuration has one more median agent call and 15.3% more median uncached input. In each case, B wins agent calls in two of four pairs and uncached input in one of four. Report understanding passes its guard: fewer median calls and 2.0% less median uncached input. Correctness passes, but the required transferable efficiency improvement does not.

| Resource, twelve sessions per arm | A | B | B relative to A |
| --- | ---: | ---: | ---: |
| Agent-facing calls | 65 | 62 | -4.6% |
| Shell executions | 142 | 140 | -1.4% |
| Locmetry invocations | 98 | 95 | -3.1% |
| Uncached input tokens | 373,109 | 405,586 | +8.7% |
| Cached input tokens | 2,249,216 | 2,174,336 | -3.3% |
| Output tokens including reasoning | 35,759 | 32,548 | -9.0% |
| Sum of run durations, seconds | 675.5 | 635.8 | -5.9% |

Every run started with `--help`. All sixteen configuration runs still read `--guide configure` and `--schema config`; the example did not eliminate either retrieval in this sample. Across all primary runs A made 18 guide calls and B 19; each made 10 config-schema calls, including repeats. B made fewer CLI calls overall but consumed more uncached input. These are observed routes, not proof that guide references caused the additional reading.

The machine-readable results (removed 2026-10-26; `git show c8ad807:docs/research/agents/explorability-help-recipe-results.json`) retain per-run costs, medians, ranges, paired wins, guide/schema retrievals and outcomes. Durations are through final delivery, not first verified success. Concurrent execution makes timing diagnostic, and token caching is reported separately rather than treated as a fixed price.

## Supplementary Results

All eight supplementary sessions completed within budget and passed executable artifact checks and independent final-answer review: **4/4 passes in each condition**, with no incomplete or critical failures, exclusions, or replacements. The reviewer received shuffled answers without condition or cost labels. All started with `--help`; all eight still read the configure guide and config schema. A made five guide calls and B four; each made four config-schema calls. Neither condition issued a nonzero-status Locmetry invocation in this probe or in the primary study.

| Simple-rule measure | A: current help | B: help example |
| --- | ---: | ---: |
| Median agent calls | 6 | 7.5 |
| Median shell executions | 12 | 11.5 |
| Median CLI invocations | 6.5 | 6.5 |
| Median uncached input | 28,559 | 37,146.5 |
| Median cached input | 205,632 | 265,088 |
| Median output including reasoning | 2,561.5 | 2,764.5 |
| Median duration, seconds | 55 | 68.9 |

B wins zero of four pairs on either agent calls or uncached input: calls are higher in three pairs and tied in one, and uncached input is higher in all four. Median uncached input is 30.1% higher, with 1.5 more median agent calls. Thus the supplementary gate fails even when the example covers the required syntax.

| Resource, four sessions per arm | A | B |
| --- | ---: | ---: |
| Agent-facing calls | 24 | 31 |
| Shell executions | 46 | 47 |
| Locmetry invocations | 26 | 26 |
| Uncached input tokens | 114,232 | 157,606 |
| Cached input tokens | 823,680 | 1,065,856 |
| Output tokens including reasoning | 10,065 | 11,875 |
| Sum of run durations, seconds | 220.3 | 265.2 |

## Interpretation and Limits

Keep production help and optional guides unchanged. The primary adoption gate failed on both configuration layouts, and the separate covered task supplies no countervailing efficiency signal. A small example can supply valid syntax without changing agents' subsequent retrieval behavior; all 24 configuration sessions still read both the guide and schema. The experiment does not identify why they did so. In particular, it does not prove that the remaining guide references caused these lookups, nor that removing those references would improve reliability or cost.

This tests one 435-byte configuration example appended to existing root long help, one model/effort configuration, and four pairs per case. It does not test replacing help text, putting examples beside individual options, changing retrieval instructions, examples for other commands, or human usability. Those are separate hypotheses. The additive design and existing help references remain explicit boundaries of the result.

The simple probe was designed after individual primary runs had been observed, although before primary aggregate analysis. Its separate protocol and decision gate preserve that provenance; it is not a replacement confirmatory sample. The small synthetic cases and prompts requesting precise evidence can encourage more verification than ordinary minimal requests. No general statistical significance or production reliability claim follows from these runs.

Uncached input varies with cache behavior and agent trajectories; the observed percentages are not stable token or monetary forecasts. Cached input and output sometimes move in different directions, which is why all resource dimensions remain visible. No token price or aggregate monetary saving was inferred. For subsequent work, use the [benchmark continuation rules](../../plans/agent-explorability-benchmark.md#current-state-and-next-session), freeze a genuinely new variable before trials, and preserve these outcomes instead of selecting only favorable repetitions.

## Evidence and Replay

The primary definition and harness were committed before launch as `d2b4723`; the supplementary definition and Task 9 support were committed before its launch as `7e661eb`. Extending the grader to Task 9 happened only after primary artifact checks completed. The primary archive retains the original grader and its matching hash; the supplementary manifest identifies the extended grader. Neither change modifies Locmetry's runtime. A separate documentation correction (`878e1f6`) changed PRODUCT's fallback category wording to `unclaimed`, matching the existing schema and behavior; it did not alter either binary or any embedded guide.

The results JSON (removed 2026-10-26; `git show c8ad807:docs/research/agents/explorability-help-recipe-results.json`) records both native binary hashes and each protocol's hashes. Replaying changed sources, prompts, models or fixtures creates a new campaign. For a replay from the recorded sources, prepare temporary native copies and verify the exact treatment before launching fresh agents. Build the two targets sequentially, retaining build logs:

```sh
experiment_root=/tmp/locmetry-help-recipe-replay-build
python3 scripts/agent-explorability.py prepare "$experiment_root" --treatment help-recipe
cargo build --locked --offline --manifest-path "$experiment_root/A/Cargo.toml" > "$experiment_root/build-A.log" 2>&1
cargo build --locked --offline --manifest-path "$experiment_root/B/Cargo.toml" > "$experiment_root/build-B.log" 2>&1
python3 scripts/verify_help_recipe.py "$experiment_root/A/target/debug/locmetry" "$experiment_root/B/target/debug/locmetry"
python3 scripts/agent-explorability.py campaign /tmp/locmetry-help-recipe-replay \
  --binaries "$experiment_root/A/target/debug/locmetry" "$experiment_root/B/target/debug/locmetry" \
  --model gpt-6.1-sol --effort xhigh --treatment help-recipe --task-ids 1 3 8 --repetitions 4
python3 scripts/score_agent_explorability.py /tmp/locmetry-help-recipe-replay
```

The experimental runner needs an authenticated Codex CLI with the recorded model/effort. Locmetry itself gains no external runtime requirement. Resolve MEDIUM from the owner's active mapping for new experiments and report unsupported configurations rather than substituting. Offline builds require dependencies already cached.

To replay a supplementary pair, use the separately frozen Task 9. `run-02` is A and `run-01` is B. Use four fresh pair directories, A-first in repetitions 1 and 3, B-first in 2 and 4, and at most two concurrent pairs; arms within a pair remain sequential. Do not use the primary `campaign` selector for Task 9: its help-recipe campaign metadata identifies the primary protocol. The archived supplementary orchestrator records the exact eight-run schedule and hashes.

```sh
python3 scripts/agent-explorability.py prepare /tmp/locmetry-simple-replay/task9-rep1 \
  --task tests/fixtures/agent-explorability/task9.json --treatment help-recipe \
  --binaries "$experiment_root/A/target/debug/locmetry" "$experiment_root/B/target/debug/locmetry"
python3 scripts/agent-explorability.py run /tmp/locmetry-simple-replay/task9-rep1 run-02 --model gpt-6.1-sol --effort xhigh
python3 scripts/agent-explorability.py run /tmp/locmetry-simple-replay/task9-rep1 run-01 --model gpt-6.1-sol --effort xhigh
python3 scripts/score_agent_explorability.py /tmp/locmetry-simple-replay
```

Artifact checks leave interpretation pending. Independently review final answers against their frozen prompts and oracles before assigning outcomes. All costs run through final delivery, including recovery and extra verification; evaluator probes and review work are excluded from explorer costs. Session cumulative counters are reconciled with the final completion event, not summed across updates. CLI calls are cut at the recorded explorer-completion boundary.

Raw evidence is retained in Git-ignored `docs/research/local/agent-explorability/2026-10-20-help-recipe-v1/` and `docs/research/local/agent-explorability/2026-10-20-help-recipe-simple-v1/`: frozen tasks/protocols/harnesses, manifests, treatment patch and parity checks, CLI streams, session telemetry, answers, delivered artifacts, executable checks, independent reviews and analysis scripts. Raw session logs can contain framework/account context, so they are not published. The public definitions and results are portable; copy the ignored archives separately when complete provenance is needed on another machine. Native binaries are identified by hash rather than copying build directories into the archive.

Verification completed: all 32 artifact sets checked and all 32 final answers independently reviewed; session counters, model/effort, source/binary hashes and isolation checks reconciled; primary and supplementary numerical claims independently audited; eight Python harness/grader regression tests passed; exact treatment and executable recipe parity passed; documentation indexes/links and Git whitespace checks passed. The native treatment build was run once with its log retained. Production Rust source and embedded guides remain unchanged.
