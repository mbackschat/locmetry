"""Regression checks for pruning Cargo's target directory oldest-first."""

import fcntl
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

SCRIPT = Path(__file__).with_name("prune-target.py")
KIB = 1024


class PruneTargetTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.target = Path(self.tmp.name) / "target"

    def artifact(self, relative, kib, age_days, target=None):
        path = (target or self.target) / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(b"x" * kib * KIB)
        stamp = 1_700_000_000 - age_days * 86_400
        os.utime(path, (stamp, stamp))
        return path

    def run_script(self, *args, stdin=None):
        return subprocess.run(
            [sys.executable, str(SCRIPT), *args],
            input=stdin, capture_output=True, text=True, check=True,
            env={k: v for k, v in os.environ.items() if k != "CARGO_TARGET_DIR"},
        )

    def prune(self, *args):
        return self.run_script("--target", str(self.target), *args)

    def test_under_the_cap_nothing_is_removed(self):
        kept = self.artifact("debug/deps/libold-1.rlib", 4, age_days=9)
        self.prune("--max-kib", "8", "--floor-kib", "4")
        self.assertTrue(kept.exists())

    def test_over_the_cap_the_oldest_units_go_until_the_floor(self):
        oldest = self.artifact("debug/deps/libtokei-old.rlib", 4, age_days=9)
        older = self.artifact("debug/incremental/tokei-old/s-1/query.bin", 4, age_days=8)
        newest = self.artifact("debug/deps/libtokei-new.rlib", 4, age_days=1)
        self.prune("--max-kib", "8", "--floor-kib", "4")
        self.assertFalse(oldest.exists())
        # Incremental state goes as a whole crate directory, never file by file.
        self.assertFalse(older.parent.parent.exists())
        self.assertTrue(newest.exists())

    def test_a_binary_cargo_links_into_the_profile_counts_once(self):
        # Cargo hard-links each built binary from `deps/` into the profile.
        built = self.artifact("debug/deps/app-1", 8, age_days=9)
        os.link(built, self.target / "debug/app")
        self.prune("--max-kib", "12", "--floor-kib", "4")
        self.assertTrue(built.exists())

    def test_a_profile_locked_by_a_running_cargo_is_left_alone(self):
        old = self.artifact("debug/deps/libold.rlib", 8, age_days=9)
        self.artifact("debug/.cargo-lock", 0, age_days=0)
        with open(self.target / "debug/.cargo-lock", "w") as lock:
            fcntl.flock(lock, fcntl.LOCK_EX)
            self.prune("--max-kib", "4", "--floor-kib", "2")
        self.assertTrue(old.exists())

    def hook(self, command, cwd):
        event = {"tool_name": "Bash", "cwd": str(cwd), "tool_input": {"command": command}}
        return self.run_script("--hook", "--max-kib", "4", "--floor-kib", "2", stdin=json.dumps(event))

    def project(self, name):
        root = Path(self.tmp.name) / name
        (root / "target").mkdir(parents=True)
        (root / "Cargo.toml").write_text("[package]\n")
        return root

    def unit(self, root):
        return self.artifact("debug/deps/libold.rlib", 8, age_days=9, target=root / "target")

    def test_the_hook_ignores_commands_without_cargo(self):
        root = self.project("app")
        old = self.unit(root)
        self.hook("git status", root)
        self.assertTrue(old.exists())

    def test_the_hook_prunes_after_cargo_and_reports_it(self):
        root = self.project("app")
        old = self.unit(root)
        result = self.hook("cargo test --locked", root)
        self.assertFalse(old.exists())
        self.assertIn("pruned", json.loads(result.stdout)["systemMessage"])

    def test_the_hook_prunes_the_project_the_command_entered_not_the_session_one(self):
        session, sibling = self.project("app"), self.project("fork")
        kept, pruned = self.unit(session), self.unit(sibling)
        self.hook(f"cd {sibling} && cargo build --release 2>&1 | tail -1", session)
        self.assertTrue(kept.exists())
        self.assertFalse(pruned.exists())

    def test_the_hook_follows_relative_cd_manifest_paths_and_target_dirs(self):
        session, fork = self.project("app"), self.project("fork")
        tool = self.project("fork/judges/tool")
        tool_unit = self.unit(tool)
        speed_unit = self.unit(fork / "speed")
        self.hook(f"cd {fork}; cd judges/tool && cargo build", session)
        self.hook(f"cargo build --manifest-path {fork}/Cargo.toml --target-dir={fork}/speed/target", session)
        self.assertFalse(tool_unit.exists())
        self.assertFalse(speed_unit.exists())

    def test_the_hook_skips_a_manifest_path_it_cannot_resolve(self):
        root = self.project("app")
        old = self.unit(root)
        self.hook("cargo build --manifest-path $LOCMETRY_UNSET_DIR/Cargo.toml", root)
        self.assertTrue(old.exists())

    def cargo_unit(self, name, unit_hash, kind="lib", *, local=True, deps=(), profile=1, invoked=0,
                   failed=False):
        """A unit as Cargo records it: a fingerprint directory holding the
        fingerprint value, its JSON and the compile's start stamp, plus its
        outputs in `deps/` and `build/`. Returns its fingerprint value."""
        profile_dir = self.target / "debug"
        fingerprint = profile_dir / ".fingerprint" / f"{name}-{unit_hash}"
        fingerprint.mkdir(parents=True)
        value = int.from_bytes(hashlib.sha256(unit_hash.encode()).digest()[:8], "big")
        stem = f"{kind}-{name}"
        # Cargo writes the value's little-endian bytes in hex (`to_hex` in
        # cargo's `util/hex.rs`), which the `deps` entries give as integers.
        # A failed compile leaves the value file empty (checked with Cargo
        # 1.98 on 2026-10-30).
        (fingerprint / stem).write_text("" if failed else value.to_bytes(8, "little").hex())
        (fingerprint / f"{stem}.json").write_text(json.dumps({
            "rustc": 1, "features": "[]", "target": 1, "profile": profile, "path": 1,
            "deps": [[0, "dep", False, dep] for dep in deps],
            "local": [{"CheckDepInfo": {"dep_info": "x", "checksum": False}}] if local
            else [{"Precalculated": "1.0.0"}],
            "rustflags": [], "config": 0, "compile_kind": 0,
        }))
        (fingerprint / "invoked.timestamp").write_text("")
        start = 1_700_000_000 + invoked
        os.utime(fingerprint / "invoked.timestamp", (start, start))
        end = start if failed else start + 5
        os.utime(fingerprint / stem, (end, end))
        crate = name.replace("-", "_")
        for output in (f"lib{crate}-{unit_hash}.rlib", f"{crate}-{unit_hash}.d"):
            (profile_dir / "deps").mkdir(exist_ok=True)
            (profile_dir / "deps" / output).write_text("x")
        (profile_dir / "build" / f"{name}-{unit_hash}").mkdir(parents=True)
        return value

    def incremental(self, crate, session_id, at):
        path = self.target / "debug/incremental" / f"{crate}-{session_id}"
        path.mkdir(parents=True)
        os.utime(path, (1_700_000_000 + at, 1_700_000_000 + at))
        return path

    def outputs(self, name, unit_hash):
        profile_dir = self.target / "debug"
        return [*(profile_dir / "deps").glob(f"*-{unit_hash}*"), profile_dir / ".fingerprint" / f"{name}-{unit_hash}",
                profile_dir / "build" / f"{name}-{unit_hash}"]

    def test_a_superseded_unit_and_the_units_only_it_reaches_go_whatever_the_size(self):
        shared = self.cargo_unit("regex", "5" * 16, local=False)
        old_dep = self.cargo_unit("tokei", "1" * 16, local=False)
        new_dep = self.cargo_unit("tokei", "2" * 16, local=False, invoked=100)
        self.cargo_unit("app", "a" * 16, deps=[shared, old_dep], invoked=10)
        self.cargo_unit("app", "b" * 16, deps=[shared, new_dep], invoked=110)
        self.prune()
        for gone in [*self.outputs("app", "a" * 16), *self.outputs("tokei", "1" * 16)]:
            self.assertFalse(gone.exists(), gone)
        for kept in [*self.outputs("app", "b" * 16), *self.outputs("tokei", "2" * 16),
                     *self.outputs("regex", "5" * 16)]:
            self.assertTrue(kept.exists(), kept)

    def test_units_of_another_kind_or_profile_are_not_superseded(self):
        # Clippy builds `lib-app` again under its own profile hash, and
        # `cargo test` builds `test-lib-app`; each stays current beside the build.
        self.cargo_unit("app", "a" * 16, invoked=10)
        self.cargo_unit("app", "b" * 16, invoked=20, profile=2)
        self.cargo_unit("app", "c" * 16, kind="test-lib", invoked=30)
        self.prune()
        for unit_hash in ("a", "b", "c"):
            self.assertTrue((self.target / f"debug/.fingerprint/app-{unit_hash * 16}").exists())

    def test_incremental_state_goes_with_the_compile_that_wrote_it(self):
        self.cargo_unit("app", "a" * 16, invoked=10)
        self.cargo_unit("app", "b" * 16, invoked=110)
        old = self.incremental("app", "old", at=13)
        new = self.incremental("app", "new", at=113)
        self.prune()
        self.assertFalse(old.exists())
        self.assertTrue(new.exists())

    def test_a_failed_compile_keeps_the_incremental_state_it_wrote(self):
        self.cargo_unit("app", "b" * 16, invoked=110, failed=True)
        session = self.incremental("app", "new", at=300)
        self.prune()
        self.assertTrue(session.exists())

    def test_a_unit_without_a_fingerprint_yet_is_kept(self):
        self.cargo_unit("app", "b" * 16, invoked=110)
        compiling = self.target / f"debug/.fingerprint/app-{'c' * 16}"
        compiling.mkdir()
        output = self.artifact(f"debug/deps/app-{'c' * 16}.d", 1, age_days=0)
        self.prune()
        self.assertTrue(compiling.exists())
        self.assertTrue(output.exists())

    def test_the_hook_reports_superseded_units_under_the_cap(self):
        root = self.project("app")
        self.target = root / "target"
        self.cargo_unit("app", "a" * 16, invoked=10)
        self.cargo_unit("app", "b" * 16, invoked=110)
        event = {"tool_name": "Bash", "cwd": str(root), "tool_input": {"command": "cargo build"}}
        result = self.run_script("--hook", stdin=json.dumps(event))
        self.assertFalse((self.target / f"debug/.fingerprint/app-{'a' * 16}").exists())
        self.assertIn("superseded", json.loads(result.stdout)["systemMessage"])

    def test_a_unit_still_read_is_kept_over_one_written_later_but_unused(self):
        read = self.artifact("debug/deps/libregex-a.rlib", 4, age_days=30)
        os.utime(read, (1_700_000_000, read.stat().st_mtime))
        unused = self.artifact("debug/deps/libtokei-old.rlib", 4, age_days=9)
        self.prune("--max-kib", "6", "--floor-kib", "4")
        self.assertTrue(read.exists())
        self.assertFalse(unused.exists())


if __name__ == "__main__":
    unittest.main()
