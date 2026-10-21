# File-Size Gate Cookbook

Locmetry can gate source file sizes in CI. Generic line counters see paths and lines; Locmetry also records what each file is for, so a gate can hold production code to a tighter limit than tests and leave generated code alone, without maintaining path patterns. Locmetry itself enforces no limit: its exit status reports the measurement, and these scripts apply the limits to its per-file JSON.

The recipes need `jq` and a full report with per-file records. Each record carries `path`, `kind`, `language`, `purpose`, `category` and `counts`; the [report contract](../PRODUCT.md#report-content) owns those fields.

## Limit file sizes by purpose

This script fails on every production file over 1,000 code lines and every test file over 2,000, listing them on stderr. It captures the measurement first, so a failed or partial measurement keeps Locmetry's exit status instead of passing as "no oversized files":

```sh
set -eu
report=$(locmetry --files --output -) || exit "$?"
printf '%s\n' "$report" | jq -e '
  {production: 1000, test: 2000} as $limit |
  [.files[] | select(.kind == "source" and $limit[.purpose] != null and .counts.code > $limit[.purpose])
   | "\(.path): \(.counts.code) code lines over the \(.purpose) limit of \($limit[.purpose])"] |
  if length == 0 then true else (join("\n") + "\n" | halt_error(1)) end'
```

A purpose without a limit is not checked. Other holds generated and unclaimed code, so a generated client never fails the gate; name `other` in `$limit` to check it too.

## Warn before failing and freeze existing large files

A soft limit warns while a split is still cheap. A ceiling lets a file that predates the limits keep its current size, and fails as soon as it grows. Keep both in a policy file beside the script, for example `file-size-limits.json`:

```json
{
  "limits": {
    "production": {"soft": 600, "hard": 1000},
    "test": {"soft": 1000, "hard": 2000}
  },
  "ceilings": {"src/legacy.rs": 2400}
}
```

The script prints warnings on stdout and fails with the errors on stderr:

```sh
set -eu
report=$(locmetry --files --output -) || exit "$?"
printf '%s\n' "$report" | jq -r --slurpfile policy file-size-limits.json '
  $policy[0] as $p |
  [.files[] | select(.kind == "source" and $p.limits[.purpose] != null)
   | $p.limits[.purpose] as $l | ($p.ceilings[.path] // $l.hard) as $max |
   if .counts.code > $max then {error: "\(.path): \(.counts.code) code lines over \($max)"}
   elif .counts.code > $l.soft and $p.ceilings[.path] == null
   then {warning: "\(.path): \(.counts.code) code lines over the \(.purpose) soft limit of \($l.soft)"}
   else empty end] as $findings |
  ($findings[] | .warning // empty),
  ([$findings[] | .error // empty] | if length == 0 then empty else join("\n") + "\n" | halt_error(1) end)'
```

Lower a ceiling in the same change that shrinks its file, and remove it once the file fits the hard limit; otherwise the file can grow back unnoticed. [Locmetry's own gate](../self-measurement.md#file-sizes) also reports ceilings that can be lowered or that name files no longer measured.

## Choose what to count

- **Code lines.** `counts.code` excludes comments and blank lines, so explanatory comments never count against a file. Use `counts.lines` to limit physical length instead.
- **Purpose, not paths.** The project's [classification policy](../guides/configure.md) decides the purpose, so a file moved into a test folder or claimed by a rule changes its limit without editing the gate.
- **Embedded tests.** Inline unit tests count with the file that contains them. Rust unit tests in `src/` therefore count as production, and so does a `tests.rs` submodule there, because Cargo module inference claims every file under `src/`; add a project rule if such files should be tests.
- **Languages.** Select `.language` to limit one language. Per-file records keep the backend's language names, so TSX files are `TSX`, not `TypeScript` as in folded tables.
- **Workspaces.** `locmetry workspace` records paths prefixed by member name, so the same scripts gate several repositories at once.

## Locmetry's own gate

Locmetry gates its repository with a Python version of the second recipe that also reports stale ceilings; [self-measurement](../self-measurement.md#file-sizes) records its limits and ceiling policy.

[CI executes both recipes](../../scripts/check-doc-examples.sh) against synthetic projects: within limits, over a limit, a larger test file under the test limit, a soft-limit warning, a ceiling that holds and one that is exceeded, and a partial measurement.
