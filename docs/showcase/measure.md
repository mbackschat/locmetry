# Measure Immich

*2026-10-21T03:19:57Z by Showboat 0.6.1*
<!-- showboat-id: c6f79590-fe17-40b8-86ab-c5ea0d8918d1 -->

Run each Bash block from the Locmetry repository root after `cargo build --locked` and `bash scripts/fetch-showcase.sh`. `source scripts/showcase-env.sh` links the pinned [Immich](https://github.com/immich-app/immich) checkouts into a fresh temporary directory, with `immich` as the `v3.3.4` showcase release, applies Locmetry's [reviewed policy](../../examples/showcase/immich/README.md) and isolates personal defaults. The [showcase index](README.md) explains replay and projections.

## Start from built-in conventions

`--config` replaces the checkout's `.locmetryrc`; a file containing only `version = 1` therefore measures with built-in conventions alone. The projection shows all counted code, the source part of it and how built-in conventions split that source by purpose.

```bash
source scripts/showcase-env.sh
printf 'version = 1\n' > builtin-only.toml
locmetry --root immich --config builtin-only.toml --format json \
  | jq '{
      all_counted: .summary.all_counted.code,
      source: .summary.source.code,
      purposes: .summary.source_purpose | map_values(.code),
      unclaimed: .summary.unclaimed_folders[:3]
    }'
```

```output
{
  "all_counted": 522776,
  "source": 282770,
  "purposes": {
    "production": 194232,
    "test": 83595,
    "other": 4943
  },
  "unclaimed": [
    {
      "folder": "mobile/ios/Runner",
      "files": 21,
      "code": 2272
    },
    {
      "folder": "mobile/ios/WidgetExtension",
      "files": 7,
      "code": 830
    },
    {
      "folder": "mobile/pigeon",
      "files": 9,
      "code": 325
    }
  ]
}
```

Almost half of the counted code is data, chiefly JSON database schemas, the OpenAPI specification and translations; Locmetry keeps it out of the source total. Module descriptors and test conventions already classify most source: npm, Gradle, Python and Dart modules are production, and unit, Flutter integration and `*.e2e-spec.ts` suites are tests. No descriptor declares the iOS app, so its Swift code stays unclaimed and is listed for review rather than guessed.

## Check the evidence behind a decision

Conventions cannot know that a file inside a module was generated. The SDK client sits in the npm module's `src/` folder, so the module layout makes it production, although its header says otherwise.

```bash
source scripts/showcase-env.sh
printf 'version = 1\n' > builtin-only.toml
head -4 immich/packages/sdk/src/fetch-client.ts
locmetry explain packages/sdk/src/fetch-client.ts --root immich \
  --config builtin-only.toml --format json \
  | jq '.result | {rule: .file.rule, purpose: .file.purpose, code: .file.counts.code, decision}'
```

```output
/**
 * Immich
 * 3.3.4
 * DO NOT MODIFY - This file has been generated using oazapfts.
{
  "rule": "builtin:npm-module",
  "purpose": "production",
  "code": 6466,
  "decision": {
    "stage": "module",
    "matched_patterns": [
      "packages/sdk/src/**"
    ],
    "module": {
      "path": "packages/sdk",
      "detector": "npm"
    }
  }
}
```

## Measure with the reviewed policy

Without `--config`, the checkout's `.locmetryrc` applies. Its entries record what only the project knows: generated code, end-to-end support code, the platform code of the iOS and Android apps, deployment files and developer tooling. `--show-categories` adds the resulting categories.

```bash
source scripts/showcase-env.sh
locmetry --root immich --format table --show-categories --no-color --no-bars \
  | sed '/^Root:/d; /^Roots:/d; /^Project:/d; /^Measured at:/d'
```

```output
 ────────────────────────────────────────────────────────────────────────────────────────────
 Language    Files    Lines     Code │ Comments  Blanks │   Prod.    Test   Other │ Test/prod
 ────────────────────────────────────┼──────────────────┼─────────────────────────┼──────────
 TypeScript  1,085  171,956  145,943 │    4,633  21,380 │  70,861  68,616   6,466 │      0.97
 Dart          823   90,871   75,726 │    2,779  12,366 │  59,495  16,231       0 │      0.27
 JavaScript     17   23,273   19,746 │      427   3,100 │  19,684      62       0 │      0.00
 Svelte        412   19,014   17,442 │      200   1,372 │  17,420      22       0 │      0.00
 SQL            37   10,534    9,699 │      436     399 │       0       0   9,699 │         -
 Python         39    6,184    4,888 │       79   1,217 │   2,818   2,070       0 │      0.73
 Kotlin         35    4,604    3,810 │      177     617 │   3,810       0       0 │      0.00
 Swift          28    3,874    3,103 │      232     539 │   3,103       0       0 │      0.00
 CSS             2      818      691 │       26     101 │     691       0       0 │      0.00
 Shell          19      565      401 │       69      95 │     401       0       0 │      0.00
 Dockerfile      4      369      267 │       12      90 │     257      10       0 │      0.04
 C++             1      295      251 │       11      33 │     251       0       0 │      0.00
 HCL            16      262      225 │        4      33 │     225       0       0 │      0.00
 C               2      225      168 │       30      27 │     168       0       0 │      0.00
 Groovy          3      173      139 │        7      27 │     139       0       0 │      0.00
 Java            1      166      124 │       25      17 │     124       0       0 │      0.00
 HTML            1       74       66 │        2       6 │      66       0       0 │      0.00
 PostCSS         0       31       28 │        0       3 │      28       0       0 │      0.00
 CMake           1       13        9 │        0       4 │       9       0       0 │      0.00
 Templ           1       11        6 │        0       5 │       6       0       0 │      0.00
 C Header        1        1        1 │        0       0 │       1       0       0 │      0.00
 ────────────────────────────────────┼──────────────────┼─────────────────────────┼──────────
 Total       2,528  333,313  282,733 │    9,149  41,431 │ 179,557  87,011  16,165 │      0.48
 Lines = Code + Comments + Blanks · Production + Test + Other = Code · Test/prod = Test / Production (Code)
 Tests written in another language than the code they test skew per-language Test/prod.
 Folded: BASH → Shell, TSX → TypeScript.

 ──────────────────────────────────────────────────────
 Category                       Files    Lines     Code
 ──────────────────────────────────────────────────────
 production:implementation      1,837  199,093  170,113
 test:test                        450   87,800   73,710
 other:generated                   38   19,173   16,165
 test:end-to-end                   41    9,285    7,871
 production:native                 67    8,870    7,206
 test:test-support                 27    3,970    3,352
 test:integration                   8    2,296    2,014
 production:build-tooling          18      886      772
 production:deployment             28      938      740
 production:tooling                11      657      513
 production:documentation-site      2      264      213
 test:performance                   1       81       64
 Rows of one purpose add up to that purpose's Code, and all rows add up to the total Code.

```

Generated code moves from production to Other with category `generated`, end-to-end helpers join the tests, the end-to-end suites split into `end-to-end`, `integration` and `test-support` by what each file does, and nothing remains unclaimed. Test/prod divides test code by production code; it describes volume, not coverage.

## Trace a policy decision

The same explanation now names the rule that decided and the reason recorded with it, so a reviewer can check the evidence instead of trusting the number.

```bash
source scripts/showcase-env.sh
locmetry explain packages/sdk/src/fetch-client.ts --root immich --format json \
  | jq '.result | {rule: .file.rule, purpose: .file.purpose, category: .file.category, decision}'
```

```output
{
  "rule": "generated-sdk-client",
  "purpose": "other",
  "category": "generated",
  "decision": {
    "stage": "rule",
    "matched_patterns": [
      "packages/sdk/src/fetch-client.ts"
    ],
    "reason": "Header: DO NOT MODIFY, generated using oazapfts from the OpenAPI specification"
  }
}
```

## Review every entry of the policy

An explanation covers one file; `summary.policy_entries` covers every entry of the policy at once, with the files and code each decided and what it replaced. `--verbose` turns the replaced decisions that changed a purpose into one note: here the generated code moves from Production to Other and the end-to-end code to Test, as the policy intends. An unintended move, such as a glob that is broader than meant, would show up in the same place.

```bash
source scripts/showcase-env.sh
locmetry --root immich --format table --verbose --no-color --no-bars \
  | grep '^ Entries that changed' \
  | demo_wrap

```

```output
 Entries that changed the purpose a later stage would give: generated-sdk-client (Production to Other, 6,466 code
lines), generated-query-snapshots (Production to Other, 9,699 code lines), end-to-end-suites (Production to Test, 3,118
code lines).
```

The JSON entries also show how a rule and a refinement share one decision. The `end-to-end-suites` rule makes the suites' helpers tests, which the npm layout would count as production, and `preempted` names the convention that made the 49 spec files tests first. The `end-to-end` refinement then gives all of them one category, and its `displaced` shows that it changed only the category of files that were already tests.

```bash
source scripts/showcase-env.sh
locmetry --root immich --format json \
  | jq '.summary.policy_entries[] | select(.id == "end-to-end-suites" or .id == "end-to-end")'
```

```output
{
  "id": "end-to-end-suites",
  "stage": "rule",
  "purpose": "test",
  "files": 23,
  "code": 3118,
  "preempted": [
    {
      "by": "builtin:js-e2e-spec-file",
      "files": 49
    }
  ],
  "displaced": [
    {
      "decision": "production:implementation",
      "files": 23,
      "code": 3118
    }
  ]
}
{
  "id": "end-to-end",
  "stage": "refinement",
  "purpose": "test",
  "files": 41,
  "code": 7871,
  "displaced": [
    {
      "decision": "test:test",
      "files": 41,
      "code": 7871
    }
  ]
}
```

## See where the code lives

Adaptive folder detail expands the largest folders within a row budget and covers the source once, so parents and expanded children never count twice.

```bash
source scripts/showcase-env.sh
locmetry --root immich --format table --folder-rows 16 --no-color --no-bars --tables-only \
  | sed -n '/Folder/,$p'
```

```output
 Folder                   Lines     Code │   Prod.    Test   Other │ Test/prod
 ────────────────────────────────────────┼─────────────────────────┼──────────
 server/src             104,285   90,467 │  52,970  27,798   9,699 │      0.52
 mobile                 100,095   83,183 │  66,952  16,231       0 │      0.24
 web                     67,833   58,255 │  52,867   5,388       0 │      0.10
 server/test             24,611   21,157 │       0  21,157       0 │         -
 e2e/src                 15,300   13,003 │       0  13,003       0 │         -
 packages                12,893   10,065 │   2,345   1,254   6,466 │      0.53
 machine-learning         6,698    5,291 │   3,221   2,070       0 │      0.64
 docs                       492      401 │     401       0       0 │      0.00
 deployment                 262      225 │     225       0       0 │      0.00
 server (direct files)      243      200 │     200       0       0 │      0.00
 e2e (direct files)         191      171 │      61     110       0 │      1.80
 server/bin                 157      126 │     126       0       0 │      0.00
 (root files)               131      111 │     111       0       0 │      0.00
 .devcontainer/server        85       58 │      58       0       0 │      0.00
 open-api/bin                26       14 │      14       0       0 │      0.00
 misc/release                11        6 │       6       0       0 │      0.00
 ────────────────────────────────────────┼─────────────────────────┼──────────
 Total                  333,313  282,733 │ 179,557  87,011  16,165 │      0.48
```

## Look at the code around the product

The category table answers what the project consists of. `--table-category` narrows every table to chosen categories, which answers a question the full table cannot: what does it take to build, ship and test Immich, apart from the application and its tests, and which languages is that written in? A purpose selects all of its categories, a `purpose:category` key one of them, and quoted wildcards match several.

```bash
source scripts/showcase-env.sh
locmetry --root immich --format table --show-categories --no-color --no-bars \
  --table-category 'production:*tooling*,production:deployment,test:test-support' \
  | sed '/^Root:/d; /^Roots:/d; /^Project:/d; /^Measured at:/d' \
  | demo_wrap
```

```output
 ─────────────────────────────────────────────────────────────────────────────
 Language    Files  Lines   Code │ Comments  Blanks │ Prod.   Test │ Comment %
 ────────────────────────────────┼──────────────────┼──────────────┼──────────
 TypeScript     29  4,076  3,453 │      205     418 │   111  3,342 │      5.6%
 JavaScript     11    558    502 │       23      33 │   502      0 │      4.4%
 Shell          16    550    395 │       64      91 │   395      0 │     13.9%
 Dart            1    388    343 │        1      44 │   343      0 │      0.3%
 Dockerfile      4    369    267 │       12      90 │   257     10 │      4.3%
 HCL            16    262    225 │        4      33 │   225      0 │      1.7%
 Groovy          3    173    139 │        7      27 │   139      0 │      4.8%
 Python          2     51     38 │        1      12 │    38      0 │      2.6%
 CMake           1     13      9 │        0       4 │     9      0 │      0.0%
 Templ           1     11      6 │        0       5 │     6      0 │      0.0%
 ────────────────────────────────┼──────────────────┼──────────────┼──────────
 Total          84  6,451  5,377 │      317     757 │ 2,025  3,352 │      5.6%
 Lines = Code + Comments + Blanks · Production + Test = Code · Comment % = Comments / (Code + Comments)
 Folded: BASH → Shell.
 Not shown (categories): production:implementation, test:test, other:generated, test:end-to-end, production:native and 3
more (277,356 code lines).

 ─────────────────────────────────────────────
 Category                  Files  Lines   Code
 ─────────────────────────────────────────────
 test:test-support            27  3,970  3,352
 production:build-tooling     18    886    772
 production:deployment        28    938    740
 production:tooling           11    657    513
 Rows of one purpose add up to that purpose's Code, and all rows add up to the total Code.

```

About 2% of the source code builds, ships and tests the product, yet it spans 10 languages: TypeScript support code for the end-to-end suites, JavaScript bundler and lint configuration, Shell and Dockerfile for deployment, HCL for the Terraform modules and Groovy for Gradle. Comment % replaces Test/prod because the selection holds only part of Production and Test, and the note names the categories left out.

No Prolog row appears, although the counter reads `mobile/android/app/proguard-rules.pro` as Prolog by its extension: the file holds ProGuard keep rules, and the policy's language correction leaves it unrecognized, because no counter language shares their syntax.

## Focus on the mobile app

`--subfolder` measures one folder under the project's policy, so the categories keep their meaning. The policy labels the platform code of both mobile apps as `native`.

```bash
source scripts/showcase-env.sh
locmetry --root immich --subfolder mobile --format table --show-categories --no-color --no-bars \
  | sed '/^Root:/d; /^Roots:/d; /^Project:/d; /^Measured at:/d; /^Subfolder:/d' \
  | demo_wrap
```

```output
 ───────────────────────────────────────────────────────────────────────────────────────
 Language  Files    Lines    Code │ Comments  Blanks │  Prod.    Test  Other │ Test/prod
 ─────────────────────────────────┼──────────────────┼───────────────────────┼──────────
 Dart        823   90,871  75,726 │    2,779  12,366 │ 59,495  16,231      0 │      0.27
 Kotlin       35    4,604   3,810 │      177     617 │  3,810       0      0 │      0.00
 Swift        28    3,874   3,103 │      232     539 │  3,103       0      0 │      0.00
 C             2      225     168 │       30      27 │    168       0      0 │      0.00
 Groovy        3      173     139 │        7      27 │    139       0      0 │      0.00
 Java          1      166     124 │       25      17 │    124       0      0 │      0.00
 Shell         4      144      85 │       34      25 │     85       0      0 │      0.00
 Python        1       24      18 │        1       5 │     18       0      0 │      0.00
 CMake         1       13       9 │        0       4 │      9       0      0 │      0.00
 C Header      1        1       1 │        0       0 │      1       0      0 │      0.00
 ─────────────────────────────────┼──────────────────┼───────────────────────┼──────────
 Total       899  100,095  83,183 │    3,285  13,627 │ 66,952  16,231      0 │      0.24
 Lines = Code + Comments + Blanks · Production + Test + Other = Code · Test/prod = Test / Production (Code)
 Tests written in another language than the code they test skew per-language Test/prod.

 ────────────────────────────────────────────────
 Category                   Files   Lines    Code
 ────────────────────────────────────────────────
 production:implementation    668  70,213  59,152
 test:test                    154  20,270  16,231
 production:native             67   8,870   7,206
 production:tooling             5     511     414
 production:build-tooling       5     231     180
 Rows of one purpose add up to that purpose's Code, and all rows add up to the total Code.

```

The mobile app has a Test/prod of 0.24, half the project's 0.48. Its platform code, Kotlin, Java and C for Android and Swift for iOS, is 7,206 code lines, about a tenth of its production code; Dart carries the rest. The Gradle module layout already counts the Android code as production, so a refinement only relabels it `native`, while no build descriptor declares the iOS code, so a rule claims it.
