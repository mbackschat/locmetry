"""Regression checks for the file-size gate over a saved Locmetry report."""

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("check-file-sizes.py")
LIMITS = {"production": {"soft": 10, "hard": 20}, "test": {"soft": 30, "hard": 40}}


def record(path, code, purpose="production", kind="source"):
    return {"path": path, "kind": kind, "purpose": purpose, "counts": {"code": code}}


class FileSizeGateTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)

    def gate(self, files, ceilings=None):
        (self.root / "report.json").write_text(json.dumps({"files": files}))
        limits = {"limits": LIMITS, "ceilings": ceilings or {}}
        (self.root / "limits.json").write_text(json.dumps(limits))
        result = subprocess.run(
            [sys.executable, str(SCRIPT), "--report", str(self.root / "report.json"),
             "--limits", str(self.root / "limits.json")],
            capture_output=True, text=True)
        return result.returncode, result.stdout + result.stderr

    def test_files_within_hard_limits_pass_and_soft_excess_only_warns(self):
        status, output = self.gate([record("src/a.rs", 15), record("tests/a.rs", 35, "test")])
        self.assertEqual(status, 0, output)
        self.assertIn("warning: src/a.rs: 15 code lines exceed the production soft limit of 10", output)
        self.assertIn("warning: tests/a.rs: 35 code lines exceed the test soft limit of 30", output)

    def test_a_file_over_its_purpose_hard_limit_fails(self):
        status, output = self.gate([record("src/a.rs", 21), record("tests/a.rs", 40, "test")])
        self.assertEqual(status, 1, output)
        self.assertIn("error: src/a.rs: 21 code lines exceed the production hard limit of 20", output)
        self.assertNotIn("tests/a.rs", output.replace("warning: tests/a.rs", ""))

    def test_purposes_without_limits_and_non_source_are_not_checked(self):
        files = [record("gen/a.rs", 500, "other"), record("data.json", 500, kind="data")]
        self.assertEqual(self.gate(files), (0, ""))

    def test_a_ceiling_lets_an_existing_file_stay_but_not_grow(self):
        ceiling = {"src/big.rs": 50}
        self.assertEqual(self.gate([record("src/big.rs", 50)], ceiling)[0], 0)
        status, output = self.gate([record("src/big.rs", 51)], ceiling)
        self.assertEqual(status, 1, output)
        self.assertIn("error: src/big.rs: 51 code lines exceed its ceiling of 50", output)

    def test_a_shrunk_file_asks_to_lower_its_ceiling(self):
        status, output = self.gate([record("src/big.rs", 45)], {"src/big.rs": 50})
        self.assertEqual(status, 0, output)
        self.assertIn("warning: src/big.rs: ceiling 50 can be lowered to 45", output)

    def test_ceilings_for_fitting_or_missing_files_fail_as_stale(self):
        ceilings = {"src/fits.rs": 50, "src/gone.rs": 50}
        status, output = self.gate([record("src/fits.rs", 20)], ceilings)
        self.assertEqual(status, 1, output)
        self.assertIn("error: src/fits.rs: fits the production hard limit of 20; remove its ceiling", output)
        self.assertIn("error: src/gone.rs: ceiling names no measured source file; remove it", output)


if __name__ == "__main__":
    unittest.main()
