# Source from the repository root in a Bash example. Keep generated reports outside measured roots.
set -euo pipefail
repository=$(pwd)
demo_binary=${LOCMETRY_BIN:-"$repository/target/debug/locmetry"}
[[ -x "$demo_binary" ]]
demo_directory=$(mktemp -d)
trap 'rm -rf "$demo_directory"' EXIT
cp -R "$repository/examples/cli-workspace/." "$demo_directory/"
mkdir "$demo_directory/bin"
ln -s "$demo_binary" "$demo_directory/bin/locmetry"
export PATH="$demo_directory/bin:$PATH"
export XDG_CONFIG_HOME="$demo_directory/personal-config"
export COLUMNS=100
export LC_ALL=C
cd "$demo_directory"

# Showboat records this display transform; saved reports retain their original formatting.
demo_wrap() {
    python3 -c 'import sys, textwrap
for line in sys.stdin:
    line = line.rstrip("\n")
    print(textwrap.fill(line, width=120, replace_whitespace=False,
                        break_long_words=False, break_on_hyphens=False)
          if len(line) > 120 else line)'
}
