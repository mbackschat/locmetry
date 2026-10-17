<img src="docs/assets/locmetry.webp" alt="locmetry logo" width="96">

# locmetry

The latest stable release is **0.4.0**. The examples below describe the current checkout; [output routing](docs/PRODUCT.md#output-routing) defines destination and format behavior.

**Lines of code, split by purpose: production, tests, tooling, and what nobody claims.** `locmetry` measures selected files that its counters can recognize and read, gives each counted file a purpose and category, and records the rule that decided it. Coverage names files whose counts are unavailable.

- **Useful without configuration:** test conventions and build files (npm, Gradle, Maven, Cargo, Go, Python, Lake, Swift) recognize common source and test layouts out of the box.
- **Flexible, reviewable configuration:** a short `.locmetryrc` states only what conventions cannot know: what is out of scope, which folders are tooling or examples, which archived code must not count as tests, your own categories, and languages for unknown file types. The precedence is fixed and documented, so classification decisions can be traced to effective policy.
- **Built for coding agents:** `locmetry --guide configure` teaches evidence-based setup and review; `locmetry --schema config` explains the configuration and its semantics from the binary alone. In selected later blind trials, agents without these docs reached 99% or greater agreement with human-reviewed purpose classifications by code line ([population and rounds](docs/research/real-project-trials.md#agent-configuration-trials)). This measures classification agreement, not parser accuracy or performance on arbitrary repositories.
- **JSON for agents and scripts:** a headline small enough for an agent's context, with totals by purpose, category, content kind, and language, per top-level folder and build system, plus the folders that still need a decision. `--output` defaults to a full JSON report with additive breakdown rows; add `--files` for per-file records. `--schema report` describes the structure and semantics for further queries.
- **Ready-made metrics:** `--folders` shows test lines per production line for every folder, and a workspace adds each repository's ratio and share. The JSON keeps exact counts, so any other ratio, such as a test-share guard in CI, is one `jq` expression.
- **Several repositories as one:** `locmetry workspace` measures sibling repositories, each with its own configuration, as one virtual monorepo with a row per repository.
- **Three outputs from one measurement:** a terminal table for people, JSON for agents and scripts, and Markdown for reviews and wikis.

It is a self-contained executable for macOS arm64 and Linux x86_64 and aarch64, with no separately installed runtime. Linux releases are statically linked; macOS uses operating-system libraries.

## What you see

Run `locmetry` in a project. The example uses an illustrative measurement date. For the [sample project](#a-tour-of-a-sample-project) used throughout this page:

```text
$ locmetry
 acme-orders · Source code
 Language    Files  Lines  Code │ Comments  Blanks │ Production  Test  Unclassified
 ───────────────────────────────┼──────────────────┼───────────────────────────────
 Kotlin          6     85    67 │        2      16 │         51    16             0
 TypeScript      2     26    21 │        1       4 │         11    10             0
 Python          2     30    20 │        2       8 │         12     0             8
 TSX             1     22    20 │        0       2 │         20     0             0
 Shell           1      3     1 │        2       0 │          0     0             1
 ───────────────────────────────┼──────────────────┼───────────────────────────────
 Total          12    166   129 │        7      30 │         94    26             9
 Lines = Code + Comments + Blanks; Production + Test + Unclassified = Code.

Measured at: 2026-10-17 12:00:00 UTC
```

Add `--folders` for a folder table, or `--folder-rows N` for adaptive folder detail. `--verbose` adds notes independently.

What the table already handles for you:

- **No language list to maintain:** recognized programming languages count, while JSON, YAML, Markdown, and other data or documentation stay out of the source totals.
- **Production from the build:** `backend/build.gradle.kts` and `web/package.json` mark their source trees as production, with no rule, and `--verbose` names both build systems.
- **Tests found by convention:** test files count as tests even inside the application's source tree.
- **Scope as policy:** the vendored SDK is excluded by one line of configuration.
- **Nothing silently promoted:** build scripts and tooling stay apart from implementation, and code no rule claims stays counted as Unclassified; `--verbose` locates it for optional review.

## Use cases

- **Understand an unfamiliar repository:** see in one screen which languages matter, where the tests live, and which code nobody claims.
- **Track test investment:** follow the test-to-production ratio per folder and over time, without hand-maintained exclusion lists.
- **Size a codebase honestly:** estimate a migration, a rewrite, an audit, an acquisition, or an outsourcing quote on maintained code only. Use configuration to exclude or label vendored, generated, and experimental code; these are not automatically identified.
- **Guard quality in CI:** fail a build when unclaimed code grows or the test share drops, with one `jq` expression ([example](#continuous-integration)).
- **Compare parts of a monorepo:** `--folders` and the additive JSON breakdown table compare packages, services, or layers by purpose.
- **Measure several repositories as one:** `locmetry workspace` merges sibling repositories, each measured with its own configuration, into one virtual monorepo whose folder rows compare the repositories ([how](#measure-several-repositories-together)).
- **Watch a codebase change:** save full JSON reports and use `locmetry compare before.json after.json` to see net change by purpose, language, folder and workspace member, including package/root arrivals and departures ([how](#compare-saved-measurements)). Reports do not establish Git revisions.
- **Let an agent configure a project:** a coding agent working in a repository writes its `.locmetryrc` itself. `locmetry --help` leads it to `--guide configure` and `--schema config`, which explains the configuration and its semantics, and verbose tables or JSON show the unclaimed folders for optional inspection ([how](#configure-with-an-agent)).
- **Give coding agents a reliable answer:** a compact JSON headline fits an agent's context, and follow-up questions are one `jq` query over the report file instead of improvised `find | xargs wc -l` pipelines.
- **Report to people:** paste the Markdown report into a pull request, an architecture review, or a wiki page.

## Features

### Counting for more than 300 languages

Code, comment, and blank lines per file, including code embedded in other languages, such as snippets in Markdown. Locmetry embeds [Tokei](https://github.com/XAMPPRocky/tokei), with a local Handlebars correction and a notebook decoder that counts each extracted cell once. Readable templates with invalid encoding are reported as parse failures rather than unreadable files. [Counting semantics](docs/measurement.md#counting) describes these boundaries.

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

A selected npm, Gradle, Maven, Cargo, Go, Python, Lake, or Swift descriptor identifies a [build module](#build-modules). Locmetry infers production from that system's conventional layout, without reading manifest targets or scripts. Custom layouts need configuration. `locmetry --verbose` adds a note showing which detectors decided:

```text
 Production from build modules: gradle 42 code lines in 1 module, npm 31 code lines in 1 module.
```

### Content kinds

Source, data, documentation, and other content are counted separately, so a large JSON fixture or a README never inflates source totals. The default table shows counted source only and omits non-source totals; `locmetry --verbose` includes them:

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

Optional [personal defaults](docs/PRODUCT.md#user-defaults) add exclusions for every project, and `--verbose`, Markdown and JSON report an exclusion that excludes nothing, so a misspelled path does not go unnoticed.

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

Read/parse failures remain visible in default tables under `Not counted:`, along with the backend-incomplete warning and stderr diagnostic; partial results retain exit 2. Unrecognized file types alone are routine findings: default tables omit them, while `locmetry --verbose` includes a separate neutral note. Omitted findings do not mean zero lines:

```text
 Unrecognized files: 1 unrecognized file (png: 1); missing lines are not zero.
```

Invalid or unsupported notebook content appears in `metadata.coverage.parse_failures` as `{path, reason, message}` and marks the measurement partial (see [command outcomes](docs/PRODUCT.md#command-outcomes)). New reports always include this array, including when empty; its absence in older reports means failures were not recorded. Workspace member coverage adds `parse_failed_files`. [Notebook limits and compatibility](docs/measurement.md#notebooks) explains supported versions, language selection and older schemas.

### Reproducible and deterministic

With the same selected file content, effective policy and Locmetry version, counts and ordering are deterministic. Personal exclusions and language mappings, repository/parent/global ignore rules, and CLI scope options can change what is measured. Reports record effective settings, but do not fingerprint source content, ignore files or executable builds; timestamps and contextual names/paths can differ. [PRODUCT.md](docs/PRODUCT.md#matching-and-precedence) defines the guarantee and precedence.

### Self-describing configuration

`locmetry --schema config` prints a JSON Schema that explains every setting, the classification order, and the built-in conventions, so an agent can write the configuration without reading these docs ([how](#configure-with-an-agent)). `--schema report`, `--schema comparison`, `--schema explanation`, `--schema user-config`, and `--schema workspace` describe the other documents.

### On-demand guidance for coding agents

An agent can discover configuration workflows, diagnostics and reporting recipes from the installed binary: `locmetry --help` points to `--guide configure`, `--guide selection`, `--guide reports` and `--guide compare`. Each guide carries the executable version and its source path, with examples and limits. Guides need no separate skill, repository checkout or network access; JSON schemas supply the exact fields and semantics. Start with [Configure with an agent](#configure-with-an-agent).

### Workspaces

`locmetry workspace --root web --root api` measures several repositories, each with its own configuration, in parallel, and merges them into one report with a row per repository:

```text

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

### Stable release (v0.4.0)

With Homebrew on macOS arm64 or Linux x86_64 and aarch64, which installs the prebuilt binary:

```sh
brew install mbackschat/tap/locmetry
```

On Linux distributions with an older toolchain, such as Ubuntu 22.04, Homebrew also installs its own gcc and glibc for this formula; locmetry itself is a static binary and uses neither.

The same binaries are on the [releases page](https://github.com/mbackschat/locmetry/releases):

```sh
# Linux targets: x86_64-unknown-linux-musl or aarch64-unknown-linux-musl
target=aarch64-apple-darwin
curl -fsSL "https://github.com/mbackschat/locmetry/releases/download/v0.4.0/locmetry-v0.4.0-$target.tar.gz" | tar -xz
install "locmetry-v0.4.0-$target/locmetry" ~/.local/bin/
```

Or build from source with a Rust toolchain:

```sh
cargo install --locked --git https://github.com/mbackschat/locmetry --tag v0.4.0
```

### Local checkout

From this checkout, build or install the binary:

```sh
cargo build --locked
cargo install --locked --path .
locmetry --version
```

This checkout identifies as `0.4.0`. Producer version differences are recorded by comparisons. Installing a checkout does not publish a release.

## Quick start

```sh
cd your/project
locmetry                      # table on a terminal
locmetry ../another-project   # measure a directory without changing into it
locmetry --format markdown    # full human report
locmetry | jq .summary        # piped: JSON headline
```

The optional directory argument is equivalent to `--root DIR`; use one or the other. Bare `compare`, `workspace`, `explain` and `help` select commands; use `locmetry ./compare` or `locmetry --root compare` to measure a directory with a command's name. In scripts, prefer `locmetry --root "$directory"` so directory names cannot select commands. Command options go after the command name.

Run `locmetry` for source counts by language; add `--folders` for the folder table or `--verbose` to inspect classification, scope and coverage notes. For optional project configuration, use `locmetry --guide configure`. Use `locmetry --help` or `locmetry COMMAND --help` for options; `--guide selection|reports|compare` provides focused workflows and caveats. Guides print the installed version’s Markdown without a network connection. Help wraps to the detected terminal width and keeps continuation lines indented; see [help presentation](docs/PRODUCT.md#human-presentation).

Without configuration, built-in exclusions, conventions, and language mappings apply. Add a `.locmetryrc` to the project root when the conventions cannot know something, such as which folders are vendored, generated, tooling, or experimental. Command-line options override it:

```sh
locmetry --exclude 'generated/**' --hidden false
```

Personal defaults for every project, such as extra exclusions or language mappings, go in `~/.config/locmetry/config.toml` (or under `$XDG_CONFIG_HOME`). They never change classification, reports name the file, and `--no-user-config` ignores it, for example in CI. [PRODUCT.md](docs/PRODUCT.md#user-defaults) lists what they may set.

## Command-line options

| Option | Effect |
| --- | --- |
| `DIR` or `--root DIR` | Directory to measure; mutually exclusive. Default: the current directory. |
| `--folders` | Add the folder table to single-root table output, including `--subfolder`; warns without changing content in other measurement formats. |
| `--folder-rows N` | Show the Folder table with adaptive detail using a positive row budget; implies `--folders`. Single-root table output only. |
| `--subfolder DIR` | Measure one root-relative folder and descendants, retaining project policy and module context; single-root mode only. |
| `--config FILE` | TOML configuration; default: `.locmetryrc` in the scan root, else built-in settings. The scan root stays unchanged. |
| `--exclude GLOB` | Add a root-relative exclusion; repeatable. |
| `--hidden true\|false` | Whether hidden files and directories are measured. |
| `--respect-ignore-files true\|false` | Whether `.gitignore`, `.ignore`, and `.tokeignore` limit the selection. |
| `--no-user-config` | Ignore personal defaults in `~/.config/locmetry/config.toml`. |
| `--format table\|markdown\|json\|headline` | Result format, including files. `json` is complete; `headline` is compact JSON. Defaults: `json` with `--output`, otherwise `table` on a terminal and `headline` in pipes. |
| `--sort code\|name` | Row order of the table and Markdown output; default: `code`, largest first. |
| `--depth N` | Folder levels in Markdown; default: 2. An explicit value warns if the output is not Markdown. |
| `--output FILE` | Write only to `FILE`, outside the scan root; `-` selects stdout. Defaults to full JSON; explicit `--format` wins. Extensions do not select a format. |
| `--files` | Add per-file records; requires full JSON (`--format json` or `--output FILE`). |
| `--verbose` | Include classification, scope and coverage notes in root/workspace table output; warns without changing content in other measurement formats. |
| `--categories` | Add purpose/category rows to table output; warns in other formats. |
| `--guide configure\|selection\|reports\|compare` | Print one embedded task guide with its source path and executable version. Standalone query; no root, configuration, format or output options. |
| `--schema config\|user-config\|report\|comparison\|explanation\|workspace` | Print the JSON Schema of `.locmetryrc`, of personal defaults, of the JSON headline and report, of a net-change comparison, of a path explanation, or of a workspace file, and exit. Standalone query: rejects explicit roots, measurement/presentation options and subcommands; redirect stdout to save a schema. |

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
- **Findings stay with their repository:** `metadata.members` lists each repository's root, configuration path, scope, coverage counts, unused entries, and unused exclusions; verbose table notes name the repository, as in `ghost (web)`, and list unclaimed source by repository. Default and verbose tables name members with read/parse failures under `Unavailable counts:`; Markdown retains comprehensive member findings.
- **All or nothing, always the same:** if one repository fails, for example with an invalid configuration, the run fails and names it; the order of the roots never changes the report.

## Explain one file

```sh
locmetry explain src/main.rs
```

For Markdown from another scan root:

```sh
locmetry explain src/main.rs --root ./service --format markdown
```

For a workspace member, prefix the path with its member name:

```sh
locmetry explain api/src/main.rs --workspace repos.toml
```

Discover the complete explanation contract:

```sh
locmetry --schema explanation
```

The explanation uses the same policy as measurement and reports either the counted file's parser, winning rule, matching patterns and module evidence, or why it has no counts. It counts only the target, while selected names still determine module layouts. Targets are root-relative; workspace targets start with the member name. Single-root configuration and scope flags are available, plus `--no-user-config` in either mode.

Ignore and hidden filtering are checked with single-setting selection probes; the walker does not expose a winning ignore-file line. Unknown causes remain explicit. The [command outcomes](docs/PRODUCT.md#command-outcomes) distinguish successful explanations—including filtered, unrecognized and symlink targets—from unavailable counts. [Troubleshooting guide](docs/guides/selection.md) gives a runnable example, state meanings and policy/path details.

## Compare saved measurements

Save full baselines outside the measured root; exports are silent on stdout. Net change is after minus before:

```sh
locmetry --root ./service --output ./before.json
# After changing the source or its measurement policy:
locmetry --root ./service --output ./after.json
locmetry compare before.json after.json
```

Human comparisons summarize coverage and producer differences under Measurement notes; full JSON retains per-folder findings. Detail rows show Lines and Code before, after and net change when either changes, or when a member/folder exists on only one side. Comment-only and blank-only growth therefore appears as a line change; data-only and file-count-only changes remain in Totals and full JSON. The terminal Totals table sizes its columns to their contents. Run `locmetry --guide compare` for saved Markdown, workspace changes, partial-input handling and interpretation. The same [comparison guide](docs/guides/compare.md) is available here; aggregate comparison needs no `--files`.

## Common issues and useful queries

| Question | On-demand guide and repository copy |
| --- | --- |
| Which files make up an unclaimed folder or a non-source total? | `locmetry --guide reports` — [queries](docs/guides/reports.md#common-issues-and-useful-queries) |
| Why does a subdirectory scan lose classification context? | `locmetry --guide selection` — [diagnostic recipes](docs/guides/selection.md#common-issues-and-useful-queries) |
| Why is a file missing or classified unexpectedly? | `locmetry --guide selection` — [interpret evidence](docs/guides/selection.md#interpret-the-evidence) |
| How do I interpret unavailable counts or a saved comparison? | `locmetry --guide reports` and `locmetry --guide compare` — [coverage](docs/guides/reports.md), [net change](docs/guides/compare.md) |

## Configure with an agent

> Create or evaluate a locmetry configuration for this repository.

Start with `locmetry --guide configure`; the [same guide](docs/guides/configure.md) provides the workflow and checked example. `locmetry --schema config` supplies exact fields, defaults and classification semantics. The agent can classify with evidence, deliberately leave code unclassified with a reason, or report an unresolved unclaimed question. A clean review list alone does not establish correct classification.

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
- **Traceable:** the verbose table and the JSON headline (`summary.build_modules`) show each build system's modules and the code they classified, the Markdown report lists every module, and each file record from `--files` names the detector that decided it, such as `builtin:gradle-module`.

### Terminal table

Plain `locmetry` on a terminal prints the table shown [above](#what-you-see):

- language rows with Files, Lines, Code, Comments and Blanks, followed by the code split by purpose, largest code first;
- under each table, a legend stating how its columns add up, such as Production + Test + Unclassified = Code.

Add `--folders` for top-level folder rows with Lines, Code, the purpose split and `Test/prod`; the rows add up to the source total. `--folder-rows N` enables adaptive folder rows without also requiring `--folders`. Both apply to single-project and subfolder table output, including saved tables. Workspace repository tables remain visible by default. `--verbose` controls notes independently and does not enable the folder table. Markdown and JSON retain their folder detail; `--folders` warns when used with a non-table measurement format.

Successful default tables contain no routine audit notes or flag reminders. Data/documentation totals and unrecognized-file counts are deliberately omitted. Add `--verbose` for non-source totals, inferred build modules, Git submodule hints, unclaimed folders, unused policy and unrecognized extensions. Real read/parse failures remain visible in both views; verbose tables additionally list up to 25 parse-failure paths. To inspect or refine purpose classification, use `locmetry --guide configure`; `--files --output -` shows each file's rule.

Table and Markdown reports end with `Measured at: YYYY-MM-DD HH:MM:SS UTC`, using the measurement-start timestamp already recorded in JSON as `metadata.measured_at_unix_seconds`. Comparison reports end with both input measurement dates. File creation times are not used.

Terminal table cells, notes and application diagnostics flatten control characters to spaces; JSON retains original paths. `--sort name` lists rows alphabetically instead, in the table and in Markdown. `--categories` adds a table of source code per `purpose:category`, such as `production:tooling`, to see what production consists of.

### Look inside large folders

If the table requested with `--folders` concentrates most code in a container such as `web` or `packages`, request deeper detail without changing the scan root or classification context:

```sh
locmetry acme-orders --folder-rows 12
```

This enables the **Folder** table with adaptive detail, without requiring `--folders` as well. The first **Language** table stays as it is. Large folders expand into non-overlapping child rows, including direct files and scoped “other child folders” rows where needed. Every row still contributes once to the total. The budget counts Folder data rows, excluding its header, total and legends. It limits additional detail; existing top-level contributions are never merged away, so an initially wider table can exceed it. `--sort name` compares path components, keeping descendants before siblings such as `src-old`; the same comparison resolves code ties. Single-child chains collapse without using extra rows. Paths stay relative to the project root, including when combined with `--subfolder`.

`--format table` is an existing option that selects human-readable text. It is optional in a terminal, where table output is the default. Pipes normally select a JSON headline, and `--output` defaults to full JSON. Explicitly request table output when saving or paging this view:

```sh
locmetry acme-orders --folder-rows 12 --format table --output folders.txt
```

When the largest displayed row still has folder detail, a gray hint names the folder and suggests increasing `--folder-rows`. This also covers a largest “other child folders” remainder. Smaller expandable folders alone do not trigger it.

The option changes only the table view, not measurement counts, coverage or the Unclaimed source note requested with `--verbose`. It cannot accompany JSON output; save full JSON separately when needed. The verbose note uses its existing partition of unclassified code and can therefore show different folder boundaries. Markdown retains its recursive `--depth` view; workspace mode rejects `--folder-rows`. [Adaptive folder semantics](docs/PRODUCT.md#adaptive-folder-detail) defines selection, budgets and limits.

### JSON headline for agents and scripts

Without `--output`, when stdout is not a terminal, `locmetry` prints a compact JSON headline; `--format headline` requests it explicitly. It is under 3 KB for this project. Abridged:

```json
{
  "schema_version": "1",
  "project": "acme-orders",
  "metadata": {
    "root": "/work/acme-orders",
    "measured_at_unix_seconds": 1792238400,
    "locmetry_version": "0.4.0",
    "scope": {
      "exclude": ["**/.git/**", "**/node_modules/**", "**/.build/**", "**/.lake/**", "third_party/**", ".locmetryrc"]
    },
    "config_path": "./.locmetryrc",
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

`--output FILE` writes only the full JSON report, without stdout: the summary, a review list, and an additive `breakdown` table with one row per folder, kind, purpose, category, and language. Each row sits on its own line:

```json
{"folder":"backend/src/main/kotlin/acme/orders","kind":"source","purpose":"production","category":"implementation","language":"Kotlin","files":3,"code":42,"comments":2,"blanks":11,"lines":55},
{"folder":"backend/src/test/kotlin/acme/orders","kind":"source","purpose":"test","category":"test","language":"Kotlin","files":1,"code":16,"comments":0,"blanks":4,"lines":20},
```

Because rows are additive, recorded dimensions can be filtered and summed:

```sh
$ locmetry acme-orders --output report.json

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

`--files` adds one record per file with its language, kind, purpose, category, deciding rule, and counts, for debugging rules. It requires full JSON, selected by `--format json` or by the default with `--output FILE` / `--output -`. A table, Markdown or headline result with `--files` is an error. Full JSON without `--files` still contains the aggregate detail needed for comparison.

### Markdown report

`locmetry --format markdown` renders a report to save or paste. Besides purpose and category tables, it contains other counted content, source languages, a recursive folder breakdown to `--depth N` levels, the review list, and coverage. Abridged:

```markdown
## Source categories

| Category | Files | Lines | **Code** | Comments | Blanks |
| --- | ---: | ---: | ---: | ---: | ---: |
| production:implementation | 5 | 91 | 73 | 3 | 15 |
| test:test | 2 | 32 | 26 | 0 | 6 |
| production:tooling | 1 | 19 | 12 | 1 | 6 |
| production:build-tooling | 2 | 10 | 9 | 0 | 1 |
| unclassified:unclassified | 1 | 11 | 8 | 1 | 2 |
| unclassified:third-party-wrapper | 1 | 3 | 1 | 2 | 0 |

## Folder breakdown

| Folder | Lines | **Code** | Production | Test | Unclassified | Test/prod |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| . | 166 | 129 | 94 | 26 | 9 | 0.28 |
| backend | 83 | 65 | 49 | 16 | 0 | 0.33 |
| backend/src | 75 | 58 | 42 | 16 | 0 | 0.38 |
| prototypes | 11 | 8 | 0 | 0 | 8 | - |
| scripts | 19 | 12 | 12 | 0 | 0 | 0.00 |
| web | 48 | 41 | 31 | 10 | 0 | 0.32 |
| web/src | 48 | 41 | 31 | 10 | 0 | 0.32 |

## Build modules

| Build system | Modules | Lines | Code |
| --- | ---: | ---: | ---: |
| gradle | 1 | 53 | 42 |
| npm | 1 | 38 | 31 |

## Source requiring review

1 file of unclassified source that no rule, convention, or build module claims. Code an entry or convention leaves unclassified on purpose is not listed.

| Folder | Files | Lines | Code |
| --- | ---: | ---: | ---: |
| prototypes | 1 | 11 | 8 |

| Path | Lines | Code | Category |
| --- | ---: | ---: | --- |
| prototypes/pricing_experiment.py | 11 | 8 | unclassified |

## Coverage

Files selected: 17; counted: 16, including 2 through language mappings; not recognized: 1; recognized but unreadable: 0. Their lines are missing, not zero. Counts describe the working copy, including untracked files, not a commit.

Measured at: 2026-10-17 12:00:00 UTC
```

## Continuous integration

Capture the measurement before applying a policy check, preserving Locmetry's exit status. This script refuses partial reports, fails when more than 50 source lines remain unclaimed, and prints the test/production ratio only when production code is nonzero. `jq -e` fails when the policy expression is false:

```sh
set -eu
report=$(locmetry --format headline) || exit "$?"
printf '%s\n' "$report" | jq -e '([.summary.unclassified_folders[].code] | add // 0) <= 50'
printf '%s\n' "$report" | jq -r '
  (.summary.source_purpose // {}) as $p |
  if ($p.production.code // 0) == 0 then
    "test/production: n/a (no production code)"
  else
    "test/production: \(($p.test.code // 0) * 100 / $p.production.code | floor)%"
  end'
```

[CI executes this block and the comparison/explanation workflows](scripts/check-doc-examples.sh) against synthetic projects. The policy scenarios check stdout and stderr as well as status, so invalid configuration cannot masquerade as an unclaimed-code rejection. The Cargo tests separately guard its JSON and exit-code assumptions without requiring `jq`.

[Command outcomes](docs/PRODUCT.md#command-outcomes) defines exit statuses for measurement, comparison and explanation, including backend inaccuracy without identified failure paths.

## Output formats

| Format | Selected by | For | Contents |
| --- | --- | --- | --- |
| Table | default on a terminal without `--output`, `--format table` | people | source code by language, split by purpose; folders with `--folders`, repository rows in workspace mode; routine notes with `--verbose`, failures always visible |
| JSON headline | default when piped without `--output`, `--format headline` | agents, scripts, CI | totals by purpose, category, kind, and language; top-level folders; effective scope; coverage |
| Full JSON | `--format json`, or the default with `--output FILE` / `--output -` | deeper questions | the headline plus review list, additive breakdown table, effective configuration, and with `--files` per-file records |
| Markdown | `--format markdown` | reviews, wikis | purpose, category, kind, language, and folder tables, review list, and coverage |

The table and Markdown group thousands with commas, such as `12,345`, while JSON keeps plain integers for scripts.

Terminal tables use compact titles, cyan row labels, bold cyan Code figures and totals, dim secondary figures, and gray zeros, separators and legends. Net changes use cyan/magenta; warning/error labels use yellow/red. Use `NO_COLOR=1` to disable terminal styling; redirected output stays plain. Markdown uses italic legends and bold emphasis. See [human presentation](docs/PRODUCT.md#human-presentation) for the stream policy and theme boundary.

Output routing for measurement, workspace and comparison follows the [same contract](docs/PRODUCT.md#output-routing). Explicit formats apply to files too: `--format markdown --output report.md` saves Markdown, and file output is always unstyled. `--output report.md` alone saves JSON because extensions are not interpreted. `explain` has no headline or `--output` option; schema output is saved with `locmetry --schema report > report.schema.json`.

Prefer `--output FILE` for protected exports. Shell redirection and `tee` bypass destination checks and may mask a partial exit status unless the pipeline preserves it. Two separate measurements need not describe the same filesystem state. A saved headline cannot reconstruct a full historical baseline.

## Compared with line counters

[cloc](https://github.com/AlDanial/cloc) provides source comparison and detailed filtering diagnostics; [scc](https://github.com/boyter/scc) provides additional analytics, configuration and integrations. Locmetry focuses on a combined contract: reviewable scope, whole-file purpose/category with provenance, source/data/documentation separation, explicit coverage, and independently configured workspace members. The dated [capability comparison](docs/research/tool-capability-comparison.md) records baseline evidence and differences; [the implementation history](docs/plans/archive/measurement-quality-and-net-change.md) records subsequent fixes and features; it is not an accuracy or performance ranking.

Known limits: classification is per file, so a file mixing production code and inline tests counts as one purpose. Git is not invoked, so a measurement describes the working copy rather than a commit. [Measurement semantics and limits](docs/measurement.md) lists the details.

## Documentation

For humans and agents working on Locmetry, every shared documentation folder has a README index. Proposal and plan tables show each document’s status, date and summary, generated from its authoritative metadata. CI checks index completeness, status/date validity, synchronization and local links, so agents can find current decisions without treating archived instructions as active work. See the [maintenance rules](docs/documentation-maintenance.md).

- [Documentation index](docs/README.md): folder navigation, dated status tables and documentation maintenance.
- [Product vision and requirements](docs/PRODUCT.md): configuration contract, report schema, and guarantees.
- [Measurement semantics and limits](docs/measurement.md): selection and counting details and known limits.
- [Configure a project](docs/guides/configure.md): evidence-based setup and evaluation, including open questions.
- [Selection and explanations](docs/guides/selection.md): path, parser and classification diagnostics.
- [Read reports](docs/guides/reports.md): coverage, counts and useful queries.
- [Compare measurements](docs/guides/compare.md): baseline capture and net-change interpretation.
- [Saved report compatibility](docs/report-compatibility.md): version-1 field history, validation and evidence required for comparison.
- [Architecture and distribution](docs/tokei-integration.md): embedded counting engine, release binaries, and the Homebrew formula.
- [Research](docs/research/README.md): counter capabilities and classification findings.
- [Development plan](docs/PLAN.md): open work, decision gates and links to historical evidence.
- [Proposals and historical decisions](docs/proposals/README.md): active designs for review, archived rationale and the retirement procedure.
- [Implementation history](docs/plans/archive/measurement-quality-and-net-change.md): completed measurement-quality stages, rationale and verification; deferred work is indexed separately in PLAN.
- [Repository guidelines](CLAUDE.md): contributor conventions and checks; `AGENTS.md` links to this file.

## License

Licensed under the [MIT License](LICENSE). Release archives and the Homebrew installation include `THIRD-PARTY-NOTICES.md` with the licences of the bundled libraries. Line counting uses the embedded [Tokei](https://github.com/XAMPPRocky/tokei) library.
