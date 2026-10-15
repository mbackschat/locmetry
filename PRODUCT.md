# Product Vision & Requirements

## Vision

Measure a project's own codebase with understandable scope, test/production distinctions, and useful folder totals. Locmetry is a policy-and-reporting layer over an established counter. Agents consume predictable JSON; developers get readable summaries and reviewable project configuration.

The product rests on three pillars:

- **Expressive configuration:** reviewable project policy covers real repositories' scope, ownership, and classification needs, including build tooling, examples, generated code, and evidence.
- **Reuse with gap filling:** the embedded counter does the counting; Locmetry layers on top of its library and fills gaps it leaves, such as unrecognized file types, instead of reimplementing it.
- **Rich output:** reports summarize the measurement, while JSON keeps every file record, provenance, and the effective policy so agents and scripts can answer further questions, including their own summations.

## Scope and Status

- Required: embedded Tokei integration in a Rust executable, test/production classification, project configuration and CLI options, configurable exclusions, language mappings for files the backend does not recognize, folder breakdowns and sums, JSON, and human output.
- Optional: additional counting adapters, custom content-kind overrides, and configuration self-checks that assert expected classifications for named paths without requiring those files to exist.
- Excluded initially: a new counting engine, a generic plugin framework, and semantic splitting of production/tests within individual files.

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

Use `.locmetryrc` in TOML. Discover it in the selected scan root, which defaults to the current directory; an explicit `--config` path may select a different file without changing that root. Without a discovered or explicit configuration, built-in settings apply; an explicit configuration that cannot be read fails. A loaded configuration inside the scan root is policy, not project content: it is excluded as a literal path, which the effective exclusions record. All patterns remain relative to the scan root. Project configuration overrides built-in settings; explicit CLI settings override project settings: `--hidden BOOL`, `--respect-ignore-files BOOL`, and repeatable `--exclude GLOB`, which adds to the configured exclusions. A configuration file ending in `.json`, the retired JSON/regex profile format, is rejected with a hint to translate it to TOML.

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

[[languages.mappings]]
id = "task-scripts"
match = ["scripts/*.task"]
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

### Matching and Precedence

1. Apply scope exclusions and the selected ignore-file policy.
2. Apply the first matching `overrides` entry.
3. Apply built-in test conventions, then built-in build conventions, if enabled; provenance records the convention, for example `builtin:gradle-test-source-set`.
4. Apply the first matching ordinary `rules` entry.
5. Assign `classification.fallback` if no classification matched.

An entry matches when any of its patterns matches the path and, if `kind` is present, the file's kind is listed. A match terminates purpose/category classification. Each file records its provenance as the matching entry ID, `builtin:<convention>`, or `fallback`; fallback files take the fallback purpose as their category. Broad production rules therefore do not swallow conventional tests or build scripts, while an explicit archived-evidence override can take precedence over test filenames. Entries that assign ownership of a whole tree, such as a test-only package whose build scripts are test support, belong in `overrides` so built-in conventions do not preempt them. No ordinary run requires an LLM; agent assistance proposes reviewable configuration.

| Setting | Type and contract |
| --- | --- |
| `version` | Required integer; initially `1`. Unsupported versions fail. |
| `backend` | String; initially only `tokei`, also the default. |
| `scope.hidden` | Boolean, default `true`; hidden-file selection is independent of metadata exclusions. |
| `scope.respect_ignore_files` | Boolean, default `true`; controls the adapter's ignore-file policy. |
| `scope.exclude` | Array of root-relative glob strings, default empty, added to the default exclusions. |
| `scope.default_excludes` | Boolean, default `true`; applies `**/.git/**`, `**/node_modules/**`, and `**/.build/**`. `false` is the only way to drop them. `build` directories are not excluded by default because they can hold maintained source; ignore files normally exclude build output. |
| `classification.builtin_tests` | Boolean, default `true`; enables documented test conventions. The configuration `version` pins the convention set, so upgrading Locmetry does not silently reclassify files. Conventions cover common test directories, test filename suffixes, and framework source sets such as Gradle/Kotlin `src/*Test/**` and `testFixtures`. Ambiguous names such as `fixtures` and `spec` are left to project entries. The version-1 set is defined by `BUILTIN_TESTS_V1` in [src/config.rs](src/config.rs) and recorded in every report as `metadata.config.builtin_conventions`. |
| `classification.builtin_build` | Boolean, default `true`; enables documented build conventions, pinned by the configuration `version`. Build scripts such as Gradle, Maven, CMake, and Make files are `production` with category `build-tooling`, so they stay separate from implementation; build-tool wrappers such as `gradlew` and `mvnw` are `unclassified` with category `third-party-wrapper`. The set is defined by `BUILTIN_BUILD_V1` in [src/config.rs](src/config.rs) and recorded with the test conventions. |
| `classification.fallback` | Purpose enum, default `unclassified`; projects can explicitly choose another purpose. |
| `languages.builtin` | Boolean, default `true`; enables documented language mappings for common file types the backend does not recognize, such as `.gradle` scripts as Groovy and `.jsonc` as JSON. The configuration `version` pins the set, defined by `BUILTIN_LANGUAGES_V1` in [src/config.rs](src/config.rs) and recorded in every report as `metadata.config.builtin_language_mappings`. |
| `languages.mappings` | Ordered array of `{id, match, language}` entries, applied before built-in mappings. `language` must be a backend language name. Mappings apply only to selected files the backend does not recognize; backend detection is never overridden. The first matching mapping counts the file with the backend's parser for that language. |
| `overrides`, `rules` | Ordered arrays of entries; omitted arrays are empty. Each entry requires a unique string `id` (not `fallback`, which names the fallback provenance), a nonempty string-array `match`, and a valid `purpose`. `category` defaults to the purpose. `kind` is an optional nonempty array of kinds. `reason` is an optional string recorded in the effective-policy snapshot. |

Match normalized root-relative paths with case-sensitive globs: `*` stays within a segment, `**` spans zero or more directories, `?` matches one non-separator character, `[...]` matches one character from a class, and `{a,b}` matches any listed alternative. Backslash escapes the next character on every platform. Any pattern in `match` selects its entry. Patterns do not use implicit basename matching, negation, or absolute paths. Reject invalid patterns, unknown keys, duplicate IDs, and invalid types/enums.

Explicit exclusions still apply when ignore files are disabled. Removing a built-in exclusion does not override a repository ignore rule; use the project's ignore exceptions or an explicit ignore-file policy. Backend pattern syntax must not silently alter these semantics.

## Reporting and Coverage

Reports serve aggregation: totals by purpose, category, kind, language, and folder. Count original file reports and embedded statistics once. `breakdown` rows are additive, and so are top-level folder totals, which cover each subtree and, with the root files, every file once. The Markdown folder table lists nested levels, in which a parent includes its subfolders, so its rows are not all additive.

Stdout defaults to `--format table` when it is a terminal and to `--format json` otherwise, so people and agents each get their format without a flag. `table` shows source code per language and per top-level folder, split by purpose, followed by one line each for non-source content, unclassified source, and uncounted files. `json` prints a compact headline: the report without `review`, `breakdown`, `files`, `metadata.config`, `metadata.kinds`, and `metadata.coverage.unrecognized`. `markdown` prints a full human report for saving or reviews; `--depth N` (default 2) limits its folder table. `--output FILE` writes the full report to one file outside the scan root, and the headline names it as `report`; `--output -` prints the full report to stdout instead of the headline and cannot be combined with `--format table` or `markdown`. `--files` adds per-file records to the report for rule debugging. Diagnostics go to stderr. Exit code 0 means a complete report, 2 a partial report whose missing counts come from unreadable files, and 1 a failure without a report, including invalid command-line usage.

JSON schema version `1`:

| Field | Contents |
| --- | --- |
| `schema_version`, `project` | `"1"` and the scan root's directory name. |
| `report` | Headline only, with `--output FILE`: the report file's path. |
| `metadata` | `root`, `measured_at_unix_seconds`, `locmetry_version`, `backend`, `backend_inaccurate`, `backend_configuration`; `scope` (effective hidden, ignore-file, symlink, and `exclude` settings); `kinds` (language lists per content kind); `coverage` (`selected_files`, `counted_files`, `mapped_files`, `unrecognized_files`, `unrecognized_extensions`, the `unrecognized` paths, and the `unread` paths); `config_path`; and `config`, the effective configuration including `builtin_conventions` and `builtin_language_mappings`. |
| `summary` | Totals: `all_counted` (every kind) and `source`; `by_kind_purpose` keyed `kind:purpose`; `source_purpose`, `source_categories` keyed `purpose:category`, and `source_languages`, all restricted to `kind = source`; and `top_folders`, source code lines per purpose for each top-level folder (`.` holds files directly in the root). |
| `review` | Unclassified source files, largest first: `path`, `language`, `category`, `rule`, `code`, `lines`. These are the files a configuration may still need to classify. |
| `breakdown` | One row per folder, kind, purpose, category, and language with `files`, `code`, `comments`, `blanks`, and `lines`. `folder` is each file's parent (`.` for the root); a folder's recursive totals are the sum over rows whose folder equals it or starts with it plus `/`. Embedded-language rows count lines but zero `files`, so every column is additive. |
| `files` | Only with `--files`: one record per counted file, sorted by path: `path`, `language`, `kind`, optional `language_mapping`, `purpose`, `category`, `rule`, `counts` (without `files`), and, only for files with embedded languages, per-language `languages` counts. |

"Unclassified" is a purpose of any kind: unclassified documentation or data is not unclassified source, which `source_purpose` and `review` report. Summary count objects have `files`, `code`, `comments`, `blanks`, and `lines`; summaries omit zero counts and empty maps, except `all_counted`, so an absent entry means zero. Report files put each top-level field and each table row on its own line, so line tools work on rows while the file stays one JSON document.

Surface known counting-coverage limitations: report how many files were selected and counted, list selected files the backend did not recognize, and separately list recognized files it could not read. Path rules cannot recover source files omitted by the backend. Language mappings recover unrecognized files; absent records must not be represented as proven zero LOC. Inline tests in mixed files remain a whole-file classification limitation.

## Sensitive Analysis

Public documentation and fixtures are synthetic; [CLAUDE.md](CLAUDE.md#sensitive-analysis) governs this repository's real measurements. That is a development-data policy: users of the released CLI may write reports anywhere outside the scan root.
