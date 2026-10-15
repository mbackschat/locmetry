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
    local heading=$1 destination=$2 wanted=${3:-0} change_source=${4:-0}
    printf 'set -eu\n' > "$destination"
    awk -v heading="$heading" -v wanted="$wanted" -v change_source="$change_source" '
        $0 == "## " heading { section = 1; next }
        section && /^## / { exit }
        section && /^```(sh|bash)$/ { block = 1; number++; next }
        block && /^```$/ { block = 0; next }
        block && (wanted == 0 || wanted == number) {
            print
            if (change_source && /^# After changing/) {
                print "printf '\''fn added() {}\\n'\'' >> service/src/main.rs"
            }
        }
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
for block in 1 3; do
    extract_blocks 'Explain one file' "$temporary/explain.sh" "$block"
    (cd "$temporary/workflows" && bash "$temporary/explain.sh") > "$temporary/explanation.json"
    jq -e '.result.state == "counted" and .result.file.counts.code == 1' "$temporary/explanation.json" > /dev/null
done
extract_blocks 'Explain one file' "$temporary/explain-markdown.sh" 2
(cd "$temporary/workflows" && bash "$temporary/explain-markdown.sh") > "$temporary/explanation.md"
grep -Fq 'counted' "$temporary/explanation.md"
extract_blocks 'Explain one file' "$temporary/explain-schema.sh" 4
(cd "$temporary/workflows" && bash "$temporary/explain-schema.sh") > "$temporary/explanation-schema.json"
jq -e '."$schema"' "$temporary/explanation-schema.json" > /dev/null

# Supply the edit explicitly requested by each README comment between snapshots.
extract_blocks 'Compare saved measurements' "$temporary/compare.sh" 1 1
(cd "$temporary/workflows" && bash "$temporary/compare.sh") > "$temporary/workflow-output"
jq -e '.document_type == "comparison" and .summary.source.delta.code == 1' "$temporary/workflows/net-change.json" > /dev/null
extract_blocks 'Compare saved measurements' "$temporary/workspace-compare.sh" 2 1
(cd "$temporary/workflows" && bash "$temporary/workspace-compare.sh") > "$temporary/workspace-change.md"
grep -Fxq '| Source code | 3 | 4 | +1 |' "$temporary/workspace-change.md"
grep -Fxq '| service (both) | 2 | 3 | +1 |' "$temporary/workspace-change.md"

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
# Keep troubleshooting queries executable against the README's synthetic layout.
mkdir -p "$temporary/issues/acme-orders/"{web/src,prototypes,docs}
printf '{}\n' > "$temporary/issues/acme-orders/web/package.json"
printf 'const cart = 1;\n' > "$temporary/issues/acme-orders/web/src/cart.ts"
printf 'test();\n' > "$temporary/issues/acme-orders/web/src/cart.test.ts"
printf 'print(1)\n' > "$temporary/issues/acme-orders/prototypes/pricing_experiment.py"
printf '# Architecture\n' > "$temporary/issues/acme-orders/docs/architecture.md"
printf 'synthetic image bytes' > "$temporary/issues/acme-orders/docs/order-flow.png"
for block in 1 2 3 4 5 6 7 8; do
    extract_blocks 'Common issues and useful queries' "$temporary/issue-query.sh" "$block"
    (cd "$temporary/issues" && bash "$temporary/issue-query.sh") > "$temporary/issue-out"
    case "$block" in
        1) grep -Fxq $'1\tPython\tprototypes/pricing_experiment.py' "$temporary/issue-out" ;;
        2) jq -e '.files == 1 and .code == 1' "$temporary/issue-out" > /dev/null ;;
        3) grep -Fxq $'1\tproduction\tbuiltin:npm-module\tweb/src/cart.ts' "$temporary/issue-out" ;;
        4) grep -Fq 'subfolder web/src' "$temporary/issue-out"; grep -Fq 'web/src (direct files)' "$temporary/issue-out" ;;
        5) jq -e '.result.state == "counted" and .result.file.purpose == "test"' "$temporary/issue-out" > /dev/null ;;
        6) jq -e '.result.state == "unrecognized"' "$temporary/issue-out" > /dev/null ;;
        7) jq -e '.unrecognized == ["docs/order-flow.png"] and .counted_files == 5' "$temporary/issue-out" > /dev/null ;;
        8) grep -Fxq $'data\t1\tweb/package.json' "$temporary/issue-out"; grep -Fxq $'documentation\t1\tdocs/architecture.md' "$temporary/issue-out" ;;
    esac
done
echo 'README examples pass: policy outcomes, compare, workspace compare, explain, partial coverage, and troubleshooting queries.'
