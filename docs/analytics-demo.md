# Capture and Analyze Two Immich Releases

*2026-10-21T05:13:33Z by Showboat 0.6.1*
<!-- showboat-id: 07461e62-985b-4c8c-b70a-a93d791f49f4 -->

The [DuckDB cookbook](duckdb-cookbook.md) and the [pandas and Polars cookbook](dataframe-cookbook.md) teach one technique per section on whole-repository reports. This demonstration combines them in a single Python script, the shape an analysis takes in a scheduled job or a notebook, and applies it to *workspace* reports of the two pinned Immich releases. In a workspace report each member is a top-level folder, so member names come for free.

Run it from the repository root after `cargo build --locked` and `bash scripts/fetch-showcase.sh`. It needs Bash and DuckDB's Python package, verified with `duckdb==1.5.6`; the [replay command](showcase/README.md#verify-and-regenerate) provides it through uv. The block works in its own temporary directory.

## Capture two workspaces and query them from Python

The [workspace file](../examples/showcase/immich/overlay/workspace.toml) names six of Immich's components as folders of one project. The script loads both captures with the cookbook's [loader](../examples/analytics/duckdb.sql), then prints each capture, the change per member and purpose, and the largest new source file of each member.

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
cp "$repository/examples/analytics/duckdb.sql" .
python3 - <<'PYTHON'
from pathlib import Path
import duckdb

db = duckdb.connect()
db.execute(Path("duckdb.sql").read_text())
db.sql("""
    SELECT snapshot_id, recency, source_code, production_code, test_code
    FROM snapshots ORDER BY sequence
""").show()
db.sql("""
    SELECT split_part(folder, '/', 1) AS member, purpose,
           sum(CASE recency WHEN 1 THEN code::HUGEINT ELSE -code::HUGEINT END) AS change
    FROM contributions
    WHERE kind = 'source' AND recency <= 2
    GROUP BY ALL HAVING change <> 0
    ORDER BY abs(change) DESC, member, purpose LIMIT 8
""").show()
db.sql("""
    SELECT split_part(n.path, '/', 1) AS member, arg_max(n.path, n.code) AS largest_new_file,
           max(n.code) AS code
    FROM physical_files n
    WHERE n.recency = 1 AND n.kind = 'source' AND NOT EXISTS (
        SELECT 1 FROM physical_files o WHERE o.recency = 2 AND o.path = n.path)
    GROUP BY member ORDER BY code DESC, member
""").show()
PYTHON
```

```output
┌─────────────┬─────────┬─────────────┬─────────────────┬───────────┐
│ snapshot_id │ recency │ source_code │ production_code │ test_code │
│   varchar   │ uint64  │   uint64    │     uint64      │  uint64   │
├─────────────┼─────────┼─────────────┼─────────────────┼───────────┤
│ v3.2.0      │       2 │      272749 │          176788 │     81051 │
│ v3.3.4      │       1 │      281955 │          178889 │     86901 │
└─────────────┴─────────┴─────────────┴─────────────────┴───────────┘

┌──────────────────┬────────────┬────────┐
│      member      │  purpose   │ change │
│     varchar      │  varchar   │ int128 │
├──────────────────┼────────────┼────────┤
│ server           │ test       │   3111 │
│ server           │ production │   1866 │
│ mobile           │ test       │   1791 │
│ mobile           │ production │  -1227 │
│ server           │ other      │   1082 │
│ machine-learning │ test       │    648 │
│ machine-learning │ production │    574 │
│ web              │ production │    574 │
└──────────────────┴────────────┴────────┘

┌──────────────────┬──────────────────────────────────────────────────────────────────────┬────────┐
│      member      │                           largest_new_file                           │  code  │
│     varchar      │                               varchar                                │ uint64 │
├──────────────────┼──────────────────────────────────────────────────────────────────────┼────────┤
│ server           │ server/test/medium/specs/repositories/person-user.repository.spec.ts │    364 │
│ packages         │ packages/scripts/src/commands/notes.ts                               │    283 │
│ mobile           │ mobile/test/presentation/pages/slideshow_page_test.dart              │    275 │
│ web              │ web/src/lib/modals/PersonBulkShareModal.svelte                       │    200 │
│ machine-learning │ machine-learning/immich_ml/sessions/prepare.py                       │     88 │
└──────────────────┴──────────────────────────────────────────────────────────────────────┴────────┘

```

`execute` runs the whole loader at once; in Python, its check queries raise an exception instead of printing. `show()` prints a result the way the DuckDB program does; `fetchall()` would return Python tuples for further processing instead. `arg_max(path, code)` picks the path of the largest new file in each group in one pass.

The member changes match the [whole-repository change](duckdb-cookbook.md#where-did-the-change-happen) and the [recorded workspace comparison](showcase/compare.md), but the totals do not match the whole-repository reports: a workspace counts only its members, and Immich's root files, `docs` and `deployment` folders lie outside all six. Analyze workspace captures as their own series and never add them to whole-repository captures of the same project.
