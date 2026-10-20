"""Project trusted schema-3 measurements into tool-neutral snapshot and contribution rows."""

import csv
import json
from pathlib import Path

COUNTS = ("files", "code", "comments", "blanks", "lines")
DIMENSIONS = ("folder", "kind", "purpose", "category", "language")


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
            snapshots.append({
                "snapshot_id": snapshot_id, "repo_id": repo_id, "sequence": sequence,
                "source_code": report["summary"].get("source", {}).get("code", 0),
                "partial": metadata["backend_inaccurate"], "has_files": "files" in report,
            })
            for row in report["breakdown"]:
                contributions.append({"snapshot_id": snapshot_id,
                                      **{key: row[key] for key in DIMENSIONS + COUNTS}})
    if not snapshots:
        raise ValueError("Expected at least one snapshot")
    return snapshots, contributions
