# Deferred Work

Items here are deferred or not scheduled; each names the condition or decision that would reopen it. They do not authorize implementation: [PLAN](PLAN.md) holds open work, and an item moves there only by the owner's decision. [PRODUCT](PRODUCT.md) owns supported requirements.

## Windows

- Generate PowerShell tab completion, together with the other shells; deferred by the owner on 2026-10-21.
- Accept UTF-16 configuration and workspace files, and add PowerShell equivalents of the `jq` recipes, only when Windows users ask for them. Windows on ARM stays excluded while x86_64 emulation serves it. The [Windows support record](proposals/archive/windows-support.md) owns the rationale.

## Built-in Detection

- Deferred framework conventions: Storybook stories and Jest manual mocks, deferred by the owner on 2026-10-23 because their purpose needs a decision and other ecosystems were not searched; and `requirements.txt` as a Python descriptor, which needs its own evidence because documentation and tool folders carry such files too. The [archived proposal](proposals/archive/framework-layouts.md) records the measurements.
- Deferred generated-code markers, each measured precise in at most one application: Django's migration header, which also stays on migrations with hand-written data code; OpenAPI Generator's `.openapi-generator/FILES` manifest, whose format is documented only in a log excerpt; `@generated` and `__generated__/`; and tool-specific headers such as jOOQ's, swag's and the protocol buffer compiler's. Add one only with measurements of its own on applications and on its generator's repository; Kotlin's `@Generated` is unmeasured. The [archived proposal](proposals/archive/generated-code-markers.md) and the [measurement record](research/generated-code-markers.md) keep the evidence.
- `ignore` directives in `go.mod`, which `go help packages` also excludes from `./...`: the Go layout does not read them, because detection reads file names, not descriptor contents. Reopen when a measured project relies on one; the [Go layout record](research/go-layout.md) quotes the rule.
- Candidates from the [policy test drives](research/policy-test-drives/README.md#findings), each seen in one to four projects and without a decision: Gradle `buildSrc` and Maven failsafe test conventions, reasons on scope exclusions, an `except` list for entries whose carved-out files fall through to a later stage, and rules that apply only to files no other stage decides. A built-in candidate needs the [research checks](../CLAUDE.md#research-and-experiments) before it is proposed.
- Built-in language corrections for project-independent names, such as `proguard-rules.pro`, in a set selected by `version`: add one only when the name recurs across measured projects; the first was observed once ([language corrections](proposals/archive/language-corrections.md)).

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
