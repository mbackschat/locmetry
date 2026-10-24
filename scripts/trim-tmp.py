#!/usr/bin/env python3
"""Keep only the newest of Locmetry's temporary trees; procedure: CLAUDE.md, Disk hygiene.

Revision builds, Showboat artifacts and campaign working copies go below the
temporary directories as `locmetry-*`, and Claude Code keeps a scratch folder
per session of this repository; none is removed by the tool that made it,
which let them grow past 18 GB (2026-10-24). This command removes all but the
newest of each. It only ever removes such a child of a temporary directory,
owned by the current user and not a symbolic link, and never one changed
within the last hour, which another session may still be using. It keeps
subject homes, whose credentials the campaign runner writes back when it
retires them, and campaigns, which hold evidence until they are archived.
"""

import argparse
import os
from pathlib import Path
import re
import shutil
import sys
import tempfile
import time

REPOSITORY = Path(__file__).resolve().parent.parent
PREFIX = "locmetry-"
SUBJECT_HOMES = "locmetry-subject-homes"
SESSION = re.compile(r"^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$")
GRACE = 3600


def last_change(path):
    """The newest modification time anywhere in the tree, without following links."""
    newest = path.lstat().st_mtime
    for directory, dirs, files in os.walk(path):
        for name in dirs + files:
            newest = max(newest, os.lstat(os.path.join(directory, name)).st_mtime)
    return newest


def is_campaign(path):
    return any(path.glob("campaign.json")) or any(p.name == "evidence" for p in path.glob("**/evidence") if p.is_dir())


def trim(root, owned, protected=lambda path: False):
    """Removes all but the newest of the root's children that `owned` accepts, and returns how many went."""
    if not root.is_dir() or root.is_symlink():
        return 0
    root = root.resolve()
    candidates = []
    for child in root.iterdir():
        if child.is_symlink() or not child.is_dir() or not owned(child.name):
            continue
        if child.lstat().st_uid != os.getuid() or child.resolve().parent != root:
            continue
        if protected(child):
            continue
        candidates.append((last_change(child), child))
    candidates.sort(reverse=True)
    removed = 0
    for changed, child in candidates[1:]:
        if changed < time.time() - GRACE:
            shutil.rmtree(child)
            removed += 1
    return removed


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--temp-root", type=Path, action="append",
                        help="a temporary directory to trim (default: /tmp and $TMPDIR)")
    parser.add_argument("--sessions", type=Path,
                        default=Path(f"/tmp/claude-{os.getuid()}") / re.sub(r"[^A-Za-z0-9]", "-", str(REPOSITORY)),
                        help="Claude Code's scratch folder of this repository")
    args = parser.parse_args()
    roots = {path.resolve() for path in args.temp_root or [Path("/tmp"), Path(tempfile.gettempdir())]}

    def unarchived(path):
        if path.name == SUBJECT_HOMES:
            return True
        if is_campaign(path):
            print(f"trim-tmp: kept an unarchived campaign, archive it first: {path}")
            return True
        return False

    removed = sum(trim(root, lambda name: name.startswith(PREFIX), unarchived) for root in sorted(roots))
    removed += trim(args.sessions, SESSION.match)
    print(f"trim-tmp: removed {removed} older temporary trees")


if __name__ == "__main__":
    sys.exit(main())
