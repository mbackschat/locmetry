#!/usr/bin/env python3
"""Keep only the current build in Cargo's target directory; procedure: CLAUDE.md, Disk hygiene.

Cargo never deletes artifacts it no longer uses: every dependency update,
version bump or profile change leaves a full copy of the earlier build, which
grew this repository's target directory to 54 GB (2026-10-24) against 2.6 GB
for one fresh build, test and Clippy run. This command runs those builds,
which are no-ops when nothing changed, takes the artifacts they report as
current, and removes the rest. If the directory still exceeds the limit, it
runs `cargo clean`. It only trims a directory that Cargo tagged as its
target, and refuses one shared through CARGO_TARGET_DIR, whose other
projects' builds this repository's build does not list.
"""

import argparse
from collections import defaultdict
import contextlib
import fcntl
import filecmp
import json
import os
from pathlib import Path
import re
import shutil
import subprocess

REPOSITORY = Path(__file__).resolve().parent.parent
GIB = 1 << 30
# The cargo commands of CLAUDE.md that build into each profile directory.
PROFILES = {
    "debug": [["build", "--locked"], ["test", "--locked", "--no-run"], ["clippy", "--locked", "--all-targets"]],
    "release": [["build", "--release", "--locked"]],
}
# Cargo names a unit's files and directories after its 16-digit metadata hash.
HASH = re.compile(r"-([0-9a-f]{16})(?=[./]|$)")
HASHED_ENTRY = re.compile(r"^(?:lib)?[\w-]+?-([0-9a-f]{16})(?:\.|$)")


def size(path):
    """Disk usage counting each hard-linked file once, as `du` does."""
    seen, total = set(), 0
    for directory, _, files in os.walk(path):
        for name in files:
            status = os.lstat(os.path.join(directory, name))
            if (status.st_dev, status.st_ino) not in seen:
                seen.add((status.st_dev, status.st_ino))
                total += status.st_blocks * 512
    return total


def uplifted(profile, path):
    """The hashes of the deps entries Cargo copied or linked a hashless file from.

    Cargo reports a binary of the build profile only by its uplifted path,
    target/debug/NAME, a hard link or, on APFS, a copy of deps/NAME-HASH
    (2026-10-24), so the entry with the same content is the unit's. When none
    has, every same-named entry counts as current, as keeping too much costs
    space while removing too much costs a rebuild.
    """
    candidates = [(entry, HASHED_ENTRY.match(entry.name)) for entry in (profile / "deps").glob(f"{path.stem}-*{path.suffix}")]
    candidates = [(entry, match.group(1)) for entry, match in candidates if match]
    if path.exists():
        same = {h for entry, h in candidates if entry.is_file() and filecmp.cmp(entry, path, shallow=False)}
        if same:
            return same
    return {h for _, h in candidates}


def current_units(target, profile, commands):
    """The metadata hashes of the units the commands build and, per crate name, the local units.

    Returns None when a command fails: a failed build reports only the units it
    reached, and pruning by it would delete current artifacts.
    """
    hashes, local = set(), defaultdict(set)
    for command in commands:
        result = subprocess.run(["cargo", *command, "--message-format=json"], cwd=REPOSITORY,
                                env=dict(os.environ, CARGO_TARGET_DIR=str(target)),
                                stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, text=True)
        if result.returncode != 0:
            return None
        for line in result.stdout.splitlines():
            message = json.loads(line) if line.startswith("{") else {}
            if message.get("reason") == "compiler-artifact":
                found = set()
                for name in message["filenames"]:
                    found |= set(HASH.findall(name)) or uplifted(profile, Path(name))
                hashes |= found
                if message["package_id"].startswith("path+"):
                    local[message["target"]["name"].replace("-", "_")] |= found
            elif message.get("reason") == "build-script-executed":
                hashes |= set(HASH.findall(message["out_dir"]))
    return hashes, {crate: len(units) for crate, units in local.items()}


@contextlib.contextmanager
def cargo_lock(profile):
    """Holds the lock cargo takes on a profile directory, so no build writes while entries go."""
    with (profile / ".cargo-lock").open("a") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        yield


def remove(path):
    if path.is_dir() and not path.is_symlink():
        shutil.rmtree(path)
    else:
        path.unlink()


def prune(profile, hashes, local_units):
    """Removes the entries of units outside the current build and returns how many went."""
    removed = 0
    for folder in ["deps", ".fingerprint", "build"]:
        for entry in (profile / folder).glob("*"):
            match = HASHED_ENTRY.match(entry.name)
            if match and match.group(1) not in hashes:
                remove(entry)
                removed += 1
    return removed + prune_incremental(profile / "incremental", local_units)


def prune_incremental(incremental, local_units):
    """Keeps the newest session of as many crate directories as the crate has current units.

    rustc names these directories after a stable crate id that no build message
    reports. Every local unit has its own directory, and a current one was used
    after its unit's metadata last changed, so after any earlier directory of
    the same crate.
    """
    removed = 0
    crates = defaultdict(list)
    for crate_dir in incremental.glob("*"):
        sessions = sorted((s for s in crate_dir.glob("s-*") if s.is_dir()), key=lambda s: s.stat().st_mtime)
        for session in sessions[:-1]:
            shutil.rmtree(session)
            session.with_name(session.name + ".lock").unlink(missing_ok=True)
            removed += 1
        newest = sessions[-1].stat().st_mtime if sessions else 0
        crates[crate_dir.name.rsplit("-", 1)[0]].append((newest, crate_dir))
    for crate, dirs in crates.items():
        dirs.sort(reverse=True)
        for _, crate_dir in dirs[local_units.get(crate, 0):]:
            shutil.rmtree(crate_dir)
            removed += 1
    return removed


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--limit", type=float, default=6, metavar="GIB",
                        help="run cargo clean when the trimmed directory exceeds this size (default: 6)")
    parser.add_argument("--target-dir", type=Path, default=REPOSITORY / "target")
    args = parser.parse_args()
    if "CARGO_TARGET_DIR" in os.environ:
        raise SystemExit("trim-target: CARGO_TARGET_DIR is set; a shared target directory holds other projects' builds")
    target = args.target_dir.resolve()
    if not target.is_dir():
        print(f"trim-target: no target directory at {target}")
        return
    if not (target / "CACHEDIR.TAG").is_file():
        raise SystemExit(f"trim-target: {target} is not a Cargo target directory (no CACHEDIR.TAG)")
    before = size(target)
    removed = 0
    for name, commands in PROFILES.items():
        profile = target / name
        if not profile.is_dir():
            continue
        units = current_units(target, profile, commands)
        if units is None:
            print(f"trim-target: a build failed, so left {name} as it is")
            continue
        with cargo_lock(profile):
            removed += prune(profile, *units)
    after = size(target)
    print(f"trim-target: {before / GIB:.1f} GiB before, {after / GIB:.1f} GiB after; "
          f"removed {removed} entries of earlier builds")
    if after > args.limit * GIB:
        print(f"trim-target: still above the {args.limit:g} GiB limit, so running cargo clean")
        subprocess.run(["cargo", "clean"], cwd=REPOSITORY, env=dict(os.environ, CARGO_TARGET_DIR=str(target)), check=True)


if __name__ == "__main__":
    main()
