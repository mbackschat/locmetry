CREATE OR REPLACE TABLE manifest (
    snapshot_id VARCHAR PRIMARY KEY CHECK (snapshot_id <> ''),
    repo_id VARCHAR NOT NULL CHECK (repo_id <> ''),
    sequence UBIGINT NOT NULL,
    path VARCHAR NOT NULL UNIQUE,
    UNIQUE (repo_id, sequence)
);
INSERT INTO manifest
SELECT * FROM read_csv('snapshots.csv', header = true, columns = {
    'snapshot_id': 'VARCHAR', 'repo_id': 'VARCHAR',
    'sequence': 'UBIGINT', 'path': 'VARCHAR'
});
SET VARIABLE report_paths = (SELECT list(path) FROM manifest);
CREATE OR REPLACE TABLE reports AS
SELECT filename, json AS doc
FROM read_json_objects(getvariable('report_paths'), format = 'unstructured');

SELECT CASE WHEN count(*) = 0 THEN 'One report per manifest path'
    ELSE error('Expected exactly one JSON document per manifest path')
END AS inventory_check
FROM (
    SELECT m.path FROM manifest m LEFT JOIN reports r ON m.path = r.filename
    GROUP BY m.path HAVING count(r.filename) <> 1
);

SELECT CASE WHEN count(*) = 0 THEN 'Full measurements loaded'
    ELSE error('Expected a full schema-3 measurement with breakdown, review and coverage evidence')
END AS input_check
FROM reports
WHERE (doc->>'document_type') IS DISTINCT FROM 'measurement'
   OR (doc->>'schema_version') IS DISTINCT FROM '3'
   OR (doc->>'detail') IS DISTINCT FROM 'full'
   OR json_type(doc, '$.breakdown') IS DISTINCT FROM 'ARRAY'
   OR json_type(doc, '$.review') IS DISTINCT FROM 'ARRAY'
   OR json_type(doc, '$.metadata.backend_inaccurate') IS DISTINCT FROM 'BOOLEAN'
   OR json_type(doc, '$.metadata.coverage.parse_failures') IS DISTINCT FROM 'ARRAY';

CREATE OR REPLACE VIEW snapshots AS
SELECT m.*, r.doc,
    -- 1 is a repository's latest capture, 2 the one before it.
    row_number() OVER (PARTITION BY m.repo_id ORDER BY m.sequence DESC)::UBIGINT AS recency,
    (doc->>'$.metadata.measured_at_unix_seconds')::UBIGINT AS measured_at_unix_seconds,
    (doc->>'$.metadata.backend_inaccurate')::BOOLEAN AS partial,
    json_exists(doc, '$.files') AS has_files,
    doc->>'$.metadata.scope.subfolder' AS subfolder,
    coalesce((doc->>'$.summary.source.code')::UBIGINT, 0) AS source_code,
    coalesce((doc->>'$.summary.source_purpose.production.code')::UBIGINT, 0) AS production_code,
    coalesce((doc->>'$.summary.source_purpose.test.code')::UBIGINT, 0) AS test_code
FROM manifest m JOIN reports r ON m.path = r.filename;

-- Empty JSON arrays carry no element types; the report contract supplies them.
CREATE OR REPLACE VIEW contributions AS
SELECT snapshot_id, repo_id, sequence, recency, b.*
FROM (
    SELECT snapshot_id, repo_id, sequence, recency,
        unnest(from_json_strict(doc->'breakdown', '[{
            "folder":"VARCHAR", "kind":"VARCHAR", "purpose":"VARCHAR",
            "category":"VARCHAR", "language":"VARCHAR", "files":"UBIGINT",
            "code":"UBIGINT", "comments":"UBIGINT", "blanks":"UBIGINT", "lines":"UBIGINT"
        }]')) AS b
    FROM snapshots
);

CREATE OR REPLACE VIEW physical_files AS
SELECT snapshot_id, repo_id, sequence, recency,
    f.path, f.kind, f.purpose, f.category, f.language, f.counts.code AS code
FROM (
    SELECT snapshot_id, repo_id, sequence, recency, unnest(from_json_strict(doc->'files', '[{
        "path":"VARCHAR", "kind":"VARCHAR", "purpose":"VARCHAR", "category":"VARCHAR",
        "language":"VARCHAR", "counts":{"code":"UBIGINT"}
    }]')) AS f
    FROM snapshots
);
