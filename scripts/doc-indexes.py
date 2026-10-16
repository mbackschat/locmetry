#!/usr/bin/env python3
"""Maintain documentation indexes; contract: docs/documentation-maintenance.md."""

import argparse
from collections import Counter
from datetime import date
import json
from pathlib import Path
import re
import subprocess
import sys
from urllib.parse import quote, unquote, urlsplit

START = "<!-- doc-index:start -->"
END = "<!-- doc-index:end -->"
METADATA = re.compile(r"<!-- doc-status\s*\n(.*?)\n-->", re.S)
STATUSES = {
    "proposals": {"draft", "accepted", "implemented", "rejected", "superseded"},
    "plans": {"planned", "active", "deferred", "completed", "historical"},
    "releases": {"unreleased", "published"},
}
EVIDENCE_REQUIRED = {"accepted", "implemented", "rejected", "superseded", "completed", "historical", "published"}
LINK = re.compile(r"!?\[([^\]\n]*)\]\((<[^>\n]+>|[^\s()]+)(?:\s+\"[^\"\n]*\")?\)")


def prose(text):
    """Ignore examples and comments when checking rendered navigation."""
    lines = []
    fence = None
    for line in text.splitlines():
        marker = re.match(r"^\s{0,3}(`{3,}|~{3,})", line)
        if fence:
            if re.fullmatch(r"\s{0,3}" + re.escape(fence[0]) + "{" + str(len(fence)) + r",}\s*", line):
                fence = None
        elif marker:
            fence = marker[1]
        elif not line.startswith(("    ", "\t")):
            lines.append(line)
    return re.sub(r"<!--.*?-->", "", "\n".join(lines), flags=re.S)


def links(text):
    text = re.sub(r"(`+).*?\1", "", prose(text))
    return [match[2].strip("<>") for match in LINK.finditer(text)]


def anchors(text):
    result = set()
    for line in prose(text).splitlines():
        match = re.match(r"^ {0,3}#{1,6}\s+(.+?)(?:\s+#+\s*)?$", line)
        if not match:
            continue
        heading = re.sub(r"!?\[([^]]+)\]\([^)]*\)", r"\1", match[1])
        heading = re.sub(r"<[^>]*>", "", heading).replace("`", "").replace("*", "")
        slug = re.sub(r"[^\w\- ]", "", heading.lower()).replace(" ", "-")
        unique = slug
        suffix = 0
        while unique in result:
            suffix += 1
            unique = f"{slug}-{suffix}"
        result.add(unique)
    return result


def local_target(source, destination):
    url = urlsplit(destination)
    if url.scheme or url.netloc:
        return None
    path = source.parent / unquote(url.path) if url.path else source
    return path.resolve(), unquote(url.fragment)


def check_links(source, text, errors):
    for destination in links(text):
        target = local_target(source, destination)
        if target is None:
            continue
        path, fragment = target
        if not path.exists():
            errors.append(f"{source}: broken link {destination}")
        elif fragment and path.suffix == ".md" and fragment not in anchors(path.read_text()):
            errors.append(f"{source}: missing fragment {destination}")


def status_kind(path):
    return path.parts[1] if len(path.parts) >= 3 and path.parts[1] in STATUSES else None


def metadata(path, text, kind, errors):
    matches = list(METADATA.finditer(text))
    if len(matches) != 1:
        errors.append(f"{path}: expected one doc-status metadata block")
        return None
    if not re.fullmatch(r"# [^\n]+\n\s*", text[:matches[0].start()]):
        errors.append(f"{path}: metadata must be immediately after the title")
    try:
        data = json.loads(matches[0][1])
    except ValueError as error:
        errors.append(f"{path}: invalid metadata: {error}")
        return None
    fields = {"status", "date", "summary", "evidence"}
    if not isinstance(data, dict) or set(data) != fields or any(not isinstance(value, str) for value in data.values()):
        errors.append(f"{path}: metadata fields must be strings: {', '.join(sorted(fields))}")
        return None
    if data["status"] not in STATUSES[kind]:
        errors.append(f"{path}: invalid status {data['status']!r} for {kind}")
    try:
        if not re.fullmatch(r"\d{4}-\d{2}-\d{2}", data["date"]):
            raise ValueError()
        date.fromisoformat(data["date"])
    except ValueError:
        errors.append(f"{path}: invalid date; use the evidenced status date YYYY-MM-DD")
    if not data["summary"].strip() or any("\n" in value or "\r" in value for value in data.values()):
        errors.append(f"{path}: metadata must contain single-line values and a nonempty summary")
    if (data["evidence"] or data["status"] in EVIDENCE_REQUIRED) and not links(data["evidence"]):
        errors.append(f"{path}: expected a decision or verification evidence link")
    title = re.search(r"^# (.+)$", text, flags=re.M)
    if not title:
        errors.append(f"{path}: missing document title")
        return None
    return title[1], data


def table_cell(value):
    return value.replace("|", "&#124;").replace("[", "&#91;").replace("]", "&#93;")


def run(root, mode):
    result = subprocess.run(["git", "ls-files", "--cached", "--others", "--exclude-standard", "-z", "--", "docs"], cwd=root, check=True, capture_output=True)
    paths = sorted({Path(name) for name in result.stdout.decode().split("\0") if name.endswith(".md") and not name.startswith("docs/research/local/") and (root / name).is_file()})
    directories = {parent for path in paths for parent in path.parents if parent != Path(".")}
    errors = []
    texts = {path: (root / path).read_text() for path in paths}
    updates = {}
    records = {}
    for path, text in texts.items():
        kind = status_kind(path)
        if kind and path.name != "README.md":
            record = metadata(path, text, kind, errors)
            if record:
                records[path] = record
                check_links(root / path, record[1]["evidence"], errors)
    for directory in sorted(directories):
        index = directory / "README.md"
        if index not in texts:
            errors.append(f"{index}: missing index")
            continue
        text = texts[index]
        children = sorted(path for path in paths if path.parent == directory and path != index)
        kind = status_kind(index)
        if kind:
            if text.count(START) != 1 or text.count(END) != 1 or text.index(START) > text.index(END):
                errors.append(f"{index}: expected one ordered managed index section")
                continue
            rows = ["| Document | Status | Date | Summary |", "| --- | --- | --- | --- |"]
            for path in children:
                if path not in records:
                    continue
                title, data = records[path]
                rows.append(f"| [{table_cell(title)}]({quote(path.name)}) | {data['status']} | {data['date']} | {table_cell(data['summary'])} |")
            content = "\n".join(rows) if children else "No documents in this directory."
            updated = text[:text.index(START)] + START + "\n\n" + content + "\n\n" + text[text.index(END):]
            if text != updated:
                updates[index] = updated
                if mode == "check":
                    errors.append(f"{index}: stale index; run python3 scripts/doc-indexes.py update")
            text = updated
        required = children + [child / "README.md" for child in directories if child.parent == directory]
        counts = Counter(target[0] for destination in links(text) if (target := local_target(root / index, destination)) and not target[1])
        for path in required:
            count = counts[(root / path).resolve()]
            if count != 1:
                errors.append(f"{index}: {'missing' if count == 0 else 'duplicate'} entry for {path}")
    for path, text in texts.items():
        check_links(root / path, updates.get(path, text), errors)
    for name in ("README.md", "CLAUDE.md"):
        path = root / name
        if path.is_file():
            check_links(path, path.read_text(), errors)
    if errors:
        print("\n".join(errors), file=sys.stderr)
        return 1
    if mode == "update":
        for path, text in updates.items():
            (root / path).write_text(text)
    print(f"Documentation indexes {mode}: {len(paths)} documents, {len(directories)} folders; {len(updates)} index(es) {'updated' if mode == 'update' else 'stale'}.")
    return 0


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("check", "update"))
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    try:
        return run(args.root.resolve(), args.mode)
    except (OSError, subprocess.CalledProcessError) as error:
        print(str(error), file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
