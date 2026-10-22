# Framework Layouts

<!-- doc-status
{
  "status": "draft",
  "date": "2026-10-23",
  "summary": "Built-in detectors and conventions for popular frameworks whose source is not in src/, and for .NET, PHP, Ruby and Django projects, so their production source is classified without configuration.",
  "evidence": ""
}
-->

## Problem

Locmetry claims production source through build descriptors and their documented default layouts. The [framework layout measurements](../research/framework-layouts.md) of 2026-10-23 measured starters and open-source applications of the most used web frameworks of the Stack Overflow Developer Survey 2025 without configuration:

- Projects keeping their code in `src/` work: Angular, Vite with React or Vue, Astro, SvelteKit, NestJS, Expo, Spring Boot and FastAPI.
- Next.js, Nuxt and React Router applications are 93–100% unclaimed. Their package root, or `app/`, is the source root, while the npm layout claims only `src/`.
- .NET, Laravel, Symfony and Rails projects are 38–97% unclaimed, because no detector knows their descriptors. A Django project without packaging metadata is 99% unclaimed.
- Root tool configuration such as `vite.config.ts` stays unclaimed in every JavaScript project.

These frameworks are used by between 4% and 22% of professional developers each. A new user's first measurement of such a project reports almost all of it as needing a decision.

## Design

**Required: new built-in detectors and conventions.** Each row follows the framework's documented default layout, cited in the measurement record. All are added to the version-1 sets, which a release may extend; no existing entry changes.

| Kind | ID | Descriptor in a folder | Claims |
| --- | --- | --- | --- |
| Detector | `next` | `next.config.{js,cjs,mjs,ts}` | `app/**`, `pages/**`, `src/**`; the shared-code folders the Next.js docs name, `components/**`, `lib/**`, `ui/**`, `utils/**`, `hooks/**`, `styles/**`; and the root `instrumentation`, `instrumentation-client`, `proxy`, `middleware` and `mdx-components` files |
| Detector | `nuxt` | `nuxt.config.{js,mjs,ts}` | `app/**`, `server/**`, `shared/**`, `layers/**`, `modules/**`; the Nuxt 3 root folders `assets/**`, `components/**`, `composables/**`, `layouts/**`, `middleware/**`, `pages/**`, `plugins/**`, `utils/**`; and the root `app.vue`, `error.vue` and `app.config.*` |
| Detector | `react-router` | `react-router.config.{js,ts}` | `app/**` |
| Detector | `dotnet` | `*.csproj`, `*.fsproj`, `*.vbproj` | `**/*.{cs,fs,vb,razor,cshtml}`: the SDK's default compile items and Razor files; static files in `wwwroot/` stay unclaimed |
| Detector | `composer` | `composer.json` | `src/**`, the PSR-4 convention Symfony and most packages use |
| Detector | `symfony` | `symfony.lock` | `src/**`, `assets/**`, `config/**`, `migrations/**`, `templates/**` |
| Detector | `laravel` | `artisan` | `app/**`, `bootstrap/**`, `config/**`, `database/migrations/**`, `resources/**`, `routes/**` |
| Detector | `rails` | `config.ru`, which `rails new` creates | `app/**`, `lib/**`, `config/**`, `db/migrate/**` |
| Detector | `gem` | `*.gemspec` | `lib/**`, `exe/**`, the RubyGems layout |
| Detector | `django` | `manage.py` | The Python flat layout: each top-level package except names such as `docs` and `scripts` |
| Test convention | `dotnet-test-project` | | `**/*.*Test/**` and `**/*.*Tests/**`, the `Project.Tests` naming of .NET test projects |
| Test convention | `rspec-file` | | `**/*_spec.rb`, RSpec's default pattern; `**/spec/{spec_helper,rails_helper}.rb`; and `**/spec/{support,factories}/**/*.rb` |
| Build convention | `js-tool-config` | | `**/*.config.{js,cjs,mjs,ts,cts,mts}` as `production:build-tooling` |

- The existing rules apply unchanged: test and build conventions run before detectors, the innermost module decides, and detectors sharing a folder combine their layouts. A Next.js app in `apps/web` of a Turborepo is its own module.
- Next.js and Nuxt layouts list only folder and file names their documentation gives. Next.js calls itself unopinionated about organization and names `components`, `lib`, `ui`, `utils`, `hooks` and `styles` as examples, so a feature folder such as `modules/` stays unclaimed and is reported for one rule. Across thirteen real applications this claims 89–100% of the source files in ten; the [measurement record](../research/framework-layouts.md#documented-folders-versus-whole-package) lists the rest.
- Descriptors given as `*.csproj` and `*.gemspec` match by file extension; all others match exact names, as today.
- The configure guide, `--help` text on module detection, and PRODUCT's [Module Detection](../PRODUCT.md#module-detection) list the new detectors and conventions.

**Optional:** `requirements.txt` as a further Python descriptor, for applications without packaging metadata. Only top-level packages would be claimed, but documentation builds and tool folders also carry such files, so it needs its own evidence.

**Excluded:**

- Reading descriptor contents, such as `package.json` dependencies or Composer's `autoload` paths. Detectors read only file names, which keeps detection fast and its results explainable from the tree.
- Whole-package layouts for Next.js and Nuxt. In the same applications they would silently count `scripts/`, Playwright helpers, mocks, vendored code and hidden tool folders as production, while the documented layout leaves such folders visible as unclaimed.
- A `spec/` test directory. Production Go packages are named `spec`, such as `pkg/validation/spec` in Kubernetes' kube-openapi, which many Go projects vendor; conventions run before detectors and would make them tests.
- Claiming the whole package of every npm package without `src/`. Repositories often have a root `package.json` only for tooling, and their scripts would silently become production.
- Frameworks that already work, and layouts a release-specific option changes, such as React Router's `appDirectory`. A rule corrects those.

## Feasibility

Detectors are a table of descriptor names and layouts in `src/modules.rs`; the Python detector already computes its layout from file names. The new parts are extension-pattern descriptors and the "all top-level entries except one" layout. Conventions are a table in `src/config/builtin.rs`. Reports already record the applied conventions and detected modules, so comparisons with earlier reports show the reclassification as configuration and module changes.

## Acceptance

- Synthetic fixtures for each detector and convention: the claimed paths are production with the detector's provenance, a Next.js `scripts/` file and a .NET `wwwroot/` script stay unclaimed, a `.Tests` project and a `*_spec.rb` file are tests, a Go file in `pkg/spec/` stays production, and a `*.config.ts` file is build tooling.
- At the commits in the [measurement record](../research/framework-layouts.md), the Next.js, Nuxt, React Router, .NET, Laravel, Symfony, Rails and Django projects leave unclaimed only the folders the record lists for them. The projects that already worked keep their production totals except for tool configuration, which becomes build tooling.
- An agent regression check before release, because `--help` and the guides change.
