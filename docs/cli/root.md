# Measure a Project

*2026-10-20T12:56:35Z by Showboat 0.6.1*
<!-- showboat-id: f949f865-6391-4e04-bcf2-5a1eae54dc23 -->

Run each Bash block from the Locmetry repository root. [Setup and replay instructions](README.md) describe the fresh temporary copy created by `source scripts/cli-demo-env.sh`, external tools and output projections. The [fixture README](../../examples/cli-workspace/README.md) introduces the shared source tree.

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
 TypeScript      3     10     7 │        3       0 │     3     3      1 │      1.00
 HTML            2      5     5 │        0       0 │     5     0      0 │      0.00
 Python          2      5     3 │        2       0 │     2     0      1 │      0.00
 TSX             1      3     3 │        0       0 │     3     0      0 │      0.00
 Makefile        1      2     2 │        0       0 │     2     0      0 │      0.00
 JavaScript      0      1     1 │        0       0 │     1     0      0 │      0.00
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 Total          11     37    29 │        6       2 │    20     7      2 │      0.35
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
 HTML            2      5     5 │        0       0 │     5     0      0 │      0.00
 JavaScript      0      1     1 │        0       0 │     1     0      0 │      0.00
 Makefile        1      2     2 │        0       0 │     2     0      0 │      0.00
 Python          2      5     3 │        2       0 │     2     0      1 │      0.00
 TSX             1      3     3 │        0       0 │     3     0      0 │      0.00
 TypeScript      3     10     7 │        3       0 │     3     3      1 │      1.00
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 Total          11     37    29 │        6       2 │    20     7      2 │      0.35
 Lines = Code + Comments + Blanks; Production + Test + Other = Code; Test/prod = Test / Production (Code).
 Tests written in another language than the code they test skew per-language Test/prod.

 ────────────────────────────────────────────────────────────────────
 Folder                  Lines  Code │ Prod.  Test  Other │ Test/prod
 ────────────────────────────────────┼────────────────────┼──────────
 service (direct files)      2     2 │     2     0      0 │      0.00
 service/experiments         2     1 │     0     0      1 │         -
 service/scripts             3     2 │     2     0      0 │      0.00
 service/src                 6     4 │     4     0      0 │      0.00
 service/tests               5     4 │     0     4      0 │         -
 web/src (direct files)      5     5 │     5     0      0 │      0.00
 web/src/api                 2     1 │     0     0      1 │         -
 web/src/orders             12    10 │     7     3      0 │      0.43
 ────────────────────────────────────┼────────────────────┼──────────
 Total                      37    29 │    20     7      2 │      0.35
 Production + Test + Other = Code; Test/prod = Test / Production (Code).
 Adaptive folder detail; rows cover measured source once.

 ─────────────────────────────────────────────
 Category                   Files  Lines  Code
 ─────────────────────────────────────────────
 other:generated                1      2     1
 other:unclaimed                1      2     1
 production:build-tooling       1      2     2
 production:implementation      5     19    16
 production:tooling             1      3     2
 test:test                      2      9     7
 Rows of one purpose add up to that purpose's Code; all rows add up to the total Code.

 Not source code: 2 data files (2 lines), 1 documentation file (3 lines); --files --output - lists each file's kind.
 Production from build modules: go 4 code lines in 1 module, npm 12 code lines in 1 module.
 Largest module-claimed production; generated or vendored code here needs an entry:
   web/src/index.html              5 code lines  npm web
   service/src/main.go             4 code lines  go service
   web/src/orders/Order.ts         3 code lines  npm web
   web/src/orders/OrderView.tsx    3 code lines  npm web
   web/src/orders/receipt.acmetpl  1 code line   npm web
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
 HTML            2      5     5 │        0       0 │      0.0%
 TSX             1      3     3 │        0       0 │      0.0%
 TypeScript      1      4     3 │        1       0 │     25.0%  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
 JavaScript      0      1     1 │        0       0 │      0.0%
 ───────────────────────────────┼──────────────────┼────────────────────────────────────────────────
 Total           4     13    12 │        1       0 │      7.7%
 Lines = Code + Comments + Blanks; Comment % = Comments / (Code + Comments).
 Purposes shown: Production; Test and Other excluded (4 code lines).
 Bars: ▓ Comment %; the densest row fills the bar.

 ─────────────────────────────────────────────────────────────────────────────────────────
 Folder                  Lines  Code │ Comment %  Comment density
 ────────────────────────────────────┼────────────────────────────────────────────────────
 web/src/orders              8     7 │     12.5%  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
 web/src (direct files)      5     5 │      0.0%
 ────────────────────────────────────┼────────────────────────────────────────────────────
 Total                      13    12 │      7.7%
 Comment % = Comments / (Code + Comments).
 Bars: ▓ Comment %; the densest row fills the bar.

```

## Focus on languages

`--table-language` filters table counts to the named source languages, in any case, and composes with `--table-purpose`; JSON and Markdown totals stay complete. A note under the language table names the excluded languages and their code, so the shown total reconciles with the full measurement. Embedded languages keep only their own lines.

```bash
source scripts/cli-demo-env.sh
locmetry --root acme-orders --no-user-config --no-color --format table \
  --table-language typescript,tsx \
  | sed '/^Root:/d; /^Roots:/d; /^Measured at:/d' \
  | demo_wrap
```

```output
 ──────────────────────────────────────────────────────────────────────────────────
 Language    Files  Lines  Code │ Comments  Blanks │ Prod.  Test  Other │ Test/prod
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 TypeScript      3     10     7 │        3       0 │     3     3      1 │      1.00
 TSX             1      3     3 │        0       0 │     3     0      0 │      0.00
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 Total           4     13    10 │        3       0 │     6     3      1 │      0.50
 Lines = Code + Comments + Blanks; Production + Test + Other = Code; Test/prod = Test / Production (Code).
 Tests written in another language than the code they test skew per-language Test/prod.
 Languages shown: TypeScript and TSX; Go, HTML, Python, Makefile and JavaScript excluded (19 code lines).

```

## Print only the tables

`--tables-only` prints the tables and nothing else: no legends, notes or location/time footer, so a transcript or script needs no filter. The columns keep the meanings the legends above define. A `--table-purpose` or `--table-language` selection is no longer named in the output; the command records it.

```bash
source scripts/cli-demo-env.sh
locmetry --root acme-orders --no-user-config --no-color --no-bars --format table \
  --show-folders --show-categories --tables-only
```

```output
 ──────────────────────────────────────────────────────────────────────────────────
 Language    Files  Lines  Code │ Comments  Blanks │ Prod.  Test  Other │ Test/prod
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 Go              2     11     8 │        1       2 │     4     4      0 │      1.00
 TypeScript      3     10     7 │        3       0 │     3     3      1 │      1.00
 HTML            2      5     5 │        0       0 │     5     0      0 │      0.00
 Python          2      5     3 │        2       0 │     2     0      1 │      0.00
 TSX             1      3     3 │        0       0 │     3     0      0 │      0.00
 Makefile        1      2     2 │        0       0 │     2     0      0 │      0.00
 JavaScript      0      1     1 │        0       0 │     1     0      0 │      0.00
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 Total          11     37    29 │        6       2 │    20     7      2 │      0.35

 ─────────────────────────────────────────────────────
 Folder   Lines  Code │ Prod.  Test  Other │ Test/prod
 ─────────────────────┼────────────────────┼──────────
 web         19    16 │    12     3      1 │      0.25
 service     18    13 │     8     4      1 │      0.50
 ─────────────────────┼────────────────────┼──────────
 Total       37    29 │    20     7      2 │      0.35

 ─────────────────────────────────────────────
 Category                   Files  Lines  Code
 ─────────────────────────────────────────────
 production:implementation      5     19    16
 test:test                      2      9     7
 production:build-tooling       1      2     2
 production:tooling             1      3     2
 other:generated                1      2     1
 other:unclaimed                1      2     1
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
  "service/Makefile",
  "service/experiments/probe.py",
  "service/ignored.py",
  "service/scripts/check.py",
  "service/src/main.go",
  "web/src/api/client.ts",
  "web/src/index.html",
  "web/src/orders/Order.ts",
  "web/src/orders/OrderView.tsx",
  "web/src/orders/receipt.acmetpl"
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
 TypeScript      3     10     7 │        3       0 │     3     3      1 │      1.00
 HTML            2      5     5 │        0       0 │     5     0      0 │      0.00
 Python          2      5     3 │        2       0 │     2     0      1 │      0.00
 TSX             1      3     3 │        0       0 │     3     0      0 │      0.00
 Makefile        1      2     2 │        0       0 │     2     0      0 │      0.00
 JavaScript      0      1     1 │        0       0 │     1     0      0 │      0.00
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 Total          11     37    29 │        6       2 │    20     7      2 │      0.35
 Lines = Code + Comments + Blanks; Production + Test + Other = Code; Test/prod = Test / Production (Code).
 Tests written in another language than the code they test skew per-language Test/prod.

 Code share  █ Production  ▓ Test  ░ Other
 Go          27.6%  ████████████████████▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
 TypeScript  24.1%  ███████████████▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░░░
 HTML        17.2%  █████████████████████████
 Python      10.3%  ██████████░░░░░
 TSX         10.3%  ███████████████
 Makefile     6.9%  ██████████
 JavaScript   3.4%  █████

{
  "document_type": "measurement",
  "schema_version": "3",
  "detail": "full",
  "source": {
    "files": 11,
    "code": 29,
    "comments": 6,
    "blanks": 2,
    "lines": 37
  },
  "physical_files": 14
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
  "source_code": 29,
  "files": 14
}
## Folder breakdown

*Recursive source code totals to depth 3. Parent rows include their children; do not sum all displayed rows. The report
file's additive `breakdown` rows cover every depth. Multi-language files count once physically but contribute to each
contained language. Production + Test + Other = Code; Test/prod = Test / Production (Code).*

| Folder | Lines | **Code** | Production | Test | Other | Test/prod |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| . | 37 | 29 | 20 | 7 | 2 | 0.35 |
| service | 18 | 13 | 8 | 4 | 1 | 0.50 |
| service/experiments | 2 | 1 | 0 | 0 | 1 | - |
| service/scripts | 3 | 2 | 2 | 0 | 0 | 0.00 |
| service/src | 6 | 4 | 4 | 0 | 0 | 0.00 |
| service/tests | 5 | 4 | 0 | 4 | 0 | - |
| web | 19 | 16 | 12 | 3 | 1 | 0.25 |
| web/src | 19 | 16 | 12 | 3 | 1 | 0.25 |
| web/src/api | 2 | 1 | 0 | 0 | 1 | - |
| web/src/orders | 12 | 10 | 7 | 3 | 0 | 0.43 |

## Build modules
{
  "document_type": "measurement",
  "schema_version": "3",
  "detail": "headline",
  "source": {
    "files": 11,
    "code": 29,
    "comments": 6,
    "blanks": 2,
    "lines": 37
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
locmetry 0.7.6
Source: docs/guides/configure.md; locmetry 0.7.6
Source: docs/guides/selection.md; locmetry 0.7.6
Source: docs/guides/reports.md; locmetry 0.7.6
Source: docs/guides/compare.md; locmetry 0.7.6
Configuration
MeasurementReport
Workspace
Comparison
Explanation
```

## Find stale configuration

Entries and exclusions that match nothing are reported, so policy does not silently outlive the code it describes. This candidate keeps the vendor exclusion but adds one for a removed folder and a rule for tools that moved; `--config` measures with it instead of the fixture policy. Both findings name the unused entry; the stale entries themselves decide no file.

```bash
source scripts/cli-demo-env.sh
cat > stale.toml <<'TOML'
version = 1

[scope]
exclude = ["service/third_party/**", "web/vendor/**"]

[[rules]]
id = "legacy-tools"
match = ["service/tools/**"]
kind = ["source"]
purpose = "production"
category = "tooling"
reason = "Developer commands before they moved to service/scripts"
TOML
locmetry --root acme-orders --no-user-config --config stale.toml --format json \
  | jq ".summary | {unused_entries, unused_exclusions}"

```

```output
{
  "unused_entries": [
    "legacy-tools"
  ],
  "unused_exclusions": [
    "web/vendor/**"
  ]
}
```
