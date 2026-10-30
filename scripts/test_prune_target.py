"""Regression checks for pruning Cargo's target directory oldest-first."""

import fcntl
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("prune-target.py")
KIB = 1024


class PruneTargetTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.target = Path(self.tmp.name) / "target"

    def artifact(self, relative, kib, age_days):
        path = self.target / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b"x" * kib * KIB)
        stamp = 1_700_000_000 - age_days * 86_400
        os.utime(path, (stamp, stamp))
        return path

    def prune(self, *args, stdin=None):
        return subprocess.run(
            [sys.executable, str(SCRIPT), "--target", str(self.target), *args],
            input=stdin, capture_output=True, text=True, check=True,
        )

    def test_under_the_cap_nothing_is_removed(self):
        kept = self.artifact("debug/deps/libold-1.rlib", 4, age_days=9)
        self.prune("--max-kib", "8", "--floor-kib", "4")
        self.assertTrue(kept.exists())

    def test_over_the_cap_the_oldest_units_go_until_the_floor(self):
        oldest = self.artifact("debug/deps/libtokei-old.rlib", 4, age_days=9)
        older = self.artifact("debug/incremental/tokei-old/s-1/query.bin", 4, age_days=8)
        newest = self.artifact("debug/deps/libtokei-new.rlib", 4, age_days=1)
        self.prune("--max-kib", "8", "--floor-kib", "4")
        self.assertFalse(oldest.exists())
        # Incremental state goes as a whole crate directory, never file by file.
        self.assertFalse(older.parent.parent.exists())
        self.assertTrue(newest.exists())

    def test_a_profile_locked_by_a_running_cargo_is_left_alone(self):
        old = self.artifact("debug/deps/libold.rlib", 8, age_days=9)
        self.artifact("debug/.cargo-lock", 0, age_days=0)
        with open(self.target / "debug/.cargo-lock", "w") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            self.prune("--max-kib", "4", "--floor-kib", "2")
        self.assertTrue(old.exists())

    def test_the_hook_ignores_commands_without_cargo(self):
        old = self.artifact("debug/deps/libold.rlib", 8, age_days=9)
        event = {"tool_name": "Bash", "tool_input": {"command": "git status"}}
        self.prune("--hook", "--max-kib", "4", "--floor-kib", "2", stdin=json.dumps(event))
        self.assertTrue(old.exists())

    def test_the_hook_prunes_after_cargo_and_reports_it(self):
        old = self.artifact("debug/deps/libold.rlib", 8, age_days=9)
        event = {"tool_name": "Bash", "tool_input": {"command": "cd x && cargo test --locked"}}
        result = self.prune("--hook", "--max-kib", "4", "--floor-kib", "2", stdin=json.dumps(event))
        self.assertFalse(old.exists())
        self.assertIn("pruned", json.loads(result.stdout)["systemMessage"])


if __name__ == "__main__":
    unittest.main()
