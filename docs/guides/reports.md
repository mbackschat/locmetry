# Read Counts, Coverage and Reports

Start with `locmetry --root . --format headline` at the intended project root. This compact JSON describes the measurement without loading all file records into your context. Use `locmetry --schema report` for exact fields and omission semantics. Retrieve a full report or a targeted explanation only when the headline leaves a concrete question.

## Choose Content and Destination

| Task | Command |
| --- | --- |
| Source counts by language and folder | `locmetry --root . --format table --folders` |
| Source tables with classification, scope and coverage notes | `locmetry --root . --format table --verbose` |
| Compact JSON | `locmetry --root . --format headline` |
| Full JSON on stdout | `locmetry --root . --format json` |
| Full JSON with per-file decisions | `locmetry --root . --files --format json` |
| Save a full baseline outside the root | `locmetry --root service --output before.json` |
| Save Markdown | `locmetry --root service --format markdown --output report.md` |
| Page a plain text report | `locmetry --root service --format table \| less` |

Without explicit options, a terminal gets a table and a pipe gets a headline. Any `--output` defaults to full JSON; explicit `--format` wins. `--output FILE` writes only that file, with empty stdout; `--output -` selects stdout. Extensions do not infer formats. Files are unstyled. `--files` requires full JSON. A headline is not comparison-ready, and cannot reconstruct the omitted baseline later. Aggregate comparison needs full reports, but not per-file records.

Default single-project tables show counted source by language, a reconciliation legend and the measurement date. Add `--folders` for the folder table; workspace repository rows remain visible by default. Default tables deliberately omit non-source totals and unrecognized-file counts, and do not prompt for classification. Use `locmetry --format table --verbose` to inspect notes such as “Unclaimed source” and “Not source code”, build inference, Git submodule hints and unused policy. To inspect or refine purpose classification, use `locmetry --guide configure`; `--files --output -` shows each file's rule. Markdown and JSON retain their existing detail. `--verbose` warns without changing the result in non-table measurement formats.

Real read/parse failures remain visible without `--verbose`: `Not counted:` summarizes only failures, workspace `Unavailable counts:` names affected members, and the backend warning, stderr path diagnostic and exit 2 remain. The report's parse-failure path list requires `--verbose`; unrecognized types use a separate neutral note there. Unclassified source has been counted and does not itself make the report partial.

Protected export destinations must be outside the scan root (every member in workspace mode), and cannot overwrite configuration or workspace files. Shell redirection and `tee` bypass those checks. `explain` is different: JSON is always complete, with no headline or `--output`; use `locmetry explain --help`. Guides and schemas are standalone queries saved by shell redirection, without measurement flags.

For adaptive folder rows and saved tables, see [Look inside large folders](#look-inside-large-folders). `--folders` applies to saved table text and subfolder measurements, warns in non-table formats and is unavailable in workspace mode. `--depth N` controls Markdown depth; `--categories` adds a table view. Explicit ineffective depth/categories warn during measurement. These presentation options do not change counts. Human count tables expose total Lines before Code; Lines = Code + Comments + Blanks, while purpose columns split Code. Table and Markdown end with the measurement time in UTC, from JSON `metadata.measured_at_unix_seconds`; exported files retain this date independently of filesystem timestamps. Use `locmetry --guide selection` before changing the measurement root to explore a large folder.

## Interpret the Numbers

`all_counted` includes source, data and documentation. `source`, purpose totals and language summaries describe source only. Purpose and content kind are separate: an unclassified documentation file is not unclaimed source. `review` lists source left unclaimed; deliberate unclassified entries keep their counts but leave that list. Use `locmetry --guide configure` before deciding how to classify anything.

Full-report `breakdown` rows describe each file's parent folder, split by kind/purpose/category/language. Sum rows whose folder equals a prefix or starts with that prefix plus `/` to obtain recursive totals; `src` must not also match `src-old`. Breakdown rows are additive, including embedded-language rows with zero files. Language summary file counts instead describe participation and can overlap. Do not sum language file counts to recover physical file counts, or average ratios to compute a total ratio.

Summary fields such as `source` and empty maps can be omitted where the schema permits that; an emitted count object includes zero-valued counters. Missing `.files` means per-file detail was not requested; it does not mean no files. Missing full-report sections in a headline are unavailable detail, not zero. Historical missing failure evidence also means “not recorded.” `metadata.scope.subfolder`, when present, means the summary measures that restriction, not the entire project. Workspace paths are member-prefixed, members have separate policy/coverage, and member sums reconcile with workspace totals. Inspect `locmetry workspace --help` and `locmetry --schema workspace` for workspace setup.

## Unavailable Is Not Zero

A selected file can be counted, unrecognized, unreadable or a recorded parse failure. `metadata.coverage` records these distinctions. Recognized-but-unavailable counts set `backend_inaccurate`; the produced report is partial and exits 2. Exit 1 means an error without a promised usable document. Unrecognized files and unresolved classification do not themselves make counts partial. `backend_inaccurate` can also be true without identified failure paths, so empty failure lists alone do not establish completeness. Preserve status before parsing output; shell pipelines can otherwise hide it.

In a fresh temporary working directory:

```sh
mkdir demo
printf '' > demo/empty.py
printf '{}' > demo/broken.ipynb
locmetry --root demo --no-user-config --files --format json
```

This exits 2 and warns on stderr. The following checked projection retains coverage totals, each failure's path/reason and each counted file's path/code. An empty Python file is known zero; the invalid notebook contributes no file record, not an invented zero:

<!-- coverage-result -->
```json
{
  "backend_inaccurate": true,
  "selected_files": 2,
  "counted_files": 1,
  "parse_failures": [{"path": "broken.ipynb", "reason": "invalid_notebook"}],
  "files": [{"path": "empty.py", "code": 0}]
}
```

For one path, use `locmetry --guide selection`. For historical evidence and partial comparisons, use `locmetry --guide compare`. JSON Schema validates structure; Locmetry's reader also checks arithmetic and comparison eligibility. Schema validity alone does not make a headline a full baseline.

## Common issues and useful queries

These queries use the [README sample project](../../README.md#what-you-see), from the directory containing `acme-orders`. They require the optional external `jq` tool, not a Locmetry runtime dependency. Personal defaults are disabled for the synthetic examples. These are interactive inspection pipelines; the [CI recipe](#continuous-integration) demonstrates preserving partial status.

### “Unclaimed source”: which files make up that folder total?

The files were counted, but no rule, convention or module claimed their purpose. The full report's `review` list already contains every unclaimed source file, largest first; no `--files` is needed. List code lines, language and path for `prototypes/`, including descendants:

```sh
locmetry acme-orders --no-user-config --output - | jq -r '.review[] | select(.path | startswith("prototypes/")) | [.code, .language, .path] | @tsv'
```

Check the same folder's file count and code-line total, including the empty case:

```sh
locmetry acme-orders --no-user-config --output - | jq '[.review[] | select(.path | startswith("prototypes/"))] | {files: length, code: (map(.code) | add // 0)}'
```

For all counted source in `web/src/`, including production and tests, request file detail instead:

```sh
locmetry acme-orders --no-user-config --files --output - | jq -r '.files[] | select(.kind == "source" and (.path | startswith("web/src/"))) | [.counts.code, .purpose, .rule, .path] | @tsv'
```


### Which files have unavailable counts?

```sh
locmetry acme-orders --no-user-config --output - | jq '.metadata.coverage'
```

### “Not source code”: which files are data or documentation?

Content kind is separate from purpose: these files contribute to `all_counted`, but not source totals. List their kinds, physical line counts and paths:

```sh
locmetry acme-orders --no-user-config --files --output - | jq -r '.files[] | select(.kind != "source") | [.kind, .counts.lines, .path] | @tsv'
```

For field-level semantics use `locmetry --schema report`; an optional narrow query is `locmetry --schema report | jq '."$defs".Coverage'`. [PRODUCT](../PRODUCT.md#reporting-and-coverage) owns the contract, and [measurement semantics](../measurement.md) records parser limits.

## Look inside large folders

If the table requested with `--folders` concentrates most code in a container such as `web` or `packages`, request deeper detail without changing the scan root or classification context:

```sh
locmetry acme-orders --folder-rows 30
```

This enables the **Folder** table with adaptive detail, without requiring `--folders` as well. The first **Language** table stays as it is. Large folders expand into non-overlapping child rows, including direct files and scoped “other child folders” rows where needed. Every row still contributes once to the total. The budget counts Folder data rows, excluding its header, total and legends. It limits additional detail; existing top-level contributions are never merged away, so an initially wider table can exceed it. `--sort name` compares path components, keeping descendants before siblings such as `src-old`; the same comparison resolves code ties. Single-child chains collapse without using extra rows. Paths stay relative to the project root, including when combined with `--subfolder`.

`--format table` is an existing option that selects human-readable text. It is optional in a terminal, where table output is the default. Pipes normally select a JSON headline, and `--output` defaults to full JSON. Explicitly request table output when saving or paging this view:

```sh
locmetry acme-orders --folder-rows 30 --format table --output folders.txt
```

When the largest displayed row still has folder detail, a gray hint names the folder and suggests increasing `--folder-rows`. This also covers a largest “other child folders” remainder. Smaller expandable folders alone do not trigger it.

The option changes only the table view, not measurement counts, coverage or the Unclaimed source note requested with `--verbose`. It cannot accompany JSON output; save full JSON separately when needed. The verbose note uses its existing partition of unclassified code and can therefore show different folder boundaries. Markdown retains its recursive `--depth` view; workspace mode rejects `--folder-rows`. [Adaptive folder semantics](../PRODUCT.md#adaptive-folder-detail) defines selection, budgets and limits.

## Measure several repositories together

`locmetry workspace` measures sibling repositories as one virtual monorepo. Each repository is measured with its own `.locmetryrc`, the repositories run in parallel, and the results merge into one report whose top-level folders are the repositories:

```sh
locmetry workspace --root ../web --root ../api
```

Roots are relative to the current directory or absolute. To avoid repeating them, list them in a workspace file, where they are relative to the file itself or absolute, and name it with `--file`; `locmetry --schema workspace` describes the format:

```toml
version = 1
roots = ["web", "api", "/srv/shared/billing"]
```

```sh
locmetry workspace --file repos.toml
```

- **Each repository keeps its policy:** `--config`, `--exclude`, `--hidden`, and `--respect-ignore-files` are rejected, because each repository's own configuration decides them, and options must follow `workspace`.
- **Paths name the repository:** every path is prefixed with its repository's folder name, such as `web/src/index.ts`. Two repositories with the same folder name, a root given twice, or one root inside another fail before measuring.
- **Repositories as a dimension:** `summary.members` holds each repository's totals by purpose, category, kind, and language, equal to a regular run of that repository. The table shows one row per repository with its test lines per production line and its share of the workspace, and the Markdown report adds grids of source categories and languages by repository.
- **Findings stay with their repository:** `metadata.members` lists each repository's root, configuration path, scope, coverage counts, unused entries, and unused exclusions; verbose table notes name the repository, as in `ghost (web)`, and list unclaimed source by repository. Default and verbose tables name members with read/parse failures under `Unavailable counts:`; Markdown retains comprehensive member findings.
- **All or nothing, always the same:** if one repository fails, for example with an invalid configuration, the run fails and names it; the order of the roots never changes the report.

## Query a Saved Report

Save full JSON outside the measured root. These queries require `jq` and use the same sample as the interactive queries above; a headline cannot supply the full `breakdown` and `review` sections.

```sh
locmetry acme-orders --no-user-config --output report.json
```

Sum test code by language across additive breakdown rows:

```sh
jq '[.breakdown[] | select(.kind == "source" and .purpose == "test")] | group_by(.language) | map({language: .[0].language, code: (map(.code) | add)})' report.json
```

Sum source code by purpose under `web/`, including descendants but not similarly named siblings:

```sh
jq '[.breakdown[] | select(.kind == "source" and (.folder == "web" or (.folder | startswith("web/"))))] | group_by(.purpose) | map({purpose: .[0].purpose, code: (map(.code) | add)})' report.json
```

Inspect files whose purpose remains unresolved; aggregate queries and this review list need no `--files`:

```sh
jq '.review[]' report.json
```

## Continuous integration

Capture the measurement before applying a policy check, preserving Locmetry's exit status. This script refuses partial reports, fails when more than 50 source lines remain unclaimed, and prints the test/production ratio only when production code is nonzero. `jq -e` fails when the policy expression is false:

```sh
set -eu
report=$(locmetry --format headline) || exit "$?"
printf '%s\n' "$report" | jq -e '([.summary.unclassified_folders[].code] | add // 0) <= 50'
printf '%s\n' "$report" | jq -r '
  (.summary.source_purpose // {}) as $p |
  if ($p.production.code // 0) == 0 then
    "test/production: n/a (no production code)"
  else
    "test/production: \(($p.test.code // 0) * 100 / $p.production.code | floor)%"
  end'
```

[CI executes this block and the comparison/explanation workflows](../../scripts/check-doc-examples.sh) against synthetic projects. The policy scenarios check stdout and stderr as well as status, so invalid configuration cannot masquerade as an unclaimed-code rejection. The Cargo tests separately guard its JSON and exit-code assumptions without requiring `jq`.

[Command outcomes](../PRODUCT.md#command-outcomes) defines exit statuses for measurement, comparison and explanation, including backend inaccuracy without identified failure paths.
