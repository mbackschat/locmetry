"""Generate the agent regression trend table from the portable study results.

Each paired study compares a control and a candidate inside one campaign, so
only its step ratios are valid; absolute counts from different campaigns are
not comparable (docs/agent-regression-checks.md, "Trend"). A task's series
continues while consecutive checks measure it on the same model and effort,
with each control being the previous candidate; the cumulative change is the
product of the step ratios since the series started.
"""

import argparse
import json
from pathlib import Path
import sys

REPO = Path(__file__).resolve().parents[1]
RESULTS = REPO / "docs/research"
DOCUMENT = REPO / "docs/agent-regression-checks.md"
START = "<!-- agent-regression-trend:start -->"
END = "<!-- agent-regression-trend:end -->"


def write_json(path, value):
    path.write_text(json.dumps(value, indent=2) + "\n")


def load_results(directory):
    """Portable results of paired version checks: two arms with per-task medians."""
    studies = []
    for path in sorted(directory.glob("*-results.json")):
        study = json.loads(path.read_text())
        arms = study.get("arms")
        if isinstance(arms, dict) and set(arms) == {"A", "B"} and study.get("analysis", {}).get("per_task"):
            studies.append(study)
    return studies


def short(revision):
    return revision[:7]


def ratio(control, candidate):
    return candidate / control if control else None


def chronological(studies):
    """Order by date; on the same date a check precedes the one it is the control of."""
    pending = sorted(studies, key=lambda s: s["date"])
    ordered = []
    while pending:
        first = pending[0]
        control = short(first["arms"]["A"]["source_revision"])
        earlier = next((s for s in pending[1:] if s["date"] == first["date"]
                        and short(s["arms"]["B"]["source_revision"]) == control), first)
        pending.remove(earlier)
        ordered.append(earlier)
    return ordered


def trend_rows(studies):
    rows = []
    for study in chronological(studies):
        control, candidate = (short(study["arms"][arm]["source_revision"]) for arm in "AB")
        runs = study.get("runs", [])
        for task, arms in sorted(study["analysis"]["per_task"].items(), key=lambda item: int(item[0])):
            task = int(task)
            steps = {}
            for key, metric in [("calls", "tools"), ("uncached", "fresh")]:
                steps[key] = (arms["A"][metric]["median"], arms["B"][metric]["median"])
            row = dict(date=study["date"], model=study["model"], effort=study["effort"], control=control,
                       candidate=candidate, task=task, base=control,
                       passes={arm: (arms[arm]["passes"], sum(r["task"] == task and r["arm"] == arm for r in runs))
                               for arm in "AB"},
                       **{key: value for key, value in steps.items()})
            previous = next((r for r in reversed(rows)
                             if r["task"] == task and r["candidate"] == control
                             and (r["model"], r["effort"]) == (row["model"], row["effort"])), None)
            for key, (a, b) in steps.items():
                step = ratio(a, b)
                before = previous[f"{key}_since"] if previous else 1.0
                row[f"{key}_since"] = None if step is None or before is None else before * step
            if previous:
                row["base"] = previous["base"]
            rows.append(row)
    return rows


def number(value):
    # Medians of four runs can end in .5; that matters for calls, not for token counts.
    return f"{value:,.0f}" if value == int(value) or value >= 1000 else f"{value:,.1f}"


def change(pair):
    step = ratio(*pair)
    return f"{number(pair[0])} → {number(pair[1])} ({'—' if step is None else f'×{step:.2f}'})"


def since(value):
    return "—" if value is None else f"×{value:.2f}"


def render(rows):
    lines = ["| Date | Model | Control → candidate | Task | Passes | Median calls | Median uncached input | Since | Calls since | Uncached since |",
             "| --- | --- | --- | ---: | --- | --- | --- | --- | ---: | ---: |"]
    for row in rows:
        passes = " → ".join(f"{passed}/{runs}" for passed, runs in (row["passes"][arm] for arm in "AB"))
        lines.append(f"| {row['date']} | {row['model']} `{row['effort']}` | `{row['control']}` → `{row['candidate']}` | "
                     f"{row['task']} | {passes} | {change(row['calls'])} | {change(row['uncached'])} | "
                     f"`{row['base']}` | {since(row['calls_since'])} | {since(row['uncached_since'])} |")
    return "\n".join(lines)


def sync(document, results, *, write):
    """Return whether the managed table was current; with write, regenerate it."""
    text = document.read_text()
    if text.count(START) != 1 or text.count(END) != 1 or text.index(START) > text.index(END):
        raise ValueError(f"{document}: expected one {START} … {END} block")
    updated = text[:text.index(START)] + START + "\n\n" + render(trend_rows(load_results(results))) + "\n\n" + text[text.index(END):]
    if write and updated != text:
        document.write_text(updated)
    return updated == text


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("command", choices=["update", "check"])
    args = parser.parse_args(argv)
    current = sync(DOCUMENT, RESULTS, write=args.command == "update")
    if args.command == "check" and not current:
        print(f"Stale trend table in {DOCUMENT.relative_to(REPO)}; run python3 scripts/agent-regression-trend.py update")
        return 1
    print("Agent regression trend table " + ("current." if current else "updated."))
    return 0


if __name__ == "__main__":
    sys.exit(main())
