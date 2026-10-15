<img src="docs/assets/locmetry.webp" alt="locmetry logo" width="96">

# locmetry

**Lines of code that mean something.** A raw counter tells you a repository has 120,000 lines. It cannot tell you how much of that is product code, tests, build glue, a vendored SDK, or a prototype nobody maintains. `locmetry` counts with [Tokei](https://github.com/XAMPPRocky/tokei) and then classifies every file by reviewable project rules, so the totals answer the questions people actually ask.

For the [sample project](#a-walk-through-a-sample-project) below, `locmetry --format markdown` starts with:

| Purpose | Files | Code | Comments | Blanks | Lines |
| --- | ---: | ---: | ---: | ---: | ---: |
| production | 8 | 94 | 4 | 22 | 120 |
| test | 2 | 26 | 0 | 6 | 32 |
| unclassified | 2 | 9 | 3 | 2 | 14 |

## Use cases

- **Track test investment:** follow the test-to-production ratio per module or over time, without hand-maintained `cloc` exclusion lists.
- **Size a codebase honestly:** estimate a migration, an audit, or due diligence on maintained code only, with vendored, generated, and experimental code excluded or labelled.
- **Give coding agents a reliable answer:** a compact JSON headline fits into an agent's context, and the report file answers follow-up questions with one `jq` query instead of ad hoc `find | wc -l` pipelines.
- **Find code nobody owns:** source that no rule claims lands in a review list, largest first.
- **Feed scripts and dashboards:** every row of the breakdown table is additive, so any grouping by folder, purpose, category, kind, or language is a plain sum.

## Features

- **Embedded Tokei 15.0.0:** more than 300 languages, embedded-language counting such as code inside Markdown, and comment and blank-line detection. It is compiled in, so no separate installation is needed.
- **Purpose and category:** each file is `production`, `test`, or `unclassified`, with an optional category such as `implementation`, `tooling`, or `build-tooling`, and the rule that decided it.
- **Built-in conventions:** test directories, Gradle and Kotlin test source sets, `*.test.ts`, `*Test.kt`, `*_test.go`, `test_*.py`, snapshot folders, build scripts, and wrapper scripts are recognized out of the box. The convention set is pinned by the configuration `version`, so upgrades never silently reclassify files.
- **Content kinds:** source, data, documentation, and other are counted separately, so a large JSON fixture or a README never inflates code totals.
- **Language mappings:** files Tokei does not recognize can be counted with a known parser, for example `.gradle` as Groovy and `.properties` as INI built in, plus your own mappings.
- **Explicit scope:** root-relative globs, ignore files, hidden-file control, and default exclusions for `.git`, `node_modules`, and `.build`.
- **Honest coverage:** selected files that could not be recognized or read are listed, never reported as zero lines.
- **Reproducible:** ambient Tokei configuration is ignored, and every report records the effective scope and policy.
- **Single static binary:** no Python, Git, Tokei, or Rust runtime required; Linux builds are fully static.

## Install

Prebuilt binaries for macOS arm64 and Linux x86_64 and aarch64 are on the [releases page](https://github.com/mbackschat/locmetry/releases):

```sh
# Linux targets: x86_64-unknown-linux-musl or aarch64-unknown-linux-musl
target=aarch64-apple-darwin
curl -fsSL "https://github.com/mbackschat/locmetry/releases/download/v0.1.0/locmetry-v0.1.0-$target.tar.gz" | tar -xz
install "locmetry-v0.1.0-$target/locmetry" ~/.local/bin/
```

Or build from source with a Rust toolchain:

```sh
cargo install --locked --git https://github.com/mbackschat/locmetry --tag v0.1.0
```

## A walk through a sample project

The synthetic project below has a Kotlin backend, a TypeScript web client, Python tooling, an abandoned prototype, a vendored SDK, and a binary diagram:

```text
acme-orders
├── .locmetryrc
├── settings.gradle.kts
├── gradlew
├── backend
│   ├── build.gradle.kts
│   └── src
│       ├── main/kotlin/acme/orders/{Order,OrderRepository,OrderService}.kt
│       ├── main/resources/application.properties
│       └── test/kotlin/acme/orders/OrderServiceTest.kt
├── web
│   ├── package.json
│   ├── tsconfig.jsonc
│   └── src/{cart.ts,Checkout.tsx,cart.test.ts}
├── scripts/seed_db.py
├── prototypes/pricing_experiment.py
├── docs/{architecture.md,order-flow.png}
└── third_party/payment-sdk/client.js
```

Its `.locmetryrc` only states what conventions cannot know: the vendored SDK is out of scope, `scripts/` is maintained tooling, and the two `src` trees are the application.

```toml
version = 1

[scope]
exclude = ["third_party/**"]

[[rules]]
id = "tooling"
match = ["scripts/**"]
purpose = "production"
category = "tooling"

[[rules]]
id = "application"
match = ["backend/src/**", "web/src/**"]
purpose = "production"
category = "implementation"
```

Everything else follows from built-in conventions. `OrderServiceTest.kt` and `cart.test.ts` are tests even though the `application` rule matches their paths, because conventions run before ordinary rules. The Gradle scripts are `build-tooling`, `gradlew` is a `third-party-wrapper`, and the prototype falls through to `unclassified`.

### Human report

`locmetry --root acme-orders --format markdown` prints, abridged:

```markdown
## Source categories

| Category | Files | Code | Comments | Blanks | Lines |
| --- | ---: | ---: | ---: | ---: | ---: |
| production:build-tooling | 2 | 9 | 0 | 1 | 10 |
| production:implementation | 5 | 73 | 3 | 15 | 91 |
| production:tooling | 1 | 12 | 1 | 6 | 19 |
| test:test | 2 | 26 | 0 | 6 | 32 |
| unclassified:third-party-wrapper | 1 | 1 | 2 | 0 | 3 |
| unclassified:unclassified | 1 | 8 | 1 | 2 | 11 |

## Other counted content

| Kind:purpose | Files | Code | Comments | Blanks | Lines |
| --- | ---: | ---: | ---: | ---: | ---: |
| data:production | 1 | 3 | 1 | 0 | 4 |
| data:unclassified | 2 | 15 | 0 | 0 | 15 |
| documentation:unclassified | 1 | 0 | 6 | 3 | 9 |

## Folder breakdown

| Folder | Production | Test | Unclassified | Total source code |
| --- | ---: | ---: | ---: | ---: |
| . | 94 | 26 | 9 | 129 |
| backend | 49 | 16 | 0 | 65 |
| backend/src | 42 | 16 | 0 | 58 |
| prototypes | 0 | 0 | 8 | 8 |
| scripts | 12 | 0 | 0 | 12 |
| web | 31 | 10 | 0 | 41 |
| web/src | 31 | 10 | 0 | 41 |

## Source requiring review

| Path | Code | Category |
| --- | ---: | --- |
| prototypes/pricing_experiment.py | 8 | unclassified |
| gradlew | 1 | third-party-wrapper |

## Coverage

Files selected: 17; counted: 16, including 2 through language mappings; not recognized: 1; recognized but unreadable: 0. Their lines are missing, not zero.
```

The vendored SDK is absent, `package.json` and `tsconfig.jsonc` count as data rather than code, `application.properties` and `tsconfig.jsonc` were counted through built-in language mappings, and the diagram is listed as unrecognized instead of silently dropped.

### JSON for agents and scripts

By default, stdout is a compact JSON headline, under 3 KB for this project. It contains totals, top-level folders, coverage, and the effective scope. `--output FILE` additionally writes the full report (jq output condensed):

```sh
$ locmetry --root acme-orders --output report.json \
    | jq '{source_purpose: .summary.source_purpose | map_values(.code), top_folders: .summary.top_folders, report}'
{
  "source_purpose": { "production": 94, "test": 26, "unclassified": 9 },
  "top_folders": {
    ".": { "production": 2, "unclassified": 1 },
    "backend": { "production": 49, "test": 16 },
    "prototypes": { "unclassified": 8 },
    "scripts": { "production": 12 },
    "web": { "production": 31, "test": 10 }
  },
  "report": "report.json"
}
```

The report file adds the review list and an additive `breakdown` table, one row per folder, kind, purpose, category, and language, each on its own line:

```json
{"folder":"backend/src/main/kotlin/acme/orders","kind":"source","purpose":"production","category":"implementation","language":"Kotlin","files":3,"code":42,"comments":2,"blanks":11,"lines":55},
{"folder":"backend/src/test/kotlin/acme/orders","kind":"source","purpose":"test","category":"test","language":"Kotlin","files":1,"code":16,"comments":0,"blanks":4,"lines":20},
```

Because rows are additive, any question is a filter and a sum:

```sh
# Test code per language
$ jq -r '.breakdown[] | select(.kind == "source" and .purpose == "test") | "\(.language)\t\(.code)"' report.json
Kotlin	16
TypeScript	10

# Source code per purpose under web/, at any depth
$ jq -c '[.breakdown[] | select(.kind == "source" and (.folder == "web" or (.folder | startswith("web/"))))]
         | group_by(.purpose) | map({(.[0].purpose): (map(.code) | add)}) | add' report.json
{"production":31,"test":10}

# Why is this file unclassified?
$ jq -c '.review[]' report.json
{"path":"prototypes/pricing_experiment.py","language":"Python","category":"unclassified","rule":"fallback","code":8,"lines":11}
{"path":"gradlew","language":"Shell","category":"third-party-wrapper","rule":"builtin:build-wrapper","code":1,"lines":3}
```

`--files` adds one record per file with its language, kind, purpose, category, deciding rule, and counts, for debugging rules.

## Usage

```sh
locmetry                                        # measure the current directory, using ./.locmetryrc if present
locmetry --root path/to/project                 # uses path/to/project/.locmetryrc if present
locmetry --config shared.toml                   # explicit configuration; the scan root is unchanged
locmetry --exclude 'generated/**' --hidden false
locmetry --output /tmp/report.json --files      # headline on stdout, full report with file records
locmetry --output - | jq .breakdown             # full report on stdout
```

Without a configuration, built-in exclusions, conventions, and language mappings apply. Command-line scope options override the configuration. Exit code 0 means a complete report, 2 a partial one because some files could not be read, and 1 a failure. [PRODUCT.md](PRODUCT.md) specifies every configuration key, the matching precedence, and the JSON schema.

## Documentation

- [Product vision and requirements](PRODUCT.md): configuration contract, report schema, and guarantees.
- [Measurement semantics and limits](docs/measurement.md): selection and counting details and known limits.
- [Tokei integration and distribution](docs/tokei-integration.md): embedded backend, release binaries, and the Homebrew formula.
- [Research](docs/research/README.md): counter capabilities and classification findings.
- [Development plan](PLAN.md): remaining work and completion evidence.
- [Repository guidelines](CLAUDE.md): contributor conventions and checks; `AGENTS.md` links to this file.

## License

Licensed under either the [Apache License, Version 2.0](LICENSE-APACHE) or the [MIT License](LICENSE-MIT), at your option.
