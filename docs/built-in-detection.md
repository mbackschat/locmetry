# Built-in Detection

Locmetry classifies a project on its first run, without configuration, through built-in detection: conventions recognize tests and build tooling by their paths, module detectors recognize build and framework files and count the source folders those tools document as production, and generated-code markers set generated code in those folders apart. This page explains how they decide and lists every one. The [product contract](PRODUCT.md#matching-and-precedence) owns the precedence and compatibility guarantees.

## How Detection Works

Each selected file takes the first stage that matches it:

1. The project's `overrides`.
2. The built-in [test conventions](#test-conventions), then the [build conventions](#build-conventions).
3. The project's `rules`.
4. The [module detectors](#module-detectors): source inside a detected module's production layout becomes `production` with category `implementation`, or category `generated` when it carries a [generated-code marker](#generated-code-markers).
5. The fallback, `other` with category `unclaimed`, which reports list by folder for review.

- **Conventions** match path patterns only. They run before the project's rules, so a broad rule such as `src/**` does not turn tests into production, and after its overrides, so a project can state exceptions. Test conventions come first: `tests/gradlew` is a test, not a build wrapper.
- **Module detectors** look for descriptor file names, such as `package.json` or `next.config.ts`, among the selected files. The folder holding a descriptor becomes a build module, and the detector's source layout inside it is production. Detectors read file names only: they parse no manifest and run no build, so a project with custom source folders needs a rule. The innermost module decides a file, so a nested project never inherits an outer project's layout, and tests inside a layout stay tests because conventions run earlier.
- **Generated-code markers** are documented signals that a tool generated a file, such as Go's `// Code generated … DO NOT EDIT.` line or a project's `linguist-generated` declaration in `.gitattributes`. They refine only what a module layout claimed: the file stays production and keeps its module's credit, but its category becomes `generated`, so `--show-categories` and `--table-category` separate it, and the largest-module-files review lists only hand-written files. Overrides, conventions and rules decide first, so a project that classifies its generated code itself is unaffected.
- **Language mappings** count files the counter does not recognize, such as `.jsonc` files as JSON; they never change a recognized language.
- **Default exclusions** skip version control, dependency checkouts and the SwiftPM and Lake build folders, as [`scope.default_excludes`](PRODUCT.md#matching-and-precedence) defines.

Every file records what decided it as its `rule`, such as `builtin:js-test-file` for a convention, `builtin:npm-module` for a module layout or `builtin:go-generated-header` for a marker, and `locmetry explain PATH` shows the evidence, including a marker's matched line. In `.locmetryrc`, `classification.builtin_tests`, `classification.builtin_build`, `classification.builtin_modules`, `classification.builtin_generated` and `languages.builtin` switch the built-ins off. The configuration `version` selects each set, and every report records what it applied as `metadata.config.builtin_conventions`, `metadata.config.builtin_generated_markers`, `metadata.config.builtin_language_mappings` and `metadata.modules`.

Each built-in rests on its tool's or framework's own documentation and was measured on real projects in both directions: what it classifies correctly and what it claims wrongly. A wrong claim is weighed higher, because it is silent, while unclaimed code is listed for review. The [research records](research/README.md) hold the measurements, such as the [framework layouts](research/framework-layouts.md).

## Module Detectors

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
| `dart` | `pubspec.yaml` | `lib/**`, `bin/**`; `test/` is a test directory, and Flutter's `integration_test/` Dart files are tests through the `flutter-integration-test` convention |
| `next` | `next.config.{js,cjs,mjs,ts}` | `app/**`, `pages/**`, `src/**`; the root `instrumentation`, `instrumentation-client`, `proxy`, `middleware` and `mdx-components` files; and the folders Next.js documents as generalized placeholders, `components/**`, `lib/**`, `ui/**`, `utils/**`, `hooks/**`, `styles/**`. Other folders, such as `scripts/`, stay unclaimed |
| `nuxt` | `nuxt.config.{js,mjs,ts}` | `app/**`, `server/**`, `shared/**`, `layers/**`, `modules/**`; the Nuxt 3 root folders `assets/**`, `components/**`, `composables/**`, `layouts/**`, `middleware/**`, `pages/**`, `plugins/**`, `utils/**`; and the root `app.vue`, `error.vue` and `app.config.*` |
| `react-router` | `react-router.config.{js,ts}` | `app/**`, framework mode's default `appDirectory` |
| `dotnet` | `*.csproj`, `*.fsproj`, `*.vbproj` | every `*.cs`, `*.fs`, `*.vb`, `*.razor`, `*.cshtml` and `*.xaml` file and Blazor's scoped `*.razor.css`, which the build compiles or bundles; static files in `wwwroot/` stay unclaimed. Test projects named `<Project>.Tests` or `<Project>.Test` are tests through the `dotnet-test-project` convention, and designer, source-generator and EF Core snapshot files are `production` with category `generated` through the [`roslyn-generated` marker](#generated-code-markers) |
| `composer` | `composer.json` | `src/**`, the PSR-4 convention |
| `symfony` | `symfony.lock` | `src/**`, `assets/**`, `config/**`, `migrations/**`, `templates/**` |
| `laravel` | `artisan` | `app/**`, `bootstrap/**`, `config/**`, `database/migrations/**`, `routes/**`, and each folder in `resources/` except `lang/`; translations stay unclaimed |
| `rails` | `config.ru` | `app/**`, `lib/**`, `config/**`, `db/migrate/**`; RSpec files are tests through the `rspec-file` convention |
| `gem` | `*.gemspec` | `lib/**`, `exe/**`, the RubyGems layout |
| `django` | `manage.py` | top-level packages with `__init__.py`, as for `python` but without `src/**`; each app's `tests.py` is a test through the `django-tests-module` convention |

A descriptor written as `*.ext` matches any file name with that extension; all others match exact names. Framework detectors claim only the folders their documentation names, never the whole package, because real applications keep scripts, mocks and vendored code beside them. Where nested modules and build systems sharing a folder meet, [module detection](PRODUCT.md#module-detection) defines which module decides and which is credited.

## Generated-Code Markers

Each refines a module layout's claim to `production` with category `generated` and becomes the file's `rule`, such as `builtin:roslyn-generated`. Only markers bound to a documented position count: free-text checks, such as "generated" near "do not edit" anywhere in a header, claimed hand-written code in the generators' own repositories, where the marker strings appear in templates and constants.

| Marker | Applies to | Matches | Basis |
| --- | --- | --- | --- |
| `go-generated-header` | Go | A line matching `^// Code generated .* DO NOT EDIT\.$` before the first line that is neither blank nor a comment; the same line after the `package` clause does not count | [`go help generate`](https://pkg.go.dev/cmd/go#hdr-Generate_Go_files_by_processing_source) |
| `roslyn-generated` | C#, Visual Basic | A file name ending `.designer`, `.generated`, `.g` or `.g.i` before the extension, ignoring case, or starting `TemporaryGeneratedFile_`; or a comment before the first code line containing `<auto-generated` or `<autogenerated`, read after a byte-order mark | Roslyn's `GeneratedCodeUtilities`, which the .NET analyzers use to skip generated code |
| `java-generated-annotation` | Java | `@Generated` from `javax.annotation.processing`, `javax.annotation` or `jakarta.annotation`, by qualified name or single-type import, annotating the first top-level type; on a member, or from another package such as Lombok's, it does not count | [Java SE `Generated`](https://docs.oracle.com/en/java/javase/21/docs/api/java.compiler/javax/annotation/processing/Generated.html), which can also mark generated parts of a hand-written file |
| `linguist-generated` | Any source | A path a `.gitattributes` file inside the scan root marks `linguist-generated`, with any value but `false`, under Git's precedence: later lines and deeper files win | [GitHub's attribute](https://docs.github.com/en/repositories/working-with-files/managing-files/customizing-how-changed-files-appear-on-github) |

A `.gitattributes` line with `-linguist-generated` or `linguist-generated=false` declares a file not generated, and that also overrides the other markers, as it overrides GitHub Linguist's own checks; `!linguist-generated` undoes earlier declarations. A project's declaration is its own judgment and can be broad, so a rule reclassifies files it should not cover. Code from generators that write no documented marker, such as MyBatis Generator or a project's own scripts, still needs a policy entry. The [generated-code research](research/generated-code-markers.md) records what each marker found and claimed on real projects, and which candidates were deferred.

## Test Conventions

Each assigns purpose `test` with category `test`, in this order. Ambiguous names, such as `fixtures` and a `spec` folder, are left to project entries.

| Convention | Matches | Basis |
| --- | --- | --- |
| `test-directory` | `**/{test,tests,__test__,__tests__}/**` | Common test folder names |
| `gradle-test-source-set` | `**/src/*Test/**`, `**/src/testFixtures/**` | Gradle and Kotlin test source sets, such as `src/test` and `src/jvmTest` |
| `snapshot-directory` | `**/__snapshots__/**` | Jest and Vitest snapshots |
| `js-test-file` | `**/*.{test,spec}.{js,jsx,mjs,cjs,ts,tsx,mts,cts}` | JavaScript and TypeScript test files |
| `js-e2e-spec-file` | `**/*.e2e-spec.{js,jsx,mjs,cjs,ts,tsx,mts,cts}` | NestJS end-to-end suites; a bare `-spec` suffix also names specifications |
| `jvm-test-class` | `**/*{Test,Tests,TestCase}.{java,kt,kts,groovy,scala}` | JVM test class names |
| `go-test-file` | `**/*_test.go` | Go's test files |
| `rust-test-module` | `**/tests.rs` | A Rust unit-test module moved out of line, which the Cargo layout would otherwise claim |
| `swift-test-target` | `**/*Tests/**/*.swift` | Swift Package Manager's `Tests/` and Xcode's `<Name>Tests` targets |
| `lean-test-library` | `**/*{Test,Tests}/**/*.lean` | Lake test libraries, such as mathlib's `MathlibTest/` |
| `flutter-integration-test` | `**/integration_test/**/*.dart` | Flutter's device-driven tests |
| `python-test-file` | `**/test_*.py`, `**/*_test.py`, `**/conftest.py` | pytest's discovery names |
| `dotnet-test-project` | `**/*.*Test/**`, `**/*.*Tests/**` | .NET test projects named `<Project>.Tests`, whose files carry no test suffix |
| `rspec-file` | `**/*_spec.rb`, `**/spec/{spec_helper,rails_helper}.rb`, `**/spec/{support,factories}/**/*.rb` | RSpec's default pattern and helpers; a whole `spec/` folder would claim Go packages named `spec` |
| `django-tests-module` | `**/tests.py` | The `tests.py` Django's `startapp` creates in every app |
| `js-test-runner-config` | `**/{vitest,jest,playwright,cypress}.config.*` and `.*.config.*` variants | Test-runner configuration |

## Build Conventions

Applied after the test conventions, in this order. Build scripts are maintained but are not implementation, so they keep their own category.

| Convention | Matches | Purpose and category |
| --- | --- | --- |
| `build-script` | Gradle `build` and `settings` scripts, `pom.xml`, `CMakeLists.txt`, `*.cmake`, Makefiles and `lakefile.lean` | `production:build-tooling` |
| `build-wrapper` | `gradlew`, `gradlew.bat`, `mvnw`, `mvnw.cmd`, and files under `gradle/wrapper/` or `.mvn/wrapper/` | `other:third-party-wrapper`, because the build tool generates them |
| `js-tool-config` | `<tool>.config.*` and `<tool>.*.config.*` for vite, webpack, rollup, rolldown, esbuild, tsup, tsdown, babel, postcss, tailwind, eslint, prettier, stylelint, lint-staged, commitlint, metro, next, nuxt, astro, svelte and react-router | `production:build-tooling`; matched by tool name, because application code such as Angular's `app.config.ts` also uses the suffix |

## Language Mappings

Applied after the project's and personal mappings, only to files the counter does not recognize.

| Mapping | Matches | Counted as |
| --- | --- | --- |
| `gradle-script` | `**/*.gradle` | Groovy |
| `jsonc` | `**/*.jsonc` | JSON |
| `json-lines` | `**/*.jsonl` | JSON |
| `java-properties` | `**/*.properties` | INI |

The [built-in definitions](../src/config/builtin.rs) and [module detectors](../src/modules.rs) are the executable source; `cargo test --locked --test integration builtin_detection::` checks that this page lists every one in order.
