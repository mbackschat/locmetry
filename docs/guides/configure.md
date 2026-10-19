# Create or Evaluate Project Configuration

Use this guide to create `.locmetryrc` or review an existing one. Start at the intended project root and inspect any existing configuration before proposing edits. Preserve policy unless repository evidence or the owner's intent justifies a change. Locmetry validates syntax and reports observed decisions; it cannot decide what the project ought to measure.

## Start with Evidence

The first measurement precedes edits. These commands work with the installed binary alone; `.` means the intended project root:

<!-- discovery-commands -->
```sh
locmetry --help
locmetry --guide configure
locmetry --schema config
locmetry --root . --format headline
```

Read the compact headline first: effective scope, loaded configuration paths, coverage, `summary.submodules`, `summary.unclaimed_folders` and unused-policy findings. `.locmetryrc` is discovered only at the scan root, not in ancestors. `--config FILE` replaces project discovery; its patterns still use the scan root as their base. Personal defaults can affect selection and language recognition; inspect them rather than silently disabling them for a real project.

Decide whether submodules, vendored dependencies and reference material belong in the intended measurement. Exclude out-of-scope trees through scope policy; keep maintained project code measured and classify it from evidence. A submodule hint does not establish ownership. Report uncertain scope to the owner instead of assuming that all external-looking code should be excluded.

The config schema describes accepted keys, defaults, exact built-ins and language names. To retrieve smaller sections, an optional external tool such as `jq` can help:

```sh
locmetry --schema config | jq -r .description
locmetry --schema config | jq '."$defs".Classification'
```

Locmetry does not require `jq`. Request `locmetry --guide reports` for evidence meanings and narrow report queries, or `locmetry --guide selection` for a missing or surprising file. Do not fetch every guide, schema or file record before you have a question.

## Decide Purpose, Not Just Coverage

Each file has one content kind, one purpose and one category. Classification uses first match: overrides → built-in test/build conventions → rules → inferred build modules → fallback. List order matters within overrides/rules. A rule cannot beat a convention; an override can. Scope exclusions run before classification and do not reclassify files.

Review representative already-classified source as well as the unclaimed list. Module inference can mark fixtures or historical samples as production, so zero unclaimed code is not proof of correctness. Names suggest inspection, not a verdict. Use a targeted explanation before changing policy:

```sh
locmetry explain src/main.py --root . --format json
```

Substitute an existing root-relative path. Inspect `result.file.rule`, `result.decision.stage` and any entry reason or module evidence. Correct built-in/module decisions need no redundant entry. For source that does need a decision, there are three legitimate outcomes:

1. Classify it through a narrow evidence-based entry.
2. Count it deliberately as other through an entry with `purpose = "other"` and a `reason`; it then leaves the unclaimed review list.
3. Leave it unclaimed and report the open question to the owner. Do not manufacture a rule just to remove the finding.

Globs match root-relative file paths. `docs/**` covers descendants; `docs` does not mean the directory. `*.py` matches root files only; `**/*.py` also covers deeper paths. Patterns are case-sensitive. Use `[[rules]]` for decisions after conventions, and `[[overrides]]` only when the evidence requires winning before conventions. An optional `[categories]` table documents permitted custom categories and catches typos. The schema supplies the complete matching contract.

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
historical = "Historical demonstrations outside maintained implementation"

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
category = "historical"
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

Finish with resolved decisions and their evidence, intentional `other` entries and reasons, remaining unclaimed questions, coverage limitations and unused-policy findings. Open questions do not prevent delivering a review, but do prevent claiming that all policy is settled. Configuration edits should remain reviewable; preserve unrelated settings.

For personal defaults, retrieve `locmetry --schema user-config`. For multiple projects, use `locmetry workspace --help` and `locmetry --schema workspace`: the workspace file names members; each member keeps its own project classification. `locmetry --guide reports` explains member evidence and totals. Guides and schemas are standalone queries: do not append root, config or output options; use shell redirection to save them. [PRODUCT](../PRODUCT.md#project-configuration) owns the full contract.
