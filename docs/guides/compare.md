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

## Inspect configuration transitions

Use the maintained [file-transition helper](../../scripts/compare-file-transitions.sh) when the question is which files changed classification or became countable. It handles two full single-root captures with `--files`; native `compare` continues to provide aggregate comparison without file detail. Preserve the intended personal defaults and selection settings for both captures. Save reports outside the measured root, using fresh output names:

```sh
locmetry --root service --files --output before.json --print table --verbose
# After reviewing and editing the project's configuration:
locmetry --root service --files --output after.json --print table --verbose
```

The helper is optional development tooling supplied in the Locmetry source checkout. Release archives and Homebrew installations contain no helper and need no `jq`; native measurement and aggregate comparison work independently. Running the helper requires Bash, `jq` and `locmetry` on `PATH`; set `LOCMETRY_BIN` to an executable path to select a local build. Run it from the checkout as below, or copy the script elsewhere and pass any two report paths. Quote paths containing spaces. No script edits are needed:

```sh
bash scripts/compare-file-transitions.sh before.json after.json > transitions.json
```

The helper validates the captures through native `compare` and emits JSON containing `changes`, both captures' complete `metadata`, their input paths, and native comparison `findings`. It reads saved reports without scanning source directories or loading current policy. Exit 0 means complete inputs; exit 2 retains usable JSON for partial inputs; exit 1 means invalid input or execution failure and produces no JSON. Preserve the exit status before inspecting the output. Use a fresh output filename: shell redirection can truncate an input before the helper starts. Keep both original captures.

Rows match exact root-relative paths and retain language, content kind, purpose, category, deciding rule, mapping provenance and counts when available. Classification-only changes remain visible with zero code delta. For a previously unrecognized file, the new count is available but the old count and numeric delta remain `null`. `not_recorded` means absent from counted and coverage records, not proven absent on disk; selection changes can also produce it. Missing historical parse-failure evidence stays missing in metadata. The helper retains the checked recipe’s state model; its projection is not a new Locmetry report schema. Native `compare` remains the route for aggregate totals and workspace comparisons.

Read `findings` alongside the changed rows. Different recorded policies establish a policy difference, not its causal share of the observed changes. Equal counts do not establish equal source contents. For a controlled policy experiment, independently verify unchanged input paths and bytes, including build descriptors and ignore files, and keep the same tool and selection settings. Store that verification separately; the helper always says source identity is not established by the reports themselves. Current `explain` output describes the current tree and policy, not either historical capture.

The [executable recipe checks](../../scripts/check-doc-examples.sh) use a synthetic tree and three policies, with policy files and reports outside the measured root. A recursive byte comparison against a saved copy independently checks unchanged input paths and contents after each capture. The checks establish these outcomes:

| Policy change | Expected observation |
| --- | --- |
| Classify three QA files as tests | Six source code lines transfer from production to test without changing those files' counts. |
| Map an extensionless launcher to JavaScript | Its state changes from unrecognized to counted production, adding one available source code line; its baseline count is not zero. |
| Refine helper, infrastructure and assertion categories | Category and deciding-rule changes remain visible; source purpose totals stay identical. |
| Restrict the archive override to `kind = ["source"]` | One source code line stays intentional `other`; documentation and data change purpose while remaining counted. |

Production starts at six source code lines and test at one; after the first two changes, production is one and test is seven. Intentional Other stays at one throughout. Those bucket changes reconcile with one additional counted source code line. The category/archive refinement then changes no source purpose totals. These small counts are fixture expectations, not defaults or claims about another project.
