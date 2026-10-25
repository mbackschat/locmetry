# Create or Evaluate Project Configuration

Use this guide to create `.locmetryrc` or review an existing one. Start at the intended project root and inspect any existing configuration before proposing edits. Preserve policy unless repository evidence or the owner's intent justifies a change. Locmetry validates syntax and reports observed decisions; it cannot decide what the project ought to measure.

## Start with Evidence

The first measurement precedes edits. This command works with the installed binary alone; `.` means the intended project root:

<!-- discovery-commands -->
```sh
locmetry --root . --format headline
```

Read the compact headline first: effective scope, loaded configuration paths, coverage, `summary.submodules`, `summary.unclaimed_folders`, `summary.largest_module_files` and unused-policy findings. Module layouts claim production without review, so read the largest files counted as production only by a build layout, and exclude or reclassify generated or vendored ones. Generated code with a documented marker, such as Go's `// Code generated … DO NOT EDIT.` line or a `.gitattributes` `linguist-generated` declaration, is already `production:generated` with the marker as its `rule` and is not listed there; code from a generator without such a marker needs an entry. `.locmetryrc` is discovered only at the scan root, not in ancestors. `--config FILE` replaces project discovery; its patterns still use the scan root as their base. Personal defaults can affect selection and language recognition; inspect them rather than silently disabling them for a real project.

Inspect `summary.unused_entries` and `summary.unused_exclusions` when present; omission means no such findings in that report. `summary.policy_entries` lists every override, rule, refinement, language correction and language mapping with the `files` and `code` it decided. A workspace of independent repositories keeps these three lists per member in `metadata.members[]`, while a workspace of one project's folders reports its shared policy once in `summary`. Compare each count with the folder or file set it is meant to cover: a glob that misses, for example through a case mismatch, decides few files and raises no finding. `preempted` names an earlier entry or convention that took some matching files, `unused_patterns` and `unused_exceptions` name globs that did nothing, and `displaced` names the decisions the entry replaced, by `purpose:category`. A displaced decision with another purpose than the entry's moves code between production, test and other; `--verbose` notes such entries. Confirm that each move is intended.

To review what a configuration edit changes, save a full baseline with `--files` before editing; the [configuration-transition recipe](compare.md#inspect-configuration-transitions) provides capture and comparison commands. A headline alone cannot recover earlier per-file decisions.

Decide whether submodules, vendored dependencies and reference material belong in the intended measurement. Exclude out-of-scope trees through scope policy; keep maintained project code measured and classify it from evidence. A submodule hint does not establish ownership. Report uncertain scope to the owner instead of assuming that all external-looking code should be excluded.

Retrieve `locmetry --help` when command syntax is unclear, or the configuration schema when the intended edit needs keys, defaults, exact built-ins or accepted language names not shown here. These are optional references, not prerequisites for the first measurement. Read only the parts you need, in one command: the overview carries the precedence and matching semantics and names every other section, such as a definition:

```sh
locmetry --schema config --section overview --section Classification
```

`locmetry --guide index` names the section for other needs. Request `locmetry --guide reports` for evidence meanings and narrow report queries, or `locmetry --guide selection` for a missing or surprising file. Do not fetch every guide, schema or file record before you have a question.

## Decide Purpose, Not Just Coverage

Each file has one content kind, one purpose and one category. The purposes are fixed, `production`, `test` and `other`; categories carry the finer distinctions within each. Classification uses first match: overrides → built-in test/build conventions → rules → inferred build modules, refined by generated-code markers → fallback. Refinements then change only the category of decided files. List order matters within overrides, rules and refinements. A rule cannot beat a convention; an override can, and a refinement relabels a convention's files without beating it. Scope exclusions run before classification and do not reclassify files.

Review representative already-classified source as well as the unclaimed list. Module inference can mark fixtures or historical samples as production, so zero unclaimed code is not proof of correctness. Names suggest inspection, not a verdict. Use a targeted explanation before changing policy:

```sh
locmetry explain src/main.py --root . --format json
```

Substitute an existing root-relative path. Inspect `result.file.rule`, `result.decision.stage` and any entry reason or module evidence. Correct built-in/module decisions need no redundant entry. For source that does need a decision, there are three legitimate outcomes:

1. Classify it through a narrow evidence-based entry.
2. Count it deliberately as other through an entry with `purpose = "other"` and a `reason`; it then leaves the unclaimed review list.
3. Leave it unclaimed and report the open question to the owner. Do not manufacture a rule just to remove the finding.

Globs match root-relative file paths. `docs/**` covers descendants; `docs` does not mean the directory. `*.py` matches root files only; `**/*.py` also covers deeper paths. Patterns are case-sensitive. Use `[[rules]]` for decisions after conventions, and `[[overrides]]` only when the evidence requires winning before conventions. To keep particular files out of an entry, list them in its `except` array: they fall through to later entries and stages, so the exception is stated instead of depending on entry order. An optional `[categories]` table documents permitted custom categories and catches typos. The schema supplies the complete matching contract.

## Recognize a Language

A language mapping makes an otherwise unrecognized selected file countable; it does not assign purpose. For example, after inspecting an extensionless `bin/launch` and establishing that it contains JavaScript, use this minimal configuration. In an existing configuration, merge only the mapping entry and preserve its single top-level `version`:

<!-- recognition-mapping -->
```toml
version = 1

[[languages.mappings]]
id = "node-launcher"
match = ["bin/launch"]
language = "JavaScript"
```

If the existing `[languages]` table declares `mappings = []`, remove that assignment before adding `[[languages.mappings]]`, or replace the array value with the intended entries. TOML cannot define the same array both ways. Preserve other language settings and existing mappings, and keep mapping IDs unique.

Mappings apply only when backend recognition fails. A mapping that matches a file the counter already recognizes does nothing to it: `--verbose` names such mappings, `summary.policy_entries` shows those files as `preempted` by `builtin:recognition`, and a mapping that counts no file appears in `summary.unused_entries`; a correction, below, changes a recognized file's language. After adding one, remeasure and explain the affected file using the same root and policy:

```sh
locmetry --root . --files --output ../mapped.json --print table --verbose
locmetry explain bin/launch --root . --format json
```

Inspect `result.file.language_mapping`, `purpose`, `category` and `rule`. Without a matching classification entry, convention or module, this launcher defaults to `other` / `unclaimed`; counting it successfully does not settle its role. Add a narrow purpose rule only after establishing that role, then verify again. A single capture establishes “mapped but unclaimed”; “newly mapped” requires comparison with an earlier capture. The [configuration-transition recipe](compare.md#inspect-configuration-transitions) shows how to inspect both classification and recognition changes.

## Correct a Misrecognized Language

The counter recognizes languages by file name and extension, and some extensions belong to several formats: ProGuard rules named `proguard-rules.pro` are read as Prolog, Helm's Go templates (`.tpl`) as Pan, and C++ headers as C Header. A mapping cannot change a recognized file, and an exclusion would hide the file instead of answering the question. After inspecting the file, correct its language in the project configuration:

```toml
version = 1

[[languages.corrections]]
id = "helm-templates"
match = ["deploy/helm/**/*.tpl"]
language = "Go HTML"
reason = "Helm Go templates; Go HTML shares their {{/* */}} comments"

[[languages.corrections]]
id = "proguard-rules"
match = ["android/app/proguard-rules.pro"]
language = "unrecognized"
reason = "ProGuard keep rules; no counter language shares their syntax"
```

Choose a counter language only when its comment syntax matches the file's, because the counts follow that syntax; otherwise use `"unrecognized"`, which leaves the file selected but uncounted. Remeasure and explain a corrected file: `result.file.language_correction` and `result.correction` name the correction, and an unused correction appears in `summary.unused_entries`. Corrections are project knowledge, so personal defaults cannot set them.

## Refine Categories

Classification selects a whole entry, not a separate winner for purpose and category. For a helper at `__test__/helpers/setup.js`, this ordinary rule loses to the built-in test-directory convention, leaving both purpose and category as `test`:

<!-- category-rule -->
```toml
version = 1

[[rules]]
id = "test-helpers"
match = ["__test__/helpers/**"]
purpose = "test"
category = "test-support"
```

When repository evidence confirms these files are test helpers and the finer category is useful, add a refinement instead. A refinement applies after every classification stage to files whose decided purpose equals its `purpose`, and changes only their category: purpose, totals, module credit and `rule` stay, so it cannot move code between production and test. These are minimal complete configurations; when merging into existing policy, preserve the top-level `version` and unrelated entries. Keep the ID unique; if `[categories]` is present, declare `test-support` there.

<!-- category-refinement -->
```toml
version = 1

[[refinements]]
id = "test-helpers"
purpose = "test"
match = ["__test__/helpers/**"]
category = "test-support"
reason = "Shared setup for the test suites"
```

Verify with `locmetry explain __test__/helpers/setup.js --root . --format json`: `result.file.rule` still names the test-directory convention, `result.file.category` becomes `test-support`, and `result.decision.refinement` names the refinement with its matching patterns and reason. The first matching refinement decides; use `except` to leave files to a later one. Refinements never change files that reached the fallback, which stay `unclaimed` for review, or files a generated-code marker refined, which stay `generated`. `summary.policy_entries` reports for each refinement the files it relabeled, under `displaced` the categories it replaced, and under `preempted` the earlier refinements that took matching files. A category subtotal covers files assigned that label; it is not proof that all conceptual helper code has been identified.

Then check the whole policy rather than one file. Measure the unrestricted project with `--format table --verbose`, which lists entries that decided nothing and the unused patterns and exceptions of the others, or read `summary.policy_entries` in full JSON. To list which files changed category, capture full reports with `--files` before and after the policy change and run the file-transition helper in `locmetry --guide compare --section inspect-configuration-transitions`. Finish with `locmetry --guide configure --section validate-and-hand-back`.

Use an override only when the purpose itself must change, such as archived tests counted as `other`.

## Test Support or Verification Tooling

Purpose, not category, decides Test/prod: the ratio divides all Test code by all Production code and ignores categories. The `Purpose` definitions (`locmetry --schema config --section Purpose`) say which code is test and which is production: tests and the support that exists only for them are test, while programs that run, prepare or score tests are production. Built-in test conventions label named test-runner configuration as test; keep such defaults unless evidence justifies an override. Run `locmetry --guide categories` for the category names within each purpose, such as `test-support` or `verification-tooling`.

## Worked Configuration

In a fresh temporary working directory, create this synthetic project. Here the project owner confirms that `scripts/tool.py` is maintained tooling, `src/sample.js` is historical demonstration code, and the role of `unknown.py` is unresolved.

```sh
mkdir -p demo/src demo/scripts
printf '{}\n' > demo/package.json
printf 'export const app = 1;\n' > demo/src/app.js
printf 'export const sample = 1;\n' > demo/src/sample.js
printf 'print(1)\n' > demo/scripts/tool.py
printf 'print(2)\n' > demo/unknown.py
locmetry --root demo --no-user-config --format headline
```

Initially npm inference claims both `src` files as production, two code lines. The historical sample therefore never appears in the unclaimed list. The two Python files remain unclaimed. Save this as `demo/.locmetryrc` after reviewing that evidence:

<!-- configuration-example -->
```toml
version = 1

[categories]
tooling = "Maintained project tools"
archived = "Historical demonstrations kept for reference, not maintained"

[[rules]]
id = "tooling"
match = ["scripts/**"]
purpose = "production"
category = "tooling"
reason = "The owner confirms these scripts maintain the project"

[[rules]]
id = "historical-sample"
match = ["src/sample.js"]
purpose = "other"
category = "archived"
reason = "The owner confirms this is a historical demonstration"
```

```sh
locmetry --root demo --no-user-config --format json
locmetry explain src/sample.js --root demo --no-user-config --format json
```

The following checked projection contains `summary.source_purpose` and the paths from `review`, renamed `unclaimed` for readability. The full report has more fields. Two production lines are `src/app.js` and the tool; the two `other` lines are the historical sample and the unresolved file. Only the unresolved file remains unclaimed:

<!-- configuration-result -->
```json
{
  "source_purpose": {
    "production": {"files": 2, "code": 2, "comments": 0, "blanks": 0, "lines": 2},
    "other": {"files": 2, "code": 2, "comments": 0, "blanks": 0, "lines": 2}
  },
  "unclaimed": ["unknown.py"]
}
```

The synthetic commands disable personal defaults for reproducibility. For a real project, compare effective policy before deciding whether an experiment should use `--no-user-config`.

## Validate and Hand Back

Measure the unrestricted project again, without `--subfolder`. Inspect unused entries and exclusions: they may be shadowed, mistyped or unused in the current checkout. Subtree-only findings do not justify deleting project policy. Review coverage too: unavailable counts are not zero, and a successful config load does not establish completeness. Exit 1 means a command/configuration failure; exit 2 means partial counts, not an unresolved purpose decision.

Compare against the saved baseline to check expected transfers, recognition changes and unchanged totals where applicable, including affected data/documentation records. A rule assigning intentional `other` keeps those files counted; a scope exclusion removes them from measurement before classification.

Review the configuration diff separately from the measured transitions. Inspect newly added or broadened `match` globs, changed `kind` restrictions, exclusions and entries moved into `overrides`: they can affect future files that neither capture contains. Facts about named files justify those paths; they do not by themselves justify a pattern such as `components/*/src/helper*.ts`. Use broader matching only when evidence establishes a directory or naming convention's role, and record that basis in `reason`. Correct current totals do not establish that future matching scope is justified.

Finish with resolved decisions and their evidence, intentional `other` entries and reasons, remaining unclaimed questions, coverage limitations and unused-policy findings. Open questions do not prevent delivering a review, but do prevent claiming that all policy is settled. Configuration edits should remain reviewable; preserve unrelated settings.

For personal defaults, retrieve `locmetry --schema user-config`. For multiple projects, use `locmetry workspace --help` and `locmetry --schema workspace`: the workspace file names members; each independent root keeps its own project classification, while the folders of one `project` share its `.locmetryrc`. `locmetry --guide reports` explains member evidence and totals. Guides and schemas are standalone queries: do not append root, config or output options; use shell redirection to save them. [PRODUCT](../PRODUCT.md#project-configuration) owns the full contract.
