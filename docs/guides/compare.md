# Compare Saved Measurements

Use `locmetry compare --help` for arguments and `locmetry --schema comparison` for structure and field meanings. Comparison reads two saved full measurements, without scanning source roots or loading their current configurations. It reports net change, **after minus before**, not Git additions/deletions, churn, effort or quality.

## A Small Example

In a fresh temporary working directory, create a one-line project and capture it. Then add a second line and capture again. Reports sit outside the measured root; personal defaults are disabled to isolate the example.

```sh
mkdir demo
printf 'print(1)\n' > demo/main.py
locmetry --root demo --no-user-config --output before.json
printf 'print(1)\nprint(2)\n' > demo/main.py
locmetry --root demo --no-user-config --output after.json
locmetry compare before.json after.json --format json
```

The source-code projection of `.summary.source` is:

<!-- comparison-result -->
```json
{"before": 1, "after": 2, "delta": 1}
```

The two exports produce no stdout. For terminal text, use `locmetry compare before.json after.json --format table`, including when piping output. Without an explicit format, a terminal gets a table and a pipe gets a compact headline. Argument order decides direction, not timestamps. Use `locmetry --guide reports` to choose content/destination or inspect baseline coverage.

## Comparable Scope and Evidence

Both inputs must be full schema-3 measurement reports with supported structure and the required provenance. Full JSON without `--files` is sufficient. A saved headline is rejected with a remedy, but rerunning measures the current tree and cannot recover a lost past baseline. A schema-3 document can omit optional historical failure evidence; that absence is reported, not treated as a recorded empty failure list.

Use two single-root reports or two workspace reports. Different `--subfolder` restrictions, including absent versus present, are rejected before output is written. The same restriction is not enough to prove identical selection: inspect policy, version, coverage and module findings too. Root locations may differ, and comparison uses recorded data even if the source folders no longer exist.

A positive delta can come from source edits, changed selection, reclassification or changed counting behavior. Identical settings are not proof of identical source. Zero total change can hide category transfers or folder/module arrivals and departures. Do not claim that presence changes prove creation/deletion on disk. Per-file deltas, rename detection and automatic Git revision capture are not implemented. Filenames such as `before-COMMIT.json` are caller labels, not verified Git provenance.

Independent measurements can observe different filesystem state. To keep a meaningful baseline pair, record the intended context and settings and avoid unrelated edits during capture. Locmetry does not check out revisions or freeze the working tree. Comparison output cannot overwrite either input, including symlink/hard-link aliases; shell redirection bypasses this protection.

## Compare saved measurements

The following recipes assume an existing `service` project, and for workspace examples a `repos.toml` file, in the current directory. Synthetic examples disable personal defaults; real captures should preserve the intended effective policy. The partial-inspection recipe uses optional external `jq`.

```sh
locmetry --root ./service --no-user-config --output ./before.json
# After changing the source or its measurement policy:
locmetry --root ./service --no-user-config --output ./after.json
locmetry compare before.json after.json
locmetry compare before.json after.json --format markdown --output net-change.md
locmetry compare before.json after.json --output net-change.json
locmetry --schema comparison
```

The compact comparison headline includes presence counts; full JSON includes every folder, package and breakdown row. Use `--format headline` for the compact projection or `--format json` for complete output.

The same command compares workspace exports, even when their root lists differ:

```sh
locmetry workspace --workspace-file repos.toml --no-user-config --output before-workspace.json
# After changing code or the workspace root list:
locmetry workspace --workspace-file repos.toml --no-user-config --output after-workspace.json
locmetry compare before-workspace.json after-workspace.json --format markdown
```

Workspace totals separate changes inside retained roots from counts entering and leaving the measurement. Names match members; renamed members appear as a departure and arrival. A folder or package disappearing from a report does not prove source deletion. Removing only a build descriptor can change purpose and package presence while source LOC stays unchanged.

Inspect Measurement notes and Findings in human output, or `findings` in JSON, before interpreting growth: versions, policy, module detection and coverage can differ. Observed input inaccuracy/read/parse failures make the comparison partial; missing historical failure evidence alone does not. See [command outcomes](../PRODUCT.md#command-outcomes). Recorded counts exclude unavailable lines, whose size remains unknown. [The contract](../PRODUCT.md#net-change) defines the output and limits; [report compatibility](../report-compatibility.md) explains schema versions and recapture requirements.

Table and Markdown detail grids show Lines and Code before, after and net change whenever either changes, plus member/folder arrivals and departures even at zero code. Comment-only and blank-only growth therefore appears when it changes source lines; other count changes remain in Totals and full JSON. Totals split All counted by content kind, so data and documentation changes appear beside source without a JSON query. Both human formats end with the before/after measurement dates in UTC, taken from the input metadata rather than file creation times. Each grid ends in a bold total row; a total covers rows the grid does not show, so only net changes add up from the visible rows. Both human formats summarize coverage and show specific producer/backend differences. “Same” describes recorded paths (and reasons for parse failures), not unchanged unavailable content. Full JSON retains individual findings, including per-folder coverage in `findings[].folder`; the following recipe retrieves them while preserving a partial comparison's status:

```sh
status=0
locmetry compare before.json after.json --output net-change.json || status=$?
case "$status" in
  0|2) jq '.findings[] | select(.code == "coverage_gap")' net-change.json ;;
  *) exit "$status" ;;
esac
exit "$status"
```


## Inspect one workspace member

After capturing `before-workspace.json` and `after-workspace.json` with the workspace recipe above, select one root by its exact, case-sensitive member name:

```sh
locmetry compare before-workspace.json after-workspace.json --member service --format table
locmetry compare before-workspace.json after-workspace.json --member service --output member-change.json
```

Check `Member: service` and its presence (`both`, `before only`, or `after only`), then the `Source total` and folder deltas. Kind, Purpose, Language, Test/prod and folder totals now cover only this member. The terminal folder table shows changed immediate subfolders with recursive counts, including unchanged descendants; deeper folders remain in Markdown and full JSON. Source totals include unchanged rows and files directly in the member root, so the visible subfolder rows need not sum to the total. Recursive parents and descendants must not be added together.

`--member` requires two full workspace reports and a name present in at least one. Unknown names and single-root reports fail without writing output. A member missing on one side contributes zero on that side because it was absent from the measurement. The selector cannot name an arbitrary nested folder, and it never rescans the source roots.

Full JSON and headline retain `member: "service"`. Summary dimensions, composition and presence counts cover that member; full folders, packages, breakdown and member-specific findings are filtered to it. JSON paths keep their member prefixes. Original input metadata and report-level findings remain whole-input provenance, and partial status still reflects either complete workspace input, including failures in other members. The human scope note names this boundary. Inspect `member`, `summary.source`, `folders["service"].recursive.source` and findings in `member-change.json` to verify scope, counts and limitations.
