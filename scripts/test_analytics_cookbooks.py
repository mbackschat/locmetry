"""Execute the cookbook's sources against synthetic Locmetry measurements."""

import contextlib
import copy
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
import sys
import unittest

import duckdb

REPOSITORY = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("LOCMETRY_BIN", REPOSITORY / "target/debug/locmetry")).resolve()
COOKBOOK = REPOSITORY / "docs/duckdb-cookbook.md"
LOADER = REPOSITORY / "examples/analytics/duckdb.sql"


def block(name, language="sql", source=COOKBOOK):
    text = source.read_text().split(f"<!-- {name} -->", 1)[1]
    return text.split(f"```{language}\n", 1)[1].split("```", 1)[0]


class CookbookTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.root = Path(self.temporary.name)
        self.previous = Path.cwd()
        os.chdir(self.root)
        self.addCleanup(os.chdir, self.previous)
        (self.root / "bin").mkdir()
        (self.root / "bin/locmetry").symlink_to(BINARY)
        self.env = dict(os.environ, PATH=f"{self.root / 'bin'}:{os.environ['PATH']}",
                        XDG_CONFIG_HOME=str(self.root / "no-user-config"))
        subprocess.run(["bash", "-euc", block("capture", "sh")], env=self.env, check=True,
                       capture_output=True, text=True)
        sys.path.insert(0, str(REPOSITORY / "examples/analytics"))
        self.addCleanup(sys.path.pop, 0)
        self.db = duckdb.connect()
        self.addCleanup(self.db.close)

    def load(self):
        self.db.execute(LOADER.read_text())

    def query(self, name):
        return self.db.execute(block(name)).fetchall()

    def test_documented_queries_match_independent_counts(self):
        self.load()
        self.assertEqual(self.query("composition"), [("Python", "production", 2), ("Python", "test", 1)])
        self.assertEqual(self.query("folders"), [("src", 2), ("tests", 1)])
        self.assertEqual(self.query("portfolio"), [(5, 4, 0.8)])
        self.assertEqual(self.query("trend"), [("api", 1, 3, None), ("api", 2, 5, 2),
                                              ("web", 1, 1, None), ("web", 2, 4, 3)])
        self.db.execute(block("owners"))
        self.assertEqual(self.query("ownership"), [("api-2", "api-team", 3), ("api-2", "quality", 2)])
        self.assertEqual(self.query("file-sizes"), [("api-1", 2, 1.5, 2), ("api-2", None, None, None),
                                                   ("web-1", 1, 1.0, 1), ("web-2", 2, 2.0, 2)])
        self.db.execute(block("parquet"))
        self.assertEqual(self.db.sql("SELECT sum(code) FROM read_parquet('contributions.parquet') WHERE kind='source'").fetchone(), (13,))
        self.assertEqual(self.db.sql("SELECT count(*) FROM read_parquet('snapshots.parquet')").fetchone(), (4,))

    def dataframe_results(self):
        results = []
        for engine in ["pandas", "polars"]:
            namespace = {}
            with contextlib.redirect_stdout(io.StringIO()):
                exec(block(engine, "python", REPOSITORY / "docs/dataframe-cookbook.md"), namespace)
            frame = namespace["composition"]
            rows = frame.to_dict("records") if engine == "pandas" else frame.to_dicts()
            results.append(sorted(tuple(row[key] for key in ["snapshot_id", "language", "purpose", "code"]) for row in rows))
            self.assertEqual(len(namespace["snapshots"]), self.db.sql("SELECT count(*) FROM snapshots").fetchone()[0])
        expected = self.db.sql("SELECT snapshot_id,language,purpose,sum(code) FROM contributions WHERE kind='source' GROUP BY ALL ORDER BY ALL").fetchall()
        self.assertEqual(results, [expected, expected])

    def test_dataframe_recipes_match_sql_and_keep_empty_snapshots(self):
        self.load()
        self.dataframe_results()
        (self.root / "empty").mkdir()
        self.replace_manifest(self.measure("empty"))
        self.load()
        self.dataframe_results()

    def replace_manifest(self, report):
        (self.root / "single.json").write_text(json.dumps(report))
        (self.root / "snapshots.csv").write_text("snapshot_id,repo_id,sequence,path\none,api,1,single.json\n")

    def measure(self, directory, *args, status=0):
        output = subprocess.run([str(BINARY), str(directory), "--output", "-", *args],
                                env=self.env, capture_output=True, text=True)
        self.assertEqual(output.returncode, status, output.stderr)
        return json.loads(output.stdout)

    def test_empty_and_unavailable_snapshots_survive_empty_detail(self):
        (self.root / "empty").mkdir()
        empty = self.measure("empty", "--files")
        self.replace_manifest(empty)
        self.load()
        self.assertEqual(self.db.sql("SELECT source_code,partial,has_files FROM snapshots").fetchall(), [(0, False, True)])
        self.assertEqual(self.db.sql("SELECT count(*) FROM contributions").fetchone(), (0,))
        self.assertEqual(self.query("file-sizes"), [("one", 0, None, None)])
        (self.root / "empty/broken.ipynb").write_text("not a notebook")
        partial = self.measure("empty", "--files", status=2)
        self.replace_manifest(partial)
        self.load()
        self.assertEqual(self.db.sql("SELECT source_code,partial FROM snapshots").fetchall(), [(0, True)])

    def test_loader_rejects_wrong_envelopes_and_duplicate_manifest_identity(self):
        report = json.loads((self.root / "api-1.json").read_text())
        for field, value in [("detail", "headline"), ("schema_version", "2"),
                             ("document_type", "comparison"), ("document_type", None),
                             ("breakdown", None)]:
            invalid = copy.deepcopy(report)
            invalid[field] = value
            self.replace_manifest(invalid)
            with self.assertRaisesRegex(duckdb.Error, "full schema-3 measurement"):
                self.load()
            from report_rows import read_reports
            with self.assertRaises(ValueError):
                read_reports()
        self.replace_manifest(report)
        with (self.root / "snapshots.csv").open("a") as output:
            output.write("two,api,1,api-1.json\n")
        with self.assertRaises(duckdb.ConstraintException):
            self.load()

    def test_manifest_paths_each_contain_exactly_one_report(self):
        text = (self.root / "api-1.json").read_text()
        for content in ["", text + "\n" + text]:
            with self.subTest(documents=0 if not content else 2):
                (self.root / "single.json").write_text(content)
                (self.root / "snapshots.csv").write_text("snapshot_id,repo_id,sequence,path\none,api,1,single.json\n")
                with self.assertRaises(duckdb.Error):
                    self.load()

    def test_embedded_languages_and_folder_boundaries_preserve_physical_totals(self):
        (self.root / "api/web").mkdir()
        (self.root / "api/web/page.html").write_text("<p>Hello</p>\n<script>\nconst x = 1;\n</script>\n")
        (self.root / "api/src-old").mkdir()
        (self.root / "api/src-old/old.py").write_text("print(1)\n")
        self.replace_manifest(self.measure("api", "--files"))
        self.load()
        self.assertEqual(self.db.sql("SELECT code,files FROM contributions WHERE language='JavaScript'").fetchall(), [(1, 0)])
        self.assertEqual(self.db.sql("SELECT sum(code),sum(files) FROM contributions WHERE kind='source'").fetchone(), (10, 4))
        self.db.execute(block("owners"))
        self.assertEqual(self.db.sql("SELECT owner FROM owned_contributions WHERE folder='src-old'").fetchall(), [("unassigned",)])
        self.assertEqual(self.query("file-sizes"), [("one", 4, 2.5, 4)])
        self.dataframe_results()

    def test_trend_marks_partial_edges_instead_of_bridging_them(self):
        report = json.loads((self.root / "api-2.json").read_text())
        report["metadata"]["backend_inaccurate"] = True
        (self.root / "api-2.json").write_text(json.dumps(report))
        self.load()
        self.assertEqual(self.query("trend")[1], ("api", 2, 5, None))
        self.assertEqual(self.query("portfolio"), [(2, 2, 1.0)])


if __name__ == "__main__":
    unittest.main()
