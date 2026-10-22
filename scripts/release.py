"""Prepare a release; --publish also commits, tags and pushes after verification."""

import argparse
import datetime
from pathlib import Path
import re
import subprocess
import sys

# The root command guide records `--version` and guide-source output, which
# name the executable version; the CI replay rejects it after any bump.
RECORDING = "docs/cli/root.md"
CHANGELOG = "CHANGELOG.md"
VERSION_FILES = ("Cargo.toml", "Cargo.lock", RECORDING, CHANGELOG)
COMPARE = "https://github.com/mbackschat/locmetry/compare/"
# The pinned replay of the command guides, as the CI documentation job runs it.
REPLAY = ("uv", "run", "--no-project", "--python", "3.11", "--with", "showboat==0.6.1",
          "python", "scripts/check-recorded-guides.py", "--set", "reference")
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


def release_date(changelog, version, today):
    """The date a preparation already recorded for `version`, else `today`."""
    match = re.search(rf"(?m)^## \[{re.escape(version)}\] - (\d{{4}}-\d{{2}}-\d{{2}})$", changelog)
    return match.group(1) if match else today


def release_changelog(text, current, version, date):
    """Moves the Unreleased entries under `version` and extends the compare links.

    Keep a Changelog keeps an empty Unreleased heading and one link per version
    (https://keepachangelog.com/en/1.1.0/). Already prepared text is unchanged.
    """
    if re.search(rf"(?m)^## \[{re.escape(version)}\] - ", text):
        return text
    heading = "## [Unreleased]\n"
    if text.count(heading) != 1:
        raise ReleaseError(f"{CHANGELOG} needs one '## [Unreleased]' section.")
    unreleased = text.split(heading, 1)[1].split("\n## [", 1)[0]
    if not re.search(r"(?m)^- ", unreleased):
        raise ReleaseError(f"{CHANGELOG} lists no Unreleased changes for {version}.")
    link = f"[Unreleased]: {COMPARE}v{current}...HEAD\n"
    if text.count(link) != 1:
        raise ReleaseError(f"{CHANGELOG} lacks the link {link.strip()}")
    text = text.replace(heading, f"{heading}\n## [{version}] - {date}\n", 1)
    return text.replace(link, f"[Unreleased]: {COMPARE}v{version}...HEAD\n[{version}]: {COMPARE}v{current}...v{version}\n")


def release_notes(changelog, version):
    """The entries of `version`, as the GitHub release text."""
    parts = re.split(rf"(?m)^## \[{re.escape(version)}\] - .*\n", changelog)
    if len(parts) != 2:
        raise ReleaseError(f"{CHANGELOG} has no section for {version}.")
    notes = re.split(r"(?m)^## \[|^\[Unreleased\]: ", parts[1])[0].strip() + "\n"
    # A release page resolves relative links against /releases/tag/, so pin them to the tag.
    return re.sub(r"\]\((?![a-z]+:|#)", f"](https://github.com/mbackschat/locmetry/blob/v{version}/", notes)


def release_files(original, version, today):
    current = crate_version(original["Cargo.toml"])
    if version_number(version) < version_number(current):
        raise ReleaseError(f"Refusing downgrade from {current} to {version}.")
    updated = dict(original)
    updated["Cargo.toml"] = original["Cargo.toml"].replace(f'version = "{current}"', f'version = "{version}"', 1)
    lock_marker = f'name = "locmetry"\nversion = "{current}"'
    if original["Cargo.lock"].count(lock_marker) != 1:
        raise ReleaseError("Cargo.lock does not match the crate version.")
    updated["Cargo.lock"] = original["Cargo.lock"].replace(lock_marker, f'name = "locmetry"\nversion = "{version}"')
    # A recorded version is a whole output line or a guide source's `(locmetry X)` tail.
    recorded = re.compile(rf"(?m)^(locmetry |Source: .* \(locmetry ){re.escape(current)}(\)?)$")
    updated[RECORDING], count = recorded.subn(rf"\g<1>{version}\g<2>", original[RECORDING])
    if not count:
        raise ReleaseError(f"{RECORDING} records no locmetry {current} output to update.")
    date = release_date(original[CHANGELOG], version, today)
    updated[CHANGELOG] = release_changelog(original[CHANGELOG], current, version, date)
    return updated


def prepare(root, version, today=None):
    original = {name: (root / name).read_text() for name in VERSION_FILES}
    updated = release_files(original, version, today or datetime.date.today().isoformat())
    for name, contents in updated.items():
        if contents != original[name]:
            (root / name).write_text(contents)


def check_worktree(root, version, today=None):
    changed = set(git(root, "diff", "HEAD", "--name-only").splitlines())
    untracked = git(root, "ls-files", "--others", "--exclude-standard")
    if untracked or changed - set(VERSION_FILES):
        raise ReleaseError("Commit unrelated changes before releasing; untracked files are not included.")
    if changed:
        original = {name: subprocess.check_output(["git", "show", f"HEAD:{name}"], cwd=root, text=True) for name in VERSION_FILES}
        # The date comes from the prepared changelog, so a resumed release matches.
        date = release_date((root / CHANGELOG).read_text(), version, today or datetime.date.today().isoformat())
        expected = release_files(original, version, date)
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
        REPLAY,
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
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--publish", action="store_true", help="commit, tag and atomically push main and the tag after all checks pass")
    mode.add_argument("--notes", action="store_true", help="print the version's changelog entries, the GitHub release text, and exit")
    args = parser.parse_args()
    root = Path(__file__).resolve().parent.parent
    try:
        if args.notes:
            print(release_notes((root / CHANGELOG).read_text(), args.version), end="")
            return
        execute(root, args.version, args.publish)
    except (ReleaseError, subprocess.CalledProcessError, OSError) as error:
        parser.exit(1, f"Release stopped: {error}\nFix the cause and rerun the same command; existing tags are never moved.\n")


if __name__ == "__main__":
    main()
