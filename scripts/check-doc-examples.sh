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

# Execute the documentation sources themselves, so edited flags or policy logic cannot evade the guard.
extract_blocks() {
    local heading=$1 destination=$2 wanted=${3:-0} change_source=${4:-0}
    local source=${5:-"$repository/README.md"}
    printf 'set -eu\n' > "$destination"
    awk -v heading="$heading" -v wanted="$wanted" -v change_source="$change_source" '
        $0 == "## " heading || $0 == "### " heading { section = 1; depth = ($0 ~ /^### / ? 3 : 2); next }
        section && (/^## / || (depth == 3 && /^### /)) { exit }
        section && /^```(sh|bash)$/ { block = 1; number++; next }
        block && /^```$/ { block = 0; next }
        block && (wanted == 0 || wanted == number) {
            if ($0 ~ /^locmetry .* --output / && $0 !~ / --output -([ ]|$)/ && $0 !~ / --print / && $0 !~ /\|\|/) {
                print $0 " > export.stdout"
                print "test ! -s export.stdout"
            } else {
                print
            }
            if (change_source && /^# After changing/) {
                print "printf '\''fn added() {}\\n'\'' >> service/src/main.rs"
            }
        }
    ' "$source" >> "$destination"
    [[ $(wc -l < "$destination") -gt 1 ]]
}
extract_blocks 'Continuous integration' "$temporary/recipe.sh" 1 0 "$repository/docs/guides/reports.md"

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
    extract_blocks 'Explain one file' "$temporary/explain.sh" "$block" 0 "$repository/docs/guides/selection.md"
    (cd "$temporary/workflows" && bash "$temporary/explain.sh") > "$temporary/explanation.json"
    jq -e '.result.state == "counted" and .result.file.counts.code == 1' "$temporary/explanation.json" > /dev/null
done
extract_blocks 'Explain one file' "$temporary/explain-markdown.sh" 2 0 "$repository/docs/guides/selection.md"
(cd "$temporary/workflows" && bash "$temporary/explain-markdown.sh") > "$temporary/explanation.md"
grep -Fq 'counted' "$temporary/explanation.md"
extract_blocks 'Explain one file' "$temporary/explain-schema.sh" 4 0 "$repository/docs/guides/selection.md"
(cd "$temporary/workflows" && bash "$temporary/explain-schema.sh") > "$temporary/explanation-schema.json"
jq -e '."$schema"' "$temporary/explanation-schema.json" > /dev/null

# Retain the short README entry point as well as the fuller guide recipes.
extract_blocks 'Compare saved measurements' "$temporary/readme-compare.sh" 1 1
(cd "$temporary/workflows" && bash "$temporary/readme-compare.sh") > "$temporary/readme-comparison.json"
jq -e '.summary.source.delta.code == 1' "$temporary/readme-comparison.json" > /dev/null
printf 'fn main() {}\n' > "$temporary/workflows/service/src/main.rs"

# Supply the edit explicitly requested by each documentation comment between snapshots.
extract_blocks 'Compare saved measurements' "$temporary/compare.sh" 1 1 "$repository/docs/guides/compare.md"
(cd "$temporary/workflows" && bash "$temporary/compare.sh") > "$temporary/workflow-output"
jq -e '.document_type == "comparison" and .summary.source.delta.code == 1' "$temporary/workflows/net-change.json" > /dev/null
grep -Eq '^\| Source \| [0-9,]+ \| [0-9,]+ \| [^|]+ \| 1 \| 2 \| \+1 \|' "$temporary/workflows/net-change.md"
extract_blocks 'Compare saved measurements' "$temporary/workspace-compare.sh" 2 1 "$repository/docs/guides/compare.md"
(cd "$temporary/workflows" && bash "$temporary/workspace-compare.sh") > "$temporary/workspace-change.md"
grep -Eq '^\| Source \| [0-9,]+ \| [0-9,]+ \| [^|]+ \| 3 \| 4 \| \+1 \|' "$temporary/workspace-change.md"
grep -Fxq '| service (both) | 2 | 3 | +1 | 2 | 3 | +1 |' "$temporary/workspace-change.md"

extract_blocks 'Compare saved measurements' "$temporary/readme-member.sh" 2 0
(cd "$temporary/workflows" && bash "$temporary/readme-member.sh") > "$temporary/readme-member.txt"
grep -Fq 'Member: service (both)' "$temporary/readme-member.txt"
extract_blocks 'Inspect one workspace member' "$temporary/member-compare.sh" 0 0 "$repository/docs/guides/compare.md"
(cd "$temporary/workflows" && bash "$temporary/member-compare.sh") > "$temporary/member-change.txt"
grep -Eq '^ src \(both\) +2 +3 +\+1' "$temporary/member-change.txt"
jq -e '.member == "service" and .summary.source.before.code == 2 and .summary.source.after.code == 3 and .summary.source.delta.code == 1 and .folders["service"].recursive.source == .summary.source and (.members | keys) == ["service"]' "$temporary/workflows/member-change.json" > /dev/null

# The agent recipe prints the table and saves full JSON from one scan.
extract_blocks 'For agents' "$temporary/agents.sh" 1 0
(cd "$temporary/workflows/service" && bash "$temporary/agents.sh") > "$temporary/agents.stdout"
grep -q '^ Language ' "$temporary/agents.stdout"
jq -e '.breakdown | length > 0' "$temporary/workflows/report.json" > /dev/null
# The same documented inspection recipe must retain an actual partial-input status.
printf '{}\n' > "$temporary/workflows/service/broken.ipynb"
status=0
(cd "$temporary/workflows" && locmetry --root service --output after.json) > "$temporary/out" 2> "$temporary/err" || status=$?
[[ $status == 2 ]]
[[ ! -s "$temporary/out" ]]
extract_blocks 'Compare saved measurements' "$temporary/partial-compare.sh" 3 0 "$repository/docs/guides/compare.md"
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
    case "$block" in
        1|2|3) source="$repository/docs/guides/reports.md"; source_block=$block ;;
        4|5|6) source="$repository/docs/guides/selection.md"; source_block=$((block - 3)) ;;
        7|8) source="$repository/docs/guides/reports.md"; source_block=$((block - 3)) ;;
    esac
    extract_blocks 'Common issues and useful queries' "$temporary/issue-query.sh" "$source_block" 0 "$source"
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
# The adaptive example saves a table even though stdout is redirected by this guard.
extract_blocks 'Look inside large folders' "$temporary/adaptive.sh" 2 0 "$repository/docs/guides/reports.md"
(cd "$temporary/issues" && bash "$temporary/adaptive.sh") > "$temporary/adaptive.stdout"
[[ ! -s "$temporary/adaptive.stdout" ]]
grep -Fq 'Adaptive folder detail; rows cover measured source once.' "$temporary/issues/folders.txt"
grep -Eq '^ web/src +2 .*1 +1 +0 .*1\.00' "$temporary/issues/folders.txt"

# Two test folders distinguish a language sum from a list of per-folder rows.
mkdir -p "$temporary/issues/acme-orders/web/specs"
printf 'test();\n' > "$temporary/issues/acme-orders/web/specs/secondary.test.ts"
extract_blocks 'Query a Saved Report' "$temporary/saved-queries.sh" 0 0 "$repository/docs/guides/reports.md"
(cd "$temporary/issues" && bash "$temporary/saved-queries.sh") > "$temporary/saved-query-results"
jq -s -e '.[0] == [{language: "TypeScript", code: 2}] and .[1] == [{purpose: "production", code: 1}, {purpose: "test", code: 2}] and .[2].path == "prototypes/pricing_experiment.py"' "$temporary/saved-query-results" > /dev/null

# Execute the tour against the same checked-in source used by its recorded guides.
mkdir "$temporary/tour"
cp -R "$repository/examples/cli-workspace/." "$temporary/tour/"
tour="$repository/docs/sample-project-tour.md"
for heading in 'Read the first measurement' 'Find where the code resides' 'Review what remains unresolved'; do
    extract_blocks "$heading" "$temporary/tour-view.sh" 0 0 "$tour"
    (cd "$temporary/tour" && bash "$temporary/tour-view.sh") > "$temporary/tour-view.txt"
    grep -Eq '^Root: .*/acme-orders$' "$temporary/tour-view.txt"
done
extract_blocks 'Trace a classification decision' "$temporary/tour-explain.sh" 0 0 "$tour"
(cd "$temporary/tour" && bash "$temporary/tour-explain.sh") > "$temporary/tour-explanation.json"
jq -e '.result.file.purpose == "test" and .result.file.rule == "builtin:js-test-file"' "$temporary/tour-explanation.json" > /dev/null
extract_blocks 'Save the evidence and ask a narrower question' "$temporary/tour-save.sh" 0 0 "$tour"
(cd "$temporary/tour" && bash "$temporary/tour-save.sh") > "$temporary/tour-query.json"
jq -e '. == [{language: "Go", code: 4}, {language: "TypeScript", code: 3}]' "$temporary/tour-query.json" > /dev/null
extract_blocks 'Share a report and follow changes' "$temporary/tour-share.sh" 0 0 "$tour"
(cd "$temporary/tour" && bash "$temporary/tour-share.sh") > "$temporary/tour-comparison.json"
grep -Fq '## Source categories' "$temporary/tour/acme-orders-report.md"
jq -e '.document_type == "comparison" and .summary.source.delta.code == 1' "$temporary/tour-comparison.json" > /dev/null
extract_blocks 'Measure the members together' "$temporary/tour-workspace.sh" 0 0 "$tour"
(cd "$temporary/tour" && bash "$temporary/tour-workspace.sh") > "$temporary/tour-workspace.txt"
grep -q 'service' "$temporary/tour-workspace.txt"

# Measured README table excerpts are checked by cargo test --locked --test integration readme::.
echo 'README and guide examples pass: silent file exports, Markdown content, policy outcomes, compare, workspace compare, explain, partial coverage, troubleshooting queries, adaptive folder detail, and the sample-project tour.'
