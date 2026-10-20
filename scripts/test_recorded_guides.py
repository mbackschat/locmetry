"""Regression checks for recorded-document readability and option coverage."""

import importlib.util
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


if __name__ == "__main__":
    unittest.main()
