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

Run `locmetry` in a project. The illustrative report below shows a hypothetical monorepo with Kotlin backend modules, a TypeScript/TSX frontend, Go workers, Python tooling and unresolved experiments.

The numbers are invented, not measured from a real checkout. Both tables use the same [data fixture](tests/fixtures/readme/README.md), rendered by Locmetry and checked for consistent totals.

**[Take the sample-project tour](docs/sample-project-tour.md)** to follow the layout, configuration, classification decisions, folder exploration and saved reports from start to finish.

```sh
locmetry
```

<!-- readme-language -->
```text
 acme-orders · Source code
 Language    Files    Lines     Code │ Comments  Blanks │ Production    Test  Unclassified
 ────────────────────────────────────┼──────────────────┼─────────────────────────────────
 Kotlin        538   93,760   74,060 │    7,530  12,170 │     46,040  28,020             0
 TypeScript    173   26,080   21,030 │    1,660   3,390 │     11,130   9,900             0
 TSX            83   16,580   14,020 │      780   1,780 │     14,020       0             0
 Go             90   17,250   13,770 │    1,300   2,180 │      8,050   5,720             0
 Python         55   10,330    7,660 │    1,340   1,330 │      4,760       0         2,900
 Shell           1      290      160 │       90      40 │          0       0           160
 ────────────────────────────────────┼──────────────────┼─────────────────────────────────
 Total         940  164,290  130,700 │   12,700  20,890 │     84,000  43,640         3,060
 Lines = Code + Comments + Blanks; Production + Test + Unclassified = Code.
```

Code is split by purpose; comments and blanks stay separate. JSON and Markdown files do not inflate source totals. The experiments and wrapper remain counted as Unclassified, for different [reasons](#how-classification-works).

To see where the code resides, add adaptive folder detail:

```sh
locmetry --folder-rows 30
```

The language table stays the same, followed by:

<!-- readme-folders -->
```text
 Folder                                      Lines     Code │ Production    Test  Unclassified │ Test/prod
 ───────────────────────────────────────────────────────────┼──────────────────────────────────┼──────────
 web/src/orders                             11,670    9,580 │      6,100   3,480             0 │      0.57
 backend/orders/src/test/kotlin             10,670    8,930 │          0   8,930             0 │         -
 backend/orders/src/main/kotlin/domain      11,396    8,722 │      8,722       0             0 │      0.00
 backend/billing/src/main/kotlin/domain      9,408    7,196 │      7,196       0             0 │      0.00
 web/src/catalog                             8,480    6,990 │      5,510   1,480             0 │      0.27
 web/src/billing                             8,510    6,980 │      4,520   2,460             0 │      0.54
 backend/billing/src/test/kotlin             8,130    6,740 │          0   6,740             0 │         -
 workers/billing                             7,820    6,260 │      3,420   2,840             0 │      0.83
 backend/catalog/src/main/kotlin/domain      7,021    5,474 │      5,474       0             0 │      0.00
 web/src/inventory                           5,520    4,530 │      3,510   1,020             0 │      0.29
 workers/inventory                           5,530    4,410 │      2,650   1,760             0 │      0.66
 backend/customers/src/main/kotlin           5,730    4,360 │      4,360       0             0 │      0.00
 backend/catalog/src/test/kotlin             5,170    4,250 │          0   4,250             0 │         -
 backend/inventory/src/main/kotlin/domain    5,460    4,186 │      4,186       0             0 │      0.00
 web/src/shipping                            4,640    3,810 │      2,970     840             0 │      0.28
 backend/orders/src/main/kotlin/api          4,884    3,738 │      3,738       0             0 │      0.00
 scripts/imports                             4,970    3,640 │      3,640       0             0 │      0.00
 backend/shipping/src/main/kotlin/domain     4,711    3,598 │      3,598       0             0 │      0.00
 backend/inventory/src/test/kotlin           4,180    3,460 │          0   3,460             0 │         -
 web/src/customers                           3,840    3,160 │      2,540     620             0 │      0.24
 workers/shipping                            3,900    3,100 │      1,980   1,120             0 │      0.57
 backend/billing/src/main/kotlin/api         4,032    3,084 │      3,084       0             0 │      0.00
 prototypes                                  3,800    2,900 │          0       0         2,900 │         -
 backend/shipping/src/test/kotlin            3,230    2,680 │          0   2,680             0 │         -
 backend/catalog/src/main/kotlin/api         3,009    2,346 │      2,346       0             0 │      0.00
 backend/customers/src/test/kotlin           2,370    1,960 │          0   1,960             0 │         -
 backend/inventory/src/main/kotlin/api       2,340    1,794 │      1,794       0             0 │      0.00
 backend/shipping/src/main/kotlin/api        2,019    1,542 │      1,542       0             0 │      0.00
 scripts/maintenance                         1,560    1,120 │      1,120       0             0 │      0.00
 (root files)                                  290      160 │          0       0           160 │         -
 ───────────────────────────────────────────────────────────┼──────────────────────────────────┼──────────
 Total                                     164,290  130,700 │     84,000  43,640         3,060 │      0.52
 Production + Test + Unclassified = Code; Test/prod = Test / Production.
 Adaptive folder detail; rows cover measured source once.
```

The rows cover the measured source once: a parent and its expanded children do not contribute twice. `Test/prod` is test code divided by production code; `-` means there is no production denominator. It describes code volume, not test coverage or quality.

The 30-row budget limits additional detail; existing top-level rows are retained even if they exceed it. Here some backend folders are expanded while `prototypes` and `scripts/imports` remain grouped. Use `--folders` for just top-level folders, or `--verbose` for classification and coverage notes. These views do not change the measurement. [Folder reporting](docs/guides/reports.md#look-inside-large-folders) explains expansion, saved tables and format limits.

## Common commands

| Task | Command |
| --- | --- |
| Measure another project | `locmetry --root ../service` |
| Inspect scope, classification and coverage notes | `locmetry --verbose` |
| List the data and documentation files behind “Not source code” | `locmetry --files --output -` with a [`jq` filter on `kind`](docs/guides/reports.md#not-source-code-which-files-are-data-or-documentation) |
| Measure a subfolder with project policy and module context | `locmetry --subfolder web/src` |
| Explain a file's classification or missing counts | `locmetry explain web/src/orders/Order.test.ts` |
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

These example paths illustrate how the rules apply in a project with the corresponding module descriptors and [configuration](#configuration):

| File | Purpose / category | Deciding rule |
| --- | --- | --- |
| `backend/orders/src/main/kotlin/domain/Order.kt` | production / implementation | `builtin:gradle-module` |
| `web/src/orders/Order.test.ts` | test / test | `builtin:js-test-file` |
| `scripts/imports/orders/order.py` | production / tooling | `tooling` (project rule) |
| `gradlew` | unclassified / third-party-wrapper | `builtin:build-wrapper` |
| `prototypes/pricing/candidate_1.py` | unclassified / unclassified | `fallback` |

Inspect the evidence for one file:

```sh
locmetry explain web/src/orders/Order.test.ts
```

The explanation identifies `builtin:js-test-file`, the matching pattern and the convention stage. It also reports parser, counts and module evidence where applicable. The [selection guide](docs/guides/selection.md) explains other states and how to investigate a missing file.

Built-in wrapper patterns cover `gradlew`, `gradlew.bat`, `mvnw`, `mvnw.cmd`, and files under `gradle/wrapper/` or `.mvn/wrapper/` at any depth. The enabled convention classifies them as `unclassified` / `third-party-wrapper` unless an override or earlier convention wins; [wrapper details and precedence](docs/PRODUCT.md#matching-and-precedence) define the conditions.

Module inference examines filenames and conventional layouts, not build-file contents, and never executes a build. Custom layouts need configuration. Classification applies to whole files, so inline tests cannot receive a separate purpose. [Module detection](docs/PRODUCT.md#module-detection) and [measurement limits](docs/measurement.md) define these boundaries.

### Configuration

No configuration is required to start. Add `.locmetryrc` when conventions cannot know local intent: which code is vendored, which scripts are maintained tooling, or which experiments should stay explicitly unclassified. Locmetry does not automatically recognize vendored or generated code.

For the layout illustrated above:

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

This removes `third_party/` from scope and classifies `scripts/` as production tooling. Tests still win over ordinary rules. The experiments remain in the review list; assigning them an explicit category such as `prototype` with purpose `unclassified` would retain their counts but remove them from that list. The wrapper is already deliberately unclassified. A clean review list alone does not establish correct classification.

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
