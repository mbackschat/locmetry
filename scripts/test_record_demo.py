"""Recording owns its staging directory and removes it when VHS finishes or fails."""

import os
from pathlib import Path
import shutil
import subprocess
import tempfile
import unittest


@unittest.skipUnless(shutil.which("bash"), "Bash is required by the recorder")
class RecordingLifetimeTests(unittest.TestCase):
    def test_recording_cleans_only_its_own_stage_on_success_and_failure(self):
        for fail in [False, True]:
            with self.subTest(fail=fail), tempfile.TemporaryDirectory() as folder:
                root = Path(folder)
                scripts = root / "scripts"
                scripts.mkdir()
                (root / "docs/assets").mkdir(parents=True)
                legacy = root / "legacy-stage"
                legacy.mkdir()
                (legacy / "unrelated.txt").write_text("keep")
                recorder = Path(__file__).with_name("record-demo.sh").read_text()
                (scripts / "record-demo.sh").write_text(recorder.replace("/tmp/locmetry-demo", str(legacy)))
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
stage=${LOCMETRY_DEMO_STAGE:-"$TEST_ROOT/legacy-stage"}
test -f "$stage/immich/source"
printf '%s\\n' "$stage" > "$TEST_ROOT/used-stage"
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
                result = subprocess.run(["bash", str(scripts / "record-demo.sh")], env=env,
                                        capture_output=True, text=True)
                self.assertEqual(result.returncode == 0, not fail, result.stderr)
                self.assertTrue((legacy / "unrelated.txt").exists(), "an existing stage belongs to another run")
                used = Path((root / "used-stage").read_text().strip())
                self.assertFalse(used.exists())
                self.assertEqual(list(root.glob("work.*")), [])
                self.assertEqual((root / "docs/assets/demo.gif").exists(), not fail)


if __name__ == "__main__":
    unittest.main()
