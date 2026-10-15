<img src="docs/assets/locmetry.webp" alt="locmetry logo" width="96">

# locmetry

**Lines of code, split by purpose: production, tests, tooling, and what nobody claims.** `locmetry` measures selected files that its counters can recognize and read, gives each counted file a purpose and category, and records the rule that decided it. Coverage names files whose counts are unavailable.

- **Useful without configuration:** test conventions and build files (npm, Gradle, Maven, Cargo, Go, Python, Lake, Swift) recognize common source and test layouts out of the box.
- **Flexible, reviewable configuration:** a short `.locmetryrc` states only what conventions cannot know: what is out of scope, which folders are tooling or examples, which archived code must not count as tests, your own categories, and languages for unknown file types. The precedence is fixed and documented, so classification decisions can be traced to effective policy.
- **Built for coding agents:** `locmetry --schema config` explains the configuration and its semantics from the binary alone. In selected later blind trials, agents without these docs reached 99% or greater agreement with human-reviewed purpose classifications by code line ([population and rounds](docs/research/real-project-trials.md#agent-configuration-trials)). This measures classification agreement, not parser accuracy or performance on arbitrary repositories.
- **JSON for agents and scripts:** a headline small enough for an agent's context, with totals by purpose, category, content kind, and language, per top-level folder and build system, plus the folders that still need a decision. `--output` writes a full report with additive breakdown rows; add `--files` for per-file records. `--schema report` describes the structure and semantics for further queries.
- **Ready-made metrics:** the tables show test lines per production line for every folder, and a workspace adds each repository's ratio and share. The JSON keeps exact counts, so any other ratio, such as a test-share guard in CI, is one `jq` expression.
- **Several repositories as one:** `locmetry workspace` measures sibling repositories, each with its own configuration, as one virtual monorepo with a row per repository.
- **Three outputs from one measurement:** a terminal table for people, JSON for agents and scripts, and Markdown for reviews and wikis.

It is a self-contained executable for macOS arm64 and Linux x86_64 and aarch64, with no separately installed runtime. Linux releases are statically linked; macOS uses operating-system libraries.

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

- **No language list to maintain:** recognized programming languages count, while JSON, YAML, Markdown, and other data or documentation stay out of the source totals.
- **Production from the build:** `backend/build.gradle.kts` and `web/package.json` mark their source trees as production, with no rule, and the table names both build systems.
- **Tests found by convention:** test files count as tests even inside the application's source tree.
- **Scope as policy:** the vendored SDK is excluded by one line of configuration.
- **Nothing silently promoted:** build scripts and tooling stay apart from implementation, and code no rule claims is reported for review instead of being counted as production.

## Use cases

- **Understand an unfamiliar repository:** see in one screen which languages matter, where the tests live, and which code nobody claims.
- **Track test investment:** follow the test-to-production ratio per folder and over time, without hand-maintained exclusion lists.
- **Size a codebase honestly:** estimate a migration, a rewrite, an audit, an acquisition, or an outsourcing quote on maintained code only. Use configuration to exclude or label vendored, generated, and experimental code; these are not automatically identified.
- **Guard quality in CI:** fail a build when unclaimed code grows or the test share drops, with one `jq` expression ([example](#continuous-integration)).
- **Compare parts of a monorepo:** the folder rows and the additive breakdown table compare packages, services, or layers by purpose.
- **Measure several repositories as one:** `locmetry workspace` merges sibling repositories, each measured with its own configuration, into one virtual monorepo whose folder rows compare the repositories ([how](#measure-several-repositories-together)).
- **Watch a codebase change:** save full JSON reports and use `locmetry compare before.json after.json` to see net change by purpose, language, folder and workspace member, including package/root arrivals and departures ([how](#compare-saved-measurements)). Reports do not establish Git revisions.
- **Let an agent configure a project:** a coding agent working in a repository writes its `.locmetryrc` itself. `locmetry --help` leads it to `--schema config`, which explains the configuration and its semantics, and the measurement shows which folders still need a decision ([how](#configure-with-an-agent)).
- **Give coding agents a reliable answer:** a compact JSON headline fits an agent's context, and follow-up questions are one `jq` query over the report file instead of improvised `find | xargs wc -l` pipelines.
- **Report to people:** paste the Markdown report into a pull request, an architecture review, or a wiki page.

## Features

### Counting for more than 300 languages

Code, comment, and blank lines per file, including code embedded in other languages, such as snippets in Markdown. Locmetry embeds [Tokei](https://github.com/XAMPPRocky/tokei), with a local Handlebars correction and a notebook decoder that counts each extracted cell once. [Counting semantics](docs/measurement.md#counting) describes these boundaries.

### Purpose, category, and provenance per file

Every counted file gets a purpose, `production`, `test`, or `unclassified`, an optional category such as `implementation`, `tooling`, `build-tooling`, or `example`, and the rule that decided it. For the [sample project](#a-tour-of-a-sample-project), `locmetry --files --output - | jq …` lists:

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

A selected npm, Gradle, Maven, Cargo, Go, Python, Lake, or Swift descriptor identifies a [build module](#build-modules). Locmetry infers production from that system's conventional layout, without reading manifest targets or scripts. Custom layouts need configuration. The table says which detectors decided:

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

Optional [personal defaults](docs/PRODUCT.md#user-defaults) add exclusions for every project, and an exclusion that excludes nothing is reported, so a misspelled path does not go unnoticed.

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

Files that could not be recognized, read or parsed are listed, never reported as zero lines:

```text
 Not counted: 1 unrecognized file (png: 1); missing lines are not zero.
```

Invalid or unsupported notebook content appears in `metadata.coverage.parse_failures` as `{path, reason, message}` and produces partial output with exit code 2, as unreadable files do. New reports always include this array, including when empty; its absence in older reports means failures were not recorded. Workspace member coverage adds `parse_failed_files`. [Notebook limits and compatibility](docs/measurement.md#notebooks) explains supported versions, language selection and older schemas.

### Reproducible and deterministic

With the same selected file content, effective policy and Locmetry version, counts and ordering are deterministic. Personal exclusions and language mappings, repository/parent/global ignore rules, and CLI scope options can change what is measured. Reports record effective settings, but do not fingerprint source content, ignore files or executable builds; timestamps and contextual names/paths can differ. [PRODUCT.md](docs/PRODUCT.md#matching-and-precedence) defines the guarantee and precedence.

### Self-describing configuration

`locmetry --schema config` prints a JSON Schema that explains every setting, the classification order, and the built-in conventions, so an agent can write the configuration without reading these docs ([how](#configure-with-an-agent)). `--schema report`, `--schema comparison`, `--schema explanation`, `--schema user-config`, and `--schema workspace` describe the other documents.

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
curl -fsSL "https://github.com/mbackschat/locmetry/releases/download/v0.1.6/locmetry-v0.1.6-$target.tar.gz" | tar -xz
install "locmetry-v0.1.6-$target/locmetry" ~/.local/bin/
```

Or build from source with a Rust toolchain:

```sh
cargo install --locked --git https://github.com/mbackschat/locmetry --tag v0.1.6
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

Personal defaults for every project, such as extra exclusions or language mappings, go in `~/.config/locmetry/config.toml` (or under `$XDG_CONFIG_HOME`). They never change classification, reports name the file, and `--no-user-config` ignores it, for example in CI. [PRODUCT.md](docs/PRODUCT.md#user-defaults) lists what they may set.

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
| `--schema config\|user-config\|report\|comparison\|explanation\|workspace` | Print the JSON Schema of `.locmetryrc`, of personal defaults, of the JSON headline and report, of a net-change comparison, of a path explanation, or of a workspace file, and exit. |

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

## Explain one file

```sh
locmetry explain src/main.rs
locmetry explain src/main.rs --root /path/to/project --format markdown
locmetry explain api/src/main.rs --workspace repos.toml
locmetry --schema explanation
```

The explanation uses the same policy as measurement and reports either the counted file's parser, winning rule, matching patterns and module evidence, or why it has no counts. It counts only the target, while selected names still determine module layouts. Targets are root-relative; workspace targets start with the member name. Single-root configuration and scope flags are available, plus `--no-user-config` in either mode.

Ignore and hidden filtering are checked with single-setting selection probes; the walker does not expose a winning ignore-file line. Unknown causes remain explicit. Exit 2 means the target's counts are unavailable; excluded or unrecognized targets are successful explanations with exit 0. [Troubleshooting guide](docs/troubleshooting.md) gives a runnable example, state meanings and policy/path details.

## Compare saved measurements

Save full reports with `--output`; the default JSON headline lacks the detail needed for comparison. The command reads only those reports, so the source checkouts need not exist:

```sh
locmetry --root ./service --output ./before.json > /dev/null
# After changing the source or its measurement policy:
locmetry --root ./service --output ./after.json > /dev/null
locmetry compare before.json after.json
locmetry compare before.json after.json --format markdown
locmetry compare before.json after.json --output net-change.json
locmetry --schema comparison
```

Net change is **after minus before**, in argument order. A terminal gets a table, redirected stdout gets a JSON headline, and `--output -` prints full JSON. The headline includes presence counts; full output includes every folder, package and breakdown row. `--output FILE` must not overwrite an input, including symlink/hard-link aliases. Aggregate comparison does not require `--files`; per-file comparison is not implemented.

The same command compares workspace exports, even when their root lists differ:

```sh
locmetry workspace --file repos.toml --output before-workspace.json > /dev/null
# After changing code or the workspace root list:
locmetry workspace --file repos.toml --output after-workspace.json > /dev/null
locmetry compare before-workspace.json after-workspace.json --format markdown
```

Workspace totals separate changes inside retained roots from counts entering and leaving the measurement. Names match members; renamed members appear as a departure and arrival. A folder or package disappearing from a report does not prove source deletion. Removing only a build descriptor can change purpose and package presence while source LOC stays unchanged.

Inspect `findings` before interpreting growth: versions, policy, module detection and coverage can differ. Unreadable files or parsing failures make the comparison partial (exit 2); missing historical failure evidence produces a finding without exit 2 by itself. Recorded counts exclude unavailable lines, whose size remains unknown. [The contract](docs/PRODUCT.md#net-change) defines the output and limits; [report compatibility](docs/report-compatibility.md) lists eligible older exports.

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
4. [build modules](#build-modules): conventional source layouts inferred from selected descriptors, here in `backend` and `web`;
5. the fallback, `unclassified` by default.

Only this stage order and the order of entries within each list matter, not where the tables appear in the file. Personal settings cannot assign purpose or category, but their language mappings can change recognition and content kind. Command-line flags can change scope; [PRODUCT.md](docs/PRODUCT.md#matching-and-precedence) defines the layers and tie-breaks.

So `OrderServiceTest.kt` and `cart.test.ts` are tests even though they sit inside the build modules' source trees. The Gradle scripts are `build-tooling`, and `gradlew` is a `third-party-wrapper`. The prototype falls through to `unclassified`, where the review list surfaces it as unclaimed. A rule that gives it `purpose = "unclassified"` with a `category` such as `prototype` parks it on purpose: it still counts as unclassified but leaves the review list, as `gradlew` does. An optional `[categories]` table declares the category names entries may use, each with its meaning, so a typo fails instead of becoming a new category. [PRODUCT.md](docs/PRODUCT.md#project-configuration) documents every key, glob semantics, and `[[languages.mappings]]`.

To measure a root from another directory, discover its policy with `--root`. An explicit config replaces only project discovery; `--no-user-config` separately removes personal defaults:

```sh
locmetry --root ../service
locmetry --root ../service --config ./policies/service.toml --no-user-config
```

Both commands match patterns relative to `../service`, not the configuration's directory. Exclusions from all active layers form a union; moving one later cannot re-include a file. Ordered rules and mappings instead use the first match. [Workspace members](#measure-several-repositories-together) discover their own policy independently.

### Build modules

A build module is a folder with a selected build descriptor that triggers conventional source-layout inference. Build systems call this unit a module, project, package, or crate; locmetry says build module throughout. It has nothing to do with modules or packages inside a language, such as a Python file or a Rust `mod`. Wherever one of these build files is among the measured files, the build system's default production layout in its folder counts as production with category `implementation`:

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
    "locmetry_version": "0.1.6",
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
      "unread": [],
      "parse_failures": []
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

Summary count objects omit zero counts; this does not apply to missing coverage evidence or optional detail. The report contract is in [PRODUCT.md](docs/PRODUCT.md#reporting-and-coverage).

### Report file for any question

`--output FILE` writes the full report next to the headline: the summary, a review list, and an additive `breakdown` table with one row per folder, kind, purpose, category, and language. Each row sits on its own line:

```json
{"folder":"backend/src/main/kotlin/acme/orders","kind":"source","purpose":"production","category":"implementation","language":"Kotlin","files":3,"code":42,"comments":2,"blanks":11,"lines":55},
{"folder":"backend/src/test/kotlin/acme/orders","kind":"source","purpose":"test","category":"test","language":"Kotlin","files":1,"code":16,"comments":0,"blanks":4,"lines":20},
```

Because rows are additive, recorded dimensions can be filtered and summed:

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

Capture the measurement before applying a policy check, preserving Locmetry's exit status. This script refuses partial reports, fails when more than 50 source lines remain unclaimed, and prints the test/production ratio only when production code is nonzero. `jq -e` fails when the policy expression is false:

```sh
set -eu
report=$(locmetry --format json) || exit "$?"
printf '%s\n' "$report" | jq -e '([.summary.unclassified_folders[].code] | add // 0) <= 50'
printf '%s\n' "$report" | jq -r '
  (.summary.source_purpose // {}) as $p |
  if ($p.production.code // 0) == 0 then
    "test/production: n/a (no production code)"
  else
    "test/production: \(($p.test.code // 0) * 100 / $p.production.code | floor)%"
  end'
```

[CI executes this block](scripts/check-doc-examples.sh) against empty, healthy, partial, invalid-configuration and policy-failing projects. The Cargo tests separately guard its JSON and exit-code assumptions without requiring `jq`.

Exit codes:

| Code | Meaning |
| --- | --- |
| 0 | Complete report. |
| 2 | Partial report: unreadable files, invalid/unsupported notebook content, or backend inaccuracy. |
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

[cloc](https://github.com/AlDanial/cloc) provides source comparison and detailed filtering diagnostics; [scc](https://github.com/boyter/scc) provides additional analytics, configuration and integrations. Locmetry focuses on a combined contract: reviewable scope, whole-file purpose/category with provenance, source/data/documentation separation, explicit coverage, and independently configured workspace members. The pinned [capability comparison](docs/research/tool-capability-comparison.md) records the evidence and differences; it is not an accuracy or performance ranking.

Known limits: classification is per file, so a file mixing production code and inline tests counts as one purpose. Git is not invoked, so a measurement describes the working copy rather than a commit. [Measurement semantics and limits](docs/measurement.md) lists the details.

## Documentation

- [Product vision and requirements](docs/PRODUCT.md): configuration contract, report schema, and guarantees.
- [Measurement semantics and limits](docs/measurement.md): selection and counting details and known limits.
- [Explain a file](docs/troubleshooting.md): selection, parser and classification diagnostics.
- [Saved report compatibility](docs/report-compatibility.md): version-1 field history, validation and evidence required for comparison.
- [Architecture and distribution](docs/tokei-integration.md): embedded counting engine, release binaries, and the Homebrew formula.
- [Research](docs/research/README.md): counter capabilities and classification findings.
- [Development plan](docs/PLAN.md): remaining work and completion evidence.
- [Measurement quality and net change work plan](docs/plans/measurement-quality-and-net-change.md): accepted sequence, rationale, implementation checks and review checkpoints; features remain planned until delivered.
- [Repository guidelines](CLAUDE.md): contributor conventions and checks; `AGENTS.md` links to this file.

## License

Licensed under the [MIT License](LICENSE). Release archives and the Homebrew installation include `THIRD-PARTY-NOTICES.md` with the licences of the bundled libraries. Line counting uses the embedded [Tokei](https://github.com/XAMPPRocky/tokei) library.
