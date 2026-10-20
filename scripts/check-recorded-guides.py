"""Check long-option coverage and replay Showboat documents without rewriting them."""

import os
from pathlib import Path
import re
import subprocess

REPOSITORY = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("LOCMETRY_BIN", REPOSITORY / "target/debug/locmetry")).resolve()
GUIDES = {"root": [], "workspace": ["workspace"], "compare": ["compare"], "explain": ["explain"]}


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


def replay(path):
    validate_document(path.read_text())
    print(f"Verifying {path}", flush=True)
    subprocess.run(["showboat", "verify", str(path)], check=True, timeout=120)


def check():
    os.chdir(REPOSITORY)
    for name, command in GUIDES.items():
        path = Path(f"docs/cli/{name}.md")
        help_text = subprocess.check_output([str(BINARY), *command, "--help"], text=True)
        missing = missing_options(help_text, path.read_text())
        if missing:
            raise SystemExit(f"{path}: missing executable examples for {sorted(missing)}")
        replay(path)
    replay(Path("docs/analytics-demo.md"))
    print("All command options covered; five Showboat documents replay successfully.")


if __name__ == "__main__":
    check()
