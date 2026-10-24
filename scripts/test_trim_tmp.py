"""Regression checks for the temporary-directory trimming command."""

import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest

SCRIPT = Path(__file__).with_name("trim-tmp.py")
HOUR = 3600


class TrimTmpTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        root = Path(self.tmp.name)
        self.temp = root / "tmp"
        self.sessions = root / "sessions"
        self.temp.mkdir()
        self.sessions.mkdir()

    def entry(self, parent, name, age, files=("log.txt",)):
        path = parent / name
        for file in files:
            (path / file).parent.mkdir(parents=True, exist_ok=True)
            (path / file).write_text("x")
        stamp = time.time() - age
        for item in [path, *path.rglob("*")]:
            os.utime(item, (stamp, stamp))
        return path

    def run_command(self):
        result = subprocess.run([sys.executable, str(SCRIPT), "--temp-root", str(self.temp), "--sessions", str(self.sessions)],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        return result.stdout

    def test_keeps_the_newest_locmetry_entry_and_removes_older_ones(self):
        newest = self.entry(self.temp, "locmetry-showboat-new", age=3 * HOUR)
        older = self.entry(self.temp, "locmetry-build-old", age=5 * HOUR)
        other = self.entry(self.temp, "other-project-cache", age=9 * HOUR)
        output = self.run_command()
        self.assertTrue(newest.exists())
        self.assertFalse(older.exists())
        self.assertTrue(other.exists())
        self.assertIn("removed 1", output)

    def test_never_removes_the_root_links_or_trees_it_does_not_own_by_name(self):
        outside = self.entry(Path(self.tmp.name), "outside", age=9 * HOUR)
        (self.temp / "locmetry-link").symlink_to(outside)
        self.entry(self.temp, "locmetry-new", age=2 * HOUR)
        foreign = self.entry(self.temp, "notlocmetry-old", age=9 * HOUR)
        session_like = self.entry(self.sessions, "not-a-session", age=9 * HOUR)
        self.entry(self.sessions, "bbbbbbbb-0000-4000-8000-000000000000", age=2 * HOUR)
        self.run_command()
        self.assertTrue(self.temp.is_dir())
        self.assertTrue((self.temp / "locmetry-link").is_symlink())
        self.assertTrue((outside / "log.txt").exists())
        self.assertTrue(foreign.exists())
        self.assertTrue(session_like.exists())

    def test_keeps_entries_changed_within_the_last_hour(self):
        # Another session may still be writing them.
        self.entry(self.temp, "locmetry-a", age=10)
        live = self.entry(self.temp, "locmetry-b", age=20 * 60)
        self.run_command()
        self.assertTrue(live.exists())

    def test_an_old_tree_with_a_recent_file_counts_as_recent(self):
        self.entry(self.temp, "locmetry-a", age=10)
        tree = self.entry(self.temp, "locmetry-b", age=5 * HOUR, files=["old.txt", "nested/new.txt"])
        os.utime(tree / "nested/new.txt", None)
        self.run_command()
        self.assertTrue(tree.exists())

    def test_never_removes_subject_homes_or_unarchived_campaigns(self):
        # Subject homes hold credentials the campaign runner writes back when
        # it retires them; a campaign holds evidence until it is archived.
        self.entry(self.temp, "locmetry-newest", age=HOUR + 60)
        homes = self.entry(self.temp, "locmetry-subject-homes", age=5 * HOUR, files=["h1/.codex/auth.json"])
        campaign = self.entry(self.temp, "locmetry-sections-v3", age=5 * HOUR, files=["campaign.json"])
        pair = self.entry(self.temp, "locmetry-smoke", age=5 * HOUR, files=["task1/evidence/run-01/calls.jsonl"])
        output = self.run_command()
        self.assertTrue(homes.exists())
        self.assertTrue(campaign.exists())
        self.assertTrue(pair.exists())
        self.assertIn(f"unarchived campaign, archive it first: {campaign.resolve()}", output)

    def test_keeps_only_the_newest_session_folder(self):
        newest = self.entry(self.sessions, "bbbbbbbb-0000-4000-8000-000000000000", age=2 * HOUR)
        older = self.entry(self.sessions, "aaaaaaaa-0000-4000-8000-000000000000", age=30 * HOUR)
        self.run_command()
        self.assertTrue(newest.exists())
        self.assertFalse(older.exists())

    def test_ignores_missing_locations(self):
        self.tmp.cleanup()
        self.assertIn("removed 0", self.run_command())


if __name__ == "__main__":
    unittest.main()
