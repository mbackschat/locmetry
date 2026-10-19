"""Check delivered benchmark artifacts; interpretation still needs review."""

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys


def grade(pair, run):
    manifest = json.loads((pair / "manifest.json").read_text())
    task = manifest["task"]
    number = int(task["revision"][4])
    root = pair / run
    evidence = pair / "evidence" / run
    summary = json.loads((evidence / "summary.json").read_text())
    binary = pair / summary["condition"] / "target/debug/locmetry"
    checks = {}
    missing = []
    env = dict(os.environ, XDG_CONFIG_HOME=str(pair / "user-config" / run), NO_COLOR="1")

    def check(name, value):
        checks[name] = bool(value)

    def load(name):
        path = root / name
        if not path.is_file():
            missing.append(name)
            return None
        try:
            return json.loads(path.read_text())
        except json.JSONDecodeError:
            check("valid-json:" + name, False)
            return None

    def command(args):
        return subprocess.run([str(binary), *args], cwd=root, env=env, capture_output=True, text=True)

    def counts(report, expected):
        for field, value in expected.items():
            check("source:" + field, report["summary"]["source"][field] == value)

    check("process-completed", summary["returncode"] == 0 and summary["stopped"] is None)
    check("protected-fixture-intact", summary["fixture_intact"])
    audit = json.loads((evidence / "context-audit.json").read_text())
    check("context-isolated", audit["repository_context"] is False and bool(audit["configurations"]))
    calls = [json.loads(l) for l in (evidence / "calls.jsonl").read_text().splitlines()][:summary["cli_invocations"]]
    (evidence / "agent-calls.jsonl").write_text("".join(json.dumps(c) + "\n" for c in calls))
    for name in task["oracle"].get("artifacts", []):
        if not (root / name).is_file():
            missing.append(name)
    try:
        if number == 1:
            for c in calls:
                try:
                    report = json.loads((evidence / (c["id"] + ".stdout")).read_text())
                except json.JSONDecodeError:
                    continue
                if isinstance(report, dict) and "summary" in report:
                    counts(report, task["oracle"]["source"])
                    for field, value in task["oracle"]["all_counted"].items():
                        check("all-counted:" + field, report["summary"]["all_counted"][field] == value)
        elif number == 2:
            web, partial = load("outputs/web.json"), load("outputs/partial.json")
            if web:
                counts(web, task["oracle"]["web"]["source"])
                check("web-files", len(web["files"]) == 9)
            if partial:
                check("partial-flag", partial["metadata"]["backend_inaccurate"] is True)
                check("known-zero-empty", len(partial["files"]) == 1 and partial["files"][0]["path"] == "empty.py" and partial["files"][0]["counts"]["code"] == 0)
                check("unknown-parse-failure", partial["metadata"]["coverage"]["parse_failures"][0]["path"] == "broken.ipynb")
            if (root / "collect.py").exists():
                probe = root / "evaluator-probes"
                probe.mkdir(exist_ok=True)
                invalid = probe / "invalid"
                invalid.mkdir(exist_ok=True)
                (invalid / ".locmetryrc").write_text("version = [\n")
                probe_records = []
                for label, project, status in [("complete", "projects/web", 0), ("partial", "projects/partial", 2), ("invalid", "evaluator-probes/invalid", 1)]:
                    before = len((evidence / "calls.jsonl").read_text().splitlines())
                    result = subprocess.run([sys.executable, str(root / "collect.py"), project, f"evaluator-probes/{label}.json"], cwd=root, env=env, capture_output=True, text=True, timeout=30)
                    after = len((evidence / "calls.jsonl").read_text().splitlines())
                    check("script-status:" + label, result.returncode == status)
                    check("one-measurement:" + label, after - before == 1)
                    if status != 1:
                        report = load(f"evaluator-probes/{label}.json")
                        check("saved-full-files:" + label, report is not None and "files" in report and "breakdown" in report)
                        check("printed-human:" + label, "Code" in result.stdout)
                    probe_records.append(dict(label=label, status=result.returncode, stdout=result.stdout, stderr=result.stderr, calls=after-before))
                (evidence / "script-replay.json").write_text(json.dumps(probe_records, indent=2) + "\n")
        elif number == 3:
            report = load("outputs/configured.json")
            actual_result = command(["--root", "projects/web", "--files", "--format", "json"])
            check("configuration-loads", actual_result.returncode == 0)
            actual = json.loads(actual_result.stdout)
            if report:
                check("saved-report-matches-policy", report["summary"] == actual["summary"])
                check("saved-per-file-evidence", report.get("files") == actual["files"])
            counts(actual, task["oracle"]["source"])
            check("remaining-review", [x["path"] for x in actual["review"]] == ["scratch/mystery.py"])
            files = {f["path"]: f for f in actual["files"]}
            check("vendor-excluded", "vendor/bundle.js" not in files)
            check("tooling-policy", files["tools/release.py"]["purpose"] == "production" and files["tools/release.py"]["category"] == "tooling")
            check("intentional-other", files["examples/old.py"]["purpose"] == "other" and files["examples/old.py"]["category"] == "examples")
            check("inference-preserved", files["src/index.js"]["rule"] == "builtin:npm-module")
            check("test-preserved", files["tests/index.test.js"]["purpose"] == "test")
        elif number == 4:
            before, after, change = load("outputs/before.json"), load("outputs/after.json"), load("outputs/change.json")
            if before and after and change:
                check("baseline-before-edit", before["summary"]["source"]["code"] == 7)
                check("after-code", after["summary"]["source"]["code"] == 8)
                check("exact-delta", change["summary"]["source"]["delta"] == task["oracle"]["delta"])
                result = command(["compare", "outputs/before.json", "outputs/after.json", "--format", "json"])
                check("valid-inputs", result.returncode == 0)
                check("comparison-replays", json.loads(result.stdout)["summary"] == change["summary"])
            source = "projects/web/src/index.js"
            check("exact-mutation", (root / source).read_text() == task["files"][source] + "export const c = 3;\n")
            check("headline-negative-demonstrated", any(c["exit"] == 1 and "compare" in c["args"] and any("headline.json" in a for a in c["args"]) for c in calls))
        elif number == 5:
            combined, direct, only, change = (load(n) for n in ["outputs/combined.json", "outputs/direct.json", "outputs/web-only.json", "outputs/workspace-change.json"])
            if combined and direct and only and change:
                check("methods-agree", combined["summary"] == direct["summary"] and combined["files"] == direct["files"])
                check("workspace-per-file-evidence", len(combined["files"]) == 11)
                counts(combined, {"code": 12, "files": 7, "lines": 15})
                check("weighted-ratio-inputs", combined["summary"]["source_purpose"]["production"]["code"] == 4 and combined["summary"]["source_purpose"]["test"]["code"] == 6)
                check("web-only", only["summary"]["source"]["code"] == 7)
                check("departure-delta", change["summary"]["source"]["delta"]["code"] == -5)
                check("worker-departure", change["members"]["worker"]["presence"] == "before_only")
                check("retained-member-unchanged", change["members"]["web"]["source"]["delta"]["code"] == 0)
        elif number == 6:
            result = load("explanations.json")
            if result:
                for key, state in task["oracle"]["states"].items():
                    check("state:" + key, result[key]["document_type"] == "explanation" and result[key]["result"]["state"] == state)
                    check("complete-explanation:" + key, {"schema_version", "root", "path", "config_path", "user_config_path", "scope", "result"}.issubset(result[key]))
                check("module-evidence", result["app"]["result"]["decision"]["stage"] == "module")
                check("rule-evidence", result["tool"]["result"]["decision"]["stage"] == "rule")
                check("unclaimed-counted", result["scratch"]["result"]["file"]["category"] == "unclaimed")
                check("workspace-identity", result["workspace_tool"]["member"] == "web" and result["workspace_tool"]["path"] == "web/tools/release.py")
                check("member-relative-pattern", result["workspace_tool"]["result"]["decision"]["matched_patterns"] == ["tools/**"])
    except (KeyError, TypeError, ValueError, subprocess.TimeoutExpired) as error:
        check("artifact-structure-or-execution", False)
        checks["error"] = str(error)
    outcome = "artifact_checks_pass" if all(v is True for v in checks.values()) and not missing else "needs_review"
    result = dict(task=number, condition=summary["condition"], outcome=outcome, checks=checks, missing=sorted(set(missing)), interpretation="pending")
    (evidence / "artifact-grade.json").write_text(json.dumps(result, indent=2) + "\n")
    return result


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("campaign", type=Path)
    args = parser.parse_args()
    for path in sorted(args.campaign.glob("task*-rep*/evidence/run-*/summary.json")):
        evidence = path.parent
        if (evidence / "artifact-grade.json").exists():
            continue
        result = grade(path.parents[2], evidence.name)
        print(path.parents[2].name, evidence.name, json.dumps(result))
