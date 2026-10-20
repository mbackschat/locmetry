"""Protect independent artifact scoring for the complete help study."""

import copy
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import tempfile
import tomllib
import unittest
from unittest.mock import patch


SPEC = importlib.util.spec_from_file_location("full_help_score", Path(__file__).with_name("score_full_help.py"))
SCORE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(SCORE)
FIXTURES = Path(__file__).resolve().parent.parent / "tests/fixtures/agent-explorability"


class FullHelpScoringTests(unittest.TestCase):
    def prepare(self, number):
        temp = tempfile.TemporaryDirectory()
        self.addCleanup(temp.cleanup)
        pair = Path(temp.name).resolve() / f"task{number}-rep1"
        root = pair / "run-01"
        evidence = pair / "evidence/run-01"
        evidence.mkdir(parents=True)
        name = "task7-full-help.json" if number == 7 else f"task{number}.json"
        task = json.loads((FIXTURES / name).read_text())
        for relative, content in task["files"].items():
            target = root / relative
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(content)
        (root / "bin").mkdir()
        (root / "bin/locmetry").write_text("launcher")
        (pair / "manifest.json").write_text(json.dumps({"task": task, "environment": {"COLUMNS": "100"}}))
        (evidence / "summary.json").write_text(json.dumps({"condition": "B", "returncode": 0, "stopped": None, "fixture_intact": True, "cli_invocations": 2}))
        (evidence / "context-audit.json").write_text(json.dumps({"repository_context": False, "configurations": ["isolated"]}))
        calls = [{"id": str(i), "args": args, "exit": 0} for i, args in enumerate([["--help"], ["--schema", "config"], ["projects/web"]])]
        (evidence / "calls.jsonl").write_text("".join(json.dumps(c) + "\n" for c in calls))
        if "user_defaults" in task:
            defaults = pair / "user-config/run-01/locmetry/config.toml"
            defaults.parent.mkdir(parents=True)
            defaults.write_text(task["user_defaults"])
        return pair, root, evidence, task

    def configure(self, root):
        path = root / "projects/worker/.locmetryrc"
        text = path.read_text().replace('automation = "Maintained source checks"\n', 'automation = "Maintained source checks"\nhistorical = "Archived evidence"\n')
        text += '\n[[languages.mappings]]\nid = "project-fragments"\nmatch = ["src/*.fragment"]\nlanguage = "Rust"\n\n[[overrides]]\nid = "archived-replay"\nmatch = ["src/replay_test.py"]\npurpose = "other"\ncategory = "historical"\nreason = "Archived evidence, not a maintained test"\n'
        path.write_text(text)
        return tomllib.loads(text)

    def report(self, task, case, policy=None):
        oracle = task["oracle"]
        expected = oracle[case]
        unrecognized = expected.get("unrecognized", [])
        files = []
        for path in expected["selected_paths"]:
            if path in unrecognized:
                continue
            file = {k: copy.deepcopy(v) for k, v in oracle["decisions"][path].items() if k not in ("stage", "reason_required")}
            file["path"] = path
            if file.get("rule") == "project-defined":
                file["rule"] = "archived-replay"
            if file.get("language_mapping") == "project-defined":
                file["language_mapping"] = "project-fragments"
            files.append(file)
        coverage = dict(expected["coverage"], unrecognized=unrecognized)
        return {
            "summary": {"source": expected["source"], "all_counted": expected["all_counted"], "source_purpose": {k: {"code": v} for k, v in expected["purpose_code"].items()}, "unused_entries": oracle.get("unused_entries", [])},
            "files": files,
            "review": [{"path": path} for path in expected.get("review", oracle.get("review", []))],
            "breakdown": [],
            "metadata": {"coverage": coverage, "scope": {"subfolder": "src"} if "subfolder" in oracle else {}, "modules": [{"path": ".", "detector": "npm" if "subfolder" in oracle else "cargo"}], "config": policy or {}, "measured_at_unix_seconds": 1, "user_config_path": None if case == "without_user_defaults" else "isolated/config.toml"},
        }

    def save(self, root, path, value):
        target = root / path
        target.parent.mkdir(exist_ok=True)
        target.write_text(json.dumps(value) if isinstance(value, dict) else value)

    def delivered(self, number):
        pair, root, evidence, task = self.prepare(number)
        policy = self.configure(root) if number == 11 else None
        cases = {"default": "outputs/default.json", "open": "outputs/open.json"} if number == 10 else {"with_user_defaults": "outputs/configured.json", "without_user_defaults": "outputs/no-user.json"}
        reports = {case: self.report(task, case, policy) for case in cases}
        for case, path in cases.items():
            self.save(root, path, reports[case])
        table = "Production categories and comment bars\nMeasured at: 2026-01-01 00:00:00 UTC\n"
        markdown = "# Sorted depth two\nMeasured at: 2026-01-01 00:00:00 UTC\n"
        control = {"document_type": "explanation", "result": {"state": "counted", "file": {k: v for k, v in task["oracle"].get("subroot_app", {}).items() if k not in ("state", "stage")}, "decision": {"stage": "fallback"}}}
        if number == 10:
            self.save(root, "outputs/open-table.txt", table)
            self.save(root, "outputs/open.md", markdown)
            self.save(root, "outputs/subroot-app.json", control)

        def probe(args, **kwargs):
            argv = args[1:]
            self.assertIn("/B/target/debug/locmetry", args[0])
            self.assertEqual(kwargs["env"]["XDG_CONFIG_HOME"], str(pair / "user-config/run-01"))
            if argv[0] == "explain":
                if number == 10:
                    value = control
                else:
                    case = "without_user_defaults" if "--no-user-config" in argv else "with_user_defaults"
                    file = next(f for f in reports[case]["files"] if f["path"] == argv[1])
                    expected = task["oracle"]["decisions"][argv[1]]
                    value = {"document_type": "explanation", "result": {"state": "counted", "file": file, "decision": {"stage": expected["stage"], "reason": "Archived evidence"}}}
            elif "table" in argv:
                return subprocess.CompletedProcess(args, 0, table.replace("2026-01-01", "2026-10-20"), "")
            elif "markdown" in argv:
                return subprocess.CompletedProcess(args, 0, markdown.replace("2026-01-01", "2026-10-20"), "")
            else:
                case = ("open" if "--hidden" in argv else "default") if number == 10 else ("without_user_defaults" if "--no-user-config" in argv else "with_user_defaults")
                value = reports[case]
            return subprocess.CompletedProcess(args, 0, json.dumps(value), "")

        return pair, root, evidence, task, probe

    def test_correct_scope_artifacts_pass_and_evaluator_calls_stay_outside_explorer_costs(self):
        pair, root, evidence, task, probe = self.delivered(10)
        with patch.object(SCORE.subprocess, "run", side_effect=probe):
            result = SCORE.grade(pair, "run-01")
        self.assertEqual(result["outcome"], "artifact_checks_pass", result)
        self.assertEqual(result["condition"], "B")
        self.assertEqual(result["explorer_cli_invocations"], 2)
        self.assertEqual(len((evidence / "agent-calls.jsonl").read_text().splitlines()), 2)
        self.assertTrue((evidence / "evaluator-probes/calls.jsonl").is_file())
        self.assertEqual(len((evidence / "calls.jsonl").read_text().splitlines()), 3)
        self.assertEqual(result["interpretation"], "pending")

    def test_wrong_counts_paths_and_decisions_fail_even_when_json_is_well_formed(self):
        for defect in ("counts", "paths", "decision"):
            with self.subTest(defect=defect):
                pair, root, evidence, task, probe = self.delivered(10)
                path = root / "outputs/open.json"
                report = json.loads(path.read_text())
                if defect == "counts":
                    report["summary"]["source"]["code"] += 1
                elif defect == "paths":
                    report["files"][0]["path"] = "src/invented.js"
                else:
                    report["files"][0]["purpose"] = "other"
                path.write_text(json.dumps(report))
                with patch.object(SCORE.subprocess, "run", side_effect=probe):
                    result = SCORE.grade(pair, "run-01")
                self.assertEqual(result["outcome"], "needs_review", result)

    def test_missing_artifacts_and_wrong_human_views_need_review(self):
        for defect in ("missing", "presentation"):
            with self.subTest(defect=defect):
                pair, root, evidence, task, probe = self.delivered(10)
                if defect == "missing":
                    (root / "outputs/open.md").unlink()
                else:
                    (root / "outputs/open-table.txt").write_text("Unfiltered table without requested detail\n")
                with patch.object(SCORE.subprocess, "run", side_effect=probe):
                    result = SCORE.grade(pair, "run-01")
                self.assertEqual(result["outcome"], "needs_review", result)

    def test_correct_advanced_policy_passes_with_arbitrary_declared_project_ids(self):
        pair, root, evidence, task, probe = self.delivered(11)
        with patch.object(SCORE.subprocess, "run", side_effect=probe):
            result = SCORE.grade(pair, "run-01")
        self.assertEqual(result["outcome"], "artifact_checks_pass", result)

    def test_personal_mapping_provenance_cannot_impersonate_a_project_mapping(self):
        pair, root, evidence, task, probe = self.delivered(11)
        path = root / "outputs/configured.json"
        report = json.loads(path.read_text())
        next(f for f in report["files"] if f["path"] == "src/extra.fragment")["language_mapping"] = "personal-fragments"
        path.write_text(json.dumps(report))
        with patch.object(SCORE.subprocess, "run", side_effect=probe):
            result = SCORE.grade(pair, "run-01")
        self.assertEqual(result["outcome"], "needs_review", result)

    def test_supplied_rules_and_immutable_source_are_checked_independently_of_harness_summary(self):
        for defect in ("rule", "source"):
            with self.subTest(defect=defect):
                pair, root, evidence, task, probe = self.delivered(11)
                path = root / ("projects/worker/.locmetryrc" if defect == "rule" else "projects/worker/src/lib.rs")
                path.write_text(path.read_text().replace('id = "source-checks"', 'id = "changed-rule"') if defect == "rule" else "pub const CHANGED: bool = true;\n")
                with patch.object(SCORE.subprocess, "run", side_effect=probe):
                    result = SCORE.grade(pair, "run-01")
                self.assertEqual(result["outcome"], "needs_review", result)

    def test_discovery_allows_queries_and_leaves_semantic_grading_pending(self):
        pair, root, evidence, task = self.prepare(7)
        result = SCORE.grade(pair, "run-01")
        self.assertEqual(result["outcome"], "artifact_checks_pass", result)
        self.assertEqual(result["interpretation"], "pending")
        self.assertEqual(result["explorer_cli_invocations"], 2)

    def test_discovery_rejects_a_measurement_inside_the_explorer_boundary(self):
        pair, root, evidence, task = self.prepare(7)
        summary = json.loads((evidence / "summary.json").read_text())
        summary["cli_invocations"] = 3
        (evidence / "summary.json").write_text(json.dumps(summary))
        self.assertEqual(SCORE.grade(pair, "run-01")["outcome"], "needs_review")

    def test_original_tasks_delegate_to_the_existing_artifact_grader(self):
        pair, root, evidence, task = self.prepare(9)
        legacy_result = {"task": 9, "condition": "B", "outcome": "artifact_checks_pass", "checks": {"legacy": True}, "missing": [], "interpretation": "pending"}
        with patch.object(SCORE.LEGACY, "grade", return_value=legacy_result) as legacy:
            result = SCORE.grade(pair, "run-01")
        legacy.assert_called_once_with(pair, "run-01")
        self.assertTrue(result["checks"]["legacy"])

    def test_malformed_report_structure_returns_review_without_aborting_the_campaign(self):
        pair, root, evidence, task, probe = self.delivered(10)
        path = root / "outputs/open.json"
        report = json.loads(path.read_text())
        report["summary"] = "not an object"
        path.write_text(json.dumps(report))
        with patch.object(SCORE.subprocess, "run", side_effect=probe):
            result = SCORE.grade(pair, "run-01")
        self.assertEqual(result["outcome"], "needs_review")

    def test_directory_runner_scores_both_run_names_using_each_summary_condition(self):
        pair, root, evidence, task = self.prepare(7)
        import shutil
        shutil.copytree(root, pair / "run-02")
        shutil.copytree(evidence, pair / "evidence/run-02")
        summary_path = pair / "evidence/run-02/summary.json"
        summary = json.loads(summary_path.read_text())
        summary["condition"] = "A"
        summary_path.write_text(json.dumps(summary))
        output = io.StringIO()
        with patch("sys.stdout", output):
            SCORE.main([str(pair.parent)])
        self.assertEqual(len(output.getvalue().splitlines()), 2)
        for run, condition in [("run-01", "B"), ("run-02", "A")]:
            result = json.loads((pair / "evidence" / run / "artifact-grade.json").read_text())
            self.assertEqual(result["condition"], condition)
            self.assertEqual(result["outcome"], "artifact_checks_pass")


if __name__ == "__main__":
    unittest.main()
