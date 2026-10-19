"""Guard experiment integrity when tasks mutate only authorized fixture files."""

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SPEC = importlib.util.spec_from_file_location("benchmark", Path(__file__).with_name("agent-explorability.py"))
BENCH = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BENCH)
SCORE_SPEC = importlib.util.spec_from_file_location("scoring", Path(__file__).with_name("score_agent_explorability.py"))
SCORING = importlib.util.module_from_spec(SCORE_SPEC)
SCORE_SPEC.loader.exec_module(SCORING)


class ExperimentTests(unittest.TestCase):
    def test_an_unfinished_last_telemetry_record_does_not_hide_earlier_events(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "events.jsonl"
            path.write_text('{"event": 1}\n{"event":')
            self.assertEqual(BENCH.records(path), [{"event": 1}])

    def test_corrupt_completed_telemetry_cannot_silently_lower_call_counts(self):
        with tempfile.TemporaryDirectory() as tmp:
            path = Path(tmp) / "events.jsonl"
            path.write_text('{"event": 1}\nbroken\n{"event": 3}\n')
            with self.assertRaises(json.JSONDecodeError):
                BENCH.records(path)

    def test_task_seed_does_not_substitute_the_report_understanding_fixture(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            BENCH.seed(root, {"files": {"projects/partial/empty.py": ""}})
            self.assertTrue((root / "projects/partial/empty.py").exists())
            self.assertFalse((root / "projects/web").exists())

    def test_help_recipe_treatment_retains_guides_and_other_product_sources(self):
        with tempfile.TemporaryDirectory() as tmp:
            pair = Path(tmp) / "pair"
            BENCH.prepare(pair, treatment="help-recipe")
            a = (pair / "A/src/main.rs").read_text()
            b = (pair / "B/src/main.rs").read_text()
            self.assertIn("guide: Option<Guide>", b)
            self.assertIn("Classify maintained scripts", b)
            self.assertNotIn("Classify maintained scripts", a)
            for path in (pair / "A/src").glob("*.rs"):
                if path.name != "main.rs":
                    self.assertEqual(path.read_bytes(), (pair / "B/src" / path.name).read_bytes())
            manifest = json.loads((pair / "manifest.json").read_text())
            self.assertEqual(manifest["treatment"], "help-recipe")

    def test_campaign_honors_selected_tasks_repetitions_and_treatment(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            binary = root / "binary"
            binary.write_text("test binary")
            with patch.object(BENCH, "run_agent") as run:
                BENCH.campaign(root / "campaign", [binary, binary], "test", "test",
                               treatment="help-recipe", task_ids=[1, 3], repetitions=2)
            self.assertEqual(run.call_count, 8)
            plan = json.loads((root / "campaign/campaign.json").read_text())
            self.assertEqual(plan["treatment"], "help-recipe")
            self.assertEqual({p["task"] for p in plan["pairs"]}, {1, 3})
            self.assertEqual(sum(p["first"] == "A" for p in plan["pairs"]), 2)
            manifests = list((root / "campaign").glob("task*-rep*/manifest.json"))
            self.assertEqual(len(manifests), 4)
            self.assertTrue(all(json.loads(p.read_text())["treatment"] == "help-recipe" for p in manifests))

    def test_independent_config_case_rejects_wrong_saved_decisions(self):
        self.check_configuration_case(8)

    def test_simple_config_case_rejects_wrong_saved_decisions(self):
        self.check_configuration_case(9)

    def check_configuration_case(self, number):
        task = json.loads(BENCH.TASK.with_name(f"task{number}.json").read_text())
        oracle = task["oracle"]
        report = {"summary": {"source": oracle["source"], "all_counted": oracle["all_counted"],
                              "source_purpose": {k: {"code": v} for k, v in oracle["purpose_code"].items()}},
                  "files": [{"path": p, **v} for p, v in oracle["decisions"].items()],
                  "review": [{"path": p} for p in oracle["review"]]}
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            evidence = root / "evidence/run-02"
            evidence.mkdir(parents=True)
            (root / "manifest.json").write_text(json.dumps({"task": task}))
            (evidence / "summary.json").write_text(json.dumps({"condition": "A", "returncode": 0, "stopped": None, "fixture_intact": True, "cli_invocations": 0}))
            (evidence / "context-audit.json").write_text(json.dumps({"repository_context": False, "configurations": [{"model": "test"}]}))
            (evidence / "calls.jsonl").write_text("")
            for name in oracle["artifacts"]:
                target = root / "run-02" / name
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_text(json.dumps(report) if name.endswith(".json") else "version = 1")
            actual = SCORING.subprocess.CompletedProcess([], 0, json.dumps(report), "")
            with patch.object(SCORING.subprocess, "run", return_value=actual):
                self.assertEqual(SCORING.grade(root, "run-02")["outcome"], "artifact_checks_pass")
                report["files"][0]["category"] = "incorrect"
                for bad in [report, {}, []]:
                    (root / "run-02/outputs/configured.json").write_text(json.dumps(bad))
                    self.assertEqual(SCORING.grade(root, "run-02")["outcome"], "needs_review")
                task["revision"] = "task10-unsupported-v1"
                (root / "manifest.json").write_text(json.dumps({"task": task}))
                self.assertEqual(SCORING.grade(root, "run-02")["outcome"], "needs_review")

    def test_process_success_does_not_mask_incorrect_delivered_counts(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            evidence = root / "evidence/run-02"
            evidence.mkdir(parents=True)
            task = json.loads(BENCH.TASK.read_text())
            (root / "manifest.json").write_text(json.dumps({"task": task}))
            (evidence / "summary.json").write_text(json.dumps({"condition": "A", "returncode": 0, "stopped": None, "fixture_intact": True, "cli_invocations": 1}))
            (evidence / "context-audit.json").write_text(json.dumps({"repository_context": False, "configurations": [{"model": "test"}]}))
            (evidence / "calls.jsonl").write_text(json.dumps({"id": "report", "args": ["--format", "json"]}) + "\n")
            summary = {key: dict(task["oracle"][key]) for key in ["source", "all_counted"]}
            (evidence / "report.stdout").write_text(json.dumps({"summary": summary}))
            self.assertEqual(SCORING.grade(root, "run-02")["outcome"], "artifact_checks_pass")
            summary["source"]["code"] = 999
            (evidence / "report.stdout").write_text(json.dumps({"summary": summary}))
            result = SCORING.grade(root, "run-02")
            self.assertEqual(result["outcome"], "needs_review")
            self.assertFalse(result["checks"]["source:code"])


if __name__ == "__main__":
    unittest.main()
