#!/usr/bin/env bash
set -euo pipefail

usage() {
    cat <<'HELP'
Usage: bash compare-file-transitions.sh BEFORE.json AFTER.json

Compare two full single-root Locmetry reports captured with --files.
Writes changed file records, both captures' metadata and native comparison
findings as JSON to stdout. No source directories are scanned.

Requires Bash, jq and locmetry on PATH; LOCMETRY_BIN can select an executable.
Exit 0: complete inputs; 2: partial inputs with usable JSON; 1: invalid input
or execution failure. Use a fresh output path when redirecting stdout.
HELP
}

if [[ $# == 1 && $1 == --help ]]; then
    usage
    exit 0
fi
if [[ $# != 2 ]]; then
    usage >&2
    exit 1
fi
if ! command -v jq >/dev/null; then
    printf '%s\n' 'This helper requires jq.' >&2
    exit 1
fi

before=$1
after=$2
binary=${LOCMETRY_BIN:-locmetry}
temporary=$(mktemp -d "${TMPDIR:-/tmp}/locmetry-transitions.XXXXXXXX")
trap 'rm -rf "$temporary"' EXIT

status=0
"$binary" compare --format json -- "$before" "$after" > "$temporary/comparison.json" || status=$?
case "$status" in 0|2) ;; *) exit 1 ;; esac

# Native comparison validates report semantics; temporary output keeps failed projections off stdout.
jq -n --slurpfile before "$before" --slurpfile after "$after" \
    --slurpfile comparison "$temporary/comparison.json" \
    --arg before_path "$before" --arg after_path "$after" '
  def eligible:
    .detail == "full" and (.metadata.root | type) == "string"
    and (.files | type) == "array";
  def records:
    [
      (.files[] | {key: .path, value: (del(.path) + {state: "counted"})}),
      (.metadata.coverage.unrecognized[] |
        {key: ., value: {state: "unrecognized", counts: null}}),
      (.metadata.coverage.unread[] |
        {key: ., value: {state: "unread", counts: null}}),
      ((.metadata.coverage.parse_failures // [])[] |
        {key: .path, value: {state: "parse_failed", counts: null, reason: .reason}})
    ] | from_entries;
  if ($before[0] | eligible) and ($after[0] | eligible) then
    ($before[0] | records) as $b | ($after[0] | records) as $a |
    {
      before: {input: $before_path, metadata: $before[0].metadata},
      after: {input: $after_path, metadata: $after[0].metadata},
      source_identity: "not established by reports",
      findings: ($comparison[0].findings // []),
      changes: [
        (($b + $a) | keys[]) as $path |
        ($b[$path] // {state: "not_recorded", counts: null}) as $old |
        ($a[$path] // {state: "not_recorded", counts: null}) as $new |
        select($old != $new) |
        {path: $path, before: $old, after: $new,
         code_delta: (if $old.state == "counted" and $new.state == "counted"
                      then $new.counts.code - $old.counts.code else null end)}
      ]
    }
  else error("Use single-root full reports with --files on both sides")
  end
' > "$temporary/transitions.json" || exit 1
cat "$temporary/transitions.json"
exit "$status"
