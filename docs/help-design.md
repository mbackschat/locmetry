# CLI Help Design

Help should let a person or agent discover a task, choose a valid route and verify its result with as little additional reading as the task requires. This document owns contributor guidance for help changes; [PRODUCT](PRODUCT.md) owns supported behavior and the [benchmark](plans/agent-explorability-benchmark.md) owns experiment design and replay.

## What Belongs Where

| Surface | Include | Rationale |
| --- | --- | --- |
| Root help | Capabilities, output defaults, common task commands and a small number of working examples with clear applicability. | An agent starting with `--help` needs enough information to take a useful next action. Every addition also costs readers doing unrelated tasks. |
| Command help | Command-specific syntax, consequential defaults, constraints and examples. | Put detail where the task selects it; root help should not become every command's manual. |
| A working example | Required inputs or configuration, the command, relevant boundaries and how to inspect the result. | Syntax alone does not establish that a recipe applies or that its result is correct. |
| Guides | Broader workflows, alternatives, caveats and interpretation beyond an example's scope. | Preserve deeper reference without making it a prerequisite for every ordinary task. |
| Schemas | Complete machine-readable structure and field semantics. | An example cannot enumerate the full configuration or report contract. |

These placement rules are design judgments informed by the evidence below, not separately measured claims about every surface.

## Covering the Feature Set

Maintain a task inventory before declaring the help experience complete. The inventory below maps each supported workflow to its help destination and verification; it owns the current coverage map. The [full-help coverage plan](plans/full-help-coverage.md) records the implementation scope. Check newly added options against that inventory and the executable option guard in [help tests](../tests/integration/help.rs). A new feature needs a discoverable capability, a valid next action, its consequential boundary and observable verification; it does not automatically need another root example.

Use command help for workspace, comparison and explanation workflows. Root help owns measurement, output routing, scope, views and enough interpretation to avoid common false conclusions. Keep complete policy syntax in the schema and detailed alternatives in guides. Run the displayed recipes after terminal-width wrapping; a test that silently drops continuation lines can validate a different command from the one readers see.

## Current Feature Inventory

| User task | Supported features and material interpretation | Help destination | Verification |
| --- | --- | --- | --- |
| Discover what is possible | Measure, workspace, compare, explain; command help; standalone guides, six schemas and version | Root capability overview and reference options | CLI option inventory; discovery task |
| Understand a measurement | Files, Lines versus Code, purpose/category/kind, source versus all-counted, Code-based Test/prod, unclaimed versus intentionally Other, unavailable versus zero | Root result notes | Report-understanding task |
| Query or save machine evidence | Full JSON versus headline, `--files`, summary/breakdown/review/coverage, `--output -`, destination protection | Root output recipes and options | JSON/automation task; executable help |
| Save and show one measurement | `--output FILE --print table`, explicit Markdown, extension does not choose format, silent exports | Root output recipes | JSON/automation task; executable help |
| Automate and reproduce | Exit 0/1/2, partial documents, preserve status, `--no-user-config`, effective policy/coverage | Root automation notes | Script replay on complete/partial/invalid input |
| Select a root or subtree | Positional root, reserved names, `--root`, root-relative `--subfolder`, retained policy/module context, no ancestor discovery | Root selection recipes/options | Scope/views task |
| Adjust selection | Repeatable quoted exclusion globs, hidden and ignore booleans, personal exclusions, exclusions do not classify | Root selection recipe/options | Scope/views task; policy task |
| Explore human views | Languages, folders, adaptive rows, categories, purpose filtering, bars/comment bars/no bars, code/name sorting, Markdown depth | Root view recipes/options | Scope/views task; format-boundary guards |
| Classify maintained source | Existing configuration, narrow production rule, root-relative globs, verification and unresolved review | Existing tested root example | Simple configuration task; executable help |
| Express broader project policy | Exclusions, intentional Other, override precedence, rules, convention/module/fallback settings, language mappings | Root policy capability notes; configuration schema/guide for full semantics | Configuration and advanced-policy tasks |
| Understand personal defaults | Location, scope/language-only settings, interaction with project policy, opt out | Root policy notes and `--schema user-config` | Advanced-policy task |
| Understand inferred context | Build descriptors/conventions, whole-file classification, no build execution; submodule hints do not establish ownership | Root result/policy notes; verbose report/schema | Discovery and policy interpretation |
| Measure repositories together | Repeated roots, explicit workspace TOML, paths relative to file, unique non-overlapping member names, per-member policy | Workspace examples and notes | Workspace task; executable help |
| Inspect member contributions | Member-prefixed paths, `summary.members`, aggregate Code-weighted Test/prod, Markdown per-member detail | Workspace result notes | Workspace task |
| Compare saved measurements | Capture before/after full JSON, single roots or workspaces, explicit table/JSON/Markdown, net change not churn, matching restrictions, presence versus deletion, findings and partial inputs | Compare examples and notes; root entry command | Comparison and workspace tasks |
| Diagnose one file | Root-relative existing path, workspace MEMBER/PATH, counted/decision stages, excluded/not-selected/unrecognized/unread/parse-failed/symlink, format and exit behavior | Explain examples and notes | Explanation task; executable help |

## Adding an Example

- Start from a concrete user task and a missing decision or action. Add information that resolves that gap; do not add prose solely because the feature exists.
- State when the example applies and what requires a deeper reference. For a covered task, make optional reference reading explicitly optional. Avoid both unconditional “read the guide first” instructions and advice to bypass references for unsupported cases.
- Make the example executable with small, explicit inputs. Name the destination and path basis, preserve existing configuration, and state precedence or other constraints that materially affect the result. Do not imply that a complete sample file should replace an existing policy.
- Pair the action with observable verification. For classification, inspect per-file decisions and remaining review items; an empty review list alone does not prove correctness. Refer to the owning contract for exhaustive semantics.
- Prefer a compact example plus its applicability and verification over another navigation layer. Keep the short-help surface concise; assess the reading cost of root long-help additions on unrelated tasks.
- Keep one implementation owner for help wording and check any documented copies. Preserve frozen experimental fixtures as evidence; later edits are new candidates, not retroactive changes to a measured treatment.

## Evidence and Its Limits

The [guide-removal study](research/agent-explorability-six-task-study.md) found no consistent efficiency advantage from removing guides and included an interpretation failure without them. Retain optional guides; this does not prove they are necessary for every task.

The [configuration-example study](research/agent-explorability-help-recipe-study.md) did not show that an example alone reduced reference retrieval or task cost. This is evidence about that example and those tasks, not a rejection of usage examples generally.

The [routing study](research/agent-explorability-help-routing-study.md) found that the same example plus explicit conditional execution-first guidance reduced calls and uncached input on its covered task, with all four candidate agents skipping the configuration guide and schema. The [bounded replication](research/agent-explorability-routing-replication.md) then passed correctness and cost guards on broader configuration and unrelated report understanding across 54 fresh sessions. Together these support adopting the tested combination, retaining guides and leaving navigation unchanged. They do not isolate individual sentences, establish general equivalence, or newly replicate the original four-run covered-task benefit.

## Evaluating Future Additions

For a material new example or routing change, define the task, applicability boundary, expected artifacts and required interpretation before editing. Include a covered task, a task beyond the example's scope and an unrelated task that still reads root help. Freeze correctness and critical-error definitions before resource thresholds; the [benchmark](plans/agent-explorability-benchmark.md#proposed-scoring-rule) supplies the reusable contract.

Use fresh agents at the owner's requested tier, record the actual model and effort, compare one changed variable with the current interface, and balance execution order. Retain failures and recovery costs. Measure agent-facing calls, uncached input, cached input, output including reasoning, shell/CLI invocations, reference retrievals and final-delivery time separately. Do not infer efficiency from help length or fewer guide calls alone. Set a bounded sample and stopping rule before running; do not pool historical runs or repeat until a favorable result appears.

Routine factual corrections do not need another agent campaign. Every executable help example still needs a synthetic regression that runs the displayed configuration and command against an independent expected result. Check applicable help aliases, README agreement, affected guides and output parity, following the repository's [verification policy](../CLAUDE.md#testing-guidelines). Record what was tested and keep any broader recommendation within that evidence.
