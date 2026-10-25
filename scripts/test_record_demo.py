"""Recording owns its staging directory and removes it when VHS finishes or fails."""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


@unittest.skipUnless(shutil.which("bash"), "Bash is required by the recorder")
class RecordingLifetimeTests(unittest.TestCase):
    def record(self, root, fail=False):
        """Runs the recorder with stub tools, staging at `root/stage` in place of /tmp/locmetry-demo."""
        scripts = root / "scripts"
        scripts.mkdir()
        (root / "docs/assets").mkdir(parents=True)
        recorder = Path(__file__).with_name("record-demo.sh").read_text()
        (scripts / "record-demo.sh").write_text(recorder.replace("/tmp/locmetry-demo", str(root / "stage")))
        shutil.copyfile(Path(__file__).with_name("demo.tape"), scripts / "demo.tape")
        (scripts / "showcase-env.sh").write_text('''
demo_directory=$(mktemp -d "$TEST_ROOT/work.XXXXXXXX")
trap 'rm -rf "$demo_directory"' EXIT
showcase_tag=v3.3.4
baseline_tag=v3.2.0
cd "$demo_directory"
mkdir -p "$showcase_tag/immich" "$baseline_tag/immich"
echo source > "$showcase_tag/immich/source"
''')
        commands = {
            "cargo": "exit 0",
            "locmetry": 'printf "{}" > "$4"',
            "gifsicle": 'printf gif > "${@: -1}"',
            "vhs": '''
test -f "$LOCMETRY_DEMO_STAGE/immich/source"
printf '%s\\n' "$LOCMETRY_DEMO_STAGE" > "$TEST_ROOT/used-stage"
test "$TEST_FAIL" = 0
printf gif > "${@: -1}"
''',
        }
        binaries = root / "bin"
        binaries.mkdir()
        for name, body in commands.items():
            path = binaries / name
            path.write_text("#!/usr/bin/env bash\nset -eu\n" + body + "\n")
            path.chmod(0o755)
        env = dict(os.environ, PATH=str(binaries) + os.pathsep + os.environ["PATH"],
                   TEST_ROOT=str(root), TEST_FAIL=str(int(fail)))
        return subprocess.run(["bash", str(scripts / "record-demo.sh")], env=env,
                              capture_output=True, text=True)

    def test_recording_stages_at_its_fixed_path_and_removes_it_on_success_and_failure(self):
        # The GIF shows the stage's canonical root, so a random suffix would
        # change the recording on every run (seen 2026-10-25 after 7c1fc43).
        for fail in [False, True]:
            with self.subTest(fail=fail), tempfile.TemporaryDirectory() as folder:
                root = Path(folder).resolve()
                result = self.record(root, fail)
                self.assertEqual(result.returncode == 0, not fail, result.stderr)
                self.assertEqual((root / "used-stage").read_text().strip(), str(root / "stage"))
                self.assertFalse((root / "stage").exists())
                self.assertEqual(list(root.glob("work.*")), [])
                self.assertEqual((root / "docs/assets/demo.gif").exists(), not fail)

    def test_recording_refuses_a_stage_it_did_not_create(self):
        with tempfile.TemporaryDirectory() as folder:
            root = Path(folder).resolve()
            (root / "stage").mkdir()
            (root / "stage/unrelated.txt").write_text("keep")
            result = self.record(root)
            self.assertNotEqual(result.returncode, 0)
            self.assertIn(str(root / "stage"), result.stderr)
            self.assertEqual((root / "stage/unrelated.txt").read_text(), "keep")
            self.assertFalse((root / "used-stage").exists())
            self.assertFalse((root / "docs/assets/demo.gif").exists())
            self.assertEqual(list(root.glob("work.*")), [])


if __name__ == "__main__":
    unittest.main()
