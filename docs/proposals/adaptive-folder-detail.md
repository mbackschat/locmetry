# Adaptive Folder Detail in Terminal Reports

Status: historical decision record. The owner approved implementation on 2026-10-16; `--folder-rows N` is implemented in the development checkout, unreleased after v0.2.0. [PRODUCT](../PRODUCT.md#adaptive-folder-detail) owns the supported contract; the [work plan](../plans/measurement-quality-and-net-change.md#adaptive-folder-detail--2026-10-16) records verification. The sections below preserve the reviewed design and acceptance criteria, not an active proposal. Do not update this record to track behavior.

## Problem and Recommendation

The terminal Folder table partitions source code into top-level folders and root files. When most code sits under one container such as `packages`, the largest row hides the structure users need to inspect. Running a child directory as an independent root can also change its configuration and module context; [subfolder measurement](../PRODUCT.md#subfolder-measurement) preserves that context but still requires another invocation.

Recommend an opt-in adaptive view that replaces large folder rows with deeper, non-overlapping rows. Every measured source contribution still appears exactly once, and displayed purpose totals still reconcile with the report total. Use existing measured folder aggregates; do not rescan or require `--files`.

The current alternatives remain useful. Markdown already provides a recursive hierarchy controlled by `--depth`, with parent totals that overlap their children. A full JSON report provides additive breakdown rows at every depth. Adaptive terminal detail adds a compact, additive overview without changing the measurement boundary.

## Scope

| Scope | Recommendation and rationale |
| --- | --- |
| Required | Single-root terminal Folder table, including `--subfolder`; project-relative labels; bounded additional detail; deterministic selection; existing purpose and ratio columns; unchanged measurement and JSON. These directly address the hidden structure. |
| Optional at invocation | Select an adaptive row budget with `--folder-rows N`; use the existing `--sort name` to change presentation order. Omitting the new option keeps today’s table. |
| Excluded | Workspace member drilldown, adaptive Markdown, saved-report rendering commands, configuration-file settings, persistent preferences, automatic default changes, interactive expansion, file listings, quality thresholds and changes to `compare` or `explain`. Workspace output currently uses repository rows instead of a Folder table; extending it is a separate design. |

## Proposed Interface

```sh
locmetry acme-orders --folder-rows 30
locmetry acme-orders --subfolder packages --folder-rows 12
locmetry acme-orders --format table --folder-rows 30 | less
```

Recommend one opt-in option, `--folder-rows N`, with a required positive-integer value and no default. Passing it enables adaptive detail; omitting it preserves current behavior. The budget counts data rows, excluding headers, separators, legends and the total. Requiring the value avoids an optional-value ambiguity with the positional root; a nonnumeric value is an argument error.

Proposed help text: “Split the largest folders into subfolders within an N-row budget; no top-level folder’s code is merged away.” The soft-budget exception is defined below, so the help must not promise an absolute cap.

Reject the option whenever the resulting format is not `table`, whether the format was explicitly selected or inferred from stdout. For example, `locmetry acme-orders --folder-rows 30 | less` infers JSON and must fail with a hint to add `--format table`. Reject `--output -` and use with any subcommand, including `workspace` even though it shares presentation options. `--output FILE` may still save the ordinary full JSON report alongside a valid terminal view. Unsupported combinations fail before writing any report.

Keep `--depth` specific to existing Markdown output. It neither controls adaptive expansion nor changes its row selection. The existing top view and Markdown depth provide predictable alternatives; this proposal adds no second depth option.

## Row Selection and Reconciliation

The view is a partition of measured source, not a tree of overlapping subtotals. A folder row includes that folder’s direct source files and descendants until expanded. Root direct files retain their existing row. In subfolder mode, start from the existing immediate-child rows and the selected folder’s direct-files row; never display contributions outside the selected measurement.

Use the following selection rules, independent of final row order:

1. Start with the existing Folder rows. Preserve all these sibling contributions; do not hide small top-level folders to make room for a large one.
2. A folder with no direct source files and only one source-bearing child may be replaced by that child at no row cost. Collapse such chains on every initial and newly revealed folder row, repeatedly to the first branching folder or a folder with direct source files. Keep the full project-relative path.
3. Among rows with positive code that can reveal additional detail within the remaining budget, expand the one with the greatest recursive source code. Break ties by the original project-relative path, using the same name ordering as ordinary output. No percentage threshold defines dominance.
4. Replace the selected parent with its source-bearing immediate children and, if present, a separate direct-files row. Never retain the parent subtotal alongside these replacement rows.
5. When all replacements fit, show them all. When they do not, preserve the direct-files row if present, show at least two named children ranked by code, and aggregate the remaining children into one scoped remainder row. Reveal as many named children as the budget permits. A remainder must contain at least two children; if only one would remain, show that child by name at the same row cost. If the minimum of two named children, a valid remainder and any direct-files row cannot fit, leave the parent intact and consider the next expandable folder.
6. Continue until no further expansion fits. Direct-files rows and remainder rows are not expansion candidates. Zero-code source contributions remain represented, but do not drive further expansion.

A source-bearing child has at least one measured source file, including files with zero code. Non-source data and documentation do not create folder rows. Remainders include all contributions of omitted child folders, including their descendants and zero-code source files. Use directory-component containment, so `src` never includes `src-old`.

The budget is deliberately **soft only for the initial rows**: if the original table already exceeds it, retain that table and perform only cost-free chain collapse. Never add rows beyond the greater of the initial row count and the budget. This preserves the familiar overview and avoids a second policy for hiding top-level siblings. It also means an unusually wide root needs a larger explicit budget to gain detail; that is a known limitation for review.

All code and purpose columns come from exactly the source contributions represented by each row. Compute `Test/prod` from that row’s summed test and production code, never by averaging child ratios. Keep the existing undefined/zero treatment and neutral ratio styling. The total row is unchanged. The same accounting must hold when comments or blanks exist, when code is unclassified, and when some selected files have unavailable counts; expansion must not turn unavailable evidence into counted zero or change coverage notes and exit status.

## Labels and Ordering

Keep full project-relative paths even when displayed rows have different depths. Expanded parents disappear as numeric rows. Direct-file and remainder labels are display annotations, not real folder identities:

- `packages (direct files)` represents only files immediately in `packages`.
- `packages (other 7 child folders)` represents the seven immediate children omitted at that expansion, including their descendants.
- `(root files)` retains its current meaning.

Reuse the shipped direct-files label convention and shared terminal-label escaping and presentation roles. A real folder name can theoretically look identical to a display annotation; this is a terminal-label limitation, not a uniqueness guarantee. JSON path identities remain unchanged and unambiguous. This feature does not introduce a new path-quoting convention.

Apply the existing `--sort code` or `--sort name` to the final rows. Selection always uses code, so changing the sort order changes presentation only. Aggregate annotations use their represented folder as the name key with a stable row-kind tie-break; code ties use that same key. A remainder’s value may exceed a named child because it aggregates multiple children; its label must make that explicit.

Add a muted note: “Adaptive folder detail; rows cover measured source once.” If no expansion fits because the initial table exhausts the budget, explain that briefly and name `--folder-rows` as the way to request more detail. Do not imply that adaptive expansion discovers oversized or problematic folders.

## Synthetic Example

Assume a synthetic project has `packages` with 720 code lines, `examples` with 180, `scripts` with 80 and 20 root-file lines. Inside `packages`, `core` has 400, `web` 220, `tools` 80 and direct files 20. The `core`, `web`, `tools`, `examples` and `scripts` folders have no subfolders.

With an eight-row budget, the resulting Folder/Code slice is:

| Folder | Code |
| --- | ---: |
| packages/core | 400 |
| packages/web | 220 |
| examples | 180 |
| packages/tools | 80 |
| scripts | 80 |
| (root files) | 20 |
| packages (direct files) | 20 |
| **Total** | **1,000** |

The full view retains Production, Test, Unclassified and Test/prod. There is no additional `packages: 720` subtotal to double-count. The view stops at seven rows because no folder can expand further. If another expansion fit, selection would continue; it does not stop early at a preferred amount of detail.

## Tradeoffs and Review Decisions

| Decision | Recommendation and rationale |
| --- | --- |
| Adaptive detail versus fixed depth | Use adaptive detail for the terminal overview. Fixed depth can explode on wide trees and still hide a deep concentration; existing Markdown depth remains available. |
| Replacement rows versus an indented subtotal tree | Use replacement rows so the terminal table remains additive. Full paths preserve ancestry without repeating totals. |
| Strict cap versus preserved initial siblings | Preserve initial siblings with the soft budget above. A strict cap would require hiding existing top-level detail; revisit only if wide-root examples show this limitation defeats the feature. |
| Budget and remainder policy | Let the caller supply N; no default budget or second option is needed. Permit scoped remainders containing at least two children so wide containers can expand within a useful budget. |
| Default and format scope | Keep this opt-in and terminal-only to limit scope and preserve existing examples and output expectations. Opt-in does not solve discoverability for users who never find the flag; reconsider a default only after actual use. Markdown already has a distinct hierarchy contract. |

Identical measured input and options must produce identical rows, but small count changes can alter which folder is expanded or included in a remainder. This is a navigation aid, not stable row identity for comparing snapshots. The saved report and aggregate comparison identities remain unchanged. No traversal or runtime improvement is claimed; rendering reuses already available folder summaries, but its cost has not been measured.

## Acceptance Criteria

- Independent synthetic counts reconcile every displayed code/purpose column with the original measurement, with no parent/child overlap. Totals use sums, and ratios use the summed numerator and denominator.
- Cover one dominant folder, several large siblings, wide branching with a remainder of at least two children, a would-be single-child remainder shown by name, deep single-child chains on initial and newly revealed rows, mixed direct files and children, test-only and unclassified rows, zero-code source files and an empty source result. Include Maven-style `src/main` production and `src/test` test trees: exposing separate purpose-only rows is deliberate even when the purpose columns already convey the split.
- Verify exact budget boundaries, budgets below the initial row count, insufficient room for a useful wide expansion, deterministic ties and identical selected contributions under both sort orders.
- Verify subtree containment and project-relative labels, including sibling-prefix traps. Retain ordinary coverage/failure notes and command outcomes.
- Check existing annotation conventions, control characters, ANSI-stripped alignment, `NO_COLOR` and explicitly requested piped table output; do not assert uniqueness for terminal labels.
- Reject missing, zero and nonnumeric budgets and incompatible invocations clearly, including inferred JSON in a pipe and `workspace` with otherwise valid arguments. Check the `--format table` remedy and that invalid combinations write no report. The default table, Markdown depth behavior, workspace repository view, JSON schema/content and comparison semantics remain unchanged. Verify that the Unclaimed source note and `summary.unclassified_folders` remain unchanged, even when their partition differs from the adaptive Folder table.
- Update the supported [PRODUCT contract](../PRODUCT.md#reporting-and-coverage), [README workflows](../../README.md), [measurement semantics](../measurement.md) and CLI help only when implementation is approved and verified. The proposal is not evidence that this behavior has shipped.

## Existing Partition and Feasibility

Source inspection on 2026-10-16 confirms that [report aggregation](../../src/report.rs) (`build_report`) records direct and recursive totals for each ancestor. [Terminal rendering](../../src/output.rs) selects top-level or subfolder-immediate rows through `human_top_folders`, while `human_folders` already uses those summaries for Markdown and subfolder direct-files rows. Workspace mode substitutes repository rows. The feature needs no new filesystem selection, counting mechanism or rescan, but it does introduce a distinct display partition.

The existing `unclassified_folders` routine in [report aggregation](../../src/report.rs) already produces an adaptive, non-overlapping partition: it collapses single-child chains, splits the largest eligible folder, normally uses `FOLDER_ROWS = 8`, and allows a split beyond that budget when a folder holds more than half the unclassified code. It does not split folders with direct files and has no remainder rows. This result serves both the Unclaimed source note and JSON `summary.unclassified_folders`, including the README’s agent workflow.

The different policies are intentional here. The existing note locates the bulk of unclaimed source, so its dominance exception can be useful. The proposed table describes all measured source within a caller-selected expansion budget; direct-files and remainder rows allow detail without overlapping counts or unbounded expansion. The same folder may therefore be split differently in the table and the note, both because the source populations differ and because their display policies differ.

Keep the new display partition separate and preserve `unclassified_folders` output. Reuse component-aware containment and the single-child-collapse rule where practical; the current collapse logic is a local closure over unclassified files, not an already reusable helper. Any narrow extraction must preserve that existing population and behavior, without adding a generic partition-policy framework. Whether the two policies should converge is a separate deferred decision in [PLAN](../PLAN.md#known-display-follow-up), requiring review of JSON consumers; this proposal does not authorize convergence.

[CLI rendering](../../src/main.rs) currently uses `--categories` only for table output and `--depth` only for Markdown, silently ignoring them elsewhere. Rejecting unsupported `--folder-rows` combinations is deliberately stricter. The same [plan section](../PLAN.md#known-display-follow-up) records existing-flag consistency as a separate cleanup decision, rather than broadening this change.

At design review, no prototype, performance test or executable acceptance check had been run. The external review of `0bd0eac` confirmed all seven findings were addressed; final wording corrections landed in `a07fa3b`. The owner subsequently approved implementation. Its verification and remaining limits are recorded in the linked work plan.
