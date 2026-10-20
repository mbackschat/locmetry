# Analyze Measurements with pandas and Polars

Measurement schema 3 is tool-neutral JSON: `document_type: "measurement"` identifies the document and `detail: "full"` identifies the analytical input. Counts and additive breakdown rows have the same meaning in SQL and dataframes. This shorter companion to the [DuckDB cookbook](duckdb-cookbook.md) demonstrates both Python libraries without routing data through DuckDB.

## Prepare the same reports

Use the [capture example](duckdb-cookbook.md#capture-a-small-portfolio) to create four reports and `snapshots.csv`. Copy [report_rows.py](../examples/analytics/report_rows.py) into that analysis directory. It uses only the Python standard library and returns separate snapshot and contribution records, preserving empty captures and partial/detail flags. It checks the envelope and basic input sections; it is a projection of trusted exports, not a full report validator. Manifest paths resolve from the working directory, as in the SQL recipe.

Install whichever library you use into an external environment. The examples were verified with pandas 3.0.6 and Polars 2.0.0. For example, `uv run --no-project --python 3.11 --with pandas==3.0.6 python analysis.py` runs a saved pandas example; substitute `polars==2.0.0` for the Polars example. Locmetry has no Python runtime dependency; no environment, installed package or dependency/lock file is committed for these recipes.

## pandas

Construct typed columns directly from Python values. Inference across missing integers can choose floating point; assigning an integer dtype after rounding cannot recover precision. The [pandas nullable integer guide](https://pandas.pydata.org/docs/user_guide/integer_na.html) describes this distinction. Explicit column names also make an all-empty contribution set queryable.

<!-- pandas -->
```python
import pandas as pd
from report_rows import COUNTS, DIMENSIONS, read_reports

snapshot_rows, contribution_rows = read_reports()
snapshots = pd.DataFrame({
    name: pd.Series([row[name] for row in snapshot_rows], dtype=dtype)
    for name, dtype in {
        'snapshot_id': 'string', 'repo_id': 'string', 'sequence': 'UInt64',
        'source_code': 'UInt64', 'partial': 'boolean', 'has_files': 'boolean',
    }.items()
})
contributions = pd.DataFrame({
    name: pd.Series([row[name] for row in contribution_rows],
                    dtype='UInt64' if name in COUNTS else 'string')
    for name in ('snapshot_id',) + DIMENSIONS + COUNTS
})
source = contributions.loc[contributions['kind'].eq('source')]
composition = source.groupby(['snapshot_id', 'language', 'purpose'], as_index=False)['code'].sum()
totals = snapshots.merge(source.groupby('snapshot_id', as_index=False)['code'].sum(),
                         on='snapshot_id', how='left', validate='one_to_one')
totals['code'] = totals['code'].fillna(0)
assert totals['code'].eq(totals['source_code']).all()
print(composition.to_string(index=False))
```

The `snapshots` table supplies the complete inventory. A left join retains empty observations; the filled zero is valid for the sum of recorded contributions, while `partial` still distinguishes unavailable measurements. Missing per-file detail is tracked separately by `has_files`. These semantics are shared with SQL rather than inferred from a dataframe's NULL behavior.

## Polars

Use the same standard-library loader and explicit schema. This example is an eager JSON workflow. Polars also supports [lazy queries](https://docs.pola.rs/user-guide/lazy/using/); for repeated analysis, the [Parquet exports](duckdb-cookbook.md#share-a-normalized-dataset) can be read with `pl.scan_parquet`.

<!-- polars -->
```python
import polars as pl
from report_rows import COUNTS, DIMENSIONS, read_reports

snapshot_rows, contribution_rows = read_reports()
snapshots = pl.DataFrame(snapshot_rows, schema={
    'snapshot_id': pl.String, 'repo_id': pl.String, 'sequence': pl.UInt64,
    'source_code': pl.UInt64, 'partial': pl.Boolean, 'has_files': pl.Boolean,
})
contributions = pl.DataFrame(contribution_rows, schema={
    name: pl.UInt64 if name in COUNTS else pl.String
    for name in ('snapshot_id',) + DIMENSIONS + COUNTS
})
source = contributions.filter(pl.col('kind') == 'source')
composition = source.group_by('snapshot_id', 'language', 'purpose').agg(pl.col('code').sum())
totals = snapshots.join(source.group_by('snapshot_id').agg(pl.col('code').sum()),
                        on='snapshot_id', how='left', validate='1:1')
totals = totals.with_columns(pl.col('code').fill_null(0))
assert totals.select((pl.col('code') == pl.col('source_code')).all()).item()
print(composition.sort('snapshot_id', 'language', 'purpose'))
```

Both examples produce API production/test counts of 2/1 then 3/2, and Web counts of 0/1 then 2/2. Missing groups do not create fabricated files. Embedded-language contribution rows remain present even when their physical-file count is zero.

## Interpret and extend

Choose pandas when the surrounding workflow already uses its notebook, plotting or statistical ecosystem; choose Polars when you prefer its typed expression API. Use [DuckDB](duckdb-cookbook.md) for SQL joins and window queries. The [research assessment](research/report-analytics.md) found all three suitable at the tested scale; these are workflow choices, not performance rankings.

The [portfolio](duckdb-cookbook.md#portfolio-totals-and-ratios), [trend](duckdb-cookbook.md#recorded-net-change) and [ownership](duckdb-cookbook.md#join-folder-ownership) interpretation rules apply equally here. Select one capture per repository for a portfolio, divide summed test/production counts, and keep a zero denominator undefined. Match scope and policy before interpreting trends; retain partial status and use signed arithmetic for decreases. Do not combine workspace totals with their constituent root captures. These examples assume aggregate counts fit the selected integer types; they are not arbitrary-precision aggregation engines.

The [cookbook checks](../scripts/test_analytics_cookbooks.py) execute both snippets and compare them with DuckDB for the same reports, including an all-empty measurement and an embedded-language fixture. The examples add no analytical dependency to the Locmetry executable or public Cargo suite.
