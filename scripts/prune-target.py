"""Prunes Cargo's target directory oldest-first once it exceeds a size cap.

Cargo never removes build units it no longer uses: every move of the pinned
Tokei revision leaves a complete set of dependent units, test binaries and
incremental state behind (31 GB of `target/`, 2026-10-30). Cargo has no stable
garbage collection for target directories, so this removes whole units, the
least recently used first, until the directory is under the floor. Cargo
rebuilds any unit whose outputs or fingerprint are missing, so a pruned unit
that is still needed costs one rebuild, not a broken build. A profile whose
`.cargo-lock` a running Cargo holds is left alone.

With `--hook` it runs as a Claude Code PostToolUse hook: it reads the event on
stdin and, after a Bash command that ran Cargo, prunes the target directory of
each project Cargo ran in, following `cd`, `--manifest-path` and
`--target-dir`, since commands also build sibling checkouts such as the Tokei
fork; it reports what it removed as a `systemMessage`.
"""

import argparse
import fcntl
import json
import os
from pathlib import Path
import shlex
import shutil
import sys

GIB = 1024**3
UNIT_FOLDERS = ("deps", "build", "incremental", ".fingerprint", "examples")


def last_use(info):
    """When a file was last read or written.

    Cargo reads the fingerprints and libraries of every unit a build still
    uses, so a dependency built once and read since is in use although its
    write time is old. macOS updates access times lazily, at most once a day
    after a file was last accessed (checked 2026-10-30), so the order is exact
    only to a day.
    """
    return max(info.st_atime, info.st_mtime)


def disk_usage(path):
    """Allocated bytes and latest file use of a file or tree.

    A directory's own times change whenever an entry is added or removed, so
    only the files inside it date a unit.
    """
    if path.is_symlink() or not path.is_dir():
        info = path.lstat()
        return info.st_blocks * 512, last_use(info)
    total, newest = 0, 0.0
    for root, _, files in os.walk(path):
        for name in files:
            info = os.lstat(os.path.join(root, name))
            total += info.st_blocks * 512
            newest = max(newest, last_use(info))
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


def cargo_targets(command, cwd):
    """The target directories of the Cargo invocations in a shell `command`
    started in `cwd`.

    Follows `cd` and the `--manifest-path` and `--target-dir` options; a path
    that names a shell variable not set here is skipped, as its directory is
    unknown.
    """
    lexer = shlex.shlex(command, posix=True, punctuation_chars=True)
    lexer.whitespace_split = True
    try:
        tokens = list(lexer)
    except ValueError:
        return []
    targets = []
    here = Path(cwd)
    i = 0
    while i < len(tokens):
        token = tokens[i]
        if token == "cd" and i + 1 < len(tokens):
            here = resolve(here, tokens[i + 1]) or here
        elif os.path.basename(token) == "cargo":
            options = {}
            j = i + 1
            while j < len(tokens) and tokens[j] not in ("&&", "||", ";", "|", "&"):
                name, _, value = tokens[j].partition("=")
                if name in ("--manifest-path", "--target-dir"):
                    if not value and j + 1 < len(tokens):
                        j += 1
                        value = tokens[j]
                    options[name] = resolve(here, value)
                j += 1
            if "--target-dir" in options:
                target = options["--target-dir"]
            elif os.environ.get("CARGO_TARGET_DIR"):
                target = resolve(here, os.environ["CARGO_TARGET_DIR"])
            else:
                manifest = options.get("--manifest-path")
                target = project_target(manifest.parent if manifest else here)
            if target and target not in targets:
                targets.append(target)
            i = j
            continue
        i += 1
    return targets


def resolve(base, path):
    expanded = os.path.expandvars(os.path.expanduser(path))
    return None if "$" in expanded else (base / expanded).resolve()


def project_target(directory):
    """The `target` beside the nearest `Cargo.toml` at or above `directory`
    that has one, as a workspace member builds into its root's."""
    if directory is None:
        return None
    for candidate in [directory, *directory.parents]:
        if (candidate / "Cargo.toml").is_file() and (candidate / "target").is_dir():
            return candidate / "target"
    return None


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
        command = event.get("tool_input", {}).get("command", "")
        targets = cargo_targets(command, event.get("cwd") or os.getcwd())
    else:
        targets = [args.target]
    messages = []
    for target in targets:
        freed = prune(target, args.max_kib * 1024, args.floor_kib * 1024)
        if freed:
            messages.append(f"pruned {freed / GIB:.1f} GiB of old build units from {target}")
    if messages and args.hook:
        print(json.dumps({"systemMessage": "prune-target: " + "; ".join(messages)}))
    elif messages:
        print("\n".join(messages))


if __name__ == "__main__":
    main()
