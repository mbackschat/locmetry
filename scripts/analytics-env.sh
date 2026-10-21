# Source from the repository root in an analytics cookbook example. It extends
# showcase-env.sh with the cookbooks' starting point: full reports of both pinned
# Immich releases, their manifest and the loaders, as the capture section of
# docs/duckdb-cookbook.md creates them, so each example runs on its own.
# `source scripts/analytics-env.sh duckdb` also loads them into immich.duckdb.
source scripts/showcase-env.sh
for tag in v3.2.0 v3.3.4; do
    locmetry --root "$tag/immich" --files --output "$tag.json"
done
cat > snapshots.csv <<'CSV'
snapshot_id,repo_id,sequence,path
v3.2.0,immich,1,v3.2.0.json
v3.3.4,immich,2,v3.3.4.json
CSV
cp "$repository/examples/analytics/duckdb.sql" "$repository/examples/analytics/report_rows.py" .
if [[ ${1-} == duckdb ]]; then
    duckdb immich.duckdb < duckdb.sql > /dev/null
fi
