# Analyze Locmetry Reports with pandas and Polars

*2026-10-21T05:18:16Z by Showboat 0.6.1*
<!-- showboat-id: 83d71393-9084-4c77-afdf-e291b70f786b -->

Locmetry's JSON reports work with any analysis tool. This cookbook loads them into [pandas](https://pandas.pydata.org/) and [Polars](https://pola.rs/) dataframes, the tables of Python's data ecosystem, which suit notebooks, plotting and statistics. It answers questions of the [DuckDB cookbook](duckdb-cookbook.md) without a database: what each component of [Immich](https://github.com/immich-app/immich) contains and how it changed between the pinned releases `v3.2.0` and `v3.3.4`. Every output below is real and replayed in CI.

## Before you start

You need Python 3.11 or later with pandas or Polars; the examples were verified with pandas 3.0.6 and Polars 2.0.0. Install them into a separate environment, never into the Locmetry repository. With [uv](https://docs.astral.sh/uv/), `uv run --no-project --python 3.11 --with pandas==3.0.6 --with polars==2.0.0 python analysis.py` runs a saved script with both.

To run the examples yourself, build Locmetry with `cargo build --locked` and fetch the pinned checkouts once with `bash scripts/fetch-showcase.sh`. Each block starts with `source scripts/analytics-env.sh`, which captures both releases with `--files` into a fresh temporary directory, writes the manifest `snapshots.csv` and copies the loader, exactly as the [DuckDB cookbook's capture section](duckdb-cookbook.md#capture-the-reports) shows and explains.

## Load the reports into typed tables

The [loader module](../examples/analytics/report_rows.py) uses only the Python standard library. It reads the manifest, checks that each path holds one full schema-3 measurement, and returns one row per capture and one row per breakdown entry, the same tables as the DuckDB cookbook's `snapshots` and `contributions` views. Its `pandas_frames()` and `polars_frames()` functions build typed tables from those rows.

<!-- pandas-load -->

```bash
source scripts/analytics-env.sh
python3 - <<'PYTHON'
from report_rows import pandas_frames

snapshots, contributions = pandas_frames()
print(snapshots.to_string(index=False))
print(contributions.dtypes.to_string())

# Check before analyzing: the breakdown must add up to each report's own total.
source = contributions[contributions["kind"] == "source"]
recorded = source.groupby("snapshot_id")["code"].sum()
totals = snapshots.set_index("snapshot_id")["source_code"]
assert (recorded.reindex(totals.index, fill_value=0) == totals).all()
print("Breakdown matches each report's source total")
PYTHON
```

```output
snapshot_id repo_id  sequence  recency  source_code  partial  has_files
     v3.2.0  immich         1        2       273566    False       True
     v3.3.4  immich         2        1       282770    False       True
snapshot_id    string
repo_id        string
sequence       UInt64
recency        UInt64
folder         string
kind           string
purpose        string
category       string
language       string
files          UInt64
code           UInt64
comments       UInt64
blanks         UInt64
lines          UInt64
Breakdown matches each report's source total
```

`recency` numbers each repository's captures from the newest, so 1 is the latest release. The column types are set explicitly, as the type listing shows. If pandas inferred them, an integer column with a missing value would become floating point and lose exactness, and an empty report would yield a table without columns; the [pandas guide on nullable integers](https://pandas.pydata.org/docs/user_guide/integer_na.html) explains the first problem. `UInt64` and `boolean` are pandas' nullable types.

The last lines reconcile the breakdown with the total each report records. A check like this costs one line and catches a wrong filter, such as forgetting `kind == "source"`, before it skews every later number.

## Components and their tests, with pandas

Which components carry Immich's code, and how much test code does each have per production line? A *pivot table* turns purposes into columns, one row per top-level folder.

<!-- pandas-components -->

```bash
source scripts/analytics-env.sh
python3 - <<'PYTHON'
import pandas as pd
from report_rows import pandas_frames

snapshots, contributions = pandas_frames()
latest = contributions[(contributions["recency"] == 1) & (contributions["kind"] == "source")]
latest = latest.assign(component=latest["folder"].str.split("/").str[0])
components = latest.pivot_table(index=["repo_id", "component"], columns="purpose",
                                values="code", aggfunc="sum", fill_value=0)
components["test_prod"] = (components["test"] / components["production"].replace(0, pd.NA)).round(2)
print(components.sort_values("production", ascending=False).reset_index().to_string(index=False))
PYTHON
```

```output
repo_id        component  other  production  test  test_prod
 immich           mobile      0       66989 16231       0.24
 immich           server   9699       53296 48955       0.92
 immich              web      0       52867  5388        0.1
 immich machine-learning      0        3221  2070       0.64
 immich         packages   6466        2345  1254       0.53
 immich             docs      0         401     0        0.0
 immich       deployment      0         225     0        0.0
 immich              e2e      0         171 13003      76.04
 immich                .      0         111     0        0.0
 immich    .devcontainer      0          58     0        0.0
 immich         open-api      0          14     0        0.0
 immich             misc      0           6     0        0.0
```

The numbers match the [DuckDB component query](duckdb-cookbook.md#which-components-carry-the-code-and-how-well-are-they-tested). The first folder segment names the component, and `fill_value=0` writes 0 where a component has no code of a purpose. Replacing a zero production total with `pd.NA` leaves the ratio missing instead of dividing by zero. As in SQL, the ratio divides summed code, and it is a volume ratio: `e2e` is Immich's end-to-end test suite, so its high ratio is expected.

## Which components comment their code?

Comment density, comment lines per code line, hints at documentation habits. The breakdown keeps comments and blank lines next to code, so the same table answers it.

<!-- pandas-comments -->

```bash
source scripts/analytics-env.sh
python3 - <<'PYTHON'
from report_rows import pandas_frames

snapshots, contributions = pandas_frames()
latest = contributions[(contributions["recency"] == 1) & (contributions["kind"] == "source")
                       & contributions["purpose"].isin(["production", "test"])]
latest = latest.assign(component=latest["folder"].str.split("/").str[0])
density = latest.groupby(["repo_id", "component", "purpose"])[["code", "comments"]].sum()
density = density[density["code"] >= 1000]
density["comments_per_100_code"] = (100 * density["comments"] / density["code"]).round(1)
print(density.sort_values("comments_per_100_code", ascending=False).reset_index().to_string(index=False))
PYTHON
```

```output
repo_id        component    purpose  code  comments  comments_per_100_code
 immich           mobile production 66989      2904                    4.3
 immich         packages production  2345        93                    4.0
 immich machine-learning production  3221        83                    2.6
 immich              e2e       test 13003       324                    2.5
 immich              web production 52867      1245                    2.4
 immich           mobile       test 16231       381                    2.3
 immich           server production 53296      1019                    1.9
 immich              web       test  5388        77                    1.4
 immich machine-learning       test  2070        24                    1.2
 immich         packages       test  1254         7                    0.6
 immich           server       test 48955       303                    0.6
```

The mobile app's production code carries the most comments, about four per 100 code lines, and the server's tests the fewest. The filter keeps groups with at least 1,000 code lines, because a ratio of a handful of lines says little. Comment lines are what the counting engine recognizes as comments, including license headers and commented-out code, so treat the density as a prompt to look, not as a quality score.

## Change between releases, with pandas

Which components grew between the two releases? The change is the latest capture's code minus the previous capture's, for each component and purpose.

<!-- pandas-changes -->

```bash
source scripts/analytics-env.sh
python3 - <<'PYTHON'
from report_rows import pandas_frames

snapshots, contributions = pandas_frames()
# Compare only repositories with two captures, neither of them partial.
pair = snapshots[snapshots["recency"] <= 2].groupby("repo_id").agg(
    captures=("snapshot_id", "size"), partial=("partial", "any"))
comparable = pair.index[(pair["captures"] == 2) & ~pair["partial"]]
rows = contributions[(contributions["kind"] == "source") & (contributions["recency"] <= 2)
                     & contributions["repo_id"].isin(comparable)]
signed = rows["code"].astype("Int64").where(rows["recency"] == 1, -rows["code"].astype("Int64"))
rows = rows.assign(component=rows["folder"].str.split("/").str[0], change=signed)
changes = rows.groupby(["repo_id", "component", "purpose"], as_index=False)["change"].sum()
changes = changes[changes["change"] != 0]
changes = (changes.assign(size=changes["change"].abs())
           .sort_values(["size", "repo_id", "component", "purpose"], ascending=[False, True, True, True])
           .drop(columns="size"))
print(changes.head(10).to_string(index=False))
PYTHON
```

```output
repo_id        component    purpose  change
 immich           server       test    3111
 immich           server production    1866
 immich           mobile       test    1791
 immich           mobile production   -1227
 immich           server      other    1082
 immich machine-learning       test     648
 immich machine-learning production     574
 immich              web production     574
 immich         packages production     309
 immich         packages       test     195
```

Server tests grew most, and the mobile app gained tests while its production code shrank, as the [DuckDB change query](duckdb-cookbook.md#where-did-the-change-happen) shows. Each row adds its code when it belongs to the latest capture and subtracts it when it belongs to the previous one, so one sum per group yields the change; sorting by the absolute size lists decreases among the largest changes. The code is converted to the signed `Int64` first, because unsigned counts cannot hold a decrease. A component that exists in only one capture still appears, with its whole size as the change.

## The same change with Polars

Polars expresses the same steps as a chain of column *expressions*. Its results are identical.

<!-- polars-changes -->

```bash
source scripts/analytics-env.sh
python3 - <<'PYTHON'
import polars as pl
from report_rows import polars_frames

snapshots, contributions = polars_frames()
comparable = (snapshots.filter(pl.col("recency") <= 2)
              .group_by("repo_id")
              .agg(pl.len().alias("captures"), pl.col("partial").any())
              .filter((pl.col("captures") == 2) & ~pl.col("partial"))["repo_id"])
code = pl.col("code").cast(pl.Int64)
changes = (contributions
           .filter((pl.col("kind") == "source") & (pl.col("recency") <= 2)
                   & pl.col("repo_id").is_in(comparable.implode()))
           .with_columns(component=pl.col("folder").str.split("/").list.first(),
                         change=pl.when(pl.col("recency") == 1).then(code).otherwise(-code))
           .group_by("repo_id", "component", "purpose")
           .agg(pl.col("change").sum())
           .filter(pl.col("change") != 0)
           .sort(pl.col("change").abs(), "repo_id", "component", "purpose",
                 descending=[True, False, False, False]))
print(changes.head(10))
PYTHON
```

```output
shape: (10, 4)
┌─────────┬──────────────────┬────────────┬────────┐
│ repo_id ┆ component        ┆ purpose    ┆ change │
│ ---     ┆ ---              ┆ ---        ┆ ---    │
│ str     ┆ str              ┆ str        ┆ i64    │
╞═════════╪══════════════════╪════════════╪════════╡
│ immich  ┆ server           ┆ test       ┆ 3111   │
│ immich  ┆ server           ┆ production ┆ 1866   │
│ immich  ┆ mobile           ┆ test       ┆ 1791   │
│ immich  ┆ mobile           ┆ production ┆ -1227  │
│ immich  ┆ server           ┆ other      ┆ 1082   │
│ immich  ┆ machine-learning ┆ test       ┆ 648    │
│ immich  ┆ machine-learning ┆ production ┆ 574    │
│ immich  ┆ web              ┆ production ┆ 574    │
│ immich  ┆ packages         ┆ production ┆ 309    │
│ immich  ┆ packages         ┆ test       ┆ 195    │
└─────────┴──────────────────┴────────────┴────────┘
```

`filter`, `with_columns`, `group_by` and `agg` correspond to the pandas steps above. Polars evaluates each expression over whole columns and can also run a query *lazily*, planning it before reading data; for repeated analysis, read the [Parquet export](duckdb-cookbook.md#share-the-dataset) with `pl.scan_parquet`, as the [Polars guide on lazy queries](https://docs.pola.rs/user-guide/lazy/using/) describes.

## Choose a tool and interpret the results

Choose pandas when your work already happens in notebooks with plotting or statistics, Polars when you prefer its typed expressions, and [DuckDB](duckdb-cookbook.md) for SQL joins, window queries and a persistent database. The [research assessment](research/report-analytics.md) found all three suitable at the tested scale; these are workflow choices, not performance rankings.

The interpretation rules of the DuckDB cookbook apply unchanged. Divide summed counts instead of averaging ratios, and leave a zero denominator undefined. Check comparability with [`locmetry compare`](guides/compare.md) before reading a change as growth, and keep partial captures out of differences. Do not combine a workspace report with standalone reports of its members.

## Verification

These recordings replay with the other showcase recordings; the [showcase index](showcase/README.md#verify-and-regenerate) has the command. The [cookbook test](../scripts/test_analytics_cookbooks.py) also runs the marked examples of this page on small synthetic reports, including an empty project, and checks that pandas, Polars and DuckDB produce the same tables. The [DuckDB cookbook's verification section](duckdb-cookbook.md#verification) shows how to run it.
