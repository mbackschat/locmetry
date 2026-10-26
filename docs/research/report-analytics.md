# Report Analytics Assessment

Evaluated 2026-10-20 using synthetic Locmetry reports, DuckDB 1.5.6, Python 3.11.14, pandas 3.0.6, Polars 2.0.0 and Showboat 0.6.1. The initial spike used measurement schema 2; the published recipes and their executable checks use schema 3. Installed tools, raw reports and benchmark scratch data are local-only. This published assessment contains no private project data.

Update 2026-10-21: the owner removed the cookbooks, their loaders and the demo; links below address their last version, and the [deferred cookbook design](../plans/rejected/analytics-cookbooks.md) describes their recreation.

## Recommendation

Keep analytics outside Locmetry. Provide a [DuckDB cookbook](https://github.com/mbackschat/locmetry/blob/8272abd21e21eb0a373c5f3ff77aa8f2d93c56b8/docs/duckdb-cookbook.md) for multi-report SQL and a smaller [pandas/Polars cookbook](https://github.com/mbackschat/locmetry/blob/8272abd21e21eb0a373c5f3ff77aa8f2d93c56b8/docs/dataframe-cookbook.md) demonstrating that the report contract is tool-neutral. Use ordinary JSON exports, caller-managed snapshot identity and explicit types. No database, Python runtime or analytics dependency belongs in the executable.

| Tool | Useful fit | Cost or limitation observed in this spike |
| --- | --- | --- |
| DuckDB | SQL grouping, portfolio joins, recorded trends, ownership and Parquet export across reports | Nested arrays need explicit types, especially when empty; JSON import still needs scope and identity decisions. |
| pandas | Python users doing custom transformations and notebook analysis | Build explicit integer/string/boolean columns; inference around empty or missing data can change types. |
| Polars | Typed dataframe workflows and a path to lazy Parquet analysis | Normalizing JSON through Python still incurs loading/materialization work; using Polars does not remove that step automatically. |
| Python standard library | A few totals or a narrow automation script | More hand-written joins, grouping and missing-evidence handling as analysis grows. |
| SQLite | SQL within an existing SQLite workflow | The prototype needed more explicit ingestion/flattening than DuckDB; useful when SQLite is already present. |
| jq | Inspecting one report, filtering rows or enforcing a small CI policy | Multi-snapshot identity, joins and trends become harder to maintain; retain it for focused queries. |

This is a workflow recommendation, not a general engine ranking. DuckDB exposes [JSON readers](https://duckdb.org/docs/current/data/json/loading_json) and [nested JSON transformations](https://duckdb.org/docs/current/data/json/json_functions); pandas offers [JSON normalization](https://pandas.pydata.org/docs/reference/api/pandas.json_normalize.html), and Polars provides [typed frame construction](https://docs.pola.rs/api/python/stable/reference/api/polars.from_dicts.html). [SQLite JSON functions](https://www.sqlite.org/json1.html) and the [Python JSON module](https://docs.python.org/3/library/json.html) support simpler alternatives without requiring a Locmetry-specific integration.

## Evidence and limits

The initial correctness corpus contained eight actual synthetic exports covering before/after root measurements, a workspace, empty input, partial counting and unavailable-only input. DuckDB, pandas and Polars agreed on independently specified aggregates; standard-library Python and SQLite also reproduced the aggregate totals. Exercises included composition, ownership, ordered changes and optional per-file statistics.

A bounded timing probe repeated one report into 200 documents: 20,000 breakdown rows and 7,872,800 JSON bytes. DuckDB JSON read plus grouping took roughly 38–43 ms; Python JSON loading, normalization and the first grouping took about 122 ms with pandas and 107 ms with Polars. Different import/process boundaries, repeated content, caching and a single-machine trial prevent a fair general speed ranking. These measurements establish feasibility for this small workload; they do not establish large-portfolio memory or throughput limits.

The maintained [cookbook assertions](https://github.com/mbackschat/locmetry/blob/8272abd21e21eb0a373c5f3ff77aa8f2d93c56b8/scripts/test_analytics_cookbooks.py) execute the published snippets against fresh synthetic captures. They check independent totals, ordered changes, folder-boundary ownership matching, empty snapshots, absent versus empty file detail, partial observations, malformed envelopes, duplicate manifest identity and embedded-language contributions. Dataframe results are compared with SQL, and aggregate sums are reconciled with snapshot summaries. This is a stronger correctness basis than keeping a timing result as a permanent expectation.

## Contract findings

The existing additive `breakdown` rows already fit SQL and dataframe operations. A flattening layer needs two tables: a snapshot inventory and its contributions. Flattening only contributions loses empty observations. Language participation counts can overlap, while embedded-language breakdown rows deliberately add code with zero additional physical files. Physical-file statistics require optional `files` detail; missing detail must not become a zero-sized project.

Schema 3 adds required `document_type: "measurement"` and `detail: "full" | "headline"`. Together with `schema_version`, these distinguish measurements from comparisons/explanations and full evidence from compact projections before engine-specific loading. The JSON schema and semantic validator enforce agreement with the projection. This benefits pandas and Polars as directly as DuckDB; no field names or types are database-specific. The [report contract](../PRODUCT.md#report-content) owns the detailed semantics and [compatibility guide](../report-compatibility.md) owns recapture requirements.

Keep repository identity, snapshot ID and sequence in a caller-owned manifest. Recorded paths are provenance, display names are not stable identity, and timestamps can tie. Separate scopes and avoid counting both a workspace and its individual members in one portfolio. Partial measurements cannot establish complete growth; a missing intermediate observation must not silently become a comparable previous state. Loader checks reject the wrong envelope and missing required analysis sections but do not replace complete schema/arithmetic validation of untrusted documents.

## Showboat assessment

[Simon Willison's introduction](https://simonwillison.net/2026/feb/10/showboat-and-rodney/) describes executable demonstrations: record commands and real output, then replay them. This suits both the cookbook and the stable CLI surface. A local trial verified a CLI export and DuckDB query; changing the query caused verification to fail, confirming that replay detects output drift.

The resulting [analytics demo](https://github.com/mbackschat/locmetry/blob/8272abd21e21eb0a373c5f3ff77aa8f2d93c56b8/docs/analytics-demo.md) and four [command guides](../cli/README.md) use Showboat directly, rather than a code-exploration walkthrough workflow. They share the real synthetic workspace with the README and tour. Commands explicitly omit variable timestamps/absolute paths from their displayed projections; saved reports retain those fields. Fresh temporary copies make examples independent, and a separate option-coverage guard detects newly added long options. Verification runs locally after relevant changes and in Linux documentation CI. Expected-output assertions remain separate: an unchanged replay alone is not an independent oracle for correctness.

Showboat is an optional pinned documentation tool, installed through uv outside the repository. Its output is ordinary Markdown. No hosted service, generated image, vendored dependency or committed environment is required.
