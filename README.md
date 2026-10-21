<img src="docs/assets/locmetry.svg" alt="locmetry logo" width="96">

# locmetry

[![CI](https://github.com/mbackschat/locmetry/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/mbackschat/locmetry/actions/workflows/ci.yml)
[![CodeQL](https://github.com/mbackschat/locmetry/actions/workflows/github-code-scanning/codeql/badge.svg?branch=main)](https://github.com/mbackschat/locmetry/actions/workflows/github-code-scanning/codeql)
[![Latest release](https://img.shields.io/github/v/release/mbackschat/locmetry)](https://github.com/mbackschat/locmetry/releases/latest)
[![Licence: MIT](https://img.shields.io/github/license/mbackschat/locmetry)](LICENSE)

Locmetry counts code, comments and blank lines, classifies files as production, test or other, and records the rule behind each classification. Reports separate source code from data and documentation and show results by language and folder.

## Install

With Homebrew on macOS arm64 or Linux x86_64 and aarch64:

```sh
brew install mbackschat/tap/locmetry
```

Or download a binary from the [latest release](https://github.com/mbackschat/locmetry/releases/latest). Locmetry is a self-contained executable with no separately installed runtime or counter. Linux binaries are static; macOS uses operating-system libraries. See [manual installation and source builds](docs/tokei-integration.md#installation) for alternatives.

## What you see

These excerpts measure [Immich](https://github.com/immich-app/immich) `v3.3.4`, a popular self-hosted photo platform with a TypeScript server, a Svelte web client, a Flutter app and a Python machine-learning service. Locmetry measures it with a short [reviewed policy](examples/showcase/immich/README.md) kept in this repository; the [refresh script](scripts/refresh-readme-showcase.py) regenerates every excerpt from the pinned checkout.

**[Take the tour](docs/showcase/tour.md)** to build that policy step by step, from the first measurement to reviewed decisions.

▶️ **The [showcase recordings](docs/showcase/README.md) replay measurement, workspace and comparison on Immich; the [command guides](docs/cli/README.md) cover every option on a small fixture.**

📊 **Ask your own questions of the reports:** the 🦆 [DuckDB cookbook](docs/duckdb-cookbook.md) and the 🐼 [pandas and Polars cookbook](docs/dataframe-cookbook.md) analyze two Immich releases step by step, with real output.

### A line counter answers "how much text, per language"

<!-- showcase-tools -->
| Tool | Reported code | of which JSON | Production / test split |
| --- | ---: | ---: | --- |
| Tokei 15.0.0 | 518,920 | 209,607 (40%) | none |
| cloc 2.10 | 530,430 | 209,931 (40%) | none |
| scc 4.1.0 | 510,075 | 209,867 (41%) | none |

Line counters are good at counting. Locmetry counts with Tokei too, but asks a different question: almost half of Immich's "code" is data, such as database schemas, the OpenAPI specification and translations.

### Locmetry answers "how much source, and how much of it is tests"

```sh
locmetry --root immich
```

<!-- showcase-language -->
```text
 ────────────────────────────────────────────────────────────────────────────────────────────
 Language    Files    Lines     Code │ Comments  Blanks │   Prod.    Test   Other │ Test/prod
 ────────────────────────────────────┼──────────────────┼─────────────────────────┼──────────
 TypeScript  1,074  171,359  145,427 │    4,624  21,308 │  70,455  68,506   6,466 │      0.97
 Dart          823   90,871   75,726 │    2,779  12,366 │  59,495  16,231       0 │      0.27
 JavaScript     17   23,273   19,746 │      427   3,100 │  19,684      62       0 │      0.00
 Svelte        412   19,014   17,442 │      200   1,372 │  17,420      22       0 │      0.00
 SQL            37   10,534    9,699 │      436     399 │       0       0   9,699 │         -
 Python         39    6,184    4,888 │       79   1,217 │   2,818   2,070       0 │      0.73
 Kotlin         35    4,604    3,810 │      177     617 │   3,810       0       0 │      0.00
 Swift          28    3,874    3,103 │      232     539 │   3,103       0       0 │      0.00
 CSS             2      818      691 │       26     101 │     691       0       0 │      0.00
 TSX            11      597      516 │        9      72 │     516       0       0 │      0.00
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
 Lines = Code + Comments + Blanks; Production + Test + Other = Code; Test/prod = Test / Production (Code).
 Tests written in another language than the code they test skew per-language Test/prod.
```

Code is split by purpose; comments and blanks stay separate. Data and documentation do not inflate source totals. `Test/prod` divides test code by production code; it describes volume, not coverage. If your numbers differ from these, personal defaults in `~/.config/locmetry/config.toml` may apply; `--no-user-config` ignores them.

### A reviewed policy answers "what does this project consist of"

Built-in conventions already recognize the npm, Gradle, Python and Dart modules and the conventional unit, integration and end-to-end tests. A policy adds what only the project knows, with a recorded reason for each decision: generated code, end-to-end support code, the iOS app, deployment files and developer tooling.

<!-- showcase-policy -->
| Source code | Built-in conventions | Reviewed policy |
| --- | ---: | ---: |
| Production | 193,662 | 179,704 |
| Test | 83,485 | 86,901 |
| Other: generated | 0 | 16,165 |
| Other: unclaimed | 5,623 | 0 |
| Test/prod | 0.43 | 0.48 |

`locmetry explain FILE` names the rule behind any decision; for the generated SDK client, it reports the override and the reason recorded with it.

### Workspaces and comparisons answer "where, and what changed"

Measure the application components together under the same policy. The workspace file names them as folders of the project:

<!-- showcase-workspace-file -->
```toml
version = 1
# The application components of Immich, each measured under the checkout's
# .locmetryrc as --subfolder would; root-only folders such as docs/ and
# deployment/ stay in the root measurement.
project = "."
folders = ["server", "web", "mobile", "machine-learning", "e2e", "packages"]
```

```sh
locmetry workspace --workspace-file immich/workspace.toml
```

<!-- showcase-workspace -->
```text
 ─────────────────────────────────────────────────────────────────────────────────
 Repository          Lines     Code │   Prod.    Test   Other │ Test/prod │  Share
 ───────────────────────────────────┼─────────────────────────┼───────────┼───────
 server            129,296  111,950 │  53,296  48,955   9,699 │      0.92 │  39.7%
 mobile            100,142   83,220 │  66,989  16,231       0 │      0.24 │  29.5%
 web                67,833   58,255 │  52,867   5,388       0 │      0.10 │  20.7%
 e2e                15,491   13,174 │     171  13,003       0 │     76.04 │   4.7%
 packages           12,893   10,065 │   2,345   1,254   6,466 │      0.53 │   3.6%
 machine-learning    6,698    5,291 │   3,221   2,070       0 │      0.64 │   1.9%
 ───────────────────────────────────┼─────────────────────────┼───────────┼───────
 Total             332,353  281,955 │ 178,889  86,901  16,165 │      0.49 │ 100.0%
 Production + Test + Other = Code; Test/prod = Test / Production (Code); Share = Code / workspace Code.
```

Compare saved reports of two releases, `v3.2.0` and `v3.3.4`:

<!-- showcase-compare -->
```text
 ──────────────────────────────────────────────────────────────────
                         Lines           │           Code
 Purpose        before    after   change │  before    after  change
 ────────────────────────────────────────┼─────────────────────────
 Production    208,180  210,881   +2,701 │ 177,605  179,704  +2,099
 Test           96,101  103,306   +7,205 │  81,051   86,901  +5,850
 Other          17,825   19,173   +1,348 │  14,910   16,165  +1,255
 ────────────────────────────────────────┼─────────────────────────
 Source total  322,106  333,360  +11,254 │ 273,566  282,770  +9,204
 Test/prod (Code): 0.46 → 0.48 (before → after)
```

Most of the change is test code. [Folder reporting](docs/guides/reports.md#look-inside-large-folders) and [comparison](docs/guides/compare.md) explain the remaining views.

## Common commands

| Task | Command |
| --- | --- |
| Measure another project | `locmetry --root ../service` |
| Inspect scope, classification and coverage notes | `locmetry --verbose` |
| Measure a subfolder with project policy and module context | `locmetry --subfolder web/src` |
| Explain a file's classification or missing counts | `locmetry explain web/src/orders/Order.test.ts` |
| Save full JSON with per-file decisions outside the root | `locmetry --files --output ../report.json` |
| Save full JSON and print the table from one scan, as for agents | `locmetry --output ../report.json --print table` |
| Save a Markdown report | `locmetry --format markdown --output ../report.md` |
| Measure repositories together, each with its own policy | `locmetry workspace --root ../web --root ../api` |
| Show production source in the table, with its comment density | `locmetry --format table --table-purpose production` |
| Inspect the configuration contract | `locmetry --schema config` |
| Read full JSON on stdout | `locmetry --root service --format json --files` |
| Measure selected source, including ignored files, except vendor | `locmetry --root service --subfolder src --exclude 'src/vendor/**' --hidden true --respect-ignore-files false --format json` |
| Show adaptive folder detail and production categories | `locmetry --root service --format table --folder-rows 30 --table-purpose production --show-categories --show-bars=comments --sort name` |
| Save a sorted Markdown tree to depth 3 | `locmetry --root service --format markdown --markdown-depth 3 --sort name --output report.md` |
| Measure an explicit workspace definition | `locmetry workspace --workspace-file repos.toml --files --output workspace.json --print table` |
| Compare two saved full reports | `locmetry compare before.json after.json --format table` |

`locmetry --help` includes the same commands, plus a working configuration example for maintained scripts with applicability and verification instructions. Verbose tables total data and documentation files under “Not source code”; to list them, [filter full JSON by `kind`](docs/guides/reports.md#not-source-code-which-files-are-data-or-documentation).

Without explicit options, a terminal gets a table and a pipe gets a compact JSON headline. `--output` defaults to full JSON; choose `--format table` or `--format markdown` explicitly for those exports. File exports are silent on stdout, and the extension does not select the format. [Output routing and reports](docs/guides/reports.md#choose-content-and-destination) explains the choices and protected destinations.

### Selected options

**Selection and policy** — choose the measured input and its rules.

| Option | Use |
| --- | --- |
| `--root DIR` / `--subfolder DIR` | Select a project root, or restrict measurement within it while retaining project policy and module context. |
| `--workspace-file FILE` | Select a workspace definition for `workspace` or `explain`. |
| `--config FILE` / `--exclude GLOB` | Choose project configuration or add a root-relative scope exclusion. |
| `--no-user-config` | Ignore personal defaults, for example in CI. |

**Output and content** — choose formats, destinations and JSON detail.

| Option | Use |
| --- | --- |
| `--format FORMAT` / `--output FILE` | Choose the result format and destination; `--output -` selects stdout. |
| `--print FORMAT` | With a saved file, also print another view from the same scan. |
| `--files` | Include per-file counts and decisions in full JSON, whether saved or printed; headlines omit them. |

**Presentation** — shape views without changing measured scope.

| Option | Use |
| --- | --- |
| `--show-folders` / `--folder-rows N` | Top-level / adaptive folder rows in a single-root table, saved or printed. |
| `--show-categories` / `--verbose` | Add category rows / classification and coverage notes to tables. |
| `--table-purpose LIST` | Filter table source and totals by purpose; JSON and Markdown retain the complete measurement. |
| `--show-bars[=KIND]` / `--no-bars` | Force / suppress table charts. `KIND` is `purpose` or `comments`; selecting one table purpose uses comment density even with `=purpose`. Terminals show fitting bars automatically. |
| `--markdown-depth N` / `--sort code\|name` | Bound Markdown Folder breakdown depth / order human rows; depth does not bound other evidence. |
| `--no-color` | Disable terminal color, bold and dim, including help and diagnostics; equivalent to `NO_COLOR=1`. |

Use the current spellings above; [the option contract](docs/PRODUCT.md#option-names-and-applicability) lists rejected former names and format-specific warning/error behavior.

Run `locmetry --help` or `locmetry COMMAND --help` for all options. The positional directory is equivalent to `--root`; in scripts, prefer `--root "$directory"` so a directory named `compare` or `workspace` cannot select a command. Command options follow the command name; global `--no-color` works before or after it.

### For agents

One call gives an agent both a readable overview and a saved report:

```sh
locmetry --root . --output ../report.json --print table
```

Stdout gets the table, unstyled and without share bars in a pipe; the file gets full JSON for queries or a later `locmetry compare`. Use `--print headline` for compact JSON on stdout instead, and `locmetry --schema report` for every field. `--show-folders` and `--verbose` extend the printed table.

### Compare saved measurements

Save full baselines outside the measured root; a headline cannot reconstruct a baseline. Net change is after minus before:

```sh
locmetry --root ./service --output ./before.json
# After changing the source or its measurement policy:
locmetry --root ./service --output ./after.json
locmetry compare before.json after.json
```

For two saved workspace reports, drill into one root by its exact member name:

```sh
locmetry compare before-workspace.json after-workspace.json --member service --format table
```

Check `Member: service` and its `Source total`; the folder table shows changes in immediate subfolders. The name must occur in either report. Totals and detail cover that member; input metadata and partial status still cover the whole workspace inputs. See [workspace capture and member selection](docs/guides/compare.md#inspect-one-workspace-member) for baseline commands and JSON scope.

The [comparison guide](docs/guides/compare.md) covers changed scope, producer versions, partial inputs and workspace changes. Reports describe measured working copies, not Git commits. For automated policies, use the [CI recipe](docs/guides/reports.md#continuous-integration), which preserves measurement failure and partial-result status before querying JSON.

For file-level recognition and classification changes, run `compare-locmetry-files.sh before.json after.json`. This optional Bash [helper](scripts/compare-locmetry-files.sh) accepts two full `--files` reports and ships in release archives and Homebrew installations starting with 0.7.3. Only the helper requires `jq`; install it separately if needed. Locmetry's native commands remain independent of `jq`. See the [transition workflow](docs/guides/compare.md#inspect-configuration-transitions) for setup, output and evidence limits.

## Analyze saved measurements

Locmetry's full JSON reports are data you can query. Load several of them into an analysis tool and ask what one report cannot answer: How did the code change between releases, and where? How much of it is generated? How much code does each team own? Which new files are the largest? What exactly did a policy change move? Both cookbooks answer such questions for two [Immich](docs/showcase/README.md) releases, explain every step for newcomers and show the real output.

- 🦆 **[DuckDB cookbook](docs/duckdb-cookbook.md)**: SQL over a local database file, covering composition, generated code, components and their tests, growth between releases, ownership, new and changed files, a policy review and Parquet export.
- 🐼 **[pandas and Polars cookbook](docs/dataframe-cookbook.md)**: the same reports as Python dataframes, covering components, comment density and change between releases, with identical results in both libraries.
- ▶️ **[Analytics demo](docs/analytics-demo.md)**: one Python script over workspace captures of both releases, the shape of a scheduled job or notebook.

The tools are optional; Locmetry itself has no analytics dependency. Capture reports with the current executable; older schema-1/2 measurements must be recaptured.

## How classification works

Locmetry embeds Tokei for counting and applies project policy to each file. Purpose (`production`, `test`, `other`), category (such as `implementation` or `tooling`) and content kind (source, data, documentation or unknown) are separate dimensions. `other` means neither production nor test; code no stage claimed is `other` with category `unclaimed`.

- **Module context:** selected npm, Gradle, Maven, Cargo, Go, Python, Lake, Swift and Dart descriptors identify conventional source layouts. The innermost module supplies context, so nested projects do not inherit an outer project's layout blindly.
- **Ordered decisions:** explicit overrides precede test and build conventions, ordinary project rules, module inference and fallback. Tests inside production source trees remain tests; overrides can express exceptions. Every counted file records the deciding rule.
- **Scope and coverage:** exclusions and ignore rules determine what is selected. Unrecognized files and unavailable counts remain distinguishable from zero. Read/parse failures produce a partial report and exit 2; unresolved classification alone does not make counts partial.

These Immich files show each kind of decision, from module layout and test conventions to project entries:

<!-- showcase-examples -->
| File | Purpose / category | Deciding rule |
| --- | --- | --- |
| `server/src/services/asset.service.ts` | production / implementation | `builtin:npm-module` |
| `mobile/lib/main.dart` | production / implementation | `builtin:dart-module` |
| `server/src/services/asset.service.spec.ts` | test / test | `builtin:js-test-file` |
| `e2e/src/specs/web/album.e2e-spec.ts` | test / test | `builtin:js-e2e-spec-file` |
| `e2e/src/utils.ts` | test / end-to-end | `end-to-end-suites` (project entry) |
| `mobile/ios/Runner/AppDelegate.swift` | production / native | `ios-app` (project entry) |
| `packages/sdk/src/fetch-client.ts` | other / generated | `generated-sdk-client` (project entry) |

Inspect the evidence for one file:

```sh
locmetry explain packages/sdk/src/fetch-client.ts --root immich
```

The explanation identifies the deciding stage, the matching pattern and the reason recorded with a project entry. It also reports parser, counts and module evidence where applicable. The [selection guide](docs/guides/selection.md) explains other states and how to investigate a missing file.

Built-in wrapper patterns cover `gradlew`, `gradlew.bat`, `mvnw`, `mvnw.cmd`, and files under `gradle/wrapper/` or `.mvn/wrapper/` at any depth. The enabled convention classifies them as `other` / `third-party-wrapper` unless an override or earlier convention wins; [wrapper details and precedence](docs/PRODUCT.md#matching-and-precedence) define the conditions.

Module inference examines filenames and conventional layouts, not build-file contents, and never executes a build. Custom layouts need configuration. Classification applies to whole files, so inline tests cannot receive a separate purpose. [Module detection](docs/PRODUCT.md#module-detection) and [measurement limits](docs/measurement.md) define these boundaries.

### Configuration

No configuration is required to start. Add `.locmetryrc` when conventions cannot know local intent: which code is vendored, which scripts are maintained tooling, or which experiments should explicitly count as `other`. Locmetry does not automatically recognize vendored or generated code.

Two entries of the Immich policy show the shape: an override that beats the npm module layout for generated code, and a rule for code no build descriptor declares. Each restricts itself to source and records its evidence:

<!-- showcase-config -->
```toml
version = 1

[[overrides]]
id = "generated-sdk-client"
match = ["packages/sdk/src/fetch-client.ts"]
kind = ["source"]
purpose = "other"
category = "generated"
reason = "Header: DO NOT MODIFY, generated using oazapfts from the OpenAPI specification"

[[rules]]
id = "ios-app"
match = ["mobile/ios/**"]
kind = ["source"]
purpose = "production"
category = "native"
reason = "Swift code of the iOS runner and widget extension; no Package.swift declares it"
```

Tests still win over ordinary rules; an override is needed only to beat a convention. Entries without evidence belong in the review list as unclaimed source, not in the policy. A clean review list alone does not establish correct classification.

The [configuration guide](docs/guides/configure.md) covers overrides, custom categories, language mappings and evidence-based review. [Personal defaults](docs/PRODUCT.md#user-defaults) can supply exclusions and language mappings across projects; they cannot assign purposes or categories.

Locmetry's own [.locmetryrc](.locmetryrc) is a maintained example: it excludes the synthetic demonstration workspace and distinguishes operational tooling, verification infrastructure and analytics examples. [Self-measurement design](docs/self-measurement.md) explains the decisions and how to verify them.

For a production-rule change covered by the example in `locmetry --help`, inspect existing configuration and measure current decisions, adapt the paths and category, then run the displayed verification command. Reading a guide or schema first is optional for that route. Use `locmetry --guide configure` or `locmetry --schema config` for exclusions, overrides, intentional Other, or semantics not shown there. Verify per-file decisions and unresolved review items before concluding. Guides and schemas are available from the installed binary without a network connection.

## Guides and reference

| Task | Guide in the executable | Repository copy |
| --- | --- | --- |
| Configure or review classification | `locmetry --guide configure` | [Configure](docs/guides/configure.md) |
| Diagnose a file or subfolder | `locmetry --guide selection` | [Selection and explanations](docs/guides/selection.md) |
| Read coverage, query reports or apply a CI policy | `locmetry --guide reports` | [Reports](docs/guides/reports.md) |
| Capture baselines and interpret net change | `locmetry --guide compare` | [Compare](docs/guides/compare.md) |

- [Product contract](docs/PRODUCT.md): supported settings, fields and guarantees.
- [Measurement limits](docs/measurement.md): selection and counting boundaries.
- [Documentation index](docs/README.md): architecture, research and current work.
- [Contributor guidelines](CLAUDE.md): development conventions and checks.

## License

Licensed under the [MIT License](LICENSE). Release archives and the Homebrew installation include `THIRD-PARTY-NOTICES.md` with the licences of bundled libraries.
