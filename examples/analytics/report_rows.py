"""Project trusted schema-3 measurements into tool-neutral snapshot and contribution rows."""

import csv
import json
from pathlib import Path

COUNTS = ("files", "code", "comments", "blanks", "lines")
DIMENSIONS = ("folder", "kind", "purpose", "category", "language")
IDENTITY = ("snapshot_id", "repo_id", "sequence", "recency")
SNAPSHOT_FIELDS = IDENTITY + ("source_code", "partial", "has_files")
CONTRIBUTION_FIELDS = IDENTITY + DIMENSIONS + COUNTS


def read_reports(manifest="snapshots.csv"):
    snapshots, contributions = [], []
    identities, positions, paths = set(), set(), set()
    with Path(manifest).open(newline="") as source:
        for entry in csv.DictReader(source):
            snapshot_id, repo_id = entry["snapshot_id"], entry["repo_id"]
            sequence, path = int(entry["sequence"]), entry["path"]
            if (not snapshot_id or not repo_id or not 0 <= sequence <= 2**64 - 1
                    or snapshot_id in identities or (repo_id, sequence) in positions or path in paths):
                raise ValueError("Expected unique snapshots, paths and repository sequence positions")
            identities.add(snapshot_id)
            positions.add((repo_id, sequence))
            paths.add(path)
            report = json.loads(Path(path).read_text())
            if (report.get("document_type"), report.get("schema_version"), report.get("detail")) != ("measurement", "3", "full"):
                raise ValueError("Expected a full schema-3 measurement")
            metadata = report["metadata"]
            if (not isinstance(report.get("breakdown"), list)
                    or not isinstance(report.get("review"), list)
                    or type(metadata.get("backend_inaccurate")) is not bool
                    or not isinstance(metadata["coverage"].get("parse_failures"), list)):
                raise ValueError("Expected breakdown, review and coverage evidence")
            snapshot = {"snapshot_id": snapshot_id, "repo_id": repo_id, "sequence": sequence}
            snapshots.append({
                **snapshot,
                "source_code": report["summary"].get("source", {}).get("code", 0),
                "partial": metadata["backend_inaccurate"], "has_files": "files" in report,
            })
            for row in report["breakdown"]:
                contributions.append({**snapshot, **{key: row[key] for key in DIMENSIONS + COUNTS}})
    if not snapshots:
        raise ValueError("Expected at least one snapshot")
    # As in duckdb.sql: recency 1 is a repository's latest capture, 2 the one before it.
    sequences = {}
    for snapshot in snapshots:
        sequences.setdefault(snapshot["repo_id"], []).append(snapshot["sequence"])
    recency = {(repo_id, sequence): rank
               for repo_id, ranked in sequences.items()
               for rank, sequence in enumerate(sorted(ranked, reverse=True), 1)}
    for row in snapshots + contributions:
        row["recency"] = recency[row["repo_id"], row["sequence"]]
    return snapshots, contributions


def column_types(fields):
    """Map each field to `count`, `flag` or `text`, which each library spells in its own types."""
    return {field: "count" if field in COUNTS + ("sequence", "recency", "source_code")
            else "flag" if field in ("partial", "has_files") else "text"
            for field in fields}


def pandas_frames(manifest="snapshots.csv"):
    """Snapshots and contributions as pandas frames with explicit nullable types.

    Inferring types would turn integer columns with gaps into floating point, and
    an empty contribution list would have no columns at all; see
    https://pandas.pydata.org/docs/user_guide/integer_na.html.
    """
    import pandas as pd
    types = {"count": "UInt64", "flag": "boolean", "text": "string"}

    def frame(rows, fields):
        return pd.DataFrame({field: pd.Series([row[field] for row in rows], dtype=types[kind])
                             for field, kind in column_types(fields).items()})

    snapshots, contributions = read_reports(manifest)
    return frame(snapshots, SNAPSHOT_FIELDS), frame(contributions, CONTRIBUTION_FIELDS)


def polars_frames(manifest="snapshots.csv"):
    """Snapshots and contributions as Polars frames with an explicit schema, for the same reasons."""
    import polars as pl
    types = {"count": pl.UInt64, "flag": pl.Boolean, "text": pl.String}

    def frame(rows, fields):
        return pl.DataFrame(rows, schema={field: types[kind] for field, kind in column_types(fields).items()})

    snapshots, contributions = read_reports(manifest)
    return frame(snapshots, SNAPSHOT_FIELDS), frame(contributions, CONTRIBUTION_FIELDS)
