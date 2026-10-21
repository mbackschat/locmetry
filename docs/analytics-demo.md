# Capture and Analyze Two Immich Releases

*2026-10-21T03:17:10Z by Showboat 0.6.1*
<!-- showboat-id: 7b492f65-86fd-47c1-bcb5-be4e7adb4ce3 -->

This demonstration analyzes the two pinned Immich releases of the [showcase](showcase/README.md) with the [DuckDB cookbook's typed views](duckdb-cookbook.md#load-typed-views). The [pandas/Polars cookbook](dataframe-cookbook.md) demonstrates the same report structure in dataframes.

Run from the repository root after `cargo build --locked` and `bash scripts/fetch-showcase.sh`. Bash, jq and the optional DuckDB Python package are required; the [replay command](showcase/README.md#verify-and-regenerate) supplies pinned Showboat and DuckDB through uv without adding packages or dependency files to the repository. The block creates and removes its own temporary directory. Reports keep their real timestamps and paths; the queries select stable evidence.

## Capture two releases and query them

Capture both workspaces with physical-file detail. A manifest assigns snapshot identity and order; display names and timestamps do not. The queries list each snapshot, attribute the change in source code to members and purposes, and name the largest source files new in `v3.3.4`.

```bash
source scripts/showcase-env.sh
for tag in v3.2.0 v3.3.4; do
  locmetry workspace --workspace-file "$tag/immich/workspace.toml" --files --output "$tag.json"
done
cat > snapshots.csv <<'CSV'
snapshot_id,repo_id,sequence,path
v3.2.0,immich,1,v3.2.0.json
v3.3.4,immich,2,v3.3.4.json
CSV
cp "$repository/examples/analytics/duckdb.sql" duckdb.sql
python3 - <<'PYTHON'
from pathlib import Path
import duckdb
connection = duckdb.connect()
connection.execute(Path("duckdb.sql").read_text())
print("Snapshot, source code, production, test, file detail:")
for row in connection.sql("""
    SELECT snapshot_id, source_code, production_code, test_code, has_files
    FROM snapshots ORDER BY sequence
""").fetchall():
    print(row)
print("Member, purpose, change in source code:")
for row in connection.sql("""
    SELECT split_part(folder, '/', 1) AS member, purpose,
        sum(CASE WHEN snapshot_id = 'v3.3.4' THEN code ELSE 0 END)::BIGINT
            - sum(CASE WHEN snapshot_id = 'v3.2.0' THEN code ELSE 0 END)::BIGINT AS change
    FROM contributions WHERE kind = 'source'
    GROUP BY ALL HAVING change <> 0 ORDER BY abs(change) DESC, member, purpose LIMIT 8
""").fetchall():
    print(row)
print("Largest new source files:")
for row in connection.sql("""
    SELECT path, purpose, code FROM physical_files
    WHERE snapshot_id = 'v3.3.4' AND kind = 'source' AND path NOT IN (
        SELECT path FROM physical_files WHERE snapshot_id = 'v3.2.0')
    ORDER BY code DESC, path LIMIT 5
""").fetchall():
    print(row)
PYTHON
```

```output
Snapshot, source code, production, test, file detail:
('v3.2.0', 272749, 176788, 81051, True)
('v3.3.4', 281955, 178889, 86901, True)
Member, purpose, change in source code:
('server', 'test', 3111)
('server', 'production', 1866)
('mobile', 'test', 1791)
('mobile', 'production', -1227)
('server', 'other', 1082)
('machine-learning', 'test', 648)
('machine-learning', 'production', 574)
('web', 'production', 574)
Largest new source files:
('server/test/medium/specs/repositories/person-user.repository.spec.ts', 'test', 364)
('packages/scripts/src/commands/notes.ts', 'production', 283)
('mobile/test/presentation/pages/slideshow_page_test.dart', 'test', 275)
('web/src/lib/modals/PersonBulkShareModal.svelte', 'production', 200)
('mobile/lib/presentation/widgets/sliver_app_bar/random_asset_background_image.widget.dart', 'production', 197)
```

The member totals reconcile with the [recorded comparison](showcase/compare.md): server tests account for the largest share of the change. File detail adds what aggregates cannot, such as which new files carry the code. [Cookbook verification](duckdb-cookbook.md#verification) checks empty reports, partial counts, missing detail, malformed envelopes and pandas/Polars parity; a recorded run proves these commands replay, while the assertion tests establish the expected semantics.
