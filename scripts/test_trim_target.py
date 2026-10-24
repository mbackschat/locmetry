"""Regression checks for the target trimming command.

A stub `cargo` on PATH replays recorded build messages, so the checks run
without a Rust toolchain and without compiling.
"""

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time
import unittest

SCRIPT = Path(__file__).with_name("trim-target.py")
CURRENT = "1111111111111111"
EARLIER = "2222222222222222"
RELEASE = "3333333333333333"
STUB = """#!/usr/bin/env python3
import os, pathlib, shutil, sys
stub = pathlib.Path(os.environ["STUB_DIR"])
with (stub / "calls.txt").open("a") as calls:
    calls.write(" ".join(sys.argv[1:]) + "\\n")
command = sys.argv[1] + ("-release" if "--release" in sys.argv else "")
if command == "clean":
    shutil.rmtree(os.environ["CARGO_TARGET_DIR"])
    sys.exit(0)
if (stub / ("fail-" + command)).exists():
    sys.exit(101)
messages = stub / (command + ".jsonl")
if messages.exists():
    sys.stdout.write(messages.read_text())
"""


def artifact(target, name, hash, kind="lib", local=False, files=None):
    package = "path+file:///work/locmetry#0.1.0" if local else f"registry+https://github.com/rust-lang/crates.io-index#{name}@1.0.0"
    files = files or [f"{target}/debug/deps/lib{name}-{hash}.rlib"]
    return dict(reason="compiler-artifact", package_id=package, target=dict(name=name, kind=[kind]), filenames=files, fresh=True)


class TrimTargetTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        root = Path(self.tmp.name)
        self.target = root / "target"
        self.stub = root / "stub"
        self.stub.mkdir()
        cargo = self.stub / "cargo"
        cargo.write_text(STUB)
        cargo.chmod(0o755)
        self.debug = self.target / "debug"
        self.debug.mkdir(parents=True)
        (self.debug / ".cargo-lock").touch()
        (self.target / "CACHEDIR.TAG").write_text("Signature: 8a477f597d28d172789f06886806bc55\n")

    def touch(self, relative, age=0, size=0):
        path = self.target / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b"x" * size)
        stamp = time.time() - age
        os.utime(path, (stamp, stamp))
        return path

    def session(self, crate_dir, name, age):
        path = self.debug / "incremental" / crate_dir / name
        path.mkdir(parents=True)
        (path / "dep-graph.bin").touch()
        (path.parent / f"{name}.lock").touch()
        stamp = time.time() - age
        os.utime(path, (stamp, stamp))
        return path

    def record(self, command, messages):
        (self.stub / f"{command}.jsonl").write_text("".join(json.dumps(m) + "\n" for m in messages))

    def run_command(self, *extra, success=True, **variables):
        env = dict(os.environ, PATH=f"{self.stub}{os.pathsep}{os.environ['PATH']}", STUB_DIR=str(self.stub))
        env.pop("CARGO_TARGET_DIR", None)
        env.update(variables)
        result = subprocess.run([sys.executable, str(SCRIPT), "--target-dir", str(self.target), *extra],
                                capture_output=True, text=True, env=env)
        self.assertEqual(result.returncode, 0 if success else 1, result.stdout + result.stderr)
        return result.stdout + result.stderr

    def calls(self):
        path = self.stub / "calls.txt"
        return path.read_text().splitlines() if path.exists() else []

    def test_removes_the_artifacts_of_earlier_builds_and_keeps_the_current_ones(self):
        target = self.target
        self.record("build", [
            artifact(target, "serde", CURRENT),
            dict(reason="build-script-executed", package_id="x", out_dir=f"{target}/debug/build/serde-{CURRENT}/out"),
        ])
        kept = [
            self.touch(f"debug/deps/libserde-{CURRENT}.rlib"),
            self.touch(f"debug/deps/serde-{CURRENT}.d"),
            self.touch(f"debug/.fingerprint/serde-{CURRENT}/lib-serde"),
            self.touch(f"debug/build/serde-{CURRENT}/out/generated.rs"),
            self.touch("debug/locmetry"),
            self.touch("debug/deps/unhashed.txt"),
        ]
        removed = [
            self.touch(f"debug/deps/libserde-{EARLIER}.rlib"),
            self.touch(f"debug/deps/locmetry-{EARLIER}.0abc.rcgu.o"),
            self.touch(f"debug/.fingerprint/serde-{EARLIER}/lib-serde"),
            self.touch(f"debug/build/serde-{EARLIER}/out/generated.rs"),
        ]
        output = self.run_command()
        for path in kept:
            self.assertTrue(path.exists(), path)
        for path in removed:
            self.assertFalse(path.exists(), path)
        self.assertFalse((self.debug / f".fingerprint/serde-{EARLIER}").exists())
        self.assertIn("removed 4 entries of earlier builds", output)

    def test_keeps_the_binary_cargo_reports_only_by_its_uplifted_hard_link(self):
        # Cargo reports a binary of the build profile as target/debug/NAME, a
        # hard link of deps/NAME-HASH (cargo 1.9x, 2026-10-24).
        unit = self.touch(f"debug/deps/locmetry-{CURRENT}", size=1)
        objects = self.touch(f"debug/deps/locmetry-{CURRENT}.locmetry.0abc-cgu.0.rcgu.o")
        os.link(unit, self.debug / "locmetry")
        earlier = self.touch(f"debug/deps/locmetry-{EARLIER}")
        self.record("build", [artifact(self.target, "locmetry", "", kind="bin", local=True, files=[f"{self.debug}/locmetry"])])
        session = self.session("locmetry-aaa", "s-1", age=10)
        self.run_command()
        self.assertTrue(unit.exists())
        self.assertTrue(objects.exists())
        self.assertFalse(earlier.exists())
        self.assertTrue(session.exists())

    def test_keeps_the_binary_whose_content_the_uplifted_copy_has(self):
        # On APFS, cargo copies the binary instead of linking it (2026-10-24).
        current = self.touch(f"debug/deps/locmetry-{CURRENT}", size=1)
        earlier = self.touch(f"debug/deps/locmetry-{EARLIER}", size=2)
        self.touch("debug/locmetry", size=1)
        self.record("build", [artifact(self.target, "locmetry", "", kind="bin", local=True, files=[f"{self.debug}/locmetry"])])
        self.run_command()
        self.assertTrue(current.exists())
        self.assertFalse(earlier.exists())

    def test_keeps_every_same_named_binary_when_none_matches_the_uplifted_one(self):
        current = self.touch(f"debug/deps/locmetry-{CURRENT}", size=1)
        other = self.touch(f"debug/deps/locmetry-{EARLIER}", size=2)
        self.touch("debug/locmetry", size=3)
        self.record("build", [artifact(self.target, "locmetry", "", kind="bin", local=True, files=[f"{self.debug}/locmetry"])])
        self.run_command()
        self.assertTrue(current.exists())
        self.assertTrue(other.exists())

    def test_keeps_one_incremental_directory_per_current_local_unit_with_its_newest_session(self):
        # Incremental directory names derive from rustc's stable crate id,
        # which no build message reports; the current build's local units say
        # how many of a crate's directories are current, and current ones were
        # used after any earlier one.
        target = self.target
        self.record("build", [
            artifact(target, "locmetry", CURRENT, local=True),
            artifact(target, "locmetry", "4444444444444444", kind="bin", local=True,
                     files=[f"{target}/debug/deps/locmetry-4444444444444444"]),
        ])
        newest = self.session("locmetry-aaa", "s-new-1", age=10)
        stale_session = self.session("locmetry-aaa", "s-old-1", age=500)
        second = self.session("locmetry-bbb", "s-new-2", age=20)
        earlier_build = self.session("locmetry-ccc", "s-old-3", age=900)
        self.run_command()
        self.assertTrue(newest.exists())
        self.assertTrue(second.exists())
        self.assertFalse(stale_session.exists())
        self.assertFalse(stale_session.with_name("s-old-1.lock").exists())
        self.assertFalse(earlier_build.parent.exists())

    def test_removes_incremental_directories_of_crates_the_build_no_longer_compiles(self):
        self.record("build", [artifact(self.target, "serde", CURRENT)])
        session = self.session("renamed-aaa", "s-1", age=10)
        self.run_command()
        self.assertFalse(session.parent.exists())

    def test_runs_the_debug_commands_and_the_release_build_only_when_a_release_tree_exists(self):
        self.run_command()
        self.assertEqual(self.calls(), [
            "build --locked --message-format=json",
            "test --locked --no-run --message-format=json",
            "clippy --locked --all-targets --message-format=json",
        ])
        (self.target / "release").mkdir()
        self.record("build-release", [artifact(self.target, "serde", RELEASE, files=[f"{self.target}/release/deps/libserde-{RELEASE}.rlib"])])
        current = self.touch(f"release/deps/libserde-{RELEASE}.rlib")
        earlier = self.touch(f"release/deps/libserde-{EARLIER}.rlib")
        self.run_command()
        self.assertIn("build --release --locked --message-format=json", self.calls())
        self.assertTrue(current.exists())
        self.assertFalse(earlier.exists())

    def test_leaves_a_profile_alone_when_its_build_fails(self):
        # A failed build lists only what it reached, so pruning by it would
        # delete current artifacts.
        self.record("build", [artifact(self.target, "serde", CURRENT)])
        (self.stub / "fail-clippy").touch()
        earlier = self.touch(f"debug/deps/libserde-{EARLIER}.rlib")
        output = self.run_command()
        self.assertTrue(earlier.exists())
        self.assertIn("left debug as it is", output)

    def test_cleans_the_target_when_it_stays_above_the_limit(self):
        self.record("build", [artifact(self.target, "serde", CURRENT)])
        self.touch(f"debug/deps/libserde-{CURRENT}.rlib", size=4096)
        output = self.run_command("--limit", "0")
        self.assertFalse(self.target.exists())
        self.assertIn("cargo clean", output)
        self.assertEqual(self.calls()[-1], "clean")

    def test_refuses_a_directory_that_is_not_a_cargo_target(self):
        (self.target / "CACHEDIR.TAG").unlink()
        earlier = self.touch(f"debug/deps/libserde-{EARLIER}.rlib")
        output = self.run_command(success=False)
        self.assertTrue(earlier.exists())
        self.assertEqual(self.calls(), [])
        self.assertIn("not a Cargo target directory", output)

    def test_refuses_a_shared_target_directory(self):
        # A target directory shared through CARGO_TARGET_DIR holds other
        # projects' builds, which this repository's build does not list.
        output = self.run_command("--limit", "0", success=False, CARGO_TARGET_DIR=str(self.target))
        self.assertTrue(self.target.exists())
        self.assertEqual(self.calls(), [])
        self.assertIn("CARGO_TARGET_DIR", output)

    def test_does_nothing_without_a_target_directory(self):
        self.tmp.cleanup()
        self.stub.mkdir(parents=True)
        (self.stub / "cargo").write_text(STUB)
        (self.stub / "cargo").chmod(0o755)
        output = self.run_command()
        self.assertEqual(self.calls(), [])
        self.assertIn("no target directory", output)


if __name__ == "__main__":
    unittest.main()
