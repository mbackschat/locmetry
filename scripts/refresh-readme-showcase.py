"""Regenerate the README's Immich showcase excerpts from the pinned checkouts.

Run from anywhere after `cargo build --locked` and `bash scripts/fetch-showcase.sh`.
The line-counter comparison also needs Tokei, cloc and scc on PATH; without
all three, that table is left unchanged and a warning names the missing tools.
"""

import json
from pathlib import Path
import re
import shutil
import subprocess
import sys

REPOSITORY = Path(__file__).resolve().parents[1]
README = REPOSITORY / "README.md"
POLICY = REPOSITORY / "examples/showcase/immich/overlay/.locmetryrc"
WORKSPACE = REPOSITORY / "examples/showcase/immich/overlay/workspace.toml"
# One file per decision stage the README explains, from conventions to policy.
EXAMPLES = [
    "server/src/services/asset.service.ts",
    "mobile/lib/main.dart",
    "server/src/services/asset.service.spec.ts",
    "e2e/src/specs/web/album.e2e-spec.ts",
    "e2e/src/utils.ts",
    "mobile/ios/Runner/AppDelegate.swift",
    "packages/sdk/src/fetch-client.ts",
]
# Policy entries quoted in the README's configuration example.
QUOTED_ENTRIES = ["generated-sdk-client", "ios-app"]


def showcase(script):
    """Run Bash in the showcase environment and return its stdout."""
    return subprocess.run(["bash", "-c", "source scripts/showcase-env.sh\n" + script],
                          cwd=REPOSITORY, check=True, capture_output=True, text=True).stdout


def grid(text, heading):
    """Return the box-drawn table whose header row starts with `heading`."""
    for part in text.split("\n\n"):
        if part.startswith(" ─") and re.search(rf"^ {heading} ", part, re.MULTILINE):
            # Comparison headers pad their group titles with trailing spaces,
            # which `git diff --check` rejects in Markdown.
            return "\n".join(line.rstrip() for line in part.rstrip("\n").split("\n")) + "\n"
    raise ValueError(f"No {heading} table in output:\n{text}")


def number(value):
    return f"{value:,}"


def policy_table():
    script = """printf 'version = 1\\n' > builtin-only.toml
locmetry --root immich --config builtin-only.toml --format json
locmetry --root immich --format json"""
    decoder = json.JSONDecoder()
    text = showcase(script)
    builtin, index = decoder.raw_decode(text)
    reviewed, _ = decoder.raw_decode(text[index:].lstrip())

    def column(report):
        summary = report["summary"]
        purposes = {name: counts["code"] for name, counts in summary["source_purpose"].items()}
        categories = {name: counts["code"] for name, counts in summary["source_categories"].items()}
        production, test = purposes.get("production", 0), purposes.get("test", 0)
        return [number(production), number(test), number(categories.get("other:generated", 0)),
                number(categories.get("other:unclaimed", 0)), f"{test / production:.2f}"]

    rows = ["Production", "Test", "Other: generated", "Other: unclaimed", "Test/prod"]
    lines = ["| Source code | Built-in conventions | Reviewed policy |", "| --- | ---: | ---: |"]
    lines += [f"| {row} | {a} | {b} |" for row, a, b in zip(rows, column(builtin), column(reviewed))]
    return "\n".join(lines) + "\n"


def examples_table():
    report = json.loads(showcase("locmetry --root immich --format json --files"))
    files = {record["path"]: record for record in report["files"]}
    lines = ["| File | Purpose / category | Deciding rule |", "| --- | --- | --- |"]
    for path in EXAMPLES:
        record = files[path]
        rule = record["rule"]
        kind = "" if rule.startswith("builtin:") else " (project entry)"
        lines.append(f"| `{path}` | {record['purpose']} / {record['category']} | `{rule}`{kind} |")
    return "\n".join(lines) + "\n"


def quoted_policy():
    blocks = POLICY.read_text().split("\n\n")
    quoted = [block for block in blocks
              if any(f'id = "{entry}"' in block for entry in QUOTED_ENTRIES)]
    if len(quoted) != len(QUOTED_ENTRIES):
        raise ValueError(f"Expected entries {QUOTED_ENTRIES} in {POLICY}")
    return "version = 1\n\n" + "\n\n".join(block.strip("\n") for block in quoted) + "\n"


def tools_table():
    missing = [tool for tool in ["tokei", "cloc", "scc"] if shutil.which(tool) is None]
    if missing:
        print(f"warning: {', '.join(missing)} not on PATH; keeping the line-counter table", file=sys.stderr)
        return None
    root = showcase("cd immich && pwd -P").strip()
    versions = {
        "Tokei": subprocess.check_output(["tokei", "--version"], text=True).split()[1],
        "cloc": subprocess.check_output(["cloc", "--version"], text=True).strip(),
        "scc": subprocess.check_output(["scc", "--version"], text=True).split()[-1],
    }
    tokei = json.loads(subprocess.check_output(["tokei", root, "--output", "json"], text=True))
    cloc = json.loads(subprocess.check_output(["cloc", "--json", "--quiet", root], text=True))
    scc = json.loads(subprocess.check_output(["scc", "--format", "json", root], text=True))
    counts = {
        "Tokei": (tokei["Total"]["code"], tokei.get("JSON", {}).get("code", 0)),
        "cloc": (cloc["SUM"]["code"], cloc.get("JSON", {}).get("code", 0)),
        "scc": (sum(row["Code"] for row in scc), sum(row["Code"] for row in scc if row["Name"] == "JSON")),
    }
    lines = ["| Tool | Reported code | of which JSON | Production / test split |", "| --- | ---: | ---: | --- |"]
    for tool, (code, json_code) in counts.items():
        lines.append(f"| {tool} {versions[tool]} | {number(code)} | {number(json_code)} "
                     f"({json_code / code:.0%}) | none |")
    return "\n".join(lines) + "\n"


def module_production(text):
    """The verbose note on the production build modules classified."""
    prefix = " Production from build modules:"
    return next(line.rstrip() for line in text.splitlines() if line.startswith(prefix)) + "\n"


def replace_text(readme, marker, content, language="text"):
    prefix = f"<!-- {marker} -->\n```{language}\n"
    start = readme.index(prefix) + len(prefix)
    end = readme.index("```\n", start)
    return readme[:start] + content + readme[end:]


def replace_table(readme, marker, content):
    prefix = f"<!-- {marker} -->\n"
    start = readme.index(prefix) + len(prefix)
    end = readme.index("\n\n", start) + 1
    return readme[:start] + content + readme[end:]


def main():
    readme = README.read_text()
    # A terminal shows share bars, which grow with its width; 120 columns
    # keeps them in the excerpts at a reproducible size.
    measurement = showcase("cd immich && COLUMNS=120 locmetry --format table --no-color --show-bars")
    categories = showcase("cd immich && COLUMNS=120 locmetry --show-categories "
                          "--format table --no-color --show-bars")
    workspace = showcase("COLUMNS=120 locmetry workspace --workspace-file immich/workspace.toml "
                         "--format table --no-color --show-bars")
    comparison = showcase("""locmetry --root v3.2.0/immich --output v3.2.0.json
locmetry --root v3.3.4/immich --output v3.3.4.json
locmetry compare v3.2.0.json v3.3.4.json --format table --no-color""")
    verbose = showcase("cd immich && COLUMNS=120 locmetry --format table --no-color --no-bars --verbose")
    readme = replace_text(readme, "showcase-language", grid(measurement, "Language"))
    readme = replace_text(readme, "showcase-categories", grid(categories, "Category"))
    readme = replace_text(readme, "showcase-modules", module_production(verbose))
    readme = replace_text(readme, "showcase-workspace", grid(workspace, "Repository"))
    readme = replace_text(readme, "showcase-compare", grid(comparison, "Purpose"))
    readme = replace_table(readme, "showcase-policy", policy_table())
    readme = replace_table(readme, "showcase-examples", examples_table())
    readme = replace_text(readme, "showcase-config", quoted_policy(), "toml")
    readme = replace_text(readme, "showcase-workspace-file", WORKSPACE.read_text(), "toml")
    tools = tools_table()
    if tools is not None:
        readme = replace_table(readme, "showcase-tools", tools)
    README.write_text(readme)
    print("README showcase excerpts refreshed")


if __name__ == "__main__":
    main()
