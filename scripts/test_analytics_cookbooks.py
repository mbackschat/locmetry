"""Execute the cookbooks' marked examples against synthetic Locmetry measurements.

The cookbooks record their output on Immich, whose two healthy releases cannot
show empty, partial or file-less captures, sibling folders sharing a prefix, or
malformed inputs. These checks run the same query and code text on small
reports whose counts are known independently.
"""

import contextlib
import copy
import io
import json
import os
from pathlib import Path
import re
import subprocess
import tempfile
import sys
import unittest

import duckdb

REPOSITORY = Path(__file__).resolve().parents[1]
BINARY = Path(os.environ.get("LOCMETRY_BIN", REPOSITORY / "target/debug/locmetry")).resolve()
DUCKDB_COOKBOOK = REPOSITORY / "docs/duckdb-cookbook.md"
DATAFRAME_COOKBOOK = REPOSITORY / "docs/dataframe-cookbook.md"
LOADER = REPOSITORY / "examples/analytics/duckdb.sql"

# Two repositories with two captures each; the second API capture omits file records.
CAPTURE = """
set -eu
mkdir -p api/src api/tests web/src web/tests
cat > api/.locmetryrc <<'TOML'
version = 1
[[rules]]
id = "maintained"
match = ["src/**"]
purpose = "production"
category = "application"
TOML
cp api/.locmetryrc web/.locmetryrc
printf 'print(1)\\nprint(2)\\n' > api/src/main.py
printf 'assert True\\n' > api/tests/test_main.py
printf 'assert True\\n' > web/tests/test_main.py
locmetry api --no-user-config --files --output api-1.json
locmetry web --no-user-config --files --output web-1.json
printf 'print(3)\\n' >> api/src/main.py
printf 'assert True\\n' >> api/tests/test_main.py
printf 'print(1)\\nprint(2)\\n' > web/src/main.py
printf 'assert True\\n' >> web/tests/test_main.py
locmetry api --no-user-config --output api-2.json
locmetry web --no-user-config --files --output web-2.json
cat > snapshots.csv <<'CSV'
snapshot_id,repo_id,sequence,path
api-1,api,1,api-1.json
api-2,api,2,api-2.json
web-1,web,1,web-1.json
web-2,web,2,web-2.json
CSV
cat > owners.csv <<'CSV'
repo_id,prefix,owner
api,src,api-team
api,tests,quality
web,src,web-team
web,tests,quality
CSV
"""


def marked(name, source, delimiter):
    """The heredoc delimited by `delimiter` in the first Bash block after `<!-- name -->`."""
    parts = source.read_text().split(f"<!-- {name} -->")
    if len(parts) != 2:
        raise ValueError(f"{source.name} must mark {name} exactly once, not {len(parts) - 1} times")
    bash = parts[1].split("```bash\n", 1)[1].split("\n```", 1)[0]
    body = re.search(rf"<<'{delimiter}'\n(.*?)\n{delimiter}$", bash, re.DOTALL | re.MULTILINE).group(1)
    # Dot commands such as `.thousand_sep` configure the DuckDB program, not SQL.
    return "\n".join(line for line in body.splitlines() if not re.match(r"\.[a-z_]+\b", line))


def sql(name):
    return marked(name, DUCKDB_COOKBOOK, "SQL")


def python(name):
    return marked(name, DATAFRAME_COOKBOOK, "PYTHON")


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
        subprocess.run(["bash", "-euc", CAPTURE], env=self.env, check=True, capture_output=True, text=True)
        sys.path.insert(0, str(REPOSITORY / "examples/analytics"))
        self.addCleanup(sys.path.pop, 0)
        self.db = duckdb.connect()
        self.addCleanup(self.db.close)

    def load(self):
        self.db.execute(LOADER.read_text())

    def query(self, name):
        return self.db.execute(sql(name)).fetchall()

    def test_marked_queries_match_independent_counts(self):
        self.load()
        # Latest captures: API 3 production and 2 test lines, Web 2 and 2.
        self.assertEqual(self.query("composition"), [("Python", "production", 5, 2), ("Python", "test", 4, 2)])
        self.assertEqual(self.query("categories"), [("production", "application", 5, 55.6),
                                                     ("test", "test", 4, 44.4)])
        self.assertEqual(self.query("components"), [("api", "src", 3, None, None), ("api", "tests", None, 2, None),
                                                     ("web", "src", 2, None, None), ("web", "tests", None, 2, None)])
        self.assertEqual(self.query("trend"), [("api", "api-1", 3, None), ("api", "api-2", 5, 2),
                                               ("web", "web-1", 1, None), ("web", "web-2", 4, 3)])
        # Web's source folder is new, so its whole size is the change.
        self.assertEqual(self.query("changes"), [("web", "src", "production", 2), ("api", "src", "production", 1),
                                                 ("api", "tests", "test", 1), ("web", "tests", "test", 1)])
        self.assertEqual(self.query("ownership"), [("quality", None, 4), ("api-team", 3, None),
                                                   ("web-team", 2, None)])
        self.assertEqual(self.query("file-sizes"), [("api-1", 2, 1.5, 2), ("api-2", None, None, None),
                                                   ("web-1", 1, 1.0, 1), ("web-2", 2, 2.0, 2)])
        # API's latest capture has no file records, so neither new nor changed files can be named for it.
        self.assertEqual(self.query("new-files"), [("src/main.py", "production", 2)])
        self.assertEqual(self.query("changed-files"), [("tests/test_main.py", 1, 2, 1)])
        self.assertEqual(self.query("parquet"), [("api-1", 3), ("api-2", 5), ("web-1", 1), ("web-2", 4)])

    def test_partial_captures_end_trends_and_changes_instead_of_bridging_them(self):
        report = json.loads((self.root / "api-2.json").read_text())
        report["metadata"]["backend_inaccurate"] = True
        (self.root / "api-2.json").write_text(json.dumps(report))
        self.load()
        self.assertEqual(self.query("trend")[1], ("api", "api-2", 5, None))
        self.assertEqual(self.query("changes"), [("web", "src", "production", 2), ("web", "tests", "test", 1)])

    def run_python(self, name):
        namespace = {}
        with contextlib.redirect_stdout(io.StringIO()):
            exec(python(name), namespace)
        return namespace

    def test_dataframe_examples_match_sql(self):
        self.load()
        self.run_python("pandas-load")
        components = self.run_python("pandas-components")["components"]
        self.assertEqual(
            sorted((repo, component, row.get("production", 0), row.get("test", 0))
                   for (repo, component), row in components.to_dict("index").items()),
            sorted((repo, component, production or 0, test or 0)
                   for repo, component, production, test, _ in self.query("components")))
        expected = self.query("changes")
        pandas_changes = self.run_python("pandas-changes")["changes"]
        polars_changes = self.run_python("polars-changes")["changes"]
        columns = ["repo_id", "component", "purpose", "change"]
        self.assertEqual([tuple(row) for row in pandas_changes[columns].itertuples(index=False)], expected)
        self.assertEqual(polars_changes.select(columns).rows(), expected)
        self.run_python("pandas-comments")

    def test_loaders_keep_an_empty_capture(self):
        (self.root / "empty").mkdir()
        self.replace_manifest(self.measure("empty", "--files"))
        self.load()
        self.assertEqual(self.db.sql("SELECT source_code, partial, has_files FROM snapshots").fetchall(),
                         [(0, False, True)])
        self.assertEqual(self.db.sql("SELECT count(*) FROM contributions").fetchone(), (0,))
        self.assertEqual(self.query("file-sizes"), [("one", 0, None, None)])
        for name in ["pandas-load", "pandas-changes", "polars-changes"]:
            namespace = self.run_python(name)
            self.assertEqual(len(namespace["snapshots"]), 1)
            if "changes" in namespace:
                self.assertEqual(len(namespace["changes"]), 0)

    def replace_manifest(self, report):
        self.write_manifest(json.dumps(report))

    def write_manifest(self, content):
        (self.root / "single.json").write_text(content)
        (self.root / "snapshots.csv").write_text("snapshot_id,repo_id,sequence,path\none,api,1,single.json\n")

    def measure(self, directory, *args, status=0):
        output = subprocess.run([str(BINARY), str(directory), "--output", "-", *args],
                                env=self.env, capture_output=True, text=True)
        self.assertEqual(output.returncode, status, output.stderr)
        return json.loads(output.stdout)

    def test_partial_capture_stays_marked(self):
        (self.root / "empty").mkdir()
        (self.root / "empty/broken.ipynb").write_text("not a notebook")
        self.replace_manifest(self.measure("empty", "--files", status=2))
        self.load()
        self.assertEqual(self.db.sql("SELECT source_code, partial FROM snapshots").fetchall(), [(0, True)])

    def test_loaders_reject_wrong_envelopes_and_duplicate_manifest_identity(self):
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
                self.write_manifest(content)
                with self.assertRaises(duckdb.Error):
                    self.load()

    def test_embedded_languages_and_folder_boundaries_preserve_physical_totals(self):
        (self.root / "api/web").mkdir()
        (self.root / "api/web/page.html").write_text("<p>Hello</p>\n<script>\nconst x = 1;\n</script>\n")
        (self.root / "api/src-old").mkdir()
        (self.root / "api/src-old/old.py").write_text("print(1)\n")
        self.replace_manifest(self.measure("api", "--files"))
        self.load()
        self.assertEqual(self.db.sql("SELECT code, files FROM contributions WHERE language = 'JavaScript'").fetchall(),
                         [(1, 0)])
        self.assertEqual(self.db.sql("SELECT sum(code), sum(files) FROM contributions WHERE kind = 'source'").fetchone(),
                         (10, 4))
        self.query("ownership")
        self.assertEqual(self.db.sql("SELECT owner FROM owned_contributions WHERE folder = 'src-old'").fetchall(),
                         [("unassigned",)])
        self.assertEqual(self.query("file-sizes"), [("one", 4, 2.5, 4)])
        self.run_python("pandas-load")


if __name__ == "__main__":
    unittest.main()
