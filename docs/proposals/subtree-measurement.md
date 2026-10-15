# Subtree Measurement with Project Context

Status: proposed design for review, 2026-10-16. The owner requested this design; the option name `--subfolder` is accepted. Implementation is not authorized by this document. Commands and report additions below are proposed, not supported behavior. [PRODUCT](../PRODUCT.md) remains the supported-contract owner.

## Problem and Recommendation

Measuring `acme-orders/web/src` today makes that directory an independent scan root. It does not load `acme-orders/.locmetryrc` or see `web/package.json`; project-relative rules and module inference can therefore classify the same files differently. Loading the parent configuration with `--config` alone does not restore its path base or the missing module context.

A whole-project full report already supports exact subtree totals by summing its additive breakdown rows for the selected folder and descendants. The new feature adds a focused human view and limits content counting, coverage and measurement failure status to that subtree. It does not promise proportional speedups: context traversal still covers the selected project, and performance has not been measured. The existing breakdown query also supplies an independent acceptance oracle.

Add one explicit directory restriction, `--subfolder DIR`, to single-root measurement. Keep the existing root as the **project root**, which owns policy, paths and module context. The **measurement subtree** limits the files whose counts and coverage enter the report. This addresses the classification problem without changing what existing directory arguments mean.

| Scope | Decision and rationale |
| --- | --- |
| Required | One subtree, existing configuration and ignore policy, project-relative identities, all existing output formats, recorded selection metadata, and comparison awareness. A directory restriction must remain explainable and reproducible from its report. |
| Optional | Existing `--files` detail; no new file-detail mode. |
| Excluded | Ancestor configuration discovery, merged/nested configuration, multiple subtrees, file/glob selectors, workspace subtree selection, new `explain` flags, and per-file comparison. None is necessary to establish the two path bases. |

## Command and Path Contract

From the directory containing the synthetic [sample project](../../README.md#a-tour-of-a-sample-project):

```sh
locmetry acme-orders --subfolder web/src
locmetry --root acme-orders --subfolder web/src --files --output -
locmetry --root acme-orders --subfolder web/src --format markdown
```

Proposed help text: “Measure this root-relative folder and its descendants, retaining project configuration and module context.”

`--subfolder` is one literal directory path relative to the project root, never the working directory or configuration file. The project root still defaults to the working directory. Positional root selection and `--root` retain their existing equivalence and mutual exclusion. Repeating `--subfolder` is an error. The option is unavailable on subcommands and is not a project or personal configuration setting: it selects this measurement, not the project's policy.

Accept an existing directory inside the project root. Normalize `.` components and trailing separators; `--subfolder .` is equivalent to omitting the option, including its report semantics. Reject empty arguments, absolute paths, any `..` component, files, missing directories, and paths through symlinks. Require exact directory-entry spelling, consistent with [path explanations](../troubleshooting.md#choose-the-correct-policy), so case/Unicode aliases cannot silently select the wrong identity. The project root itself retains its existing resolution behavior.

The argument is a literal path, not a glob pattern; names containing glob metacharacters are treated literally. Individual files are not accepted. Directory boundaries are structural: `web/src` includes its descendants, not `web/src-old`. Existing selection rules remain authoritative; `--subfolder` cannot restore excluded, ignored or hidden files. If the target directory itself or an ancestor below the project root is excluded, ignored or hidden by policy, fail with exit 1 and no report. Establish directory selection directly, independently of whether it contains any selected files; an empty file list cannot distinguish an empty directory from a filtered directory. Name a recorded exclusion or a cause established by the existing single-setting diagnostic approach; when overlapping filters prevent attribution, say that the directory is not selected and the cause could not be established. Never invent a winning ignore-file rule. When a single-setting probe establishes the cause, name the applicable remedy (`--respect-ignore-files false` or `--hidden true`) and state that it changes policy for the whole project, including module context. For an explicit exclusion, point to the recorded pattern and its policy rather than implying that another flag can re-include it. A selected directory with no selected contents produces a valid empty measurement. A selected directory containing only unrecognized or failed files retains their coverage and applicable failure status rather than certifying zero content.

## Counting and Classification Invariant

For the same filesystem state, root and effective policy, every counted file inside the subtree must have the same counts, language, kind, purpose, category, mapping and deciding rule as in a whole-project measurement. The subtree report's totals are the aggregation of those restricted file records. Coverage is restricted by the same directory boundary; unavailable counts remain unavailable.

The subtree limit does not change the context used to discover modules. Apply the project's ordinary selection to context names before module detection: an excluded or ignored descriptor cannot become evidence just because it is above the subtree. Preserve nested-module precedence and layout evidence such as Python package markers, even when that evidence lies outside the measured subtree. Nested `.locmetryrc` files remain inactive, as today. If a `.locmetryrc` exists directly in the requested subfolder and is not the configuration actually loaded (including via `--config`), emit one stderr warning naming the inactive file and the effective configuration, or the absence of one. Do not load it, merge it, or change exit status. This alerts users switching from an independent-root run without recursively warning about every nested config; `--subfolder .` retains ordinary behavior.

Only files inside the subtree are counted or decoded as measurement content. An unreadable source file or malformed notebook elsewhere does not create subtree coverage failures or exit 2. Configuration and ignore processing still apply, and failure to establish project context through traversal is an error with no report, even if the traversal failure is outside the subtree. This is the same isolation boundary already used by focused explanations; it is not a promise that only the subtree is traversed.

## Report and Human Output

Keep `project` and `metadata.root` tied to the project root. All file, folder, module, coverage and provenance paths remain project-relative; a file remains `web/src/cart.ts`, never `cart.ts`. Effective configuration and pattern bases are unchanged. Output destinations remain outside the project root, including when a destination would lie outside the measured subtree, preserving the existing output-protection rule.

Add an optional `subfolder: string` to the effective selection receipt at `metadata.scope`, present in both headline and full reports. It holds the normalized project-relative directory, without a trailing slash. Absence means no subtree restriction; `null`, an empty string and `.` are not serialized alternatives. A minimal JSON fragment illustrating the addition is:

```json
{"metadata":{"scope":{"subfolder":"web/src"}}}
```

This fragment is not a complete report. The generated report schema must describe the field's path base, context/counting distinction and absence semantics. Historical reports lack this field because earlier producers did not support this restriction; they continue to describe their recorded whole-root selection with its existing exclusions. Preserve that absence when reading and reserializing them. Keep report schema version `1` as an additive field; older strict readers can reject the unfamiliar field and require an update. Do not silently strip the restriction for compatibility. New readers must reject a subtree receipt in workspace members while workspace subtree selection is unsupported. Nonvalidating consumers must inspect `metadata.scope.subfolder` before interpreting summaries as whole-project totals; record this obligation in the compatibility guide when the feature ships.

All numeric measurement summaries, breakdowns, review rows, file records and coverage lists describe only the measured subtree. `metadata.modules` retains the whole selected project module context, including modules with no measured files; it is classification context, not another counted-file set. In subtree mode, `summary.build_modules` includes only modules that classified at least one measured source file through a module rule; a counted file with zero code still qualifies. Each detector’s `modules` counts only those contributing modules, and its `code` sums only their measured contributions. Merely containing a file classified by an earlier rule is not a module contribution. Unrestricted reports retain their current module-summary meaning. Context-only modules belong in `metadata.modules`, not numeric summaries or the human contribution table. Human output may label context separately, but must not interpret a context module’s absent contribution as a misleading descriptor. Submodule shares likewise use measured file contributions.

Keep JSON `summary.top_folders` and breakdown identities project-relative with their existing aggregation meaning. For human output only, group the table by immediate children of the selected subfolder, with a separate direct-files row for files immediately in it. Child rows include descendants and partition the measured source exactly once. Labels remain project-relative, for example `web/src/components` and `web/src (direct files)`. Markdown starts its folder hierarchy at the selected subfolder and counts `--depth` from there: depth 1 shows immediate children and direct files, depth 2 also shows their children. Recursive Markdown parent rows are not additive, as today. Derive these human views from the same measured breakdown, without requiring `--files` or changing JSON folder identities.

Human output identifies both the project and restriction, for example `acme-orders · Source code · subfolder web/src`. No format presents these numbers as the whole project's totals. `summary.unused_entries` means entries that decided no measured file; human wording says “unused in this subtree”, not unused across the project. `summary.unused_exclusions` likewise concerns exclusions that excluded no candidate in the subtree, with target/ancestor exclusions instead causing the selection error above. Matches elsewhere neither clear nor create a subtree exclusion match. Schema descriptions must make these scopes explicit. A rule outside this view is not thereby a recommendation for removal. The configuration-authoring instruction to rerun until unused entries/exclusions disappear applies only to an unrestricted measurement. Qualify that instruction in the configuration schema, README and PRODUCT; never recommend pruning project policy from subtree-only evidence.

Retain existing command-outcome rules. Subtree selection alone does not imply partial measurement or exit 2; unread/parse failures and backend inaccuracy among measured files do. Configuration/path/context errors produce no measurement. See the current [command outcomes](../PRODUCT.md#command-outcomes) for the existing boundary this feature preserves.

## Comparison and Explanation

`compare` reads the saved restriction and never rescans. Comparing matching subtrees uses normal net-change semantics. Different `metadata.scope.subfolder` values, including present versus absent, are incompatible inputs: fail with exit 1 before producing a comparison or writing its output, naming both restrictions. Starting strict prevents selection-only differences from masquerading as growth or departure; the same eligibility rule applies if per-file comparison is introduced later. The measurement command omits the field for `--subfolder .`; readers reject a serialized `"."`, so comparison performs no extra normalization. Matching restrictions remain subject to ordinary policy and coverage findings; this check does not prove that all other selection settings agree. Retaining project-relative paths prevents artificial folder renames. Module/package presence continues to describe recorded project context; count changes describe the measured subtree. Human and JSON comparison input descriptions must retain the restriction so that identical project names cannot hide differing scope.

Use the existing explanation command for a file returned by the subtree report:

```sh
locmetry explain web/src/cart.ts --root acme-orders --format table
```

Use the same configuration and scope flags as the measurement. No `--subfolder` is needed: the target already limits counting to one file while retaining project context. The result explains ordinary selection and classification; it does not claim that a file outside a prior subtree was included in that measurement.

## Saved-Report Validation

Validate normalized `metadata.scope.subfolder` without consulting the filesystem. Reject null, empty strings, `.`, absolute paths, `..` or `.` components, repeated separators and trailing separators. Persist `/`-separated project-relative paths under the existing path contract; a backslash in a Unix filename is not an alternative separator. A saved receipt cannot establish that a directory still exists, has exact on-disk spelling or is not a symlink; those checks belong to measurement.

| Evidence | Boundary in a subtree report |
| --- | --- |
| `files`, `review`, `metadata.coverage.unrecognized`, `unread`, `parse_failures` | File paths strictly below the subfolder. No sibling-prefix matches. |
| `breakdown.folder`, `summary.unclassified_folders[].folder` | The subfolder itself or a descendant; no project-root `.` sentinel for a proper subfolder. |
| `summary.top_folders` | The project-top-level ancestor or the subfolder itself when it is top-level, derived and reconciled from the measured breakdown; a proper subfolder has no project-root direct-file contribution. |
| `summary.submodules[].path` | May contain the subfolder or lie inside it, but shares must concern measured files only. Disjoint sibling submodules cannot contribute. |
| `metadata.modules[].path` | May be anywhere in the selected project context, including ancestors and unrelated siblings; never treat it as measured-file evidence. |
| Effective configuration, exclusion patterns, configuration provenance | Retain project context; do not constrain these to the subfolder. |

Validate subtree-specific module-summary semantics as well as existing arithmetic and identity invariants. When optional file records are absent, use available breakdown/provenance evidence and structural bounds without claiming stronger verification than the report supports. Comparison folder rows for ancestors such as `web` aggregate measured contributions only.

## Acceptance Criteria

- With independently known synthetic counts, subtree records and totals equal the corresponding whole-project subset. Cover explicit rules/exclusions, test conventions, ancestor descriptors, nested modules and Python package evidence outside the subtree.
- An excluded ancestor descriptor stays ineffective. Unrelated malformed/unreadable source does not affect subtree coverage; a context traversal failure fails clearly. Selected-file coverage still reconciles across counted, unrecognized, unread and parse-failed files.
- Directory boundary, path validation, symlink and alias cases behave as specified. Excluded, gitignored and hidden target directories or ancestors fail without a report, including overlapping filters with unknown attribution. Selected empty directories and selected directories whose children are all filtered remain valid; `--subfolder .` matches an unrestricted measurement apart from ordinary volatile metadata.
- Project-relative identities, module context, subtree-only module contributions and scoped unused-entry/exclusion findings agree across JSON, table and Markdown. Check that an exclusion matching only a sibling does not count as used in the subtree, and that an entry deciding only sibling files is labeled unused in this subtree. Module summaries omit context-only modules, retain a module classifying a zero-code file, and preserve unrestricted behavior. Human immediate-child rows reconcile independently with breakdown totals, including direct files; Markdown’s default depth reveals subtree children.
- Headline/full schemas describe and preserve the restriction. Exercise every boundary in the validation table, including review/failure paths, allowed ancestor summaries and context-only modules; reject every malformed restriction listed above and restrictions in workspace members. Derived summary totals still reconcile; historical fixtures round-trip without invented restrictions.
- Comparison refuses differing restrictions, including absence versus presence, without touching output; matching restrictions remain eligible. Explanation matches the same file's subtree record. Output protection still covers the whole project root.
- Repeat the descriptor-removal oracle with `--subfolder web/src` on both sides: removing `web/package.json` removes its module context and transfers its source from production to unclassified while source and all-counted subtree line totals stay unchanged, because the descriptor was never measured in this view.
- Reject `--subfolder` on `workspace`, `compare` and `explain`. Check the single inactive-subfolder-config warning, its lack of effect on policy/status, and its absence when that config is explicitly loaded or the restriction is `.`.

## Documentation Ownership

This proposal owns only the proposed design until acceptance. When implemented, [PRODUCT](../PRODUCT.md) owns the supported selection, report and comparison contract; generated report/comparison schemas expose its structure and semantics. The [configuration schema guidance](../../src/config.rs) must restrict its unused-entry/exclusion cleanup instruction to unrestricted measurements. [Report compatibility](../report-compatibility.md) must tell consumers to inspect `metadata.scope.subfolder` before interpreting summary totals as whole-project totals. The [README](../../README.md) needs executable sample-project commands and troubleshooting guidance contrasting an independent subdirectory root with `--subfolder`. [Measurement semantics](../measurement.md) owns context traversal and coverage limits, and [troubleshooting](../troubleshooting.md) explains how existing file explanations relate to the restricted measurement. The [work plan](../plans/measurement-quality-and-net-change.md) records verified acceptance evidence. Retire the proposal as a decision record after implementation instead of maintaining a second live contract.

## Feasibility and Review Focus

The existing [selection boundary](../../src/backend.rs) discovers selected names and modules before restricting `explain` to one target, supporting this context/counting separation. This is evidence of feasibility, not proof of subtree correctness. [Module detection](../../src/modules.rs) includes path-dependent Python layouts, so looking only for ancestor descriptor filenames would be insufficient. [The typed report contract](../../src/contract.rs) and [comparison policy checks](../../src/compare/engine.rs) already own selection receipts and policy findings; the extension must use those owners, not a second representation.

The main tradeoffs are deliberate: preserve classification parity even when context traversal reaches outside the subtree; retain project-relative JSON identities while focusing human grouping beneath the subfolder; and retain module context separately from measured contribution. Review whether those tradeoffs satisfy the intended exploration workflow before implementation. Ancestor discovery remains a separate proposal decision: finding a parent config must ultimately select this same project-root/subtree pair, not merely change the config filename.

## External Review Disposition — 2026-10-16

Accepted findings 1–8: reject an unselected target directory; require matching comparison restrictions; qualify configuration-cleanup guidance; restrict module summaries to measured contributions; specify field-level validation; focus human folder views; acknowledge the existing full-report workaround; and require scope-aware consumers. The proposal incorporates their acceptance cases. Accept the inactive subfolder-config warning as a narrow transition aid, retain the field in `metadata.scope`, and reject malformed serialized restrictions instead of normalizing them in comparison. These are proposed contract corrections, not implementation completion claims.

Source checks confirmed whole-project module discovery before focused counting, all-adopted-module counting in `build_systems`, and ignore filtering before the walker’s custom exclusion callback. Directory selection needs evidence even for an empty directory; the existing file-only selected-path list alone is insufficient. Performance and the new behavior remain untested. Executable config-schema inspection also confirmed accepted language names and aliases already exist; the work plan withdraws the redundant language-list recommendation.
