"""Regression checks for the campaign compaction command."""

import gzip
import hashlib
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("compact-campaign.py")


class CompactCampaignTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.archive = Path(self.tmp.name) / "archive"
        self.campaign = self.archive / "2026-10-24-study-v1"
        self.write("campaign.json", "{}")

    def write(self, name, content):
        path = self.campaign / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(content.encode() if isinstance(content, str) else content)
        return path

    def run_command(self, campaign=None, success=True):
        result = subprocess.run([sys.executable, str(SCRIPT), "--archive-root", str(self.archive), str(campaign or self.campaign)],
                                capture_output=True, text=True)
        self.assertEqual(result.returncode, 0 if success else 1, result.stdout + result.stderr)
        return result.stdout + result.stderr

    def test_records_and_removes_the_binaries_and_arm_build_trees(self):
        self.write("binaries/A", "arm a")
        self.write("task1-rep1/B/target/debug/locmetry", "arm b")
        seed = self.write("task1-rep1/run-01/project/target/kept.txt", "a seed's own target folder is evidence")
        self.run_command()
        self.assertFalse((self.campaign / "binaries").exists())
        self.assertFalse((self.campaign / "task1-rep1/B/target").exists())
        self.assertTrue(seed.exists())
        manifest = (self.campaign / "binaries.sha256").read_text().splitlines()
        self.assertTrue(manifest[0].startswith("# Binaries removed on "))
        self.assertEqual(manifest[1:], [
            f"{hashlib.sha256(b'arm a').hexdigest()}  binaries/A",
            f"{hashlib.sha256(b'arm b').hexdigest()}  task1-rep1/B/target/debug/locmetry",
        ])

    def test_compresses_logs_without_changing_them(self):
        log = self.write("task1-rep1/evidence/run-01/session.jsonl", '{"a": 1}\n' * 100)
        self.run_command()
        self.assertFalse(log.exists())
        self.assertEqual(gzip.decompress(log.with_name("session.jsonl.gz").read_bytes()), b'{"a": 1}\n' * 100)

    def test_a_second_run_changes_nothing(self):
        self.write("binaries/A", "arm a")
        self.run_command()
        manifest = (self.campaign / "binaries.sha256").read_text()
        output = self.run_command()
        self.assertEqual((self.campaign / "binaries.sha256").read_text(), manifest)
        self.assertIn("nothing to compact", output)

    def test_adds_to_an_existing_manifest(self):
        self.write("binaries.sha256", "# Binaries removed on 2026-10-20.\nabc  binaries/old\n")
        self.write("binaries/B", "arm b")
        self.run_command()
        self.assertEqual((self.campaign / "binaries.sha256").read_text().splitlines(), [
            "# Binaries removed on 2026-10-20.", "abc  binaries/old",
            f"{hashlib.sha256(b'arm b').hexdigest()}  binaries/B",
        ])

    def test_refuses_a_folder_outside_the_archive(self):
        outside = Path(self.tmp.name) / "elsewhere"
        (outside / "binaries").mkdir(parents=True)
        (outside / "campaign.json").write_text("{}")
        output = self.run_command(outside, success=False)
        self.assertTrue((outside / "binaries").exists())
        self.assertIn("not inside", output)

    def test_refuses_a_folder_without_a_campaign_record(self):
        (self.campaign / "campaign.json").unlink()
        self.write("binaries/A", "arm a")
        output = self.run_command(success=False)
        self.assertTrue((self.campaign / "binaries/A").exists())
        self.assertIn("campaign.json", output)


if __name__ == "__main__":
    unittest.main()
