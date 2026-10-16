# Read Counts, Coverage and Reports

Start with `locmetry --root . --format headline` at the intended project root. This compact JSON describes the measurement without loading all file records into your context. Use `locmetry --schema report` for exact fields and omission semantics. Retrieve a full report or a targeted explanation only when the headline leaves a concrete question.

## Choose Content and Destination

| Task | Command |
| --- | --- |
| Compact JSON | `locmetry --root . --format headline` |
| Full JSON on stdout | `locmetry --root . --format json` |
| Full JSON with per-file decisions | `locmetry --root . --files --format json` |
| Save a full baseline outside the root | `locmetry --root service --output before.json` |
| Save Markdown | `locmetry --root service --format markdown --output report.md` |
| Page a plain text report | `locmetry --root service --format table \| less` |

Without explicit options, a terminal gets a table and a pipe gets a headline. Any `--output` defaults to full JSON; explicit `--format` wins. `--output FILE` writes only that file, with empty stdout; `--output -` selects stdout. Extensions do not infer formats. Files are unstyled. `--files` requires full JSON. A headline is not comparison-ready, and cannot reconstruct the omitted baseline later. Aggregate comparison needs full reports, but not per-file records.

Protected export destinations must be outside the scan root (every member in workspace mode), and cannot overwrite configuration or workspace files. Shell redirection and `tee` bypass those checks. `explain` is different: JSON is always complete, with no headline or `--output`; use `locmetry explain --help`. Guides and schemas are standalone queries saved by shell redirection, without measurement flags.

The Folder table's `--folder-rows N` increases additive detail for a single root and requires table format. `--depth N` controls Markdown depth; `--categories` adds a table view. Explicit ineffective depth/categories warn during measurement. These presentation options do not change counts. Human count tables expose total Lines before Code; Lines = Code + Comments + Blanks, while purpose columns split Code. Table and Markdown end with the measurement time in UTC, from JSON `metadata.measured_at_unix_seconds`; exported files retain this date independently of filesystem timestamps. Use `locmetry --guide selection` before changing the measurement root to explore a large folder.

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

These queries use the [README sample project](../../README.md#a-tour-of-a-sample-project), from the directory containing `acme-orders`. They require the optional external `jq` tool, not a Locmetry runtime dependency. Personal defaults are disabled for the synthetic examples. These are interactive inspection pipelines; the [CI recipe](../../README.md#continuous-integration) demonstrates preserving partial status.

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
