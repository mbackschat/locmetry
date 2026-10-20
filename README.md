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

Run `locmetry` in a project. These table excerpts come from the checked-in [synthetic Acme Orders workspace](examples/cli-workspace/README.md): a tiny Go service and TypeScript/TSX web client, with Python tooling and an unresolved experiment. Tests run the CLI against its actual source files and check the published tables.

**[Take the sample-project tour](docs/sample-project-tour.md)** to follow configuration, classification decisions, folder exploration and saved reports. The [recorded command guides](docs/cli/README.md) show reproducible commands and their captured output.

From this repository's root:

```sh
locmetry --root examples/cli-workspace/acme-orders --no-user-config --no-color --no-bars --format table
```

The excerpts omit the root-location and measurement-time footer.

<!-- readme-language -->
```text
 ──────────────────────────────────────────────────────────────────────────────────
 Language    Files  Lines  Code │ Comments  Blanks │ Prod.  Test  Other │ Test/prod
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 Go              2     11     8 │        1       2 │     4     4      0 │      1.00
 TypeScript      2      8     6 │        2       0 │     3     3      0 │      1.00
 HTML            1      4     4 │        0       0 │     4     0      0 │      0.00
 Python          2      5     3 │        2       0 │     2     0      1 │      0.00
 TSX             1      3     3 │        0       0 │     3     0      0 │      0.00
 JavaScript      0      1     1 │        0       0 │     1     0      0 │      0.00
 ───────────────────────────────┼──────────────────┼────────────────────┼──────────
 Total           8     32    25 │        5       2 │    17     7      1 │      0.41
 Lines = Code + Comments + Blanks; Production + Test + Other = Code; Test/prod = Test / Production (Code).
 Tests written in another language than the code they test skew per-language Test/prod.
```

Code is split by purpose; comments and blanks stay separate. JSON and Markdown files do not inflate source totals. The HTML file contributes both HTML and JavaScript; its embedded JavaScript row adds code with zero additional physical files. The experiment remains Other/unclaimed. TSX has no test code in the same language, illustrating why a per-language ratio is not test coverage.

To see where the code resides, add adaptive folder detail:

```sh
locmetry --root examples/cli-workspace/acme-orders --no-user-config --no-color --no-bars --format table --folder-rows 12
```

<!-- readme-folders -->
```text
 ────────────────────────────────────────────────────────────────────
 Folder                  Lines  Code │ Prod.  Test  Other │ Test/prod
 ────────────────────────────────────┼────────────────────┼──────────
 web/src/orders             11     9 │     6     3      0 │      0.50
 web/src (direct files)      5     5 │     5     0      0 │      0.00
 service/src                 6     4 │     4     0      0 │      0.00
 service/tests               5     4 │     0     4      0 │         -
 service/scripts             3     2 │     2     0      0 │      0.00
 service/experiments         2     1 │     0     0      1 │         -
 ────────────────────────────────────┼────────────────────┼──────────
 Total                      32    25 │    17     7      1 │      0.41
 Production + Test + Other = Code; Test/prod = Test / Production (Code).
 Adaptive folder detail; rows cover measured source once.
```

The rows cover the measured source once: a parent and its expanded children do not contribute twice. `Test/prod` is test code divided by production code; `-` means there is no production denominator. It describes code volume, not test coverage or quality.

The 12-row budget allows this small tree to expand to six source rows. Use `--folders` for top-level folders or `--verbose` for classification and coverage notes. These views do not change the measurement. [Folder reporting](docs/guides/reports.md#look-inside-large-folders) explains expansion, saved tables and format limits.

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
| Count production source only, with its comment density | `locmetry --purpose production` |
| Inspect the configuration contract | `locmetry --schema config` |
| Read full JSON on stdout | `locmetry --root service --format json --files` |
| Measure selected source, including ignored files, except vendor | `locmetry --root service --subfolder src --exclude 'src/vendor/**' --hidden true --respect-ignore-files false --format json` |
| Show adaptive folder detail and production categories | `locmetry --root service --format table --folder-rows 30 --purpose production --categories --bars=comments --sort name` |
| Save a sorted Markdown tree to depth 3 | `locmetry --root service --format markdown --depth 3 --sort name --output report.md` |
| Measure an explicit workspace definition | `locmetry workspace --file repos.toml --files --output workspace.json --print table` |
| Compare two saved full reports | `locmetry compare before.json after.json --format table` |

`locmetry --help` includes the same commands, plus a working configuration example for maintained scripts with applicability and verification instructions. Verbose tables total data and documentation files under “Not source code”; to list them, [filter full JSON by `kind`](docs/guides/reports.md#not-source-code-which-files-are-data-or-documentation).

Without explicit options, a terminal gets a table and a pipe gets a compact JSON headline. `--output` defaults to full JSON; choose `--format table` or `--format markdown` explicitly for those exports. File exports are silent on stdout, and the extension does not select the format. [Output routing and reports](docs/guides/reports.md#choose-content-and-destination) explains the choices and protected destinations.

### Selected options

| Option | Use |
| --- | --- |
| `--folders` / `--folder-rows N` | Top-level / adaptive folder rows in single-root table output; adaptive rows require table format. |
| `--no-color` | Disable terminal colors, bold and dim text, including help and diagnostics; equivalent to `NO_COLOR=1`. |
| `--verbose` | Add diagnostic notes to tables independently of folder rows. |
| `--bars` / `--no-bars` | Share bars per language, folder or repository row, split by purpose. A terminal shows them beside tables they fit; `--bars` also charts them below other tables and in saved tables, `--bars=comments` charts comment density instead, and `--no-bars` omits them. |
| `--purpose LIST` | Count only these purposes' source in tables, such as `production` to drop tests; without test or production, `Comment %` replaces `Test/prod`. |
| `--config FILE` | Replace discovery of the root's `.locmetryrc`; patterns still refer to the scan root. |
| `--exclude GLOB` | Add an exclusion to the effective scope. |
| `--no-user-config` | Ignore personal defaults, for example when measuring in CI. |
| `--files` | Add per-file counts and deciding rules to full JSON. |
| `--categories` | Show the source purpose/category breakdown in a table. |

Run `locmetry --help` or `locmetry COMMAND --help` for all options. The positional directory is equivalent to `--root`; in scripts, prefer `--root "$directory"` so a directory named `compare` or `workspace` cannot select a command. Command options follow the command name; global `--no-color` works before or after it.

### For agents

One call gives an agent both a readable overview and a saved report:

```sh
locmetry --root . --output ../report.json --print table
```

Stdout gets the table, unstyled and without share bars in a pipe; the file gets full JSON for queries or a later `locmetry compare`. Use `--print headline` for compact JSON on stdout instead, and `locmetry --schema report` for every field. `--folders` and `--verbose` extend the printed table.

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

## Analyze saved measurements

Full schema-3 reports support external analytics without adding runtime dependencies to Locmetry. Capture reports with the current executable; older schema-1/2 measurements must be recaptured.

- [DuckDB cookbook](docs/duckdb-cookbook.md): typed SQL views, portfolio totals, trends, ownership and Parquet export.
- [pandas and Polars cookbook](docs/dataframe-cookbook.md): equivalent dataframe analysis with optional external packages.
- [Recorded analytics demo](docs/analytics-demo.md): replay a capture and query against the shared synthetic workspace.

## How classification works

Locmetry embeds Tokei for counting and applies project policy to each file. Purpose (`production`, `test`, `other`), category (such as `implementation` or `tooling`) and content kind (source, data, documentation or unknown) are separate dimensions. `other` means neither production nor test; code no stage claimed is `other` with category `unclaimed`.

- **Module context:** selected npm, Gradle, Maven, Cargo, Go, Python, Lake and Swift descriptors identify conventional source layouts. The innermost module supplies context, so nested projects do not inherit an outer project's layout blindly.
- **Ordered decisions:** explicit overrides precede test and build conventions, ordinary project rules, module inference and fallback. Tests inside production source trees remain tests; overrides can express exceptions. Every counted file records the deciding rule.
- **Scope and coverage:** exclusions and ignore rules determine what is selected. Unrecognized files and unavailable counts remain distinguishable from zero. Read/parse failures produce a partial report and exit 2; unresolved classification alone does not make counts partial.

These example paths illustrate how the rules apply in a project with the corresponding module descriptors and [configuration](#configuration):

| File | Purpose / category | Deciding rule |
| --- | --- | --- |
| `service/src/main.go` | production / implementation | `builtin:go-module` |
| `web/src/orders/Order.test.ts` | test / test | `builtin:js-test-file` |
| `service/scripts/check.py` | production / tooling | `tooling` (project rule) |
| `service/experiments/probe.py` | other / unclaimed | `fallback` |

Inspect the evidence for one file:

```sh
locmetry explain web/src/orders/Order.test.ts \
  --root examples/cli-workspace/acme-orders --no-user-config
```

The explanation identifies `builtin:js-test-file`, the matching pattern and the convention stage. It also reports parser, counts and module evidence where applicable. The [selection guide](docs/guides/selection.md) explains other states and how to investigate a missing file.

Built-in wrapper patterns cover `gradlew`, `gradlew.bat`, `mvnw`, `mvnw.cmd`, and files under `gradle/wrapper/` or `.mvn/wrapper/` at any depth. The enabled convention classifies them as `other` / `third-party-wrapper` unless an override or earlier convention wins; [wrapper details and precedence](docs/PRODUCT.md#matching-and-precedence) define the conditions.

Module inference examines filenames and conventional layouts, not build-file contents, and never executes a build. Custom layouts need configuration. Classification applies to whole files, so inline tests cannot receive a separate purpose. [Module detection](docs/PRODUCT.md#module-detection) and [measurement limits](docs/measurement.md) define these boundaries.

### Configuration

No configuration is required to start. Add `.locmetryrc` when conventions cannot know local intent: which code is vendored, which scripts are maintained tooling, or which experiments should explicitly count as `other`. Locmetry does not automatically recognize vendored or generated code.

For the combined Acme Orders root:

```toml
version = 1

[scope]
hidden = false
exclude = ["service/third_party/**"]

[[rules]]
id = "tooling"
match = ["service/scripts/**"]
purpose = "production"
category = "tooling"
```

This removes `service/third_party/` from scope and classifies `service/scripts/` as production tooling. Tests still win over ordinary rules. The experiment remains in the review list; assigning them an explicit category such as `prototype` with purpose `other` would retain their counts but remove them from that list. A clean review list alone does not establish correct classification.

The [configuration guide](docs/guides/configure.md) covers overrides, custom categories, language mappings and evidence-based review. [Personal defaults](docs/PRODUCT.md#user-defaults) can supply exclusions and language mappings across projects; they cannot assign purposes or categories.

For a production-rule change covered by the example in `locmetry --help`, inspect existing configuration and measure current decisions, adapt the paths and category, then run the displayed verification command. Reading a guide or schema first is optional for that route. Use `locmetry --guide configure` or `locmetry --schema config` for exclusions, overrides, intentional Other, or semantics not shown there. Verify per-file decisions and unresolved review items before concluding. Guides and schemas are available from the installed binary without a network connection.

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
