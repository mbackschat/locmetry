"""Regression checks for the documentation index command."""

import json
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("doc-indexes.py")
START = "<!-- doc-index:start -->"
END = "<!-- doc-index:end -->"


class DocIndexesTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        self.write(".gitignore", "docs/research/local/\n")
        self.write("docs/README.md", "# Docs\n\n- [Plans](plans/README.md)\n")
        self.write("docs/plans/README.md", f"# Plans\n\nHandwritten introduction.\n\n{START}\n{END}\n")
        self.write("docs/plans/work.md", self.document())

    def write(self, name, content):
        path = self.root / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(content)

    def document(self, **changes):
        metadata = dict(status="planned", date="2026-10-16", summary="A bounded change.", evidence="[Decision](#decision)")
        metadata.update(changes)
        return '# Work\n\n<!-- doc-status\n' + json.dumps(metadata) + '\n-->\n\n## Decision\n\nOwner approval.\n'

    def run_command(self, mode, success=True):
        result = subprocess.run([sys.executable, str(SCRIPT), mode, "--root", str(self.root)], capture_output=True, text=True)
        self.assertEqual(result.returncode, 0 if success else 1, result.stdout + result.stderr)
        return result.stdout + result.stderr

    def test_update_is_deterministic_and_preserves_handwritten_content(self):
        self.run_command("update")
        path = self.root / "docs/plans/README.md"
        first = path.read_text()
        self.assertIn("Handwritten introduction.", first)
        self.assertIn("| [Work](work.md) | planned | 2026-10-16 | A bounded change. |", first)
        self.run_command("update")
        self.assertEqual(first, path.read_text())
        self.run_command("check")

    def test_check_rejects_stale_status_without_writing(self):
        self.run_command("update")
        path = self.root / "docs/plans/README.md"
        first = path.read_text()
        self.write("docs/plans/work.md", self.document(status="active"))
        self.assertIn("stale", self.run_command("check", False))
        self.assertEqual(first, path.read_text())

    def test_rejects_missing_indexes_entries_and_duplicate_entries(self):
        self.run_command("update")
        for content, error in [("# Docs\n", "missing entry"), ("# Docs\n- [A](plans/README.md)\n- [B](plans/README.md)\n", "duplicate entry")]:
            with self.subTest(error=error):
                self.write("docs/README.md", content)
                self.assertIn(error, self.run_command("check", False))
        self.write("docs/new/topic.md", "# Topic\n")
        self.assertIn("missing index", self.run_command("check", False))

    def test_rejects_invalid_status_date_and_missing_evidence(self):
        for changes, error in [({"status": "accepted"}, "invalid status"), ({"date": "2026-02-30"}, "invalid date"), ({"date": "2026-1-2"}, "invalid date"), ({"status": "completed", "evidence": ""}, "evidence"), ({"summary": "two\nlines"}, "single-line"), ({"evidence": "No linked decision"}, "evidence"), ({"surprise": "field"}, "metadata fields")]:
            with self.subTest(changes=changes):
                self.write("docs/plans/work.md", self.document(**changes))
                self.assertIn(error, self.run_command("update", False))

    def test_checks_evidence_fragments_and_relative_paths(self):
        self.write("docs/plans/work.md", self.document(status="completed", evidence="[Proof](#absent)"))
        self.assertIn("fragment", self.run_command("update", False))
        self.write("docs/plans/work.md", self.document(evidence="[Proof](missing.md)"))
        self.assertIn("broken link", self.run_command("update", False))

    def test_new_documents_are_discovered_but_private_ignored_files_are_not(self):
        self.write("docs/research/local/secret.md", "# Secret\n[Bad](missing.md)\n")
        self.write("docs/plans/second.md", self.document(status="deferred"))
        self.run_command("update")
        self.assertIn("second.md", (self.root / "docs/plans/README.md").read_text())
        self.run_command("check")

    def test_checks_links_but_ignores_code_examples_and_external_urls(self):
        self.run_command("update")
        base = (self.root / "docs/README.md").read_text()
        self.write("docs/README.md", base + '\n```md\n[Example](absent.md)\n```\n`[Example](absent.md)`\n[Web](https://example.invalid/no)\n')
        self.run_command("check")
        self.write("docs/README.md", base + "\n[Broken](plans/work.md#missing)\n")
        self.assertIn("fragment", self.run_command("check", False))

    def test_duplicate_heading_fragments_and_inline_formatting(self):
        self.write("docs/plans/work.md", self.document() + '\n## `Code` & **style**\n\n## `Code` & **style**\n\n[Repeat](#code--style-1)\n')
        self.run_command("update")
        self.run_command("check")

    def test_invalid_metadata_does_not_partially_update_indexes(self):
        before = (self.root / "docs/plans/README.md").read_text()
        self.write("docs/plans/second.md", "# Missing metadata\n")
        self.run_command("update", False)
        self.assertEqual(before, (self.root / "docs/plans/README.md").read_text())

    def test_status_date_change_requires_regeneration(self):
        self.run_command("update")
        self.write("docs/plans/work.md", self.document(date="2026-10-17"))
        self.assertIn("stale", self.run_command("check", False))
        self.run_command("update")
        self.assertIn("| 2026-10-17 |", (self.root / "docs/plans/README.md").read_text())

    def test_generated_links_handle_spaces_and_parentheses_in_filenames(self):
        self.write("docs/plans/new plan (review).md", self.document())
        self.run_command("update")
        self.assertIn("new%20plan%20%28review%29.md", (self.root / "docs/plans/README.md").read_text())
        self.run_command("check")

    def test_metadata_must_be_immediately_after_the_title(self):
        self.write("docs/plans/work.md", self.document().replace("# Work\n\n", "# Work\n\nOther content.\n\n"))
        self.assertIn("after the title", self.run_command("check", False))

    def test_archive_placement_does_not_imply_completion(self):
        self.write("docs/plans/archive/README.md", f"# Archive\n\n{START}\n{END}\n")
        self.write("docs/plans/archive/snapshot.md", self.document(status="historical", evidence="[Record](#decision)"))
        with (self.root / "docs/plans/README.md").open("a") as file:
            file.write("\n[Archive](archive/README.md)\n")
        self.run_command("update")
        self.run_command("check")
        self.assertIn("| historical |", (self.root / "docs/plans/archive/README.md").read_text())


if __name__ == "__main__":
    unittest.main()
