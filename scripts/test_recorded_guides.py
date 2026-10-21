"""Regression checks for recorded-document readability and option coverage."""

import contextlib
import importlib.util
import io
import os
import shutil
import subprocess
import tempfile
from unittest.mock import patch
from pathlib import Path
import unittest

spec = importlib.util.spec_from_file_location("recorded", Path(__file__).with_name("check-recorded-guides.py"))
recorded = importlib.util.module_from_spec(spec)
spec.loader.exec_module(recorded)
HEADER = "# Demo\n\n*2026-10-20T00:00:00Z by Showboat 0.6.1*\n<!-- showboat-id: demo -->\n"


class RecordedGuideTests(unittest.TestCase):
    def test_new_option_needs_an_executable_example_not_prose_or_output(self):
        text = HEADER + "Use --new.\n```bash\nlocmetry --root .\n```\n```output\n--new\n```\n"
        help_text = "  --root <ROOT>\n  --new <VALUE>\n"
        self.assertEqual(recorded.missing_options(help_text, text), {"--new"})

    def test_wrapped_commands_cover_options(self):
        text = HEADER + "```bash\nlocmetry --root . \\\n  --new true\n```\n"
        self.assertEqual(recorded.missing_options("  --new <VALUE>\n", text), set())
        recorded.validate_document(text)

    def test_long_commands_and_output_fail_but_prose_stays_one_paragraph(self):
        recorded.validate_document(HEADER + "A paragraph " * 100)
        for language in ["bash", "output"]:
            with self.subTest(language=language), self.assertRaisesRegex(ValueError, "120"):
                recorded.validate_document(HEADER + f"```{language}\n" + "x" * 121 + "\n```\n")

    def test_missing_showboat_header_fails_before_replay(self):
        with self.assertRaisesRegex(ValueError, "header"):
            recorded.validate_document("# Handwritten\n```bash\ntrue\n```\n")


class ReplayTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.document = self.root / "cli/guide.md"
        self.document.parent.mkdir()
        self.document.write_text(HEADER + "```bash\ntrue\n```\n")
        self.artifacts = self.root / "evidence"
        self.artifacts.mkdir()

    def verify_result(self, status):
        def run(command, **kwargs):
            Path(command[command.index("--output") + 1]).write_text("regenerated candidate")
            return subprocess.CompletedProcess(command, status, "output drift\n" if status else "", "")
        return run

    def test_drift_preserves_candidate_and_log_without_reexecuting(self):
        original = self.document.read_bytes()
        with patch.object(recorded.subprocess, "run", side_effect=self.verify_result(1)) as run:
            with contextlib.redirect_stdout(io.StringIO()) as console:
                self.assertFalse(recorded.replay(self.document, self.artifacts))
        self.assertEqual(run.call_count, 1)
        self.assertEqual(self.document.read_bytes(), original)
        self.assertEqual((self.artifacts / "cli-guide.candidate.md").read_text(), "regenerated candidate")
        self.assertIn("output drift", (self.artifacts / "cli-guide.log").read_text())
        self.assertRegex(console.getvalue(), r"FAIL.*guide.md.*[0-9.]+s")

    def test_success_reports_elapsed_time_and_discards_candidate(self):
        with patch.object(recorded.subprocess, "run", side_effect=self.verify_result(0)):
            with contextlib.redirect_stdout(io.StringIO()) as console:
                self.assertTrue(recorded.replay(self.document, self.artifacts))
        self.assertFalse((self.artifacts / "cli-guide.candidate.md").exists())
        self.assertRegex(console.getvalue(), r"PASS.*guide.md.*[0-9.]+s")

    def test_invalid_document_does_not_keep_a_stale_candidate(self):
        self.document.write_text("# Missing header\n")
        (self.artifacts / "cli-guide.candidate.md").write_text("old failure")
        with patch.object(recorded.subprocess, "run") as run, contextlib.redirect_stdout(io.StringIO()):
            self.assertFalse(recorded.replay(self.document, self.artifacts))
        run.assert_not_called()
        self.assertFalse((self.artifacts / "cli-guide.candidate.md").exists())
        self.assertIn("header", (self.artifacts / "cli-guide.log").read_text())

    def test_documents_sharing_a_name_keep_separate_evidence(self):
        showcase = self.root / "showcase/guide.md"
        showcase.parent.mkdir()
        showcase.write_text(self.document.read_text())
        with patch.object(recorded.subprocess, "run", side_effect=self.verify_result(1)), \
                contextlib.redirect_stdout(io.StringIO()):
            recorded.replay(self.document, self.artifacts)
        with patch.object(recorded.subprocess, "run", side_effect=self.verify_result(0)), \
                contextlib.redirect_stdout(io.StringIO()):
            recorded.replay(showcase, self.artifacts)
        self.assertTrue((self.artifacts / "cli-guide.candidate.md").exists())
        self.assertIn("FAIL", (self.artifacts / "cli-guide.log").read_text())
        self.assertIn("PASS", (self.artifacts / "showcase-guide.log").read_text())

    def test_timeout_retains_partial_diagnostic(self):
        failure = subprocess.TimeoutExpired("showboat", 120, output=b"partial output\n")
        with patch.object(recorded.subprocess, "run", side_effect=failure):
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertFalse(recorded.replay(self.document, self.artifacts))
        log = (self.artifacts / "cli-guide.log").read_text()
        self.assertIn("partial output", log)
        self.assertIn("120", log)


class GitGuardTests(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        previous = Path.cwd()
        os.chdir(self.root)
        self.addCleanup(os.chdir, previous)
        subprocess.run(["git", "init", "-q"], check=True)
        self.fixture = self.root / "fixture.txt"
        self.fixture.write_text("original\n")
        subprocess.run(["git", "add", "fixture.txt"], check=True)
        subprocess.run(["git", "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
                        "commit", "-qm", "fixture", "--no-verify"], check=True)

    def test_gate_accepts_clean_tree_and_preserves_existing_edits(self):
        artifacts = Path(tempfile.mkdtemp())
        self.addCleanup(shutil.rmtree, artifacts)
        for content in ["original\n", "existing edit\n"]:
            with self.subTest(content=content):
                self.fixture.write_text(content)
                with patch.object(recorded, "REPOSITORY", self.root), \
                        patch.object(recorded, "showcase_unavailable", return_value=None), \
                        patch.object(recorded, "replay", return_value=True), \
                        contextlib.redirect_stdout(io.StringIO()):
                    self.assertTrue(recorded.check(artifacts))
                self.assertEqual(self.fixture.read_text(), content)

    def test_detects_mutation_even_when_path_was_already_dirty(self):
        self.fixture.write_text("existing edit\n")
        before = recorded.git_state()
        self.fixture.write_text("accidental replacement\n")
        self.assertNotEqual(before, recorded.git_state())

    def test_detects_staged_change_and_new_untracked_file(self):
        before = recorded.git_state()
        self.fixture.write_text("staged edit\n")
        subprocess.run(["git", "add", "fixture.txt"], check=True)
        self.assertNotEqual(before, recorded.git_state())
        before = recorded.git_state()
        (self.root / "unexpected.txt").write_text("created by replay\n")
        self.assertNotEqual(before, recorded.git_state())

    def test_gate_reports_mutation_and_still_checks_remaining_documents(self):
        docs = self.root / "docs/cli"
        docs.mkdir(parents=True)
        for name in ["first", "second"]:
            (docs / f"{name}.md").write_text(HEADER)
        (self.root / "docs/showcase.md").write_text(HEADER)
        artifacts = self.root.parent / (self.root.name + "-artifacts")
        self.addCleanup(shutil.rmtree, artifacts, True)
        visited = []

        def replay(path, destination, command=None):
            visited.append(path.name)
            if path.stem == "first":
                self.fixture.write_text("accidental replacement\n")
                return False
            return True

        with patch.object(recorded, "REPOSITORY", self.root), \
                patch.object(recorded, "GUIDES", {"first": [], "second": []}), \
                patch.object(recorded, "SHOWCASE", ["docs/showcase.md"]), \
                patch.object(recorded, "showcase_unavailable", return_value=None), \
                patch.object(recorded, "replay", side_effect=replay), \
                contextlib.redirect_stdout(io.StringIO()) as console:
            self.assertFalse(recorded.check(artifacts))
        self.assertEqual(visited, ["first.md", "second.md", "showcase.md"])
        self.assertIn("Git", console.getvalue())
        self.assertIn("Total", console.getvalue())
        self.assertTrue((artifacts / "git-after.txt").exists())


class DocumentSetTests(unittest.TestCase):
    """Reference guides replay from the checked-in fixture; showcase documents
    need the pinned Immich checkouts, so CI replays them in a separate step."""

    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        previous = Path.cwd()
        self.addCleanup(os.chdir, previous)
        os.chdir(self.root)
        subprocess.run(["git", "init", "-q"], check=True)
        subprocess.run(["git", "-c", "user.name=Test", "-c", "user.email=test@example.invalid",
                        "commit", "-q", "--allow-empty", "-m", "empty", "--no-verify"], check=True)
        self.artifacts = self.root.parent / (self.root.name + "-artifacts")
        self.addCleanup(shutil.rmtree, self.artifacts, True)

    def run_check(self, selection, unavailable=None):
        visited = []

        def replay(path, destination, command=None):
            visited.append(str(path))
            return True

        with patch.object(recorded, "REPOSITORY", self.root), \
                patch.object(recorded, "GUIDES", {"root": []}), \
                patch.object(recorded, "SHOWCASE", ["docs/showcase/measure.md"]), \
                patch.object(recorded, "showcase_unavailable", return_value=unavailable), \
                patch.object(recorded, "replay", side_effect=replay), \
                contextlib.redirect_stdout(io.StringIO()) as console:
            passed = recorded.check(self.artifacts, selection)
        return passed, visited, console.getvalue()

    def test_sets_select_reference_and_showcase_documents(self):
        for selection, expected in [
            ("reference", ["docs/cli/root.md"]),
            ("showcase", ["docs/showcase/measure.md"]),
            ("all", ["docs/cli/root.md", "docs/showcase/measure.md"]),
        ]:
            with self.subTest(selection=selection):
                passed, visited, _ = self.run_check(selection)
                self.assertTrue(passed)
                self.assertEqual(visited, expected)

    def test_missing_checkouts_fail_showcase_documents_without_replaying_them(self):
        diagnostic = "Missing the pinned showcase checkout; run bash scripts/fetch-showcase.sh"
        passed, visited, console = self.run_check("all", unavailable=diagnostic)
        self.assertFalse(passed)
        self.assertEqual(visited, ["docs/cli/root.md"])
        self.assertIn("FAIL docs/showcase/measure.md", console)
        self.assertIn("fetch-showcase.sh", console)
        self.assertIn("fetch-showcase.sh", (self.artifacts / "showcase-measure.log").read_text())


if __name__ == "__main__":
    unittest.main()
