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

The configuration below is implemented. Provisional [JSON/regex profiles](poc/README.md) remain accepted until existing profiles are migrated. Locmetry is implemented in Rust with the Tokei library embedded. The installed executable must not require Python, Git, a separate Tokei executable, or a separately installed Rust runtime. Homebrew Rust dependencies are build-only; release bottles should require no runtime formula dependencies. [Tokei integration and Homebrew distribution](docs/tokei-integration.md) records the implemented boundary and planned packaging.

## Measurement Model

| Concept | Meaning |
| --- | --- |
| Scope | Whether a file belongs in the measurement; external/reference material is an inclusion decision, not a test category. |
| Kind | `source`, `data`, `documentation`, or `other`, derived from the backend language through a reviewed mapping; a JSON test fixture remains data. |
| Purpose | `production`, `test`, or `unclassified`. Production means maintained non-test source, not necessarily shipped runtime code. |
| Category | Optional project-defined label such as implementation, tooling, test-support, example, benchmark, or evidence. It does not change counting. |

Report code, comments, blanks, and total lines separately. Preserve each file's language and classification provenance; a language supplied by a mapping records the mapping ID. Embedded snippets inherit the host file's scope and purpose; documentation snippets do not inflate source totals. Documentation embedded in a non-documentation file, such as Rust `///` doc comments that the backend parses as Markdown, contributes its lines, including doctest code, as comments of the host language. Derive human summaries and versioned JSON from the same records.

## Project Configuration

Use `.locmetryrc` in TOML. Discover it in the selected scan root, which defaults to the current directory; an explicit `--config` path may select a different file without changing that root. Without a discovered or explicit configuration, built-in settings apply; an explicit configuration that cannot be read fails. All patterns remain relative to the scan root. Project configuration overrides built-in settings; explicit CLI settings override project settings: `--hidden BOOL`, `--respect-ignore-files BOOL`, and repeatable `--exclude GLOB`, which adds to the configured exclusions.

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
3. Apply built-in test conventions if enabled; provenance records the convention, for example `builtin:gradle-test-source-set`.
4. Apply the first matching ordinary `rules` entry.
5. Assign `classification.fallback` if no classification matched.

An entry matches when any of its patterns matches the path and, if `kind` is present, the file's kind is listed. A match terminates purpose/category classification. Each file records its provenance as the matching entry ID, `builtin:<convention>`, or `fallback`; fallback files take the fallback purpose as their category. Broad production rules therefore do not swallow conventional tests, while an explicit archived-evidence override can take precedence over test filenames. No ordinary run requires an LLM; agent assistance proposes reviewable configuration.

| Setting | Type and contract |
| --- | --- |
| `version` | Required integer; initially `1`. Unsupported versions fail. |
| `backend` | String; initially only `tokei`, also the default. |
| `scope.hidden` | Boolean, default `true`; hidden-file selection is independent of metadata exclusions. |
| `scope.respect_ignore_files` | Boolean, default `true`; controls the adapter's ignore-file policy. |
| `scope.exclude` | Array of root-relative glob strings, default empty, added to the default exclusions. |
| `scope.default_excludes` | Boolean, default `true`; applies `**/.git/**`, `**/node_modules/**`, and `**/.build/**`. `false` is the only way to drop them. `build` directories are not excluded by default because they can hold maintained source; ignore files normally exclude build output. |
| `classification.builtin_tests` | Boolean, default `true`; enables documented test conventions. The configuration `version` pins the convention set, so upgrading Locmetry does not silently reclassify files. Conventions cover common test directories, test filename suffixes, and framework source sets such as Gradle/Kotlin `src/*Test/**` and `testFixtures`. Ownership-ambiguous names such as `fixtures` are left to project entries. The version-1 set is defined by `BUILTIN_TESTS_V1` in [src/config.rs](src/config.rs) and recorded in every report as `metadata.config.builtin_test_conventions`. |
| `classification.fallback` | Purpose enum, default `unclassified`; projects can explicitly choose another purpose. |
| `languages.builtin` | Boolean, default `true`; enables documented language mappings for common file types the backend does not recognize, such as `.gradle` scripts as Groovy and `.jsonc` as JSON. The configuration `version` pins the set, defined by `BUILTIN_LANGUAGES_V1` in [src/config.rs](src/config.rs) and recorded in every report as `metadata.config.builtin_language_mappings`. |
| `languages.mappings` | Ordered array of `{id, match, language}` entries, applied before built-in mappings. `language` must be a backend language name. Mappings apply only to selected files the backend does not recognize; backend detection is never overridden. The first matching mapping counts the file with the backend's parser for that language. |
| `overrides`, `rules` | Ordered arrays of entries; omitted arrays are empty. Each entry requires a unique string `id` (not `fallback`, which names the fallback provenance), a nonempty string-array `match`, and a valid `purpose`. `category` defaults to the purpose. `kind` is an optional nonempty array of kinds. `reason` is an optional string recorded in the effective-policy snapshot. |

Match normalized root-relative paths with case-sensitive globs: `*` stays within a segment, `**` spans zero or more directories, `?` matches one non-separator character, `[...]` matches one character from a class, and `{a,b}` matches any listed alternative. Backslash escapes the next character on every platform. Any pattern in `match` selects its entry. Patterns do not use implicit basename matching, negation, or absolute paths. Reject invalid patterns, unknown keys, duplicate IDs, and invalid types/enums.

Explicit exclusions still apply when ignore files are disabled. Removing a built-in exclusion does not override a repository ignore rule; use the project's ignore exceptions or an explicit ignore-file policy. Backend pattern syntax must not silently alter these semantics.

## Reporting and Coverage

Expose direct folder totals and recursive totals. A recursive parent contains its direct files plus its immediate children's recursive totals; displayed recursive rows are not all additive. Count original file reports and embedded statistics once.

Keep output format and folder depth as CLI options initially. Exact flags, JSON fields, and incomplete-result exit behavior remain open. Recommend clear backend/version, scope, effective-policy, and classification-provenance fields, with diagnostics separated from JSON stdout.

Surface known counting-coverage limitations: report how many files were selected and counted, list selected files the backend did not recognize, and separately list recognized files it could not read. Path rules cannot recover source files omitted by the backend. Language mappings recover unrecognized files; absent records must not be represented as proven zero LOC. Inline tests in mixed files remain a whole-file classification limitation.

## Sensitive Analysis

Keep measured-project identities, paths, configurations, counts, raw responses, and analyses exclusively in Git-ignored `docs/research/local/`. Public documentation and fixtures must be generic or synthetic. The local measurement restriction is a development-data policy, not a requirement that users of a released CLI write reports into this repository.
