# Deferred Work

Items here are deferred or not scheduled; each names the condition or decision that would reopen it. They do not authorize implementation: [PLAN](PLAN.md) holds open work, and an item moves there only by the owner's decision. [PRODUCT](PRODUCT.md) owns supported requirements.

## Windows

- Generate PowerShell tab completion, together with the other shells; deferred by the owner on 2026-10-21.
- Accept UTF-16 configuration and workspace files, and add PowerShell equivalents of the `jq` recipes, only when Windows users ask for them. Windows on ARM stays excluded while x86_64 emulation serves it. The [Windows support record](proposals/archive/windows-support.md) owns the rationale.

## Built-in Detection

- Deferred framework conventions: Storybook stories and Jest manual mocks, deferred by the owner on 2026-10-23 because their purpose needs a decision and other ecosystems were not searched; and `requirements.txt` as a Python descriptor, which needs its own evidence because documentation and tool folders carry such files too. The [archived proposal](proposals/archive/framework-layouts.md) records the measurements.
- Deferred generated-code markers, each measured precise in at most one application: Django's migration header, which also stays on migrations with hand-written data code; OpenAPI Generator's `.openapi-generator/FILES` manifest, whose format is documented only in a log excerpt; `@generated` and `__generated__/`; and tool-specific headers such as jOOQ's, swag's and the protocol buffer compiler's. Add one only with measurements of its own on applications and on its generator's repository; Kotlin's `@Generated` is unmeasured. The [archived proposal](proposals/archive/generated-code-markers.md) and the [measurement record](research/generated-code-markers.md) keep the evidence.
- `ignore` directives in `go.mod`, which `go help packages` also excludes from `./...`: the Go layout does not read them, because detection reads file names, not descriptor contents. Reopen when a measured project relies on one; the [Go layout record](research/go-layout.md) quotes the rule.
- Candidates from the [policy test drives](research/policy-test-drives/README.md#findings), each seen in one to four projects and without a decision: Gradle `buildSrc` and Maven failsafe test conventions, reasons on scope exclusions, and rules that apply only to files no other stage decides. The `except` list shipped with the [policy entry review](proposals/archive/policy-entry-review.md). A built-in candidate needs the [research checks](../CLAUDE.md#research-and-experiments) before it is proposed.
- Reading Lake files to tell test drivers from library modules: the Lake layout claims every `.lean` file, and Lean projects whose tests are plain modules set `builtin_modules = false`. Detection reads file names, not descriptor contents, as for the `go.mod` item above; reopen only with a decision to read descriptors.
- Built-in language corrections for project-independent names, such as `proguard-rules.pro`, in a set selected by `version`: add one only when the name recurs across measured projects; the first was observed once ([language corrections](proposals/archive/language-corrections.md)).

## Configuration and Categories

Requests from three projects categorizing their tests on 2026-10-24; the [policy entry review](proposals/archive/policy-entry-review.md) implemented the rest.

- Category groups with subtotals, such as a `law` group over several law categories, asked for by two projects. `--table-category 'test:*law*'` already shows the selection's subtotal as the language table's Total. Reopen when a group subtotal is needed in JSON or several groups in one table.
- Categories of data files in tables. Full JSON `breakdown` has them, while `--show-categories` covers source. Reopen when fixture growth becomes a recurring review question, together with a decision on the source-only table contract.
- Inline test hints in other languages, such as Lean `example` and `#guard` declarations in production files, reported like Rust's `inline_tests` without changing purposes. Reopen when a second Lean project asks.
- Content predicates on refinements, such as "mostly `theorem` declarations", which only change a category. Reopen after refinements ship, when a project's categories cannot be expressed by paths.
- Orthogonal tags beside the category, which must not break the guarantee that categories add up to purpose totals. Reopen when a second project needs a second axis.
- Test/prod between two chosen categories rather than purposes. Reopen with a second request.
- `decision.also_matched` in `explain`, listing the entries that match a file without deciding it. `summary.policy_entries` reports such files per entry as `preempted`, and `--files` shows which entry decided each one. Reopen when reviewers need the per-file view to untangle entry order.
- Accepting reference category names without a declaration in `[categories]`, with reports recording their reference meaning. It would change validation and the effective-policy snapshot, as the [category reference](proposals/archive/category-reference.md#excluded) explains. Reopen when the vocabulary of `--guide categories` has held unchanged for a release and a contract defines which meaning wins when a project declares a reference name differently.
- Hints in reports when a policy uses a name the category reference replaces, such as `benchmark` for `performance`. Reopen when the vocabulary has held unchanged for a release.
- A display name for a language correction, so a DSL whose comment syntax matches a counter language is counted with that parser but reported under its own name. A counter of its own stays out of scope while Tokei counts. Reopen when a second project needs one.

## Guides and Help

Excluded by the [guide sections proposal](proposals/archive/guide-sections.md#excluded) and its agent runs.

- Splitting the reports guide (19 KB, 9 sections) into more topics. Self-contained sections give small reads without more routing choices. Reopen if routed reads of its sections prove too coarse.
- Trimming the categories guide's "Write a Reviewed Policy". Its `kind = ["source"]` advice has no other home yet. Reopen when that advice has one.
- One command that reads guide sections and schema definitions together. Routes read them in separate commands, which cost the configuration tasks about one call per session in the [adoption decision](research/agent-regression-run-2026-10-24-sections-v2.md#decision). Reopen if calls, rather than uncached input or time, become the binding cost.

## Display

- Folder-partition convergence remains deferred: adaptive folder detail and JSON `unclassified_folders` intentionally use different policies. Revisit only if a workflow demonstrates confusing divergence, with explicit compatibility review. The [adaptive-folder decision](proposals/archive/adaptive-folder-detail.md#existing-partition-and-feasibility) owns the rationale.
- JSON finding grammar, such as “1 unrecognized files”, remains an optional separate cleanup. The comparison-readability changes intentionally preserved JSON message text.

## Measurement and Comparison

- [Conditional measurement/comparison capabilities](plans/measurement-quality-and-net-change.md#conditional-follow-ups-and-preserved-rationales): consumer-driven candidates and their triggers, including [per-file comparison](plans/measurement-quality-and-net-change.md#per-file-comparison), generated/minified evidence and duplicate evidence. Ancestor discovery remains outside accepted scope.
- Revisit native file comparison only when recurring usage justifies expanding the CLI. The optional [file-transition helper](../scripts/compare-locmetry-files.sh) ships with releases and Homebrew, while its `jq` prerequisite remains a user-installed choice; avoiding that prerequisite alone does not activate native implementation.
- [Analytics cookbooks](plans/analytics-cookbooks.md): removed 2026-10-21; recreate them, refocused on questions native output cannot answer, only on the owner's decision. The design names the recovery commit and every removed file and pin.

## Command-Line Options

- [Ineffective-option compatibility policy](proposals/archive/cli-option-clarity.md#deferred-compatibility-decision): blanket rejection remains deferred. Reopen only with a separate compatibility assessment of reusable option sets, terminal-to-pipe transitions and existing script expectations.
- [File-detail option naming](proposals/archive/cli-option-clarity.md#optional-and-excluded-scope): retain `--files`; revisit `--include-files` only if task evidence shows recurring confusion with input selection.
