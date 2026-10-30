# Tour: Build a Policy for Immich

*2026-10-21T03:19:50Z by Showboat 0.6.1*
<!-- showboat-id: 6e44268b-01d8-47f4-8396-719e682e3925 -->

This tour builds a Locmetry policy for [Immich](https://github.com/immich-app/immich) `v3.3.4` the way a maintainer or an agent would: measure with built-in conventions, review what they cannot know, record evidence-based entries and check what changed. Run each Bash block from the Locmetry repository root after `cargo build --locked` and `bash scripts/fetch-showcase.sh`; the [showcase index](README.md) explains the environment. `--config` with a file containing only `version = 1` hides the checkout's reviewed `.locmetryrc`, so the tour starts from built-in conventions alone.

## Review what built-in conventions decide

`--verbose` adds the review notes to the table. `sed` keeps only those notes, and `demo_wrap` wraps them.

```bash
source scripts/showcase-env.sh
printf 'version = 1\n' > builtin-only.toml
locmetry --root immich --config builtin-only.toml --format table --verbose --no-color --no-bars \
  | sed '/^Root:/d; /^Roots:/d; /^Project:/d; /^Measured at:/d' \
  | sed -n '/^ Not source code/,$p' \
  | demo_wrap
```

```output
 Not source code: 296 data files (245,954 lines), 210 documentation files (12,477 lines, including 749 code lines in
embedded snippets). '--files --output -' lists each file's kind.
 Production from build modules: 127,140 code lines in 9 'npm' modules, 59,170 code lines in 2 'dart' modules, 4,102 code
lines in 1 'gradle' module, 3,037 code lines in 1 'python' module.
 Largest files counted as Production only by a build layout. If one is generated or vendored, exclude or reclassify it
in '.locmetryrc':
   packages/sdk/src/fetch-client.ts                                6,466 code lines  packages/sdk (npm)
   server/src/queries/search.repository.sql                        1,856 code lines  server (npm)
   server/src/queries/person.repository.sql                        1,344 code lines  server (npm)
   server/src/schema/migrations/1752267649968-StandardizeNames.ts  1,193 code lines  server (npm)
   server/src/repositories/asset.repository.ts                     1,161 code lines  server (npm)
 Unclaimed source: 4,945 code lines in 89 files, here:
   mobile/ios/Runner           2,272 code lines  21 files
   mobile/ios/WidgetExtension    830 code lines   7 files
   mobile/pigeon                 325 code lines   9 files
   packages                      265 code lines   8 files
   machine-learning              248 code lines   4 files
   deployment                    225 code lines  16 files
   server                        218 code lines   6 files
   docs                          213 code lines   2 files
   and 10 more folders (349 code lines)
 To inspect or refine purpose classification, use '--guide configure'. '--files --output -' shows each file's rule.
 Unrecognized files: 436 (png: 165, webp: 146, no extension: 61, ttf: 19, lock: 10). Their lines are unknown, not zero.

```

Data and documentation are already separate, including the code examples inside the documentation, and four build systems claim most production source. A module layout makes everything in it production without review, so the note lists the largest files the layouts claimed: the SDK client and two SQL query snapshots dwarf the hand-written code around them. The unclaimed list is the other review queue: the iOS app has no build descriptor Locmetry reads, and smaller folders hold configuration and scripts.

## Look for generated code

Generators usually leave a header. Searching for common markers checks those suspects and finds the rest of their kind.

```bash
source scripts/showcase-env.sh
grep -rlE 'DO NOT MODIFY|auto generated|@generated' immich/ \
  --include='*.ts' --include='*.sql' --include='*.dart' --include='*.swift' --include='*.kt' \
  | sed 's#/[^/]*$##' | sort | uniq -c | awk '{ printf "%4d %s\n", $1, $2 }'
```

```output
   1 immich/packages/sdk/src
   1 immich/server/src/bin
  37 immich/server/src/queries
```

The SDK client and the 37 query snapshots carry generator headers. `server/src/bin/sync-sql.ts` matches because it is the generator that writes the snapshot header; it stays production. A marker search proposes candidates; reading each file decides.

## Draft a policy from the evidence

Rules take the generated files from the module layout, because rules apply before build modules, and claim the end-to-end support code and the iOS app, which conventions leave as production or unclaimed. No override is needed, since none of these files is a conventional test or build script. Every entry restricts itself to source and records its reason.

```bash
source scripts/showcase-env.sh
cat > draft.toml <<'TOML'
version = 1

[[rules]]
id = "generated-sdk-client"
match = ["packages/sdk/src/fetch-client.ts"]
kind = ["source"]
purpose = "other"
category = "generated"
reason = "Header: DO NOT MODIFY, generated using oazapfts"

[[rules]]
id = "generated-query-snapshots"
match = ["server/src/queries/**"]
kind = ["source"]
purpose = "other"
category = "generated"
reason = "Header: auto generated by ./sql-generator"

[[rules]]
id = "end-to-end-suites"
match = ["e2e/src/**"]
kind = ["source"]
purpose = "test"
category = "end-to-end"
reason = "Playwright and API suites with their fixtures and utilities"

[[rules]]
id = "ios-app"
match = ["mobile/ios/**"]
kind = ["source"]
purpose = "production"
category = "native"
reason = "Swift code of the iOS app. No Package.swift declares it"
TOML
locmetry --root immich --config draft.toml --format table --verbose --no-color --no-bars \
  | sed '/^Root:/d; /^Roots:/d; /^Project:/d; /^Measured at:/d' \
  | sed -n '/^ Unclaimed source/,/^ To inspect/p' \
  | demo_wrap
```

```output
 Unclaimed source: 1,789 code lines in 57 files, here:
   mobile                413 code lines  12 files
   packages              265 code lines   8 files
   machine-learning      248 code lines   4 files
   deployment            225 code lines  16 files
   server                218 code lines   6 files
   docs                  213 code lines   2 files
   (root files)          114 code lines   2 files
   .devcontainer/server   58 code lines   3 files
   and 3 more folders (35 code lines)
 To inspect or refine purpose classification, use '--guide configure'. '--files --output -' shows each file's rule.
```

Four entries resolve more than half of the review list. What remains is deployment, tooling and a few single files, each needing its own evidence, such as the plugin SDK's declared `bin` entry. The [reviewed policy](../../examples/showcase/immich/overlay/.locmetryrc) finishes that work and is what the checkout's `.locmetryrc` contains.

## Check what the policy changed

Capture full reports with and without the reviewed policy. Native `compare` shows the aggregate effect: counts moving between purposes, and 58 lines leaving the source total: the ProGuard rules, which a language correction leaves uncounted, and the release-notes template, which another counts as Markdown documentation. `sed` keeps the purpose table.

```bash
source scripts/showcase-env.sh
printf 'version = 1\n' > builtin-only.toml
locmetry --root immich --config builtin-only.toml --files --output builtin.json
locmetry --root immich --files --output reviewed.json
locmetry compare builtin.json reviewed.json --format table --no-color \
  | sed -n '/ Purpose /,/Test\/prod (Code)/p'
```

```output
 Purpose        before    after   change │  before    after   change
 ────────────────────────────────────────┼──────────────────────────
 Production    227,764  210,696  -17,068 │ 194,167  179,488  -14,679
 Test           99,381  103,432   +4,051 │  83,595   87,011   +3,416
 Other           6,214   19,173  +12,959 │   4,945   16,165  +11,220
 ────────────────────────────────────────┼──────────────────────────
 Source total  333,359  333,301      -58 │ 282,707  282,664      -43

 Test/prod (Code): 0.43 → 0.48 (before → after)
```

The [file-transition helper](../../scripts/compare-locmetry-files.sh) names the files behind those totals. Grouping its changed rows by old and new classification gives a reviewable summary of every decision the policy made.

```bash
source scripts/showcase-env.sh
printf 'version = 1\n' > builtin-only.toml
locmetry --root immich --config builtin-only.toml --files --output builtin.json
locmetry --root immich --files --output reviewed.json
bash "$repository/scripts/compare-locmetry-files.sh" builtin.json reviewed.json \
  | jq '[.changes[] | select(.before.state == "counted" and .after.state == "counted")
      | {from: "\(.before.purpose)/\(.before.category)", to: "\(.after.purpose)/\(.after.category)",
         code: .before.counts.code}]
    | group_by([.from, .to])
    | map({from: .[0].from, to: .[0].to, files: length, code: (map(.code) | add)})
    | sort_by(-.code)'
```

```output
[
  {
    "from": "production/implementation",
    "to": "other/generated",
    "files": 38,
    "code": 16165
  },
  {
    "from": "test/test",
    "to": "test/end-to-end",
    "files": 41,
    "code": 7871
  },
  {
    "from": "production/implementation",
    "to": "production/native",
    "files": 38,
    "code": 4102
  },
  {
    "from": "production/implementation",
    "to": "test/test-support",
    "files": 23,
    "code": 3118
  },
  {
    "from": "other/unclaimed",
    "to": "production/native",
    "files": 29,
    "code": 3104
  },
  {
    "from": "test/test",
    "to": "test/integration",
    "files": 8,
    "code": 2014
  },
  {
    "from": "other/unclaimed",
    "to": "production/deployment",
    "files": 28,
    "code": 743
  },
  {
    "from": "production/implementation",
    "to": "production/distribution",
    "files": 5,
    "code": 567
  },
  {
    "from": "production/implementation",
    "to": "production/tooling",
    "files": 1,
    "code": 343
  },
  {
    "from": "other/unclaimed",
    "to": "production/implementation",
    "files": 10,
    "code": 326
  },
  {
    "from": "other/unclaimed",
    "to": "test/test-support",
    "files": 4,
    "code": 234
  },
  {
    "from": "other/unclaimed",
    "to": "production/documentation-site",
    "files": 2,
    "code": 213
  },
  {
    "from": "production/implementation",
    "to": "production/documentation-site",
    "files": 4,
    "code": 169
  },
  {
    "from": "other/unclaimed",
    "to": "production/tooling",
    "files": 9,
    "code": 164
  },
  {
    "from": "other/unclaimed",
    "to": "test/performance",
    "files": 1,
    "code": 64
  },
  {
    "from": "other/unclaimed",
    "to": "production/build-tooling",
    "files": 4,
    "code": 54
  },
  {
    "from": "other/unclaimed",
    "to": "other/unclaimed",
    "files": 1,
    "code": 6
  }
]
```

Counts stay identical; only purposes and categories move, and each row traces back to an entry with a recorded reason. The last row is the release-notes template, which leaves source for documentation, so its purpose and category stay the same. From here, [Measure Immich](measure.md) reads the configured result, [the workspace recording](workspace.md) splits it by component, and [the release comparison](compare.md) follows it over time.
