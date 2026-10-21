# Measure a Workspace

*2026-10-20T12:56:36Z by Showboat 0.6.1*
<!-- showboat-id: 448a7c9d-e870-462f-b593-af2c578de37b -->

Run each Bash block from the Locmetry repository root. [Setup and replay instructions](README.md) describe the fresh temporary copy created by `source scripts/cli-demo-env.sh`, external tools and output projections. The [fixture README](../../examples/cli-workspace/README.md) introduces the shared source tree.

## Use a workspace file

`--workspace-file` resolves member roots relative to the TOML file. Each member loads its own policy. The source total agrees with the combined root, but member names and configuration provenance differ. The location/time footer is omitted explicitly.

```bash
source scripts/cli-demo-env.sh
locmetry workspace --workspace-file workspace.toml --no-user-config --no-color --no-bars --format table \
  --verbose --show-categories --sort name \
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

 ─────────────────────────────────────────────────────────────────
 Repository  Lines  Code │ Prod.  Test  Other │ Test/prod │  Share
 ────────────────────────┼────────────────────┼───────────┼───────
 service        16    11 │     6     4      1 │      0.67 │  44.0%
 web            16    14 │    11     3      0 │      0.27 │  56.0%
 ────────────────────────┼────────────────────┼───────────┼───────
 Total          32    25 │    17     7      1 │      0.41 │ 100.0%
 Production + Test + Other = Code; Test/prod = Test / Production (Code); Share = Code / workspace Code.

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
 Unclaimed source by repository: service: 1 unclaimed file (1 code line).
 Unrecognized files: 2 unrecognized files (asset: 1, mod: 1); missing lines are not zero.

```

## Name roots directly

Repeated `--root` is an alternative to `--workspace-file`. Root folder names become member names and path prefixes. Full JSON with `--files` retains those paths; `--output` writes silently. Workspace output cannot be inside any member.

```bash
source scripts/cli-demo-env.sh
locmetry workspace --root acme-orders/service --root acme-orders/web --no-user-config --files \
  --output workspace.json
jq '{
      document_type, detail,
      source: .summary.source,
      members: (.summary.members | keys),
      source_paths: [.files[] | select(.kind == "source") | .path]
    }' \
  workspace.json
```

```output
{
  "document_type": "measurement",
  "detail": "full",
  "source": {
    "files": 8,
    "code": 25,
    "comments": 5,
    "blanks": 2,
    "lines": 32
  },
  "members": [
    "service",
    "web"
  ],
  "source_paths": [
    "service/experiments/probe.py",
    "service/scripts/check.py",
    "service/src/main.go",
    "service/tests/main_test.go",
    "web/src/index.html",
    "web/src/orders/Order.test.ts",
    "web/src/orders/Order.ts",
    "web/src/orders/OrderView.tsx"
  ]
}
```

## Select purposes and show shares

`--table-purpose production,test` restricts table counts. `--show-bars` charts purpose shares; `--sort name` makes the repository order explicit. Shared presentation flags have the same meaning as in root measurement.

```bash
source scripts/cli-demo-env.sh
locmetry workspace --workspace-file workspace.toml --no-user-config --no-color --format table \
  --table-purpose production,test --show-bars --sort name \
  | sed '/^Root:/d; /^Roots:/d; /^Measured at:/d' \
  | demo_wrap
```

```output
 ───────────────────────────────────────────────────────────────────────────────────────────────────
 Language    Files  Lines  Code │ Comments  Blanks │ Prod.  Test │ Test/prod │  Share  Code share
 ───────────────────────────────┼──────────────────┼─────────────┼───────────┼──────────────────────
 Go              2     11     8 │        1       2 │     4     4 │      1.00 │  33.3%  ███████▓▓▓▓▓▓
 HTML            1      4     4 │        0       0 │     4     0 │      0.00 │  16.7%  ███████
 JavaScript      0      1     1 │        0       0 │     1     0 │      0.00 │   4.2%  ██
 Python          1      3     2 │        1       0 │     2     0 │      0.00 │   8.3%  ███
 TSX             1      3     3 │        0       0 │     3     0 │      0.00 │  12.5%  █████
 TypeScript      2      8     6 │        2       0 │     3     3 │      1.00 │  25.0%  █████▓▓▓▓▓
 ───────────────────────────────┼──────────────────┼─────────────┼───────────┼──────────────────────
 Total           7     30    24 │        4       2 │    17     7 │      0.41 │ 100.0%
 Lines = Code + Comments + Blanks; Production + Test = Code; Test/prod = Test / Production (Code).
 Tests written in another language than the code they test skew per-language Test/prod.
 Purposes shown: Production and Test; Other excluded.
 Share = Code / total Code. Bars: █ Production  ▓ Test; the largest row fills the bar.

 ───────────────────────────────────────────────────────────────────────────────────────────────────
 Repository  Lines  Code │ Prod.  Test │ Test/prod │  Share  Code share
 ────────────────────────┼─────────────┼───────────┼────────────────────────────────────────────────
 service        14    10 │     6     4 │      0.67 │  41.7%  █████████████████▓▓▓▓▓▓▓▓▓▓▓
 web            16    14 │    11     3 │      0.27 │  58.3%  ███████████████████████████████▓▓▓▓▓▓▓▓
 ────────────────────────┼─────────────┼───────────┼────────────────────────────────────────────────
 Total          30    24 │    17     7 │      0.41 │ 100.0%
 Production + Test = Code; Test/prod = Test / Production (Code); Share = Code / workspace Code.
 Bars: █ Production  ▓ Test; the largest row fills the bar.

```

## Export and print together

`--print` accompanies a real file destination. Here the file has complete evidence and stdout has a headline. `--output -` instead emits the complete document to stdout. The stable projection shows the member totals.

```bash
source scripts/cli-demo-env.sh
locmetry workspace --workspace-file workspace.toml --no-user-config --output workspace.json \
  --print headline \
  | jq '{document_type, detail, source: .summary.source}'
locmetry workspace --workspace-file workspace.toml --no-user-config --output - \
  | jq '.summary.members | with_entries(.value = .value.source)'
```

```output
{
  "document_type": "measurement",
  "detail": "headline",
  "source": {
    "files": 8,
    "code": 25,
    "comments": 5,
    "blanks": 2,
    "lines": 32
  }
}
{
  "service": {
    "files": 4,
    "code": 11,
    "comments": 3,
    "blanks": 2,
    "lines": 16
  },
  "web": {
    "files": 4,
    "code": 14,
    "comments": 2,
    "blanks": 0,
    "lines": 16
  }
}
```

## Share Markdown detail

`--markdown-depth` bounds only the Markdown Folder breakdown; member names occupy level 1 and their immediate folders level 2. Other evidence can still name deeper paths. The displayed section comes from the saved document; the saved file also includes metadata and other sections.

```bash
source scripts/cli-demo-env.sh
locmetry workspace --workspace-file workspace.toml --no-user-config --format markdown --markdown-depth 3 \
  --sort name --output workspace.md
sed -n '/^## Source purpose/,/^## Source categories/p' workspace.md
```

```output
## Source purpose

| Purpose | Files | Lines | **Code** | Comments | Blanks |
| --- | ---: | ---: | ---: | ---: | ---: |
| production | 5 | 21 | 17 | 3 | 1 |
| test | 2 | 9 | 7 | 1 | 1 |
| other | 1 | 2 | 1 | 1 | 0 |

## Repositories

| Repository | Lines | **Code** | Production | Test | Other | Test/prod | Share |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| service | 16 | 11 | 6 | 4 | 1 | 0.67 | 44.0% |
| web | 16 | 14 | 11 | 3 | 0 | 0.27 | 56.0% |
| **Total** | **32** | **25** | **17** | **7** | **1** | **0.41** | **100.0%** |

*Production + Test + Other = Code; Test/prod = Test / Production (Code); Share = Code / workspace Code.*

## Source categories by repository
```

## Inspect available options

Workspace selection is configured per member: root-only `--config`, `--exclude`, `--hidden`, `--respect-ignore-files` and `--subfolder` are unsupported here. Use `explain --workspace-file` to diagnose a member path. The help excerpt shows selection and output groups; full command help also lists presentation options.

```bash
source scripts/cli-demo-env.sh
locmetry workspace --no-color --help | sed '/^Presentation:/,$d'
locmetry explain web/src/orders/Order.test.ts --workspace-file workspace.toml --no-user-config \
  --format json \
  | jq '{
      member, path,
      result: {state: .result.state, rule: .result.file.rule}
    }'
```

```output
Measure several repositories as one virtual monorepo: each root with its own .locmetryrc, merged
with paths prefixed by the root's folder name. Defaults to a table on a terminal, a JSON headline in
pipes. --output FILE saves only to that file, defaulting to full JSON; --format overrides the format

Usage: locmetry workspace [OPTIONS]

Options:
  -h, --help
          Print help (see a summary with '-h')

Selection and policy:
      --root <DIR>
          Member root; repeatable. Its folder name prefixes its paths in the report

      --workspace-file <FILE>
          Read the member roots from a TOML workspace file instead; its roots are relative to the
          file (see --schema workspace)

      --no-user-config
          Ignore user defaults in ~/.config/locmetry/config.toml

Output and content:
      --output <FILE>
          Write only to FILE, outside the scan root; - selects stdout. Defaults to full JSON;
          --format overrides it. File extensions do not select a format. Add --print table to also
          print the overview from the same run

      --print <FORMAT>
          With --output FILE, also print the same measurement to stdout in FORMAT, such as table for
          an overview or headline for compact JSON

          Possible values:
          - table:    Human-readable tables and evidence
          - json:     Complete JSON document; measurement reports can be compared later
          - headline: Compact JSON for agents and scripts; not a comparison-ready measurement
          - markdown: Markdown tables and evidence to save or paste into reviews

      --files
          Include per-file records in full JSON, saved or printed; headline omits them

      --format <FORMAT>
          Result format [default: json with --output; otherwise table on a terminal, headline in
          pipes]

          Possible values:
          - table:    Human-readable tables and evidence
          - json:     Complete JSON document; measurement reports can be compared later
          - headline: Compact JSON for agents and scripts; not a comparison-ready measurement
          - markdown: Markdown tables and evidence to save or paste into reviews

{
  "member": "web",
  "path": "web/src/orders/Order.test.ts",
  "result": {
    "state": "counted",
    "rule": "builtin:js-test-file"
  }
}
```
