#!/usr/bin/env bash
set -euo pipefail

repository=$(cd "$(dirname "$0")/.." && pwd)
binary=${LOCMETRY_BIN:-"$repository/target/debug/locmetry"}
command -v jq >/dev/null
temporary=$(mktemp -d)
trap 'rm -rf "$temporary"' EXIT
mkdir "$temporary/bin"
ln -s "$binary" "$temporary/bin/locmetry"
export PATH="$temporary/bin:$PATH"
export XDG_CONFIG_HOME="$temporary/no-user-config"

# Run the README itself so a separately maintained example cannot hide drift.
awk '
    /^## Continuous integration$/ { section = 1; next }
    section && /^## / { exit }
    section && /^```(sh|bash)$/ { block = 1; next }
    block && /^```$/ { exit }
    block { print }
' "$repository/README.md" > "$temporary/recipe.sh"
test -s "$temporary/recipe.sh"

check_recipe() {
    local scenario=$1 expected_status=$2 expected_text=$3 status=0
    (cd "$temporary/$scenario" && bash "$temporary/recipe.sh") > "$temporary/out" 2> "$temporary/err" || status=$?
    if [[ $status != "$expected_status" ]]; then
        cat "$temporary/out" "$temporary/err"
        echo "$scenario: expected exit $expected_status, got $status" >&2
        exit 1
    fi
    if [[ -n $expected_text ]]; then
        [[ $(<"$temporary/out") == *"$expected_text"* ]]
    fi
}

mkdir "$temporary/empty" "$temporary/healthy" "$temporary/partial" "$temporary/invalid" "$temporary/unclaimed"
printf 'version = 1\n[classification]\nfallback = "production"\n' > "$temporary/healthy/.locmetryrc"
printf 'print(1)\n' > "$temporary/healthy/main.py"
printf '{}\n' > "$temporary/partial/broken.ipynb"
printf 'not valid toml\n' > "$temporary/invalid/.locmetryrc"
for ((line = 0; line < 51; line++)); do printf 'print(1)\n'; done > "$temporary/unclaimed/main.py"

check_recipe empty 0 'test/production: n/a (no production code)'
check_recipe healthy 0 'test/production: 0%'
check_recipe partial 2 ''
check_recipe invalid 1 ''
check_recipe unclaimed 1 ''
echo 'README CI recipes pass: empty, healthy, partial, invalid, and policy failure.'
