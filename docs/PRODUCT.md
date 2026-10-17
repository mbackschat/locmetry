# Product Vision & Requirements

## Vision

Measure a project's own codebase with understandable scope, test/production distinctions, and useful folder totals. Locmetry is a policy-and-reporting layer over an established counter. Agents consume predictable JSON; developers get readable summaries and reviewable project configuration.

The product rests on three pillars:

- **Expressive configuration:** reviewable project policy covers real repositories' scope, ownership, and classification needs, including build tooling, examples, generated code, and evidence.
- **Reuse with gap filling:** the embedded counter does the counting; Locmetry layers on top of its library and fills gaps it leaves, such as unrecognized file types, instead of reimplementing it.
- **Rich output:** reports summarize the measurement, while full JSON keeps provenance and the effective policy, with per-file records when `--files` is requested so agents and scripts can answer further questions, including their own summations.

## Scope and Status

- Required: embedded Tokei integration in a Rust executable, test/production classification, production source derived from build descriptors, project configuration, personal defaults, and CLI options, a configuration schema that explains its semantics, configurable exclusions, language mappings for files the backend does not recognize, folder breakdowns and sums, a workspace mode that merges several repositories, JSON, human output that locates unclassified source, net change between saved full reports, and focused explanations of a file's selection and classification.
- Optional: further module detectors, additional counting adapters, custom content-kind overrides, and configuration self-checks that assert expected classifications for named paths without requiring those files to exist.
- Excluded initially: a new counting engine, a generic plugin framework (module detectors are compiled-in implementations of one trait, not loadable plugins), and semantic splitting of production/tests within individual files.

Locmetry is a Rust executable with the Tokei library embedded. The installed executable must not require Python, Git, a separate Tokei executable, or a separately installed Rust runtime; the Homebrew formula installs the prebuilt release executable without runtime formula dependencies, needing Rust only to build `--HEAD`. [Tokei integration and distribution](tokei-integration.md) records the backend boundary and packaging.

## Measurement Model

| Concept | Meaning |
| --- | --- |
| Scope | Whether a file belongs in the measurement; external/reference material is an inclusion decision, not a test category. |
| Kind | `source`, `data`, `documentation`, or `other`, derived from the backend language through a reviewed mapping; a JSON test fixture remains data. |
| Purpose | `production`, `test`, or `unclassified`. Production means maintained non-test source, not necessarily shipped runtime code. |
| Category | Optional project-defined label such as implementation, tooling, test-support, example, benchmark, or evidence. It does not change counting. |

Report code, comments, blanks, and total lines separately. Preserve each file's language and classification provenance; a language supplied by a mapping records the mapping ID. Embedded snippets inherit the host file's scope and purpose; documentation snippets do not inflate source totals. Documentation embedded in a non-documentation file, such as Rust `///` doc comments that the backend parses as Markdown, contributes its lines, including doctest code, as comments of the host language. Derive human summaries and versioned JSON from the same records.

## Project Configuration

Use `.locmetryrc` in TOML. Discover it in the selected scan root (`locmetry DIR` or `locmetry --root DIR`), which defaults to the current directory; an explicit `--config` path may select a different file without changing that root. Without a discovered or explicit configuration, built-in settings apply; an explicit configuration that cannot be read fails. A loaded configuration inside the scan root is policy, not project content: it is excluded as a literal path, which the effective exclusions record. All patterns remain relative to the scan root. How it combines with built-in settings, user defaults, and command-line flags is defined in [Matching and Precedence](#matching-and-precedence). A configuration file ending in `.json`, the retired JSON/regex profile format, is rejected with a hint to translate it to TOML.

Single-root measurement accepts one optional directory argument, resolved from the working directory, and rejects combining it with `--root` or another directory. Bare command names retain their subcommand meaning regardless of directory existence; use a path such as `./compare`, `--root compare`, or `-- compare` to measure a directory with that name. Root measurement arguments cannot be combined with a subcommand; each command's options follow its name. Positional directory selection uses the same config discovery, scope, output validation and counting as `--root`. Workspace and explanation commands retain their own argument forms. Adding a reserved command name can affect existing bare-directory invocations and requires compatibility review; scripts should use `--root` for arbitrary directory names.

`locmetry --schema config` prints the JSON Schema (Draft 2020-12) of this file, `--schema user-config` that of personal defaults, and `--schema workspace` that of a [workspace](#workspace-mode) file, then exits; `--schema report` describes the report (see [Reporting and Coverage](#reporting-and-coverage)), `--schema comparison` describes [net change](#net-change), and `--schema explanation` describes [path explanations](#path-explanations). The schema is generated from the parser's types, so it accepts the same keys, types, and enums; checks it cannot express, such as glob syntax and unique IDs, still fail when the file loads. Its descriptions state each setting's default and meaning, the classification order, and this version's built-in conventions, build-module layouts, and language names, so an agent can write a configuration from the binary alone.

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

### Subfolder Measurement

`locmetry ROOT --subfolder DIR` (equivalently `--root ROOT`) measures one literal root-relative folder and its descendants while retaining the project root's policy, selected module context and path identities. It is a single-root CLI option, not a configuration setting, glob, file selector or workspace option. Ancestor configuration discovery remains unsupported. Other command options and explicit configuration keep their existing bases; `--config` never changes the project root.

The folder must exist, have exact on-disk spelling, and stay beneath the project root without symlinks. Reject empty/absolute paths, `..` components and files. Normalize `.` components and trailing separators; `--subfolder .` is equivalent to no restriction. A target directory or ancestor filtered by exclusions, ignore or hidden policy fails with no report. Diagnostics name recorded exclusions or a cause established by a single-setting probe, with a remedy; overlapping filters may leave the cause unknown. Overrides such as `--hidden true` or `--respect-ignore-files false` change policy for the whole project, including module context. A selected empty folder, including one whose children are all filtered, remains a valid empty measurement.

Count and classify each measured file exactly as an unrestricted run under the same policy would. Discover module context from the whole selected project before restricting counting; excluded descriptors do not become evidence. Only measured files contribute counts, coverage and backend inaccuracy. Context traversal failures still abort. A `.locmetryrc` directly inside the selected subfolder stays inactive unless explicitly loaded; one stderr warning names it and the effective project configuration. This warning does not change status or policy.

Record the normalized directory as optional `metadata.scope.subfolder` in headline and full JSON. Absence means no restriction; null, empty strings, `.`, dot/parent components, repeated/trailing separators and absolute paths are invalid saved values. Paths remain project-relative. `metadata.modules` retains all selected project context; in a restricted report, `summary.build_modules` counts only modules that classified at least one measured source file through a module rule (even a zero-code file), and their measured code. Other summary totals, breakdown, review and coverage describe the measured selection. Unused-entry/exclusion findings concern this subtree only; configuration-cleanup instructions apply only to an unrestricted run.

JSON folder identities and top-level grouping stay project-relative. Folder table rows requested with `--folders` partition source into immediate children of the selected subfolder and a direct-files row, with full project-relative labels. Markdown depth starts below the selected subfolder: depth 1 includes immediate children, depth 2 their children. Parent rows remain recursive, so Markdown rows are not additive. Human titles expose the restriction. Output destinations must remain outside the entire project root.

Comparison requires identical recorded subfolder values, including presence versus absence, and otherwise fails without producing or overwriting output. Matching restrictions still retain ordinary policy and coverage findings. Comparison module/package topology describes recorded project context; totals come only from measured contributions. Existing `explain PATH --root ROOT` provides the same per-file decision using project-relative paths, with no subfolder option. [Measurement semantics](measurement.md#subfolder-context-and-coverage) covers traversal limits; [report compatibility](report-compatibility.md) covers consumer obligations.

### User Defaults

Personal defaults live in `$XDG_CONFIG_HOME/locmetry/config.toml`, or in `~/.config/locmetry/config.toml` when `XDG_CONFIG_HOME` is unset, on every platform. They may only narrow scope and teach languages, never assign purpose or category. Personal mappings can nevertheless change recognition and content kind, so reproducibility requires the same effective personal settings or `--no-user-config`:

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

A selected build descriptor triggers conventional source-layout inference: its directory becomes a build module, and the detector's default source layout inside that module is production. This is whole-file classification, not semantic membership in build targets; custom manifest targets and scripts require explicit policy. Build systems call this unit a module, project, package, or crate; it is unrelated to language-level modules or packages, and this document's "module" always means a build module. Detectors inspect selected file names, without parsing manifests or running builds. Each build system is one detector, so supporting another one adds a detector without changing the stages.

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

The innermost module containing a file decides, as build tools treat nested projects: an outer module's layout never claims files of a module nested inside it, while build systems sharing one directory, such as Gradle and npm, combine their layouts. Where those layouts overlap, the detector whose ID sorts first decides, such as `cargo` before `npm`; purpose and category are the same either way, so only provenance and the per-module totals depend on it. Test and build conventions run earlier, so tests inside these trees stay tests. Files outside every module's production source keep the fallback, because no applicable conventional layout claimed them. The report file lists the modules as `metadata.modules`; the summary's `build_modules`, the table, and the Markdown report, which lists every module, show the code each one classified, where `0` hints at a misleading build file.

### Matching and Precedence

Four layers combine into the effective configuration. Only the project layer sets purpose/category policy; personal mappings and scope options can still affect which files and kinds enter that policy:

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
5. Apply module detection, if enabled: source in a conventional production layout inferred from a selected build descriptor is `production` with category `implementation`, and provenance names the detector, for example `builtin:npm-module`.
6. Assign `classification.fallback` if no classification matched.

An entry matches when any of its patterns matches the path and, if `kind` is present, the file's kind is listed. A match terminates purpose/category classification. Each file records its provenance as the matching entry ID, `builtin:<convention>`, or `fallback`; fallback files take the fallback purpose as their category. Broad production rules therefore do not swallow conventional tests or build scripts, while an explicit archived-evidence override can take precedence over test filenames. Entries that assign ownership of a whole tree, such as a test-only package whose build scripts are test support, belong in `overrides` so built-in conventions do not preempt them. No ordinary run requires an LLM; agent assistance proposes reviewable configuration.

The built-in `build-wrapper` convention matches `gradlew`, `gradlew.bat`, `mvnw`, `mvnw.cmd`, and files under `gradle/wrapper/` or `.mvn/wrapper/` at any depth. It assigns purpose `unclassified` and category `third-party-wrapper` when enabled and no earlier classification wins. Overrides and built-in test conventions take precedence; for example, `tests/gradlew` matches the test-directory convention under default policy. `classification.builtin_build = false` disables build conventions, including wrappers. These patterns do not change selection or make unrecognized binary files countable. The [built-in definitions](../src/config.rs) and `locmetry --schema config` enumerate the executable rules.

The stage order is fixed, so where `[[overrides]]`, `[[rules]]`, and other tables appear in the file is irrelevant. Order matters only within a list: the first matching entry of `overrides`, of `rules`, or of `languages.mappings` decides, and built-in conventions apply in their documented order. Because rules precede build modules, a rule also takes source a build module would claim; it should match only the folders it is about. Within the module stage, nesting and same-folder overlaps resolve as [Module Detection](#module-detection) defines.

Measurement counts and ordering are deterministic for identical selected file contents, effective configuration, backend settings and Locmetry version. Selection also depends on repository, parent and global ignore context. Timestamps, project/member names and contextual paths can differ across runs or locations; reports do not fingerprint source, ignore contents or executable builds. No counting result depends on traversal order, and report lists have defined ordering.

| Setting | Type and contract |
| --- | --- |
| `version` | Required integer; initially `1`. Unsupported versions fail. It selects the sets of built-in conventions and language mappings: a release may add entries to a set and new build-module detectors, which can reclassify files, but changing or removing an entry requires a new version. Every report records what it applied as `metadata.config.builtin_conventions`, `metadata.config.builtin_language_mappings`, and `metadata.modules`, so measurements by different releases can be compared. |
| `backend` | String; initially only `tokei`, also the default. |
| `scope.hidden` | Boolean, default `true`; hidden-file selection is independent of metadata exclusions. |
| `scope.respect_ignore_files` | Boolean, default `true`; controls the adapter's ignore-file policy. |
| `scope.exclude` | Array of root-relative glob strings, default empty, added to the default exclusions. A folder is excluded as `<folder>/**`; patterns that exclude nothing are reported as `summary.unused_exclusions`. |
| `scope.default_excludes` | Boolean, default `true`; applies `**/.git/**`, `**/node_modules/**`, and the SwiftPM and Lake build folders `**/.build/**` and `**/.lake/**`. `false` is the only way to drop them. `build` directories are not excluded by default because they can hold maintained source; ignore files normally exclude build output. |
| `classification.builtin_tests` | Boolean, default `true`; enables documented test conventions, selected by `version`. Conventions cover common test directories, test filename suffixes, and framework source sets such as Gradle/Kotlin `src/*Test/**` and `testFixtures`. Ambiguous names such as `fixtures` and `spec` are left to project entries. The version-1 set is defined by `BUILTIN_TESTS_V1` in [src/config.rs](../src/config.rs) and recorded in every report as `metadata.config.builtin_conventions`. |
| `classification.builtin_build` | Boolean, default `true`; enables documented build conventions, selected by `version`. Build scripts such as Gradle, Maven, CMake, Make, and Lake files are `production` with category `build-tooling`, so they stay separate from implementation; build-tool wrappers such as `gradlew` and `mvnw` are `unclassified` with category `third-party-wrapper`. The set is defined by `BUILTIN_BUILD_V1` in [src/config.rs](../src/config.rs) and recorded with the test conventions. |
| `classification.builtin_modules` | Boolean, default `true`; enables module detection (see [Module Detection](#module-detection)). It runs after `rules`, so it changes only files that would otherwise reach the fallback. |
| `classification.fallback` | Purpose enum, default `unclassified`; projects can explicitly choose another purpose. |
| `languages.builtin` | Boolean, default `true`; enables documented language mappings for common file types the backend does not recognize, such as `.gradle` scripts as Groovy and `.jsonc` as JSON. `version` selects the set, defined by `BUILTIN_LANGUAGES_V1` in [src/config.rs](../src/config.rs) and recorded in every report as `metadata.config.builtin_language_mappings`. |
| `languages.mappings` | Ordered array of `{id, match, language}` entries, applied before built-in mappings. `language` must be a backend language name. Mappings apply only to selected files the backend does not recognize; backend detection is never overridden. The first matching mapping counts the file with the backend's parser for that language. |
| `categories` | Optional table mapping category names to their meanings. When present, every entry `category` must be declared there or be built in: a purpose name, `implementation`, or a built-in convention's category (`test`, `build-tooling`, `third-party-wrapper`). It catches typos and invented names; without it, categories are free-form. The effective configuration records the meanings. |
| `overrides`, `rules` | Ordered arrays of entries; omitted arrays are empty. Each entry requires a unique string `id` (not `fallback`, which names the fallback provenance), a nonempty string-array `match`, and a valid `purpose`. `category` defaults to the purpose. `kind` is an optional nonempty array of kinds. `reason` is an optional string recorded in the effective-policy snapshot. |

Match normalized root-relative paths with case-sensitive globs: `*` stays within a segment, `**` spans zero or more directories, `?` matches one non-separator character, `[...]` matches one character from a class, and `{a,b}` matches any listed alternative. Backslash escapes the next character on every platform. Any pattern in `match` selects its entry. Patterns do not use implicit basename matching, negation, or absolute paths. Reject invalid patterns, unknown keys, duplicate IDs, and invalid types/enums.

Explicit exclusions still apply when ignore files are disabled. Removing a built-in exclusion does not override a repository ignore rule; use the project's ignore exceptions or an explicit ignore-file policy. Backend pattern syntax must not silently alter these semantics.

## Workspace Mode

A workspace measures several repositories as one virtual monorepo: each member is measured exactly as a regular run would measure it, with its own `.locmetryrc`, and the results merge into one report whose top-level folders are the members.

- **Selection:** `locmetry workspace --root DIR [--root DIR ...]` names the members, relative to the working directory or absolute; `locmetry workspace --file FILE` reads them instead from a TOML workspace file with `version = 1` and `roots`, an array of paths relative to the file's folder or absolute. `--file` and `--root` exclude each other, and a workspace file is read only when named, never discovered. `locmetry --schema workspace` describes the file. Plain `locmetry` is unaffected, and options given before `workspace` are rejected, so the mode is always explicit.
- **Options:** `--config`, `--exclude`, `--hidden`, and `--respect-ignore-files` are rejected, because each member's own configuration decides them. `--no-user-config` applies to every member; `--format`, `--output`, `--files`, `--categories`, `--sort`, and `--depth` apply to the merged report using [Output Routing](#output-routing). `--output` must lie outside every member and must not overwrite the workspace file.
- **Members:** each member is named after its root's folder name, and every report path is prefixed with it, as in `web/src/a.ts`. At least one root is required; two members with the same name, a root named twice, or one root inside another fail before measuring. A member that contains the workspace file does not measure it, as with a configuration file.
- **Determinism and failure:** members are measured in parallel and merged in name order, so neither the order of roots nor thread scheduling changes the report. If any member fails, for example with an invalid configuration, the run fails and names that member; there is no partial merge.
- **Report:** the merged report keeps the regular shape: totals, `top_folders` (one per member), categories, `unclassified_folders`, `submodules`, `build_modules`, `review`, `breakdown`, `files`, and `metadata.coverage` and `metadata.modules` cover the prefixed records, and `project` joins the member names with ` + `. `metadata.members` lists each member's `name`, `root`, `config_path`, `scope`, `unused_entries`, `unused_exclusions`, and `coverage`, plus its effective `config` in full JSON; `metadata.workspace_path` names the workspace file when one was used. These settings and findings are relative to one member, so in workspace mode they appear there instead of in `metadata.root`, `metadata.scope`, `metadata.config_path`, `metadata.config`, and `summary.unused_entries` and `summary.unused_exclusions`. Human notes name the member, as in `ghost (web)`.

Excluded: discovering a workspace file, explicit member names, overlapping members, and configuration shared or inherited across members.

### Per-Repository Breakdown

A workspace adds the member as a dimension of the merged report, so repositories can be compared beyond code per purpose.

- `summary.members`, keyed by member name, holds each member's totals in the shape of the summary's own: `all_counted`, `source`, `by_kind_purpose`, `source_purpose`, `source_categories`, and `source_languages`. They equal what a regular run of that member reports.
- `metadata.members` also holds each member's `coverage` counts: `selected_files`, `counted_files`, `mapped_files`, `unrecognized_files`, `unread_files`, and `parse_failed_files` (always emitted now; absent in older reports).
- The terminal table shows repository rows in place of folder rows: total source lines, code, production, test, unclassified, test lines per production line, and the share of the workspace's source code, with a total row.
- The Markdown report adds a Repositories section with the same table, followed by source categories and source languages by repository, as grids with Lines and Code columns per member and corresponding total columns; rows follow `--sort`.
- Default and verbose tables name each member with read/parse failures under `Unavailable counts:`, listing only its failure counts. `--verbose` separately lists unclaimed source by repository. Markdown retains its comprehensive combined member note. Unrecognized types appear only in the verbose table’s neutral `Unrecognized files:` note and in Markdown/JSON coverage; they are not counting failures.

Excluded: detecting copies across members. Saved measurements can be compared through [net change](#net-change).

## Reporting and Coverage

Reports serve aggregation: totals by purpose, category, kind, language, and folder. Count original file reports and embedded statistics once. `breakdown` rows are additive, and so are top-level folder totals, which cover each subtree and, with the root files, every file once. The Markdown folder table lists nested levels, in which a parent includes its subfolders, so its rows are not all additive; it also shows each folder's `Test/prod` ratio.

### Embedded Task Guides

Available since v0.4.0: `--guide configure|selection|reports|compare` prints one task-oriented Markdown guide with a source-path and producer-version header. The rest of stdout equals the repository Markdown source; successful output has no stderr, styling, paging, project/configuration reads or network dependency. Relative links remain unchanged and supplementary; essential instructions are self-contained, and cross-guide navigation names executable commands. The four [guide sources](guides/configure.md) cover configuration, selection, report interpretation and comparison.

Root long/short/help-command output leads with source counts by language, names `--folders` for folder rows and `--verbose` for classification/scope/coverage notes, and presents `locmetry --guide configure` as optional configuration. Long help describes all topics; explain, compare and workspace help point to the appropriate root-level guide queries. Topics are required, case-sensitive enum values. Missing/unknown topics fail with exit 1, accepted names and a help remedy. `--guide` follows the same standalone-query rule as `--schema`, and the two conflict. Save guide text by shell redirection.

Configuration guidance distinguishes evidence-based classification, intentional unclassified entries with reasons, and unresolved unclaimed questions reported to the owner. Correct inferred decisions need no redundant entry. A clean review list does not prove correct policy, and open questions are distinct from invalid configuration or partial counts. Guides own workflows; executable schemas and this contract remain the semantic authorities.

### Output Routing

Output contract since v0.4.0. Measurement, workspace and comparison produce one result at one destination. An explicit `--format table|markdown|json|headline` always wins. Without it, any `--output` selects full JSON; otherwise stdout selects a table on a terminal and a compact JSON headline in pipes. File extensions never infer the format.

| Destination | Behavior |
| --- | --- |
| No `--output` | Write the chosen result to stdout. |
| `--output FILE` | Write only to FILE; stdout stays empty. Full JSON is the default, including when run interactively. |
| `--output -` | Write the chosen result to stdout; full JSON is the default. Explicit table, Markdown and headline formats are accepted. |

`json` is the complete document. `headline` is compact JSON: measurement omits `review`, `breakdown`, `files`, `metadata.config`, `metadata.kinds`, `metadata.modules`, `metadata.coverage.unrecognized` and workspace member configurations. Comparison uses its [headline projection](#net-change). New output does not emit the historical `report` pointer; historical documents retain it when read and reserialized. `--files` adds per-file measurement records and requires the resolved format to be `json`; incompatible formats fail before scanning or writing. Full JSON without file records is sufficient for aggregate comparison.

File output is always unstyled. Stdout styling follows [Human Presentation](#human-presentation). Measurement file destinations must be outside the scan root and must not overwrite the configuration; workspace destinations must be outside every member and must not overwrite the workspace file. Comparison protects both input files and their aliases. These protections apply to every exported format. Shell redirection and `tee` are outside these checks; they may truncate inputs before Locmetry runs and need pipeline status handling to preserve partial-result failures.

Diagnostics go to stderr. File exports do not echo the result or its informational findings to stderr; informational findings follow the selected format: saved tables require `--verbose` for routine notes, while Markdown and JSON retain their established evidence. Partial results are written, produce a stderr warning and retain exit 2; output failures exit 1. [Command outcomes](#command-outcomes) owns the status rules. Running twice to display and save separately is not a guarantee of identical filesystem state; rendering saved measurements is outside this interface.

`--categories` and `--verbose` take effect only for measurement table output. Single-root `--folders` also takes effect only for table output. `--depth N` (default 2) limits Markdown folder depth. Explicitly supplying any of these options where it cannot take effect produces a stderr warning without changing the result or exit status; defaulted depth stays silent. `--sort code` (default) orders human rows by descending code and `--sort name` alphabetically; the Markdown folder table retains its hierarchy and review rows stay largest first.

`--schema` is a standalone query: it emits its JSON schema on stdout without loading configuration or measuring. It rejects explicit roots, measurement/presentation options and subcommands with exit 1 before scanning or writing. Explicit default-valued flags such as `--depth 2` also conflict; implicit defaults do not. Help/version retain their normal early exits. Redirect stdout to save a schema. `explain` retains `table|markdown|json`, no `headline` and no `--output`; its pipe default is the complete JSON explanation.

### Report Content

`table` shows source Lines and Code per language by default, with language columns ordered Files, Lines, Code, Comments, Blanks before the code split by purpose. Optional folder rows and default workspace repository rows include Lines before Code, followed by purpose code, `Test/prod` ratios and reconciliation legends; optional category rows also include Lines before Code. Successful default tables contain no routine findings: non-source totals, build-module inference, Git submodule hints, unclaimed folders, unused configuration and unrecognized-file counts require `--verbose`. No reminder is added to ordinary output. The default table deliberately gives no indication of data/documentation totals or unrecognized files; its totals cover counted source. This presentation choice changes neither selection nor counts. `markdown` provides the full human report with recursive folder rows and coverage details. Both group thousands with commas; JSON keeps integers. These are views of the same measurement, not separate scans. Markdown count tables likewise place Lines before Code, including folder, repository, build-module and review tables. Lines always means Code + Comments + Blanks; purpose columns and ratios continue to describe Code. JSON count objects already retain `lines`; explicitly code-only projections such as `top_folders` and build-system classification summaries keep their existing semantics.

`--folders` is a boolean single-root presentation option that adds the folder table. It applies equally to stdout and saved tables, including `--subfolder`. `--folder-rows N` implicitly enables the folder table with adaptive detail; both flags may be supplied together. `--verbose` selects notes independently and does not enable the folder table. Workspace retains repository rows by default and rejects `--folders`; comparison, explanation and standalone queries also reject it. Markdown and JSON retain their folder evidence regardless of this option. Measurement dates and real-failure diagnostics remain visible with or without folder rows.

`--verbose` is a boolean root/workspace presentation option, applying equally to stdout and saved tables and to subfolder/adaptive views. Verbose tables include neutral optional classification advice: “To inspect or refine purpose classification, use locmetry --guide configure; --files --output - shows each file's rule.” Unclaimed source remains counted under Unclassified in either view. Unrecognized files and their leading extension groups appear in a separate muted `Unrecognized files:` note, not a warning-styled failure summary.

In default and verbose tables, `Not counted:` contains only unreadable and parse-failed counts; the backend-incomplete warning, existing stderr diagnostic and partial-result exit 2 remain even without identified failure paths. Workspace tables also name affected members under `Unavailable counts:`; their unclaimed counts require the separate verbose note. The report’s per-path parse-failure list, capped at 25 paths, appears only with `--verbose`; the existing stderr diagnostic still names up to 10 unavailable paths. Markdown, JSON/headline evidence and explicit comparison/explanation findings retain their detail. Invalid configuration/output errors and ineffective-option warnings remain visible.

Table and Markdown measurement reports end with `Measured at: YYYY-MM-DD HH:MM:SS UTC`, using the measurement-start value in `metadata.measured_at_unix_seconds`, including workspace and partial reports. JSON and headline retain that existing timestamp; no filesystem creation time or duplicate date field is used. Comparison table and Markdown reports end with Before/After measurement dates from the inputs, not the comparison-render time. Dates outside the formatter’s supported calendar range retain their numeric Unix seconds with an explicit range notice.

JSON schema version `1`, which `locmetry --schema report` prints as a JSON Schema covering the headline and the full report. The schema is generated from the typed contract used to serialize and deserialize reports, sharing configuration types for effective policy. Tests validate current root/workspace output, historical release fixtures and malformed inputs. [Saved report compatibility](report-compatibility.md) documents field history, internal semantic validation and the stricter evidence required for comparison:

| Field | Contents |
| --- | --- |
| `schema_version`, `project` | `"1"` and the scan root's directory name. |
| `report` | Historical headline pointer through v0.3.0; retained for reading and round-tripping, not emitted by new CLI output. |
| `metadata` | `root`, `measured_at_unix_seconds`, `locmetry_version`, `backend`, `backend_inaccurate`, `backend_configuration`; `scope` (effective hidden, ignore-file, symlink, and `exclude` settings, plus optional [subfolder selection](#subfolder-measurement)); `kinds` (language lists per content kind); `coverage` (`selected_files`, `counted_files`, `mapped_files`, `unrecognized_files`, `unrecognized_extensions`, the `unrecognized` paths, the `unread` paths, and `parse_failures`, an always-emitted array of `{path, reason, message}` for readable files without counts); `modules` (full JSON only: detected modules with `path` and `detector`); `config_path` and `user_config_path`; and `config`, the effective configuration after merging user defaults including `builtin_conventions` and `builtin_language_mappings`. In [workspace mode](#workspace-mode), `members` and, for a workspace file, `workspace_path` replace `root`, `scope`, `config_path`, and `config`. |
| `summary` | Totals: `all_counted` (every kind) and `source`; `by_kind_purpose` keyed `kind:purpose`; `source_purpose`, `source_categories` keyed `purpose:category`, and `source_languages`, all restricted to `kind = source`; `top_folders`, source code lines per purpose for each top-level folder (`.` holds files directly in the root); `submodules`, present when Git submodules declared in the scan root's `.gitmodules` hold measured source, an array of `{path, files, code}` that disappears once they are excluded (see [submodule hint semantics](measurement.md#git-submodule-hints); Git is never invoked); `unused_entries`, present when configuration entries decided no file, because their globs match nothing or an earlier stage always decides first; `unused_exclusions`, present when exclusions from the project, user defaults, or `--exclude` left out no file or folder, such as a folder written without `/**` or a path ignore files already hide (default exclusions and the configuration's own path are never listed); `build_modules`, keyed by detector with `modules` and the source `code` they classified (adopted modules normally; only contributing modules with [subfolder selection](#subfolder-measurement)); and `unclassified_folders`, an array of `{folder, files, code}` that locates all unclaimed source (unclassified source that reached the fallback) in non-overlapping folders, always present and empty when every file is decided, largest first: chains of single subfolders collapse to the deepest folder, and the largest folder splits into its subfolders while the list stays within eight rows, or always while it holds most of the code. |
| `review` | Unclaimed source files, largest first: `path`, `language`, `category`, `rule`, `code`, `lines`. These are the files a configuration may still need to classify; code an entry or convention leaves `unclassified` on purpose, such as a parked prototype or a build wrapper, is counted as unclassified but not listed. |
| `breakdown` | One row per folder, kind, purpose, category, and language with `files`, `code`, `comments`, `blanks`, and `lines`. `folder` is each file's parent (`.` for the root); a folder's recursive totals are the sum over rows whose folder equals it or starts with it plus `/`. Embedded-language rows count lines but zero `files`, so every column is additive. |
| `files` | Only with `--files`: one record per counted file, sorted by path: `path`, `language`, `kind`, optional `language_mapping`, `purpose`, `category`, `rule`, `counts` (without `files`), and, only for files with embedded languages, per-language `languages` counts. |

"Unclassified" is a purpose of any kind: unclassified documentation or data is not unclassified source, which `source_purpose` and `review` report. Count objects have `files`, `code`, `comments`, `blanks`, and `lines`, including zero-valued counters. Summaries may omit an empty `source` object or empty maps, except `all_counted`; absence means zero only for the aggregate fields whose schema explicitly permits this omission. Report files put each top-level field and each table row on its own line, so line tools work on rows while the file stays one JSON document.

Surface known counting-coverage limitations: report how many files were selected and counted, list selected files the backend did not recognize, and separately list recognized files it could not read. Readable Handlebars files with undecodable content and notebooks with invalid or unsupported counting structure are separately recorded in `parse_failures`; they have no counted file record, set `backend_inaccurate`, and follow the [partial-measurement outcome](#command-outcomes). The array is optional only for historical version-1 schema compatibility: absence means not recorded, not zero failures. [Counting semantics](measurement.md#counting) defines Handlebars decoding failures (`invalid_encoding`); [notebook semantics](measurement.md#notebooks) defines notebook versions, cell treatment, language selection and reason codes. Path rules cannot recover source files omitted by the backend. Language mappings recover unrecognized files; absent records must not be represented as proven zero LOC. Inline tests in mixed files remain a whole-file classification limitation.

### Adaptive Folder Detail

`--folder-rows N` (introduced in v0.3.0) enables the optional Folder table with adaptive detail, implicitly selecting `--folders`; it does not change the Language table. N is a required positive integer; there is no default. It is accepted only for single-root measurement with resolved table format. Non-table formats (including inferred headlines or the full JSON default with `--output`) fail with a `--format table` remedy before scanning or writing. `--output FILE --format table` saves the adaptive table; `--output - --format table` prints it. Workspace, comparison, explanation and schema queries reject the option.

The view starts with existing folder and direct-files rows and keeps every measured source contribution exactly once. Collapse single-child chains with no direct source files on both initial and newly revealed rows. Then repeatedly split the largest positive-code folder whose replacement fits, breaking ties by project-relative path components. A split replaces its parent with immediate source-bearing children and a direct-files row when necessary; parent subtotals are never retained alongside their children. Source-bearing includes counted files with zero code. Zero-code folders remain represented but do not drive expansion.

If all children do not fit, keep the direct-files row, show as many largest children as fit (at least two, code ties by path components), and collect the other children and their descendants into one scoped remainder. A remainder must represent at least two children; a single remaining child is named instead at the same row cost. If this minimum cannot fit, leave the parent intact and consider the next largest eligible row. Direct-files and remainder rows are never expanded. Continue until no further split fits.

N counts data rows, excluding the total, headers and notes. It is a soft budget only for initial rows: retain them even when they exceed N, allowing cost-free chain collapse but no further rows beyond `max(initial rows, N)`. A muted hint names the largest displayed row when it has positive code and is either a folder with source-bearing children or a remainder whose children could be named with more budget. Select that row by code descending, then path components and row kind, independently of `--sort`. A flat folder or direct-files row as the largest row produces no hint, even if smaller folders have depth. The hint suggests increasing `--folder-rows` without proposing N, because another budget can change earlier expansion choices. No code-share or quality threshold drives expansion.

Labels stay project-relative; `X (direct files)` and `X (other K child folders)` are annotations, not folder identities. As with the existing direct-files convention, a real name can resemble an annotation; terminal labels have no uniqueness guarantee, while JSON identities are unaffected. Adaptive labels use the shared terminal-cell sanitization described under [Human Presentation](#human-presentation). `--sort` orders final rows without changing selected contributions: code descending or ascending by path components, ties by path components then row kind (folder, direct files, remainder). Ratios use each row’s summed test and production code. The total, source-purpose reconciliation, coverage notes and exit status are unchanged.

Adaptive rows may change when counts change. The Unclaimed source note requested with `--verbose` and JSON `summary.unclassified_folders` retain their separate partition of unclassified source, including its existing dominance/budget behavior. Different boundaries between these two views are expected; neither changes measured scope. Markdown retains its overlapping parent/child hierarchy, and workspace retains repository rows.

### Human Presentation

Root and subcommand help wrap descriptions to the detected terminal width, preserving indentation for command and option descriptions. When no terminal width is available, help uses 100 columns. Clap owns this layout independently of report-table presentation.

Human output uses semantic presentation roles shared by measurement, workspace, comparison and explanation rendering. Measurement tables start with one compact project title immediately above the column headers, without repeated group captions. The default terminal palette uses bold for column headings, bold cyan for the measurement title, primary Code figures and total rows, cyan for row identities and informational finding labels, dim text for secondary measurement figures, and gray for legends, separators, zero body values and unavailable ratios. Comparison changes use cyan for increases, magenta for decreases and gray for zero deltas; warning labels use yellow and runtime error labels red. Positive and negative changes are not judgments of quality. Purpose categories remain neutral; intentionally unclassified code is not itself a warning. Numeric signs and textual states retain the meaning without color.

Terminal styling is enabled only when the result destination is stdout attached to a terminal and `NO_COLOR` is absent or empty and `TERM` is not `dumb`. Application diagnostics assess stderr independently using the same `NO_COLOR` and `TERM` rules, even when the result goes to a file. Redirected tables and diagnostics receive no styling escapes; JSON is never styled. Markdown uses portable italic legends and bold emphasis instead of ANSI or HTML colors. Shared measurement-table cells (headers, body and totals), table notes and application stderr diagnostics replace control characters with spaces before styling. Unclaimed-source folder labels are sanitized before padding; their intentional list layout is retained. JSON retains the original path identities. Styling is applied after terminal padding, preserving column alignment. Custom theme files, palette flags and configuration fields are not supported; the shared palette is the implementation boundary for future themes.

## Net Change

Inputs must have matching [subfolder restrictions](#subfolder-measurement), including present versus absent.

`locmetry compare BEFORE.json AFTER.json` compares saved full measurements, with **net change = after − before**. Direction follows argument order, not timestamps. The command reads the two files without rescanning roots, loading current policy/ignore files, or invoking Git. It compares two single-root reports or two workspaces; mixed modes fail. [Saved report compatibility](report-compatibility.md) owns version support, structural/semantic validation, and full-input eligibility. Per-file records are optional input detail; per-file comparison is excluded from this initial interface.

Comparison follows [Output Routing](#output-routing): explicit `--format` controls stdout or file content, `json` is complete, and `headline` is compact JSON. `--output FILE` writes only that file. Output must not overwrite either input, including symlink/hard-link aliases. Unlike measurement, there is no scan-root output restriction. Input validity and observed input inaccuracy determine the [command outcome](#command-outcomes); changed settings, unrecognized files and absent historical failure evidence alone do not make a comparison partial.

The comparison has `document_type: "comparison"` and its own `schema_version: "1"`. Typed models generate its self-contained Draft 2020-12 schema with the same mechanism as configuration and measurement; schema examples and actual outputs are tested. `CountChange` holds complete `before`/`after` unsigned counts and signed `delta` counts for files, code, comments, blanks and lines. Unsigned sums must fit `u64`, deltas `i64`; overflow fails rather than wrapping or rounding. Line equations hold on both sides and in the delta. Language file counts remain overlapping participation counts; breakdown file counts remain additive. Validation bounds participation by the available host-file evidence and total source files.

| Field | Meaning |
| --- | --- |
| `before`, `after` | Input filename, project name and measurement metadata; filenames are not verified Git provenance. |
| `summary` | Changes for `all_counted`, `source`, and the union of `by_kind_purpose`, `source_purpose`, `source_categories`, `source_languages`. Workspace `composition` provides each of `all_counted` and `source` as signed `matched`, unsigned `entering`, and unsigned `leaving` counts: total delta = matched + entering − leaving. |
| `members` | Workspace only, keyed by member name, with `presence` and the same count dimensions. |
| `folders` | Full output only, keyed by root-relative paths (member-prefixed in workspaces). Each row has `presence` and `direct`/`recursive` changes for `all_counted` and `source`. Parents include descendants; recursive rows are not additive across levels. |
| `packages` | Full output only: rows identified by explicit `folder` and `detector`, with `presence`. The folder key references its recursive counts, including tests/descendants; package rows do not independently contribute LOC. |
| `breakdown` | Full output only: union of `(folder, kind, purpose, category, language)` rows, each with additive `counts: CountChange`. |
| `presence_summary` | Always-present `members`, `folders`, `packages`, each with `before_only` and `after_only` entity counts, including zeros. These are topology counts, not LOC. |
| `findings` | Stable tagged variants `counter_changed`, `policy_changed`, `modules_changed`, `identity_changed`, `coverage_gap`, `presence_changed`; each includes `side`, human `message`, affected input-field JSON-pointer `paths`, and optional `member`. Presence findings add typed `entity` identity and `presence`; coverage findings can add the direct `folder` with missing counts. |

Identity is evidence-based: single roots are paired by the caller, workspace roots by member name, folders by recorded path, packages by folder/detector. Absolute locations and timestamps are provenance, not matching keys. Names do not prove repository identity. Every entity has `presence: both|before_only|after_only`, referring to its appearance in the reports. Zero-count members and zero-code arrivals/departures stay visible; entirely disjoint nonempty workspaces are supported. Missing members contribute zero to that side's workspace aggregate because they were not included, not because they were measured empty.

Folder presence comes from breakdown rows, coverage paths, adopted modules and their ancestors, with `.` for a single root and the member name for each workspace root. Physical empty directories and directories containing only excluded files leave no evidence. Coverage-only folders retain coverage findings: their zero recorded counts do not estimate their unavailable lines. Removing a three-line `package.json` beside two JavaScript source lines changes all-counted LOC from 5 to 2, source LOC from 2 to 2, production source from 2 to 0 and unclassified source from 0 to 2; package departure is not another subtraction.

The headline omits `folders`, `packages`, `breakdown` and individual presence findings, and uses the measurement headline's compact metadata projection. It retains summary, member changes, other findings and `presence_summary`. Only the topology summary is fixed-size; member rows and coverage findings can grow with evidence.

Full JSON preserves all union rows. Human Purpose and Languages grids select changes to source Lines or Code; Members and Folders also retain recorded arrivals/departures, including zero-code entities. Each grid shows before, after and signed net change for Lines and Code. Comment-only or blank-only growth appears when it changes total lines. Changes to data, file counts or the comment/blank split that leave source Lines and Code unchanged remain in Totals and full JSON. Table Totals columns fit their headers and values, with right-aligned numbers and two spaces between columns. Human output shows signed changes and keeps Markdown folders in hierarchy order. Arrays/maps have deterministic identity order; findings sort by code, member, side and paths, with a deterministic tie-break.

Recorded version/backend settings, content-kind mappings, effective policy, module sets, names and coverage differences produce findings. Policy comparisons ignore location/timestamps, treat exclusion lists as sets, and preserve rule/mapping order. Identical unavailable paths still produce coverage findings in JSON. Historical absent failure evidence means not recorded; it does not certify old notebook counts. These are differences between recorded measured subsets, not Git line additions/deletions, editing effort, rename detection or proof of source-only growth. Reports lack source-content hashes, ignore-file contents and executable fingerprints. No history store or source checkout is required.

Table and Markdown summarize coverage and producer/backend differences under Measurement notes, using typed input metadata; policy, module, identity and presence findings remain under Findings.

Coverage categories appear in order: inaccurate inputs, unreadable files, parse failures, missing failure evidence, then unrecognized files. Unrecognized files are muted notes; inaccurate inputs and read/count failures remain warnings. “Same” refers only to equal recorded path sets, or equal `(path, reason)` sets for parse failures, never unchanged unavailable content. Different sets show before/after counts and shared/before-only/after-only identities; absent historical failure evidence remains “not recorded”, not zero or unchanged. Workspace summaries span all members, name members with recorded read/parse failures and count members lacking failure evidence. Per-folder coverage stays in JSON `findings[].folder`; human output gives one explanation of counted-only folder totals.

Producer notes show each differing version/backend/configuration value without asserting whether counting behavior changed. The no-recorded-settings-differences caveat depends on counter, policy, module and identity findings, independently of coverage or presence findings. JSON findings and their messages, schemas, arithmetic and exit statuses are unchanged.

Folder-presence findings cite the recorded source of each folder identity: breakdown, coverage paths, modules or root/member metadata. Ancestors inherit those evidence pointers; a coverage-only folder is not attributed to nonexistent breakdown rows.

## Path Explanations

`locmetry explain PATH` inspects one current file with the same selection, parser and classifier used by measurement. Single-root mode accepts `--root`, `--config`, `--exclude`, `--hidden`, `--respect-ignore-files` and `--no-user-config`; target paths are root-relative. `--workspace FILE` instead requires `MEMBER/PATH`, applies that member's policy and rejects single-root policy flags. It validates membership but loads only the queried member's configuration. Selected names determine module inference; only the target's content is counted. Absolute targets, parent traversal and spellings that differ from directory entries are rejected. Case-insensitive or Unicode-equivalent filesystem aliases must use the exact on-disk spelling. Directories and missing paths fail; encountering a symlink stops inspection without following it.

Table, JSON and Markdown describe the same result. The default is table on a terminal, JSON otherwise; `--format` chooses explicitly. JSON is the full explanation on stdout, with no separate headline or `--output` option. It has `document_type: "explanation"`, its own `schema_version: "1"`, canonical member/root location, target path, optional member name, nullable loaded configuration paths, effective selection and a tagged `result`. `--schema explanation` derives its structure and semantics from the serialized types through the shared schema generator. An explanation cannot be used as a comparison input.

| `result.state` | Evidence |
| --- | --- |
| `counted` | The same `file` object as a measurement with `--files`, plus `decision`: winning stage, matching patterns, optional user reason and optional module path/detector. Counts may be a known zero. |
| `excluded` | `at` and `patterns` recorded by the ordinary exclusion filter at the target or its pruned ancestor. |
| `not_selected` | `reason` is `ignore_policy` if disabling only ignore filtering selects the target, `hidden_policy` if enabling only hidden paths does so, or `reason_not_recorded` when neither establishes a cause. No winning ignore file/line is claimed. |
| `unrecognized` | Selected but no recognized or mapped parser; no count is supplied. |
| `unread` | Selected and recognized but unreadable; counts unavailable. |
| `parse_failed` | The measurement failure's path, stable reason and human message; counts unavailable. |
| `symlink` | `at` identifies the target or ancestor link. No link destination or competing filter is inspected. |

Workspace result paths, including module and failure paths, are member-prefixed; globs remain relative to the member root. Stage precedence and language-mapping behavior remain those of [matching and precedence](#matching-and-precedence). The [command outcome](#command-outcomes) distinguishes a successful explanation from unavailable target counts or a command failure. Unrelated counting failures do not make the target partial. Filesystem changes between runs or probes are not snapshot-isolated. The [troubleshooting guide](guides/selection.md) owns task examples and explains the evidence limits.

## Command Outcomes

Diagnostics go to stderr; successful or partial documents use the selected stdout format. Exit status describes execution and known count availability, not whether a policy, purpose or source change is desirable.

| Command | Exit 0 | Exit 2 |
| --- | --- | --- |
| Single-root or workspace measurement | A report with `metadata.backend_inaccurate = false`. Unrecognized files may still be present. | A report with `metadata.backend_inaccurate = true`, including read/parse failures or backend-reported inaccuracy without identified paths. Coverage lists known paths; their absence does not cancel the flag. |
| `compare` | Valid inputs with no observed inaccuracy/read/parse failure. Missing historical evidence, policy differences and presence changes alone do not make the result partial. | A comparison using an input with recorded inaccuracy or read/parse failures. Missing lines are not estimated. |
| `explain` | A `counted`, `excluded`, `not_selected`, `unrecognized` or `symlink` result. | An `unread` or `parse_failed` result for the target. |

Exit 1 means a command, configuration, input, traversal, arithmetic or output failure. A usable document is not promised. In particular, traversal errors abort `explain`; they do not produce a successful result with incomplete module inference. Help, version and schema queries exit 0 when successful.

## Sensitive Analysis

Public documentation and fixtures are synthetic; [CLAUDE.md](../CLAUDE.md#sensitive-analysis) governs this repository's real measurements. That is a development-data policy: users of the released CLI may write reports anywhere outside the scan root.
