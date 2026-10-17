<img src="docs/assets/locmetry.webp" alt="locmetry logo" width="96">

# locmetry

Locmetry counts code, comments and blank lines, classifies files as production, test or unclassified, and records the rule behind each classification. Reports separate source code from data and documentation and show results by language and folder.

## Install

With Homebrew on macOS arm64 or Linux x86_64 and aarch64:

```sh
brew install mbackschat/tap/locmetry
```

Or download a binary from the [latest release](https://github.com/mbackschat/locmetry/releases/latest). Locmetry is a self-contained executable with no separately installed runtime or counter. Linux binaries are static; macOS uses operating-system libraries. See [manual installation and source builds](docs/tokei-integration.md#installation) for alternatives.

## What you see

Run `locmetry` in a project. This [synthetic sample](tests/fixtures/readme/acme-orders) has a Kotlin backend, a TypeScript client, a maintained Python script, a prototype and a build wrapper. Its [configuration](#configuration) excludes a vendored SDK and identifies the script as tooling. The tables use an illustrative measurement date.

```sh
locmetry
```

<!-- readme-language -->
```text
 acme-orders · Source code
 Language    Files  Lines  Code │ Comments  Blanks │ Production  Test  Unclassified
 ───────────────────────────────┼──────────────────┼───────────────────────────────
 Kotlin          3     14    11 │        1       2 │          7     4             0
 TypeScript      2      8     7 │        0       1 │          3     4             0
 Python          2      6     5 │        1       0 │          3     0             2
 Shell           1      3     1 │        2       0 │          0     0             1
 ───────────────────────────────┼──────────────────┼───────────────────────────────
 Total           8     31    24 │        4       3 │         13     8             3
 Lines = Code + Comments + Blanks; Production + Test + Unclassified = Code.

Measured at: 2026-10-17 12:00:00 UTC
```

Code is split by purpose; comments and blanks stay separate. JSON and Markdown files do not inflate source totals. The prototype and wrapper remain counted as Unclassified, for different [reasons](#how-classification-works).

To see where the code resides, add adaptive folder detail:

```sh
locmetry --folder-rows 30
```

The language table stays the same, followed by:

<!-- readme-folders -->
```text
 Folder                   Lines  Code │ Production  Test  Unclassified │ Test/prod
 ─────────────────────────────────────┼────────────────────────────────┼──────────
 web/src                      8     7 │          3     4             0 │      1.33
 backend/src/main/kotlin      6     4 │          4     0             0 │      0.00
 backend/src/test/kotlin      5     4 │          0     4             0 │         -
 backend (direct files)       3     3 │          3     0             0 │      0.00
 scripts                      4     3 │          3     0             0 │      0.00
 prototypes                   2     2 │          0     0             2 │         -
 (root files)                 3     1 │          0     0             1 │         -
 ─────────────────────────────────────┼────────────────────────────────┼──────────
 Total                       31    24 │         13     8             3 │      0.62
 Production + Test + Unclassified = Code; Test/prod = Test / Production.
 Adaptive folder detail; rows cover measured source once.

Measured at: 2026-10-17 12:00:00 UTC
```

The rows cover the measured source once: a parent and its expanded children do not contribute twice. `Test/prod` is test code divided by production code; `-` means there is no production denominator. It describes code volume, not test coverage or quality.

The 30-row budget limits additional detail; existing top-level rows are retained even if they exceed it. Small projects can produce fewer rows. Use `--folders` for just top-level folders, or `--verbose` for classification and coverage notes. These views do not change the measurement. [Folder reporting](docs/guides/reports.md#look-inside-large-folders) explains expansion, saved tables and format limits.

## Common commands

| Task | Command |
| --- | --- |
| Measure another project | `locmetry --root ../service` |
| Inspect scope, classification and coverage notes | `locmetry --verbose` |
| Measure a subfolder with project policy and module context | `locmetry --subfolder web/src` |
| Explain a file's classification or missing counts | `locmetry explain web/src/cart.test.ts` |
| Save full JSON with per-file decisions outside the root | `locmetry --files --output ../report.json` |
| Save a Markdown report | `locmetry --format markdown --output ../report.md` |
| Measure repositories together, each with its own policy | `locmetry workspace --root ../web --root ../api` |
| Inspect the configuration contract | `locmetry --schema config` |

Without explicit options, a terminal gets a table and a pipe gets a compact JSON headline. `--output` defaults to full JSON; choose `--format table` or `--format markdown` explicitly for those exports. File exports are silent on stdout, and the extension does not select the format. [Output routing and reports](docs/guides/reports.md#choose-content-and-destination) explains the choices and protected destinations.

### Selected options

| Option | Use |
| --- | --- |
| `--folders` / `--folder-rows N` | Top-level / adaptive folder rows in single-root table output; adaptive rows require table format. |
| `--verbose` | Add diagnostic notes to tables independently of folder rows. |
| `--config FILE` | Replace discovery of the root's `.locmetryrc`; patterns still refer to the scan root. |
| `--exclude GLOB` | Add an exclusion to the effective scope. |
| `--no-user-config` | Ignore personal defaults, for example when measuring in CI. |
| `--files` | Add per-file counts and deciding rules to full JSON. |
| `--categories` | Show the source purpose/category breakdown in a table. |

Run `locmetry --help` or `locmetry COMMAND --help` for all options. The positional directory is equivalent to `--root`; in scripts, prefer `--root "$directory"` so a directory named `compare` or `workspace` cannot select a command. Command options follow the command name.

### Compare saved measurements

Save full baselines outside the measured root; a headline cannot reconstruct a baseline. Net change is after minus before:

```sh
locmetry --root ./service --output ./before.json
# After changing the source or its measurement policy:
locmetry --root ./service --output ./after.json
locmetry compare before.json after.json
```

The [comparison guide](docs/guides/compare.md) covers changed scope, producer versions, partial inputs and workspace changes. Reports describe measured working copies, not Git commits. For automated policies, use the [CI recipe](docs/guides/reports.md#continuous-integration), which preserves measurement failure and partial-result status before querying JSON.

## How classification works

Locmetry embeds Tokei for counting and applies project policy to each file. Purpose (`production`, `test`, `unclassified`), category (such as `implementation` or `tooling`) and content kind (source, data, documentation or other) are separate dimensions.

- **Module context:** selected npm, Gradle, Maven, Cargo, Go, Python, Lake and Swift descriptors identify conventional source layouts. The innermost module supplies context, so nested projects do not inherit an outer project's layout blindly.
- **Ordered decisions:** explicit overrides precede test and build conventions, ordinary project rules, module inference and fallback. Tests inside production source trees remain tests; overrides can express exceptions. Every counted file records the deciding rule.
- **Scope and coverage:** exclusions and ignore rules determine what is selected. Unrecognized files and unavailable counts remain distinguishable from zero. Read/parse failures produce a partial report and exit 2; unresolved classification alone does not make counts partial.

In the sample:

| File | Purpose / category | Deciding rule |
| --- | --- | --- |
| `backend/src/main/kotlin/Order.kt` | production / implementation | `builtin:gradle-module` |
| `web/src/cart.test.ts` | test / test | `builtin:js-test-file` |
| `scripts/seed_db.py` | production / tooling | `tooling` (project rule) |
| `gradlew` | unclassified / third-party-wrapper | `builtin:build-wrapper` |
| `prototypes/pricing_experiment.py` | unclassified / unclassified | `fallback` |

Inspect the evidence for one file:

```sh
locmetry explain web/src/cart.test.ts
```

The explanation identifies `builtin:js-test-file`, the matching pattern and the convention stage. It also reports parser, counts and module evidence where applicable. The [selection guide](docs/guides/selection.md) explains other states and how to investigate a missing file.

Built-in wrapper patterns cover `gradlew`, `gradlew.bat`, `mvnw`, `mvnw.cmd`, and files under `gradle/wrapper/` or `.mvn/wrapper/` at any depth. The enabled convention classifies them as `unclassified` / `third-party-wrapper` unless an override or earlier convention wins; [wrapper details and precedence](docs/PRODUCT.md#matching-and-precedence) define the conditions.

Module inference examines filenames and conventional layouts, not build-file contents, and never executes a build. Custom layouts need configuration. Classification applies to whole files, so inline tests cannot receive a separate purpose. [Module detection](docs/PRODUCT.md#module-detection) and [measurement limits](docs/measurement.md) define these boundaries.

### Configuration

No configuration is required to start. Add `.locmetryrc` when conventions cannot know local intent: which code is vendored, which scripts are maintained tooling, or which experiments should stay explicitly unclassified. Locmetry does not automatically recognize vendored or generated code.

The sample uses:

<!-- readme-config -->
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

This removes `third_party/` from scope and classifies `scripts/` as production tooling. Tests still win over ordinary rules. The prototype remains in the review list; assigning it an explicit category such as `prototype` with purpose `unclassified` would retain its counts but remove it from that list. The wrapper is already deliberately unclassified. A clean review list alone does not establish correct classification.

The [configuration guide](docs/guides/configure.md) covers overrides, custom categories, language mappings and evidence-based review. [Personal defaults](docs/PRODUCT.md#user-defaults) can supply exclusions and language mappings across projects; they cannot assign purposes or categories.

For a coding agent, start with: “Create or evaluate a locmetry configuration for this repository. Read `locmetry --guide configure` and `locmetry --schema config` first; report the evidence for decisions and leave unresolved questions explicit.” Guides and schemas are available from the installed binary without a network connection.

## Compared with related tools

| Tool | Relationship and documented capabilities |
| --- | --- |
| [Tokei](https://github.com/XAMPPRocky/tokei) | Supplies Locmetry's embedded counting backend. |
| [SLOC](https://github.com/flosse/sloc) | Line-count statistics through a CLI and library. |
| [cloc](https://github.com/AlDanial/cloc) | Source comparison and filtering diagnostics alongside counting. |
| [scc](https://github.com/boyter/scc) | Counting with additional analytics and integrations. |
| [ALOC](https://github.com/modern-tooling/aloc) | Counting with classification by role. |

Locmetry combines reviewable scope, whole-file purpose/category classification with provenance, source/data/documentation separation, explicit coverage and independently configured workspace members. The dated [capability research](docs/research/tool-capability-comparison.md) examines several related tools and its stated versions; it is not a performance ranking or a current comparison of every tool named here.

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
