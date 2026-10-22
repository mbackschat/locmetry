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
    "production": 193662,
    "test": 83485,
    "other": 5623
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
      "folder": "packages",
      "files": 11,
      "code": 361
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
 TypeScript  1,085  171,956  145,943 │    4,633  21,380 │  70,971  68,506   6,466 │      0.97
 Dart          823   90,871   75,726 │    2,779  12,366 │  59,495  16,231       0 │      0.27
 JavaScript     17   23,273   19,746 │      427   3,100 │  19,684      62       0 │      0.00
 Svelte        412   19,014   17,442 │      200   1,372 │  17,420      22       0 │      0.00
 SQL            37   10,534    9,699 │      436     399 │       0       0   9,699 │         -
 Python         39    6,184    4,888 │       79   1,217 │   2,818   2,070       0 │      0.73
 Kotlin         35    4,604    3,810 │      177     617 │   3,810       0       0 │      0.00
 Swift          28    3,874    3,103 │      232     539 │   3,103       0       0 │      0.00
 CSS             2      818      691 │       26     101 │     691       0       0 │      0.00
 Shell          17      525      371 │       67      87 │     371       0       0 │      0.00
 Dockerfile      4      369      267 │       12      90 │     257      10       0 │      0.04
 C++             1      295      251 │       11      33 │     251       0       0 │      0.00
 HCL            16      262      225 │        4      33 │     225       0       0 │      0.00
 C               2      225      168 │       30      27 │     168       0       0 │      0.00
 Groovy          3      173      139 │        7      27 │     139       0       0 │      0.00
 Java            1      166      124 │       25      17 │     124       0       0 │      0.00
 HTML            1       74       66 │        2       6 │      66       0       0 │      0.00
 Prolog          1       47       37 │        0      10 │      37       0       0 │      0.00
 BASH            2       40       30 │        2       8 │      30       0       0 │      0.00
 PostCSS         0       31       28 │        0       3 │      28       0       0 │      0.00
 CMake           1       13        9 │        0       4 │       9       0       0 │      0.00
 Templ           1       11        6 │        0       5 │       6       0       0 │      0.00
 C Header        1        1        1 │        0       0 │       1       0       0 │      0.00
 ────────────────────────────────────┼──────────────────┼─────────────────────────┼──────────
 Total       2,529  333,360  282,770 │    9,149  41,441 │ 179,704  86,901  16,165 │      0.48
 Lines = Code + Comments + Blanks · Production + Test + Other = Code · Test/prod = Test / Production (Code)
 Tests written in another language than the code they test skew per-language Test/prod.
 Folded: TSX → TypeScript.

 ──────────────────────────────────────────────────
 Category                   Files    Lines     Code
 ──────────────────────────────────────────────────
 production:implementation  1,837  199,093  170,113
 test:test                    496   99,255   83,485
 other:generated               38   19,173   16,165
 production:native             67    8,870    7,206
 test:end-to-end               23    3,719    3,118
 production:tooling            28    1,709    1,439
 production:deployment         31    1,023      798
 test:test-infrastructure       4      251      234
 production:build-tooling       4      186      148
 test:performance-test          1       81       64
 Rows of one purpose add up to that purpose's Code, and all rows add up to the total Code.

```

Generated code moves from production to Other with category `generated`, end-to-end helpers join the tests, and nothing remains unclaimed. Test/prod divides test code by production code; it describes volume, not coverage.

## Trace a policy decision

The same explanation now names the override that decided and the reason recorded with it, so a reviewer can check the evidence instead of trusting the number.

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
    "stage": "override",
    "matched_patterns": [
      "packages/sdk/src/fetch-client.ts"
    ],
    "reason": "Header: DO NOT MODIFY, generated using oazapfts from the OpenAPI specification"
  }
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
 mobile                 100,142   83,220 │  66,989  16,231       0 │      0.24
 web                     67,833   58,255 │  52,867   5,388       0 │      0.10
 server/test             24,611   21,157 │       0  21,157       0 │         -
 e2e/src                 15,300   13,003 │       0  13,003       0 │         -
 packages                12,893   10,065 │   2,345   1,254   6,466 │      0.53
 machine-learning         6,698    5,291 │   3,221   2,070       0 │      0.64
 docs                       492      401 │     401       0       0 │      0.00
 deployment                 262      225 │     225       0       0 │      0.00
 server (direct files)      243      200 │     200       0       0 │      0.00
 e2e (direct files)         191      171 │     171       0       0 │      0.00
 server/bin                 157      126 │     126       0       0 │      0.00
 (root files)               131      111 │     111       0       0 │      0.00
 .devcontainer/server        85       58 │      58       0       0 │      0.00
 open-api/bin                26       14 │      14       0       0 │      0.00
 misc/release                11        6 │       6       0       0 │      0.00
 ────────────────────────────────────────┼─────────────────────────┼──────────
 Total                  333,360  282,770 │ 179,704  86,901  16,165 │      0.48
```

## Look at the code around the product

The category table answers what the project consists of. `--table-category` narrows every table to chosen categories, which answers a question the full table cannot: what does it take to build, ship and test Immich, apart from the application and its tests, and which languages is that written in? A purpose selects all of its categories, a `purpose:category` key one of them, and quoted wildcards match several.

```bash
source scripts/showcase-env.sh
locmetry --root immich --format table --show-categories --no-color --no-bars \
  --table-category 'production:*tooling*,production:deployment,test:*infra*' \
  | sed '/^Root:/d; /^Roots:/d; /^Project:/d; /^Measured at:/d' \
  | demo_wrap
```

```output
 ────────────────────────────────────────────────────────────────────────────
 Language    Files  Lines   Code │ Comments  Blanks │ Prod.  Test │ Comment %
 ────────────────────────────────┼──────────────────┼─────────────┼──────────
 JavaScript     13    822    715 │       62      45 │   715     0 │      8.0%
 TypeScript      9    483    445 │        8      30 │   221   224 │      1.8%
 Shell          14    510    365 │       62      83 │   365     0 │     14.5%
 Dart            1    388    343 │        1      44 │   343     0 │      0.3%
 Dockerfile      4    369    267 │       12      90 │   257    10 │      4.3%
 HCL            16    262    225 │        4      33 │   225     0 │      1.7%
 Groovy          3    173    139 │        7      27 │   139     0 │      4.8%
 Python          2     51     38 │        1      12 │    38     0 │      2.6%
 Prolog          1     47     37 │        0      10 │    37     0 │      0.0%
 BASH            2     40     30 │        2       8 │    30     0 │      6.2%
 CMake           1     13      9 │        0       4 │     9     0 │      0.0%
 Templ           1     11      6 │        0       5 │     6     0 │      0.0%
 ────────────────────────────────┼──────────────────┼─────────────┼──────────
 Total          67  3,169  2,619 │      159     391 │ 2,385   234 │      5.7%
 Lines = Code + Comments + Blanks · Production + Test = Code · Comment % = Comments / (Code + Comments)
 Not shown (categories): production:implementation, test:test, other:generated, production:native, test:end-to-end and
test:performance-test (280,151 code lines).

 ─────────────────────────────────────────────
 Category                  Files  Lines   Code
 ─────────────────────────────────────────────
 production:tooling           28  1,709  1,439
 production:deployment        31  1,023    798
 test:test-infrastructure      4    251    234
 production:build-tooling      4    186    148
 Rows of one purpose add up to that purpose's Code, and all rows add up to the total Code.

```

Less than 1% of the source code builds, ships and tests the product, yet it spans 12 languages: JavaScript bundler and lint configuration, Shell and Dockerfile for deployment, HCL for the Terraform modules and Groovy for Gradle. Comment % replaces Test/prod because the selection holds only part of Production and Test, and the note names the categories left out.

The Prolog row is a test-drive finding rather than Prolog: `mobile/android/app/proguard-rules.pro` holds ProGuard rules, which the counter recognizes as Prolog by their extension. Language mappings apply only to files the counter does not recognize, so the policy can classify the file as tooling but cannot correct its language.

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
 Prolog        1       47      37 │        0      10 │     37       0      0 │      0.00
 Python        1       24      18 │        1       5 │     18       0      0 │      0.00
 CMake         1       13       9 │        0       4 │      9       0      0 │      0.00
 C Header      1        1       1 │        0       0 │      1       0      0 │      0.00
 ─────────────────────────────────┼──────────────────┼───────────────────────┼──────────
 Total       900  100,142  83,220 │    3,285  13,637 │ 66,989  16,231      0 │      0.24
 Lines = Code + Comments + Blanks · Production + Test + Other = Code · Test/prod = Test / Production (Code)
 Tests written in another language than the code they test skew per-language Test/prod.

 ────────────────────────────────────────────────
 Category                   Files   Lines    Code
 ────────────────────────────────────────────────
 production:implementation    668  70,213  59,152
 test:test                    154  20,270  16,231
 production:native             67   8,870   7,206
 production:tooling             7     603     483
 production:build-tooling       4     186     148
 Rows of one purpose add up to that purpose's Code, and all rows add up to the total Code.

```

The mobile app has a Test/prod of 0.24, half the project's 0.48. Its platform code, Kotlin, Java and C for Android and Swift for iOS, is 7,206 code lines, about a tenth of its production code; Dart carries the rest. Android code needs an override because the Gradle module convention would otherwise label it `implementation`, while no build descriptor declares the iOS code, so an ordinary rule suffices there.
