"""Guard experiment integrity when tasks mutate only authorized fixture files."""

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest

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
