"""Guard the generated agent regression trend table."""

import importlib.util
from pathlib import Path
import tempfile
import unittest

SPEC = importlib.util.spec_from_file_location("trend", Path(__file__).with_name("agent-regression-trend.py"))
TREND = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(TREND)


def results(date, control, candidate, tasks, model="model-m", effort="xhigh"):
    """Synthetic portable results: tasks maps a number to (calls A, calls B, uncached A, uncached B)."""
    per_task, runs = {}, []
    for number, (calls_a, calls_b, fresh_a, fresh_b) in tasks.items():
        per_task[str(number)] = {
            "A": {"tools": {"median": calls_a}, "fresh": {"median": fresh_a}, "passes": 4},
            "B": {"tools": {"median": calls_b}, "fresh": {"median": fresh_b}, "passes": 3},
        }
        runs += [{"task": number, "arm": arm} for arm in "AB" for _ in range(4)]
    return {"date": date, "model": model, "effort": effort,
            "arms": {"A": {"source_revision": control}, "B": {"source_revision": candidate}},
            "analysis": {"per_task": per_task}, "runs": runs}


class TrendTests(unittest.TestCase):
    def test_a_task_measured_in_consecutive_checks_chains_its_step_ratios(self):
        rows = TREND.trend_rows([
            results("2026-10-20", "aaaaaaa111", "bbbbbbb222", {4: (2, 3, 1000, 1100)}),
            results("2026-10-22", "bbbbbbb", "ccccccc333", {4: (3, 3, 1000, 800), 8: (2, 4, 500, 500)}),
        ])
        latest = {row["task"]: row for row in rows if row["candidate"] == "ccccccc"}
        self.assertEqual(latest[4]["base"], "aaaaaaa")
        self.assertAlmostEqual(latest[4]["calls_since"], 1.5)
        self.assertAlmostEqual(latest[4]["uncached_since"], 0.88)
        # Task 8 has no measured predecessor step, so its series starts at this control.
        self.assertEqual(latest[8]["base"], "bbbbbbb")
        self.assertAlmostEqual(latest[8]["calls_since"], 2.0)

    def test_a_different_model_or_effort_starts_a_new_series(self):
        rows = TREND.trend_rows([
            results("2026-10-20", "aaaaaaa", "bbbbbbb", {4: (2, 4, 1000, 2000)}),
            results("2026-10-22", "bbbbbbb", "ccccccc", {4: (2, 2, 1000, 1000)}, model="model-n"),
            results("2026-10-23", "ccccccc", "ddddddd", {4: (2, 2, 1000, 1000)}, effort="high"),
        ])
        self.assertEqual([row["base"] for row in rows], ["aaaaaaa", "bbbbbbb", "ccccccc"])
        self.assertAlmostEqual(rows[1]["calls_since"], 1.0)

    def test_table_shows_passes_medians_and_ratios_and_leaves_zero_controls_unrated(self):
        table = TREND.render(TREND.trend_rows([
            results("2026-10-22", "bbbbbbb", "ccccccc", {3: (0, 2, 20067.5, 21000)}),
        ]))
        self.assertIn("| 2026-10-22 | model-m `xhigh` | `bbbbbbb` → `ccccccc` | 3 | 4/4 → 3/4 |", table)
        self.assertIn("0 → 2 (—)", table)
        self.assertIn("20,068 → 21,000 (×1.05)", table)

    def test_half_medians_keep_their_decimal(self):
        table = TREND.render(TREND.trend_rows([results("2026-10-22", "b", "c", {3: (5.5, 6, 10, 10)})]))
        self.assertIn("5.5 → 6 (×1.09)", table)

    def test_a_same_day_predecessor_is_chained_whatever_the_input_order(self):
        rows = TREND.trend_rows([
            results("2026-10-22", "bbbbbbb", "ccccccc", {4: (2, 4, 10, 10)}),
            results("2026-10-22", "aaaaaaa", "bbbbbbb", {4: (2, 4, 10, 10)}),
        ])
        self.assertEqual([row["candidate"] for row in rows], ["bbbbbbb", "ccccccc"])
        self.assertAlmostEqual(rows[1]["calls_since"], 4.0)

    def test_only_paired_version_checks_enter_the_trend(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            TREND.write_json(root / "paired-results.json", results("2026-10-22", "b", "c", {3: (1, 1, 1, 1)}))
            TREND.write_json(root / "treatment-results.json", {"date": "2026-10-20", "analysis": {"per_task": {}}})
            self.assertEqual(len(TREND.load_results(root)), 1)

    def test_check_reports_a_stale_table_and_update_rewrites_only_the_managed_block(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            TREND.write_json(root / "x-results.json", results("2026-10-22", "b", "c", {3: (1, 2, 10, 10)}))
            document = root / "checks.md"
            document.write_text(f"# Checks\n\n{TREND.START}\nold\n{TREND.END}\n\nAfter.\n")
            self.assertFalse(TREND.sync(document, root, write=False))
            self.assertFalse(TREND.sync(document, root, write=True))
            self.assertTrue(TREND.sync(document, root, write=False))
            text = document.read_text()
            self.assertTrue(text.startswith("# Checks\n\n"))
            self.assertTrue(text.endswith("\n\nAfter.\n"))
            self.assertIn("1 → 2 (×2.00)", text)


if __name__ == "__main__":
    unittest.main()
