# Agent Explorability: Configuration Recipe in Help

Study started 2026-10-20. This directly revisits the owner's earlier proposal to put usage examples at the end of help, as man pages do. The assistant had argued against that proposal and subsequently recommended a similar experiment without crediting it; that earlier dismissal was too broad. The owner authorized this follow-up after the [guide-removal study](agent-explorability-six-task-study.md). This record tests the example's effect rather than assuming it helps or hurts.

## Frozen Design

The [study definition](../../tests/fixtures/agent-explorability/help-recipe-study.json) fixes two conditions: A is the current executable; B adds the exact [435-byte recipe](../../tests/fixtures/agent-explorability/help-recipe.txt) at the end of root long help's examples, immediately before the existing guide-reference paragraph. Both retain all guides, schemas, options, and output behavior. The recipe shows a valid TOML production rule for maintained `scripts/**`, a custom category, root-relative matching and test/build precedence, followed by a per-file JSON export plus human overview command. It does not supply the full solution for either configuration case. Short help and subcommand help are unchanged.

The study runs 24 fresh MEDIUM sessions: three tasks × two conditions × four repetitions. Actual configuration is `gpt-6.1-sol` with `xhigh` effort. No previous sessions count toward this sample. Pairs alternate A-first/B-first, with two of each order per task. Up to three pairs run concurrently, arms sequentially within pairs. Task prompts, fixtures, treatment, budgets, runner, and grader hashes are frozen before launch.

| Task | Purpose | Oracle |
| --- | --- | --- |
| [Task 3](../../tests/fixtures/agent-explorability/task3.json) | Original npm/Python configuration task; direct comparison with current help, using fresh controls. | Production tooling, intentional Other examples, vendor exclusion, preserved inference/tests, unresolved scratch retained for review. |
| [Task 8](../../tests/fixtures/agent-explorability/task8.json) | Independently designed Cargo/Rust/shell layout; tests transfer beyond the fixture used to motivate the recipe. | Source Files 6, Lines 21, Code 16; Production/Test/Other Code 8/4/4. All-counted Files 9, Lines 30, Code 22. Only `scratch/probe.rs` remains review. |
| [Task 1](../../tests/fixtures/agent-explorability/task1.json) | Unrelated report-understanding task; measures the reading cost of longer help. | Original source/all-counted figures, Code-based Test/prod, non-source interpretation, and unknown unrecognized counts. |

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

The primary configuration cases need exclusions and intentional Other rules beyond the displayed example. A separate [Task 9](../../tests/fixtures/agent-explorability/task9.json) tests a single production rule, custom category, retained conventional tests and unresolved review work; its required syntax is covered by the recipe. This limitation and the [supplementary definition](../../tests/fixtures/agent-explorability/help-recipe-simple-study.json) were recorded before primary aggregate results were inspected, although individual primary runs and artifact results had been observed. Its eight new sessions are not pooled with the primary sample and cannot override the original adoption gate. Four pairs alternate launch order; at most two pairs run concurrently. Both frozen native binaries are reused.

The supplementary gate requires all B runs to pass and at least three of four paired wins on the improved metric: either one fewer median agent call with uncached input no more than 5% higher, or at least 10% lower median uncached input with calls no higher. The source oracle is Files 3, Lines 5, Code 4; Production/Test/Other Code 2/1/1, with only `notes/probe.py` unresolved. Both native binaries independently reproduce it.

## Results

Pending execution, artifact checks, semantic review, and telemetry reconciliation. The frozen definition above must not change based on intermediate results.
