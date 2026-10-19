"""Verify that the help-recipe treatment changes only root long help."""

import argparse
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import textwrap

SPEC = importlib.util.spec_from_file_location("benchmark", Path(__file__).with_name("agent-explorability.py"))
BENCH = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(BENCH)


def verify(control, treatment, addition=None):
    recipe = (BENCH.TASK.parent / "help-recipe.txt").read_text()
    addition = recipe if addition is None else addition
    with tempfile.TemporaryDirectory(prefix="locmetry-recipe-parity-") as tmp:
        root = Path(tmp)
        BENCH.seed(root)
        env = dict(os.environ, NO_COLOR="1", COLUMNS="100", LC_ALL="C", XDG_CONFIG_HOME=str(root / "personal"))

        def run(binary, args):
            result = subprocess.run([str(binary), *args], cwd=root, env=env, capture_output=True, text=True)
            assert result.returncode == 0, (args, result.stderr)
            return result.stdout

        for args in [["--help"], ["-h"], ["help"], ["--version"], *[[n, "--help"] for n in ["workspace", "compare", "explain"]]]:
            a, b = run(control, args), run(treatment, args)
            if args in [["--help"], ["help"]]:
                assert b.count(addition) == 1, "Treatment recipe missing or duplicated"
                assert b.replace(addition, "") == a, args
            else:
                assert b == a, args
        for flag, topics in [("--guide", ["configure", "selection", "reports", "compare"]),
                             ("--schema", ["config", "user-config", "workspace", "report", "comparison", "explanation"])]:
            for topic in topics:
                assert run(control, [flag, topic]) == run(treatment, [flag, topic]), (flag, topic)
        for fmt in ["json", "headline", "table", "markdown"]:
            args = ["--root", "projects/web", "--format", fmt] + (["--files"] if fmt == "json" else [])
            a, b = run(control, args), run(treatment, args)
            if fmt in ["json", "headline"]:
                a, b = json.loads(a), json.loads(b)
                assert BENCH.normalize(a) == BENCH.normalize(b), fmt
                oracle = json.loads(BENCH.TASK.read_text())["oracle"]
                for group in ["source", "all_counted"]:
                    assert all(a["summary"][group][k] == v for k, v in oracle[group].items())
            else:
                assert [l for l in a.splitlines() if not l.startswith("Measured at:")] == [l for l in b.splitlines() if not l.startswith("Measured at:")], fmt
        service = root / "service"
        (service / "scripts").mkdir(parents=True)
        (service / "tests").mkdir()
        configuration = []
        for line in recipe.splitlines()[1:]:
            if line and not line.startswith("    "):
                break
            configuration.append(line)
        (service / ".locmetryrc").write_text(textwrap.dedent("\n".join(configuration)) + "\n")
        (service / "scripts/release.py").write_text('print("release")\n')
        (service / "tests/release_test.py").write_text("assert True\n")
        for binary in [control, treatment]:
            stdout = run(binary, ["--root", "service", "--files", "--output", "report.json", "--print", "table", "--verbose"])
            files = {f["path"]: f for f in json.loads((root / "report.json").read_text())["files"]}
            assert "Code" in stdout
            assert files["scripts/release.py"]["purpose"] == "production"
            assert files["scripts/release.py"]["category"] == "tooling"
            assert files["tests/release_test.py"]["purpose"] == "test"
    print("PASS: exact help addition, guides/schemas/report parity, independent counts, executable recipe and retained tests")


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("control", type=Path)
    parser.add_argument("treatment", type=Path)
    parser.add_argument("--routing-only", action="store_true", help="Compare example-only with routed-example help")
    args = parser.parse_args()
    addition = (BENCH.TASK.parent / "help-routing.txt").read_text() if args.routing_only else None
    verify(args.control.resolve(), args.treatment.resolve(), addition)
