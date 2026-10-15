# Product Vision & Requirements

## Vision

Measure a project's own codebase with understandable scope, test/production distinctions, and useful folder totals. Locmetry is a policy-and-reporting layer over an established counter. Agents consume predictable JSON; developers get readable summaries and reviewable project configuration.

The product rests on three pillars:

- **Expressive configuration:** reviewable project policy covers real repositories' scope, ownership, and classification needs, including build tooling, examples, generated code, and evidence.
- **Reuse with gap filling:** the embedded counter does the counting; Locmetry layers on top of its library and fills gaps it leaves, such as unrecognized file types, instead of reimplementing it.
- **Rich output:** reports summarize the measurement, while JSON keeps every file record, provenance, and the effective policy so agents and scripts can answer further questions, including their own summations.

## Scope and Status

- Required: embedded Tokei integration in a Rust executable, test/production classification, production source derived from build descriptors, project configuration, personal defaults, and CLI options, a configuration schema that explains its semantics, configurable exclusions, language mappings for files the backend does not recognize, folder breakdowns and sums, a workspace mode that merges several repositories, JSON, and human output that locates unclassified source.
- Optional: further module detectors, additional counting adapters, custom content-kind overrides, and configuration self-checks that assert expected classifications for named paths without requiring those files to exist.
- Excluded initially: a new counting engine, a generic plugin framework (module detectors are compiled-in implementations of one trait, not loadable plugins), and semantic splitting of production/tests within individual files.

Locmetry is a Rust executable with the Tokei library embedded. The installed executable must not require Python, Git, a separate Tokei executable, or a separately installed Rust runtime; the Homebrew formula installs the prebuilt release executable without runtime formula dependencies, needing Rust only to build `--HEAD`. [Tokei integration and distribution](docs/tokei-integration.md) records the backend boundary and packaging.

## Measurement Model

| Concept | Meaning |
| --- | --- |
| Scope | Whether a file belongs in the measurement; external/reference material is an inclusion decision, not a test category. |
| Kind | `source`, `data`, `documentation`, or `other`, derived from the backend language through a reviewed mapping; a JSON test fixture remains data. |
| Purpose | `production`, `test`, or `unclassified`. Production means maintained non-test source, not necessarily shipped runtime code. |
| Category | Optional project-defined label such as implementation, tooling, test-support, example, benchmark, or evidence. It does not change counting. |

Report code, comments, blanks, and total lines separately. Preserve each file's language and classification provenance; a language supplied by a mapping records the mapping ID. Embedded snippets inherit the host file's scope and purpose; documentation snippets do not inflate source totals. Documentation embedded in a non-documentation file, such as Rust `///` doc comments that the backend parses as Markdown, contributes its lines, including doctest code, as comments of the host language. Derive human summaries and versioned JSON from the same records.

## Project Configuration

Use `.locmetryrc` in TOML. Discover it in the selected scan root (`--root`), which defaults to the current directory; an explicit `--config` path may select a different file without changing that root. Without a discovered or explicit configuration, built-in settings apply; an explicit configuration that cannot be read fails. A loaded configuration inside the scan root is policy, not project content: it is excluded as a literal path, which the effective exclusions record. All patterns remain relative to the scan root. How it combines with built-in settings, user defaults, and command-line flags is defined in [Matching and Precedence](#matching-and-precedence). A configuration file ending in `.json`, the retired JSON/regex profile format, is rejected with a hint to translate it to TOML.

`locmetry --schema config` prints the JSON Schema (Draft 2020-12) of this file, `--schema user-config` that of personal defaults, and `--schema workspace` that of a [workspace](#workspace-mode) file, then exits; `--schema report` describes the report (see [Reporting and Coverage](#reporting-and-coverage)). The schema is generated from the parser's types, so it accepts the same keys, types, and enums; checks it cannot express, such as glob syntax and unique IDs, still fail when the file loads. Its descriptions state each setting's default and meaning, the classification order, and this version's built-in conventions, build-module layouts, and language names, so an agent can write a configuration from the binary alone.

This example is synthetic:

```toml
version = 1
backend = "tokei"

[scope]
hidden = true
respect_ignore_files = true
exclude = ["reference-material/**"]

[classification]
builtin_tests = true
fallback = "unclassified"

[languages]
builtin = true

[categories]
evidence = "Archived probes kept for provenance, not maintained."
test-support = "Helpers that exist only for tests."
tooling = "Maintained scripts that build, check, or release the project."
unmapped-source = "Source no entry claims yet."

[[languages.mappings]]
id = "runbooks"
match = ["scripts/*.runbook"]
language = "BASH"

[[overrides]]
id = "archived-evidence"
match = ["docs/archive/**"]
kind = ["source"]
purpose = "unclassified"
category = "evidence"
reason = "Archived probes are kept for provenance, not maintained."

[[overrides]]
id = "shared-test-support"
match = ["packages/test-tools/**"]
purpose = "test"
category = "test-support"

[[rules]]
id = "maintained-tooling"
match = ["scripts/**"]
purpose = "production"
category = "tooling"

[[rules]]
id = "application-source"
match = ["src/**", "packages/*/{src,bin}/**"]
purpose = "production"
category = "implementation"

[[rules]]
id = "unmapped-source"
match = ["**"]
kind = ["source"]
purpose = "unclassified"
category = "unmapped-source"
```

### User Defaults

Personal defaults live in `$XDG_CONFIG_HOME/locmetry/config.toml`, or in `~/.config/locmetry/config.toml` when `XDG_CONFIG_HOME` is unset, on every platform. They may only narrow scope and teach languages, never classify, so a project is classified the same way for everyone:

```toml
version = 1

[scope]
exclude = ["**/*.generated.ts"]

[[languages.mappings]]
id = "runbooks"
match = ["**/*.runbook"]
language = "BASH"
```

- `version` is required and must be `1`. `scope.exclude` is added after the default exclusions and before the project's. `languages.mappings` apply after the project's mappings and before the built-in ones; mapping IDs must be unique across both files.
- Any other key fails, as does invalid content; a missing file is ignored.
- Reports record the file as `metadata.user_config_path`, or `null`. `--no-user-config` skips it, for CI or for comparing measurements across machines.
- Excluded: user-level classification entries, scope booleans, and nested project configurations.

### Module Detection

Production source is evidence from the build, not a guess: a build descriptor among the selected files makes its directory a build module, and each build system's default source layout inside that module is production. Build systems call this unit a module, project, package, or crate; it is unrelated to language-level modules or packages, and this document's "module" always means a build module. Detectors only read selected file names and never run a build. Each build system is one detector, so supporting another one adds a detector without changing the stages.

| Detector | Descriptor | Production source in the module |
| --- | --- | --- |
| `npm` | `package.json` | `src/**` |
| `gradle` | `build.gradle`, `build.gradle.kts` | `src/main/**`, Kotlin Multiplatform `src/*Main/**` |
| `maven` | `pom.xml` | `src/main/**` |
| `cargo` | `Cargo.toml` | `src/**` |
| `go` | `go.mod` | every `*.go` file |
| `python` | `pyproject.toml`, `setup.py`, `setup.cfg` | `src/**`, plus top-level packages with `__init__.py` as setuptools discovers them, excluding names such as `docs`, `scripts`, and `tools` |
| `lake` | `lakefile.lean`, `lakefile.toml` | every `*.lean` file; the lakefile is a build script, and test libraries named `<Name>Test/` or `<Name>Tests/` are tests through the `lean-test-library` convention |
| `swift` | `Package.swift` | `Sources/**`; test targets in `Tests/` or `<Name>Tests/` are tests through the `swift-test-target` convention |

The innermost module containing a file decides, as build tools treat nested projects: an outer module's layout never claims files of a module nested inside it, while build systems sharing one directory, such as Gradle and npm, combine their layouts. Where those layouts overlap, the detector whose ID sorts first decides, such as `cargo` before `npm`; purpose and category are the same either way, so only provenance and the per-module totals depend on it. Test and build conventions run earlier, so tests inside these trees stay tests. Files outside every module's production source keep the fallback, which then means the build declares nothing about them. The report file lists the modules as `metadata.modules`; the summary's `build_modules`, the table, and the Markdown report, which lists every module, show the code each one classified, where `0` hints at a misleading build file.

### Matching and Precedence

Four layers combine into the effective configuration; user defaults and the command line never classify, so a project is classified the same way for everyone:

| Layer | Source | May set | Combination |
| --- | --- | --- | --- |
| Built-in | `version` | conventions, language mappings, default exclusions | applies unless the project switches it off |
| User defaults | [User Defaults](#user-defaults) file | `scope.exclude`, `languages.mappings` | exclusions are added; mappings apply after the project's and before the built-in ones |
| Project | `.locmetryrc` in the scan root, or the file `--config` names | every key | exactly one file: `--config` replaces discovery, and other `.locmetryrc` files are never read |
| Command line | `--exclude GLOB`, `--hidden BOOL`, `--respect-ignore-files BOOL`, `--no-user-config` | scope | exclusions are added; the booleans replace the project's; `--no-user-config` drops the user layer; in [workspace mode](#workspace-mode) only `--no-user-config` is accepted |

Exclusions from all layers form one union, so their order is irrelevant and no layer can re-include a file another excluded. Every file then takes the first matching stage:

1. Apply scope exclusions and the selected ignore-file policy.
2. Apply the first matching `overrides` entry.
3. Apply built-in test conventions, then built-in build conventions, if enabled; provenance records the convention, for example `builtin:gradle-test-source-set`.
4. Apply the first matching ordinary `rules` entry.
5. Apply module detection, if enabled: source in the production layout of a module that a build descriptor declares is `production` with category `implementation`, and provenance names the detector, for example `builtin:npm-module`.
6. Assign `classification.fallback` if no classification matched.

An entry matches when any of its patterns matches the path and, if `kind` is present, the file's kind is listed. A match terminates purpose/category classification. Each file records its provenance as the matching entry ID, `builtin:<convention>`, or `fallback`; fallback files take the fallback purpose as their category. Broad production rules therefore do not swallow conventional tests or build scripts, while an explicit archived-evidence override can take precedence over test filenames. Entries that assign ownership of a whole tree, such as a test-only package whose build scripts are test support, belong in `overrides` so built-in conventions do not preempt them. No ordinary run requires an LLM; agent assistance proposes reviewable configuration.

The stage order is fixed, so where `[[overrides]]`, `[[rules]]`, and other tables appear in the file is irrelevant. Order matters only within a list: the first matching entry of `overrides`, of `rules`, or of `languages.mappings` decides, and built-in conventions apply in their documented order. Because rules precede build modules, a rule also takes source a build module would claim; it should match only the folders it is about. Within the module stage, nesting and same-folder overlaps resolve as [Module Detection](#module-detection) defines.

Measurement is deterministic: the same selected files, configuration layers, and Locmetry version give the same report, apart from `metadata.root` and `metadata.measured_at_unix_seconds`. No result depends on the order in which the file system lists files or build files are found, and every list in a report has a defined order.

| Setting | Type and contract |
| --- | --- |
| `version` | Required integer; initially `1`. Unsupported versions fail. It selects the sets of built-in conventions and language mappings: a release may add entries to a set and new build-module detectors, which can reclassify files, but changing or removing an entry requires a new version. Every report records what it applied as `metadata.config.builtin_conventions`, `metadata.config.builtin_language_mappings`, and `metadata.modules`, so measurements by different releases can be compared. |
| `backend` | String; initially only `tokei`, also the default. |
| `scope.hidden` | Boolean, default `true`; hidden-file selection is independent of metadata exclusions. |
| `scope.respect_ignore_files` | Boolean, default `true`; controls the adapter's ignore-file policy. |
| `scope.exclude` | Array of root-relative glob strings, default empty, added to the default exclusions. A folder is excluded as `<folder>/**`; patterns that exclude nothing are reported as `summary.unused_exclusions`. |
| `scope.default_excludes` | Boolean, default `true`; applies `**/.git/**`, `**/node_modules/**`, and the SwiftPM and Lake build folders `**/.build/**` and `**/.lake/**`. `false` is the only way to drop them. `build` directories are not excluded by default because they can hold maintained source; ignore files normally exclude build output. |
| `classification.builtin_tests` | Boolean, default `true`; enables documented test conventions, selected by `version`. Conventions cover common test directories, test filename suffixes, and framework source sets such as Gradle/Kotlin `src/*Test/**` and `testFixtures`. Ambiguous names such as `fixtures` and `spec` are left to project entries. The version-1 set is defined by `BUILTIN_TESTS_V1` in [src/config.rs](src/config.rs) and recorded in every report as `metadata.config.builtin_conventions`. |
| `classification.builtin_build` | Boolean, default `true`; enables documented build conventions, selected by `version`. Build scripts such as Gradle, Maven, CMake, Make, and Lake files are `production` with category `build-tooling`, so they stay separate from implementation; build-tool wrappers such as `gradlew` and `mvnw` are `unclassified` with category `third-party-wrapper`. The set is defined by `BUILTIN_BUILD_V1` in [src/config.rs](src/config.rs) and recorded with the test conventions. |
| `classification.builtin_modules` | Boolean, default `true`; enables module detection (see [Module Detection](#module-detection)). It runs after `rules`, so it changes only files that would otherwise reach the fallback. |
| `classification.fallback` | Purpose enum, default `unclassified`; projects can explicitly choose another purpose. |
| `languages.builtin` | Boolean, default `true`; enables documented language mappings for common file types the backend does not recognize, such as `.gradle` scripts as Groovy and `.jsonc` as JSON. `version` selects the set, defined by `BUILTIN_LANGUAGES_V1` in [src/config.rs](src/config.rs) and recorded in every report as `metadata.config.builtin_language_mappings`. |
| `languages.mappings` | Ordered array of `{id, match, language}` entries, applied before built-in mappings. `language` must be a backend language name. Mappings apply only to selected files the backend does not recognize; backend detection is never overridden. The first matching mapping counts the file with the backend's parser for that language. |
| `categories` | Optional table mapping category names to their meanings. When present, every entry `category` must be declared there or be built in: a purpose name, `implementation`, or a built-in convention's category (`test`, `build-tooling`, `third-party-wrapper`). It catches typos and invented names; without it, categories are free-form. The effective configuration records the meanings. |
| `overrides`, `rules` | Ordered arrays of entries; omitted arrays are empty. Each entry requires a unique string `id` (not `fallback`, which names the fallback provenance), a nonempty string-array `match`, and a valid `purpose`. `category` defaults to the purpose. `kind` is an optional nonempty array of kinds. `reason` is an optional string recorded in the effective-policy snapshot. |

Match normalized root-relative paths with case-sensitive globs: `*` stays within a segment, `**` spans zero or more directories, `?` matches one non-separator character, `[...]` matches one character from a class, and `{a,b}` matches any listed alternative. Backslash escapes the next character on every platform. Any pattern in `match` selects its entry. Patterns do not use implicit basename matching, negation, or absolute paths. Reject invalid patterns, unknown keys, duplicate IDs, and invalid types/enums.

Explicit exclusions still apply when ignore files are disabled. Removing a built-in exclusion does not override a repository ignore rule; use the project's ignore exceptions or an explicit ignore-file policy. Backend pattern syntax must not silently alter these semantics.

## Workspace Mode

A workspace measures several repositories as one virtual monorepo: each member is measured exactly as a regular run would measure it, with its own `.locmetryrc`, and the results merge into one report whose top-level folders are the members.

- **Selection:** `locmetry workspace --root DIR [--root DIR ...]` names the members, relative to the working directory or absolute; `locmetry workspace --file FILE` reads them instead from a TOML workspace file with `version = 1` and `roots`, an array of paths relative to the file's folder or absolute. `--file` and `--root` exclude each other, and a workspace file is read only when named, never discovered. `locmetry --schema workspace` describes the file. Plain `locmetry` is unaffected, and options given before `workspace` are rejected, so the mode is always explicit.
- **Options:** `--config`, `--exclude`, `--hidden`, and `--respect-ignore-files` are rejected, because each member's own configuration decides them. `--no-user-config` applies to every member; `--format`, `--output`, `--files`, `--categories`, `--sort`, and `--depth` apply to the merged report. `--output` must lie outside every member and must not overwrite the workspace file.
- **Members:** each member is named after its root's folder name, and every report path is prefixed with it, as in `web/src/a.ts`. At least one root is required; two members with the same name, a root named twice, or one root inside another fail before measuring. A member that contains the workspace file does not measure it, as with a configuration file.
- **Determinism and failure:** members are measured in parallel and merged in name order, so neither the order of roots nor thread scheduling changes the report. If any member fails, for example with an invalid configuration, the run fails and names that member; there is no partial merge.
- **Report:** the merged report keeps the regular shape: totals, `top_folders` (one per member), categories, `unclassified_folders`, `submodules`, `build_modules`, `review`, `breakdown`, `files`, and `metadata.coverage` and `metadata.modules` cover the prefixed records, and `project` joins the member names with ` + `. `metadata.members` lists each member's `name`, `root`, `config_path`, `scope`, `unused_entries`, `unused_exclusions`, and `coverage`, plus its effective `config` in the report file; `metadata.workspace_path` names the workspace file when one was used. These settings and findings are relative to one member, so in workspace mode they appear there instead of in `metadata.root`, `metadata.scope`, `metadata.config_path`, `metadata.config`, and `summary.unused_entries` and `summary.unused_exclusions`. Human notes name the member, as in `ghost (web)`.

Excluded: discovering a workspace file, explicit member names, overlapping members, and configuration shared or inherited across members.

### Per-Repository Breakdown

A workspace adds the member as a dimension of the merged report, so repositories can be compared beyond code per purpose.

- `summary.members`, keyed by member name, holds each member's totals in the shape of the summary's own: `all_counted`, `source`, `by_kind_purpose`, `source_purpose`, `source_categories`, and `source_languages`. They equal what a regular run of that member reports.
- `metadata.members` also holds each member's `coverage` counts: `selected_files`, `counted_files`, `mapped_files`, `unrecognized_files`, and `unread_files`.
- The terminal table shows repository rows in place of folder rows: code, production, test, unclassified, test lines per production line, and the share of the workspace's source code, with a total row.
- The Markdown report adds a Repositories section with the same table, followed by source categories and source languages by repository, as grids with one column per member and a total column; rows follow `--sort`.
- A note in both human formats names each member with unclaimed source or unreadable files, with their counts; members without any are not listed. Unrecognized files stay in the coverage counts and the "Not counted" note, because they are mostly images and similar non-source files that need no action.

Excluded: detecting copies across members and comparing measurements over time.

## Reporting and Coverage

Reports serve aggregation: totals by purpose, category, kind, language, and folder. Count original file reports and embedded statistics once. `breakdown` rows are additive, and so are top-level folder totals, which cover each subtree and, with the root files, every file once. The Markdown folder table lists nested levels, in which a parent includes its subfolders, so its rows are not all additive.

Stdout defaults to `--format table` when it is a terminal and to `--format json` otherwise, so people and agents each get their format without a flag. `table` shows source code per language and per top-level folder, split by purpose, followed by notes on non-source content, the build systems whose modules classified code, Git submodules with measured source, the folders that hold unclaimed source, configuration entries that decided no file, and uncounted files; the Markdown report repeats the submodule and entry notes under Coverage. `json` prints a compact headline: the report without `review`, `breakdown`, `files`, `metadata.config`, `metadata.kinds`, and `metadata.coverage.unrecognized`. `markdown` prints a full human report for saving or reviews; `--depth N` (default 2) limits its folder table. `--sort code` (default) orders the rows of both human formats by code, largest first with ties alphabetical, and `--sort name` alphabetically; the Markdown folder table keeps its path hierarchy, and the review list stays largest first. `--output FILE` writes the full report to one file outside the scan root, and the headline names it as `report`; `--output -` prints the full report to stdout instead of the headline and cannot be combined with `--format table` or `markdown`. `--files` adds per-file records to the report for rule debugging. Diagnostics go to stderr. Exit code 0 means a complete report, 2 a partial report whose missing counts come from unreadable files, and 1 a failure without a report, including invalid command-line usage.

JSON schema version `1`, which `locmetry --schema report` prints as a JSON Schema covering the headline and the full report. That schema is written by hand, and tests validate real reports against it with unknown keys rejected:

| Field | Contents |
| --- | --- |
| `schema_version`, `project` | `"1"` and the scan root's directory name. |
| `report` | Headline only, with `--output FILE`: the report file's path. |
| `metadata` | `root`, `measured_at_unix_seconds`, `locmetry_version`, `backend`, `backend_inaccurate`, `backend_configuration`; `scope` (effective hidden, ignore-file, symlink, and `exclude` settings); `kinds` (language lists per content kind); `coverage` (`selected_files`, `counted_files`, `mapped_files`, `unrecognized_files`, `unrecognized_extensions`, the `unrecognized` paths, and the `unread` paths); `modules` (report file only: detected modules with `path` and `detector`); `config_path` and `user_config_path`; and `config`, the effective configuration after merging user defaults including `builtin_conventions` and `builtin_language_mappings`. In [workspace mode](#workspace-mode), `members` and, for a workspace file, `workspace_path` replace `root`, `scope`, `config_path`, and `config`. |
| `summary` | Totals: `all_counted` (every kind) and `source`; `by_kind_purpose` keyed `kind:purpose`; `source_purpose`, `source_categories` keyed `purpose:category`, and `source_languages`, all restricted to `kind = source`; `top_folders`, source code lines per purpose for each top-level folder (`.` holds files directly in the root); `submodules`, present when Git submodules declared in the scan root's `.gitmodules` hold measured source, an array of `{path, files, code}` that disappears once they are excluded (locmetry reads the file and never invokes Git); `unused_entries`, present when configuration entries decided no file, because their globs match nothing or an earlier stage always decides first; `unused_exclusions`, present when exclusions from the project, user defaults, or `--exclude` left out no file or folder, such as a folder written without `/**` or a path ignore files already hide (default exclusions and the configuration's own path are never listed); `build_modules`, present when build modules were adopted, keyed by detector with `modules` and the source `code` they classified; and `unclassified_folders`, an array of `{folder, files, code}` that locates all unclaimed source (unclassified source that reached the fallback) in non-overlapping folders, always present and empty when every file is decided, largest first: chains of single subfolders collapse to the deepest folder, and the largest folder splits into its subfolders while the list stays within eight rows, or always while it holds most of the code. |
| `review` | Unclaimed source files, largest first: `path`, `language`, `category`, `rule`, `code`, `lines`. These are the files a configuration may still need to classify; code an entry or convention leaves `unclassified` on purpose, such as a parked prototype or a build wrapper, is counted as unclassified but not listed. |
| `breakdown` | One row per folder, kind, purpose, category, and language with `files`, `code`, `comments`, `blanks`, and `lines`. `folder` is each file's parent (`.` for the root); a folder's recursive totals are the sum over rows whose folder equals it or starts with it plus `/`. Embedded-language rows count lines but zero `files`, so every column is additive. |
| `files` | Only with `--files`: one record per counted file, sorted by path: `path`, `language`, `kind`, optional `language_mapping`, `purpose`, `category`, `rule`, `counts` (without `files`), and, only for files with embedded languages, per-language `languages` counts. |

"Unclassified" is a purpose of any kind: unclassified documentation or data is not unclassified source, which `source_purpose` and `review` report. Summary count objects have `files`, `code`, `comments`, `blanks`, and `lines`; summaries omit zero counts and empty maps, except `all_counted`, so an absent entry means zero. Report files put each top-level field and each table row on its own line, so line tools work on rows while the file stays one JSON document.

Surface known counting-coverage limitations: report how many files were selected and counted, list selected files the backend did not recognize, and separately list recognized files it could not read. Path rules cannot recover source files omitted by the backend. Language mappings recover unrecognized files; absent records must not be represented as proven zero LOC. Inline tests in mixed files remain a whole-file classification limitation.

## Sensitive Analysis

Public documentation and fixtures are synthetic; [CLAUDE.md](CLAUDE.md#sensitive-analysis) governs this repository's real measurements. That is a development-data policy: users of the released CLI may write reports anywhere outside the scan root.
