# Proposal: a useful first run without a configuration audit

<!-- doc-status
{
  "status": "implemented",
  "date": "2026-10-17",
  "summary": "Quiet default measurement tables with opt-in findings and visible real failures.",
  "evidence": "[Supported contract](../../PRODUCT.md#report-content), [acceptance guards](../../../tests/integration/verbosity.rs)"
}
-->

Status: historical decision record — implemented and verified on 2026-10-17. The owner approved the revised proposal on that date and excluded the subsequent Tokei discussion; the historical parity paragraph below creates no scheduled follow-up. [PRODUCT](../../PRODUCT.md#report-content) owns the supported contract; [acceptance guards](../../../tests/integration/verbosity.rs) cover the default/detail boundary, failures, output formats and saved tables. Imperative wording, examples and limitations below describe the reviewed design, not current guarantees. Implementation does not imply publication.

Total Lines and UTC measurement dates: implemented in `846283b`; not part of this decision.

Redacted review copy: measured-project paths, counts and raw output are omitted. Synthetic examples below are illustrative and do not reproduce private measurements. Repository-relative source links require this repository context. No copy has been sent externally.

## Problem and owner's requirements

The owner expects a simple, useful out-of-the-box experience: run locmetry against a project and receive tables, as with Tokei. A representative command is `tokei -t=Java,Kotlin,TypeScript,Lean,JavaScript,Shell,Python`; the subsequently supplied comparison used the same list without Python. The initial command measured a12-rulekit by directory argument. Its appended commentary, reproduced in substance below, was disturbing because a successful counting run appeared to demand configuration work.

The concern is not merely output length. Ordinary data/documentation, inferred build ownership, source without an inferred purpose and unknown file types appear together as if they are problems to resolve. The instruction to classify source makes optional refinement feel like a prerequisite for using the tool. The default should make counts immediately useful without requiring a configuration project first.

The owner also questioned apparent differences from Tokei, specifically Kotlin. Kotlin actually matches exactly; Tokei places total Lines before Code, while locmetry omitted Lines. This misleading layout undermined confidence in the measurement. The owner considers total lines fundamental to a LOC tool and requested restoring them throughout reports wherever missing, not only in the terminal language table. The owner additionally requested a date at the end of reports, at least terminal tables and preferably Markdown and JSON too, because filesystem creation time is not dependable report provenance.

The owner requested a Markdown proposal preserving the input, options, alternatives, recommendations and rationales for external review.

## Evidence supplied by the owner — redacted

The initial report appended non-source totals, build-module inference, a multi-folder unclaimed-source list with a classification instruction, and unrecognized-file counts grouped by extension. These represent normal measurement and classification facts, not necessarily failures. Its tables were omitted from that excerpt.

A later Tokei/locmetry comparison raised concern about Kotlin. All shown Kotlin count categories and its purpose arithmetic reconcile; the apparent mismatch came from locmetry omitting the total Lines column. Java and TypeScript also match in the excerpts. Shell differs in file and line counts; JavaScript differs in code lines despite equal file counts. Locmetry includes languages outside the supplied Tokei filter, so the grand totals are not directly comparable. The two locmetry excerpts are separate observations, not a synchronized snapshot. Per-file causes remain unverified.

The supplied locmetry language table predates `846283b`; it must not be read as current output. Exact historical counts and folder labels remain solely in the full local evidence version.

## Current mechanism

The [terminal renderer](../../../src/output.rs) always appends informational findings to its count tables. It presents non-source totals, inferred build ownership, Git submodules, unclaimed folders, unused configuration and coverage gaps in the same closing area. There is no explicit request for a configuration review.

Unclaimed source has already been counted and appears under Unclassified. Its purpose was not established by the classification rules. Unrecognized files can be ordinary images or other unsupported types; they are distinct from recognized files that could not be read or parsed. Neither unrecognized files nor unresolved classification alone makes a measurement partial. Read/parse failures and backend inaccuracy do, with partial output and exit 2. Invalid configuration is an error. The [product contract](../../PRODUCT.md#command-outcomes) owns these distinctions.

The coverage summary currently combines unrecognized, unreadable and parse-failed files on one warning-styled `Not counted:` line; it also emits up to 25 parse-failure paths separately. `repositories_note` similarly combines unclaimed source and unavailable counts per member under “Repositories to review:”. Filtering whole lines cannot implement the intended boundary. The [CLI report handling](../../../src/main.rs) independently retains an inline backend-incomplete warning and a stderr diagnostic naming up to 10 unavailable paths, with a remaining-path count or a no-path-evidence fallback.

## Recommended public contract

For a successful terminal measurement, show the existing count tables and their reconciliation legends, with explicit total Lines and the measurement-date footer. Source purpose columns remain; Unclassified is a normal category, not an invitation to mandatory setup. No configuration is required for a useful first report.

The following content contract applies to single-root, subfolder and workspace measurements, including explicitly saved table text. It classifies facts within existing mixed notes, not entire current output lines.

| Finding | Default table | Table with `--verbose` | Rationale |
| --- | --- | --- | --- |
| Data/documentation totals, inferred build ownership, Git submodule hints, unused entries/exclusions | Omitted | Included | Useful inspection evidence, not counting failures |
| Unclaimed source, including per-member unclaimed counts | Only the existing Unclassified table columns | Folder detail and separate unclaimed-by-repository notes | Source was counted; configuration is optional refinement |
| Unrecognized files and extension groups | Omitted | Separate neutral `Unrecognized files:` note | Unsupported types are distinct from failed counts; do not mix them into a yellow failure summary |
| Unreadable and parse-failed file counts | `Not counted:` summary containing only these failures | Same failure summary | Counts are unavailable, so ordinary output must disclose partial results |
| Workspace members with unreadable or parse-failed files | `Unavailable counts:` note naming each affected member and its failure counts | Same member-failure note, separate from unclaimed-source notes | The global failure summary alone does not identify affected members |
| Individual parse-failure paths in the report | Omitted | Existing bounded list, up to 25 paths | The default stderr diagnostic already supplies bounded path evidence |
| Backend-incomplete warning, including cases without known failure paths | Retained | Retained | Empty failure-path lists do not establish complete counts |

For example, a synthetic workspace with an unrecognized image, unclaimed source, one unreadable file in member `api`, and one parse failure in member `web` shows `Not counted: 1 unreadable file, 1 file with a parsing failure; missing lines are not zero.` and `Unavailable counts: api: 1 unreadable file; web: 1 file with a parsing failure.` by default. The image and unclaimed findings appear only with `--verbose`; they are not added to either failure line. The existing backend warning, stderr diagnostic and exit 2 remain in both modes.

Preserve the current stderr diagnostic, including its bounded unavailable-path list, and the inline backend-incomplete warning. Invalid configuration and output-write errors remain errors. Warnings for explicitly supplied ineffective options remain visible because they concern the invocation. Successful measurement does not acquire a new warning or exit failure merely because classification is unresolved or files are unrecognized.

The default table intentionally provides no data/documentation totals and no indication of unrecognized files. Its totals describe recognized, counted source only. That matches the owner's language-filtered Tokei baseline; it does not promise the same scope as unrestricted Tokei, which may include data and documentation languages. Omission from this view does not mean those files were excluded by selection rules, nor does it mean their lines are zero. Complete findings remain available in the detail view, Markdown and JSON according to their existing contracts.

Markdown remains the comprehensive human report. JSON headline and full JSON retain their existing evidence and omission rules. Comparison and path-explanation commands are already explicit inspection workflows; their findings remain visible. The neutral optional-next-step wording described below replaces the current imperative wherever that advice is presented, including documentation examples; it does not suppress Markdown detail.

Do not add a flag reminder to successful measurement output. Instead, replace the setup-first opening of root `after_help` with: “Run locmetry for source-count tables. Add --verbose to inspect classification, scope and coverage notes. For optional project configuration, use locmetry --guide configure.” Keep the reserved-command-name guidance. Describe the same inspection option in workspace help and the reports guide so discovery does not depend on a noisy first run.

## Options and alternatives

| Option | Advantages | Costs | Recommendation |
| --- | --- | --- | --- |
| Tables by default; `--verbose` restores the existing informational notes | Familiar CLI convention; small public interface; preserves a direct route to current notes | “Verbose” can suggest internal logs, so help must explicitly say it adds report findings; discovery moves to help/docs | Preferred: the smallest complete way to make a normal run simple while preserving inspection |
| Tables by default; `--details` or `--notes` restores notes | Names report content more directly | Adds a less conventional switch; `--details` may be mistaken for per-file output or folder expansion | Reasonable alternative if reviewers find `--verbose` misleading; prefer `--notes` over `--details` for precision |
| A dedicated `audit` command | Makes configuration review an explicit task with its own name | Implies a broader audit contract, duplicates measurement entry points and raises questions about audit success criteria | Exclude from this change; the request does not justify a new command workflow |
| Preserve noisy defaults and add `--quiet` | Minimal disruption to existing output consumers | Requires users to learn a workaround before getting the expected first-run experience | Reject: it leaves the reported problem as the default |
| Keep one short default findings summary and put details behind a flag | Preserves discovery and some visibility | Still frames normal unclassified/unsupported content as unfinished work; is still commentary after every ordinary run | Reject as the initial design; the owner's baseline is tables, not an abbreviated audit |
| Hide notes whenever no explicit project configuration exists | May distinguish casual users from policy authors automatically | Output changes based on ambient file discovery; configured users can still want simple counts; personal defaults complicate the rule | Reject: use explicit user intent rather than an inferred review mode |
| Remove informational findings from every format | Very simple terminal experience | Loses useful inspection evidence and disrupts automation/review workflows | Reject: presentation noise does not justify discarding evidence |

Recommendation on scope: preserve Markdown and JSON detail. The reported friction is the ordinary terminal run, while these formats already serve documentation and machine inspection. Applying quietness to saved Markdown as well would require an additional explicit owner decision.

Recommendation on warning language: in the optional detail view, describe unclaimed source neutrally as counted source without an inferred purpose. In the detail view, replace “Classify them” with “To inspect or refine purpose classification, use locmetry --guide configure; --files --output - shows each file’s rule.” Replace existing occurrences and affected examples rather than retaining the imperative alongside the new wording. Separate unrecognized types from genuine unavailable counts. Do not suggest that unknown files have zero lines, and do not declare all unknown files harmless; the distinction is evidence, not an assumption about extensions.

The choice between `--verbose` and `--notes` is balanced. Recommend `--verbose` because users can guess a familiar flag when ordinary output does not advertise it. A demonstrated need for diagnostic logging would tip the decision toward reserving that name and using `--notes`; no such requirement is established here. Use `--verbose` with help text such as “Include classification, scope and coverage notes in table output.” It should be boolean, not an unspecified verbosity ladder. Outside table measurement output, follow the existing ineffective-presentation-option convention rather than silently changing JSON or Markdown. A short alias is optional and unnecessary for the minimum contract.

## Required, optional and excluded

Required scope:

- Tables without routine audit commentary, deliberately omitting any indication of data/documentation totals and unrecognized files from the default table view.
- Explicit access to retained informational notes; neutral optional classification advice in the detail view and wherever that advice is documented.
- Failure-only `Not counted:` summaries, failure-only workspace member notes, preserved backend/stderr warnings and unchanged exit semantics.
- Consistent root/subfolder/workspace behavior across stdout and saved table text; comprehensive Markdown and unchanged JSON evidence.
- Help-first discovery and the affected documentation and regression scope listed below.

Optional: a short alias for the inspection switch. Recommendation: omit it initially because the long name is sufficient and no frequent-use need has been demonstrated.

Excluded: changing counting, ignore rules, language recognition, classification precedence or purpose ratios; automatic project configuration; requiring configuration before measurement; claiming the unexplained Shell/JavaScript differences are resolved; new audit scoring or CI failure criteria; quieting explicit comparison/explanation workflows; changing JSON schema or omitting evidence to reduce terminal noise.

## Affected documentation and regression scope

These references identify the scope to reconcile if the proposal is approved; source line numbers from review are not durable anchors.

| Owner or guard | Required reconciliation |
| --- | --- |
| [CLI help](../../../src/main.rs) | Root `after_help` becomes count-first and mentions `--verbose`; workspace help describes the same option |
| [README](../../../README.md) | Default-output example, Git submodule/selection advice and Honest coverage example distinguish default failures from optional notes; affected examples use neutral classification advice |
| [Reports guide](../../guides/reports.md) | Explain how to request notes before sections organised around “Unclaimed source” and “Not source code”; retain the inspection recipes |
| [PRODUCT workspace contract](../../PRODUCT.md#per-repository-breakdown) | Replace the unconditional mixed member-note guarantee with the default failure/detail boundary; preserve comprehensive Markdown |
| [PRODUCT reporting](../../PRODUCT.md#report-content) and [adaptive folders](../../PRODUCT.md#adaptive-folder-detail) | Document the default and verbose content contract; the unclaimed-note partition remains independent of `--folder-rows` when requested |
| [CLI regressions](../../../tests/integration/cli.rs) and [workspace regressions](../../../tests/integration/workspace.rs) | Preserve detail assertions with explicit `--verbose`; add default, mixed-finding and per-member failure cases |
| [Adaptive folder regressions](../../../tests/integration/folder_rows.rs) | Preserve the note-partition guards with `--verbose`; also ensure adaptive tables stay quiet by default |
| [Terminal safety regressions](../../../tests/integration/terminal_safety.rs) | Preserve sanitization guards with `--verbose` for optional notes and default coverage for visible failures; do not delete existing guards |
| [Query guards](../../../tests/integration/queries.rs), [output routing guards](../../../tests/integration/output_contract.rs), [guide guards](../../../tests/integration/guides.rs) and [executable examples](../../../scripts/check-doc-examples.sh) | Cover standalone-query rejection of the presentation flag, its ineffective-format warning, saved-table behavior and changed embedded help/recipes |

## Review disposition and parity follow-up

The external review's five substantive findings are incorporated as recommendations: separate mixed coverage facts, split mixed workspace notes, explicitly accept hidden non-source/unknown coverage in the default view, name affected documentation and tests, and make help count-first. The recommended decisions remain the successful-run contract, a boolean `--verbose`, comprehensive Markdown/JSON and replacement of imperative classification advice in the detail view. These are design recommendations, not a claim that the owner has approved them.

The review independently reported that the supplied totals reconcile, Kotlin matches, the noted Shell/JavaScript differences are real in the excerpts, links resolve and the full evidence document is ignored. Its evidence check does not establish the cause of the remaining count differences.

For a separate parity investigation, hidden-directory selection and language mappings are plausible hypotheses for Shell. The supplied evidence includes shell-related files beneath a hidden directory, but it does not establish that these account for the observed gap. Compare the exact binaries, effective selection policies and per-file language assignments before attributing the difference; the reviewer's statement about Tokei defaults is a lead to verify for the installed version, not a conclusion of this proposal.

## Acceptance criteria for reviewers

- A plain successful measurement of an unconfigured source folder produces count tables, legends and a measurement date, with no instruction to classify files.
- A mixed project containing source, documentation, data and unsupported image files has the same simple default experience. No data/documentation total or unrecognized-file count appears by default; explicit detail and machine evidence retain those facts.
- An unrecognized file alone produces no finding line after the tables; the measurement-date footer remains.
- With both unrecognized files and real failures, default `Not counted:` mentions only unreadable/parse-failed counts. The report’s per-path parse-failure list appears only with `--verbose`; the default stderr path diagnostic and inline backend warning remain.
- With both unclaimed source and failures in a workspace, default `Unavailable counts:` names only affected members and their failures. Unclaimed-by-member details require `--verbose`, and both views retain exit 2 for partial measurements.
- Source without an inferred purpose remains counted and visibly Unclassified. No configuration is synthesized merely to clear a review list.
- An unreadable or invalid recognized file still produces a clear warning and partial-result exit 2. Invalid configuration remains an error.
- The inspection view exposes the existing useful facts, including unused policy and Git submodule hints, without conflating them with counting failures. Its optional next step replaces the imperative “Classify them”; affected examples use the same new wording.
- Single-root, subfolder and workspace tables follow one contract; saved table text follows the same content selection as terminal text.
- Markdown/JSON and explicit comparison/explanation findings retain their established detail unless separately authorized.
- Root help leads with measurement, explicitly names `--verbose`, and presents configuration as optional. Existing adaptive-folder and sanitization guards remain, with the flag added where needed.
- Existing Lines columns, purpose reconciliation and measurement-date footers are preserved.

## Review decision requested

Review the recommended successful-run contract, the `--verbose` interface, and the boundary between routine findings and real failures. Recommendation: accept the contract and the single boolean switch, preserve comprehensive Markdown/JSON, and leave counting/parity investigation separate. This addresses the owner's first-run problem without expanding it into a new audit product. Any implementation of the default-output change still requires explicit owner authorization after review.
