#!/usr/bin/env python3
"""Compact an archived agent campaign; procedure: CLAUDE.md, Disk hygiene.

A campaign keeps a copy of each arm's binary per pair and its session logs
uncompressed, which grew the archive to 592 MB (2026-10-24). The binaries are
reproducible from the source revision the campaign records, so this command
writes their SHA-256 to `binaries.sha256`, removes the `binaries` folders and
the arms' `target` trees, and gzips the `.jsonl` logs; read one with
`gzip -dc`. It only touches a folder inside the campaign archive that holds
the `campaign.json` of a campaign or the `manifest.json` of a single prepared
pair, and leaves other `target` folders, which may belong to a
task's seed.
"""

import argparse
import datetime
import gzip
import hashlib
import os
from pathlib import Path
import re
import shutil

REPOSITORY = Path(__file__).resolve().parent.parent
ARCHIVE = REPOSITORY / "docs/research/local/agent-explorability"
# prepare() in agent-explorability.py places each arm's build in ARM/target.
ARM = re.compile(r"^[A-Z]$")


def build_trees(campaign):
    found = [p for p in campaign.rglob("binaries") if p.is_dir() and not p.is_symlink()]
    found += [p for p in campaign.rglob("target") if p.is_dir() and not p.is_symlink() and ARM.match(p.parent.name)]
    return sorted(found)


def digest(path):
    hasher = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            hasher.update(block)
    return hasher.hexdigest()


def compact(campaign):
    """Removes the build trees after recording their files and compresses the logs; returns what changed."""
    trees = build_trees(campaign)
    lines = [f"{digest(file)}  {file.relative_to(campaign)}"
             for tree in trees for file in sorted(tree.rglob("*")) if file.is_file() and not file.is_symlink()]
    if lines:
        manifest = campaign / "binaries.sha256"
        header = [] if manifest.exists() else [
            f"# Binaries removed on {datetime.date.today()} to save space; the campaign records the source revision each was built from."]
        with manifest.open("a") as handle:
            handle.write("".join(line + "\n" for line in header + lines))
    for tree in trees:
        shutil.rmtree(tree)
    logs = [p for p in campaign.rglob("*.jsonl") if p.is_file() and not p.is_symlink()]
    for log in logs:
        packed = log.with_name(log.name + ".gz")
        with log.open("rb") as source, gzip.GzipFile(packed, "wb", mtime=log.stat().st_mtime) as target:
            shutil.copyfileobj(source, target)
        status = log.stat()
        os.utime(packed, (status.st_atime, status.st_mtime))
        log.unlink()
    return len(trees), len(logs)


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("campaigns", type=Path, nargs="+", metavar="CAMPAIGN")
    parser.add_argument("--archive-root", type=Path, default=ARCHIVE)
    args = parser.parse_args()
    archive = args.archive_root.resolve()
    for campaign in args.campaigns:
        campaign = campaign.resolve()
        if archive not in campaign.parents:
            raise SystemExit(f"compact-campaign: {campaign} is not inside the campaign archive {archive}")
        records = [p for name in ["campaign.json", "manifest.json"] for p in [*campaign.glob(name), *campaign.glob(f"*/{name}")]]
        if not records:
            raise SystemExit(f"compact-campaign: {campaign} holds no campaign.json or manifest.json, so it is not a campaign")
        trees, logs = compact(campaign)
        if trees or logs:
            print(f"compact-campaign: {campaign.name}: removed {trees} build trees, compressed {logs} logs")
        else:
            print(f"compact-campaign: {campaign.name}: nothing to compact")


if __name__ == "__main__":
    main()
