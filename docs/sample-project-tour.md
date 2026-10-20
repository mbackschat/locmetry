# A Tour of the Sample Project

Follow the checked-in [Acme Orders workspace](../examples/cli-workspace/README.md) from measurement through classification, folder exploration and saved reports. This is the same source tree used by the [README tables](../README.md#what-you-see) and the [recorded command guides](cli/README.md). Its files are tiny measurement examples; nothing needs building or installing.

## Prepare a working copy

From the Locmetry repository root, with `locmetry` on `PATH`, copy the fixture into a fresh temporary directory. All following commands run there. Reports stay outside the measured roots, and `--no-user-config` isolates the examples from personal defaults.

```sh
sample_directory=$(mktemp -d)
cp -R examples/cli-workspace/. "$sample_directory/"
cd "$sample_directory"
```

The tree contains a Go service and a TypeScript/TSX web client:

```text
acme-orders/
├── .locmetryrc
├── service/
│   ├── .locmetryrc, go.mod, .ignore
│   ├── src/main.go
│   ├── tests/main_test.go
│   ├── scripts/check.py
│   ├── experiments/probe.py
│   ├── third_party/sdk.py
│   ├── .hidden.py
│   └── ignored.py
└── web/
    ├── .locmetryrc, package.json
    ├── src/orders/{Order.ts,Order.test.ts,OrderView.tsx}
    ├── src/index.html
    ├── data/status.json
    └── docs/{architecture.md,wireframe.asset}
```

The combined-root policy excludes `service/third_party/**`, disables hidden files and classifies `service/scripts/**` as production tooling. The service's own policy uses member-relative patterns for workspace measurement. Module descriptors establish conventional layouts; Locmetry never runs their build tools. The experiment has no ownership decision yet. These policies are visible in the fixture's configuration files.

## Read the first measurement

```sh
locmetry --root acme-orders --no-user-config --no-color --no-bars --format table
```

The source total is 25 code lines: 17 production, 7 test and 1 other. Data and documentation stay separate. The HTML file contributes four HTML lines and one embedded JavaScript line; the JavaScript row does not add a physical file. `Test/prod` is a volume ratio, not test coverage. The [recorded root guide](cli/root.md) includes the captured output.

## Find where the code resides

```sh
locmetry --root acme-orders --no-user-config --no-color --no-bars --format table --folder-rows 12
locmetry --root acme-orders --subfolder web/src --no-user-config --no-color --no-bars --format table --show-folders
```

Adaptive rows cover the source once, separating direct files from expanded descendants. `--subfolder` retains the combined root's policy and module context; changing the root would change that context. [Folder reporting](guides/reports.md#look-inside-large-folders) owns the expansion rules.

## Trace a classification decision

```sh
locmetry explain web/src/orders/Order.test.ts --root acme-orders --no-user-config --format json
```

The deciding rule is `builtin:js-test-file`, even though the file sits inside production source. The [recorded explanation guide](cli/explain.md) also traces an explicit tooling rule, exclusion, ignored file, hidden file, unresolved source and unrecognized content.

## Review what remains unresolved

```sh
locmetry --root acme-orders --no-user-config --no-color --no-bars --format table --verbose
```

`service/experiments/probe.py` is counted as Other/unclaimed. `web/docs/wireframe.asset` is selected but unrecognized, so it has no invented zero count. Neither condition alone makes the successfully measured counts partial. Review is about local intent: assigning a category can resolve classification without establishing that the choice is correct.

## Save the evidence and ask a narrower question

With optional external `jq`:

```sh
locmetry --root acme-orders --no-user-config --files --output acme-orders-report.json
jq '[.breakdown[] | select(.kind == "source" and .purpose == "test")] | group_by(.language) | map({language: .[0].language, code: (map(.code) | add)})' acme-orders-report.json
```

The result contains Go 4 and TypeScript 3. Full reports contain additive breakdown rows even without `--files`; optional file detail supports path-level questions. The [DuckDB cookbook](duckdb-cookbook.md) and [pandas/Polars cookbook](dataframe-cookbook.md) use this tool-neutral structure for broader analysis.

## Share a report and follow changes

```sh
locmetry --root acme-orders --no-user-config --format markdown --markdown-depth 3 --output acme-orders-report.md
printf 'export const discount = 1;\n' >> acme-orders/web/src/orders/Order.ts
locmetry --root acme-orders --no-user-config --output acme-orders-after.json
locmetry compare acme-orders-report.json acme-orders-after.json --format json
```

Source code increases by one line, from 25 to 26. This is net change between measured states, not Git churn. The [recorded comparison guide](cli/compare.md) shows tables, exports, member selection and an explicit workspace membership change.

## Measure the members together

```sh
locmetry workspace --workspace-file workspace.toml --no-user-config --no-color --no-bars --format table
```

The workspace file names `service` and `web`. Their own policies give the same source total as the combined root at the same state; paths and provenance reflect separate members. The [recorded workspace guide](cli/workspace.md) explains the distinction and demonstrates workspace report options. Delete your temporary copy when finished; the checked-in source remains unchanged.
