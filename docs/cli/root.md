# Measure a Project

*2026-10-20T12:56:35Z by Showboat 0.6.1*
<!-- showboat-id: f949f865-6391-4e04-bcf2-5a1eae54dc23 -->

Run each Bash block from the Locmetry repository root. [Setup and replay instructions](README.md) describe the fresh temporary copy created by `source scripts/cli-demo-env.sh`, external tools and output projections. The [sample tour](../sample-project-tour.md) introduces the shared source tree.

## Measure the whole tree

`--root` selects the project. The equivalent positional directory form appears in the next example. Explicit `--format table`, `--no-color` and `--no-bars` make a plain table suitable for a transcript. `sed` omits only the variable location/time footer.

```bash
source scripts/cli-demo-env.sh
locmetry --root acme-orders --no-user-config --no-color --no-bars --format table \
  | sed '/^Root:/d; /^Roots:/d; /^Measured at:/d' \
  | demo_wrap
```

```output
 ──────────────────────────────────────────────────────────────────────────────────
 Language    Files  Lines  Code │ Comments  Blanks │ Prod.  Test  Other │ Test/prod
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 Go              2     11     8 │        1       2 │     4     4      0 │      1.00
 TypeScript      2      8     6 │        2       0 │     3     3      0 │      1.00
 HTML            1      4     4 │        0       0 │     4     0      0 │      0.00
 Python          2      5     3 │        2       0 │     2     0      1 │      0.00
 TSX             1      3     3 │        0       0 │     3     0      0 │      0.00
 JavaScript      0      1     1 │        0       0 │     1     0      0 │      0.00
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 Total           8     32    25 │        5       2 │    17     7      1 │      0.41
 Lines = Code + Comments + Blanks; Production + Test + Other = Code; Test/prod = Test / Production (Code).
 Tests written in another language than the code they test skew per-language Test/prod.

```

## Explore folders and decisions

`--folder-rows` expands folders within a row budget; `--sort name` changes presentation order. `--show-categories` and `--verbose` expose classification and coverage, including the unclaimed experiment and unrecognized files. The positional directory selects the same root.

```bash
source scripts/cli-demo-env.sh
locmetry acme-orders --no-user-config --no-color --no-bars --format table --folder-rows 12 \
  --sort name --show-categories --verbose \
  | sed '/^Root:/d; /^Roots:/d; /^Measured at:/d' \
  | demo_wrap
```

```output
 ──────────────────────────────────────────────────────────────────────────────────
 Language    Files  Lines  Code │ Comments  Blanks │ Prod.  Test  Other │ Test/prod
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 Go              2     11     8 │        1       2 │     4     4      0 │      1.00
 HTML            1      4     4 │        0       0 │     4     0      0 │      0.00
 JavaScript      0      1     1 │        0       0 │     1     0      0 │      0.00
 Python          2      5     3 │        2       0 │     2     0      1 │      0.00
 TSX             1      3     3 │        0       0 │     3     0      0 │      0.00
 TypeScript      2      8     6 │        2       0 │     3     3      0 │      1.00
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 Total           8     32    25 │        5       2 │    17     7      1 │      0.41
 Lines = Code + Comments + Blanks; Production + Test + Other = Code; Test/prod = Test / Production (Code).
 Tests written in another language than the code they test skew per-language Test/prod.

 ────────────────────────────────────────────────────────────────────
 Folder                  Lines  Code │ Prod.  Test  Other │ Test/prod
 ────────────────────────────────────┼────────────────────┼──────────
 service/experiments         2     1 │     0     0      1 │         -
 service/scripts             3     2 │     2     0      0 │      0.00
 service/src                 6     4 │     4     0      0 │      0.00
 service/tests               5     4 │     0     4      0 │         -
 web/src (direct files)      5     5 │     5     0      0 │      0.00
 web/src/orders             11     9 │     6     3      0 │      0.50
 ────────────────────────────────────┼────────────────────┼──────────
 Total                      32    25 │    17     7      1 │      0.41
 Production + Test + Other = Code; Test/prod = Test / Production (Code).
 Adaptive folder detail; rows cover measured source once.

 ─────────────────────────────────────────────
 Category                   Files  Lines  Code
 ─────────────────────────────────────────────
 other:unclaimed                1      2     1
 production:implementation      4     18    15
 production:tooling             1      3     2
 test:test                      2      9     7
 Rows of one purpose add up to that purpose's Code; all rows add up to the total Code.

 Not source code: 2 data files (2 lines), 1 documentation file (3 lines); --files --output - lists each file's kind.
 Production from build modules: go 4 code lines in 1 module, npm 11 code lines in 1 module.
 Unclaimed source: 1 code line in 1 file, here:
   service/experiments  1 code line  1 file
 To inspect or refine purpose classification, use locmetry --guide configure; --files --output - shows each file's rule.
 Unrecognized files: 2 unrecognized files (asset: 1, mod: 1); missing lines are not zero.

```

## Focus on production in a subtree

`--subfolder` keeps the original policy and module context. `--show-folders` adds top-level rows; `--table-purpose production` filters table counts and replaces Test/prod with comment density, leaving JSON and Markdown totals complete. `--show-bars=comments` charts comment density relative to the densest row. Selecting just one table purpose also uses comment-density bars with bare `--show-bars` or explicit `=purpose`. The purpose option accepts a comma-separated list.

```bash
source scripts/cli-demo-env.sh
locmetry --root acme-orders --subfolder web/src --no-user-config --no-color --format table \
  --show-folders --table-purpose production --show-bars=comments \
  | sed '/^Root:/d; /^Roots:/d; /^Measured at:/d' \
  | demo_wrap
```

```output
 ───────────────────────────────────────────────────────────────────────────────────────────────────
 Language    Files  Lines  Code │ Comments  Blanks │ Comment %  Comment density
 ───────────────────────────────┼──────────────────┼────────────────────────────────────────────────
 HTML            1      4     4 │        0       0 │      0.0%
 TSX             1      3     3 │        0       0 │      0.0%
 TypeScript      1      4     3 │        1       0 │     25.0%  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
 JavaScript      0      1     1 │        0       0 │      0.0%
 ───────────────────────────────┼──────────────────┼────────────────────────────────────────────────
 Total           3     12    11 │        1       0 │      8.3%
 Lines = Code + Comments + Blanks; Comment % = Comments / (Code + Comments).
 Purposes shown: Production; Test and Other excluded.
 Bars: ▓ Comment %; the densest row fills the bar.

 ─────────────────────────────────────────────────────────────────────────────────────────
 Folder                  Lines  Code │ Comment %  Comment density
 ────────────────────────────────────┼────────────────────────────────────────────────────
 web/src/orders              7     6 │     14.3%  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
 web/src (direct files)      5     5 │      0.0%
 ────────────────────────────────────┼────────────────────────────────────────────────────
 Total                      12    11 │      8.3%
 Comment % = Comments / (Code + Comments).
 Bars: ▓ Comment %; the densest row fills the bar.

```

## Override selection

The fixture policy hides dotfiles and its ignore file excludes scratch work. These CLI settings include both, while the repeated `--exclude` flags remove vendor and test paths. `--config` replaces discovered project policy; its patterns remain relative to the scan root. JSON is projected to the resulting source paths.

```bash
source scripts/cli-demo-env.sh
locmetry --root acme-orders --config acme-orders/.locmetryrc --hidden true \
  --respect-ignore-files false --exclude 'service/tests/**' --exclude 'web/**/*.test.ts' \
  --no-user-config --files --output - \
  | jq '[.files[] | select(.kind == "source") | .path]'
```

```output
[
  "service/.hidden.py",
  "service/experiments/probe.py",
  "service/ignored.py",
  "service/scripts/check.py",
  "service/src/main.go",
  "web/src/index.html",
  "web/src/orders/Order.ts",
  "web/src/orders/OrderView.tsx"
]
```

## Save full evidence and print an overview

`--output` defaults to full JSON regardless of extension. `--files` adds physical-file detail; `--print table` prints the same measurement while saving it. `--show-bars` explicitly adds purpose bars to the printed table. The following query demonstrates the schema-3 envelope and optional file inventory.

```bash
source scripts/cli-demo-env.sh
locmetry --root acme-orders --no-user-config --no-color --files --output report.json \
  --print table --show-bars \
  | sed '/^Root:/d; /^Roots:/d; /^Measured at:/d' \
  | demo_wrap
jq '{
      document_type, schema_version, detail,
      source: .summary.source,
      physical_files: (.files | length)
    }' \
  report.json
```

```output
 ──────────────────────────────────────────────────────────────────────────────────
 Language    Files  Lines  Code │ Comments  Blanks │ Prod.  Test  Other │ Test/prod
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 Go              2     11     8 │        1       2 │     4     4      0 │      1.00
 TypeScript      2      8     6 │        2       0 │     3     3      0 │      1.00
 HTML            1      4     4 │        0       0 │     4     0      0 │      0.00
 Python          2      5     3 │        2       0 │     2     0      1 │      0.00
 TSX             1      3     3 │        0       0 │     3     0      0 │      0.00
 JavaScript      0      1     1 │        0       0 │     1     0      0 │      0.00
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 Total           8     32    25 │        5       2 │    17     7      1 │      0.41
 Lines = Code + Comments + Blanks; Production + Test + Other = Code; Test/prod = Test / Production (Code).
 Tests written in another language than the code they test skew per-language Test/prod.

 Code share  █ Production  ▓ Test  ░ Other
 Go          32.0%  ████████████████████▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
 TypeScript  24.0%  ███████████████▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
 HTML        16.0%  ████████████████████
 Python      12.0%  ██████████░░░░░
 TSX         12.0%  ███████████████
 JavaScript   4.0%  █████

{
  "document_type": "measurement",
  "schema_version": "3",
  "detail": "full",
  "source": {
    "files": 8,
    "code": 25,
    "comments": 5,
    "blanks": 2,
    "lines": 32
  },
  "physical_files": 11
}
```

## Choose Markdown or headline output

`--markdown-depth` limits only the Markdown Folder breakdown; other evidence can still name deeper paths. The first command saves Markdown and prints full JSON with per-file records from the same scan: `--files` applies to that printed JSON. The query selects stable counts, and the Markdown excerpt selects its folder section. Headline output is compact and cannot serve as a comparison baseline. File exports are silent unless `--print` is supplied; `--output -` selects stdout.

```bash
source scripts/cli-demo-env.sh
locmetry --root acme-orders --no-user-config --format markdown --markdown-depth 3 --sort name \
  --output report.md --print json --files \
  | jq '{detail, source_code: .summary.source.code, files: (.files | length)}'
sed -n '/^## Folder breakdown/,/^## Build modules/p' report.md | demo_wrap
locmetry --root acme-orders --no-user-config --format headline \
  | jq '{document_type, schema_version, detail, source: .summary.source}'
```

```output
{
  "detail": "full",
  "source_code": 25,
  "files": 11
}
## Folder breakdown

*Recursive source code totals to depth 3. Parent rows include their children; do not sum all displayed rows. The report
file's additive `breakdown` rows cover every depth. Multi-language files count once physically but contribute to each
contained language. Production + Test + Other = Code; Test/prod = Test / Production (Code).*

| Folder | Lines | **Code** | Production | Test | Other | Test/prod |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| . | 32 | 25 | 17 | 7 | 1 | 0.41 |
| service | 16 | 11 | 6 | 4 | 1 | 0.67 |
| service/experiments | 2 | 1 | 0 | 0 | 1 | - |
| service/scripts | 3 | 2 | 2 | 0 | 0 | 0.00 |
| service/src | 6 | 4 | 4 | 0 | 0 | 0.00 |
| service/tests | 5 | 4 | 0 | 4 | 0 | - |
| web | 16 | 14 | 11 | 3 | 0 | 0.27 |
| web/src | 16 | 14 | 11 | 3 | 0 | 0.27 |
| web/src/orders | 11 | 9 | 6 | 3 | 0 | 0.50 |

## Build modules
{
  "document_type": "measurement",
  "schema_version": "3",
  "detail": "headline",
  "source": {
    "files": 8,
    "code": 25,
    "comments": 5,
    "blanks": 2,
    "lines": 32
  }
}
```

## Discover syntax and contracts

Help lists syntax; guides explain workflows; schemas define machine-readable fields. These are standalone queries. Their projections keep the transcript short. The version query is deliberately recorded, so a release version change requires refreshing this example.

```bash
source scripts/cli-demo-env.sh
locmetry --no-color --help | sed -n '1,16p'
locmetry --version
for topic in configure selection reports compare; do
  locmetry --guide "$topic" | sed -n '1p'
done
for schema in config report workspace comparison explanation; do
  locmetry --schema "$schema" | jq -r '.title'
done
```

```output
Measure source code by language, folder, and purpose: production, test, or other. Defaults to a
table on a terminal, a JSON headline in pipes. --output FILE saves only to that file, defaulting to
full JSON; --format overrides the format.

Usage: locmetry [OPTIONS] [DIR] [COMMAND]

Commands:
  workspace  Measure several repositories as one virtual monorepo: each root with its own
             .locmetryrc, merged with paths prefixed by the root's folder name. Defaults to a table
             on a terminal, a JSON headline in pipes. --output FILE saves only to that file,
             defaulting to full JSON; --format overrides the format
  compare    Compare two saved full reports: net change is after minus before. Defaults to a table
             on a terminal, a JSON headline in pipes. --output FILE saves only to that file,
             defaulting to full JSON; --format overrides the format
  explain    Explain selection and classification of one existing root-relative file. JSON prints
             the complete explanation; there is no separate headline
locmetry 0.7.2
Source: docs/guides/configure.md; locmetry 0.7.2
Source: docs/guides/selection.md; locmetry 0.7.2
Source: docs/guides/reports.md; locmetry 0.7.2
Source: docs/guides/compare.md; locmetry 0.7.2
Configuration
MeasurementReport
Workspace
Comparison
Explanation
```
