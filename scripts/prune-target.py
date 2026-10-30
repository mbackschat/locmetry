"""Prunes the build units of Cargo's target directory that no current build uses.

Cargo never removes build units it no longer uses: every move of the pinned
Tokei revision leaves a complete set of dependent units, test binaries and
incremental state behind (31 GB of `target/`, 2026-10-30), and Cargo has no
stable garbage collection for target directories. A newer build of a local
package with the same kind, profile, features and flags supersedes the older
one; this removes superseded units and every unit only they reach, whatever
the directory's size. As a backstop it then removes whole units, the least
recently used first, while the directory exceeds a size cap. Cargo rebuilds
any unit whose outputs or fingerprint are missing, so a pruned unit that is
still needed costs one rebuild, not a broken build. A profile whose
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
import re
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


def allocated(info):
    """A file's share of its allocated bytes: Cargo hard-links each built
    binary from `deps/` into the profile, and the bytes are counted once
    across the links."""
    return info.st_blocks * 512 // info.st_nlink


def disk_usage(path):
    """Allocated bytes and latest file use of a file or tree.

    A directory's own times change whenever an entry is added or removed, so
    only the files inside it date a unit.
    """
    if path.is_symlink() or not path.is_dir():
        info = path.lstat()
        return allocated(info), last_use(info)
    total, newest = 0, 0.0
    for root, _, files in os.walk(path):
        for name in files:
            info = os.lstat(os.path.join(root, name))
            total += allocated(info)
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


UNIT_HASH = re.compile(r"-([0-9a-f]{16})(?:\.|$)")


def remove(path):
    if path.is_dir() and not path.is_symlink():
        shutil.rmtree(path)
    else:
        path.unlink()


def read_units(profile):
    """Each unit's fingerprints under `.fingerprint/<package>-<hash>/`: the
    `<kind>-<target>` file holds the fingerprint value that dependents'
    `deps` entries name, its `.json` the inputs, and `invoked.timestamp` dates
    the compile's start. Cargo empties the value when a compile starts and
    writes it once the compile succeeds (checked with Cargo 1.98 on
    2026-10-30), so an empty value marks a compile that has not finished and
    reads as `None`. A directory without fingerprints yet is `None`."""
    units = {}
    for directory in (profile / ".fingerprint").iterdir():
        unit_hash = directory.name.rsplit("-", 1)[-1]
        stamp = directory / "invoked.timestamp"
        started = stamp.stat().st_mtime if stamp.exists() else 0.0
        fingerprints = []
        for record in directory.glob("*.json"):
            value_file = record.with_suffix("")
            try:
                fingerprint = json.loads(record.read_text())
                # Cargo writes the value's little-endian bytes in hex (`to_hex`
                # in cargo's `util/hex.rs`), while `deps` gives it as an integer.
                text = value_file.read_text().strip()
                value = int.from_bytes(bytes.fromhex(text), "little") if text else None
                built = value_file.stat().st_mtime
            except (OSError, ValueError):
                continue
            fingerprints.append((value_file.name, fingerprint, value, built))
        units[unit_hash] = (directory.name.rsplit("-", 1)[0], started, fingerprints) if fingerprints else None
    return units


def current_units(units):
    """Hashes of the units the newest build of each local package reaches.

    Local packages, whose fingerprints check their sources' dep-info, root the
    builds; of their units with the same kind, profile, features, target
    platform and flags only the latest started is current. The toolchain is
    left out of that identity, so an upgrade supersedes the old builds.
    """
    by_value, newest = {}, {}
    for unit_hash, unit in units.items():
        if unit is None:
            continue
        name, started, fingerprints = unit
        for kind, fingerprint, value, _ in fingerprints:
            if value is not None:
                by_value[value] = unit_hash
            if any("CheckDepInfo" in local for local in fingerprint.get("local", [])):
                identity = (name, kind, fingerprint.get("profile"), fingerprint.get("features"),
                            fingerprint.get("compile_kind"), json.dumps(fingerprint.get("rustflags")))
                if identity not in newest or started > newest[identity][0]:
                    newest[identity] = (started, unit_hash)
    current = {unit_hash for unit_hash, unit in units.items() if unit is None}
    pending = [unit_hash for _, unit_hash in newest.values()]
    while pending:
        unit_hash = pending.pop()
        if unit_hash in current:
            continue
        current.add(unit_hash)
        for _, fingerprint, _, _ in units[unit_hash][2]:
            pending.extend(by_value[dep[3]] for dep in fingerprint.get("deps", []) if dep[3] in by_value)
    return current


def compile_windows(profile, units, current):
    """When each current unit's crate last compiled, by crate name.

    rustc names incremental directories after the crate and a hash Cargo does
    not record, so a session is matched by time: it ends between the
    compile's start and its fingerprint value's write, or after the start of
    a compile that has not finished.
    """
    windows = {}
    for output in (profile / "deps").glob("*.d") if (profile / "deps").is_dir() else []:
        crate, _, unit_hash = output.stem.rpartition("-")
        if unit_hash in current and units.get(unit_hash):
            _, started, fingerprints = units[unit_hash]
            finished = all(f[2] is not None for f in fingerprints)
            built = max(f[3] for f in fingerprints) if finished else float("inf")
            windows.setdefault(crate, []).append((started - 2, built + 2))
    return windows


def prune_superseded(profile):
    """Removes the units of `profile` no current build reaches."""
    if not (profile / ".fingerprint").is_dir():
        return
    units = read_units(profile)
    current = current_units(units)
    if not current:
        return
    windows = compile_windows(profile, units, current)
    for folder in ("deps", "build", "examples", ".fingerprint"):
        for entry in (profile / folder).iterdir() if (profile / folder).is_dir() else []:
            match = UNIT_HASH.search(entry.name)
            if match and match.group(1) not in current:
                remove(entry)
    for session in (profile / "incremental").iterdir() if (profile / "incremental").is_dir() else []:
        ended = session.stat().st_mtime
        crate = session.name.rpartition("-")[0]
        if not any(start <= ended <= end for start, end in windows.get(crate, [])):
            remove(session)


def prune(target, max_bytes, floor_bytes):
    """Removes superseded units, then the oldest units while `target` exceeds
    `max_bytes`; returns the bytes each step freed."""
    if not target.is_dir():
        return 0, 0
    held = []
    try:
        before, _ = disk_usage(target)
        open_profiles = [profile for profile in profiles(target) if unlocked(profile, held)]
        for profile in open_profiles:
            prune_superseded(profile)
        total, _ = disk_usage(target)
        if total <= max_bytes:
            return before - total, 0
        units = []
        for profile in open_profiles:
            for folder in UNIT_FOLDERS:
                if (profile / folder).is_dir():
                    for unit in (profile / folder).iterdir():
                        size, newest = disk_usage(unit)
                        units.append((newest, size, unit))
        freed = 0
        for _, size, unit in sorted(units, key=lambda u: u[0]):
            if total - freed <= floor_bytes:
                break
            remove(unit)
            freed += size
        return before - total, total - disk_usage(target)[0]
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
            elif "--manifest-path" in options:
                manifest = options["--manifest-path"]
                target = manifest and project_target(manifest.parent)
            else:
                target = project_target(here)
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
        # Tokenizing takes about a millisecond per KiB of command (measured
        # 2026-10-30), so commands that cannot run Cargo skip it.
        if "cargo" not in command:
            return
        targets = cargo_targets(command, event.get("cwd") or os.getcwd())
    else:
        targets = [args.target]
    messages = []
    for target in targets:
        superseded, oldest = prune(target, args.max_kib * 1024, args.floor_kib * 1024)
        if superseded:
            messages.append(f"pruned {superseded / GIB:.2f} GiB of superseded build units from {target}")
        if oldest:
            messages.append(f"pruned {oldest / GIB:.1f} GiB of old build units from {target}")
    if messages and args.hook:
        print(json.dumps({"systemMessage": "prune-target: " + "; ".join(messages)}))
    elif messages:
        print("\n".join(messages))


if __name__ == "__main__":
    main()
