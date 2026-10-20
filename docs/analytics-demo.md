# Capture and Analyze Acme Orders

*2026-10-20T12:55:38Z by Showboat 0.6.1*
<!-- showboat-id: 16d1fd9f-aa9d-4bb2-b3bb-fac2b1b6aae4 -->

This demonstration uses the same [synthetic workspace](../examples/cli-workspace/README.md) as the [CLI guides](cli/README.md), measured by the current Locmetry executable. It captures a known edit and queries full schema-3 reports using the [DuckDB cookbook's typed views](duckdb-cookbook.md#load-typed-views). The [pandas/Polars cookbook](dataframe-cookbook.md) demonstrates the same report structure in dataframes.

Run from the repository root after `cargo build --locked`. Bash, jq and the optional DuckDB Python package are required. The [replay command](cli/README.md#verify-and-regenerate) supplies pinned Showboat and DuckDB through uv without adding packages or dependency files to the repository. Every Bash block creates and cleans up a fresh temporary fixture copy. The generated reports keep their real timestamps and paths; the printed queries select stable evidence.

## Capture a visible change

One added TypeScript line should increase source code from 25 to 26. First capture physical-file detail; omit it from the second capture to show that aggregate analytics does not require `--files`. Caller-assigned snapshot IDs and sequence establish identity and order; the display name and timestamps do not.

```bash
source scripts/cli-demo-env.sh
locmetry --root acme-orders --no-user-config --files --output before.json
printf 'export const discount = 1;\n' >> acme-orders/web/src/orders/Order.ts
locmetry --root acme-orders --no-user-config --output after.json
locmetry compare before.json after.json --format json \
  | jq '.summary.source | {
      before: .before.code,
      after: .after.code,
      delta: .delta.code
    }'
cat > snapshots.csv <<'CSV'
snapshot_id,repo_id,sequence,path
before,acme-orders,1,before.json
after,acme-orders,2,after.json
CSV
cp "$repository/examples/analytics/duckdb.sql" duckdb.sql
python3 - <<'PYTHON'
from pathlib import Path
import duckdb
connection = duckdb.connect()
connection.execute(Path("duckdb.sql").read_text())
print("Snapshot, source code, partial, file detail:")
snapshot_rows = connection.sql("""
    SELECT snapshot_id, source_code, partial, has_files
    FROM snapshots ORDER BY sequence
""").fetchall()
for row in snapshot_rows:
    print(row)
print("After: language, purpose, code:")
composition_rows = connection.sql("""
    SELECT language, purpose, sum(code) FROM contributions
    WHERE snapshot_id = 'after' AND kind = 'source'
    GROUP BY ALL ORDER BY ALL
""").fetchall()
for row in composition_rows:
    print(row)
PYTHON
```

```output
{
  "before": 25,
  "after": 26,
  "delta": 1
}
Snapshot, source code, partial, file detail:
('before', 25, False, True)
('after', 26, False, False)
After: language, purpose, code:
('Go', 'production', 4)
('Go', 'test', 4)
('HTML', 'production', 4)
('JavaScript', 'production', 1)
('Python', 'other', 1)
('Python', 'production', 2)
('TSX', 'production', 3)
('TypeScript', 'production', 4)
('TypeScript', 'test', 3)
```

The aggregate delta agrees with the independently known edit. Both snapshots survive in the inventory even though file detail is absent from the second. Embedded JavaScript remains a contribution. [Cookbook verification](duckdb-cookbook.md#verification) additionally checks empty reports, partial counts, missing detail, malformed envelopes and pandas/Polars parity. A recorded run proves these commands replay; the assertion tests establish the expected semantics.
