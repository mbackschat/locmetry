# Compare Saved Measurements

*2026-10-20T12:56:37Z by Showboat 0.6.1*
<!-- showboat-id: 194707e9-1e05-4d62-a642-d42abb57625e -->

Run each Bash block from the Locmetry repository root. [Setup and replay instructions](README.md) describe the fresh temporary copy created by `source scripts/cli-demo-env.sh`, external tools and output projections. The [sample tour](../sample-project-tour.md) introduces the shared source tree.

## Capture and explain a known change

Each example starts fresh. Here one added source line changes code from 25 to 26. `compare` reads saved full measurements; optional per-file records are unnecessary. The table excerpt omits the variable input-date footer.

```bash
source scripts/cli-demo-env.sh
locmetry --root acme-orders --no-user-config --output before.json
printf 'export const discount = 1;\n' >> acme-orders/web/src/orders/Order.ts
locmetry --root acme-orders --no-user-config --output after.json
locmetry compare before.json after.json --no-color --format table \
  | sed -e '/^Before measured at:/d; /^After measured at:/d' -e 's/[[:blank:]]*$//' \
  | demo_wrap
```

```output
Net change: acme-orders → acme-orders

 ──────────────────────────────────────────────────────────────────────────────────────
                        Lines         │         Code          │         Change
 Kind           before  after  change │ before  after  change │ Comments  Blanks  Files
 ─────────────────────────────────────┼───────────────────────┼────────────────────────
 Source             32     33      +1 │     25     26      +1 │        0       0      0
 Data                2      2       0 │      2      2       0 │        0       0      0
 Documentation       3      3       0 │      0      0       0 │        0       0      0
 ─────────────────────────────────────┼───────────────────────┼────────────────────────
 All counted        37     38      +1 │     27     28      +1 │        0       0      0

 ───────────────────────────────────────────────────────────
                       Lines         │         Code
 Purpose       before  after  change │ before  after  change
 ────────────────────────────────────┼──────────────────────
 Production        21     22      +1 │     17     18      +1
 ────────────────────────────────────┼──────────────────────
 Source total      32     33      +1 │     25     26      +1
 Test/prod (Code): 0.41 → 0.39 (before → after)

 ───────────────────────────────────────────────────────────
                       Lines         │         Code
 Language      before  after  change │ before  after  change
 ────────────────────────────────────┼──────────────────────
 TypeScript         8      9      +1 │      6      7      +1
 ────────────────────────────────────┼──────────────────────
 Source total      32     33      +1 │     25     26      +1

 ───────────────────────────────────────────────────────────
                       Lines         │         Code
 Folder        before  after  change │ before  after  change
 ────────────────────────────────────┼──────────────────────
 web (both)        16     17      +1 │     14     15      +1
 ────────────────────────────────────┼──────────────────────
 Source total      32     33      +1 │     25     26      +1
 Folders are recursive; do not sum parents and children.

▸ Measurement notes

- Net change = after − before, from the two saved reports.
- Unreadable files count as 0; a folder seen in only one report may be newly visible, not newly created.
- Unrecognized files: same 2 paths in both reports; not counted on either side.
- Folder totals containing these files cover counted files only; per-folder detail is in JSON findings[].folder.

No recorded differences in measurement settings. Reports do not fingerprint source, ignore files or executable builds;
this does not prove source-only growth.

```

## Save the comparison

`--output` silently saves full JSON by default. `--format markdown` saves a readable document; `--output -` sends the selected format to stdout. Queries and excerpts select stable fields. A headline is a compact comparison projection.

```bash
source scripts/cli-demo-env.sh
locmetry --root acme-orders --no-user-config --output before.json
printf 'export const discount = 1;\n' >> acme-orders/web/src/orders/Order.ts
locmetry --root acme-orders --no-user-config --output after.json
locmetry compare before.json after.json --output change.json
jq '{document_type, schema_version, source: .summary.source}' change.json
locmetry compare before.json after.json --format markdown --output change.md
sed -n '/^## Purpose/,/^## /p' change.md | demo_wrap
locmetry compare before.json after.json --format headline --output - \
  | jq '{document_type, source: .summary.source}'
```

```output
{
  "document_type": "comparison",
  "schema_version": "2",
  "source": {
    "before": {
      "files": 8,
      "code": 25,
      "comments": 5,
      "blanks": 2,
      "lines": 32
    },
    "after": {
      "files": 8,
      "code": 26,
      "comments": 5,
      "blanks": 2,
      "lines": 33
    },
    "delta": {
      "files": 0,
      "code": 1,
      "comments": 0,
      "blanks": 0,
      "lines": 1
    }
  }
}
## Purpose

| Purpose | Lines before | Lines after | Lines change | Code before | Code after | Code change |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Production | 21 | 22 | +1 | 17 | 18 | +1 |
| **Source total** | **32** | **33** | **+1** | **25** | **26** | **+1** |

*Test/prod (Code): 0.41 → 0.39 (before → after)*

## Languages
{
  "document_type": "comparison",
  "source": {
    "before": {
      "files": 8,
      "code": 25,
      "comments": 5,
      "blanks": 2,
      "lines": 32
    },
    "after": {
      "files": 8,
      "code": 26,
      "comments": 5,
      "blanks": 2,
      "lines": 33
    },
    "delta": {
      "files": 0,
      "code": 1,
      "comments": 0,
      "blanks": 0,
      "lines": 1
    }
  }
}
```

## Focus on a workspace member

`--member web` selects the exact member name from either input. Counts and detail cover that member; input provenance and partial status still cover the workspace reports. Names identify recorded members, not Git revisions.

```bash
source scripts/cli-demo-env.sh
locmetry workspace --workspace-file workspace.toml --no-user-config --output before.json
printf 'export const discount = 1;\n' >> acme-orders/web/src/orders/Order.ts
locmetry workspace --workspace-file workspace.toml --no-user-config --output after.json
locmetry compare before.json after.json --member web --no-color --format table \
  | sed -e '/^Before measured at:/d; /^After measured at:/d' -e 's/[[:blank:]]*$//' \
  | demo_wrap
```

```output
Net change: service + web → service + web
Member: web (both); counts and detail cover this member only.
Input metadata, measurement notes, report-level findings and partial status cover the whole workspace inputs.

 ──────────────────────────────────────────────────────────────────────────────────────
                        Lines         │         Code          │         Change
 Kind           before  after  change │ before  after  change │ Comments  Blanks  Files
 ─────────────────────────────────────┼───────────────────────┼────────────────────────
 Source             16     17      +1 │     14     15      +1 │        0       0      0
 Data                2      2       0 │      2      2       0 │        0       0      0
 Documentation       3      3       0 │      0      0       0 │        0       0      0
 ─────────────────────────────────────┼───────────────────────┼────────────────────────
 All counted        21     22      +1 │     16     17      +1 │        0       0      0

 ───────────────────────────────────────────────────────────
                       Lines         │         Code
 Purpose       before  after  change │ before  after  change
 ────────────────────────────────────┼──────────────────────
 Production        12     13      +1 │     11     12      +1
 ────────────────────────────────────┼──────────────────────
 Source total      16     17      +1 │     14     15      +1
 Test/prod (Code): 0.27 → 0.25 (before → after)

 ───────────────────────────────────────────────────────────
                       Lines         │         Code
 Language      before  after  change │ before  after  change
 ────────────────────────────────────┼──────────────────────
 TypeScript         8      9      +1 │      6      7      +1
 ────────────────────────────────────┼──────────────────────
 Source total      16     17      +1 │     14     15      +1

Member source lines: matched +1, entering +0, leaving -0; total +1.
Member all-counted lines: matched +1, entering +0, leaving -0; total +1.

Member source code: matched +1, entering +0, leaving -0; total +1.
Member all-counted code: matched +1, entering +0, leaving -0; total +1.

 ───────────────────────────────────────────────────────────
                       Lines         │         Code
 Folder        before  after  change │ before  after  change
 ────────────────────────────────────┼──────────────────────
 src (both)        16     17      +1 │     14     15      +1
 ────────────────────────────────────┼──────────────────────
 Source total      16     17      +1 │     14     15      +1
 Folders are recursive; do not sum parents and children.

▸ Measurement notes

- Net change = after − before, from the two saved reports.
- Unreadable files count as 0; a folder seen in only one report may be newly visible, not newly created.
- Unrecognized files: same 2 paths in both reports; not counted on either side.
- Folder totals containing these files cover counted files only; per-folder detail is in JSON findings[].folder.

No recorded differences in measurement settings. Reports do not fingerprint source, ignore files or executable builds;
this does not prove source-only growth.

```

## Distinguish membership from deletion

The second capture deliberately selects only the service. Web leaves the measured workspace; its files still exist. Full comparison JSON retains the member presence and count evidence.

```bash
source scripts/cli-demo-env.sh
locmetry workspace --workspace-file workspace.toml --no-user-config --output before.json
locmetry workspace --root acme-orders/service --no-user-config --output after.json
locmetry compare before.json after.json --format json \
  | jq '{
      source: .summary.source,
      members: (.members | with_entries(.value = {
        presence: .value.presence,
        source: .value.source
      }))
    }'
test -f acme-orders/web/src/orders/Order.ts
printf 'Web source still exists.\n'
```

```output
{
  "source": {
    "before": {
      "files": 8,
      "code": 25,
      "comments": 5,
      "blanks": 2,
      "lines": 32
    },
    "after": {
      "files": 4,
      "code": 11,
      "comments": 3,
      "blanks": 2,
      "lines": 16
    },
    "delta": {
      "files": -4,
      "code": -14,
      "comments": -2,
      "blanks": 0,
      "lines": -16
    }
  },
  "members": {
    "service": {
      "presence": "both",
      "source": {
        "before": {
          "files": 4,
          "code": 11,
          "comments": 3,
          "blanks": 2,
          "lines": 16
        },
        "after": {
          "files": 4,
          "code": 11,
          "comments": 3,
          "blanks": 2,
          "lines": 16
        },
        "delta": {
          "files": 0,
          "code": 0,
          "comments": 0,
          "blanks": 0,
          "lines": 0
        }
      }
    },
    "web": {
      "presence": "before_only",
      "source": {
        "before": {
          "files": 4,
          "code": 14,
          "comments": 2,
          "blanks": 0,
          "lines": 16
        },
        "after": {
          "files": 0,
          "code": 0,
          "comments": 0,
          "blanks": 0,
          "lines": 0
        },
        "delta": {
          "files": -4,
          "code": -14,
          "comments": -2,
          "blanks": 0,
          "lines": -16
        }
      }
    }
  }
}
Web source still exists.
```

## Respect the input contract

The help excerpt includes input arguments, member selection and output options, stopping before presentation options.

A headline cannot replace a full baseline. This example checks exit 1 and prints the actual diagnostic. Capture earlier states before editing; a new measurement cannot recover the past. Exit 2 instead indicates partial measured input. The comparison guide owns the full interpretation rules.

```bash
source scripts/cli-demo-env.sh
locmetry --root acme-orders --no-user-config --format headline --output headline.json
locmetry --root acme-orders --no-user-config --output full.json
status=0
locmetry compare headline.json full.json --no-color --format json 2> diagnostic.txt || \
  status=$?
test "$status" -eq 1
cat diagnostic.txt | demo_wrap
locmetry compare --no-color --help | sed '/^Presentation:/,$d'
```

```output
locmetry: Cannot compare before report headline.json: Full measurement report required; missing review, breakdown,
metadata.kinds, metadata.modules, metadata.coverage.unrecognized, metadata.config. Capture a full measurement with
--output FILE or --format json. A headline cannot reconstruct the missing baseline; rerunning measures the current tree
Compare two saved full reports: net change is after minus before. Defaults to a table on a terminal,
a JSON headline in pipes. --output FILE saves only to that file, defaulting to full JSON; --format
overrides the format

Usage: locmetry compare [OPTIONS] <BEFORE> <AFTER>

Arguments:
  <BEFORE>
          Full JSON measurement before the change

  <AFTER>
          Full JSON measurement after the change

Options:
  -h, --help
          Print help (see a summary with '-h')

Selection and policy:
      --member <NAME>
          Focus on one exact workspace member name; accepts presence on either side

Output and content:
      --format <FORMAT>
          Result format [default: json with --output; otherwise table on a terminal, headline in
          pipes]

          Possible values:
          - table:    Human-readable tables and evidence
          - json:     Complete JSON document; measurement reports can be compared later
          - headline: Compact JSON for agents and scripts; not a comparison-ready measurement
          - markdown: Markdown tables and evidence to save or paste into reviews

      --output <FILE>
          Write only to FILE; - selects stdout. Defaults to full JSON; --format overrides it

```
