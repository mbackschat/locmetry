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

# Execute the README itself, so edited flags or policy logic cannot evade the guard.
extract_blocks() {
    local heading=$1 destination=$2 wanted=${3:-0}
    printf 'set -eu\n' > "$destination"
    awk -v heading="$heading" -v wanted="$wanted" '
        $0 == "## " heading { section = 1; next }
        section && /^## / { exit }
        section && /^```(sh|bash)$/ { block = 1; number++; next }
        block && /^```$/ { block = 0; next }
        block && (wanted == 0 || wanted == number) { print }
    ' "$repository/README.md" >> "$destination"
    [[ $(wc -l < "$destination") -gt 1 ]]
}
extract_blocks 'Continuous integration' "$temporary/recipe.sh" 1

check_recipe() {
    local scenario=$1 expected_status=$2 expected_out=$3 expected_err=$4 status=0
    (cd "$temporary/$scenario" && bash "$temporary/recipe.sh") > "$temporary/out" 2> "$temporary/err" || status=$?
    if [[ $status != "$expected_status" ]]; then
        cat "$temporary/out" "$temporary/err"
        echo "$scenario: expected exit $expected_status, got $status" >&2
        exit 1
    fi
    if [[ -n $expected_out ]]; then
        [[ $(<"$temporary/out") == *"$expected_out"* ]]
    else
        [[ ! -s "$temporary/out" ]]
    fi
    if [[ -n $expected_err ]]; then
        [[ $(<"$temporary/err") == *"$expected_err"* ]]
    else
        [[ ! -s "$temporary/err" ]]
    fi
}

mkdir "$temporary/empty" "$temporary/healthy" "$temporary/partial" "$temporary/invalid" "$temporary/unclaimed"
printf 'version = 1\n[classification]\nfallback = "production"\n' > "$temporary/healthy/.locmetryrc"
printf 'print(1)\n' > "$temporary/healthy/main.py"
printf '{}\n' > "$temporary/partial/broken.ipynb"
printf 'not valid toml\n' > "$temporary/invalid/.locmetryrc"
for ((line = 0; line < 51; line++)); do printf 'print(1)\n'; done > "$temporary/unclaimed/main.py"

check_recipe empty 0 'test/production: n/a (no production code)' ''
check_recipe healthy 0 'test/production: 0%' ''
check_recipe partial 2 '' 'invalid_notebook'
check_recipe invalid 1 '' 'Invalid configuration'
check_recipe unclaimed 1 'false' ''

mkdir -p "$temporary/workflows/src" "$temporary/workflows/service/src" "$temporary/workflows/api/src"
for project in '' service/ api/; do
    printf 'fn main() {}\n' > "$temporary/workflows/${project}src/main.rs"
done
printf 'version = 1\nroots = ["api", "service"]\n' > "$temporary/workflows/repos.toml"
extract_blocks 'Explain one file' "$temporary/explain.sh"
extract_blocks 'Compare saved measurements' "$temporary/compare.sh"
(cd "$temporary/workflows" && bash "$temporary/explain.sh" && bash "$temporary/compare.sh") > "$temporary/workflow-output"
jq -e '.document_type == "comparison" and .summary.source.delta.code == 0' "$temporary/workflows/net-change.json" > /dev/null

# The same documented inspection recipe must retain an actual partial-input status.
printf '{}\n' > "$temporary/workflows/service/broken.ipynb"
status=0
(cd "$temporary/workflows" && locmetry --root service --output after.json) > "$temporary/out" 2> "$temporary/err" || status=$?
[[ $status == 2 ]]
extract_blocks 'Compare saved measurements' "$temporary/partial-compare.sh" 3
status=0
(cd "$temporary/workflows" && bash "$temporary/partial-compare.sh") > "$temporary/out" 2> "$temporary/err" || status=$?
[[ $status == 2 ]]
jq -s -e 'any(.[]; .code == "coverage_gap")' "$temporary/out" > /dev/null
[[ $(<"$temporary/err") == *'partial input measurements'* ]]
echo 'README examples pass: policy outcomes, compare, workspace compare, explain, and partial coverage.'
