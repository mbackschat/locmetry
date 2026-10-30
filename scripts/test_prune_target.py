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

    def hook(self, command, cwd):
        event = {"tool_name": "Bash", "cwd": str(cwd), "tool_input": {"command": command}}
        return subprocess.run(
            [sys.executable, str(SCRIPT), "--hook", "--max-kib", "4", "--floor-kib", "2"],
            input=json.dumps(event), capture_output=True, text=True, check=True,
            env={k: v for k, v in os.environ.items() if k != "CARGO_TARGET_DIR"},
        )

    def project(self, name):
        root = Path(self.tmp.name) / name
        (root / "target").mkdir(parents=True)
        (root / "Cargo.toml").write_text("[package]\n")
        return root

    def unit(self, root, kib=8):
        path = root / "target/debug/deps/libold.rlib"
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b"x" * kib * KIB)
        os.utime(path, (1_000_000_000, 1_000_000_000))
        return path

    def test_the_hook_ignores_commands_without_cargo(self):
        root = self.project("app")
        old = self.unit(root)
        self.hook("git status", root)
        self.assertTrue(old.exists())

    def test_the_hook_prunes_after_cargo_and_reports_it(self):
        root = self.project("app")
        old = self.unit(root)
        result = self.hook("cargo test --locked", root)
        self.assertFalse(old.exists())
        self.assertIn("pruned", json.loads(result.stdout)["systemMessage"])

    def test_the_hook_prunes_the_project_the_command_entered_not_the_session_one(self):
        session, sibling = self.project("app"), self.project("fork")
        kept, pruned = self.unit(session), self.unit(sibling)
        self.hook(f"cd {sibling} && cargo build --release 2>&1 | tail -1", session)
        self.assertTrue(kept.exists())
        self.assertFalse(pruned.exists())

    def test_the_hook_follows_relative_cd_manifest_paths_and_target_dirs(self):
        session, fork = self.project("app"), self.project("fork")
        tool = self.project("fork/judges/tool")
        tool_unit = self.unit(tool)
        speed_unit = self.unit(fork / "speed")
        self.hook(f"cd {fork}; cd judges/tool && cargo build", session)
        self.hook(f"cargo build --manifest-path {fork}/Cargo.toml --target-dir={fork}/speed/target", session)
        self.assertFalse(tool_unit.exists())
        self.assertFalse(speed_unit.exists())

    def test_a_unit_still_read_is_kept_over_one_written_later_but_unused(self):
        read = self.artifact("debug/deps/libregex-a.rlib", 4, age_days=30)
        os.utime(read, (1_700_000_000, read.stat().st_mtime))
        unused = self.artifact("debug/deps/libtokei-old.rlib", 4, age_days=9)
        self.prune("--max-kib", "6", "--floor-kib", "4")
        self.assertTrue(read.exists())
        self.assertFalse(unused.exists())


if __name__ == "__main__":
    unittest.main()
