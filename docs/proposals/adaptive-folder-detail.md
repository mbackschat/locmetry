# Adaptive Folder Detail in Terminal Reports

Status: proposed 2026-10-16; awaiting external review and owner approval. This document recommends a display-only feature; no implementation is authorized. [PRODUCT](../PRODUCT.md#reporting-and-coverage) remains the supported contract, and [PLAN](../PLAN.md#proposals--under-discussion) indexes this proposal.

## Problem and Recommendation

The terminal Folder table partitions source code into top-level folders and root files. When most code sits under one container such as `packages`, the largest row hides the structure users need to inspect. Running a child directory as an independent root can also change its configuration and module context; [subfolder measurement](../PRODUCT.md#subfolder-measurement) preserves that context but still requires another invocation.

Recommend an opt-in adaptive view that replaces large folder rows with deeper, non-overlapping rows. Every measured source contribution still appears exactly once, and displayed purpose totals still reconcile with the report total. Use existing measured folder aggregates; do not rescan or require `--files`.

The current alternatives remain useful. Markdown already provides a recursive hierarchy controlled by `--depth`, with parent totals that overlap their children. A full JSON report provides additive breakdown rows at every depth. Adaptive terminal detail adds a compact, additive overview without changing the measurement boundary.

## Scope

| Scope | Recommendation and rationale |
| --- | --- |
| Required | Single-root terminal Folder table, including `--subfolder`; project-relative labels; bounded additional detail; deterministic selection; existing purpose and ratio columns; unchanged measurement and JSON. These directly address the hidden structure. |
| Optional at invocation | A larger row budget and the existing `--sort name` presentation order. Users can trade compactness for detail explicitly. |
| Excluded | Workspace member drilldown, adaptive Markdown, saved-report rendering commands, configuration-file settings, persistent preferences, automatic default changes, interactive expansion, file listings, quality thresholds and changes to `compare` or `explain`. Workspace output currently uses repository rows instead of a Folder table; extending it is a separate design. |

## Proposed Interface

```sh
locmetry acme-orders --format table --folder-view adaptive
locmetry acme-orders --format table --folder-view adaptive --folder-rows 30
locmetry acme-orders --subfolder packages --format table --folder-view adaptive
```

Recommend `--folder-view top|adaptive`, with `top` preserving current behavior, and `--folder-rows N` as the adaptive view’s positive-integer row budget, default `20`. The budget counts data rows, excluding headers, separators, legends and the total. Twenty is an initial readability choice to review, not a measured optimum or code-quality boundary.

The view applies only to single-root measurement with effective `--format table`. Reject explicit use with Markdown, JSON, `--output -`, or subcommands instead of silently ignoring it. `--folder-rows` requires explicitly selecting `--folder-view adaptive`. Ordinary redirected-output format selection remains unchanged; use `--format table` to request this view in a pipe. `--output FILE` may still save the ordinary full JSON report alongside the terminal view.

Keep `--depth` specific to existing Markdown output. It neither controls adaptive expansion nor changes its row selection. The existing top view and Markdown depth provide predictable alternatives; this proposal adds no second depth option.

## Row Selection and Reconciliation

The view is a partition of measured source, not a tree of overlapping subtotals. A folder row includes that folder’s direct source files and descendants until expanded. Root direct files retain their existing row. In subfolder mode, start from the existing immediate-child rows and the selected folder’s direct-files row; never display contributions outside the selected measurement.

Use the following selection rules, independent of final row order:

1. Start with the existing Folder rows. Preserve all these sibling contributions; do not hide small top-level folders to make room for a large one.
2. A folder with no direct source files and only one source-bearing child may be replaced by that child at no row cost. Collapse such chains to the first branching folder or a folder with direct source files. Keep the full project-relative path.
3. Among rows with positive code that can reveal additional detail within the remaining budget, expand the one with the greatest recursive source code. Break ties by the original project-relative path, using the same name ordering as ordinary output. No percentage threshold defines dominance.
4. Replace the selected parent with its source-bearing immediate children and, if present, a separate direct-files row. Never retain the parent subtotal alongside these replacement rows.
5. When all replacements fit, show them all. When they do not, preserve the direct-files row if present, show at least two named children ranked by code, and aggregate the remaining children into one scoped remainder row. Reveal as many named children as the budget permits. If that minimum cannot fit, leave the parent intact and consider the next expandable folder.
6. Continue until no further expansion fits. Direct-files rows and remainder rows are not expansion candidates. Zero-code source contributions remain represented, but do not drive further expansion.

A source-bearing child has at least one measured source file, including files with zero code. Non-source data and documentation do not create folder rows. Remainders include all contributions of omitted child folders, including their descendants and zero-code source files. Use directory-component containment, so `src` never includes `src-old`.

The budget is deliberately **soft only for the initial rows**: if the original table already exceeds it, retain that table and perform only cost-free chain collapse. Never add rows beyond the greater of the initial row count and the budget. This preserves the familiar overview and avoids a second policy for hiding top-level siblings. It also means an unusually wide root needs a larger explicit budget to gain detail; that is a known limitation for review.

All code and purpose columns come from exactly the source contributions represented by each row. Compute `Test/prod` from that row’s summed test and production code, never by averaging child ratios. Keep the existing undefined/zero treatment and neutral ratio styling. The total row is unchanged. The same accounting must hold when comments or blanks exist, when code is unclassified, and when some selected files have unavailable counts; expansion must not turn unavailable evidence into counted zero or change coverage notes and exit status.

## Labels and Ordering

Keep full project-relative paths even when displayed rows have different depths. Expanded parents disappear as numeric rows. Direct-file and remainder labels are display annotations, not real folder identities:

- `packages (direct files)` represents only files immediately in `packages`.
- `packages (other 7 child folders)` represents the seven immediate children omitted at that expansion, including their descendants.
- `(root files)` retains its current meaning.

An annotated row must remain distinguishable from a real folder with the same text; use unambiguous quoting or annotation placement if a collision occurs. Preserve original path identities in JSON and use the shared terminal-label escaping and presentation roles.

Apply the existing `--sort code` or `--sort name` to the final rows. Selection always uses code, so changing the sort order changes presentation only. Aggregate annotations use their represented folder as the name key with a stable row-kind tie-break; code ties use that same key. A remainder’s value may exceed a named child because it aggregates multiple children; its label must make that explicit.

Add a muted note: “Adaptive folder detail; rows cover measured source once.” If no expansion fits because the initial table exhausts the budget, explain that briefly and name `--folder-rows` as the way to request more detail. Do not imply that adaptive expansion discovers oversized or problematic folders.

## Synthetic Example

Assume a synthetic project has `packages` with 720 code lines, `examples` with 180, `scripts` with 80 and 20 root-file lines. Inside `packages`, `core` has 400, `web` 220, `tools` 80 and direct files 20. Assume no further branching below those children.

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

The full view retains Production, Test, Unclassified and Test/prod. There is no additional `packages: 720` subtotal to double-count. Seven rows are enough to expose the structure; the budget is a ceiling on added detail, not a target to fill.

## Tradeoffs and Review Decisions

| Decision | Recommendation and rationale |
| --- | --- |
| Adaptive detail versus fixed depth | Use adaptive detail for the terminal overview. Fixed depth can explode on wide trees and still hide a deep concentration; existing Markdown depth remains available. |
| Replacement rows versus an indented subtotal tree | Use replacement rows so the terminal table remains additive. Full paths preserve ancestry without repeating totals. |
| Strict cap versus preserved initial siblings | Preserve initial siblings with the soft budget above. A strict cap would require hiding existing top-level detail; revisit only if wide-root examples show this limitation defeats the feature. |
| Budget and remainder policy | Start with 20 rows and permit scoped remainders during expansion. This balances readability and wide-folder usefulness; review synthetic wide and deep cases before approving the value. |
| Default and format scope | Keep this opt-in and terminal-only. Adaptive rows can change when counts change; users should choose that tradeoff, and Markdown already has a distinct hierarchy contract. |

Identical measured input and options must produce identical rows, but small count changes can alter which folder is expanded or included in a remainder. This is a navigation aid, not stable row identity for comparing snapshots. The saved report and aggregate comparison identities remain unchanged. No traversal or runtime improvement is claimed; rendering reuses already available folder summaries, but its cost has not been measured.

## Acceptance Criteria

- Independent synthetic counts reconcile every displayed code/purpose column with the original measurement, with no parent/child overlap. Totals use sums, and ratios use the summed numerator and denominator.
- Cover one dominant folder, several large siblings, wide branching with a remainder, deep single-child chains, mixed direct files and children, test-only and unclassified rows, zero-code source files and an empty source result.
- Verify exact budget boundaries, budgets below the initial row count, insufficient room for a useful wide expansion, deterministic ties and identical selected contributions under both sort orders.
- Verify subtree containment and project-relative labels, including sibling-prefix traps. Retain ordinary coverage/failure notes and command outcomes.
- Check annotated-label collisions, control characters, ANSI-stripped alignment, `NO_COLOR` and explicitly requested piped table output.
- Reject incompatible invocations clearly. The default table, Markdown depth behavior, workspace repository view, JSON schema/content and comparison semantics remain unchanged.
- Update the supported [PRODUCT contract](../PRODUCT.md#reporting-and-coverage), [README workflows](../../README.md), [measurement semantics](../measurement.md) and CLI help only when implementation is approved and verified. The proposal is not evidence that this behavior has shipped.

## Feasibility and Evidence

Source inspection on 2026-10-16 confirms that [terminal rendering](../../src/output.rs) selects top-level or subfolder-immediate rows through `human_top_folders`, while `human_folders` already accesses direct and recursive folder summaries for Markdown. The same file substitutes repository rows in workspace mode. [CLI options](../../src/main.rs) currently document `--depth` as Markdown-only. These observations support a presentation-only design without a second selection/counting mechanism; no prototype, performance test or executable acceptance check has been run for this proposal.
