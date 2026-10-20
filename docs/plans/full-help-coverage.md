# Complete CLI Help Coverage

<!-- doc-status
{
  "status": "active",
  "date": "2026-10-20",
  "summary": "Inventory all supported workflows, implement actionable help, and evaluate the combined interface.",
  "evidence": "[Help design](../help-design.md)"
}
-->

The owner clarified that the task is the complete agent-facing help experience, not one configuration example. The [earlier adoption](../research/agent-explorability-routing-replication.md) is one measured improvement within this scope. Continue autonomously through inventory, implementation, executable guards and fresh MEDIUM-agent evaluation. Do not equate a passing example with completion of the feature set or claim literal perfection from a finite sample.

## Feature and Use-Case Inventory

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

Every user-facing option must map to one of these tasks. Root help must expose the capability and the next executable action; command help owns command-specific recipes. Guides and schemas remain available for detail. Do not invent features such as Git churn, historical rescanning, per-file diff, rendering saved measurements, workspace-wide policy overrides or arbitrary format-independent purpose filtering.

## Completion Criteria

- All inventory rows have an actionable help route, applicable constraints and an observable verification path. New help examples execute on synthetic inputs; every visible option remains discoverable.
- Root and subcommand help, README entry points and PRODUCT agree. Preserve the adopted configuration wording and existing guide/schema content unless concrete evidence requires a separately recorded correction.
- Compare the current `c17fece` interface (the adopted help plus the shared Code-based workspace-ratio correction) with the combined help candidate using fresh MEDIUM agents, exact binaries, isolated inputs, captured calls/tokens/time and independent final-answer grading. Freeze tasks, oracles, schedule and decision thresholds before launch. Retain all valid failures and recovery costs.
- Resolve observed correctness defects and material discovery gaps; distinguish functional coverage from measured efficiency. If a cost guard fails, identify the route responsible and record a new candidate before corrective testing; do not relabel the failure or repeat unchanged trials until they pass.
- Record findings, verified limits and replay inputs, update contributor guidance, commit the completed implementation and close this active plan. No remaining row may be silently treated as outside scope.
