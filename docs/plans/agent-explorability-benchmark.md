# Agent Explorability Benchmark

<!-- doc-status
{
  "status": "active",
  "date": "2026-10-19",
  "summary": "Evaluate a configuration recipe in help while retaining guides and measuring unrelated-task cost.",
  "evidence": "[Six-task study](../research/agent-explorability-six-task-study.md)"
}
-->

This is the working specification and continuation record for an experiment-driven assessment of Locmetry's agent-facing interface. It preserves the discussion and exploratory evidence from 2026-10-19 and the subsequent experiments from 2026-10-20. After the first pilot, the owner delegated experiment design, recording, and evaluation. The [six-task study](../research/agent-explorability-six-task-study.md) owns the frozen campaign definitions, results, separate discovery track, and recommendation. The draft prompts below preserve design context; use the linked executable task definitions to replay the measured campaign. No production guide removal, new guide navigation, or other product-interface change was made.

[PLAN](../PLAN.md) owns the open-work index. [PRODUCT](../PRODUCT.md) owns supported behavior; this document owns the experiment, not a competing product contract. The earlier [embedded-guidance checkpoint](archive/measurement-quality-and-net-change.md#embedded-guidance-implementation--2026-10-16) preserves prior implementation evidence.

## Objective and Agreed Direction

The owner's goal is a highly explorable, agent-first binary: an agent starting with `--help` should quickly discover what is possible and accomplish ordinary tasks correctly. Success includes accurate interpretation, not merely producing syntactically valid commands.

The following direction was explicitly established in the conversation:

- Focus on MEDIUM agents. LOW-agent mistakes may provide supporting observations but must not determine the improvements.
- Treat tool calls and token consumption as important metrics alongside correctness.
- Do not add guide navigation yet. First test whether guides should be necessary at all.
- Establish metrics, benchmarks, task oracles, and critical-error definitions before making product changes.
- Preserve this work as a reviewable Markdown document that later sessions can continue, revise, extend with new use cases, and replay.

The working hypothesis is that guides might act as an additional interface that agents feel obliged to read. The exploratory run supports investigating this hypothesis; it does not establish that guides cause excessive cost or that removing them improves outcomes. The broad original assignment itself encouraged extensive reading.

The recommended target, still subject to review, is successful ordinary use without reading a guide. Guides could remain optional deeper explanations. A shorter interaction that produces a confidently wrong answer is not an improvement.

## Current State and Next Session

The [first Task 1 pilot](../research/agent-explorability-task1-pilot.md) established isolation and telemetry. The subsequent [study](../research/agent-explorability-six-task-study.md) completed 36 new task runs, excluded the earlier pair, and added a separate six-run capability-discovery track. The six-task sample passed 18/18 with guides and 17/18 without; resource totals were close, with 4.9% more uncached input after guide removal. Retain guides as optional reference and leave navigation unchanged; the study records the interpretation failure and task-specific tradeoffs. This is a tested recommendation, not evidence that guides are indispensable.

The owner subsequently authorized the [configuration-recipe experiment](../research/agent-explorability-help-recipe-study.md), revisiting their original usage-example proposal. Its frozen design uses 24 fresh sessions across the original configuration task, an independent Cargo layout, and report understanding. It retains guides in both conditions and measures the cost imposed on unrelated tasks.

For a later session:

1. Read the study's conclusions, limitations, and per-run results before proposing interface changes. Do not repeat the original pilot or pool sequential historical runs with fresh-task results.
2. Use the frozen task files and runner for replay. Resolve MEDIUM from the owner's active CLI mapping and record the actual model/effort; a changed model is a new campaign.
3. If testing an improvement, preserve the current interface as control and keep guides available. The [next-variable proposal](../research/agent-explorability-six-task-study.md#next-variable-information-placement) tests a minimal configuration recipe in help, with an independent configuration variant and unrelated-task checks for the cost of longer help. This hypothesis is motivated by repeated schema retrieval, not yet validated by an experiment.
4. Add an independent fixture variant when testing generalization, and give each changed prompt, fixture, oracle, or treatment a new revision. Keep old outcomes and all-run costs available.
5. Treat personal-policy interactions, extended scope cases, and long-session reuse as separate future cases, not as already verified behavior of the benchmark.

The owner authorized autonomous experimental design on 2026-10-20. The configuration-recipe follow-up is explicitly authorized; no production modification is authorized by this continuation record.

## Proposed Experiment

The initial design below was frozen for the 2026-10-20 campaign; the study owns its exact execution details.

| Condition | Interface available to the explorer |
| --- | --- |
| A — Current | Current binary, including help, guides, schemas, reports, and errors. |
| B — Without guides | Identical measurement functionality, schemas, reports, and errors; guides are neither advertised nor available. |

Do not improve help or output in B during the initial comparison. Combining removal with an interface redesign would obscure what the guides contribute. The implementation of B must be reviewed to ensure it removes guide access and references without introducing broken navigation or changing measurement behavior. Simply telling an agent not to use an advertised feature is a different treatment and must not be silently substituted.

If B reveals missing semantics, a later condition may move the minimum necessary information into command help or result output. That is a subsequent experiment, not part of the initial removal comparison.

Use two separate benchmark tracks:

- **Task execution:** a fresh agent for each task and condition, given an outcome-oriented request. This is the primary benchmark.
- **Initial discovery:** start with `--help` and assess the resulting capability map, unknowns, and next discovery choices against a predefined inventory. Keep this separate because “discover everything” encourages exhaustive reading. Its frozen inventory and scoring are in [Task 7](../../tests/fixtures/agent-explorability/task7.json).

A sequential six-task session may later measure reuse and amortization, but must not be pooled with fresh-task runs. The historical exploration used a sequential session and is not an interchangeable baseline for the proposed primary benchmark.

The original recommendation was a twelve-run six-task pilot before a 36-run campaign. After the initial Task 1 pair and independent fixture/harness checks, the delegated study froze six tasks and ran three independent repetitions per condition directly. The earlier pair was excluded. Three repeats provide diagnostic evidence, not strong statistical certainty; preserve that limit when interpreting the result.

Alternate condition order, hold model and effort constant, use isolated starting fixtures, and report per-task medians and ranges. Resolve the active MEDIUM tier from the owner's model mappings when starting a campaign, then record the actual model selector and effort. Never silently substitute an unsupported configuration or mix model changes into an interface comparison.

## Proposed Scoring Rule

An oracle is the independently established expected result. It checks observable artifacts, execution evidence, and delivered conclusions. It must not require a particular command sequence, documentation route, JSON-query utility, or phrasing unless that is itself part of the user's requested outcome.

| Outcome | Definition |
| --- | --- |
| Pass | Every requested result is correct and supported by execution evidence, with no critical error. |
| Incomplete | A required result is missing or explicitly unresolved, without a misleading conclusion. |
| Critical failure | A delivered answer or artifact would cause a materially wrong downstream decision, or an action irreversibly violates the task's constraints. |

Rule used in the pilot and frozen campaign: a mistaken command or hypothesis that is corrected before delivery can still pass. Record the failed attempt and its extra calls, tokens, and time as recovery cost. An irreversible harmful action remains a failure even if acknowledged afterward. A wrong conclusion that survives into the delivered answer or artifact is not excused by otherwise correct work.

An assertion of certainty unsupported by the available evidence can be a critical error. An explicit, justified statement of uncertainty is not. Unresolved required work still makes the task incomplete; uncertainty must not become a way to earn a pass without doing the work.

Do not silently repair an explorer's artifact before scoring it. Verify it independently, retain the failure evidence, and distinguish an explorer error from a harness failure or an actual Locmetry defect. All three matter, but they have different causes.

Do not impose hidden prose requirements. A task must request the interpretations that its oracle grades. Extra volunteered claims can be checked for harmful misinformation, but an agent should not have to recite every product caveat to pass an ordinary task.

## Original Task-Oracle Discussion

The exact executed definitions are linked from the [study task table](../research/agent-explorability-six-task-study.md#design-fixed-before-trials). The draft prompts and historical expectations below remain design context, not a substitute for those definitions.

Each future frozen task needs an ID and revision, an exact user prompt, a starting-fixture manifest, required outputs, executable assertions, required semantic conclusions, critical-error definitions, and explicit exclusions. The fixture must make important wrong interpretations produce different answers. For example, the ratio fixture must distinguish Code-based ratios from Lines-based ratios.

### Task 1: Understand a Report

**Execution status:** exercised in `task1-pilot-v1` and reused unchanged in the frozen six-task campaign. The [versioned pilot task](../../tests/fixtures/agent-explorability/task1.json) explicitly asks for the graded counts and Test/prod basis. The shorter original draft below is preserved as discussion history, not the exact executed prompt.

Proposed user prompt:

> Measure this project. Explain its source size, how Production and Test compare, and how other content contributes to the counts. Identify any selected content whose size the report cannot establish.

The existing synthetic web fixture provides independently verified expectations:

| Fact | Expected result |
| --- | --- |
| Counted source | 6 files, 11 Lines, 8 Code. |
| Source purposes, in Code | Production 2, Test 2, Other 4. |
| Test/prod | 1.00, using Code. The corresponding Test/Production Lines ratio is 3/4, so the wrong basis cannot accidentally pass. |
| All counted content | 9 files, 16 Lines, 10 Code. |
| Unrecognized content | One selected file, `assets/opaque.xyz`; its counts are unavailable. |
| Execution status | Successful. Unrecognized content alone does not make the report partial. |

Pass requires correct requested figures, source/all-counted distinction, correct ratio meaning, and identification of the unrecognized file without inventing its size. The status is an execution check; the answer need not recite exit codes unless asked.

Proposed critical errors:

- Reporting all-counted totals as source totals.
- Calculating Test/prod from Lines or including Other in its denominator.
- Treating the unrecognized file as measured zero or claiming every selected file was counted.
- Claiming unresolved purpose means source was not counted.
- Delivering materially incorrect requested figures despite available evidence.

An omitted ratio, omitted explanation of non-source content, or explicitly unresolved coverage question makes the task incomplete. Reading every guide, fetching unnecessary schemas, or recovering from truncation affects efficiency, not correctness by itself.

### Task 2: Consume JSON

**Campaign definition:** frozen and exercised as [Task 2](../../tests/fixtures/agent-explorability/task2.json). The original outline below is retained as design context.

Desired outcome: produce reusable machine-readable measurement data with per-file decisions and a human view from the same measurement; extract Production Code and unrecognized paths programmatically; distinguish compact output from comparison-ready data.

Candidate oracle: the saved document contains the requested per-file records, parsed values equal independently expected fixture values, and the human view agrees with the saved measurement. The extraction is executable and does not interpret absent optional detail as zero. Full measurement data is eligible for later comparison; per-file records are useful requested detail, not a prerequisite for aggregate comparison.

Candidate critical errors: presenting a headline as a complete saved baseline; treating missing `.files` as evidence that no files exist; silently returning zero for missing or unavailable data; delivering automation that accepts a known partial report as complete when completeness is required.

Open boundary: handling partial reports should be an explicitly requested automation variant, not a hidden requirement added to a simple extraction task. Use a deterministic parse-failure fixture, such as an invalid notebook, rather than relying on OS permission behavior to manufacture unreadability. Freeze that variant and its status expectations before including it in the campaign.

### Task 3: Configure Classification

**Campaign definition:** frozen and exercised as [Task 3](../../tests/fixtures/agent-explorability/task3.json). The original outline below is retained as design context.

Desired outcome: classify maintained tools as Production/tooling, classify examples intentionally as Other/examples, exclude vendor content, retain inferred application Production, and leave genuinely unresolved scratch content in review.

Candidate oracle: inspect the resulting configuration and independently remeasure. Verify each requested file decision, the remaining review set, source preservation, and expected scope. The historical configured web fixture has 5 source files, 10 Lines, 7 Code, split as Production 3, Test 2, Other 2; only `scratch/mystery.py` remains unclaimed.

Candidate critical errors: clearing the review list through unsupported blanket classification or exclusions; converting intentional Other into unresolved content or vice versa; changing correct inference unintentionally; altering source files to obtain expected counts; silently discarding relevant active personal policy.

Open boundary: the previous fixture had no active personal configuration. It supports checking an explanation of precedence but cannot establish behavioral correctness of personal/project interactions. Add a controlled personal-policy fixture only with an explicit requirement and an independently verified oracle.

### Task 4: Compare Measurements

**Campaign definition:** frozen and exercised as [Task 4](../../tests/fixtures/agent-explorability/task4.json). The original outline below is retained as design context.

Desired outcome: preserve a valid baseline, measure a controlled one-line source addition, obtain human and full JSON comparisons, interpret the change, and respond correctly to an unsuitable baseline.

Candidate oracle: the baseline remains unchanged; the controlled mutation produces +1 source Code and +1 Line with Files, Comments, and Blanks unchanged; the JSON and human result agree. Before/after direction follows the requested order. The negative case rejects a headline and does not pretend that rescanning the changed tree can reconstruct the historical baseline.

Candidate critical errors: reversing the delta; describing net count change as additions/deletions or churn; overwriting the historical baseline; claiming a current rescan recovers missing historical detail; interpreting equal recorded settings as proof of source-only growth.

Task 4 now explicitly asks about net change and the inability to recover omitted historical evidence by rescanning, with preconfigured policy independent of Task 3. Scope mismatch and partial-input handling remain potential additional comparison variants; do not silently add them to the core task.

### Task 5: Measure a Workspace

**Campaign definition:** frozen and exercised as [Task 5](../../tests/fixtures/agent-explorability/task5.json). The original outline below is retained as design context.

Desired outcome: measure two repositories through a workspace definition, reconcile member evidence, compare against direct-root measurement, and demonstrate saved-workspace comparison.

Candidate oracle: file-based and direct-root summaries and per-file decisions agree; member counts reconcile with independent single-root measurements; each member retains its own project policy; paths and evidence identify the correct member. Aggregate ratios use summed Code rather than an unweighted average of member ratios.

Candidate critical errors: applying one repository's classification policy to all members; omitting or double-counting a member; treating a member's absence from a report as evidence that the repository was measured empty; delivering a wrong requested aggregate ratio.

The historical experiment tested a zero-change comparison between equivalent workspaces. The later [Task 5](../../tests/fixtures/agent-explorability/task5.json) adds a departing member and unequal member ratios; its aggregate Test/prod is 1.50. Entering-member and broader policy-change variants remain outside that case.

### Task 6: Explain Selection and Classification

**Campaign definition:** frozen and exercised as [Task 6](../../tests/fixtures/agent-explorability/task6.json). The original outline below is retained as design context.

Desired outcome: diagnose application source, an explicit tooling rule, excluded vendor content, an unrecognized asset, unresolved scratch source, and a file in workspace context.

Candidate oracle: each explanation agrees with the independently measured record or coverage state. Application source uses module inference; tooling uses the requested rule; vendor is excluded; the asset is unrecognized; scratch is counted but unclaimed. Workspace evidence identifies the correct member while patterns retain member-relative meaning.

Candidate critical errors: conflating excluded, unrecognized, and counted-but-unclaimed states; claiming fallback source was not counted; inventing a classification cause contrary to recorded evidence; using another workspace member's policy to explain the target.

Do not award extra correctness credit for prescribing configuration changes when the task only asks for diagnosis. Wrong-path attempts corrected before delivery are recovery costs under the proposed scoring rule.

## Metrics and Attribution

Correctness is the gate. Do not combine accuracy and resource use into a weighted score that lets token savings compensate for critical failures.

| Metric | Collection and interpretation |
| --- | --- |
| Outcome and critical-error IDs | Score each run against the frozen task oracle. Preserve the assertions and evidence supporting the judgment. |
| Agent-facing tool calls | Count actual calls issued by the model, including non-shell tools. A batch counts as one agent-facing call. |
| Nested actions and CLI invocations | Separately count shell executions, file edits, and individual Locmetry invocations so batching cannot hide work. |
| Uncached input tokens | Record provider-reported input minus cached input where those fields have that relationship. Includes task and framework context, not just product documentation. |
| Cached input tokens | Record separately; repeated cached context is not equivalent to fresh reading and must not disappear from the cost record. |
| Output tokens | Include reasoning according to the provider's telemetry definition; record reasoning as a subset when available, without double-counting it. |
| Recovery work | Tag commands or retrievals used to recover from invalid guesses, truncation, wrong assumptions, or redundant requests. Keep deliberate negative tests separate from accidental failures. |
| Time to first verified success | Use the earliest recorded point at which the required artifacts and conclusions satisfy the oracle. Do not infer success from an agent's unsupported declaration. Mark unavailable if the trace cannot establish it. |
| Completion time and cost | Retain the full run totals, including the bounded final response, separately from first-success measurements when both are available. |

Classify actions as discovery, execution, verification, or reporting support; record a primary purpose and secondary tags where a batch crosses categories. Classification rules need to be frozen with the harness. Verification is legitimate work, not automatically waste.

Retain raw telemetry and its cumulative/per-turn interpretation. Some session events repeat cumulative totals: summing them would overcount usage. Tool-output bytes describe generated payload, not necessarily what reached the model after truncation, and are not a replacement for token telemetry.

Report per-task success, critical failures, incomplete runs, and resource distributions. Do not make an interface look cheaper by excluding its failed runs from the resource report; show successful-run costs and all-run costs with their denominators. No price estimate is required for the initial benchmark.

## Isolation, Fixtures, and Replay Contract

The [runner](../../scripts/agent-explorability.py) supports isolated task preparation, individual runs, and the six-task campaign. [Artifact checks](../../scripts/score_agent_explorability.py) verify executable outcomes but deliberately leave semantic review separate. The following contract governed the campaign:

- Place each explorer outside the repository with only the test binary, task fixtures, and output space. Do not provide source, project instructions, this specification, expected answers, or previous sessions.
- Verify the actual initial context. In this environment, a supposedly fresh subagent inherited repository instructions; a fresh agent alone did not establish blindness.
- Preserve identical general instructions, tools, locale, terminal/nonterminal behavior, width, environment, and personal configuration across conditions. Keep reference fixtures and telemetry outside measured roots where appropriate.
- Capture evidence automatically. Do not require explorers to build their own logging harness or write long assessments; those tasks added overhead to the exploratory run.
- Give each primary task a fresh state. Supply any required preconfigured policy or saved inputs directly from a verified fixture; do not rely on another task's output.
- Freeze command/time/token limits before the campaign and record limit exhaustion distinctly. The measured campaign used 600 seconds, 40 agent-facing calls, 60 CLI invocations, and 100,000 uncached input tokens per run; record any change as a new revision.
- Keep the same actual model, effort, harness, fixtures, prompt, and budget for an A/B comparison. Record provider/model changes that may affect reproducibility.

Each run manifest should contain campaign and benchmark revision, task ID/revision, condition, actual model/effort, fixture and prompt hashes, binary hash and source revision, harness revision, relevant environment, start/end time, resource limits, raw telemetry location, result artifacts, scoring result, and critical-error IDs. Per-call records should preserve arguments, status, stdout/stderr, timing, and the corresponding model-usage checkpoints when available.

Do not rely on temporary directories as the only durable record. The measured campaigns retain raw evidence under Git-ignored `docs/research/local/agent-explorability/`; public task definitions, metrics, and research records support replay without exposing session/account context. Synthetic public fixture definitions and runnable checks can be versioned; private measured-project data must remain exclusively in Git-ignored `docs/research/local/`, per repository policy. The owner has not requested a new artifact-storage service.

### Revising or Adding a Use Case

Give each new case a stable ID, an explicit requested outcome, a minimal fixture, independently known expected results, and bounded critical errors. Include a case only when it represents a meaningful user task or an identified semantic risk; do not grow the benchmark into an arbitrary checklist of every flag.

Changing a prompt, fixture, oracle, budget, or scoring rule creates a new revision. Keep old results tied to their original revision. Rerun both conditions for affected cases; do not compare a revised task in one condition against an old task in the other. Changes to shared instructions, the model, or the harness can require rerunning the campaign. Adding a case changes aggregate denominators, so retain the previous aggregate alongside the new one.

Keep at least one independently constructed fixture variant for checking whether an interface improvement generalizes beyond the examples used to design it. Exact variant selection remains open. Never silently rewrite historical evidence to match a new oracle.

## Exploratory Evidence — 2026-10-19

This is diagnostic evidence predating the frozen benchmark, not a controlled guide-removal result. The clean run used `gpt-6.1-sol` with `xhigh` effort. The binary was built from the working state at commit `7ca98cb` (`fix(output): clarify code ratios and style comparison summaries`), with SHA-256 `907013df0cbb49b58e1b8fcfe5a1e4d9f910ceba9187315368a3453b5ffe9469`. The project working tree was clean after the assessment.

Initially, both LOW and MEDIUM agents completed six tasks, but transcript inspection revealed inherited repository guidance. Those runs were excluded from the clean numerical comparison. MEDIUM was rerun outside the repository; transcript inspection confirmed that Locmetry repository instructions were absent. General user/framework instructions remained and contribute to token usage.

The clean run first received a help-only discovery task, then a sequential six-task assignment. Its task prompts explicitly requested several semantics and detailed evidence. Consequently, it cannot establish unprompted discovery of those semantics or the cost of a minimal ordinary request.

### Measured Results

| Metric | Initial help-only exploration | Whole sequential six-task evaluation, including initial help |
| --- | ---: | ---: |
| Agent-facing tool calls | 1 | 13 |
| Shell executions inside those calls | 1 | 21 |
| Locmetry invocations | 1 | 40 |
| Uncached input tokens | 13,174 | 85,193 |
| Cached input tokens | 35,584 | 766,848 |
| Output tokens, including reasoning | 1,721 | 17,517 |
| Total tokens | 50,479 | 869,558 |

The full output count includes 5,799 reasoning tokens; do not add those again. Total input was 852,041 tokens. The 13 agent-facing calls also orchestrated five file-edit actions. These are explorer-session costs, including framework context, task prompts, evidence preparation, and final reporting. They exclude the parent evaluator's setup/audit and discarded contaminated trials.

Of 40 Locmetry invocations, 17 were discovery queries: seven help requests, four guides, and six schemas. Three help requests repeated previously read subcommand help to retain evidence, leaving 14 distinct discovery queries. The remaining invocations were nine single-root measurements, five comparisons, two workspace measurements, and seven explanations. Summed executable time was about 1.46 seconds, excluding agent reasoning and surrounding tool overhead.

The only nonzero CLI exit was the deliberately unsuitable headline comparison. There were no accidental CLI failures or shell mistakes. Combined documentation output was truncated, causing selective rereading. The agent read all four guides, read the workspace schema fully, and queried the other schemas selectively rather than comprehending every emitted enumeration.

Independent checks verified initial/configured/changed counts, the exact source mutation, comparison delta, workspace equivalence, review and coverage results, explanation agreement, and noncreation of an output file after the rejected comparison.

| Sequential state | Source files | Source Lines | Source Code | Production Code | Test Code | Other Code |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Initial web | 6 | 11 | 8 | 2 | 2 | 4 |
| Configured web | 5 | 10 | 7 | 3 | 2 | 2 |
| Web after one-line addition | 5 | 11 | 8 | 4 | 2 | 2 |
| Worker | 2 | 2 | 2 | 1 | 1 | 0 |
| Workspace after addition | 7 | 13 | 10 | 5 | 3 | 2 |

The controlled addition was exactly `export const c = 3;` followed by one newline in `projects/web/src/index.js`. Configured web excluded vendor, classified tools as Production/tooling and examples as intentional Other/examples, retained npm inference, and left only scratch in review. File-based and direct-root workspace measurements matched despite reversed root argument order.

One help call yielded a broad capability map. The strongest observed product wording issue was the comparison renderer's “Unreadable files count as 0,” while the contract describes unknown unavailable counts. Both MEDIUM trials independently noticed it. The locus at the tested revision was [the comparison renderer](../../src/compare/render.rs); the [report guide](../guides/reports.md#unavailable-is-not-zero) and [command outcomes](../PRODUCT.md#command-outcomes) describe the relevant semantics. No unreadable fixture was tested in that run, so this observation concerns contradictory wording, not verified unreadable-file behavior. Fixing it is separate from this benchmark specification and would require identifying which binary each later result used.

Other findings: simultaneous saved JSON and human output was discoverable; the headline rejection identified missing evidence and explained why a current rescan cannot restore a lost historical baseline. Large schemas and combined guide reads incurred retrieval overhead. These observations support investigating targeted information placement, but do not yet justify a new guide-navigation interface.

Behavioral boundaries not tested included active personal-policy precedence, partial-count handling, Markdown, adaptive folders, bars, purpose filtering, subfolder semantics, classification overrides, ignore/hidden selection, symlinks, historical compatibility, and workspace membership changes. A capability map is not behavioral verification of every feature.

### Historical Fixture Definition

The following JSON maps relative paths to exact UTF-8 contents before any configuration or source mutation. There was no project `.locmetryrc`, no active personal configuration, and no Git work tree in the fixture roots. A replay must create the files in a fresh scratch directory, preserving escaped newlines. This is the historical sequential fixture, not the independent-task definitions used by the later study.

```json
{
  "projects/web/README.md": "# Demo\n\nSynthetic project.\n",
  "projects/web/assets/opaque.xyz": "opaque data\n",
  "projects/web/data.json": "{\"enabled\":true}\n",
  "projects/web/examples/old.py": "print(\"historical example\")\n",
  "projects/web/package.json": "{\"name\":\"demo-web\",\"version\":\"1.0.0\"}\n",
  "projects/web/scratch/mystery.py": "print(\"purpose undecided\")\n",
  "projects/web/src/index.js": "// application\nexport const a = 1;\nexport const b = 2;\n\n",
  "projects/web/tests/index.test.js": "// tests\nassert(a === 1);\nassert(b === 2);\n",
  "projects/web/tools/release.py": "print(\"release\")\n",
  "projects/web/vendor/bundle.js": "const vendor = 1;\n",
  "projects/worker/package.json": "{\"name\":\"demo-worker\",\"version\":\"1.0.0\"}\n",
  "projects/worker/src/job.js": "export const job = 1;\n",
  "projects/worker/tests/job.test.js": "assert(job === 1);\n"
}
```

### Historical Prompts and Evidence Location

The original help-only prompt was:

```text
You are participating in a black-box CLI usability evaluation. The only product information available is ./bin/locmetry in the current working directory. Do not access product source, other directories, external documentation, internet, wrapper implementation, or logs. Work only in this scratch directory. Stage 1: invoke ./bin/locmetry --help and ONLY that invocation. Then report your initial capability map of everything this binary can do, unresolved questions, built-in discovery commands you would try next, and immediate wording/navigation friction. Stop after this report; later instructions will supply tasks. Do not guess capabilities from the product name. This is evaluation, not product implementation.
```

The original sequential assignment was:

```text
Stage 2 of the black-box usability evaluation. Complete all six tasks below using only ./bin/locmetry and its built-in discovery material, plus ordinary shell/Python tools to inspect or edit the synthetic fixtures. Work ONLY inside your current scratch directory. Do not read product source, external docs, the bin wrapper implementation, or logs/. Do not use the internet or other agents. You may read synthetic projects/ and create evidence/ and outputs outside the measured roots. Save raw evidence for verification. Do not change the locmetry executable. This is assessment, not product implementation.

Tasks (start by inspecting fixtures as needed):
1. Produce a human report for projects/web and explain Files, Lines, Code, purposes, Test/prod, non-source content, and coverage. Distinguish uncertainty from zero.
2. Save full JSON including per-file decisions, while printing a human table from the same scan. Extract Production Code and unrecognized file paths programmatically. Find the machine-readable contract. Explain how full JSON differs from the default compact machine output.
3. Configure projects/web so tools/ is Production with category tooling, examples/ is intentionally Other with category examples, vendor/ is excluded, inferred application Production remains intact, and scratch/ remains unresolved review work. Verify the result and explain personal/project precedence. Do not modify the synthetic source files yet.
4. Save a configured baseline. Append exactly one executable source line to projects/web/src/index.js, save the after report, and compare in human and full JSON formats. Verify +1 Code and explain relevant interpretation caveats. Deliberately try an unsuitable compact/headline baseline and report how the error helps recovery.
5. Create a workspace definition for projects/web and projects/worker, measure it with per-member evidence, and compare its counts against a workspace measurement made from direct root arguments. Explain configuration ownership and workspace compare support.
6. Explain the application source, tools/release.py, excluded vendor/bundle.js, unrecognized assets/opaque.xyz, unresolved scratch/mystery.py, and a file in workspace context. Verify explanations agree with the configured reports.

Report your evidence-backed findings and exact commands, difficulties, wrong hypotheses and failed commands (including shell mistakes separately), built-in docs consulted, and discovered capabilities not behaviorally tested. Record which docs you read fully vs queried selectively. Do not create a Markdown assessment file; final response is the assessment. Give a brief progress update after completing the first three tasks. Stop when all six are complete. Task success and discovery efficiency both matter; do not artificially minimize calls at the expense of checking correctness.
```

Local raw evidence was stored under `/var/folders/jb/8sp0t5yx5zg4tfj3b8pz6bbc0000gn/T/locmetry-exploration-d1b9iq34/`: the `medium-clean/` directory contains CLI logs, fixture snapshots, reports, explanations, and verification scripts; `clean-stage1-events.jsonl` and `clean-stage2-events.jsonl` contain session events. The clean session ID was `01a15571-c916-7b20-9011-5a2c8766e085`. These temporary paths are a retrieval aid on the original machine, not a portable or guaranteed evidence archive. The facts, fixture, and task prompts above preserve the essential historical record if those files disappear.

The CLI wrapper isolated personal configuration using a scratch `XDG_CONFIG_HOME`, set `COLUMNS=100`, captured stdout/stderr without a TTY, and preserved `locmetry` as the displayed executable name. The clean runner used an outside-repository working directory, ignored user CLI configuration, and requested zero project-document bytes; transcript inspection, rather than flags alone, established absence of repository guidance. Exact agent decisions and token counts are not deterministic replay guarantees.

## Review Checklist and Decision Gates

The owner delegated design after the first pilot; these completed checks describe evaluator work, not a claim that the owner individually approved every oracle.

- [x] Freeze pass/incomplete/critical definitions, including recovered mistakes and material volunteered misinformation.
- [x] Freeze exact six-task prompts, independent starting states, required artifacts, semantic conclusions, and budgets.
- [x] Include deterministic partial-count failure, unequal workspace ratios, and a departing member; defer broader personal-policy and scope variants.
- [x] Define and run the separate capability-discovery inventory.
- [x] Verify the guide-free treatment isolates guide access/references and preserves measurement behavior.
- [x] Capture actual model/effort, tool calls, token totals, fixture integrity, and durable evidence.
- [x] Check artifacts and independently review final interpretation before comparing outcomes and costs.
- [ ] Evaluate any proposed interface improvement as a new treatment with an independent fixture variant; no such product change is authorized by this benchmark record.

Decision interpretation: successful guide-free execution establishes feasibility for the tested task, not that removal improves the product. Preserve task-specific resource tradeoffs and critical failures. The present study supports keeping optional guides without adding navigation. A future change needs its own observed mechanism, oracle, and comparative evidence.
