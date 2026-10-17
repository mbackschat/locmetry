"""Prepare a release; --publish also commits, tags and pushes after verification."""

import argparse
from pathlib import Path
import re
import subprocess
import sys

VERSION_FILES = ("Cargo.toml", "Cargo.lock", "README.md")
ORIGINS = ("https://github.com/mbackschat/locmetry.git", "git@github.com:mbackschat/locmetry.git")
VERSION = r"(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)\.(?:0|[1-9][0-9]*)"


class ReleaseError(Exception):
    pass


def run(root, *args, capture=False):
    result = subprocess.run(args, cwd=root, check=True, text=True, stdout=subprocess.PIPE if capture else None)
    return result.stdout.strip() if capture else None


def git(root, *args):
    return run(root, "git", *args, capture=True)


def version_number(version):
    if not re.fullmatch(VERSION, version):
        raise ReleaseError("Use a stable version such as 0.4.1 (without v).")
    return tuple(map(int, version.split(".")))


def crate_version(text):
    match = re.search(r'(?ms)^\[package\]\n(?:(?!^\[).)*?^version = "([^"]+)"$', text)
    if not match:
        raise ReleaseError("Cannot find the crate version in Cargo.toml.")
    return match.group(1)


def release_files(original, version):
    current = crate_version(original["Cargo.toml"])
    if version_number(version) < version_number(current):
        raise ReleaseError(f"Refusing downgrade from {current} to {version}.")
    updated = dict(original)
    updated["Cargo.toml"] = original["Cargo.toml"].replace(f'version = "{current}"', f'version = "{version}"', 1)
    lock_marker = f'name = "locmetry"\nversion = "{current}"'
    if original["Cargo.lock"].count(lock_marker) != 1:
        raise ReleaseError("Cargo.lock does not match the crate version.")
    updated["Cargo.lock"] = original["Cargo.lock"].replace(lock_marker, f'name = "locmetry"\nversion = "{version}"')
    readme = original["README.md"]
    markers = (
        f"The latest stable release is **{current}**",
        f"### Stable release (v{current})",
        f"/download/v{current}/",
        f"locmetry-v{current}-$target",
        f"--tag v{current}",
        f"This checkout identifies as `{current}`",
        f'"locmetry_version": "{current}"',
    )
    for marker in markers:
        if marker not in readme:
            raise ReleaseError(f"README release marker missing: {marker}")
        readme = readme.replace(marker, marker.replace(current, version))
    updated["README.md"] = readme
    return updated


def prepare(root, version):
    original = {name: (root / name).read_text() for name in VERSION_FILES}
    updated = release_files(original, version)
    for name, contents in updated.items():
        if contents != original[name]:
            (root / name).write_text(contents)


def check_worktree(root, version):
    changed = set(git(root, "diff", "HEAD", "--name-only").splitlines())
    untracked = git(root, "ls-files", "--others", "--exclude-standard")
    if untracked or changed - set(VERSION_FILES):
        raise ReleaseError("Commit unrelated changes before releasing; untracked files are not included.")
    if changed:
        original = {name: subprocess.check_output(["git", "show", f"HEAD:{name}"], cwd=root, text=True) for name in VERSION_FILES}
        expected = release_files(original, version)
        if git(root, "diff", "HEAD", "--summary") or any((root / name).read_text() != expected[name] for name in VERSION_FILES):
            raise ReleaseError("Dirty release files contain edits beyond the requested version update.")


def check_publisher(root):
    if git(root, "branch", "--show-current") != "main":
        raise ReleaseError("Release from main only.")
    if git(root, "remote", "get-url", "--all", "origin") not in ORIGINS or git(root, "remote", "get-url", "--push", "--all", "origin") not in ORIGINS:
        raise ReleaseError("origin must fetch from and push to mbackschat/locmetry on GitHub.")
    if git(root, "config", "user.email") != "mbackschat@mail.de":
        raise ReleaseError("GitHub releases require user.email=mbackschat@mail.de.")


def check_local_tag(root, version):
    tag = f"v{version}"
    if git(root, "tag", "--list", tag):
        if git(root, "rev-parse", f"{tag}^{{commit}}") != git(root, "rev-parse", "HEAD") or crate_version((root / "Cargo.toml").read_text()) != version:
            raise ReleaseError(f"Local {tag} conflicts with this release; it will not be moved.")


def check_remote(root, version):
    tag = f"refs/tags/v{version}"
    refs = dict(line.split()[::-1] for line in git(root, "ls-remote", "origin", tag, tag + "^{}").splitlines())
    remote_commit = refs.get(tag + "^{}", refs.get(tag))
    if remote_commit:
        if remote_commit != git(root, "rev-parse", "HEAD") or git(root, "status", "--porcelain") or crate_version((root / "Cargo.toml").read_text()) != version:
            raise ReleaseError(f"Remote v{version} already exists at a different release state.")
        return True
    run(root, "git", "fetch", "origin", "main")
    try:
        git(root, "merge-base", "--is-ancestor", "FETCH_HEAD", "HEAD")
    except subprocess.CalledProcessError as error:
        raise ReleaseError("Remote main has changes not present locally; integrate them first.") from error
    return False


def verify(root, version):
    commands = (
        (sys.executable, "scripts/test_release.py"),
        (sys.executable, "scripts/test_doc_indexes.py"),
        (sys.executable, "scripts/doc-indexes.py", "check"),
        ("cargo", "fmt", "--all", "--check"),
        ("cargo", "test", "--locked"),
        ("cargo", "clippy", "--locked", "--all-targets", "--", "-D", "warnings"),
        ("cargo", "build", "--release", "--locked"),
        ("bash", "scripts/check-doc-examples.sh"),
        ("git", "diff", "--check"),
    )
    for command in commands:
        print("+ " + " ".join(command), flush=True)
        run(root, *command)
    if run(root, "target/release/locmetry", "--version", capture=True) != f"locmetry {version}":
        raise ReleaseError("Built executable reports an unexpected version.")


def push(root, version):
    run(root, "git", "push", "--atomic", "origin", "HEAD:refs/heads/main", f"refs/tags/v{version}")


def execute(root, version, publish):
    version_number(version)
    check_worktree(root, version)
    if publish:
        check_publisher(root)
        check_local_tag(root, version)
        if check_remote(root, version):
            print(f"v{version} is already pushed. Check the Release workflow; no new tag was created.")
            return
    prepare(root, version)
    checked_head = git(root, "rev-parse", "HEAD")
    checked_files = {name: (root / name).read_bytes() for name in VERSION_FILES}
    verify(root, version)
    if git(root, "rev-parse", "HEAD") != checked_head or any((root / name).read_bytes() != contents for name, contents in checked_files.items()):
        raise ReleaseError("Release inputs changed during verification; rerun the command.")
    check_worktree(root, version)
    if not publish:
        print("Preparation verified. Review the diff; rerun with --publish to commit, tag and push.")
        return
    check_publisher(root)
    if git(root, "diff", "HEAD", "--name-only"):
        run(root, "git", "add", "--", *VERSION_FILES)
        run(root, "git", "commit", "-m", f"chore(release): prepare v{version}")
    check_local_tag(root, version)
    if not git(root, "tag", "--list", f"v{version}"):
        run(root, "git", "tag", "-a", f"v{version}", "-m", f"Release v{version}")
    push(root, version)
    print("Release requested. Publication is complete only when the Release workflow succeeds:")
    print("https://github.com/mbackschat/locmetry/actions/workflows/release.yml")
    print(f"Then update Homebrew with packaging/homebrew/render-formula.sh {version}.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("version", help="stable version without v, e.g. 0.4.1")
    parser.add_argument("--publish", action="store_true", help="commit, tag and atomically push main and the tag after all checks pass")
    args = parser.parse_args()
    try:
        execute(Path(__file__).resolve().parent.parent, args.version, args.publish)
    except (ReleaseError, subprocess.CalledProcessError, OSError) as error:
        parser.exit(1, f"Release stopped: {error}\nFix the cause and rerun the same command; existing tags are never moved.\n")


if __name__ == "__main__":
    main()
