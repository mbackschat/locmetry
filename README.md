<img src="docs/assets/locmetry.webp" alt="locmetry logo" width="96">

# locmetry

**Lines of code, split by purpose: production, tests, tooling, and what nobody claims.** A line counter tells you a repository has 120,000 lines. It cannot tell you how much of that is product code, how much is tests, and how much is build glue, a vendored SDK, or a prototype nobody maintains. `locmetry` counts every file, gives it a purpose and a category, and records the rule that decided it.

- **Useful without configuration:** test conventions and build files (npm, Gradle, Maven, Cargo, Go, Python, Lake, Swift) classify most code out of the box.
- **Flexible, reviewable configuration:** a short `.locmetryrc` states only what conventions cannot know: what is out of scope, which folders are tooling or examples, which archived code must not count as tests, your own categories, and languages for unknown file types. The precedence is fixed and documented, so the same files and configuration always give the same numbers.
- **Built for coding agents:** `locmetry --schema config` explains the configuration and its semantics from the binary alone. In blind trials, agents without access to these docs wrote configurations that agreed with human-reviewed ones on 99% or more of code lines ([trials](docs/research/real-project-trials.md)).
- **JSON for agents and scripts:** a headline small enough for an agent's context, with totals by purpose, category, content kind, and language, per top-level folder and build system, plus the folders that still need a decision. `--output` adds a full report with additive breakdown rows and per-file records, and `--schema report` describes every key, so nothing has to be guessed or scraped, and every follow-up question is a `jq` query.
- **Ready-made metrics:** the tables show test lines per production line for every folder, and a workspace adds each repository's ratio and share. The JSON keeps exact counts, so any other ratio, such as a test-share guard in CI, is one `jq` expression.
- **Several repositories as one:** `locmetry workspace` measures sibling repositories, each with its own configuration, as one virtual monorepo with a row per repository.
- **Three outputs from one measurement:** a terminal table for people, JSON for agents and scripts, and Markdown for reviews and wikis.

It is one static binary for macOS arm64 and Linux x86_64 and aarch64, with no runtime dependencies.

## What you see

Run `locmetry` in a project. For the [sample project](#a-tour-of-a-sample-project) used throughout this page:

```text
$ locmetry

 acme-orders: source code lines by purpose

                                              Code by purpose
 Language    Files  Code │ Comments  Blanks │ Production  Test  Unclassified
 ────────────────────────┼──────────────────┼───────────────────────────────
 Kotlin          6    67 │        2      16 │         51    16             0
 TypeScript      2    21 │        1       4 │         11    10             0
 Python          2    20 │        2       8 │         12     0             8
 TSX             1    20 │        0       2 │         20     0             0
 Shell           1     1 │        2       0 │          0     0             1
 ────────────────────────┼──────────────────┼───────────────────────────────
 Total          12   129 │        7      30 │         94    26             9
 Code + Comments + Blanks = all lines; Production + Test + Unclassified = Code.

                      Code by purpose
 Folder        Code │ Production  Test  Unclassified │ Test/prod
 ───────────────────┼────────────────────────────────┼──────────
 backend         65 │         49    16             0 │      0.33
 web             41 │         31    10             0 │      0.32
 scripts         12 │         12     0             0 │      0.00
 prototypes       8 │          0     0             8 │         -
 (root files)     3 │          2     0             1 │      0.00
 ───────────────────┼────────────────────────────────┼──────────
 Total          129 │         94    26             9 │      0.28
 Production + Test + Unclassified = Code; Test/prod = Test / Production.

 Not source code: 3 data files (19 lines), 1 documentation file (9 lines).
 Production from build modules: gradle 42 code lines in 1 module, npm 31 code lines in 1 module.
 Unclaimed source: 8 code lines in 1 file, here:
   prototypes  8 code lines  1 file
 Classify them in the configuration; locmetry --schema config explains how, --files --output - shows each file's rule.
 Not counted: 1 unrecognized file (png: 1); missing lines are not zero.
```

What the table already handles for you:

- **No language list to maintain:** every programming language counts, while JSON, YAML, Markdown, and other data or documentation stay out of the source totals.
- **Production from the build:** `backend/build.gradle.kts` and `web/package.json` mark their source trees as production, with no rule, and the table names both build systems.
- **Tests found by convention:** test files count as tests even inside the application's source tree.
- **Scope as policy:** the vendored SDK is excluded by one line of configuration.
- **Nothing silently promoted:** build scripts and tooling stay apart from implementation, and code no rule claims is reported for review instead of being counted as production.

## Use cases

- **Understand an unfamiliar repository:** see in one screen which languages matter, where the tests live, and which code nobody claims.
- **Track test investment:** follow the test-to-production ratio per folder and over time, without hand-maintained exclusion lists.
- **Size a codebase honestly:** estimate a migration, a rewrite, an audit, an acquisition, or an outsourcing quote on maintained code only. Vendored, generated, and experimental code is excluded or labelled.
- **Guard quality in CI:** fail a build when unclaimed code grows or the test share drops, with one `jq` expression ([example](#continuous-integration)).
- **Compare parts of a monorepo:** the folder rows and the additive breakdown table compare packages, services, or layers by purpose.
- **Measure several repositories as one:** `locmetry workspace` merges sibling repositories, each measured with its own configuration, into one virtual monorepo whose folder rows compare the repositories ([how](#measure-several-repositories-together)).
- **Watch a codebase change:** measure two releases or branches and compare production and test growth instead of raw line deltas, for example to see what an agent-written change really added.
- **Let an agent configure a project:** a coding agent working in a repository writes its `.locmetryrc` itself. `locmetry --help` leads it to `--schema config`, which explains the configuration and its semantics, and the measurement shows which folders still need a decision ([how](#configure-with-an-agent)).
- **Give coding agents a reliable answer:** a compact JSON headline fits an agent's context, and follow-up questions are one `jq` query over the report file instead of improvised `find | xargs wc -l` pipelines.
- **Report to people:** paste the Markdown report into a pull request, an architecture review, or a wiki page.

## Features

### Counting for more than 300 languages

Code, comment, and blank lines per file, including code embedded in other languages, such as snippets in Markdown. The embedded [Tokei](https://github.com/XAMPPRocky/tokei) library does the counting, so nothing is reimplemented.

### Purpose, category, and provenance per file

Every file gets a purpose, `production`, `test`, or `unclassified`, an optional category such as `implementation`, `tooling`, `build-tooling`, or `example`, and the rule that decided it. For the [sample project](#a-tour-of-a-sample-project), `locmetry --files --output - | jq …` lists:

```text
path                                                     purpose       category             rule
backend/build.gradle.kts                                 production    build-tooling        builtin:build-script
backend/src/main/kotlin/acme/orders/Order.kt             production    implementation       builtin:gradle-module
backend/src/main/kotlin/acme/orders/OrderRepository.kt   production    implementation       builtin:gradle-module
backend/src/main/kotlin/acme/orders/OrderService.kt      production    implementation       builtin:gradle-module
backend/src/test/kotlin/acme/orders/OrderServiceTest.kt  test          test                 builtin:test-directory
gradlew                                                  unclassified  third-party-wrapper  builtin:build-wrapper
prototypes/pricing_experiment.py                         unclassified  unclassified         fallback
scripts/seed_db.py                                       production    tooling              tooling
settings.gradle.kts                                      production    build-tooling        builtin:build-script
web/src/Checkout.tsx                                     production    implementation       builtin:npm-module
web/src/cart.test.ts                                     test          test                 builtin:js-test-file
web/src/cart.ts                                          production    implementation       builtin:npm-module
```

Only `tooling` comes from the project's configuration; conventions and build files decide everything else, and the prototype nobody claimed stays `unclassified` for review.

### Built-in conventions

Recognized out of the box, as the listing above shows:

- test directories, Gradle and Kotlin test source sets, Swift test targets, Lean test libraries, `*.test.ts`, `*Test.kt`, `*_test.go`, `test_*.py`, and snapshot folders;
- Gradle, Maven, CMake, Make, and Lake build scripts, as `build-tooling`;
- build-tool wrappers such as `gradlew`, as `third-party-wrapper`.

A release may add conventions, which can reclassify files, but changing or removing one requires a new configuration `version`; every report records the conventions it applied.

### Production from the build

A folder with an npm, Gradle, Maven, Cargo, Go, Python, Lake, or Swift build file is a [build module](#build-modules), and that build system's source layout inside it counts as production. Nothing is assumed where no build declares anything, and the table says which build systems decided:

```text
 Production from build modules: gradle 42 code lines in 1 module, npm 31 code lines in 1 module.
```

### Content kinds

Source, data, documentation, and other content are counted separately, so a large JSON fixture or a README never inflates code totals:

```text
 Not source code: 3 data files (19 lines), 1 documentation file (9 lines).
```

### Explicit scope

Root-relative globs, ignore files, hidden-file control, and default exclusions for `.git`, `node_modules`, and the `.build` and `.lake` build folders decide what is measured. Vendored or generated code is one line of configuration away:

```toml
version = 1

[scope]
exclude = ["third_party/**", "**/*.generated.ts"]
```

Optional [personal defaults](PRODUCT.md#user-defaults) add exclusions for every project, and an exclusion that excludes nothing is reported, so a misspelled path does not go unnoticed.

### Language mappings

Files with an extension the counter does not know can be counted as a known language. `.gradle` (as Groovy), `.jsonc`, `.jsonl`, and `.properties` are built in, and you can add your own:

```toml
version = 1

[[languages.mappings]]
id = "runbooks"
match = ["**/*.runbook"]
language = "BASH"
```

### Honest coverage

Files that could not be recognized or read are listed, never reported as zero lines:

```text
 Not counted: 1 unrecognized file (png: 1); missing lines are not zero.
```

### Reproducible and deterministic

Only the project's configuration, command-line options, and optional personal exclusions affect results, and every report records the effective scope and policy. The same files and configuration always give the same report, independent of the order in which files are found; [PRODUCT.md](PRODUCT.md#matching-and-precedence) defines the precedence.

### Self-describing configuration

`locmetry --schema config` prints a JSON Schema that explains every setting, the classification order, and the built-in conventions, so an agent can write the configuration without reading these docs ([how](#configure-with-an-agent)). `--schema report`, `--schema user-config`, and `--schema workspace` describe the other files.

### Workspaces

`locmetry workspace --root web --root api` measures several repositories, each with its own configuration, in parallel, and merges them into one report with a row per repository:

```text

                    Code by purpose
 Repository  Code │ Production  Test  Unclassified │ Test/prod   Share
 ─────────────────┼────────────────────────────────┼──────────────────
 api           14 │          6     8             0 │      1.33   51.9%
 web           13 │          7     5             1 │      0.71   48.1%
 ─────────────────┼────────────────────────────────┼──────────────────
 Total         27 │         13    13             1 │      1.00  100.0%
 Production + Test + Unclassified = Code; Test/prod = Test / Production; Share = Code / workspace Code.
```

[Measure several repositories together](#measure-several-repositories-together) describes the mode.

### Three outputs, one measurement

The [terminal table](#terminal-table), [JSON](#json-headline-for-agents-and-scripts), and [Markdown](#markdown-report) are rendered from the same per-file records.

## Install

With Homebrew on macOS arm64 or Linux x86_64 and aarch64, which installs the prebuilt binary:

```sh
brew install mbackschat/tap/locmetry
```

On Linux distributions with an older toolchain, such as Ubuntu 22.04, Homebrew also installs its own gcc and glibc for this formula; locmetry itself is a static binary and uses neither.

The same binaries are on the [releases page](https://github.com/mbackschat/locmetry/releases):

```sh
# Linux targets: x86_64-unknown-linux-musl or aarch64-unknown-linux-musl
target=aarch64-apple-darwin
curl -fsSL "https://github.com/mbackschat/locmetry/releases/download/v0.1.5/locmetry-v0.1.5-$target.tar.gz" | tar -xz
install "locmetry-v0.1.5-$target/locmetry" ~/.local/bin/
```

Or build from source with a Rust toolchain:

```sh
cargo install --locked --git https://github.com/mbackschat/locmetry --tag v0.1.5
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

## Command-line options

| Option | Effect |
| --- | --- |
| `--root DIR` | Directory to measure; default: the current directory. |
| `--config FILE` | TOML configuration; default: `.locmetryrc` in the scan root, else built-in settings. The scan root stays unchanged. |
| `--exclude GLOB` | Add a root-relative exclusion; repeatable. |
| `--hidden true\|false` | Whether hidden files and directories are measured. |
| `--respect-ignore-files true\|false` | Whether `.gitignore`, `.ignore`, and `.tokeignore` limit the selection. |
| `--no-user-config` | Ignore personal defaults in `~/.config/locmetry/config.toml`. |
| `--format table\|json\|markdown` | Stdout format; default: `table` on a terminal, `json` otherwise. |
| `--sort code\|name` | Row order of the table and Markdown output; default: `code`, largest first. |
| `--depth N` | Folder levels in the Markdown folder table; default: 2. |
| `--output FILE` | Also write the full JSON report to `FILE`, outside the scan root; `-` prints it instead of the headline. |
| `--files` | Add per-file records to the report. |
| `--categories` | Add a table of source code per purpose and category to the terminal table. |
| `--schema config\|user-config\|report\|workspace` | Print the JSON Schema of `.locmetryrc`, of personal defaults, of the JSON headline and report, or of a workspace file, and exit. |

`locmetry workspace` takes `--root DIR`, repeatable, or `--file FILE` instead, plus `--no-user-config` and the output options from `--format` to `--categories`; see [Measure several repositories together](#measure-several-repositories-together).

## Measure several repositories together

`locmetry workspace` measures sibling repositories as one virtual monorepo. Each repository is measured with its own `.locmetryrc`, the repositories run in parallel, and the results merge into one report whose top-level folders are the repositories:

```sh
locmetry workspace --root ../web --root ../api
```

Roots are relative to the current directory or absolute. To avoid repeating them, list them in a workspace file, where they are relative to the file itself or absolute, and name it with `--file`; `locmetry --schema workspace` describes the format:

```toml
version = 1
roots = ["web", "api", "/srv/shared/billing"]
```

```sh
locmetry workspace --file repos.toml
```

- **Each repository keeps its policy:** `--config`, `--exclude`, `--hidden`, and `--respect-ignore-files` are rejected, because each repository's own configuration decides them, and options must follow `workspace`.
- **Paths name the repository:** every path is prefixed with its repository's folder name, such as `web/src/index.ts`. Two repositories with the same folder name, a root given twice, or one root inside another fail before measuring.
- **Repositories as a dimension:** `summary.members` holds each repository's totals by purpose, category, kind, and language, equal to a regular run of that repository. The table shows one row per repository with its test lines per production line and its share of the workspace, and the Markdown report adds grids of source categories and languages by repository.
- **Findings stay with their repository:** `metadata.members` lists each repository's root, configuration path, scope, coverage counts, unused entries, and unused exclusions; table notes name the repository, as in `ghost (web)`, and list repositories with unclaimed source or unreadable files.
- **All or nothing, always the same:** if one repository fails, for example with an invalid configuration, the run fails and names it; the order of the roots never changes the report.

## Configure with an agent

`locmetry --schema config` prints the JSON Schema of `.locmetryrc`. Besides keys, types, and defaults, its descriptions carry the semantics an agent needs to write a configuration: the classification order, when to use `[[overrides]]` instead of `[[rules]]`, what turning off built-in conventions or [build modules](#build-modules) changes, and the exact conventions, build-module layouts, and language names of the installed version. `--schema user-config` describes personal defaults, and `--schema report` the JSON headline and report, including per-file records.

```sh
locmetry --schema config | jq -r .description          # classification order, globs, workflow
locmetry --schema config | jq '."$defs".Classification' # built-in stages and their switches
```

A one-line task is enough, because `--help` leads to the schema and the schema states the workflow:

> Create a locmetry configuration for this repository.

The agent measures, reads `summary.unclassified_folders`, inspects the project for evidence, writes entries with a `reason`, and measures again until no folder is left undecided.

The schema checks keys, types, and enums. Locmetry itself also rejects invalid globs and duplicate IDs when it loads the file, so a final `locmetry` run validates the result.

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

Its `.locmetryrc` states only what conventions and build files cannot know: the vendored SDK is out of scope, and `scripts/` is maintained tooling.

```toml
version = 1

[scope]
exclude = ["third_party/**"]

[[rules]]
id = "tooling"
match = ["scripts/**"]
purpose = "production"
category = "tooling"
```

Each file gets the first match of five stages:

1. `[[overrides]]`, for decisions that must beat everything;
2. built-in test conventions, then built-in build conventions;
3. `[[rules]]`;
4. [build modules](#build-modules): the source layout that a build file declares, here in `backend` and `web`;
5. the fallback, `unclassified` by default.

Only this stage order and the order of entries within each list matter, not where the tables appear in the file. User defaults and command-line flags may add exclusions or switch scope settings but never classify, and the same files and configuration always give the same report; [PRODUCT.md](PRODUCT.md#matching-and-precedence) defines the layers and tie-breaks.

So `OrderServiceTest.kt` and `cart.test.ts` are tests even though they sit inside the build modules' source trees. The Gradle scripts are `build-tooling`, and `gradlew` is a `third-party-wrapper`. The prototype falls through to `unclassified`, where the review list surfaces it as unclaimed. A rule that gives it `purpose = "unclassified"` with a `category` such as `prototype` parks it on purpose: it still counts as unclassified but leaves the review list, as `gradlew` does. An optional `[categories]` table declares the category names entries may use, each with its meaning, so a typo fails instead of becoming a new category. [PRODUCT.md](PRODUCT.md#project-configuration) documents every key, glob semantics, and `[[languages.mappings]]`.

### Build modules

A build module is a folder whose build file declares where its source lives. Build systems call this unit a module, project, package, or crate; locmetry says build module throughout. It has nothing to do with modules or packages inside a language, such as a Python file or a Rust `mod`. Wherever one of these build files is among the measured files, the build system's default production layout in its folder counts as production with category `implementation`:

| Build file | Production source in its folder |
| --- | --- |
| `package.json` (npm) | `src/**` |
| `build.gradle`, `build.gradle.kts` (Gradle) | `src/main/**`, Kotlin Multiplatform `src/*Main/**` |
| `pom.xml` (Maven) | `src/main/**` |
| `Cargo.toml` (Cargo) | `src/**` |
| `go.mod` (Go) | every `.go` file |
| `pyproject.toml`, `setup.py`, `setup.cfg` (Python) | `src/**`, or the top-level packages that contain `__init__.py` |
| `lakefile.lean`, `lakefile.toml` (Lake) | every `.lean` file |
| `Package.swift` (Swift) | `Sources/**` |

In the sample, `backend` and `web` are build modules, so `backend/src/main/**` and `web/src/**` are production without a rule.

- **Tests stay tests:** test conventions run first, so `src/test/**` or `cart.test.ts` inside a production layout still count as tests.
- **Nested modules:** the innermost build module containing a file decides, and build files sharing one folder, such as `build.gradle.kts` and `package.json`, combine their layouts; where those layouts overlap, the build system whose detector name sorts first is recorded.
- **Only file names are read:** locmetry neither runs the build nor parses the build file, so source directories that a build file reconfigures need a rule.
- **Rules win:** `[[rules]]` run before build modules, so one rule corrects a folder that detection gets wrong. The flip side: a broad rule also takes module source, so match only the folders a rule is about. `builtin_modules = false` under `[classification]` turns detection off.
- **Traceable:** the table and the JSON headline (`summary.build_modules`) show each build system's modules and the code they classified, the Markdown report lists every module, and each file record from `--files` names the detector that decided it, such as `builtin:gradle-module`.

### Terminal table

Plain `locmetry` on a terminal prints the table shown [above](#what-you-see):

- language rows, largest first;
- top-level folders, whose rows add up to the total, each with `Test/prod`, its test code lines per production code line;
- under each table, a legend stating how its columns add up, such as Production + Test + Unclassified = Code;
- notes on content that is not source, the build modules that decided production, Git submodules, which are often other teams' code, the folders that hold unclaimed source, configuration entries that decided no file, and files that could not be counted.

`--sort name` lists rows alphabetically instead, in the table and in Markdown. `--categories` adds a table of source code per `purpose:category`, such as `production:tooling`, to see what production consists of.

### JSON headline for agents and scripts

When stdout is not a terminal, `locmetry` prints a compact JSON headline, under 3 KB for this project. Abridged:

```json
{
  "schema_version": "1",
  "project": "acme-orders",
  "metadata": {
    "root": "/work/acme-orders",
    "locmetry_version": "0.1.5",
    "scope": {
      "exclude": ["**/.git/**", "**/node_modules/**", "**/.build/**", "**/.lake/**", "third_party/**", ".locmetryrc"]
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
    },
    "build_modules": {
      "gradle": { "modules": 1, "code": 42 },
      "npm": { "modules": 1, "code": 31 }
    },
    "unclassified_folders": [
      { "folder": "prototypes", "files": 1, "code": 8 }
    ]
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
```

`--files` adds one record per file with its language, kind, purpose, category, deciding rule, and counts, for debugging rules. `--output -` prints the full report to stdout instead.

### Markdown report

`locmetry --format markdown` renders a report to save or paste. Besides purpose and category tables, it contains other counted content, source languages, a recursive folder breakdown to `--depth N` levels, the review list, and coverage. Abridged:

```markdown
## Source categories

| Category | Files | **Code** | Comments | Blanks | Lines |
| --- | ---: | ---: | ---: | ---: | ---: |
| production:implementation | 5 | 73 | 3 | 15 | 91 |
| test:test | 2 | 26 | 0 | 6 | 32 |
| production:tooling | 1 | 12 | 1 | 6 | 19 |
| production:build-tooling | 2 | 9 | 0 | 1 | 10 |
| unclassified:unclassified | 1 | 8 | 1 | 2 | 11 |
| unclassified:third-party-wrapper | 1 | 1 | 2 | 0 | 3 |

## Folder breakdown

| Folder | Production | Test | Unclassified | **Total source code** | Test/prod |
| --- | ---: | ---: | ---: | ---: | ---: |
| . | 94 | 26 | 9 | 129 | 0.28 |
| backend | 49 | 16 | 0 | 65 | 0.33 |
| backend/src | 42 | 16 | 0 | 58 | 0.38 |
| prototypes | 0 | 0 | 8 | 8 | - |
| scripts | 12 | 0 | 0 | 12 | 0.00 |
| web | 31 | 10 | 0 | 41 | 0.32 |
| web/src | 31 | 10 | 0 | 41 | 0.32 |

## Build modules

| Build system | Modules | Code |
| --- | ---: | ---: |
| gradle | 1 | 42 |
| npm | 1 | 31 |

## Source requiring review

1 file of unclassified source that no rule, convention, or build module claims. Code an entry or convention leaves unclassified on purpose is not listed.

| Folder | Files | Code |
| --- | ---: | ---: |
| prototypes | 1 | 8 |

| Path | Code | Category |
| --- | ---: | --- |
| prototypes/pricing_experiment.py | 8 | unclassified |

## Coverage

Files selected: 17; counted: 16, including 2 through language mappings; not recognized: 1; recognized but unreadable: 0. Their lines are missing, not zero. Counts describe the working copy, including untracked files, not a commit.
```

## Continuous integration

The headline makes guardrails one-liners. `jq -e` exits nonzero when the expression is false:

```sh
# Fail when more than 50 lines of source are still unclaimed
locmetry | jq -e '([.summary.unclassified_folders[].code] | add // 0) <= 50'

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

The table and Markdown group thousands with commas, such as `12,345`, while JSON keeps plain integers for scripts.

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

Licensed under the [MIT License](LICENSE). Release archives and the Homebrew installation include `THIRD-PARTY-NOTICES.md` with the licences of the bundled libraries. Line counting uses the embedded [Tokei](https://github.com/XAMPPRocky/tokei) library.
