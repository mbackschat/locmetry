# Analytics Cookbooks

<!-- doc-status
{
  "status": "deferred",
  "date": "2026-10-21",
  "summary": "Removed DuckDB, pandas/Polars cookbooks and demo; recreate them refocused on questions native output cannot answer.",
  "evidence": "[Report analytics assessment](../research/report-analytics.md)"
}
-->

The owner removed the analytics cookbooks on 2026-10-21, after a re-assessment found that half of their questions are now answered by native tables and `compare`. This deferred design preserves them and describes their recreation with that re-assessment folded in. Nothing here is current behavior; the [report contract](../PRODUCT.md#report-content) owns what reports guarantee.

## Activation

The owner decides. Until then, no cookbook, loader, pinned analytics package or CI step is part of the repository.

## Recovery source

Commit [`8272abd`](https://github.com/mbackschat/locmetry/tree/8272abd21e21eb0a373c5f3ff77aa8f2d93c56b8) holds the last complete version, recorded on Immich `v3.2.0` and `v3.3.4` and passing CI. Restore a file with `git show 8272abd:PATH > PATH`, then edit it as described below. Recreated cookbooks belong in [`docs/cookbooks/`](../cookbooks/README.md), decided 2026-10-21 by the owner, not in `docs/` where they used to be.

| Path at `8272abd` | Role |
| --- | --- |
| `docs/duckdb-cookbook.md` | DuckDB cookbook, a Showboat recording |
| `docs/dataframe-cookbook.md` | pandas and Polars cookbook, a Showboat recording |
| `docs/analytics-demo.md` | Python script over workspace captures, a Showboat recording |
| `examples/analytics/duckdb.sql` | Loader: manifest and reports into the `snapshots`, `contributions` and `physical_files` views |
| `examples/analytics/report_rows.py` | Standard-library loader returning typed pandas or Polars frames |
| `scripts/analytics-env.sh` | Per-example setup: captures both releases with `--files`, writes the manifest, copies the loaders |
| `scripts/test_analytics_cookbooks.py` | Runs the marked examples on synthetic reports: empty project, partial capture, missing file records, sibling prefixes, malformed inputs |

Wiring removed with them, to restore in the same change: the cookbook entries in `SHOWCASE` of [check-recorded-guides.py](../../scripts/check-recorded-guides.py); the pins `duckdb==1.5.6`, `duckdb-cli==1.5.6`, `pandas==3.0.6` and `polars==2.0.0` in the showcase replay of [CI](../../.github/workflows/ci.yml), the [showcase index](../showcase/README.md#verify-and-regenerate) and the [command-guide index](../cli/README.md#verify-and-regenerate); CI's "Check analytics recipes" step; the `example` category, the `scripts/analytics-env.sh` entry and the `analytics-examples` rule in [.locmetryrc](../../.locmetryrc), with their row in [self-measurement design](../self-measurement.md); and the entry points in the README, the [documentation index](../README.md), the [showcase index](../showcase/README.md), the [report guide](../guides/reports.md#analyze-saved-measurements), the [Showboat guidelines](../showboat-authoring.md) and CLAUDE.md. The reference replay's `duckdb` pin in CI and [release.py](../../scripts/release.py) was unused by the command guides and is not restored.

## Design

The cookbooks answer questions that Locmetry's native output cannot: joins with external data, file-level change, groupings that combine two dimensions, and trends across more than two captures. Each opens by saying so: use them when the tables and `compare` run out. Pointing to the native equivalent replaces any section that only repeats it.

### Required: DuckDB cookbook

Keep the capture, manifest and load sections, and the views table. Then:

| Section at `8272abd` | Decision | Reason |
| --- | --- | --- |
| What is Immich made of? | Keep as the only warm-up; add one sentence naming the native language table | Introduces the views on a familiar result |
| How much is generated code, tooling or test support? | Remove | `--show-categories` shows the same table, with share bars |
| Which components carry the code? | Remove; move its subtree-prefix rule into the ownership section | A workspace or `--show-folders` shows it natively |
| How much did the code grow between releases? | Keep, presented as the template for a series of many captures | `compare` covers two captures |
| Where did the change happen? | Keep | `compare` grids show folder or purpose, not both |
| Changed files | Keep | `compare` has no per-file deltas |
| Who owns the code? | Keep | Joins data a report cannot hold |
| Largest and new files | Keep | Needs per-file records across captures |
| What did the reviewed policy change? | Keep; link the [file-transition helper](../../scripts/compare-locmetry-files.sh) for the per-file list | The grouped transition matrix is a review summary the helper does not give |
| Share the dataset | Keep | Parquet for other tools |
| Analytics demo | Fold in as a last section, "From Python, on workspace reports"; delete the page | Its new content is the DuckDB Python API and the rule never to mix workspace and member captures |

### Required: pandas and Polars cookbook

Reduce it to the typed load with its reconciliation check, then one question that native output cannot answer, preferably ownership or new files, in pandas and then in Polars with identical results. Remove the components, comment-density and duplicate change sections: components is native, and comment density is native as `Comment %` with `--table-purpose production --show-folders`.

### Required: entry points and verification

The README lists the cookbooks by the questions only they answer: ownership, new and changed files, trends across many captures. Recordings replay with the showcase set; the synthetic test covers every marked example that remains; the pins above return to CI and the replay instructions.

### Optional

A capture cache in `scripts/analytics-env.sh`: measure each release once per replay and give every example its own copy. On 2026-10-21 one example's setup took about 2.3 s with the debug executable, the DuckDB cookbook 32.5 s and pandas/Polars 12.3 s of a 65 s local replay; the cache would save about 35 s, and the removals above about 9 s. It shares state between examples, so it needs a guard keyed on the executable and pinned tag and an amendment to the [Showboat guidelines](../showboat-authoring.md#isolate-every-example).

### Excluded

Analytics inside the executable, a Locmetry-specific report format, and history analytics such as hotspots or authorship, which [conditional follow-ups](measurement-quality-and-net-change.md#conditional-follow-ups-and-preserved-rationales) keep separate.

## Acceptance

Every recorded output is real and replays in CI; the synthetic test passes with the pinned packages; no section's question is answered by a native table or `compare` alone; and the documentation index and link checks pass.
