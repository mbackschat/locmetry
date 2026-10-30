"""Prunes Cargo's target directory oldest-first once it exceeds a size cap.

Cargo never removes build units it no longer uses: every move of the pinned
Tokei revision leaves a complete set of dependent units, test binaries and
incremental state behind (31 GB of `target/`, 2026-10-30). Cargo has no stable
garbage collection for target directories, so this removes whole units, the
least recently written first, until the directory is under the floor. Cargo
rebuilds any unit whose outputs or fingerprint are missing, so a pruned unit
that is still needed costs one rebuild, not a broken build. A profile whose
`.cargo-lock` a running Cargo holds is left alone.

With `--hook` it runs as a Claude Code PostToolUse hook: it reads the event on
stdin, prunes only after a Bash command that ran Cargo, and reports what it
removed as a `systemMessage`.
"""

import argparse
import fcntl
import json
import os
from pathlib import Path
import shutil
import sys

GIB = 1024**3
UNIT_FOLDERS = ("deps", "build", "incremental", ".fingerprint", "examples")


def disk_usage(path):
    """Allocated bytes and newest file modification time of a file or tree.

    A directory's own time changes whenever an entry is added or removed, so
    only the files inside it date a unit.
    """
    if path.is_symlink() or not path.is_dir():
        info = path.lstat()
        return info.st_blocks * 512, info.st_mtime
    total, newest = 0, 0.0
    for root, _, files in os.walk(path):
        for name in files:
            info = os.lstat(os.path.join(root, name))
            total += info.st_blocks * 512
            newest = max(newest, info.st_mtime)
    return total, newest


def profiles(target):
    """Profile directories: `target/<profile>` and `target/<triple>/<profile>`."""
    for candidate in [*target.glob("*"), *target.glob("*/*")]:
        if candidate.is_dir() and any((candidate / folder).is_dir() for folder in UNIT_FOLDERS):
            yield candidate


def unlocked(profile, held):
    lock_path = profile / ".cargo-lock"
    if not lock_path.exists():
        return True
    lock = open(lock_path, "a")
    try:
        fcntl.flock(lock, fcntl.LOCK_EX | fcntl.LOCK_NB)
    except BlockingIOError:
        lock.close()
        return False
    held.append(lock)
    return True


def prune(target, max_bytes, floor_bytes):
    """Removes the oldest units when `target` exceeds `max_bytes`; returns the bytes freed."""
    total, _ = disk_usage(target) if target.is_dir() else (0, 0)
    if total <= max_bytes:
        return 0
    held = []
    try:
        units = []
        for profile in profiles(target):
            if not unlocked(profile, held):
                continue
            for folder in UNIT_FOLDERS:
                if (profile / folder).is_dir():
                    for unit in (profile / folder).iterdir():
                        size, newest = disk_usage(unit)
                        units.append((newest, size, unit))
        freed = 0
        for _, size, unit in sorted(units, key=lambda u: u[0]):
            if total - freed <= floor_bytes:
                break
            if unit.is_dir() and not unit.is_symlink():
                shutil.rmtree(unit)
            else:
                unit.unlink()
            freed += size
        return freed
    finally:
        for lock in held:
            lock.close()


def main():
    parser = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    parser.add_argument("--target", type=Path, default=Path(os.environ.get(
        "CARGO_TARGET_DIR", Path(__file__).resolve().parent.parent / "target")))
    parser.add_argument("--max-kib", type=int, default=10 * GIB // 1024)
    parser.add_argument("--floor-kib", type=int, default=6 * GIB // 1024)
    parser.add_argument("--hook", action="store_true")
    args = parser.parse_args()
    if args.hook:
        event = json.load(sys.stdin)
        if "cargo" not in event.get("tool_input", {}).get("command", ""):
            return
    freed = prune(args.target, args.max_kib * 1024, args.floor_kib * 1024)
    if freed and args.hook:
        print(json.dumps({"systemMessage": f"prune-target: pruned {freed / GIB:.1f} GiB of old build units from {args.target}"}))
    elif freed:
        print(f"pruned {freed / GIB:.1f} GiB from {args.target}")


if __name__ == "__main__":
    main()
