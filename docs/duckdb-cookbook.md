# Analyze Locmetry Reports with DuckDB

*2026-10-21T05:14:56Z by Showboat 0.6.1*
<!-- showboat-id: 34e5cb54-8d78-42a8-94c0-f4af5e8d8aa5 -->

Locmetry saves each measurement as a JSON report. One report answers questions about one moment: how much code, of which purpose, in which language and folder. Many useful questions need several reports side by side, or a join with your own data: How did the code change between two releases? Which component grew? How much code does each team own? Which new files are the largest? This cookbook answers these questions with SQL in [DuckDB](https://duckdb.org/), a database that runs as one program on your machine and reads JSON directly.

The examples measure [Immich](https://github.com/immich-app/immich), a popular self-hosted photo platform and Locmetry's [showcase project](showcase/README.md), at its two pinned releases `v3.2.0` and `v3.3.4`, with Locmetry's [reviewed policy](../examples/showcase/immich/README.md). Every output below is real, recorded with [Showboat](showboat-authoring.md) and replayed in CI. Each section states a question, shows the query and its result, and then explains how the query works and what the numbers do and do not mean. The [pandas and Polars cookbook](dataframe-cookbook.md) answers the same questions with Python dataframes.

## Before you start

You need the current `locmetry` and the [DuckDB command-line program](https://duckdb.org/docs/current/installation/), verified with version 1.5.6; on macOS run `brew install duckdb`, elsewhere `uv tool install duckdb-cli==1.5.6` also works. Locmetry itself has no database or Python dependency.

To run the examples yourself, build Locmetry with `cargo build --locked` and fetch the pinned checkouts once with `bash scripts/fetch-showcase.sh`, then run a block from the repository root. Every block starts in a fresh temporary directory, so it runs on its own: the first two sections show each step, and later blocks repeat those steps with `source scripts/analytics-env.sh duckdb`. For your own project, capture and load once and keep the database file.

## Capture the reports

SQL analysis needs *full* reports, which `--output FILE` writes; the compact headline printed to a terminal omits the per-folder breakdown. `--files` adds one record per file, which the file questions below need. Write the reports outside the measured folder, so a report is never measured itself.

```bash
source scripts/showcase-env.sh
for tag in v3.2.0 v3.3.4; do
  locmetry --root "$tag/immich" --files --output "$tag.json"
done
cat > snapshots.csv <<'CSV'
snapshot_id,repo_id,sequence,path
v3.2.0,immich,1,v3.2.0.json
v3.3.4,immich,2,v3.3.4.json
CSV
for report in v3.2.0.json v3.3.4.json; do
  jq -c '{project, detail, source_code: .summary.source.code, files: (.files | length)}' "$report"
done
```

```output
{"project":"immich","detail":"full","source_code":273566,"files":3017}
{"project":"immich","detail":"full","source_code":282770,"files":3035}
```

Each report is a complete measurement of one release. The `jq` line peeks at both: the same project, full detail, the source code total and the number of per-file records.

`snapshots.csv` is the *manifest*: it tells the analysis which report is which, because a report alone cannot say that. `snapshot_id` is your label for one capture. `repo_id` is your name for the repository; a report's display name and its absolute root are not reliable identities, since checkouts move and folders get renamed. `sequence` orders the captures of one repository, even when measurement times tie. Each path names exactly one report. Keep whole-project and `--subfolder` captures in separate series, and do not mix a workspace report with standalone reports of its members, or the same code would count twice.

If your own numbers differ from recorded ones, personal defaults in `~/.config/locmetry/config.toml` may be the reason; `--no-user-config` measures without them.

## Load them into a database

The [loader script](../examples/analytics/duckdb.sql) reads the manifest and its reports into the database file `immich.duckdb`. It first checks that each path holds exactly one full schema-3 measurement and stops with an error otherwise. The second command asks the new `snapshots` view for one row per capture.

```bash
source scripts/analytics-env.sh
duckdb immich.duckdb < duckdb.sql
duckdb immich.duckdb -c '.thousand_sep ,' -c '
  SELECT snapshot_id, recency, source_code, production_code, test_code, partial, has_files
  FROM snapshots ORDER BY repo_id, sequence'
```

```output
┌──────────────────────────────┐
│       inventory_check        │
│           varchar            │
├──────────────────────────────┤
│ One report per manifest path │
└──────────────────────────────┘
┌──────────────────────────┐
│       input_check        │
│         varchar          │
├──────────────────────────┤
│ Full measurements loaded │
└──────────────────────────┘
┌─────────────┬─────────┬─────────────┬─────────────────┬───────────┬─────────┬───────────┐
│ snapshot_id │ recency │ source_code │ production_code │ test_code │ partial │ has_files │
│   varchar   │ uint64  │   uint64    │     uint64      │  uint64   │ boolean │  boolean  │
├─────────────┼─────────┼─────────────┼─────────────────┼───────────┼─────────┼───────────┤
│ v3.2.0      │       2 │     273,566 │         177,605 │    81,051 │ false   │ true      │
│ v3.3.4      │       1 │     282,770 │         179,704 │    86,901 │ false   │ true      │
└─────────────┴─────────┴─────────────┴─────────────────┴───────────┴─────────┴───────────┘
```

The two check tables confirm the inputs. `.thousand_sep ,` is a display setting of the DuckDB program that groups digits; the queries below set it too. The loader copies the reports into the database, so later changes to the JSON files do not silently change an analysis. Run it again to refresh. It creates three views, which every query in this cookbook builds on:

| View | One row is | Columns you will use |
| --- | --- | --- |
| `snapshots` | One capture, even when it measured nothing | `snapshot_id`, `repo_id`, `sequence`, `recency`, `source_code`, `production_code`, `test_code`, `partial`, `has_files`, the raw report as `doc` |
| `contributions` | One row of a report's breakdown: the counts of one folder, kind, purpose, category and language | Snapshot identity, `folder`, `kind`, `purpose`, `category`, `language`, `files`, `code`, `comments`, `blanks`, `lines` |
| `physical_files` | One measured file, when the report has `--files` | Snapshot identity, `path`, `kind`, `purpose`, `language`, `code` |

`recency` numbers each repository's captures from the newest: 1 is the latest, 2 the one before. Queries can therefore ask for "the latest release" without naming it. `partial` marks a report whose counts are incomplete, for example after a file failed to parse; `has_files` tells whether per-file records exist. The column types come from the [report contract](PRODUCT.md#report-content): counts are unsigned integers, even when a report has no rows to infer types from. DuckDB's [JSON reader](https://duckdb.org/docs/current/data/json/loading_json) accepts JSON documents up to 16 MiB by default; an Immich report with file records has about 1 MB. For larger reports, raise `maximum_object_size` in the loader's `read_json_objects` call.

## What is Immich made of?

A first look at an unfamiliar project: which languages carry its code, and how much of it is production and how much is test?

<!-- composition -->

```bash
source scripts/analytics-env.sh duckdb
duckdb immich.duckdb <<'SQL'
.thousand_sep ,
SELECT language, purpose, sum(code) AS code, sum(files) AS files
FROM contributions
WHERE recency = 1 AND kind = 'source'
GROUP BY language, purpose
ORDER BY code DESC, language, purpose
LIMIT 10;
SQL
```

```output
┌────────────┬────────────┬────────┬────────┐
│  language  │  purpose   │  code  │ files  │
│  varchar   │  varchar   │ int128 │ int128 │
├────────────┼────────────┼────────┼────────┤
│ TypeScript │ production │ 70,455 │    711 │
│ TypeScript │ test       │ 68,506 │    362 │
│ Dart       │ production │ 59,495 │    669 │
│ JavaScript │ production │ 19,684 │     15 │
│ Svelte     │ production │ 17,420 │    410 │
│ Dart       │ test       │ 16,231 │    154 │
│ SQL        │ other      │  9,699 │     37 │
│ TypeScript │ other      │  6,466 │      1 │
│ Kotlin     │ production │  3,810 │     35 │
│ Swift      │ production │  3,103 │     28 │
└────────────┴────────────┴────────┴────────┘
  10 rows                         4 columns
```

TypeScript dominates, almost evenly split between production and test, followed by the Dart code of the mobile app. The query keeps only `kind = 'source'`, because reports also count data and documentation, such as Immich's JSON schemas and translations, which are not program source. It sums breakdown rows of the latest capture and groups them by language and purpose.

The JavaScript row looks odd: almost 20,000 code lines in 15 files. Most of it is the `<script>` part of Svelte components, which Tokei, Locmetry's counting engine, reports as embedded JavaScript. Embedded code adds its lines to its own language, while the file counts once, under the Svelte row; so `files` here counts each physical file exactly once. The [report guide](guides/reports.md#interpret-the-numbers) explains the difference from the summary's language file counts, which count every language a file contains.

## How much is generated code, tooling or test support?

Purpose answers *what code is for*; the category refines it. Immich's reviewed policy records categories such as `generated`, `end-to-end` and `tooling`, so the share of each is one query away.

<!-- categories -->

```bash
source scripts/analytics-env.sh duckdb
duckdb immich.duckdb <<'SQL'
.thousand_sep ,
SELECT purpose, category, sum(code) AS code,
       round(100 * sum(code) / sum(sum(code)) OVER (), 1) AS percent
FROM contributions
WHERE recency = 1 AND kind = 'source'
GROUP BY purpose, category
ORDER BY code DESC, purpose, category;
SQL
```

```output
┌────────────┬─────────────────────┬─────────┬─────────┐
│  purpose   │      category       │  code   │ percent │
│  varchar   │       varchar       │ int128  │ double  │
├────────────┼─────────────────────┼─────────┼─────────┤
│ production │ implementation      │ 174,215 │    61.6 │
│ test       │ test                │  83,485 │    29.5 │
│ other      │ generated           │  16,165 │     5.7 │
│ production │ native              │   3,156 │     1.1 │
│ test       │ end-to-end          │   3,118 │     1.1 │
│ production │ tooling             │   1,387 │     0.5 │
│ production │ deployment          │     798 │     0.3 │
│ test       │ test-infrastructure │     234 │     0.1 │
│ production │ build-tooling       │     148 │     0.1 │
│ test       │ performance-test    │      64 │     0.0 │
└────────────┴─────────────────────┴─────────┴─────────┘
  10 rows                                    4 columns
```

About 6% of Immich's source is generated: the SDK client and the SQL query snapshots its tooling writes. Counting it as production would overstate the hand-written product and depress the test ratio. `sum(sum(code)) OVER ()` is a window over the grouped result: it adds up all groups, so each row's percentage refers to the whole latest source. Categories without an entry, such as `implementation` from build modules or `test` from naming conventions, come from Locmetry's built-in rules; the [configuration guide](guides/configure.md) explains how a policy adds its own.

## Which components carry the code, and how well are they tested?

Immich keeps its components in top-level folders: `server`, `web`, `mobile`, `machine-learning` and more. Grouping by that first folder shows each component's size and its test-to-production ratio.

<!-- components -->

```bash
source scripts/analytics-env.sh duckdb
duckdb immich.duckdb <<'SQL'
.thousand_sep ,
SELECT repo_id, split_part(folder, '/', 1) AS component,
       sum(code) FILTER (purpose = 'production') AS production,
       sum(code) FILTER (purpose = 'test') AS test,
       round(test / nullif(production, 0), 2) AS test_prod
FROM contributions
WHERE recency = 1 AND kind = 'source'
GROUP BY ALL
ORDER BY sum(code) DESC, repo_id, component;
SQL
```

```output
┌─────────┬──────────────────┬────────────┬────────┬───────────┐
│ repo_id │    component     │ production │  test  │ test_prod │
│ varchar │     varchar      │   int128   │ int128 │  double   │
├─────────┼──────────────────┼────────────┼────────┼───────────┤
│ immich  │ server           │     53,296 │ 48,955 │      0.92 │
│ immich  │ mobile           │     66,989 │ 16,231 │      0.24 │
│ immich  │ web              │     52,867 │  5,388 │       0.1 │
│ immich  │ e2e              │        171 │ 13,003 │     76.04 │
│ immich  │ packages         │      2,345 │  1,254 │      0.53 │
│ immich  │ machine-learning │      3,221 │  2,070 │      0.64 │
│ immich  │ docs             │        401 │   NULL │      NULL │
│ immich  │ deployment       │        225 │   NULL │      NULL │
│ immich  │ .                │        111 │   NULL │      NULL │
│ immich  │ .devcontainer    │         58 │   NULL │      NULL │
│ immich  │ open-api         │         14 │   NULL │      NULL │
│ immich  │ misc             │          6 │   NULL │      NULL │
└─────────┴──────────────────┴────────────┴────────┴───────────┘
  12 rows                                            5 columns
```

The server has almost as much test code as production code, while the web client has one test line per ten production lines. `split_part(folder, '/', 1)` takes the first segment of a breakdown row's folder; files directly in the root have the folder `.`. `FILTER` sums only the rows of one purpose, and `nullif` turns a zero denominator into NULL, so a component without production code shows no ratio instead of failing.

Two cautions apply. The ratio divides summed test code by summed production code; never average the ratios of several rows, because small components would weigh as much as large ones. And it measures volume, not coverage or quality: `e2e` holds Immich's end-to-end suites, so 13,003 test lines against 171 production lines are expected.

Breakdown rows hold the files *directly* in their folder. To select a whole subtree, match the folder and everything below it, `folder = 'server' OR starts_with(folder, 'server/')`; without the slash, the prefix test would also select a sibling such as `server-old`.

## How much did the code grow between releases?

A series of captures shows a trend. This query lists each capture with its source code and the change from the capture before.

<!-- trend -->

```bash
source scripts/analytics-env.sh duckdb
duckdb immich.duckdb <<'SQL'
.thousand_sep ,
WITH ordered AS (
    SELECT *, lag(source_code::HUGEINT) OVER series AS previous_code,
              lag(partial) OVER series AS previous_partial
    FROM snapshots
    WINDOW series AS (PARTITION BY repo_id ORDER BY sequence)
)
SELECT repo_id, snapshot_id, source_code,
       CASE WHEN NOT partial AND NOT previous_partial
            THEN source_code::HUGEINT - previous_code END AS change
FROM ordered
ORDER BY repo_id, sequence;
SQL
```

```output
┌─────────┬─────────────┬─────────────┬────────┐
│ repo_id │ snapshot_id │ source_code │ change │
│ varchar │   varchar   │   uint64    │ int128 │
├─────────┼─────────────┼─────────────┼────────┤
│ immich  │ v3.2.0      │     273,566 │   NULL │
│ immich  │ v3.3.4      │     282,770 │  9,204 │
└─────────┴─────────────┴─────────────┴────────┘
```

Immich grew by 9,204 source code lines from `v3.2.0` to `v3.3.4`. The *window* `series` walks each repository's captures in sequence order, and `lag` reads the value of the previous capture; the first capture has none, so its change is NULL. Counts are unsigned, so the query converts them to the signed `HUGEINT` before subtracting: a shrinking project yields a negative change instead of an error. When either capture is partial, the change is NULL rather than a misleading jump; the query never bridges a partial capture to reach an older one.

Locmetry's native comparison reports the same totals:

```bash
source scripts/analytics-env.sh
locmetry compare v3.2.0.json v3.3.4.json --format json \
  | jq -c '.summary.source | {before: .before.code, after: .after.code}'
```

```output
{"before":273566,"after":282770}
```

Use [`locmetry compare`](guides/compare.md) to check that two captures are comparable at all: it reports differing policies, Locmetry versions, scope and coverage as findings. SQL computes differences between recorded measurements; it cannot tell whether a change came from new code or from a changed policy that moved code between purposes. And a net change is not churn: rewriting 1,000 lines and deleting 1,000 others shows as zero.

## Where did the change happen?

The total hides where the work went. Splitting the change by component and purpose shows which parts of Immich grew or shrank.

<!-- changes -->

```bash
source scripts/analytics-env.sh duckdb
duckdb immich.duckdb <<'SQL'
.thousand_sep ,
WITH comparable AS (
    SELECT repo_id FROM snapshots
    WHERE recency <= 2
    GROUP BY repo_id
    HAVING count(*) = 2 AND NOT bool_or(partial)
)
SELECT repo_id, split_part(folder, '/', 1) AS component, purpose,
       sum(CASE recency WHEN 1 THEN code::HUGEINT ELSE -code::HUGEINT END) AS change
FROM contributions
WHERE kind = 'source' AND recency <= 2 AND repo_id IN (SELECT repo_id FROM comparable)
GROUP BY ALL
HAVING change <> 0
ORDER BY abs(change) DESC, repo_id, component, purpose
LIMIT 10;
SQL
```

```output
┌─────────┬──────────────────┬────────────┬────────┐
│ repo_id │    component     │  purpose   │ change │
│ varchar │     varchar      │  varchar   │ int128 │
├─────────┼──────────────────┼────────────┼────────┤
│ immich  │ server           │ test       │  3,111 │
│ immich  │ server           │ production │  1,866 │
│ immich  │ mobile           │ test       │  1,791 │
│ immich  │ mobile           │ production │  -1227 │
│ immich  │ server           │ other      │  1,082 │
│ immich  │ machine-learning │ test       │    648 │
│ immich  │ machine-learning │ production │    574 │
│ immich  │ web              │ production │    574 │
│ immich  │ packages         │ production │    309 │
│ immich  │ packages         │ test       │    195 │
└─────────┴──────────────────┴────────────┴────────┘
  10 rows                                4 columns
```

Most growth is server test code, and the mobile app gained tests while its production code shrank. The query compares each repository's latest capture with the one before it. `comparable` keeps only repositories that have both captures and no partial one. The signed sum then adds the latest capture's code and subtracts the previous one's in a single pass, so a component that exists in only one of the two captures still appears, with its whole size as the change. DuckDB 1.5.6 does not apply `.thousand_sep` to negative numbers, which is why the decrease reads `-1227`.

Third, the existing files whose size changed most. Together with new files, they show where the release's work went at file level.

<!-- changed-files -->

```bash
source scripts/analytics-env.sh duckdb
duckdb immich.duckdb <<'SQL'
.thousand_sep ,
SELECT n.path, o.code AS before, n.code AS after,
       n.code::HUGEINT - o.code::HUGEINT AS change
FROM physical_files n
JOIN physical_files o ON o.repo_id = n.repo_id AND o.path = n.path AND o.recency = 2
WHERE n.recency = 1 AND n.kind = 'source' AND n.code <> o.code
ORDER BY abs(change) DESC, n.path
LIMIT 5;
SQL
```

```output
┌─────────────────────────────────────────────────────────────────┬────────┬────────┬────────┐
│                              path                               │ before │ after  │ change │
│                             varchar                             │ uint64 │ uint64 │ int128 │
├─────────────────────────────────────────────────────────────────┼────────┼────────┼────────┤
│ server/src/queries/person.repository.sql                        │    565 │  1,344 │    779 │
│ machine-learning/test_main.py                                   │  1,212 │  1,828 │    616 │
│ server/test/medium/specs/repositories/person.repository.spec.ts │    191 │    657 │    466 │
│ server/test/medium/specs/services/person.service.spec.ts        │    889 │  1,340 │    451 │
│ server/test/medium/specs/services/memory.service.spec.ts        │    375 │    653 │    278 │
└─────────────────────────────────────────────────────────────────┴────────┴────────┴────────┘
```

The inner `JOIN` pairs each file with its version in the previous capture, so only files present in both appear. Large changes in generated files are expected; in hand-written files they point to refactorings worth a look. As with new files, a rename breaks the pairing.

## Who owns the code?

Ownership is not part of a report: it lives in your organization. Joining it from a small CSV file answers questions such as "how much code does each team maintain?". Each row maps a folder prefix to an owner. Nested prefixes are allowed and the longest match wins, so here `server/test` belongs to the quality team, while the rest of `server` belongs to the backend team.

<!-- ownership -->

```bash
source scripts/analytics-env.sh duckdb
cat > owners.csv <<'CSV'
repo_id,prefix,owner
immich,server,backend
immich,server/test,quality
immich,web,web-team
immich,mobile,mobile-team
immich,mobile/test,quality
immich,machine-learning,ml-team
immich,e2e,quality
CSV
duckdb immich.duckdb <<'SQL'
.thousand_sep ,
CREATE OR REPLACE TABLE ownership (
    repo_id VARCHAR, prefix VARCHAR, owner VARCHAR NOT NULL,
    PRIMARY KEY (repo_id, prefix)
);
INSERT INTO ownership SELECT * FROM read_csv('owners.csv', header = true, all_varchar = true);
CREATE OR REPLACE VIEW owned_contributions AS
SELECT c.*, coalesce(m.owner, 'unassigned') AS owner
FROM contributions c
LEFT JOIN LATERAL (
    SELECT owner FROM ownership o
    WHERE o.repo_id = c.repo_id
      AND (o.prefix = '.' OR c.folder = o.prefix OR starts_with(c.folder, o.prefix || '/'))
    ORDER BY length(o.prefix) DESC
    LIMIT 1
) m ON true;
SELECT owner,
       sum(code) FILTER (purpose = 'production') AS production,
       sum(code) FILTER (purpose = 'test') AS test
FROM owned_contributions
WHERE recency = 1 AND kind = 'source'
GROUP BY owner
ORDER BY sum(code) DESC, owner;
SQL
```

```output
┌─────────────┬────────────┬────────┐
│    owner    │ production │  test  │
│   varchar   │   int128   │ int128 │
├─────────────┼────────────┼────────┤
│ backend     │     53,296 │ 27,798 │
│ mobile-team │     66,989 │    835 │
│ web-team    │     52,867 │  5,388 │
│ quality     │        171 │ 49,556 │
│ unassigned  │      3,160 │  1,254 │
│ ml-team     │      3,221 │  2,070 │
└─────────────┴────────────┴────────┘
```

For every breakdown row, the `LATERAL` subquery looks up the owners whose prefix matches the row's folder and keeps the longest one, so each row counts for exactly one owner. A prefix matches its folder and everything below it, with the same slash rule as above; `.` would match every folder and is the way to name a default owner. Rows without any match stay visible as `unassigned`, here Immich's docs, deployment files and shared packages, instead of silently disappearing as they would in an inner join.

The backend and web teams own test code too: Immich keeps unit tests next to the code they test, such as `server/src/services/*.spec.ts`, so no folder prefix separates them. Folder ownership and purpose are independent dimensions. The mapping describes today's ownership; to attribute older captures to the owners of their time, add a capture-specific mapping.

## Which files are the largest, and which are new?

File records answer questions that folder totals cannot. First, the size distribution of the source files in each capture:

<!-- file-sizes -->

```bash
source scripts/analytics-env.sh duckdb
duckdb immich.duckdb <<'SQL'
.thousand_sep ,
SELECT s.snapshot_id,
       CASE WHEN s.has_files THEN count(f.path) END AS source_files,
       median(f.code) AS median_code, max(f.code) AS largest_code
FROM snapshots s
LEFT JOIN physical_files f ON f.snapshot_id = s.snapshot_id AND f.kind = 'source'
GROUP BY s.snapshot_id, s.has_files
ORDER BY s.snapshot_id;
SQL
```

```output
┌─────────────┬──────────────┬─────────────┬──────────────┐
│ snapshot_id │ source_files │ median_code │ largest_code │
│   varchar   │    int64     │   double    │    uint64    │
├─────────────┼──────────────┼─────────────┼──────────────┤
│ v3.2.0      │        2,517 │        54.0 │        6,293 │
│ v3.3.4      │        2,529 │        56.0 │        6,466 │
└─────────────┴──────────────┴─────────────┴──────────────┘
```

Half of Immich's source files have at most 56 code lines, and the largest has 6,466: the generated SDK client, which the reviewed policy classifies as `other`. The `LEFT JOIN` from `snapshots` keeps a capture without file records in the result, with NULL statistics: unknown, not zero. A capture of an empty folder instead shows 0 files.

Second, the largest source files that are new in the latest release:

<!-- new-files -->

```bash
source scripts/analytics-env.sh duckdb
duckdb immich.duckdb <<'SQL'
.thousand_sep ,
SELECT n.path, n.purpose, n.code
FROM physical_files n
WHERE n.recency = 1 AND n.kind = 'source'
  AND n.repo_id IN (SELECT repo_id FROM snapshots WHERE recency = 2 AND has_files)
  AND NOT EXISTS (
      SELECT 1 FROM physical_files o
      WHERE o.repo_id = n.repo_id AND o.recency = 2 AND o.path = n.path)
ORDER BY n.code DESC, n.path
LIMIT 5;
SQL
```

```output
┌──────────────────────────────────────────────────────────────────────────────────────────┬────────────┬────────┐
│                                           path                                           │  purpose   │  code  │
│                                         varchar                                          │  varchar   │ uint64 │
├──────────────────────────────────────────────────────────────────────────────────────────┼────────────┼────────┤
│ server/test/medium/specs/repositories/person-user.repository.spec.ts                     │ test       │    364 │
│ packages/scripts/src/commands/notes.ts                                                   │ production │    283 │
│ mobile/test/presentation/pages/slideshow_page_test.dart                                  │ test       │    275 │
│ web/src/lib/modals/PersonBulkShareModal.svelte                                           │ production │    200 │
│ mobile/lib/presentation/widgets/sliver_app_bar/random_asset_background_image.widget.dart │ production │    197 │
└──────────────────────────────────────────────────────────────────────────────────────────┴────────────┴────────┘
```

`NOT EXISTS` keeps a file of the latest capture only when the previous capture has no file at the same path. The condition on `has_files` matters: without file records in the previous capture, every file would look new. Paths are the only file identity in a report, so a renamed or moved file also appears as new.

## What did the reviewed policy change?

A policy edit reclassifies files without changing any count, so totals alone cannot show what it did. Measuring one release twice, with built-in rules only and with the reviewed policy, and joining the two file lists by path lists every move. This uses a separate analysis directory, because the two captures are alternative views of one release rather than a series.

```bash
source scripts/analytics-env.sh
mkdir policy && cd policy
printf 'version = 1\n' > builtin-only.toml
locmetry --root ../immich --config builtin-only.toml --files --output builtin.json
cp ../v3.3.4.json reviewed.json
cat > snapshots.csv <<'CSV'
snapshot_id,repo_id,sequence,path
builtin,builtin,1,builtin.json
reviewed,reviewed,1,reviewed.json
CSV
duckdb policy.duckdb < ../duckdb.sql > /dev/null
duckdb policy.duckdb <<'SQL'
.thousand_sep ,
SELECT b.purpose || ':' || b.category AS built_in, r.purpose || ':' || r.category AS reviewed,
       count(*) AS files, sum(r.code) AS code
FROM physical_files b
JOIN physical_files r ON r.path = b.path AND r.snapshot_id = 'reviewed'
WHERE b.snapshot_id = 'builtin' AND r.kind = 'source'
  AND (b.purpose, b.category) IS DISTINCT FROM (r.purpose, r.category)
GROUP BY ALL
ORDER BY code DESC, built_in, reviewed;
SQL
```

```output
┌───────────────────────────┬───────────────────────────┬───────┬────────┐
│         built_in          │         reviewed          │ files │  code  │
│          varchar          │          varchar          │ int64 │ int128 │
├───────────────────────────┼───────────────────────────┼───────┼────────┤
│ production:implementation │ other:generated           │    38 │ 16,165 │
│ other:unclaimed           │ production:native         │    32 │  3,156 │
│ production:implementation │ test:end-to-end           │    23 │  3,118 │
│ other:unclaimed           │ production:tooling        │    24 │  1,044 │
│ other:unclaimed           │ production:deployment     │    31 │    798 │
│ production:implementation │ production:tooling        │     1 │    343 │
│ other:unclaimed           │ production:implementation │    10 │    327 │
│ other:unclaimed           │ test:test-infrastructure  │     4 │    234 │
│ other:unclaimed           │ test:performance-test     │     1 │     64 │
└───────────────────────────┴───────────────────────────┴───────┴────────┘
```

A file containing only `version = 1` hides the checkout's `.locmetryrc`, so the first capture uses built-in rules alone. Each result row is one kind of decision: 38 files moved from module-inferred production to `generated`, the iOS app's previously unclaimed Swift code became production, and end-to-end support code joined the tests. Every row traces back to a policy entry with a recorded reason, which makes this table a compact review of a policy change. The [Immich policy tour](showcase/tour.md) builds that policy step by step.

## Share the dataset

Other tools, such as pandas, Polars or a spreadsheet, can read the normalized tables as [Parquet](https://parquet.apache.org/) files. Reading the export back is a quick check that nothing was lost.

<!-- parquet -->

```bash
source scripts/analytics-env.sh duckdb
duckdb immich.duckdb <<'SQL'
.thousand_sep ,
COPY (SELECT * FROM snapshots) TO 'snapshots.parquet' (FORMAT PARQUET);
COPY (SELECT * FROM contributions) TO 'contributions.parquet' (FORMAT PARQUET);
SELECT snapshot_id, sum(code) AS source_code
FROM 'contributions.parquet'
WHERE kind = 'source'
GROUP BY snapshot_id
ORDER BY snapshot_id;
SQL
```

```output
┌─────────────┬─────────────┐
│ snapshot_id │ source_code │
│   varchar   │   int128    │
├─────────────┼─────────────┤
│ v3.2.0      │     273,566 │
│ v3.3.4      │     282,770 │
└─────────────┴─────────────┘
```

The per-snapshot totals match the `source_code` column of `snapshots`. The export keeps empty snapshots and the raw reports in the `doc` column. Parquet files are derived data; keep the original reports and the manifest as well, because they are the evidence the analysis rests on. The [pandas and Polars cookbook](dataframe-cookbook.md) reads the reports directly instead, and the [dated tool assessment](research/report-analytics.md) explains why DuckDB, pandas and Polars are all suitable.

## Verification

These recordings replay with the other showcase recordings; the [showcase index](showcase/README.md#verify-and-regenerate) has the command. Two healthy Immich releases cannot exhibit every case the queries handle, so the [cookbook test](../scripts/test_analytics_cookbooks.py) runs the marked queries of this page on small synthetic reports as well: an empty project, a partial capture, a capture without file records, a sibling folder that shares a prefix, and malformed inputs the loader must reject. Run it with the pinned packages:

```sh
uv run --no-project --python 3.11 \
  --with duckdb==1.5.6 --with pandas==3.0.6 --with polars==2.0.0 \
  python scripts/test_analytics_cookbooks.py
```

Python packages and environments stay outside Git, and neither the Locmetry executable nor its Cargo tests need them.
