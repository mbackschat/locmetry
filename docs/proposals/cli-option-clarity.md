# CLI Option Clarity and Applicability

<!-- doc-status
{
  "status": "draft",
  "date": "2026-10-20",
  "summary": "Clarify option names and help, extend file detail across output views, and preserve existing warning and success behavior.",
  "evidence": ""
}
-->

This proposal is under external review, not approved for implementation. [PRODUCT](../PRODUCT.md#output-routing) remains the supported contract. The assessment covers root measurement, workspace, comparison, explanation, standalone queries, help and their option interactions at revision `185c1bc`.

## Recommendation

Make optional table additions recognizable through `--show-*`, name format-specific filters explicitly, align workspace-file selection across commands, and extend `--files` to any requested full JSON view. Preserve existing warning, success and rejection behavior for other options, established names, compact machine output, single-scan exports and bounded task-oriented help.

The original concern is valid, but a prefix alone does not solve the interface problem. `--subfolder` selects measured input; `--folders` selects a table; `--purpose` changes table totals while leaving JSON totals untouched. Agents and people need to distinguish measurement selection, report content, view filtering and decoration without reconstructing those differences from several documents.

This is a design judgment, not a claim that bare-noun options violate a naming standard. The [Command Line Interface Guidelines](https://clig.dev/#arguments-and-flags) recommend descriptive long names and established conventions; their [compatibility guidance](https://clig.dev/#future-proofing) favors additive changes. No universal `--show-*` convention is prescribed. Broad rejection of ineffective options is a [separate deferred compatibility decision](#deferred-compatibility-decision), not part of the required scope.

## Findings and Evidence

The sweep inspected [argument definitions, routing and validation](../../src/main.rs), [rendering](../../src/output.rs), the [public contract](../PRODUCT.md), [report guide](../guides/reports.md), and existing [CLI](../../tests/integration/cli.rs), [help](../../tests/integration/help.rs), [folder](../../tests/integration/folders.rs), [adaptive-folder](../../tests/integration/folder_rows.rs), [purpose](../../tests/integration/purposes.rs) and [query](../../tests/integration/queries.rs) guards.

| Finding | Exact mechanism and consequence | Assessment |
| --- | --- | --- |
| Selection and presentation share ambiguous names | `Cli.subfolder` restricts measurement; `Cli.folders` only adds table rows. `SharedArgs.purpose` only filters table source, totals and ratios. | Rename display additions and the purpose filter; never imply that the latter filters a saved JSON baseline. |
| Ineffective requests have three outcomes | `warn_ineffective_options` warns for table controls and explicit depth; `run_root` rejects ineffective folder-row budgets; explicit `--sort` on JSON silently succeeds. | Document the existing outcomes and preserve them here; assess blanket rejection separately because wrappers can rely on successful ignored preferences. |
| File-detail validation ignores a valid secondary view | `destination()` requires the primary format to be JSON before assigning `--print`; `report()` can construct full JSON for either view. Table and Markdown controls already use `Destination::shows`. | Validate per-file detail against either full JSON destination. This is an interface asymmetry, not a missing renderer. |
| Adaptive-folder prose overstates the format restriction | PRODUCT's Adaptive Folder Detail and root help imply `--format table` is required, but `run_root` accepts a saved JSON report with `--print table --folder-rows N`. | Describe applicability across views once, then link to it and correct the root-help restriction. |
| Help has no semantic option grouping | All four command surfaces place their flags under `Options`; root selection and presentation flags are adjacent. | Group options by task and effect, preserving command-specific scope. |
| Workspace-file selection has two names | `workspace --file FILE` and `explain --workspace FILE` select the same kind of input. `--files` elsewhere means emitted per-file records. | Use `--workspace-file FILE` for both input selectors. |
| Some descriptions no longer match behavior precisely | `BarArg::Comments`, bars help and the purpose-test module comment describe Code against Comments; rendering scales comment density to the densest row. Categories help says “terminal” although saved tables work. The `--files` option help implies `--output FILE` is sufficient even with explicit non-JSON format; its error diagnostic already excludes a conflicting format. | Correct the stale help and comments; preserve the diagnostic's qualification while extending its remedy to a secondary JSON view. |
| An explicit chart kind has a hidden dependency | `PurposeColumns::bar_kind` substitutes comment-density bars when a single purpose is shown, even if `purpose` was explicitly requested. | Preserve the current rendering decision, but expose the single-purpose exception in bar help; do not promise an unconditional purpose split. |
| Context-dependent defaults can defeat a visual request | In a pipe, `--folders` alone returns a headline plus a warning. `--output FILE` selects JSON regardless of the extension. The production-only root-help example omits an explicit table format. | Keep defaults and outcomes; make explicit format selection prominent in executable visual recipes and applicable diagnostic remedies. |
| Markdown depth is not a general detail limit | `human_folders` limits Folder breakdown rows; review paths and other evidence are rendered separately. Selected subfolders change the origin, and workspace member names consume a level. | Name the format and define the exact depth boundary without promising a bound on all Markdown content. |
| Several asymmetries are intentional and documented | Compare has no `--print`; explain has no `--output` or headline; workspace rejects root-wide policy overrides and single-root folder views. | Preserve these boundaries; uniform spelling does not require identical capabilities. |

On 2026-10-20, ten synthetic invocations against the existing local debug binary corroborated the source-level routing findings. This was an assessment probe, not a fresh build or implementation test run. The fixture had one production and one test code line; personal defaults were disabled, stdout was captured, and destinations were outside the fixture root.

| Current invocation, after `locmetry --root PROJECT --no-user-config` | Observed outcome |
| --- | --- |
| `--format json --folders` | Exit 0, table-only warning, full JSON with both code lines. |
| `--format json --purpose production` | Exit 0, table-only warning, full JSON with both code lines. |
| `--format json --sort name` | Exit 0, no warning. |
| `--format table --depth 2` | Exit 0, Markdown-only warning. |
| `--format json --folder-rows 10` | Exit 1, no stdout. |
| `--output SAVED --format table --print json --files` | Exit 1, despite the requested full JSON view. |
| `--output SAVED --format headline --print json --files` | Same rejection with a second primary format. |
| `--output SAVED --print table --folders` | Exit 0, no warning. |
| `--output SAVED --print table --folder-rows 10` | Exit 0, no warning. |
| `--folders` | Exit 0, table-only warning, headline output. |

The two primary-format reproductions establish the file-detail mechanism beyond one command spelling. Source inspection establishes that it also belongs to workspace's shared routing. Workspace-specific execution of that case remains an implementation acceptance requirement.

## Required Public Contract

### Names and Meaning

| Canonical spelling | Accepted existing spelling | Contract and reason |
| --- | --- | --- |
| `--show-folders` | `--folders` | Add the single-root folder table; no measurement restriction. Distinguishes it from `--subfolder DIR`. |
| `--show-categories` | `--categories` | Add the measurement purpose/category table. Same naming rule as folder display. |
| `--show-bars[=KIND]` | `--bars[=KIND]` | Explicitly request charts for measurement tables. `KIND` is `purpose` or `comments`, defaulting to `purpose` when omitted; only `=` supplies the optional value. |
| `--table-purpose PURPOSES` | `--purpose PURPOSES` | Filter source contributions in tables only. `PURPOSES` is a nonempty comma-separated selection of `production`, `test`, `other`; preserve existing set behavior and derived ratios. |
| `--markdown-depth N` | `--depth N` | Folder-breakdown depth in Markdown; retain the existing nonnegative integer domain and default 2. Does not limit scanning or other Markdown evidence. |
| `--workspace-file FILE` | `workspace --file FILE`; `explain --workspace FILE` | Select a workspace definition in either command, retaining each command's existing conflicts and path interpretation. |

Existing spellings remain supported aliases with the same parsing, values, conflicts and applicability as their canonical counterparts. No alias removal or deprecation warning is proposed. Advertise canonical spellings in routine help and examples; maintain one concise compatibility mapping in reference documentation rather than duplicate option entries. Mixed old/new spellings count as the same option for duplicate and conflict handling.

`--show-*` means an optional table addition in this interface, not applicability to every output format. It never selects a format implicitly. `--show-bars` preserves automatic terminal behavior, width-dependent placement, saved-table forcing, the existing purpose-dependent effective chart kind and the conflict with `--no-bars`. Keep `--no-bars`: it clearly suppresses charts, whereas adding `--hide-bars` or `--no-show-bars` adds vocabulary without resolving another ambiguity.

### Markdown Depth Boundary

Depth applies only to the Markdown Folder breakdown. Without a subfolder restriction, immediate project folders are level 1. With `--subfolder`, depth is relative to that selection: its immediate child folders are level 1, while labels remain project-relative; direct source files in the selected folder may have a separate row when depth is positive. In a workspace, member names occupy level 1 and their immediate folders level 2. Depth 0 leaves the Folder breakdown without data rows; it does not remove the section or suppress deeper paths in review, module or other evidence.

These are existing rendering semantics, not a new report-detail limiter. [PRODUCT's output-routing contract](../PRODUCT.md#output-routing) owns the guarantee and its [subfolder contract](../PRODUCT.md#subfolder-measurement) owns the relative origin; the implementation change must clarify the remaining depth boundaries there and link to that owner from the [report guide](../guides/reports.md#choose-content-and-destination). JSON keeps its existing complete folder evidence.

### Applicability Across Views

Resolve output formats using the existing defaults and explicit `--format`/`--print` values. A view option takes effect when at least one requested view supports it. It affects every supporting view and leaves other views unchanged. Applicability and command validity are distinct: the table below describes accepted measurement options, not options unavailable on a command.

| Options | Supporting view | Explicit request without a supporting view |
| --- | --- | --- |
| `--show-folders` | Single-root measurement table. | Warn; preserve the measurement outcome and output. |
| `--folder-rows N` | Single-root measurement table. | Reject with exit 1. |
| `--show-categories`, `--verbose`, `--show-bars[=KIND]`, `--no-bars`, `--table-purpose PURPOSES` | Root or workspace measurement table. | Warn; preserve the measurement outcome and output. |
| `--markdown-depth N` | Root or workspace measurement Markdown. | Warn; preserve the measurement outcome and output. |
| `--sort code\|name` | Root or workspace measurement table or Markdown. Preserve hierarchical Markdown folders and existing review ordering. | Preserve silent acceptance. |
| `--files` | At least one full measurement JSON view, whether saved or printed. Headline does not qualify. | Reject with exit 1. |

Only the last row extends current behavior: full JSON requested through `--print json` qualifies even when the saved view is table, Markdown or headline. Full JSON includes requested per-file records; a simultaneous headline still omits them. Existing success/warning behavior applies equally to aliases and canonical spellings, including explicit default-valued sort/depth options. Omitted defaults do not warn or fail.

Applicability does not require a visible difference on this dataset. Sorting an empty table, suppressing already-absent bars and selecting all purposes remain valid. Global `--no-color` remains accepted everywhere, including queries and machine output; it is a cross-command styling policy, not a required report view.

### Validation and Diagnostic Boundaries

| Validation category | Contract |
| --- | --- |
| Unsupported command options, parser conflicts and standalone queries | Preserve existing rejection and help/version early exits. Query restrictions still distinguish explicit default-valued options from implicit defaults. No resolved-format diagnostic is required for an option the command does not accept. |
| Invalid destination combinations | Preserve constraints such as `--print` requiring a real `--output FILE`, not absent output or `--output -`. Explain the destination constraint; do not imply that choosing a different format resolves it. |
| Applicability of accepted measurement options | Preserve the outcomes above. A warning or error names the option and supporting format and gives a valid format remedy. Resolved formats may supply context; a parser-wide format-resolution guarantee is not required. Silent `--sort` acceptance remains silent. |

For example, a warning for `--show-folders` with inferred headline output can suggest `--format table`; with saved JSON it can suggest `--print table`. Never suggest `--print` without a real file destination. A comparison invocation with `--show-folders` remains an unsupported-option error, not a request to change comparison format.

Rejected destination/applicability combinations fail before scanning or writing, with empty stdout and no creation or modification of Locmetry-managed output files. Acceptance must verify both noncreation and preservation of an existing file's sentinel contents, plus an independent observation that scanning was not entered. These guarantees exclude shell redirection and `tee`, which can truncate before Locmetry runs, as explained by the [output contract](../PRODUCT.md#output-routing).

### Complete Surface Disposition

The following inventory covers existing options not fully specified by the rename table, plus positional and query surfaces. Names remain unchanged unless the rename table says otherwise.

| Surface | Disposition |
| --- | --- |
| Root `[DIR]`, `--root DIR`; explain `PATH`, `--root DIR` | Keep project-root and file-selection semantics, root-relative path rules and reserved-command disambiguation. Standardize root's displayed metavar to `DIR`. |
| Root `--subfolder DIR` | Keep; help must lead with measurement restriction and retained project policy/module context. |
| Root/explain `--config FILE` | Keep; standardize the displayed metavar to `FILE`. Explicit configuration does not change the root. |
| Root/explain `--exclude GLOB` | Keep repeatability and root-relative exclusion semantics. |
| Root/explain `--hidden BOOL`, `--respect-ignore-files BOOL` | Keep explicit `true\|false` overrides of configuration. Renaming to a bare inclusion switch would obscure the ability to override in both directions. |
| Root/workspace/explain `--no-user-config` | Keep; group with policy and selection, not presentation. |
| Workspace `--root DIR` | Keep repeatable member roots and mutual exclusion with the workspace-file selector. |
| Compare `BEFORE AFTER`, `--member NAME` | Keep; member selection affects comparison data across formats while provenance/findings/partial status retain their documented wider scope. It is not a table filter. |
| Root/workspace/compare `--format FORMAT` | Keep `table\|json\|headline\|markdown`, current precedence and destination defaults. |
| Explain `--format FORMAT` | Keep `table\|json\|markdown`, table on terminals and full JSON otherwise. |
| Root/workspace/compare `--output FILE` | Keep silent file exports, `-` for stdout and existing destination protection. |
| Root/workspace `--print FORMAT` | Keep the second view from one scan; require an actual saved-file destination, so `--output - --print ...` remains invalid. |
| Root/workspace `--files` | Keep the name under JSON content; apply the across-views correction above. It includes records rather than selecting measured files. |
| Root `--folder-rows N` | Keep required positive integer, implicit folder display and adaptive soft-budget semantics. “Rows” and help grouping identify presentation without a longer spelling. |
| Root/workspace `--verbose`, `--no-bars`, `--sort code\|name` | Keep; these established or descriptive names do not need `--show-` mechanically added. |
| Global `--no-color` | Keep before/after-subcommand support and disabling color, bold and dim. Does not disable bars or Markdown emphasis. |
| Root `--schema TOPIC` | Keep `config\|user-config\|report\|comparison\|explanation\|workspace`, standalone validation and stdout JSON. |
| Root `--guide TOPIC` | Keep `configure\|selection\|reports\|compare`, standalone validation and optional workflow/reference role. |
| `-h`, `--help`, `help [COMMAND]`; root `-V`, `--version` | Keep current early exits and short/long help distinction. |

### Help and Documentation

Use bounded semantic grouping to distinguish selection/policy, output/content and presentation. Keep table-only and Markdown-only applicability adjacent to their controls, use only applicable groups in other commands, and keep positional inputs and command discovery visible. An exact number or wording of headings is not prescribed: assess the rendered help at actual supported widths and avoid tiny groups that add reading overhead. Display each option once. Use typed metavariables (`DIR`, `FILE`, `GLOB`, `BOOL`, `N`, `KIND`, `PURPOSES`, `FORMAT`, `TOPIC`, `NAME`) with values and format limits adjacent to the option.

Keep short descriptions concise and put detailed bar geometry, counting semantics and policy precedence in existing owners. Bar help must disclose that selecting exactly one purpose substitutes comment-density bars, even with explicit `=purpose`; a dataset containing only one purpose does not itself trigger that substitution. Preserve actionable long-help examples and conditional routing to guides/schemas. Replace ambiguous “count production only” wording with “show production source in the table”, add explicit `--format table` to that recipe, and state that JSON remains the complete selected measurement.

Reconcile PRODUCT's Output Routing and Adaptive Folder Detail around applicability across views while retaining the documented warning/error distinctions; update affected README entries, embedded guides, diagnostics and current examples. Correct root help's claim that `--folder-rows` requires `--format table`: a secondary `--print table` also qualifies. Correct stale bar descriptions to comment density, and categories wording to saved or printed tables. Archived proposals, historical study treatments and captured demonstration output are not editable copies of current help. Recorded demonstrations must be regenerated under the existing [Showboat rules](../showboat-authoring.md).

## Agent Efficiency and Minimal Examples

Agents benefit from predictable formats, a compact initial result, one scan for multiple views, explicit scope and short errors that provide a usable next command. Preserve these strengths. The desired improvement is fewer wrong invocations and reference lookups, not simply fewer characters per flag or fewer bytes of help. New spellings cost some input tokens; grouped descriptions and avoiding synonym lists should constrain the reading cost. These are hypotheses, not measured savings.

These proposed commands assume a `service` project with `src/`; saved destinations are outside it.

```sh
locmetry --root service --subfolder src --format table --show-folders
locmetry --root service --output baseline.json --print table --show-folders --table-purpose production
locmetry --root service --output overview.md --format markdown --print json --files
locmetry workspace --workspace-file repos.toml --format table --show-categories --show-bars=comments
```

The first changes measured scope and requests folder display independently. The second saves the full selected measurement and prints only production source in its table. The third saves Markdown while emitting full per-file JSON from the same scan. The fourth uses the workspace definition's own member policies and adds table details.

Immediate acceptance is behavioral and documentary:

- Canonical names and aliases agree on values, repeatability, mixed-spelling duplicate handling, conflicts, supported commands and emitted results; optional bar values never consume a following positional argument. Queries still reject explicit default-valued options and accept omitted defaults.
- Every applicability row preserves its specified outcome. Cover primary and secondary supporting views, both-table views, full JSON in either position and JSON/headline projection separation, with root/workspace parity where the option is supported. Only `--files` gains formerly rejected valid combinations.
- Rejected destination/applicability combinations satisfy the noncreation, existing-file preservation and independent pre-scan guards above. Parser rejection is assessed separately; no format remedy is expected for an unsupported command option.
- Purpose-filtered table totals differ appropriately while full JSON and Markdown retain complete selected-measurement totals. Guard bare and explicit `=purpose` chart substitution for a one-purpose selection, and distinguish that selection from a dataset containing one purpose. Compare member selection retains its different, documented scope.
- Preserve successful JSON exports with `--no-bars` and successful piped headlines with explicit `--sort code`. Under a pipe, `--show-folders` without table format still warns and emits the inferred headline, while explicit `--format table --show-folders` succeeds. Valid empty-data requests remain valid.
- Depth guards cover zero, a selected subfolder's relative origin and project-relative labels, workspace member levels, and deeper review paths surviving a shallow Folder breakdown.
- Execute displayed recipes for scope plus folder display, saved JSON plus a production-only table, saved Markdown plus full per-file JSON, and workspace presentation. Independent oracles establish selected counts and projection boundaries. Review short/long help at supported widths for option grouping, adjacent applicability, accurate descriptions, executable commands and intact conditional reference routing. Existing comparison help remains a guard against unrelated factual or routing regressions.

Reduced calls, reading cost and reference retrieval remain unmeasured hypotheses, not acceptance gates for this change. Use [help-design evaluation guidance](../help-design.md#evaluating-future-additions) before making an efficiency claim: define fresh-session tasks including the covered workflows and an unrelated comparison task, then freeze correctness oracles and cost gates. No benchmark campaign is authorized or required for these factual corrections by this draft. Historical studies support retaining conditional reference routing, but do not establish performance of the proposed names or grouping.

## Deferred Compatibility Decision

Blanket rejection of ineffective options is excluded from this proposal. The earlier draft recommended it; external review identified successful reusable-option workflows that it would break, including `--no-bars --output baseline.json` and explicit `--sort code` under a pipe. A misleading purpose-filter assumption and a redundant chart-suppression preference do not necessarily warrant the same failure policy.

Recommendation: retain existing outcomes now. Reopen the decision only with a separate compatibility proposal that evaluates reusable option sets, terminal-to-pipe transitions, filtered-table versus complete-JSON interpretation and existing script expectations. Any future decision must justify its behavioral break independently of naming clarity. Different semantics for aliases and a speculative strictness switch are not proposed workarounds.

## Review Disposition

The external review of `c7c814b` concluded “revise before implementation.” This revision accepts its scope reduction: preserve successful-command behavior, narrow diagnostic guarantees, define depth boundaries, make immediate acceptance concrete and correct the mistaken attribution of the `--files` help defect to its diagnostic. The review reported eight further read-only probes of the existing binary; they are reviewer evidence, not fresh-build or usability validation. The proposal remains a draft pending owner review.

## Optional and Excluded Scope

Optional, not recommended for this change: rename `--files` to `--include-files`. It would make addition explicit, but JSON-content grouping and an accurate description already disambiguate it; the stronger defects above justify their naming cost more directly. Revisit only if review or task evidence shows recurring file-selection confusion.

Excluded: blanket rejection of ineffective options, parser-wide resolved-format diagnostics, a global Markdown-detail limiter, new measurement filters, changing purpose classification or count formulas, automatically choosing table format when a display flag appears, a universal `--show` list, new short aliases, machine-readable CLI discovery schemas, command reorganization, additional compare/explain capabilities, render-from-saved-report support, arbitrary flag placement around subcommands, theme configuration and benchmark execution.

The existing policy of command options following the command name remains explicit; only global styling crosses that boundary. A broader parser redesign is not necessary to resolve this proposal's scope, naming and applicability problems.
