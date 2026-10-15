<img src="docs/assets/locmetry.webp" alt="locmetry logo" width="96">

# locmetry

**Lines of code that mean something.** A line counter tells you a repository has 120,000 lines. It cannot tell you how much of that is product code, how much is tests, and how much is build glue, a vendored SDK, or a prototype nobody maintains. `locmetry` counts every file and classifies it by purpose, using built-in conventions plus a few reviewable project rules. The numbers then answer the questions people actually ask.

- **For people:** a terminal table of source code by language, folder, and purpose.
- **For agents and scripts:** compact JSON, plus a full report whose rows add up under any grouping.
- **For reviews and wikis:** a Markdown report.

It is one static binary for macOS arm64 and Linux x86_64 and aarch64, with no runtime dependencies.

## What you see

Run `locmetry` in a project. For the [sample project](#a-tour-of-a-sample-project) used throughout this page:

```text
$ locmetry

 acme-orders: source code lines by purpose

 Language    Files  Production  Test  Unclassified  Code  Comments  Blanks
 ─────────────────────────────────────────────────────────────────────────
 Kotlin          6          51    16             0    67         2      16
 TypeScript      2          11    10             0    21         1       4
 Python          2          12     0             8    20         2       8
 TSX             1          20     0             0    20         0       2
 Shell           1           0     0             1     1         2       0
 ─────────────────────────────────────────────────────────────────────────
 Total          12          94    26             9   129         7      30

 Folder        Production  Test  Unclassified  Code
 ──────────────────────────────────────────────────
 backend               49    16             0    65
 web                   31    10             0    41
 scripts               12     0             0    12
 prototypes             0     0             8     8
 (root files)           2     0             1     3

 Not source code: 3 data files (19 lines), 1 documentation file (9 lines).
 Review: 2 unclassified source files (9 code lines); details with --format markdown.
 Not counted: 1 unrecognized file (png: 1); missing lines are not zero.
```

What the table already handles for you:

- **No language list to maintain:** every programming language counts, while JSON, YAML, Markdown, and other data or documentation stay out of the source totals.
- **Tests found by convention:** test files count as tests even inside the application's source tree.
- **Scope as policy:** the vendored SDK is excluded by one line of configuration.
- **Nothing silently promoted:** build scripts and tooling stay apart from implementation, and code no rule claims is reported for review instead of being counted as production.

## Use cases

- **Understand an unfamiliar repository:** see in one screen which languages matter, where the tests live, and which code nobody claims.
- **Track test investment:** follow the test-to-production ratio per module and over time, without hand-maintained exclusion lists.
- **Size a codebase honestly:** estimate a migration, a rewrite, an audit, an acquisition, or an outsourcing quote on maintained code only. Vendored, generated, and experimental code is excluded or labelled.
- **Guard quality in CI:** fail a build when unclassified code grows or the test share drops, with one `jq` expression ([example](#continuous-integration)).
- **Compare modules in a monorepo:** the folder rows and the additive breakdown table compare packages, services, or layers by purpose.
- **Watch a codebase change:** measure two releases or branches and compare production and test growth instead of raw line deltas, for example to see what an agent-written change really added.
- **Give coding agents a reliable answer:** a compact JSON headline fits an agent's context, and follow-up questions are one `jq` query over the report file instead of improvised `find | xargs wc -l` pipelines.
- **Report to people:** paste the Markdown report into a pull request, an architecture review, or a wiki page.

## Features

- **Accurate counting for more than 300 languages:** code, comment, and blank lines, including code embedded in other languages, such as snippets in Markdown.
- **Purpose and category per file:** `production`, `test`, or `unclassified`, with an optional category such as `implementation`, `tooling`, `build-tooling`, or `example`. Every file records the rule that decided it.
- **Built-in conventions:** these are recognized out of the box:
  - test directories, Gradle and Kotlin test source sets, `*.test.ts`, `*Test.kt`, `*_test.go`, `test_*.py`, and snapshot folders;
  - Gradle, Maven, CMake, and Make build scripts;
  - build-tool wrappers such as `gradlew`.

  The configuration `version` pins the convention set, so upgrades never silently reclassify files.
- **Content kinds:** source, data, documentation, and other are counted separately, so a large JSON fixture or a README never inflates code totals.
- **Language mappings:** files with an unknown extension can be counted as a known language. `.gradle` (as Groovy), `.jsonc`, `.jsonl`, and `.properties` are built in, and you can add your own.
- **Explicit scope:** root-relative globs, ignore files, hidden-file control, default exclusions for `.git`, `node_modules`, and `.build`, and optional personal exclusions shared by all projects.
- **Honest coverage:** files that could not be recognized or read are listed, never reported as zero lines.
- **Reproducible:** only the project's configuration, command-line options, and optional personal exclusions affect results, and every report records the effective scope and policy.
- **Three outputs, one measurement:** the terminal table, JSON, and Markdown are rendered from the same per-file records.

## Install

With Homebrew on macOS arm64 or Linux x86_64 and aarch64, which installs the prebuilt binary:

```sh
brew install mbackschat/tap/locmetry
```

The same binaries are on the [releases page](https://github.com/mbackschat/locmetry/releases):

```sh
# Linux targets: x86_64-unknown-linux-musl or aarch64-unknown-linux-musl
target=aarch64-apple-darwin
curl -fsSL "https://github.com/mbackschat/locmetry/releases/download/v0.1.1/locmetry-v0.1.1-$target.tar.gz" | tar -xz
install "locmetry-v0.1.1-$target/locmetry" ~/.local/bin/
```

Or build from source with a Rust toolchain:

```sh
cargo install --locked --git https://github.com/mbackschat/locmetry --tag v0.1.1
```

## Quick start

```sh
cd your/project
locmetry                      # table on a terminal
locmetry --format markdown    # full human report
locmetry | jq .summary        # piped: JSON headline
```

Without configuration, built-in exclusions, conventions, and language mappings apply. Add a `.locmetryrc` to the project root when the conventions cannot know something, such as which folders are vendored, generated, tooling, or experimental. Command-line options override it:

```sh
locmetry --exclude 'generated/**' --hidden false
```

Personal defaults for every project, such as extra exclusions or language mappings, go in `~/.config/locmetry/config.toml` (or under `$XDG_CONFIG_HOME`). They never change classification, reports name the file, and `--no-user-config` ignores it, for example in CI. [PRODUCT.md](PRODUCT.md#user-defaults) lists what they may set.

## A tour of a sample project

The synthetic project behind every output on this page has a Kotlin backend, a TypeScript web client, Python tooling, an abandoned prototype, a vendored SDK, and a binary diagram:

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

### Configuration

Its `.locmetryrc` states only what conventions cannot know: the vendored SDK is out of scope, `scripts/` is maintained tooling, and the two `src` trees are the application.

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

Each file gets the first match of four stages:

1. `[[overrides]]`, for decisions that must beat everything;
2. built-in test conventions, then built-in build conventions;
3. `[[rules]]`;
4. the fallback, `unclassified` by default.

So `OrderServiceTest.kt` and `cart.test.ts` are tests even though the `application` rule matches their paths. The Gradle scripts are `build-tooling`, and `gradlew` is a `third-party-wrapper`. The prototype falls through to `unclassified`, where the review list surfaces it. [PRODUCT.md](PRODUCT.md#project-configuration) documents every key, glob semantics, and `[[languages.mappings]]`.

### Terminal table

Plain `locmetry` on a terminal prints the table shown [above](#what-you-see):

- language rows, largest first;
- top-level folders, whose rows add up to the total;
- one line each for content that is not source, source awaiting classification, and files that could not be counted.

`--sort name` lists rows alphabetically instead, in the table and in Markdown.

### JSON headline for agents and scripts

When stdout is not a terminal, `locmetry` prints a compact JSON headline, under 3 KB for this project. Abridged:

```json
{
  "schema_version": "1",
  "project": "acme-orders",
  "metadata": {
    "root": "/work/acme-orders",
    "locmetry_version": "0.1.1",
    "scope": {
      "exclude": ["**/.git/**", "**/node_modules/**", "**/.build/**", "third_party/**", ".locmetryrc"]
    },
    "config_path": "acme-orders/.locmetryrc",
    "coverage": {
      "selected_files": 17,
      "counted_files": 16,
      "mapped_files": 2,
      "unrecognized_files": 1,
      "unrecognized_extensions": { "png": 1 },
      "unread": []
    }
  },
  "summary": {
    "all_counted": { "files": 16, "code": 147, "comments": 14, "blanks": 33, "lines": 194 },
    "source": { "files": 12, "code": 129, "comments": 7, "blanks": 30, "lines": 166 },
    "source_purpose": {
      "production": { "files": 8, "code": 94, "comments": 4, "blanks": 22, "lines": 120 },
      "test": { "files": 2, "code": 26, "comments": 0, "blanks": 6, "lines": 32 },
      "unclassified": { "files": 2, "code": 9, "comments": 3, "blanks": 2, "lines": 14 }
    },
    "source_categories": {
      "production:build-tooling": { "files": 2, "code": 9, "comments": 0, "blanks": 1, "lines": 10 },
      "production:implementation": { "files": 5, "code": 73, "comments": 3, "blanks": 15, "lines": 91 },
      "production:tooling": { "files": 1, "code": 12, "comments": 1, "blanks": 6, "lines": 19 },
      "...": "test:test, unclassified:third-party-wrapper, unclassified:unclassified"
    },
    "source_languages": { "...": "Kotlin, Python, Shell, TSX, TypeScript" },
    "top_folders": {
      ".": { "production": 2, "unclassified": 1 },
      "backend": { "production": 49, "test": 16 },
      "prototypes": { "unclassified": 8 },
      "scripts": { "production": 12 },
      "web": { "production": 31, "test": 10 }
    }
  }
}
```

Summaries omit zero counts, so a missing entry means zero. The full schema is in [PRODUCT.md](PRODUCT.md#reporting-and-coverage).

### Report file for any question

`--output FILE` writes the full report next to the headline: the summary, a review list, and an additive `breakdown` table with one row per folder, kind, purpose, category, and language. Each row sits on its own line:

```json
{"folder":"backend/src/main/kotlin/acme/orders","kind":"source","purpose":"production","category":"implementation","language":"Kotlin","files":3,"code":42,"comments":2,"blanks":11,"lines":55},
{"folder":"backend/src/test/kotlin/acme/orders","kind":"source","purpose":"test","category":"test","language":"Kotlin","files":1,"code":16,"comments":0,"blanks":4,"lines":20},
```

Because rows are additive, any question is a filter and a sum:

```sh
$ locmetry --output report.json > /dev/null

# Test code per language
$ jq -r '.breakdown[] | select(.kind == "source" and .purpose == "test") | "\(.language)\t\(.code)"' report.json
Kotlin	16
TypeScript	10

# Source code per purpose under web/, at any depth
$ jq -c '[.breakdown[] | select(.kind == "source" and (.folder == "web" or (.folder | startswith("web/"))))]
         | group_by(.purpose) | map({(.[0].purpose): (map(.code) | add)}) | add' report.json
{"production":31,"test":10}

# Which source still needs a rule?
$ jq -c '.review[]' report.json
{"path":"prototypes/pricing_experiment.py","language":"Python","category":"unclassified","rule":"fallback","code":8,"lines":11}
{"path":"gradlew","language":"Shell","category":"third-party-wrapper","rule":"builtin:build-wrapper","code":1,"lines":3}
```

`--files` adds one record per file with its language, kind, purpose, category, deciding rule, and counts, for debugging rules. `--output -` prints the full report to stdout instead.

### Markdown report

`locmetry --format markdown` renders a report to save or paste. Besides purpose and category tables, it contains other counted content, source languages, a recursive folder breakdown to `--depth N` levels, the review list, and coverage. Abridged:

```markdown
## Source categories

| Category | Files | Code | Comments | Blanks | Lines |
| --- | ---: | ---: | ---: | ---: | ---: |
| production:implementation | 5 | 73 | 3 | 15 | 91 |
| test:test | 2 | 26 | 0 | 6 | 32 |
| production:tooling | 1 | 12 | 1 | 6 | 19 |
| production:build-tooling | 2 | 9 | 0 | 1 | 10 |
| unclassified:unclassified | 1 | 8 | 1 | 2 | 11 |
| unclassified:third-party-wrapper | 1 | 1 | 2 | 0 | 3 |

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

Files selected: 17; counted: 16, including 2 through language mappings; not recognized: 1; recognized but unreadable: 0. Their lines are missing, not zero. Counts describe the working copy, including untracked files, not a commit.
```

## Continuous integration

The headline makes guardrails one-liners. `jq -e` exits nonzero when the expression is false:

```sh
# Fail when more than 50 lines of source lack a classification
locmetry | jq -e '(.summary.source_purpose.unclassified.code // 0) <= 50'

# Print the test share of production code
locmetry | jq -r '.summary.source_purpose | "test/production: \((.test.code // 0) * 100 / .production.code | floor)%"'
# test/production: 27%
```

Exit codes:

| Code | Meaning |
| --- | --- |
| 0 | Complete report. |
| 2 | Partial report: some files could not be read. |
| 1 | Failure, such as invalid configuration or usage. |

## Output formats

| Format | Selected by | For | Contents |
| --- | --- | --- | --- |
| Table | default on a terminal, `--format table` | people | source code by language and top-level folder, split by purpose, plus one-line notes |
| JSON headline | default when piped, `--format json` | agents, scripts, CI | totals by purpose, category, kind, and language; top-level folders; effective scope; coverage |
| Report file | `--output FILE`, or `--output -` for stdout | deeper questions | the headline plus review list, additive breakdown table, effective configuration, and with `--files` per-file records |
| Markdown | `--format markdown` | reviews, wikis | purpose, category, kind, language, and folder tables, review list, and coverage |

## Compared with line counters

Line counters such as [cloc](https://github.com/AlDanial/cloc) and [scc](https://github.com/boyter/scc) report lines per language. locmetry adds what language totals lack: scope as reviewable policy, purpose and category per file with provenance, separation of data and documentation from source, and coverage that names what was not counted. [Research](docs/research/README.md) compares the tools in more detail.

Known limits: classification is per file, so a file mixing production code and inline tests counts as one purpose. Git is not invoked, so a measurement describes the working copy rather than a commit. [Measurement semantics and limits](docs/measurement.md) lists the details.

## Documentation

- [Product vision and requirements](PRODUCT.md): configuration contract, report schema, and guarantees.
- [Measurement semantics and limits](docs/measurement.md): selection and counting details and known limits.
- [Architecture and distribution](docs/tokei-integration.md): embedded counting engine, release binaries, and the Homebrew formula.
- [Research](docs/research/README.md): counter capabilities and classification findings.
- [Development plan](PLAN.md): remaining work and completion evidence.
- [Repository guidelines](CLAUDE.md): contributor conventions and checks; `AGENTS.md` links to this file.

## License

Licensed under the [MIT License](LICENSE). Line counting uses the embedded [Tokei](https://github.com/XAMPPRocky/tokei) library.
