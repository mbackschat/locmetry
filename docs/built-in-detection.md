# Built-in Detection

Locmetry classifies a project on its first run, without configuration, through built-in detection: conventions recognize tests and build tooling by their paths, and module detectors recognize build and framework files and count the source folders those tools document as production. This page explains how they decide and lists every one. The [product contract](PRODUCT.md#matching-and-precedence) owns the precedence and compatibility guarantees.

## How Detection Works

Each selected file takes the first stage that matches it:

1. The project's `overrides`.
2. The built-in [test conventions](#test-conventions), then the [build conventions](#build-conventions).
3. The project's `rules`.
4. The [module detectors](#module-detectors): source inside a detected module's production layout becomes `production` with category `implementation`.
5. The fallback, `other` with category `unclaimed`, which reports list by folder for review.

- **Conventions** match path patterns only. They run before the project's rules, so a broad rule such as `src/**` does not turn tests into production, and after its overrides, so a project can state exceptions. Test conventions come first: `tests/gradlew` is a test, not a build wrapper.
- **Module detectors** look for descriptor file names, such as `package.json` or `next.config.ts`, among the selected files. The folder holding a descriptor becomes a build module, and the detector's source layout inside it is production. Detectors read file names only: they parse no manifest and run no build, so a project with custom source folders needs a rule. The innermost module decides a file, so a nested project never inherits an outer project's layout, and tests inside a layout stay tests because conventions run earlier.
- **Language mappings** count files the counter does not recognize, such as `.jsonc` files as JSON; they never change a recognized language.
- **Default exclusions** skip version control, dependency checkouts and the SwiftPM and Lake build folders, as [`scope.default_excludes`](PRODUCT.md#matching-and-precedence) defines.

Every file records what decided it as its `rule`, such as `builtin:js-test-file` for a convention or `builtin:npm-module` for a module layout, and `locmetry explain PATH` shows the evidence. In `.locmetryrc`, `classification.builtin_tests`, `classification.builtin_build`, `classification.builtin_modules` and `languages.builtin` switch the built-ins off. The configuration `version` selects each set, and every report records what it applied as `metadata.config.builtin_conventions`, `metadata.config.builtin_language_mappings` and `metadata.modules`.

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
| `dotnet` | `*.csproj`, `*.fsproj`, `*.vbproj` | every `*.cs`, `*.fs`, `*.vb`, `*.razor`, `*.cshtml` and `*.xaml` file and Blazor's scoped `*.razor.css`, which the build compiles or bundles; static files in `wwwroot/` stay unclaimed. Test projects named `<Project>.Tests` or `<Project>.Test` are tests through the `dotnet-test-project` convention, and designer, source-generator and EF Core snapshot files are `production` with category `generated` through the `dotnet-generated` convention |
| `composer` | `composer.json` | `src/**`, the PSR-4 convention |
| `symfony` | `symfony.lock` | `src/**`, `assets/**`, `config/**`, `migrations/**`, `templates/**` |
| `laravel` | `artisan` | `app/**`, `bootstrap/**`, `config/**`, `database/migrations/**`, `routes/**`, and each folder in `resources/` except `lang/`; translations stay unclaimed |
| `rails` | `config.ru` | `app/**`, `lib/**`, `config/**`, `db/migrate/**`; RSpec files are tests through the `rspec-file` convention |
| `gem` | `*.gemspec` | `lib/**`, `exe/**`, the RubyGems layout |
| `django` | `manage.py` | top-level packages with `__init__.py`, as for `python` but without `src/**`; each app's `tests.py` is a test through the `django-tests-module` convention |

A descriptor written as `*.ext` matches any file name with that extension; all others match exact names. Framework detectors claim only the folders their documentation names, never the whole package, because real applications keep scripts, mocks and vendored code beside them. Where nested modules and build systems sharing a folder meet, [module detection](PRODUCT.md#module-detection) defines which module decides and which is credited.

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
| `dotnet-generated` | `*.Designer.cs`, `*.g.cs`, `*.g.i.cs`, `*ModelSnapshot.cs` | `production:generated`: designer, source-generator and EF Core snapshot files |

## Language Mappings

Applied after the project's and personal mappings, only to files the counter does not recognize.

| Mapping | Matches | Counted as |
| --- | --- | --- |
| `gradle-script` | `**/*.gradle` | Groovy |
| `jsonc` | `**/*.jsonc` | JSON |
| `json-lines` | `**/*.jsonl` | JSON |
| `java-properties` | `**/*.properties` | INI |

The [built-in definitions](../src/config/builtin.rs) and [module detectors](../src/modules.rs) are the executable source; `cargo test --locked --test integration builtin_detection::` checks that this page lists every one in order.
