"""Check long-option coverage and replay Showboat documents without rewriting them."""

import argparse
import os
from pathlib import Path
import re
import subprocess
import tempfile
import time

REPOSITORY = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("LOCMETRY_BIN", REPOSITORY / "target/debug/locmetry")).resolve()
GUIDES = {"root": [], "workspace": ["workspace"], "compare": ["compare"], "explain": ["explain"]}
# Recordings of the pinned Immich checkouts (examples/showcase/immich/README.md).
SHOWCASE = ["docs/showcase/tour.md", "docs/showcase/measure.md", "docs/showcase/workspace.md", "docs/showcase/compare.md",
            "docs/duckdb-cookbook.md", "docs/dataframe-cookbook.md", "docs/analytics-demo.md"]
SELECTIONS = ["reference", "showcase", "all"]


def missing_options(help_text, document):
    options = set(re.findall(r"^\s+(?:-\w, )?(--[\w-]+)", help_text, re.MULTILINE))
    code = "\n".join(re.findall(r"^```bash\n(.*?)^```$", document, re.MULTILINE | re.DOTALL))
    return options - set(re.findall(r"--[\w-]+", code))


def validate_document(document):
    if not re.search(r"^\*.* by Showboat [0-9.]+\*$", document, re.MULTILINE) or "<!-- showboat-id:" not in document:
        raise ValueError("Missing Showboat header; create the document with showboat init")
    language = None
    for number, line in enumerate(document.splitlines(), 1):
        if line.startswith("```"):
            language = line[3:] if language is None else None
        elif language is not None and len(line) > 120:
            raise ValueError(f"Line {number}: {language} line exceeds 120 characters; wrap the command or its displayed output")


def git_state():
    commands = {
        "status": ["status", "--porcelain=v1", "--untracked-files=all"],
        "working-tree": ["diff", "--no-ext-diff", "--no-textconv", "--binary", "HEAD", "--"],
        "index": ["diff", "--no-ext-diff", "--no-textconv", "--binary", "--cached", "--"],
    }
    return b"\n".join(
        name.encode() + b"\n" + subprocess.check_output(["git", *args])
        for name, args in commands.items()
    )


def showcase_unavailable():
    """Return the fetch script's diagnostic when a pinned checkout is missing or moved."""
    result = subprocess.run(["bash", "scripts/fetch-showcase.sh", "--check"],
                            stdout=subprocess.DEVNULL, stderr=subprocess.PIPE, text=True)
    return result.stderr or "Showcase checkouts are unavailable" if result.returncode else None


def evidence(path, artifacts, suffix):
    """Name evidence after the document's folder too: docs/cli and docs/showcase share file names."""
    return artifacts / f"{path.parent.name}-{path.stem}{suffix}"


def record(path, artifacts, status, diagnostic, passed):
    """Log and print one document's outcome; a failure shows its diagnostic."""
    evidence(path, artifacts, ".log").write_text(status + "\n" + diagnostic)
    print(status, flush=True)
    if not passed and diagnostic:
        print(diagnostic, end="" if diagnostic.endswith("\n") else "\n", flush=True)
    return passed


def unavailable(path, artifacts, diagnostic):
    return record(path, artifacts, f"FAIL {path} (not replayed)", diagnostic, False)


def replay(path, artifacts, command=None):
    started = time.perf_counter()
    candidate = evidence(path, artifacts, ".candidate.md")
    candidate.unlink(missing_ok=True)
    print(f"Verifying {path}", flush=True)
    try:
        document = path.read_text()
        validate_document(document)
        if command is not None:
            help_text = subprocess.check_output([str(BINARY), *command, "--help"], text=True)
            missing = missing_options(help_text, document)
            if missing:
                raise ValueError(f"Missing executable examples for {sorted(missing)}")
        result = subprocess.run(
            ["showboat", "verify", str(path), "--output", str(candidate)],
            stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True, timeout=120,
        )
        passed = result.returncode == 0
        diagnostic = result.stdout
    except subprocess.TimeoutExpired as error:
        partial = error.stdout or b""
        diagnostic = partial.decode(errors="replace") if isinstance(partial, bytes) else partial
        diagnostic += "\nShowboat exceeded the 120-second document timeout.\n"
        passed = False
    except (OSError, ValueError, subprocess.CalledProcessError) as error:
        diagnostic = f"{error}\n"
        passed = False
    elapsed = time.perf_counter() - started
    if passed:
        candidate.unlink(missing_ok=True)
    status = f"{'PASS' if passed else 'FAIL'} {path} ({elapsed:.2f}s)"
    return record(path, artifacts, status, diagnostic, passed)


def check(artifacts, selection="all"):
    os.chdir(REPOSITORY)
    artifacts = artifacts.resolve()
    artifacts.mkdir(parents=True, exist_ok=True)
    for name in ["git-before.txt", "git-after.txt"]:
        (artifacts / name).unlink(missing_ok=True)
    started = time.perf_counter()
    before = git_state()
    documents = []
    if selection != "showcase":
        documents += [(Path(f"docs/cli/{name}.md"), command, False) for name, command in GUIDES.items()]
    if selection != "reference":
        documents += [(Path(path), None, True) for path in SHOWCASE]
    missing = showcase_unavailable() if any(showcase for *_, showcase in documents) else None
    passed = True
    try:
        for path, command, showcase in documents:
            if showcase and missing:
                passed = unavailable(path, artifacts, missing) and passed
            elif not replay(path, artifacts, command):
                passed = False
    except BaseException:
        passed = False
        raise
    finally:
        after = git_state()
        if before != after:
            (artifacts / "git-before.txt").write_bytes(before)
            (artifacts / "git-after.txt").write_bytes(after)
            print("FAIL Git state changed during replay; inspect git-before.txt and git-after.txt.", flush=True)
            passed = False
        elapsed = time.perf_counter() - started
        summary = f"Total: {len(documents)} documents, {elapsed:.2f}s; {'PASS' if passed else 'FAIL'}."
        (artifacts / "summary.log").write_text(summary + "\n")
        print(summary, flush=True)
        if not passed:
            print(f"Replay evidence: {artifacts}", flush=True)
    return passed


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--artifacts-dir", type=Path, help="Evidence directory outside the checkout; default: a new temporary directory")
    parser.add_argument("--set", choices=SELECTIONS, default="all",
                        help="reference: guides on the checked-in fixture; showcase: recordings of the pinned Immich checkouts")
    args = parser.parse_args()
    artifacts = args.artifacts_dir or Path(tempfile.mkdtemp(prefix="locmetry-showboat-"))
    raise SystemExit(0 if check(artifacts, args.set) else 1)
