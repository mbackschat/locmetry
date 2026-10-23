# Framework Layouts

<!-- doc-status
{
  "status": "implemented",
  "date": "2026-10-23",
  "summary": "Built-in detectors and conventions for popular frameworks whose source is not in src/, and for .NET, PHP, Ruby and Django projects, so their production source is classified without configuration.",
  "evidence": "[PRODUCT](../../PRODUCT.md#module-detection), [regressions](../../../tests/integration/framework_layouts.rs), [detector run](../../research/framework-layouts.md#detector-run)"
}
-->

Status: historical decision record — implemented. The owner accepted the design on 2026-10-23, and it was implemented test-first and verified on real applications the same day in commit `7d95a41`. Archived on 2026-10-23 under the [retirement procedure](../README.md#retirement-procedure). [PRODUCT](../../PRODUCT.md#module-detection) owns the current behavior, and `locmetry --schema config` lists the executable conventions and layouts; `--help` and the configure guide route there rather than listing detectors. The optional parts and the gaps the [detector run](../../research/framework-layouts.md#detector-run) found are tracked in [PLAN](../../PLAN.md). The wording, examples and limits below describe the reviewed design, not current guarantees.

## Problem

Locmetry claims production source through build descriptors and their documented default layouts. The [framework layout measurements](../../research/framework-layouts.md) of 2026-10-23 measured starters and real applications of the most used web frameworks, by the Stack Overflow Developer Survey 2025 and by GitHub stars, without configuration:

- Next.js, Nuxt and React Router applications are 93–100% unclaimed. Their package root, or `app/`, is the source root, while the npm layout claims only `src/`.
- .NET, Laravel, Symfony and Rails projects are 38–99% unclaimed, because no detector knows their descriptors. Django projects without packaging metadata are 71–99% unclaimed.
- Projects keeping their code in `src/` get it claimed, but backends in folders such as `server/` stay unclaimed.
- Root tool configuration such as `vite.config.ts` stays unclaimed in every JavaScript project.

A new user's first measurement of such a project reports most of it as needing a decision.

## Design

**Required: new built-in detectors and conventions.** Each row follows the framework's or tool's primary documentation, cited in the measurement record, and passed a check for false claims on real applications. All are added to the version-1 sets, which a release may extend; no existing entry changes.

| Kind | ID | Descriptor in a folder | Claims |
| --- | --- | --- | --- |
| Detector | `next` | `next.config.{js,cjs,mjs,ts}` | `app/**`, `pages/**`, `src/**`, the root `instrumentation`, `instrumentation-client`, `proxy`, `middleware` and `mdx-components` files; and `components/**`, `lib/**`, `ui/**`, `utils/**`, `hooks/**`, `styles/**` |
| Detector | `nuxt` | `nuxt.config.{js,mjs,ts}` | `app/**`, `server/**`, `shared/**`, `layers/**`, `modules/**`; the Nuxt 3 root folders `assets/**`, `components/**`, `composables/**`, `layouts/**`, `middleware/**`, `pages/**`, `plugins/**`, `utils/**`; and the root `app.vue`, `error.vue` and `app.config.*` |
| Detector | `react-router` | `react-router.config.{js,ts}` | `app/**` |
| Detector | `dotnet` | `*.csproj`, `*.fsproj`, `*.vbproj` | `**/*.{cs,fs,vb,razor,cshtml}`: the SDK's default compile items and Razor files; static files in `wwwroot/` stay unclaimed |
| Detector | `composer` | `composer.json` | `src/**`, the PSR-4 convention Symfony and most packages use |
| Detector | `symfony` | `symfony.lock` | `src/**`, `assets/**`, `config/**`, `migrations/**`, `templates/**` |
| Detector | `laravel` | `artisan` | `app/**`, `bootstrap/**`, `config/**`, `database/migrations/**`, `routes/**`, and each folder in `resources/` except `lang/` |
| Detector | `rails` | `config.ru`, which `rails new` creates | `app/**`, `lib/**`, `config/**`, `db/migrate/**` |
| Detector | `gem` | `*.gemspec` | `lib/**`, `exe/**`, the RubyGems layout |
| Detector | `django` | `manage.py` | The Python flat layout: each top-level package except names such as `docs` and `scripts` |
| Test convention | `dotnet-test-project` | | `**/*.*Test/**` and `**/*.*Tests/**`, the `Project.Tests` naming of .NET test projects |
| Test convention | `rspec-file` | | `**/*_spec.rb`, RSpec's default pattern; `**/spec/{spec_helper,rails_helper}.rb`; and `**/spec/{support,factories}/**/*.rb` |
| Test convention | `django-tests-module` | | `**/tests.py`, which Django's `startapp` creates in every app |
| Test convention | `js-test-runner-config` | | `{vitest,jest,playwright,cypress}.config.*` and `{vitest,jest,playwright,cypress}.*.config.*`, in any folder |
| Build convention | `js-tool-config` | | `{vite,webpack,rollup,rolldown,esbuild,tsup,tsdown,babel,postcss,tailwind,eslint,prettier,stylelint,lint-staged,commitlint,metro,next,nuxt,astro,svelte,react-router}.config.*` and the same names with a `.*` variant before `.config`, in any folder, as `production:build-tooling` |
| Build convention | `dotnet-generated` | | `**/*.Designer.cs`, `**/*.g.cs`, `**/*.g.i.cs`, `**/*ModelSnapshot.cs` as `production:generated`: designer, source-generator and EF Core migration metadata files |

- The existing rules apply unchanged: test and build conventions run before detectors, the innermost module decides, and detectors sharing a folder combine their layouts. A Next.js app in `apps/web` of a Turborepo is its own module.
- In the `next` row, only `app`, `pages`, `src` and the root files are Next.js conventions. The docs call themselves unopinionated about organization and name `components`, `lib`, `ui`, `utils`, `hooks` and `styles` as "generalized placeholders" with no framework significance; the row adopts them because they are what the docs show, and feature folders such as `modules/` stay unclaimed for one rule. Across thirteen real Next.js and Nuxt applications this claimed 89–100% of the source files in ten, and line counts of two of them agreed; no generated or vendored files were among the claims.
- Tool configuration is matched by tool name, not by the `*.config.*` suffix. Application code uses the suffix too: Angular's `src/app/app.config.ts`, Nuxt's `app.config.ts`, NestJS configuration modules, Sentry's `sentry.server.config.ts` and fifteen plugin configurations in one Angular and NestJS monorepo. Test-runner configuration is test tooling, so it gets the test purpose.
- The Laravel row leaves translations out: `resources/lang/` held 54% of one application's source as PHP arrays, and Laravel 12 moved them to a root `lang/` folder, which no row claims either.
- Descriptors given as `*.csproj` and `*.gemspec` match by file extension; all others match exact names, as today.
- The configure guide, `--help` text on module detection, and PRODUCT's [Module Detection](../../PRODUCT.md#module-detection) list the new detectors and conventions.

**Optional:**

- `requirements.txt` as a further Python descriptor, for applications without packaging metadata. Only top-level packages would be claimed, but documentation builds and tool folders also carry such files, so it needs its own evidence.
- Conventions for Storybook stories (`*.stories.*`) and Jest manual mocks (`__mocks__/`), which the existing `src/**` layouts already claim as production: 37,314 lines of stories in one Angular application. Their purpose needs a decision, and other ecosystems have not been searched for counterexamples.

**Excluded:**

- Generated code inside claimed layouts, such as Django migrations, OpenAPI clients, swag documentation and MyBatis output, is a problem of every detector, measured at up to 82% of an application's production. It needs a mechanism of its own, tracked in [PLAN](../../PLAN.md); only the .NET designer files, recognizable by name, are handled here.
- Reading descriptor contents, such as `package.json` dependencies or Composer's `autoload` paths. Detectors read only file names, which keeps detection fast and its results explainable from the tree.
- Whole-package layouts for Next.js and Nuxt. In real applications they would silently count `scripts/`, Playwright helpers, mocks, vendored code and hidden tool folders as production.
- A `spec/` test directory. Production Go packages are named `spec`, such as `pkg/validation/spec` in Kubernetes' kube-openapi, which many Go projects vendor; conventions run before detectors and would make them tests.
- A `*.config.*` suffix convention, for the application code listed above.
- Claiming the whole package of every npm package without `src/`. Repositories often have a root `package.json` only for tooling, and their scripts would silently become production.
- Frameworks that already work, and layouts a release-specific option changes, such as React Router's `appDirectory`. A rule corrects those.

## Feasibility

Detectors are a table of descriptor names and layouts in `src/modules.rs`; the Python detector already computes its layout from file names, as the Laravel row's "each folder in `resources/` except `lang/`" needs. The new parts are extension-pattern descriptors and that computed layout. Conventions are a table in `src/config/builtin.rs`. Reports already record the applied conventions and detected modules, so comparisons with earlier reports show the reclassification as configuration and module changes.

## Acceptance

- Synthetic fixtures for each detector and convention: the claimed paths are production with the detector's provenance; a Next.js `scripts/` file, a .NET `wwwroot/` script and a Laravel `resources/lang/` file stay unclaimed; a `.Tests` project, a `*_spec.rb` file, a Django `tests.py` and a `vitest.config.ts` are tests; a Go file in `pkg/spec/` and an Angular `src/app/app.config.ts` stay production; a `vite.config.ts` is build tooling; and an EF Core `*.Designer.cs` is `production:generated`.
- At the commits in the [measurement record](../../research/framework-layouts.md), each measured application leaves unclaimed only the folders the record lists for it, and the false claims the record lists are absent.
- An agent regression check before release, because `--help` and the guides change.
