#!/usr/bin/env python3
"""Gate source file sizes by purpose with Locmetry's own measurement; policy: docs/self-measurement.md."""

import argparse
import json
from pathlib import Path
import subprocess
import sys

REPOSITORY = Path(__file__).resolve().parent.parent


def measure(locmetry):
    """The repository's full report with per-file records; a partial report fails, as missing counts are not zero."""
    result = subprocess.run([locmetry, "--no-user-config", "--files", "--output", "-"],
                            cwd=REPOSITORY, capture_output=True, text=True)
    if result.returncode != 0:
        raise SystemExit(f"locmetry exited {result.returncode}: {result.stderr.strip()}")
    return json.loads(result.stdout)


def check(files, limits, ceilings):
    """Errors and warnings for source files whose purpose has limits.

    A ceiling replaces the hard limit for a file that predates the limits; it may only shrink, so a
    ceiling above the file's size, or one the file no longer needs, is reported for lowering or removal.
    """
    errors, warnings = [], []
    measured = set()
    for file in files:
        limit = limits.get(file["purpose"])
        if file["kind"] != "source" or limit is None:
            continue
        path, code, purpose = file["path"], file["counts"]["code"], file["purpose"]
        measured.add(path)
        ceiling = ceilings.get(path)
        if ceiling is None:
            if code > limit["hard"]:
                errors.append(f"{path}: {code} code lines exceed the {purpose} hard limit of {limit['hard']}")
            elif code > limit["soft"]:
                warnings.append(f"{path}: {code} code lines exceed the {purpose} soft limit of {limit['soft']}")
        elif code <= limit["hard"]:
            errors.append(f"{path}: fits the {purpose} hard limit of {limit['hard']}; remove its ceiling")
        elif code > ceiling:
            errors.append(f"{path}: {code} code lines exceed its ceiling of {ceiling}")
        elif code < ceiling:
            warnings.append(f"{path}: ceiling {ceiling} can be lowered to {code}")
    errors.extend(f"{path}: ceiling names no measured source file; remove it"
                  for path in sorted(set(ceilings) - measured))
    return errors, warnings


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--limits", default=Path(__file__).with_name("file-size-limits.json"), type=Path)
    parser.add_argument("--locmetry", default=REPOSITORY / "target/debug/locmetry", type=Path)
    parser.add_argument("--report", type=Path, help="a saved full report with --files, instead of measuring")
    args = parser.parse_args()
    policy = json.loads(args.limits.read_text())
    report = json.loads(args.report.read_text()) if args.report else measure(args.locmetry)
    errors, warnings = check(report["files"], policy["limits"], policy.get("ceilings", {}))
    for warning in warnings:
        print(f"warning: {warning}")
    for error in errors:
        print(f"error: {error}", file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main())
