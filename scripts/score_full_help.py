"""Check full-help study artifacts; final-answer interpretation remains separate."""

import argparse
import importlib.util
import json
import os
from pathlib import Path
import re
import subprocess
import time
import tomllib


SPEC = importlib.util.spec_from_file_location("legacy_artifact_score", Path(__file__).with_name("score_agent_explorability.py"))
LEGACY = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(LEGACY)


def without_measurement_time(value):
    if isinstance(value, dict):
        return {key: without_measurement_time(item) for key, item in value.items() if key != "measured_at_unix_seconds"}
    if isinstance(value, list):
        return [without_measurement_time(item) for item in value]
    return value


def human_without_time(text):
    return "\n".join(line.rstrip() for line in text.splitlines() if not line.startswith("Measured at: "))


def preserved_policy(original, actual):
    if actual.get("version") != original["version"] or actual.get("backend", "tokei") != original.get("backend", "tokei"):
        return False
    defaults = {
        "scope": {"hidden": True, "respect_ignore_files": True, "default_excludes": True, "exclude": []},
        "classification": {"builtin_tests": True, "builtin_build": True, "builtin_modules": True, "builtin_generated": True, "fallback": "other"},
    }
    for section, settings in defaults.items():
        for key, default in settings.items():
            if actual.get(section, {}).get(key, default) != original.get(section, {}).get(key, default):
                return False
    if actual.get("languages", {}).get("builtin", True) != original.get("languages", {}).get("builtin", True):
        return False
    if actual.get("rules", []) != original.get("rules", []):
        return False
    for key, meaning in original.get("categories", {}).items():
        if actual.get("categories", {}).get(key) != meaning:
            return False
    for old, new in [(original.get("overrides", []), actual.get("overrides", [])),
                     (original.get("languages", {}).get("mappings", []), actual.get("languages", {}).get("mappings", []))]:
        if new[:len(old)] != old:
            return False
    return True


def grade(pair, run):
    pair = Path(pair).resolve()
    manifest = json.loads((pair / "manifest.json").read_text())
    task = manifest["task"]
    number = int(re.match(r"task(\d+)-", task["revision"])[1])
    root, evidence = pair / run, pair / "evidence" / run
    summary = json.loads((evidence / "summary.json").read_text())
    if number in (1, 2, 3, 4, 5, 6, 8, 9, 12):
        result = LEGACY.grade(pair, run)
        result["explorer_cli_invocations"] = summary["cli_invocations"]
        (evidence / "artifact-grade.json").write_text(json.dumps(result, indent=2) + "\n")
        return result

    checks, missing = {}, []
    binary = pair / summary["condition"] / "target/debug/locmetry"
    env = dict(os.environ, XDG_CONFIG_HOME=str(pair / "user-config" / run), NO_COLOR="1", LC_ALL="C",
               COLUMNS=str(manifest.get("environment", {}).get("COLUMNS", "100")))

    def check(name, value):
        checks[name] = bool(value)

    def load(name):
        path = root / name
        if not path.is_file():
            missing.append(name)
            return None
        try:
            document = json.loads(path.read_text())
            check("object:" + name, isinstance(document, dict) and bool(document))
            return document if isinstance(document, dict) and document else None
        except (json.JSONDecodeError, UnicodeError):
            check("json:" + name, False)
            return None

    def probe(label, args):
        directory = evidence / "evaluator-probes"
        directory.mkdir(exist_ok=True)
        identity = f"probe-{len(list(directory.glob('*.stdout'))) + 1:03}"
        started = time.monotonic()
        result = subprocess.run([str(binary), *args], cwd=root, env=env, capture_output=True, text=True, timeout=30)
        (directory / (identity + ".stdout")).write_text(result.stdout)
        (directory / (identity + ".stderr")).write_text(result.stderr)
        record = dict(id=identity, label=label, args=args, exit=result.returncode, elapsed=time.monotonic() - started)
        with (directory / "calls.jsonl").open("a") as handle:
            handle.write(json.dumps(record) + "\n")
        check("probe-success:" + label, result.returncode == 0)
        return result

    def project_ids(policy, kind):
        entries = policy.get("languages", {}).get("mappings", []) if kind == "language_mapping" else policy.get("overrides", [])
        return {entry["id"]: entry for entry in entries}

    def file_matches(label, file, expected, policy):
        for key, value in expected.items():
            if key in ("stage", "reason_required"):
                continue
            actual = file.get(key)
            if value == "project-defined":
                entry = project_ids(policy, key).get(actual)
                check(label + ":" + key, entry is not None)
                if key == "rule":
                    check(label + ":intentional-other-reason", entry is not None and bool(entry.get("reason", "").strip()))
            else:
                check(label + ":" + key, actual == value)
        if "language_mapping" not in expected:
            check(label + ":backend-recognition", "language_mapping" not in file)

    def report_matches(case, report, policy):
        if report is None:
            return
        expected = task["oracle"][case]
        check(case + ":full-report", {"files", "review", "breakdown", "metadata", "summary"}.issubset(report)
              and {"config", "coverage", "modules", "scope"}.issubset(report.get("metadata", {})))
        for section in ("source", "all_counted"):
            check(case + ":" + section, report.get("summary", {}).get(section) == expected[section])
        for purpose, code in expected["purpose_code"].items():
            check(case + ":purpose:" + purpose, report.get("summary", {}).get("source_purpose", {}).get(purpose, {}).get("code", 0) == code)
        coverage = report["metadata"]["coverage"]
        for key, value in expected["coverage"].items():
            check(case + ":coverage:" + key, coverage.get(key) == value)
        unrecognized = expected.get("unrecognized", [])
        check(case + ":unrecognized", coverage.get("unrecognized") == unrecognized)
        expected_paths = set(expected["selected_paths"]) - set(unrecognized)
        files = report.get("files", [])
        actual_paths = [file["path"] for file in files]
        check(case + ":counted-paths", set(actual_paths) == expected_paths and len(actual_paths) == len(expected_paths))
        review = expected.get("review", task["oracle"].get("review", []))
        check(case + ":review", [file["path"] for file in report.get("review", [])] == review)
        for file in files:
            expected_file = task["oracle"]["decisions"].get(file["path"])
            if expected_file is not None:
                file_matches(case + ":file:" + file["path"], file, expected_file, policy)

    def measurement(case, artifact, args, policy):
        saved = load(artifact)
        actual = json.loads(probe(case, args + ["--files", "--format", "json"]).stdout)
        report_matches(case, saved, policy)
        # Frozen physical-file oracles settle correctness; replay additionally detects stale policy evidence.
        if saved is not None:
            check(case + ":saved-matches-replay", without_measurement_time(saved) == without_measurement_time(actual))
        return actual

    try:
        check("process-completed", summary["returncode"] == 0 and summary["stopped"] is None)
        check("protected-fixture-intact", summary["fixture_intact"])
        audit = json.loads((evidence / "context-audit.json").read_text())
        check("context-isolated", audit["repository_context"] is False and bool(audit["configurations"]))
        count = summary["cli_invocations"]
        calls = [json.loads(line) for line in (evidence / "calls.jsonl").read_text().splitlines()][:count]
        check("explorer-call-boundary", len(calls) == count)
        (evidence / "agent-calls.jsonl").write_text("".join(json.dumps(call) + "\n" for call in calls))
        mutable = set(task.get("mutable", []))
        for name, content in task["files"].items():
            path = root / name
            if name not in mutable:
                check("fixture:" + name, path.is_file() and not path.is_symlink() and path.read_bytes() == content.encode())
        project_paths = list((root / "projects").rglob("*"))
        expected_paths = set(task["files"]) | mutable
        check("no-extra-project-content", all(not path.is_symlink() and (not path.is_file() or str(path.relative_to(root)) in expected_paths) for path in project_paths))
        for name in task["oracle"].get("artifacts", []):
            if not (root / name).is_file():
                missing.append(name)

        if number == 7:
            query_flags = {"--help", "-h", "--version", "-V", "--schema", "--guide"}
            check("discovery-only", all(call["exit"] == 1 or (call["args"] and call["args"][0] == "help")
                                        or any(arg.split("=", 1)[0] in query_flags for arg in call["args"]) for call in calls))
            allowed = set(task["files"]) | {"bin/locmetry"}
            check("read-only-scratch", all(str(path.relative_to(root)) in allowed for path in root.rglob("*") if path.is_file()))
        elif number == 10:
            basic = ["--root", "projects/grove", "--subfolder", "src", "--no-user-config", "--exclude", "src/scratch/**", "--exclude", "src/hold/**"]
            opened = basic + ["--hidden", "true", "--respect-ignore-files", "false"]
            measurement("default", "outputs/default.json", basic, {})
            measurement("open", "outputs/open.json", opened, {})
            for label, artifact, args in [
                ("table", "outputs/open-table.txt", ["--format", "table", "--purpose", "production", "--categories", "--bars=comments", "--sort", "name", "--folder-rows", "2"]),
                ("markdown", "outputs/open.md", ["--format", "markdown", "--sort", "name", "--depth", "2"]),
            ]:
                result = probe(label, opened + args)
                if (root / artifact).is_file():
                    check(label + ":requested-view", human_without_time((root / artifact).read_text()) == human_without_time(result.stdout))
            saved = load("outputs/subroot-app.json")
            actual = json.loads(probe("subroot-app", ["explain", "core/app.js", "--root", "projects/grove/src", "--no-user-config", "--format", "json"]).stdout)
            if saved is not None:
                check("subroot:saved-matches-replay", saved == actual)
                check("subroot:state", saved["result"]["state"] == task["oracle"]["subroot_app"]["state"])
                check("subroot:stage", saved["result"]["decision"]["stage"] == "fallback")
                file_matches("subroot:file", saved["result"]["file"], {key: value for key, value in task["oracle"]["subroot_app"].items() if key != "state"}, {})
        elif number == 11:
            config_name = "projects/worker/.locmetryrc"
            policy = tomllib.loads((root / config_name).read_text())
            original = tomllib.loads(task["files"][config_name])
            check("project-policy-preserved", preserved_policy(original, policy))
            defaults = pair / "user-config" / run / "locmetry/config.toml"
            check("personal-defaults-preserved", defaults.is_file() and defaults.read_text() == task["user_defaults"])
            basic = ["--root", "projects/worker"]
            actuals = {
                "with_user_defaults": measurement("with_user_defaults", "outputs/configured.json", basic, policy),
                "without_user_defaults": measurement("without_user_defaults", "outputs/no-user.json", basic + ["--no-user-config"], policy),
            }
            for case, actual in actuals.items():
                check(case + ":personal-path", bool(actual["metadata"].get("user_config_path")) == (case == "with_user_defaults"))
                check(case + ":unused-entry", actual["summary"].get("unused_entries", []) == task["oracle"]["unused_entries"])
            for path, expected in task["oracle"]["decisions"].items():
                with_user = path in task["oracle"]["with_user_defaults"]["selected_paths"]
                args = ["explain", path, *basic, "--format", "json"]
                if not with_user:
                    args.append("--no-user-config")
                result = json.loads(probe("decision:" + path, args).stdout)["result"]
                check("decision-state:" + path, result["state"] == "counted")
                check("decision-stage:" + path, result["decision"]["stage"] == expected["stage"])
                file_matches("decision-file:" + path, result["file"], expected, policy)
                if expected.get("reason_required"):
                    check("decision-reason:" + path, bool(result["decision"].get("reason", "").strip()))
        else:
            check("supported-task", False)
    except (AttributeError, IndexError, KeyError, TypeError, ValueError, OSError, subprocess.TimeoutExpired) as error:
        check("artifact-structure-or-execution", False)
        checks["error"] = str(error)

    result = dict(task=number, condition=summary["condition"], outcome="artifact_checks_pass" if all(value is True for value in checks.values()) and not missing else "needs_review",
                  checks=checks, missing=sorted(set(missing)), interpretation="pending", explorer_cli_invocations=summary["cli_invocations"])
    (evidence / "artifact-grade.json").write_text(json.dumps(result, indent=2) + "\n")
    return result


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("campaign", type=Path)
    args = parser.parse_args(argv)
    for path in sorted(args.campaign.glob("task*-rep*/evidence/run-*/summary.json")):
        evidence = path.parent
        if (evidence / "artifact-grade.json").exists():
            continue
        result = grade(path.parents[2], evidence.name)
        print(path.parents[2].name, evidence.name, json.dumps(result))


if __name__ == "__main__":
    main()
