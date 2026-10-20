# Analyze Measurements with DuckDB

Use SQL to explore saved Locmetry measurements: composition, portfolio totals, recorded changes and folder ownership. DuckDB is an optional analysis tool; Locmetry itself has no database or Python dependency. The [pandas and Polars cookbook](dataframe-cookbook.md) uses the same reports in dataframes, and the [recorded demo](analytics-demo.md) shows the CLI and SQL running together.

This recipe targets full **measurement schema 3** exports and was verified with DuckDB 1.5.6. Install the [DuckDB CLI](https://duckdb.org/docs/current/installation/) separately; on macOS, use `brew install duckdb`. Copy [duckdb.sql](../examples/analytics/duckdb.sql) into your analysis directory. The SQL is ordinary DuckDB SQL and also works through its Python API. It reads trusted Locmetry exports and checks their envelope and required input sections; it is not a replacement for complete schema and semantic validation.

## Capture a small portfolio

Run this block in a fresh working directory with the current `locmetry` on `PATH`. It creates two synthetic projects and four full reports outside their roots. The shell stops on failure or partial measurement; each successful capture has a caller-assigned repository identity and sequence. The second API report deliberately omits optional file detail.

<!-- capture -->
```sh
set -eu
mkdir -p api/src api/tests web/src web/tests
cat > api/.locmetryrc <<'TOML'
version = 1
[[rules]]
id = "maintained"
match = ["src/**"]
purpose = "production"
category = "application"
TOML
cp api/.locmetryrc web/.locmetryrc
printf 'print(1)\nprint(2)\n' > api/src/main.py
printf 'assert True\n' > api/tests/test_main.py
printf 'assert True\n' > web/tests/test_main.py
locmetry api --no-user-config --files --output api-1.json
locmetry web --no-user-config --files --output web-1.json
printf 'print(3)\n' >> api/src/main.py
printf 'assert True\n' >> api/tests/test_main.py
printf 'print(1)\nprint(2)\n' > web/src/main.py
printf 'assert True\n' >> web/tests/test_main.py
locmetry api --no-user-config --output api-2.json
locmetry web --no-user-config --files --output web-2.json
cat > snapshots.csv <<'CSV'
snapshot_id,repo_id,sequence,path
api-1,api,1,api-1.json
api-2,api,2,api-2.json
web-1,web,1,web-1.json
web-2,web,2,web-2.json
CSV
cat > owners.csv <<'CSV'
repo_id,prefix,owner
api,src,api-team
api,tests,quality
web,src,web-team
web,tests,quality
CSV
```

The API grows from 3 to 5 source-code lines; Web grows from 1 to 4. Their second captures contain 5 production and 4 test lines together. These are deliberately small counts, independently checked by the [cookbook tests](../scripts/test_analytics_cookbooks.py).

For your own reports, keep the manifest columns and replace its rows. Paths are relative to the analysis process's working directory, including paths in a manifest loaded from another directory. Each path must name one JSON document, not a glob. Snapshot IDs, paths and `(repo_id, sequence)` pairs must be unique. A repository ID is your assertion of identity; neither the report's display name nor its absolute root establishes it. Sequence establishes order even when measurement timestamps tie. Keep whole-project and subfolder captures in separate series. A workspace can be analyzed as one measured collection, but do not add it to standalone captures of the same members.

## Load typed views

Start `duckdb` from the analysis directory and run:

```sql
.read duckdb.sql
```

The loader creates these temporary views; the manifest and raw reports are materialized for the session, so changing files does not silently change an existing analysis. Start a new session or rerun the loader to refresh them.

| View | One row represents | Useful fields |
| --- | --- | --- |
| `snapshots` | One measurement, even when empty | `snapshot_id`, `repo_id`, `sequence`, `measured_at_unix_seconds`, `source_code`, `production_code`, `test_code`, `partial`, `has_files`, `subfolder`, original `doc` |
| `contributions` | One additive breakdown row | Snapshot identity, `folder`, `kind`, `purpose`, `category`, `language`, and integer `files/code/comments/blanks/lines` |
| `physical_files` | One counted physical file, when requested | `snapshot_id`, `path`, `kind`, `purpose`, `language`, `code` |

The explicit types come from the [report contract](PRODUCT.md#report-content); empty arrays cannot supply them through inference. Counts use unsigned integers and net-change arithmetic widens to signed integers before subtraction. Empty snapshots remain in `snapshots`; absent file detail remains distinguishable from a recorded empty file list. The [JSON reader](https://duckdb.org/docs/current/data/json/loading_json) defaults to a 16 MiB limit per JSON object; for larger full reports, increase `maximum_object_size` in the loader's `read_json_objects` call deliberately.

## Composition and concentration

Sum additive breakdown rows for one snapshot. Keep embedded-language rows even when `files = 0`.

<!-- composition -->
```sql
SELECT language, purpose, sum(code) AS code
FROM contributions
WHERE snapshot_id = 'api-1' AND kind = 'source'
GROUP BY language, purpose
ORDER BY code DESC, language, purpose;
```

Expected: Python production 2, Python test 1. File counts from `contributions` are additive; summary language file counts instead describe overlapping participation. See [report interpretation](guides/reports.md#interpret-the-numbers).

<!-- folders -->
```sql
SELECT folder, sum(code) AS code
FROM contributions
WHERE snapshot_id = 'api-1' AND kind = 'source'
GROUP BY folder
ORDER BY code DESC, folder;
```

Expected: `src` 2 and `tests` 1. These are direct-parent counts. For a recursive subtree use `folder = 'src' OR starts_with(folder, 'src/')`; a prefix test without the slash would also select `src-old`. Do not sum recursive parent and child totals together.

## Portfolio totals and ratios

Select one explicit capture per repository. Here, both repositories use sequence 2; real portfolios may use different sequences or an explicit list of snapshot IDs. Partial captures are excluded from this complete-observation calculation; inspect their presence before interpreting the portfolio as complete.

<!-- portfolio -->
```sql
SELECT sum(production_code) AS production_code, sum(test_code) AS test_code,
       sum(test_code) / nullif(sum(production_code), 0) AS test_prod_ratio
FROM snapshots
WHERE repo_id IN ('api', 'web') AND sequence = 2 AND NOT partial;
```

Expected: 5 production, 4 test, ratio 0.8. Divide summed counts; do not average repository ratios. Zero production produces NULL, not zero. Test/production is a code-volume ratio, not test coverage or a quality score.

## Recorded net change

The example's policy and selection are controlled. For real histories, inspect the recorded configuration, scope, counter versions, coverage and workspace membership; [native comparison](guides/compare.md#comparable-scope-and-evidence) supplies findings for a pair. The SQL below orders the selected series and suppresses a delta touching a partial capture. It does not skip partial captures and connect their neighbors, nor does it decide that two policies are comparable.

<!-- trend -->
```sql
WITH ordered AS (
    SELECT *, lag(source_code::HUGEINT) OVER series AS previous_code,
              lag(partial) OVER series AS previous_partial
    FROM snapshots
    WHERE repo_id IN ('api', 'web')
    WINDOW series AS (PARTITION BY repo_id ORDER BY sequence)
)
SELECT repo_id, sequence, source_code,
       CASE WHEN NOT partial AND NOT previous_partial
            THEN source_code::HUGEINT - previous_code END AS delta
FROM ordered
ORDER BY repo_id, sequence;
```

Expected: API `NULL, +2`; Web `NULL, +3`. These are differences between recorded measurements, not churn, source-edit detection or productivity. Changed classification can transfer counts between purposes without changing the source total. A partial report's zero means zero recorded code, not proof that the project is empty.

## Join folder ownership

Ownership is external data, keyed by caller repository identity and a root-relative folder prefix. Use workspace member-prefixed paths when mapping a workspace. One prefix has one owner; nested prefixes choose the longest match, so an aggregate row contributes only once. `.` matches every folder. This mapping describes the chosen ownership policy, not historical ownership unless you supply capture-specific mappings.

<!-- owners -->
```sql
CREATE TEMP TABLE ownership (
    repo_id VARCHAR, prefix VARCHAR, owner VARCHAR NOT NULL,
    PRIMARY KEY (repo_id, prefix)
);
INSERT INTO ownership SELECT * FROM read_csv('owners.csv', header = true, all_varchar = true);
CREATE TEMP VIEW owned_contributions AS
SELECT c.*, coalesce(m.owner, 'unassigned') AS owner
FROM contributions c
LEFT JOIN LATERAL (
    SELECT owner FROM ownership o
    WHERE o.repo_id = c.repo_id
      AND (o.prefix = '.' OR c.folder = o.prefix OR starts_with(c.folder, o.prefix || '/'))
    ORDER BY length(o.prefix) DESC
    LIMIT 1
) m ON true;
```

<!-- ownership -->
```sql
SELECT snapshot_id, owner, sum(code) AS code
FROM owned_contributions
WHERE snapshot_id = 'api-2' AND kind = 'source'
GROUP BY snapshot_id, owner
ORDER BY snapshot_id, owner;
```

Expected: `api-team` 3, `quality` 2. Unmatched folders remain `unassigned` rather than disappearing in an inner join.

## Optional file-size summaries

This recipe requires `--files` for the snapshots you want to inspect. Physical-file counts already include embedded content; do not add their language breakdown again. Results cover counted source files, with partialness available from `snapshots`.

<!-- file-sizes -->
```sql
SELECT s.snapshot_id,
       CASE WHEN s.has_files THEN count(f.path) END AS counted_source_files,
       median(f.code) AS median_code, max(f.code) AS largest_code
FROM snapshots s
LEFT JOIN physical_files f ON f.snapshot_id = s.snapshot_id AND f.kind = 'source'
GROUP BY s.snapshot_id, s.has_files
ORDER BY s.snapshot_id;
```

The first API capture has two counted source files, median 1.5 and maximum 2 code lines. The second API capture has NULL file statistics because it was captured without `--files`. A recorded empty file list has zero counted files and NULL size statistics.

## Share a normalized dataset

Export both snapshot metadata and contributions, retaining empty snapshots and provenance. These files are derived analytical data; preserve the original reports and manifest too.

<!-- parquet -->
```sql
COPY (SELECT * FROM snapshots) TO 'snapshots.parquet' (FORMAT PARQUET);
COPY (SELECT * FROM contributions) TO 'contributions.parquet' (FORMAT PARQUET);
```

The same tables can be read by pandas or Polars. [Their cookbook](dataframe-cookbook.md) also demonstrates direct JSON ingestion without DuckDB. The [dated tool assessment](research/report-analytics.md) explains the engine choice and experiment limits.

## Verification

From the Locmetry repository, build the current executable with `cargo build --locked`. Run the pinned optional checks through uv's external cache:

```sh
uv run --no-project --python 3.11 \
  --with duckdb==1.5.6 --with pandas==3.0.6 --with polars==2.0.0 \
  python scripts/test_analytics_cookbooks.py
```

Alternatively, run `python scripts/test_analytics_cookbooks.py` in an external environment already containing those packages. The checks execute these source snippets against temporary synthetic projects, assert independent counts, preserve empty and unavailable observations, reject mismatched envelopes, and compare the dataframe results. Python packages and environments are not stored in Git or required by Locmetry or its public Cargo tests.
