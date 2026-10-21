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

check_configuration_transitions() {
    local work="$temporary/configuration"
    mkdir -p "$work/project/"{qa,bin,archive,__test__/helpers}
    printf 'helper();\nhelper();\n' > "$work/project/qa/helper.js"
    printf 'run();\n' > "$work/project/qa/runner.js"
    printf 'check();\ncheck();\ncheck();\n' > "$work/project/qa/assertions.js"
    printf 'setup();\n' > "$work/project/__test__/helpers/setup.js"
    printf 'console.log("start");\n' > "$work/project/bin/launch"
    printf 'old();\n' > "$work/project/archive/old.js"
    printf '# History\n' > "$work/project/archive/history.md"
    printf '{"old":true}\n' > "$work/project/archive/data.json"
    cp -R "$work/project" "$work/unchanged-input"
    mkdir "$work/capture"
    cp -R "$work/project" "$work/capture/service"
    extract_blocks 'Inspect configuration transitions' "$work/capture.sh" 1 0 "$repository/docs/guides/compare.md"
    (cd "$work/capture" && bash "$work/capture.sh") > "$work/capture.stdout"
    jq -e '.detail == "full" and (.files | length) == 7' "$work/capture/before.json" "$work/capture/after.json" > /dev/null
    cat > "$work/baseline.toml" <<'TOML'
version = 1
[classification]
fallback = "production"
[[overrides]]
id = "archive"
match = ["archive/**"]
purpose = "other"
category = "historical"
reason = "Historical material remains counted"
TOML
    cp "$work/baseline.toml" "$work/classified.toml"
    cat >> "$work/classified.toml" <<'TOML'
[[languages.mappings]]
id = "node-launcher"
match = ["bin/launch"]
language = "JavaScript"
[[rules]]
id = "qa"
match = ["qa/**"]
purpose = "test"
TOML
    cat > "$work/refined.toml" <<'TOML'
version = 1
[classification]
fallback = "production"
[[languages.mappings]]
id = "node-launcher"
match = ["bin/launch"]
language = "JavaScript"
[[overrides]]
id = "archive"
match = ["archive/**"]
kind = ["source"]
purpose = "other"
category = "historical"
reason = "Only historical source needs this purpose decision"
[[overrides]]
id = "test-helpers"
match = ["__test__/helpers/**"]
purpose = "test"
category = "test-support"
[[rules]]
id = "qa-helpers"
match = ["qa/helper.js"]
purpose = "test"
category = "test-support"
[[rules]]
id = "qa-infrastructure"
match = ["qa/runner.js"]
purpose = "test"
category = "infrastructure"
[[rules]]
id = "qa-assertions"
match = ["qa/assertions.js"]
purpose = "test"
category = "assertions"
TOML
    local policy
    for policy in baseline classified refined; do
        (cd "$work" && locmetry --root project --config "$policy.toml" --no-user-config --files --output "$policy.json")
        diff -r "$work/unchanged-input" "$work/project"
    done
    local helper="$temporary/bin/compare-locmetry-files.sh"
    install -m 755 "$repository/scripts/compare-locmetry-files.sh" "$helper"
    cp "$work/baseline.json" "$work/old capture.json"
    cp "$work/classified.json" "$work/new capture.json"
    "$helper" "$work/old capture.json" "$work/new capture.json" > "$work/standalone.json"
    jq -e --arg before "$work/old capture.json" --arg after "$work/new capture.json" '
      .before.input == $before and .after.input == $after
      and any(.findings[]; .code == "policy_changed")
      and (.changes | length) == 4
    ' "$work/standalone.json" > /dev/null
    cmp "$work/baseline.json" "$work/old capture.json"
    cmp "$work/classified.json" "$work/new capture.json"
    extract_blocks 'Inspect configuration transitions' "$work/transitions.sh" 2 0 "$repository/docs/guides/compare.md"
    cp "$work/baseline.json" "$work/before.json"
    cp "$work/classified.json" "$work/after.json"
    (cd "$work" && bash transitions.sh)
    jq -e --slurpfile before "$work/before.json" --slurpfile after "$work/after.json" '
      .before.metadata == $before[0].metadata and .after.metadata == $after[0].metadata
      and .source_identity == "not established by reports"
      and (.changes | length) == 4
      and ([.changes[] | select(.before.purpose == "production" and .after.purpose == "test") | .after.counts.code] | add) == 6
      and ([.changes[] | select(.path == "bin/launch")] | .[0] |
        .before.state == "unrecognized" and .before.counts == null
        and .after.state == "counted" and .after.purpose == "production"
        and .after.language_mapping == "node-launcher" and .after.counts.code == 1 and .code_delta == null)
      and $before[0].summary.source_purpose.production.code == 6
      and $after[0].summary.source_purpose.production.code == 1
      and $before[0].summary.source_purpose.test.code == 1
      and $after[0].summary.source_purpose.test.code == 7
      and ($after[0].summary.source.code - $before[0].summary.source.code) == 1
    ' "$work/transitions.json" > /dev/null
    cp "$work/classified.json" "$work/before.json"
    cp "$work/refined.json" "$work/after.json"
    (cd "$work" && bash transitions.sh)
    jq -e --slurpfile before "$work/before.json" --slurpfile after "$work/after.json" '
      $before[0].summary.source_purpose == $after[0].summary.source_purpose
      and $after[0].summary.source_purpose.other.code == 1
      and (.changes | length) == 6
      and all(.changes[]; .code_delta == 0)
      and ([.changes[] | select(.after.kind == "source") | .after.category] | sort) ==
        ["assertions", "infrastructure", "test-support", "test-support"]
      and ([.changes[] | select(.before.purpose != .after.purpose) | .after.kind] | sort) ==
        ["data", "documentation"]
    ' "$work/transitions.json" > /dev/null

    # Actual recaptures exercise unavailable and absent states without inventing report evidence.
    printf '{}\n' > "$work/project/broken.ipynb"
    rm "$work/project/bin/launch"
    local status=0
    (cd "$work" && locmetry --root project --config refined.toml --no-user-config --files --output after.json) > "$work/out" 2> "$work/err" || status=$?
    [[ $status == 2 ]]
    status=0
    (cd "$work" && bash transitions.sh) > "$work/out" 2> "$work/err" || status=$?
    [[ $status == 2 ]]
    jq -e '
      any(.changes[]; .path == "broken.ipynb" and .before.state == "not_recorded" and .after.state == "parse_failed" and .after.counts == null and .code_delta == null)
      and any(.changes[]; .path == "bin/launch" and .after.state == "not_recorded" and .after.counts == null and .code_delta == null)
    ' "$work/transitions.json" > /dev/null
    rm "$work/project/broken.ipynb"
    (cd "$work" && locmetry --root project --config refined.toml --no-user-config --output after.json)
    rm "$work/transitions.json"
    if (cd "$work" && bash transitions.sh) > "$work/out" 2> "$work/err"; then
        echo 'Configuration recipe accepted a report without file detail' >&2
        exit 1
    fi
    grep -Fq 'single-root full reports with --files' "$work/err"
    [[ ! -s "$work/transitions.json" ]]

    # Synthetic historical evidence avoids relying on platform-specific unreadable-file permissions.
    jq '.metadata.backend_inaccurate = true
        | .metadata.coverage.unread = ["unread.py"]
        | .metadata.coverage.selected_files += 1' "$work/classified.json" > "$work/unread.json"
    for direction in forward reverse; do
        local before="$work/classified.json" after="$work/unread.json"
        if [[ $direction == reverse ]]; then
            before="$work/unread.json"; after="$work/classified.json"
        fi
        status=0
        bash "$helper" "$before" "$after" > "$work/unread-transition.json" 2> "$work/err" || status=$?
        [[ $status == 2 ]]
        jq -e --arg direction "$direction" '
          .changes | length == 1 and (.[0] |
            .path == "unread.py" and .code_delta == null
            and .before.counts == null and .after.counts == null
            and (if $direction == "forward"
                 then .before.state == "not_recorded" and .after.state == "unread"
                 else .before.state == "unread" and .after.state == "not_recorded" end))
        ' "$work/unread-transition.json" > /dev/null
    done
    jq 'del(.metadata.coverage.parse_failures)' "$work/classified.json" > "$work/historical.json"
    bash "$helper" "$work/historical.json" "$work/classified.json" > "$work/historical-transition.json"
    jq -e '(.before.metadata.coverage | has("parse_failures") | not) and .changes == []' "$work/historical-transition.json" > /dev/null
    printf 'not json\n' > "$work/invalid.json"
    status=0
    bash "$helper" "$work/invalid.json" "$work/classified.json" > "$work/out" 2> "$work/err" || status=$?
    [[ $status == 1 && ! -s "$work/out" ]]
    status=0
    bash "$helper" "$work/classified.json" > "$work/out" 2> "$work/err" || status=$?
    [[ $status == 1 && ! -s "$work/out" ]]
    grep -Fq 'BEFORE.json AFTER.json' "$work/err"
    (cd "$work" && locmetry workspace --root project --no-user-config --files --output workspace.json)
    status=0
    bash "$helper" "$work/workspace.json" "$work/workspace.json" > "$work/out" 2> "$work/err" || status=$?
    [[ $status == 1 && ! -s "$work/out" ]]
    grep -Fq 'single-root full reports with --files' "$work/err"
}
check_configuration_transitions

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

echo 'README and guide examples pass: silent file exports, Markdown content, policy outcomes, configuration transitions, compare, workspace compare, explain, partial coverage, troubleshooting queries, and adaptive folder detail.'
