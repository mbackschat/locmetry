<img src="docs/assets/locmetry.svg" alt="locmetry logo" width="96">

# locmetry

[![CI](https://github.com/mbackschat/locmetry/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/mbackschat/locmetry/actions/workflows/ci.yml)
[![CodeQL](https://github.com/mbackschat/locmetry/actions/workflows/github-code-scanning/codeql/badge.svg?branch=main)](https://github.com/mbackschat/locmetry/actions/workflows/github-code-scanning/codeql)
[![Latest release](https://img.shields.io/github/v/release/mbackschat/locmetry)](https://github.com/mbackschat/locmetry/releases/latest)
[![Licence: MIT](https://img.shields.io/github/license/mbackschat/locmetry)](LICENSE)

Line counters such as Tokei, cloc and scc report how many lines of each language a repository holds. Locmetry reports what those lines are for: it counts with Tokei, then classifies every file as production, test or other code and by category, such as generated code or tooling, keeps data and documentation out of source totals, and records the rule behind each classification so you can check it.

| | Line counters (Tokei, cloc, scc) | Locmetry |
| --- | --- | --- |
| Code, comments and blanks per language | Yes | Yes, counted by Tokei |
| Production, test and other code | Not built in; scc can map file patterns to custom categories | Built in from build modules and test conventions, refined by a reviewed policy |
| Data and documentation | Counted next to source: JSON counts as code | Reported apart from source code |
| Why a file counts where it does | No classification to explain | `locmetry explain PATH` names the deciding rule |
| Change between two measurements | cloc `--diff` compares source trees | `locmetry compare` compares saved reports by purpose, language and folder |

The [comparison on Immich](#compared-with-a-line-counter-how-much-text-per-language) shows what that changes: the counters count its database schemas, OpenAPI specification and translations as code.

![Locmetry measures Immich by language and purpose, adds its folder and category tables, explains why a generated SDK client counts as other code, and compares two Immich releases](docs/assets/demo.gif)

It needs no configuration to start: Locmetry recognizes the [build modules](#build-modules) of npm, Gradle, Maven, Cargo, Go, Python, Lake, Swift, Dart, .NET, Composer and RubyGems projects and of Next.js, Nuxt, React Router, Symfony, Laravel, Rails and Django applications, counts their conventional source layouts as production and finds tests by built-in conventions, so even a multi-language monorepo gets a production/test split on the first run.

## Install

With Homebrew on macOS arm64 or Linux x86_64 and aarch64:

```sh
brew install mbackschat/tap/locmetry
```

Or download a binary from the [latest release](https://github.com/mbackschat/locmetry/releases/latest); releases after 0.7.6 also include a zip for Windows x86_64. Locmetry is a self-contained executable with no separately installed runtime or counter. Linux binaries are static, the Windows binary needs no Visual C++ runtime, and macOS uses operating-system libraries. Releases after 0.9.0 also include `.deb` and `.rpm` packages for Debian, Ubuntu, Fedora and other Linux distributions. See [manual installation, Linux packages and source builds](docs/tokei-integration.md#installation) for alternatives.

### Windows and PowerShell

Install with [Scoop](https://scoop.sh):

```powershell
scoop bucket add mbackschat https://github.com/mbackschat/scoop-bucket
scoop install mbackschat/locmetry
```

Run Locmetry from PowerShell; the [common commands](#common-commands) work there as written, including globs in single quotes such as `--exclude 'vendor/**'`.

- Save reports with `--output FILE` rather than `>`. Windows PowerShell 5.1 redirects as UTF-16; `locmetry compare` reads that, but other tools may not.
- Personal defaults live in `%APPDATA%\locmetry\config.toml`.
- The optional `compare-locmetry-files.sh` helper is a Bash script and needs Bash, for example from Git for Windows, and `jq` (`winget install jqlang.jq`).

## What you see

These excerpts measure [Immich](https://github.com/immich-app/immich) `v3.3.4`, a popular self-hosted photo platform with a TypeScript server, a Svelte web client, a Flutter app and a Python machine-learning service. Locmetry measures it with a short [reviewed policy](examples/showcase/immich/README.md) kept in this repository; the [refresh script](scripts/refresh-readme-showcase.py) regenerates every excerpt from the pinned checkout.

New to the vocabulary? [Terms](#terms) explains purpose, category, unclaimed and the other words in these excerpts.

**[Take the tour](docs/showcase/tour.md)** to build that policy step by step, from the first measurement to reviewed decisions.

▶️ **The [showcase recordings](docs/showcase/README.md) replay measurement, workspace and comparison on Immich; the [command guides](docs/cli/README.md) cover every option on a small fixture.**

### Locmetry answers "how much source, and how much of it is tests"

```sh
cd immich && locmetry
```

<!-- showcase-language -->
```text
 ───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
 Language    Files    Lines     Code │ Comments  Blanks │   Prod.    Test   Other │ Test/prod │  Share  Code share
 ────────────────────────────────────┼──────────────────┼─────────────────────────┼───────────┼─────────────────────────
 TypeScript  1,085  171,956  145,943 │    4,633  21,380 │  70,861  68,616   6,466 │      0.97 │  51.6%  ████████▓▓▓▓▓▓▓░
 Dart          823   90,871   75,726 │    2,779  12,366 │  59,495  16,231       0 │      0.27 │  26.8%  ██████▓▓
 JavaScript     17   23,273   19,746 │      427   3,100 │  19,684      62       0 │      0.00 │   7.0%  ██
 Svelte        412   19,014   17,442 │      200   1,372 │  17,420      22       0 │      0.00 │   6.2%  ██
 SQL            37   10,534    9,699 │      436     399 │       0       0   9,699 │         - │   3.4%  ░
 Python         39    6,184    4,888 │       79   1,217 │   2,818   2,070       0 │      0.73 │   1.7%  █
 Kotlin         35    4,604    3,810 │      177     617 │   3,810       0       0 │      0.00 │   1.3%  ▏
 Swift          28    3,874    3,103 │      232     539 │   3,103       0       0 │      0.00 │   1.1%  ▏
 CSS             2      818      691 │       26     101 │     691       0       0 │      0.00 │   0.2%  ▏
 Shell          19      565      401 │       69      95 │     401       0       0 │      0.00 │   0.1%  ▏
 Dockerfile      4      369      267 │       12      90 │     257      10       0 │      0.04 │   0.1%  ▏
 C++             1      295      251 │       11      33 │     251       0       0 │      0.00 │   0.1%  ▏
 HCL            16      262      225 │        4      33 │     225       0       0 │      0.00 │   0.1%  ▏
 C               2      225      168 │       30      27 │     168       0       0 │      0.00 │   0.1%  ▏
 Groovy          3      173      139 │        7      27 │     139       0       0 │      0.00 │   0.0%  ▏
 Java            1      166      124 │       25      17 │     124       0       0 │      0.00 │   0.0%  ▏
 HTML            1       74       66 │        2       6 │      66       0       0 │      0.00 │   0.0%  ▏
 PostCSS         0       31       28 │        0       3 │      28       0       0 │      0.00 │   0.0%  ▏
 CMake           1       13        9 │        0       4 │       9       0       0 │      0.00 │   0.0%  ▏
 Templ           1       11        6 │        0       5 │       6       0       0 │      0.00 │   0.0%  ▏
 C Header        1        1        1 │        0       0 │       1       0       0 │      0.00 │   0.0%  ▏
 ────────────────────────────────────┼──────────────────┼─────────────────────────┼───────────┼─────────────────────────
 Total       2,528  333,313  282,733 │    9,149  41,431 │ 179,557  87,011  16,165 │      0.48 │ 100.0%
 Lines = Code + Comments + Blanks · Production + Test + Other = Code · Test/prod = Test / Production (Code)
 Tests written in another language than the code they test skew per-language Test/prod.
 Folded: BASH → Shell, TSX → TypeScript.
 Share = Code / total Code. Bars: █ Production  ▓ Test  ░ Other. The largest row fills the bar.
```

Code is split by purpose; comments and blanks stay separate. Data and documentation do not inflate source totals. Language variants count within their family's row, here TSX within TypeScript, and the `Folded:` line names each fold; `--split-languages` shows them separately, and JSON keeps every backend language. `Test/prod` divides test code by production code; it describes volume, not coverage. If your numbers differ from these, personal defaults in `~/.config/locmetry/config.toml` (on Windows `%APPDATA%\locmetry\config.toml`) may apply; `--no-user-config` ignores them.

#### Compared with a line counter: "how much text, per language"

Line counters are good at counting, and Locmetry counts with Tokei too. Their totals for the same checkout run to almost twice the code above, because they count data as code: about 40% is JSON alone, such as database schemas, the OpenAPI specification and translations.

<!-- showcase-tools -->
| Tool | Reported code | of which JSON | Production / test split |
| --- | ---: | ---: | --- |
| Tokei 15.0.0 | 518,920 | 209,607 (40%) | none |
| cloc 2.10 | 529,921 | 209,931 (40%) | none |
| scc 4.1.0 | 510,075 | 209,867 (41%) | none |

### A reviewed policy answers "what does this project consist of"

Built-in conventions already recognize the npm, Gradle, Python and Dart [build modules](#build-modules) and the conventional unit, integration and end-to-end tests. A policy adds what only the project knows, with a recorded reason for each decision: generated code, end-to-end support code, the iOS app, deployment files and developer tooling.

<!-- showcase-policy -->
| Source code | Built-in conventions | Reviewed policy |
| --- | ---: | ---: |
| Production | 194,232 | 179,557 |
| Test | 83,595 | 87,011 |
| Other: generated | 0 | 16,165 |
| Other: unclaimed | 4,943 | 0 |
| Test/prod | 0.43 | 0.48 |

`--show-categories` breaks each purpose down by category, which shows what the project consists of:

```sh
cd immich && locmetry --show-categories
```

<!-- showcase-categories -->
```text
 ────────────────────────────────────────────────────────────────────────────────────────────────────
 Category                   Files    Lines     Code │ Share  Code share
 ───────────────────────────────────────────────────┼────────────────────────────────────────────────
 production:implementation  1,837  199,093  170,113 │ 60.2%  ████████████████████████████████████████
 test:test                    499   99,381   83,595 │ 29.6%  ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓
 other:generated               38   19,173   16,165 │  5.7%  ░░░░
 production:native             67    8,870    7,206 │  2.5%  ██
 test:end-to-end               23    3,719    3,118 │  1.1%  ▓
 production:deployment         31    1,023      798 │  0.3%  ▏
 production:tooling            14      909      722 │  0.3%  ▏
 production:build-tooling      14      813      718 │  0.3%  ▏
 test:test-infrastructure       4      251      234 │  0.1%  ▏
 test:performance-test          1       81       64 │  0.0%  ▏
 Rows of one purpose add up to that purpose's Code, and all rows add up to the total Code.
 Share = Code / total Code. Bars: █ Production  ▓ Test  ░ Other. The largest row fills the bar.
```

Categories are labels assigned by built-in conventions, such as `test` and `build-tooling`, and by the policy, such as `generated`, `native` and `tooling`; a line counter has no equivalent. The showcase narrows the tables to [the code around the product](docs/showcase/measure.md#look-at-the-code-around-the-product) with `--table-category` and [focuses on the mobile app](docs/showcase/measure.md#focus-on-the-mobile-app) with `--subfolder`.

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
 ───────────────────────────────────────────────────────────────────────────────────────────────────────────────────────
 Member              Lines     Code │   Prod.    Test   Other │ Test/prod │  Share  Code share
 ───────────────────────────────────┼─────────────────────────┼───────────┼─────────────────────────────────────────────
 server            129,296  111,950 │  53,296  48,955   9,699 │      0.92 │  39.7%  █████████████████▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓░░░
 mobile            100,095   83,183 │  66,952  16,231       0 │      0.24 │  29.5%  ██████████████████████▓▓▓▓▓
 web                67,833   58,255 │  52,867   5,388       0 │      0.10 │  20.7%  █████████████████▓▓
 e2e                15,491   13,174 │      61  13,113       0 │    214.97 │   4.7%  ▓▓▓▓
 packages           12,893   10,065 │   2,345   1,254   6,466 │      0.53 │   3.6%  █░░
 machine-learning    6,698    5,291 │   3,221   2,070       0 │      0.64 │   1.9%  █▓
 ───────────────────────────────────┼─────────────────────────┼───────────┼─────────────────────────────────────────────
 Total             332,306  281,918 │ 178,742  87,011  16,165 │      0.49 │ 100.0%
 Production + Test + Other = Code · Test/prod = Test / Production (Code) · Share = Code / workspace Code
 Bars: █ Production  ▓ Test  ░ Other. The largest row fills the bar.
```

Compare saved reports of two releases, `v3.2.0` and `v3.3.4`:

<!-- showcase-compare -->
```text
 ──────────────────────────────────────────────────────────────────
                         Lines           │           Code
 Purpose        before    after   change │  before    after  change
 ────────────────────────────────────────┼─────────────────────────
 Production    208,007  210,708   +2,701 │ 177,458  179,557  +2,099
 Test           96,227  103,432   +7,205 │  81,161   87,011  +5,850
 Other          17,825   19,173   +1,348 │  14,910   16,165  +1,255
 ────────────────────────────────────────┼─────────────────────────
 Source total  322,059  333,313  +11,254 │ 273,529  282,733  +9,204
 Test/prod (Code): 0.46 → 0.48 (before → after)
```

Most of the change is test code. [Folder reporting](docs/guides/reports.md#look-inside-large-folders) and [comparison](docs/guides/compare.md) explain the remaining views.

## Terms

| Term | Meaning |
| --- | --- |
| Content kind | Each counted file is source, data such as JSON, or documentation such as Markdown, decided by its language. Tables and totals count source; data and documentation are reported separately. |
| Purpose | `production`, `test` or `other`. Production is maintained non-test source, including build scripts and tooling; `other` is source that counts as neither, such as generated code a policy sets aside or code nothing claimed. |
| Category | A finer label inside a purpose, written `purpose:category`, such as `production:implementation`, `production:build-tooling` or `test:test`. It changes no totals; `--show-categories` breaks tables down by it. |
| Unclaimed | Source that no convention, module or policy entry classified, reported as `other:unclaimed` and listed by folder for review. Its lines are counted; only its purpose is open. |
| Build module | A folder with a build or framework file, such as `package.json`, `Cargo.toml` or `next.config.ts`; its conventional source folders are production. See [build modules](#build-modules). |
| Convention | A built-in classification, such as files in `test/` being tests or `vite.config.ts` being build tooling. Conventions apply before the project's own rules; [built-in detection](docs/built-in-detection.md#test-conventions) lists them. |
| Policy | The project's `.locmetryrc`: scope exclusions and entries that classify what the built-ins cannot. `rules` apply after conventions, `overrides` before them. See [configuration](#configuration). |
| Provenance | The convention, module or entry that decided a file, recorded as its `rule`; `locmetry explain PATH` shows it. |
| Test/prod | Test code divided by production code, with Other left out. It describes volume, not test coverage. |
| Coverage | How many selected files were counted. Unrecognized or unreadable files are reported, never counted as zero. |
| Workspace, member | Several repositories, each under its own policy, or folders of one project under its policy, measured together into one report; each repository or folder is a member. |

[PRODUCT](docs/PRODUCT.md#measurement-model) defines these terms in full.

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

Without explicit options, a terminal gets a table and a pipe gets a compact JSON headline. `--output` defaults to full JSON; choose `--format table` or `--format markdown` explicitly for those exports. File exports are silent on stdout, and the extension does not select the format. For tables alone, add `--tables-only`: it drops legends, notes and the closing location and date lines, as for transcripts and scripts that need only rows. [Output routing and reports](docs/guides/reports.md#choose-content-and-destination) explains the choices and protected destinations.

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
| `--table-category LIST` | Filter table source and totals by category, such as `test,production:tooling` or `'production:*tooling*'`; a note names the excluded categories and their code. |
| `--table-language LIST` | Filter table source and totals by language, such as `TypeScript,Svelte`; a note names the excluded languages and their code. |
| `--split-languages` | Show language variants such as TSX, JSX and C++ Header as their own rows instead of within TypeScript, JavaScript and C++. |
| `--show-bars[=KIND]` / `--no-bars` | Force / suppress table charts. `KIND` is `purpose` or `comments`; selecting one table purpose uses comment density even with `=purpose`. Terminals show fitting bars automatically. |
| `--tables-only` | Print only the tables of a root, workspace or compare table report, without legends, notes or the closing location and date lines. |
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

The [comparison guide](docs/guides/compare.md) covers changed scope, producer versions, partial inputs and workspace changes. Reports describe measured working copies, not Git commits. For automated policies, use the [CI recipe](docs/guides/reports.md#continuous-integration), which preserves measurement failure and partial-result status before querying JSON; to limit file sizes by purpose, use the [file-size gate cookbook](docs/cookbooks/file-size-gate.md).

For file-level recognition and classification changes, run `compare-locmetry-files.sh before.json after.json`. This optional Bash [helper](scripts/compare-locmetry-files.sh) accepts two full `--files` reports and ships in release archives and Homebrew installations starting with 0.7.3. Only the helper requires `jq`; install it separately if needed. Locmetry's native commands remain independent of `jq`. See the [transition workflow](docs/guides/compare.md#inspect-configuration-transitions) for setup, output and evidence limits.

## How classification works

Locmetry embeds Tokei for counting and applies project policy to each file. Purpose, category and content kind are separate dimensions, as [Terms](#terms) explains. Kind follows the physical file, so a fenced code block in Markdown or other documentation counts as documentation, never as source; `--verbose` shows how many code lines such snippets hold. Rust doc comments, doctests included, stay comments of their file.

- **Module context:** selected build descriptors, such as npm, Gradle, Cargo, Go, Python or .NET project files, and framework descriptors, such as `next.config.ts` or Laravel's `artisan`, identify conventional source layouts, as [build modules](#build-modules) describes. The innermost module supplies context, so nested projects do not inherit an outer project's layout blindly.
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

Inspect the evidence for one file, inside the Immich checkout:

```sh
locmetry explain packages/sdk/src/fetch-client.ts
```

The explanation identifies the deciding stage, the matching pattern and the reason recorded with a project entry. It also reports parser, counts and module evidence where applicable. The [selection guide](docs/guides/selection.md) explains other states and how to investigate a missing file.

Built-in wrapper patterns cover `gradlew`, `gradlew.bat`, `mvnw`, `mvnw.cmd`, and files under `gradle/wrapper/` or `.mvn/wrapper/` at any depth. The enabled convention classifies them as `other` / `third-party-wrapper` unless an override or earlier convention wins; [wrapper details and precedence](docs/PRODUCT.md#matching-and-precedence) define the conditions.

Classification applies to whole files, so inline tests cannot receive a separate purpose. When Rust production files contain `#[cfg(test)]` items, a line below the language table names their code, such as “Production in 'Rust' includes 145 code lines of inline #[cfg(test)] tests in 3 files”, and full JSON reports them as `inline_tests`. [Measurement limits](docs/measurement.md) define this and other boundaries.

### Build modules

Locmetry recognizes build modules by their descriptors and counts each one's conventional source layout as production, with no configuration and across any mix of build systems in one tree:

| Build system | Descriptor | Production source |
| --- | --- | --- |
| npm | `package.json` | `src/**` |
| Gradle | `build.gradle`, `build.gradle.kts` | `src/main/**`, Kotlin Multiplatform `src/*Main/**` |
| Maven | `pom.xml` | `src/main/**` |
| Cargo | `Cargo.toml` | `src/**` |
| Go | `go.mod` | every `*.go` file outside `vendor/`, `testdata/` and names starting with `_` or `.` |
| Python | `pyproject.toml`, `setup.py`, `setup.cfg` | `src/**` and top-level packages |
| Lake (Lean) | `lakefile.lean`, `lakefile.toml` | every `*.lean` file |
| Swift | `Package.swift` | `Sources/**` |
| Dart and Flutter | `pubspec.yaml` | `lib/**`, `bin/**` |
| Next.js | `next.config.*` | `app/**`, `pages/**`, `src/**`, root `middleware`, `instrumentation` and similar files, and the folders the docs show, such as `components/**` and `lib/**` |
| Nuxt | `nuxt.config.*` | `app/**`, `server/**` and the other documented folders, `app.vue`, `error.vue` |
| React Router | `react-router.config.*` | `app/**` |
| .NET | `*.csproj`, `*.fsproj`, `*.vbproj` | every `*.cs`, `*.fs`, `*.vb`, `*.razor`, `*.cshtml` and `*.xaml` file, and `*.razor.css` |
| Composer | `composer.json` | `src/**` |
| Symfony | `symfony.lock` | `src/**`, `assets/**`, `config/**`, `migrations/**`, `templates/**` |
| Laravel | `artisan` | `app/**`, `bootstrap/**`, `config/**`, `database/migrations/**`, `routes/**`, `resources/**` except `lang/` |
| Rails | `config.ru` | `app/**`, `lib/**`, `config/**`, `db/migrate/**` |
| RubyGems | `*.gemspec` | `lib/**`, `exe/**` |
| Django | `manage.py` | top-level packages |

On Immich, `cd immich && locmetry --verbose` reports what the modules contributed:

<!-- showcase-modules -->
```text
 Production from build modules: 107,922 code lines in 8 'npm' modules, 59,170 code lines in 2 'dart' modules, 4,102 code lines in 1 'gradle' module, 3,037 code lines in 1 'python' module.
```

Tests inside these trees stay tests, and nested modules decide for themselves. Generated code inside these layouts that carries a documented marker, such as Go's `// Code generated … DO NOT EDIT.` line, Roslyn's `*.Designer.cs` names and `<auto-generated>` comments, Java's `@Generated` on a class or a `linguist-generated` declaration in `.gitattributes`, stays production with category `generated`, so `--show-categories` separates it from hand-written code. Before module layouts apply, built-in conventions recognize tests, such as `test/` folders, `*.test.ts`, `*Test.java`, `*_test.go`, `test_*.py` and `*_spec.rb`, and build tooling, such as Gradle, Maven, CMake and Make scripts, build wrappers and the configuration of named JavaScript tools. Detection reads file names only and never runs a build, so custom layouts need configuration. The [built-in detection reference](docs/built-in-detection.md) explains how detectors, markers and conventions decide and lists every one.

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
match = ["mobile/ios/**/*.swift", "mobile/ios/**/*.h"]
kind = ["source"]
purpose = "production"
category = "native"
reason = "Swift code and headers of the iOS runner and its extensions; no Package.swift declares it"
```

Tests still win over ordinary rules; an override is needed only to beat a convention. Entries without evidence belong in the review list as unclaimed source, not in the policy. A clean review list alone does not establish correct classification.

The [configuration guide](docs/guides/configure.md) covers overrides, custom categories, language mappings and evidence-based review. [Personal defaults](docs/PRODUCT.md#user-defaults) can supply exclusions and language mappings across projects; they cannot assign purposes or categories.

Locmetry's own [.locmetryrc](.locmetryrc) is a maintained example: it excludes the synthetic demonstration workspace and distinguishes operational tooling, distribution and verification infrastructure. [Self-measurement design](docs/self-measurement.md) explains the decisions and how to verify them.

For a production-rule change covered by the example in `locmetry --help`, inspect existing configuration and measure current decisions, adapt the paths and category, then run the displayed verification command. Reading a guide or schema first is optional for that route. Use `locmetry --guide configure` or `locmetry --schema config` for exclusions, overrides, intentional Other, or semantics not shown there. Verify per-file decisions and unresolved review items before concluding. Guides and schemas are available from the installed binary without a network connection.

## Guides and reference

| Task | Guide in the executable | Repository copy |
| --- | --- | --- |
| Configure or review classification | `locmetry --guide configure` | [Configure](docs/guides/configure.md) |
| Diagnose a file or subfolder | `locmetry --guide selection` | [Selection and explanations](docs/guides/selection.md) |
| Read coverage, query reports or apply a CI policy | `locmetry --guide reports` | [Reports](docs/guides/reports.md) |
| Capture baselines and interpret net change | `locmetry --guide compare` | [Compare](docs/guides/compare.md) |
| Gate source file sizes by purpose in CI | — | [File-size gate cookbook](docs/cookbooks/file-size-gate.md) |

- [Changelog](CHANGELOG.md): user-visible changes per release, with breaking changes marked.
- [Product contract](docs/PRODUCT.md): supported settings, fields and guarantees.
- [Built-in detection](docs/built-in-detection.md): every module detector, generated-code marker, convention and language mapping, and how they decide.
- [Measurement limits](docs/measurement.md): selection and counting boundaries.
- [Documentation index](docs/README.md): architecture, research and current work.
- [Contributor guidelines](CLAUDE.md): development conventions and checks.

## License

Licensed under the [MIT License](LICENSE). Release archives and the Homebrew installation include `THIRD-PARTY-NOTICES.md` with the licences of bundled libraries.
