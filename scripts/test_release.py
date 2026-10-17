"""Release preparation and publication guards, without network or Cargo builds."""

import importlib.util
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("release", Path(__file__).with_name("release.py"))
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


class ReleaseTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.addCleanup(self.tmp.cleanup)
        self.root = Path(self.tmp.name)
        self.git("init", "-q", "-b", "main")
        self.git("config", "user.name", "Release Test")
        self.git("config", "user.email", "mbackschat@mail.de")
        self.git("remote", "add", "origin", "https://github.com/mbackschat/locmetry.git")
        self.write("Cargo.toml", '[package]\nname = "locmetry"\nversion = "0.4.0"\n')
        self.write("Cargo.lock", '[[package]]\nname = "locmetry"\nversion = "0.4.0"\n\n[[package]]\nname = "other"\nversion = "0.4.0"\n')
        self.write("README.md", '''The latest stable release is **0.4.0**.
### Stable release (v0.4.0)
curl /download/v0.4.0/locmetry-v0.4.0-$target.tar.gz
install "locmetry-v0.4.0-$target/locmetry"
cargo install --tag v0.4.0
This checkout identifies as `0.4.0`.
    "locmetry_version": "0.4.0",
''')
        self.git("add", ".")
        self.git("commit", "-qm", "Initial fixture")

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.root, text=True).strip()

    def write(self, name, text):
        (self.root / name).write_text(text)

    def test_prepare_updates_only_release_identity_and_is_repeatable(self):
        release.prepare(self.root, "0.4.1")
        first = {name: (self.root / name).read_text() for name in release.VERSION_FILES}
        release.prepare(self.root, "0.4.1")
        self.assertEqual(first, {name: (self.root / name).read_text() for name in first})
        self.assertIn('name = "other"\nversion = "0.4.0"', first["Cargo.lock"])
        self.assertNotIn("0.4.0", first["README.md"])

    def test_inconsistent_readme_fails_before_any_write(self):
        self.write("README.md", "Missing release markers\n")
        before = (self.root / "Cargo.toml").read_text()
        with self.assertRaises(release.ReleaseError):
            release.prepare(self.root, "0.4.1")
        self.assertEqual(before, (self.root / "Cargo.toml").read_text())

    def test_invalid_versions_and_downgrades_are_rejected(self):
        for version in ("v0.4.1", "0.4.1;echo bad", "0.4", "00.4.1", "0.3.9"):
            with self.subTest(version=version), self.assertRaises(release.ReleaseError):
                release.prepare(self.root, version)

    def test_unrelated_dirty_files_are_not_committed(self):
        self.write("notes.txt", "private notes")
        with self.assertRaises(release.ReleaseError):
            release.check_worktree(self.root, "0.4.1")

    def test_modified_release_file_is_not_silently_committed(self):
        with (self.root / "README.md").open("a") as file:
            file.write("Unrelated edit\n")
        with self.assertRaises(release.ReleaseError):
            release.check_worktree(self.root, "0.4.1")

    def test_exact_preparation_can_be_resumed(self):
        release.prepare(self.root, "0.4.1")
        release.check_worktree(self.root, "0.4.1")

    def test_wrong_identity_remote_and_branch_are_rejected(self):
        for key, value in (("user.email", "wrong@example.org"), ("remote.origin.url", "https://github.com/elsewhere/project.git")):
            original = self.git("config", key)
            self.git("config", key, value)
            with self.assertRaises(release.ReleaseError):
                release.check_publisher(self.root)
            self.git("config", key, original)
        self.git("checkout", "-qb", "feature")
        with self.assertRaises(release.ReleaseError):
            release.check_publisher(self.root)

    def test_failed_checks_do_not_commit_tag_or_push(self):
        head = self.git("rev-parse", "HEAD")
        with patch.object(release, "check_remote", return_value=False), patch.object(release, "verify", side_effect=release.ReleaseError("test failed")):
            with self.assertRaises(release.ReleaseError):
                release.execute(self.root, "0.4.1", publish=True)
        self.assertEqual(head, self.git("rev-parse", "HEAD"))
        self.assertEqual("", self.git("tag", "--list"))

    def test_local_tag_conflict_is_not_replaced(self):
        self.git("tag", "v0.4.1")
        with self.assertRaises(release.ReleaseError):
            release.check_local_tag(self.root, "0.4.1")

    def bare_remote(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        remote = Path(temporary.name) / "remote.git"
        subprocess.run(["git", "init", "--bare", "-q", str(remote)], check=True)
        self.git("remote", "set-url", "origin", str(remote))
        return remote

    def test_atomic_push_rejection_does_not_publish_tag(self):
        remote = self.bare_remote()
        old = self.git("rev-parse", "HEAD")
        self.write("new.txt", "Remote change\n")
        self.git("add", "new.txt")
        self.git("commit", "-qm", "Advance remote")
        self.git("push", "-q", "origin", "main")
        self.git("reset", "--hard", old)
        self.git("tag", "-a", "v0.4.1", "-m", "Release")
        with self.assertRaises(subprocess.CalledProcessError):
            release.push(self.root, "0.4.1")
        tags = subprocess.check_output(["git", "--git-dir", str(remote), "tag", "--list"], text=True)
        self.assertEqual("", tags)

    def test_remote_tag_conflict_and_newer_main_are_rejected(self):
        self.bare_remote()
        old = self.git("rev-parse", "HEAD")
        self.write("new.txt", "Remote change\n")
        self.git("add", "new.txt")
        self.git("commit", "-qm", "Advance remote")
        self.git("push", "-q", "origin", "main")
        self.git("reset", "--hard", old)
        with self.assertRaises(release.ReleaseError):
            release.check_remote(self.root, "0.4.1")
        self.git("tag", "v0.4.1", "origin/main")
        self.git("push", "-q", "origin", "v0.4.1")
        with self.assertRaises(release.ReleaseError):
            release.check_remote(self.root, "0.4.1")

    def test_publication_and_retry_against_a_real_remote(self):
        self.bare_remote()
        self.git("push", "-q", "origin", "main")
        with patch.object(release, "check_publisher"), patch.object(release, "verify") as verify:
            release.execute(self.root, "0.4.1", publish=True)
            release.execute(self.root, "0.4.1", publish=True)
            verify.assert_called_once_with(self.root, "0.4.1")
        head = self.git("rev-parse", "HEAD")
        refs = self.git("ls-remote", "origin", "refs/heads/main", "refs/tags/v0.4.1^{}")
        self.assertEqual([head, head], [line.split()[0] for line in refs.splitlines()])

    def test_prepare_only_never_commits_or_tags(self):
        head = self.git("rev-parse", "HEAD")
        with patch.object(release, "verify") as verify:
            release.execute(self.root, "0.4.1", publish=False)
            verify.assert_called_once_with(self.root, "0.4.1")
        self.assertEqual(head, self.git("rev-parse", "HEAD"))
        self.assertEqual("", self.git("tag", "--list"))

    def test_publish_pushes_branch_and_tag_atomically_and_retry_is_safe(self):
        with patch.object(release, "check_remote", return_value=False), patch.object(release, "verify"), patch.object(release, "push") as push:
            release.execute(self.root, "0.4.1", publish=True)
            push.assert_called_once_with(self.root, "0.4.1")
        head = self.git("rev-parse", "HEAD")
        self.assertEqual(head, self.git("rev-parse", "v0.4.1^{commit}"))
        self.assertEqual("", self.git("status", "--porcelain"))
        with patch.object(release, "check_remote", return_value=True), patch.object(release, "verify") as verify:
            release.execute(self.root, "0.4.1", publish=True)
            verify.assert_not_called()
        self.assertEqual(head, self.git("rev-parse", "HEAD"))


if __name__ == "__main__":
    unittest.main()
