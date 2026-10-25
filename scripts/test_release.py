"""Release preparation and publication guards, without network or Cargo builds."""

import importlib.util
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import textwrap
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location("release", Path(__file__).with_name("release.py"))
release = importlib.util.module_from_spec(spec)
spec.loader.exec_module(release)


# The recorded root guide prints the executable version; its other text,
# including a version-like example, must survive a release unchanged.
RECORDING = """```bash
locmetry --version
locmetry guide --source
```

```output
locmetry {version}
Source: docs/guides/configure.md (locmetry {version})
```

Reports from locmetry 0.4.0 or later compare.
"""

CHANGELOG = """# Changelog

User-visible changes.

## [Unreleased]

### Added

- Something new.

## [0.4.0] - 2026-10-01

### Added

- First release.

[Unreleased]: https://github.com/mbackschat/locmetry/compare/v0.4.0...HEAD
[0.4.0]: https://github.com/mbackschat/locmetry/compare/v0.3.0...v0.4.0
"""


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
        self.write("README.md", '# Locmetry\n\nInstall from the latest release.\n\nIllustrative producer: 0.1.6.\n')
        (self.root / "docs/cli").mkdir(parents=True)
        self.write("docs/cli/root.md", RECORDING.format(version="0.4.0"))
        self.write("CHANGELOG.md", CHANGELOG)
        self.git("add", ".")
        self.git("commit", "-qm", "Initial fixture")

    def git(self, *args):
        return subprocess.check_output(["git", *args], cwd=self.root, text=True).strip()

    def write(self, name, text):
        (self.root / name).write_text(text)

    def test_prepare_updates_only_release_identity_and_is_repeatable(self):
        readme = (self.root / "README.md").read_bytes()
        release.prepare(self.root, "0.4.1")
        first = {name: (self.root / name).read_text() for name in release.VERSION_FILES}
        release.prepare(self.root, "0.4.1")
        self.assertEqual(first, {name: (self.root / name).read_text() for name in first})
        self.assertIn('name = "other"\nversion = "0.4.0"', first["Cargo.lock"])
        self.assertEqual(readme, (self.root / "README.md").read_bytes())
        self.assertEqual(set(release.VERSION_FILES), set(self.git("diff", "--name-only").splitlines()))
        self.assertEqual(RECORDING.format(version="0.4.1").replace("from locmetry 0.4.1", "from locmetry 0.4.0"),
                         first["docs/cli/root.md"])
        self.assertEqual("0.4.1", release.crate_version(first["Cargo.toml"]))
        self.assertIn('name = "locmetry"\nversion = "0.4.1"', first["Cargo.lock"])

    def test_prepare_releases_the_unreleased_changelog_entries(self):
        release.prepare(self.root, "0.4.1", today="2026-10-22")
        changelog = (self.root / "CHANGELOG.md").read_text()
        self.assertIn("## [Unreleased]\n\n## [0.4.1] - 2026-10-22\n\n### Added\n\n- Something new.\n", changelog)
        self.assertIn("[Unreleased]: https://github.com/mbackschat/locmetry/compare/v0.4.1...HEAD\n"
                      "[0.4.1]: https://github.com/mbackschat/locmetry/compare/v0.4.0...v0.4.1\n"
                      "[0.4.0]: https://github.com/mbackschat/locmetry/compare/v0.3.0...v0.4.0\n", changelog)
        self.assertEqual(release.release_notes(changelog, "0.4.1"), "### Added\n\n- Something new.\n")

    # A release page resolves relative links against /releases/tag/, not the repository.
    def test_release_notes_pin_repository_links_to_the_release_tag(self):
        changelog = "## [0.4.1] - 2026-10-22\n\n- See [install](README.md#install) and [docs](https://example.org).\n"
        self.assertEqual(release.release_notes(changelog, "0.4.1"),
                         "- See [install](https://github.com/mbackschat/locmetry/blob/v0.4.1/README.md#install)"
                         " and [docs](https://example.org).\n")

    def test_a_release_without_changelog_entries_fails_before_any_write(self):
        self.write("CHANGELOG.md", CHANGELOG.replace("### Added\n\n- Something new.\n\n", "", 1))
        before = (self.root / "Cargo.toml").read_text()
        with self.assertRaises(release.ReleaseError):
            release.prepare(self.root, "0.4.1")
        self.assertEqual(before, (self.root / "Cargo.toml").read_text())

    # A preparation resumed on a later day keeps the date it recorded.
    def test_preparation_resumed_on_another_day_keeps_its_release_date(self):
        release.prepare(self.root, "0.4.1", today="2026-10-22")
        release.check_worktree(self.root, "0.4.1", today="2026-10-23")
        release.prepare(self.root, "0.4.1", today="2026-10-23")
        self.assertIn("## [0.4.1] - 2026-10-22\n", (self.root / "CHANGELOG.md").read_text())

    def test_inconsistent_lockfile_fails_before_any_write(self):
        self.write("Cargo.lock", '[[package]]\nname = "locmetry"\nversion = "0.3.9"\n')
        before = (self.root / "Cargo.toml").read_text()
        with self.assertRaises(release.ReleaseError):
            release.prepare(self.root, "0.4.1")
        self.assertEqual(before, (self.root / "Cargo.toml").read_text())

    def test_recording_without_version_output_fails_before_any_write(self):
        self.write("docs/cli/root.md", "No recorded version here.\n")
        before = (self.root / "Cargo.toml").read_text()
        with self.assertRaises(release.ReleaseError):
            release.prepare(self.root, "0.4.1")
        self.assertEqual(before, (self.root / "Cargo.toml").read_text())

    def test_checks_replay_command_guides_with_the_ci_pins_after_building_them(self):
        with patch.object(release, "run", return_value="locmetry 0.4.1") as run:
            release.verify(self.root, "0.4.1")
        commands = [call.args[1:] for call in run.call_args_list]
        replay = next(command for command in commands if "scripts/check-recorded-guides.py" in command)
        self.assertIn("--set", replay)
        self.assertEqual("reference", replay[replay.index("--set") + 1])
        self.assertLess(commands.index(("cargo", "test", "--locked")), commands.index(replay))
        repository = Path(__file__).resolve().parents[1]
        workflow = (repository / ".github/workflows/ci.yml").read_text()
        step = workflow.split("      - name: Replay command guides\n", 1)[1].split("      - name:", 1)[0]
        pins = [replay[index + 1] for index, part in enumerate(replay) if part == "--with"]
        self.assertTrue(pins)
        for pin in pins:
            self.assertIn(f"--with {pin}", step)

    def test_invalid_versions_and_downgrades_are_rejected(self):
        for version in ("v0.4.1", "0.4.1;echo bad", "0.4", "00.4.1", "0.3.9"):
            with self.subTest(version=version), self.assertRaises(release.ReleaseError):
                release.prepare(self.root, version)

    def test_unrelated_dirty_files_are_not_committed(self):
        self.write("notes.txt", "private notes")
        with self.assertRaises(release.ReleaseError):
            release.check_worktree(self.root, "0.4.1")

    def test_readme_edit_is_not_silently_committed(self):
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


class PackageTests(unittest.TestCase):
    def test_release_notes_print_a_published_version_from_the_repository_changelog(self):
        script = Path(__file__).with_name("release.py")
        notes = subprocess.run([sys.executable, str(script), "0.7.6", "--notes"],
                               check=True, text=True, capture_output=True).stdout
        self.assertTrue(notes.startswith("### "), notes)
        self.assertNotIn("## [", notes)


    def test_release_archive_contains_an_executable_helper_with_optional_jq(self):
        repository = Path(__file__).resolve().parents[1]
        workflow = (repository / ".github/workflows/release.yml").read_text()
        package = workflow.split("      - name: Package\n        run: |\n", 1)[1].split("      - name:", 1)[0]
        package = textwrap.dedent(package).replace("${{ matrix.target }}", "test-target")
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            binary = root / "target/test-target/release/locmetry"
            binary.parent.mkdir(parents=True)
            binary.write_text("synthetic package payload\n")
            for name in ("README.md", "CHANGELOG.md", "LICENSE", "THIRD-PARTY-NOTICES.md"):
                (root / name).write_text(name)
            (root / "scripts").mkdir()
            shutil.copy2(repository / "scripts/compare-locmetry-files.sh", root / "scripts")
            # The Generate step writes the man page and completions here; Package ships them.
            (root / "target/generated/complete").mkdir(parents=True)
            (root / "target/generated/locmetry.1").write_text("man page\n")
            (root / "target/generated/complete/locmetry.bash").write_text("bash completion\n")
            env = dict(os.environ, GITHUB_REF_NAME="v0.0.0", GITHUB_EVENT_NAME="push", RUNNER_OS="Linux",
                       GITHUB_ENV=str(root / "github-env"))
            subprocess.run(["bash", "-eu", "-c", package], cwd=root, env=env, check=True, capture_output=True)
            name = "locmetry-v0.0.0-test-target"
            with tarfile.open(root / f"{name}.tar.gz") as archive:
                helper = archive.getmember(f"{name}/compare-locmetry-files.sh")
                self.assertTrue(helper.mode & 0o111)
                self.assertEqual(archive.extractfile(helper).read(),
                                 (repository / "scripts/compare-locmetry-files.sh").read_bytes())
                self.assertEqual(archive.extractfile(f"{name}/locmetry.1").read(), b"man page\n")
                self.assertEqual(archive.extractfile(f"{name}/complete/locmetry.bash").read(), b"bash completion\n")
            result = subprocess.run(
                ["/bin/bash", str(root / name / "compare-locmetry-files.sh"), "before.json", "after.json"],
                env=dict(os.environ, PATH=""), text=True, capture_output=True,
            )
            self.assertEqual(result.returncode, 1)
            self.assertEqual(result.stdout, "")
            self.assertIn("brew install jq", result.stderr)
            self.assertIn("package manager", result.stderr)

    def test_every_built_asset_is_checksummed_and_released(self):
        workflow = (Path(__file__).resolve().parents[1] / ".github/workflows/release.yml").read_text()
        lines = workflow.splitlines()
        start = lines.index("          path: |") + 1
        uploaded = []
        for line in lines[start:]:
            if not line.startswith(" " * 12):
                break
            uploaded.append(line.strip())
        checksummed = next(line for line in lines if "sha256sum" in line).split()
        released = next(line for line in lines if "gh release create" in line).split()
        self.assertTrue({"locmetry*.deb", "locmetry*.rpm"} <= set(uploaded), uploaded)
        for pattern in uploaded:
            self.assertIn(pattern, checksummed)
            self.assertIn(pattern, released)


if __name__ == "__main__":
    unittest.main()
