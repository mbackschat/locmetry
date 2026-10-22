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
 HTML            2      5     5 │        0       0 │     5     0      0 │      0.00
 JavaScript      0      1     1 │        0       0 │     1     0      0 │      0.00
 Makefile        1      2     2 │        0       0 │     2     0      0 │      0.00
 Python          2      5     3 │        2       0 │     2     0      1 │      0.00
 TypeScript      4     13    10 │        3       0 │     6     3      1 │      0.50
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 Total          11     37    29 │        6       2 │    20     7      2 │      0.35
 Lines = Code + Comments + Blanks · Production + Test + Other = Code · Test/prod = Test / Production (Code)
 Tests written in another language than the code they test skew per-language Test/prod.
 Folded: TSX → TypeScript.

 ──────────────────────────────────────────────────────────────
 Member   Lines  Code │ Prod.  Test  Other │ Test/prod │  Share
 ─────────────────────┼────────────────────┼───────────┼───────
 service     18    13 │     8     4      1 │      0.50 │  44.8%
 web         19    16 │    12     3      1 │      0.25 │  55.2%
 ─────────────────────┼────────────────────┼───────────┼───────
 Total       37    29 │    20     7      2 │      0.35 │ 100.0%
 Production + Test + Other = Code · Test/prod = Test / Production (Code) · Share = Code / workspace Code

 ─────────────────────────────────────────────
 Category                   Files  Lines  Code
 ─────────────────────────────────────────────
 other:generated                1      2     1
 other:unclaimed                1      2     1
 production:build-tooling       1      2     2
 production:implementation      5     19    16
 production:tooling             1      3     2
 test:test                      2      9     7
 Rows of one purpose add up to that purpose's Code, and all rows add up to the total Code.

 Not source code: 2 data files (2 lines), 1 documentation file (3 lines). '--files --output -' lists each file's kind.
 Production from build modules: 4 code lines in 1 'go' module, 12 code lines in 1 'npm' module.
 Largest files counted as Production only by a build layout. If one is generated or vendored, exclude or reclassify it
in '.locmetryrc':
   web/src/index.html              5 code lines  web (npm)
   service/src/main.go             4 code lines  service (go)
   web/src/orders/Order.ts         3 code lines  web (npm)
   web/src/orders/OrderView.tsx    3 code lines  web (npm)
   web/src/orders/receipt.acmetpl  1 code line   web (npm)
 Unclaimed source: 1 code line in 1 file, here:
   service/experiments  1 code line  1 file
 To inspect or refine purpose classification, use '--guide configure'. '--files --output -' shows each file's rule.
 Unclaimed source by member: 'service' has 1 unclaimed file (1 code line).
 Unrecognized files: 2 (asset: 1, mod: 1). Their lines are unknown, not zero.

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
    "files": 11,
    "code": 29,
    "comments": 6,
    "blanks": 2,
    "lines": 37
  },
  "members": [
    "service",
    "web"
  ],
  "source_paths": [
    "service/Makefile",
    "service/experiments/probe.py",
    "service/scripts/check.py",
    "service/src/main.go",
    "service/tests/main_test.go",
    "web/src/api/client.ts",
    "web/src/index.html",
    "web/src/orders/Order.test.ts",
    "web/src/orders/Order.ts",
    "web/src/orders/OrderView.tsx",
    "web/src/orders/receipt.acmetpl"
  ]
}
```

## Select purposes and show shares

`--table-purpose production,test` restricts table counts. `--show-bars` charts purpose shares; `--sort name` makes the repository order explicit; `--split-languages` keeps TSX apart from TypeScript. Shared presentation flags have the same meaning as in root measurement.

```bash
source scripts/cli-demo-env.sh
locmetry workspace --workspace-file workspace.toml --no-user-config --no-color --format table \
  --table-purpose production,test --show-bars --sort name --split-languages \
  | sed '/^Root:/d; /^Roots:/d; /^Measured at:/d' \
  | demo_wrap
```

```output
 ───────────────────────────────────────────────────────────────────────────────────────────────────
 Language    Files  Lines  Code │ Comments  Blanks │ Prod.  Test │ Test/prod │  Share  Code share
 ───────────────────────────────┼──────────────────┼─────────────┼───────────┼──────────────────────
 Go              2     11     8 │        1       2 │     4     4 │      1.00 │  29.6%  ███████▓▓▓▓▓▓
 HTML            2      5     5 │        0       0 │     5     0 │      0.00 │  18.5%  ████████
 JavaScript      0      1     1 │        0       0 │     1     0 │      0.00 │   3.7%  ██
 Makefile        1      2     2 │        0       0 │     2     0 │      0.00 │   7.4%  ███
 Python          1      3     2 │        1       0 │     2     0 │      0.00 │   7.4%  ███
 TSX             1      3     3 │        0       0 │     3     0 │      0.00 │  11.1%  █████
 TypeScript      2      8     6 │        2       0 │     3     3 │      1.00 │  22.2%  █████▓▓▓▓▓
 ───────────────────────────────┼──────────────────┼─────────────┼───────────┼──────────────────────
 Total           9     33    27 │        4       2 │    20     7 │      0.35 │ 100.0%
 Lines = Code + Comments + Blanks · Production + Test = Code · Test/prod = Test / Production (Code)
 Tests written in another language than the code they test skew per-language Test/prod.
 Not shown (purpose): Other (2 code lines).
 Share = Code / total Code. Bars: █ Production  ▓ Test. The largest row fills the bar.

 ─────────────────────────────────────────────────────────────────────────────────────────────────
 Member   Lines  Code │ Prod.  Test │ Test/prod │  Share  Code share
 ─────────────────────┼─────────────┼───────────┼─────────────────────────────────────────────────
 service     16    12 │     8     4 │      0.50 │  44.4%  █████████████████████▓▓▓▓▓▓▓▓▓▓▓
 web         17    15 │    12     3 │      0.25 │  55.6%  ████████████████████████████████▓▓▓▓▓▓▓▓
 ─────────────────────┼─────────────┼───────────┼─────────────────────────────────────────────────
 Total       33    27 │    20     7 │      0.35 │ 100.0%
 Production + Test = Code · Test/prod = Test / Production (Code) · Share = Code / workspace Code
 Bars: █ Production  ▓ Test. The largest row fills the bar.

```

`--table-language go` counts only Go source in every table, so the member rows show where the Go code lives; the note under the language table names what the selection leaves out.

```bash
source scripts/cli-demo-env.sh
locmetry workspace --workspace-file workspace.toml --no-user-config --no-color --format table \
  --table-language go \
  | sed '/^Root:/d; /^Roots:/d; /^Measured at:/d' \
  | demo_wrap
```

```output
 ────────────────────────────────────────────────────────────────────────────────
 Language  Files  Lines  Code │ Comments  Blanks │ Prod.  Test  Other │ Test/prod
 ─────────────────────────────┼──────────────────┼────────────────────┼──────────
 Go            2     11     8 │        1       2 │     4     4      0 │      1.00
 ─────────────────────────────┼──────────────────┼────────────────────┼──────────
 Total         2     11     8 │        1       2 │     4     4      0 │      1.00
 Lines = Code + Comments + Blanks · Production + Test + Other = Code · Test/prod = Test / Production (Code)
 Tests written in another language than the code they test skew per-language Test/prod.
 Not shown (languages): TypeScript, HTML, Python, Makefile and JavaScript (21 code lines).

 ──────────────────────────────────────────────────────────────
 Member   Lines  Code │ Prod.  Test  Other │ Test/prod │  Share
 ─────────────────────┼────────────────────┼───────────┼───────
 service     11     8 │     4     4      0 │      1.00 │ 100.0%
 web          0     0 │     0     0      0 │         - │   0.0%
 ─────────────────────┼────────────────────┼───────────┼───────
 Total       11     8 │     4     4      0 │      1.00 │ 100.0%
 Production + Test + Other = Code · Test/prod = Test / Production (Code) · Share = Code / workspace Code

```

`--table-category '*:generated'` counts only code in a `generated` category, of any purpose, so the member rows show which member holds it.

```bash
source scripts/cli-demo-env.sh
locmetry workspace --workspace-file workspace.toml --no-user-config --no-color --format table \
  --table-category '*:generated' \
  | sed '/^Root:/d; /^Roots:/d; /^Measured at:/d' \
  | demo_wrap
```

```output
 ──────────────────────────────────────────────────────────────────────────────────
 Language    Files  Lines  Code │ Comments  Blanks │ Prod.  Test  Other │ Test/prod
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 TypeScript      1      2     1 │        1       0 │     0     0      1 │         -
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 Total           1      2     1 │        1       0 │     0     0      1 │         -
 Lines = Code + Comments + Blanks · Production + Test + Other = Code · Test/prod = Test / Production (Code)
 Tests written in another language than the code they test skew per-language Test/prod.
 Not shown (categories): production:implementation, test:test, production:build-tooling, production:tooling and
other:unclaimed (28 code lines).

 ──────────────────────────────────────────────────────────────
 Member   Lines  Code │ Prod.  Test  Other │ Test/prod │  Share
 ─────────────────────┼────────────────────┼───────────┼───────
 web          2     1 │     0     0      1 │         - │ 100.0%
 service      0     0 │     0     0      0 │         - │   0.0%
 ─────────────────────┼────────────────────┼───────────┼───────
 Total        2     1 │     0     0      1 │         - │ 100.0%
 Production + Test + Other = Code · Test/prod = Test / Production (Code) · Share = Code / workspace Code

```

`--tables-only` keeps the language and member tables and drops their legends and the roots/time footer, so the rows need no filter.

```bash
source scripts/cli-demo-env.sh
locmetry workspace --workspace-file workspace.toml --no-user-config --no-color --no-bars \
  --format table --tables-only
```

```output
 ──────────────────────────────────────────────────────────────────────────────────
 Language    Files  Lines  Code │ Comments  Blanks │ Prod.  Test  Other │ Test/prod
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 TypeScript      4     13    10 │        3       0 │     6     3      1 │      0.50
 Go              2     11     8 │        1       2 │     4     4      0 │      1.00
 HTML            2      5     5 │        0       0 │     5     0      0 │      0.00
 Python          2      5     3 │        2       0 │     2     0      1 │      0.00
 Makefile        1      2     2 │        0       0 │     2     0      0 │      0.00
 JavaScript      0      1     1 │        0       0 │     1     0      0 │      0.00
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 Total          11     37    29 │        6       2 │    20     7      2 │      0.35

 ──────────────────────────────────────────────────────────────
 Member   Lines  Code │ Prod.  Test  Other │ Test/prod │  Share
 ─────────────────────┼────────────────────┼───────────┼───────
 web         19    16 │    12     3      1 │      0.25 │  55.2%
 service     18    13 │     8     4      1 │      0.50 │  44.8%
 ─────────────────────┼────────────────────┼───────────┼───────
 Total       37    29 │    20     7      2 │      0.35 │ 100.0%
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
    "files": 11,
    "code": 29,
    "comments": 6,
    "blanks": 2,
    "lines": 37
  }
}
{
  "service": {
    "files": 5,
    "code": 13,
    "comments": 3,
    "blanks": 2,
    "lines": 18
  },
  "web": {
    "files": 6,
    "code": 16,
    "comments": 3,
    "blanks": 0,
    "lines": 19
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
| production | 7 | 24 | 20 | 3 | 1 |
| test | 2 | 9 | 7 | 1 | 1 |
| other | 2 | 4 | 2 | 2 | 0 |

## Members

| Member | Lines | **Code** | Production | Test | Other | Test/prod | Share |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| service | 18 | 13 | 8 | 4 | 1 | 0.50 | 44.8% |
| web | 19 | 16 | 12 | 3 | 1 | 0.25 | 55.2% |
| **Total** | **37** | **29** | **20** | **7** | **2** | **0.35** | **100.0%** |

*Production + Test + Other = Code · Test/prod = Test / Production (Code) · Share = Code / workspace Code*

## Source categories by member
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
Measure several repositories or one project's folders as one virtual monorepo.

Defaults to a table on a terminal, a JSON headline in pipes. --output FILE saves only to that file,
defaulting to full JSON; --format overrides the format.

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

## Measure folders of one project

When the members are components of one repository, a workspace file can name them as folders of that project instead of roots. [folders.toml](../../examples/cli-workspace/folders.toml) names `service` and `web` under `acme-orders`. Each folder is measured as `--subfolder` measures it, under the combined root's single `.locmetryrc`, so the member configurations are not used. The totals equal the root-based workspace above; the member records name the project as root and the folder as subfolder.

```bash
source scripts/cli-demo-env.sh
locmetry workspace --workspace-file folders.toml --no-user-config --format json \
  | jq "{source: .summary.source.code,
         members: [.metadata.members[]
           | {name, subfolder: .scope.subfolder, config: (.config_path | split(\"/\") | .[-2:] | join(\"/\"))}]}"

```

```output
{
  "source": 29,
  "members": [
    {
      "name": "service",
      "subfolder": "service",
      "config": "acme-orders/.locmetryrc"
    },
    {
      "name": "web",
      "subfolder": "web",
      "config": "acme-orders/.locmetryrc"
    }
  ]
}
```
