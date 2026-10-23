# Framework Layouts Out of the Box

Measured 2026-10-23 with the development build of Locmetry 0.9.0 (`--no-user-config`, no project policy) on shallow clones, removed afterwards. Question: does a project built with a popular framework get its production source classified without any configuration? The [module credit proposal](../proposals/archive/module-credit-by-coverage.md) and the [framework layouts proposal](../proposals/archive/framework-layouts.md) use these results.

## Selection

The candidates are the web frameworks most used by professional developers in the [Stack Overflow Developer Survey 2025](https://survey.stackoverflow.co/2025/technology/), with each framework's share given in the table below. Generic runtimes and libraries without a project layout (Node.js, jQuery, Express, Flask, Fastify, WordPress, AngularJS) were left out. For each framework the measurement used an official starter or a well-known open-source application at the commit listed.

## Results

"Unclaimed" is the share of source code lines that no stage decided; it is the code the tool reports as needing a decision. Tests decided by the built-in conventions are not unclaimed.

| Framework (share) | Project measured | Commit | Unclaimed | Why |
| --- | --- | --- | ---: | --- |
| Next.js (21.5%) | [vercel/commerce](https://github.com/vercel/commerce) | `f7521d57d29f` | 100% | Code in `app/`, `components/`, `lib/`; the npm layout claims only `src/` |
| Next.js | [shadcn-ui/taxonomy](https://github.com/shadcn-ui/taxonomy) | `298a8857c712` | 100% | Same |
| Next.js in a Turborepo | [vercel/turborepo](https://github.com/vercel/turborepo) `examples/basic` | `8003ff0fe919` | 93% | Both apps use `app/` |
| Nuxt (4.1%) | [nuxt-ui-templates/dashboard](https://github.com/nuxt-ui-templates/dashboard) | `7d78d4078eed` | 100% | Nuxt 4 code in `app/` and `server/` |
| React Router framework mode | [remix-run/react-router-templates](https://github.com/remix-run/react-router-templates) `default` | `067adb378a00` | 100% | Code in `app/` |
| ASP.NET Core (21.3%), Blazor (7.6%) | [jasontaylordev/CleanArchitecture](https://github.com/jasontaylordev/CleanArchitecture) | `968abf7dc488` | 38% | No .NET detector: all 2,097 production C# lines are unclaimed; only the two npm client apps are claimed |
| ASP.NET Core | [dotnet/eShop](https://github.com/dotnet/eShop) | `913acea66708` | 0% | Claimed by accident: a root `package.json` for Playwright makes the npm layout's `src/**` claim the C# projects |
| Django (11.7%) | [wagtail/bakerydemo](https://github.com/wagtail/bakerydemo) | `c9b38d7a7170` | 99% | `manage.py` and `requirements.txt`, but no `pyproject.toml`, `setup.py` or `setup.cfg`, so no Python module |
| Laravel (9.3%) | [laravel/laravel](https://github.com/laravel/laravel) | `9ffef3bc4241` | 97% | No PHP detector |
| Symfony (4.3%) | [symfony/demo](https://github.com/symfony/demo) | `8d2e2ef75c3e` | 88% | No PHP detector; the code is in `src/` |
| Ruby on Rails (6.2%) | [maybe-finance/maybe](https://github.com/maybe-finance/maybe) | `77b546983275` | 72% | No Ruby detector; the remaining 28% are conventional tests |
| Svelte (6.9%) | [sveltejs/realworld](https://github.com/sveltejs/realworld) | `df796708040f` | 52% | The application in `src/` is claimed; the unclaimed lines are one stylesheet in `static/`, which is a public asset folder |
| Angular (19.8%) | [angular-realworld-example-app](https://github.com/realworld-apps/angular-realworld-example-app) | `dd99ed2cf39c` | 1% | Works: code in `src/` |
| React and Vue with Vite | [vitejs/vite](https://github.com/vitejs/vite) `create-vite` templates | `4ca25e1d3a24` | 4–5% | Works; the unclaimed lines are root tool configuration such as `vite.config.ts` |
| Astro (4.3%) | [withastro/astro](https://github.com/withastro/astro) `examples/blog` | `6cf3423e4c62` | 4% | Works |
| NestJS (7.4%) | [nestjs/typescript-starter](https://github.com/nestjs/typescript-starter) | `0b6d71261ba3` | 19% | Works; the unclaimed 20 lines are root tool configuration |
| Expo (React Native) | [expo/expo](https://github.com/expo/expo) `templates/expo-template-default` | `651db16bdc7f` | 11% | Works: code in `src/app`; the unclaimed lines are a reset script |
| Spring Boot (15.6%) | [spring-projects/spring-petclinic](https://github.com/spring-projects/spring-petclinic) | `500158f73241` | 0% | Works |
| FastAPI (15.1%) | [fastapi/full-stack-fastapi-template](https://github.com/fastapi/full-stack-fastapi-template) | `b1125d883948` | 6% | Works: Python module and npm modules |

## Documented layouts

- **Next.js:** the [project structure](https://nextjs.org/docs/app/getting-started/project-structure) lists `app`, `pages`, `public` and an optional `src` at the top level, with configuration in `next.config.js`. Components and libraries may live anywhere in the project, so the project root is the source root.
- **Nuxt 4:** the [directory structure](https://nuxt.com/docs/4.x/directory-structure) puts application code in `app/`, server code in `server/`, code shared by both in `shared/`, and layers in `layers/`; configuration is `nuxt.config.ts`, and `public/` holds static files.
- **React Router framework mode:** the application lives in `app/` by default (`appDirectory`), configured by `react-router.config.ts`.
- **.NET SDK projects:** the [project SDK](https://learn.microsoft.com/dotnet/core/project-sdk/overview) includes `**/*.cs` below the project file by default (`EnableDefaultCompileItems`), so every source file in a project folder is compiled.
- **Laravel:** the [directory structure](https://laravel.com/docs/12.x/structure) puts almost all application classes in `app/`, routes in `routes/`, migrations and seeders in `database/`, views and uncompiled assets in `resources/`.
- **Rails:** application code in `app/`, extensions in `lib/`, configuration in `config/` and migrations in `db/`.
- **Symfony:** the [best practices](https://symfony.com/doc/current/best_practices.html) recommend the default flat layout: code in `src/`, front-end sources in `assets/`, and `config/`, `migrations/`, `templates/`, `translations/` and `tests/` at the top level; `public/` is the web root. Flex projects carry `symfony.lock`.
- **Django:** `startproject` creates `manage.py` beside the project package; apps are packages next to it.

## Emulation of the proposed layouts

Each proposed detector was emulated with equivalent `[[rules]]`, plus a rule for root tool configuration, on the same commits:

| Project | Unclaimed before | Emulated | Remaining unclaimed |
| --- | ---: | ---: | --- |
| vercel/commerce | 100% | 0% | none |
| shadcn-ui/taxonomy | 100% | 0% | none |
| Turborepo `examples/basic` | 93% | 10% | `packages/eslint-config`, a configuration package without `src/` |
| nuxt-ui-templates/dashboard | 100% | 0% | none |
| React Router `default` | 100% | 9% | the root `Dockerfile` |
| jasontaylordev/CleanArchitecture | 38% | 3% | `build/` scripts and `dotnet new` item templates |
| laravel/laravel | 97% | 5% | `database/factories`, `database/seeders` and `public/index.php` |
| symfony/demo | 88% | 3% | root files and `public/` |
| maybe-finance/maybe | 72% | 3% | `db/` schema and seeds, `public/`, `bin/` |
| wagtail/bakerydemo | 99% | 1% | root files and `vagrant/` |

The Next.js and Nuxt rows emulate a whole-package layout, which the next section replaces. A generic Composer layout of `src/**` alone left 48% of symfony/demo unclaimed, all in `assets/` and `templates/`; the Symfony-specific layout closed it. The Turborepo remainder shows why an npm package without `src/` should not be claimed as a whole: it is a tooling package.

## Documented folders versus whole package

A whole-package layout for Next.js and Nuxt also claims folders that are not application code. To compare it with a layout of only the documented names, the file trees of thirteen applications were read through the GitHub API on 2026-10-23. Counted are JavaScript, TypeScript and Vue files of the application package, without nested packages, conventional tests and `*.config.*` files.

| Application | Framework | Files | Claimed by documented names | Left unclaimed |
| --- | --- | ---: | ---: | --- |
| [vercel/ai-chatbot](https://github.com/vercel/ai-chatbot) | Next.js | 141 | 94% | `artifacts/` 8 |
| [mfts/papermark](https://github.com/mfts/papermark) | Next.js | 1,375 | 89% | `ee/` 154, `context/` 3 |
| [dubinc/dub](https://github.com/dubinc/dub) `apps/web` | Next.js | 3,714 | 96% | `scripts/` 129, `playwright/` 23 |
| [elie222/inbox-zero](https://github.com/elie222/inbox-zero) `apps/web` | Next.js | 2,335 | 96% | `scripts/` 43, `providers/` 13, `ee/` 12, `store/` 11 |
| [shadcn-ui/ui](https://github.com/shadcn-ui/ui) `apps/v4` | Next.js | 3,661 | 10% | `examples/` 1,769, `registry/` 1,513 |
| [nextjs/saas-starter](https://github.com/nextjs/saas-starter) | Next.js | 39 | 100% | none |
| vercel/commerce | Next.js | 64 | 100% | none |
| shadcn-ui/taxonomy | Next.js | 126 | 94% | `config/` 5, `types/` 2 |
| [formbricks/formbricks](https://github.com/formbricks/formbricks) `apps/web` | Next.js | 2,644 | 35% | `modules/` 1,673, `scripts/` 28, `playwright/` 9 |
| [calcom/cal.com](https://github.com/calcom/cal.com) `apps/web` | Next.js | 920 | 43% | `modules/` 428, `playwright/` 77 |
| [elk-zone/elk](https://github.com/elk-zone/elk) | Nuxt 3 | 377 | 93% | `scripts/`, `service-worker/`, `config/`, `mocks/` |
| [nuxt/nuxt.com](https://github.com/nuxt/nuxt.com) | Nuxt 4 | 339 | 99% | `scripts/` 3 |
| nuxt-ui-templates/dashboard | Nuxt 4 | 31 | 100% | none |

The whole-package layout would have claimed every unclaimed folder above, including scripts, Playwright helpers and mocks, and also hidden folders such as `.github` and `.cursor` at an application root. The documented layout leaves feature folders such as `modules/`, `examples/` and `registry/` for a rule, which keeps the uncertain part visible.

A `spec/` test directory was also rejected: Kubernetes' [kube-openapi](https://github.com/kubernetes/kube-openapi) keeps production Go in `pkg/validation/spec/`, and GitHub code search found about 3,900 Go files declaring `package spec` under a `spec` path on 2026-10-23. RSpec's own pattern, `spec/**/*_spec.rb`, does not have this problem.

## Precision pass

On 2026-10-23, after the first draft of the proposal was found to have been validated on coverage and templates only, every proposed row was checked for false claims on real applications, as [CLAUDE.md](../../CLAUDE.md#research-and-experiments) now requires. File trees of 44 applications were read through the GitHub API, and three agents (Claude Code `sonnet` at `xhigh`) measured 23 applications of the ten web frameworks with the most GitHub stars: React, Vue, Next.js, FastAPI, Angular, Django, Gin, Svelte, Laravel and Spring Boot. Rows were emulated with rules or computed from file lists, not run as detectors.

### Changes to the proposal

| Row | False claim found | Evidence | Change |
| --- | --- | --- | --- |
| `*.config.*` as build tooling | Application code named `*.config.ts` | 29 `app.config` files in fetched trees, including Angular's generated `src/app/app.config.ts` and Nuxt's runtime `app.config.ts`; 15 plugin and policy configurations, 742 lines, in [ever-co/ever-gauzy](https://github.com/ever-co/ever-gauzy) `b3fe0424c59a`; `sentry.server.config.ts` and `sentry.edge.config.ts` in [polarsource/polar](https://github.com/polarsource/polar) `ad44441b7edf`; `auth.config`, `content.config`, `i18n.config` elsewhere | Match named tools only; test runners as tests |
| `django` | Tests counted as production | Django's `startapp` creates `tests.py`, which no convention covers; [zulip/zulip](https://github.com/zulip/zulip) has 91 outside test folders, [django/djangoproject.com](https://github.com/django/djangoproject.com) `2f3a10a91b68` 11 (3.9k lines, 7.5% of production); GitHub code search found about 678,000 `tests.py` files importing Django's `TestCase` | Add a `tests.py` test convention |
| `dotnet` | Generated EF Core metadata counted as hand-written | `*.Designer.cs` are 67% of the C# bytes of [bitwarden/server](https://github.com/bitwarden/server) and 20% of [jellyfin/jellyfin](https://github.com/jellyfin/jellyfin); [EF Core migrations](https://learn.microsoft.com/ef/core/managing-schemas/migrations/) generate a designer and a model snapshot file per migration | Add a `dotnet-generated` convention |
| `laravel` | Translations counted as production | `resources/lang` is 411,394 lines, 54% of all source, in [snipe/snipe-it](https://github.com/snipe/snipe-it) `e6c9fee06eb6`; Laravel 12 documents a root `lang/` instead | Leave `resources/lang/` out |

### Rows that held

- `next` and `nuxt` with documented names: in thirteen applications at most 1% of the claimed files look like mocks, stories or generated code, apart from shadcn/ui's documentation examples; [mfts/papermark](https://github.com/mfts/papermark) `ed19717ec02a` and [dubinc/dub](https://github.com/dubinc/dub) `880c344b0223` measured 87% and 89% production with no generated or vendored claims.
- `react-router`, `rails`, `gem`, `symfony`, `composer`, the `.Tests` folder convention and the RSpec file pattern: no false claims beyond the shared ones below. No `.Test` or `.Tests` folder in the fetched trees was anything but a test project.
- `manage.py` sits outside the root in several projects (`netbox/manage.py`, `src/manage.py` in paperless-ngx); rooting the module at its folder handles them. In example folders it would make examples production modules, as every detector already does for examples with a descriptor.

### Shared false claims, not specific to a row

- Generated code inside claimed layouts is the largest class: MyBatis Generator output 81.6% of production in [macrozheng/mall](https://github.com/macrozheng/mall) `dcaa93b31503`; an OpenAPI client 16% in [halo-dev/halo](https://github.com/halo-dev/halo) `6745dd2f85de`; swag, Wire and MockGen 14% in both Gin applications ([apache/answer](https://github.com/apache/answer) `6744295fa888`, [flipped-aurora/gin-vue-admin](https://github.com/flipped-aurora/gin-vue-admin) `f7f1009c98e6`); generated SDKs and clients about 30% in Polar; Django migrations 20% and a generated GraphQL schema in [saleor/saleor](https://github.com/saleor/saleor) `ec535e23e86b`. Go's official marker, `^// Code generated .* DO NOT EDIT\.$`, matches Wire and MockGen but not swag; MyBatis output has no marker.
- Vendored libraries inside source folders: `fontawesome-all.js` (820 KiB) in Pixelfed's `resources/assets/js/lib/`, a vendored gem with its own `.gemspec` in a Discourse plugin, `swagger-ui-bundle.js` in open-webui.
- Storybook stories and test helpers under `src/`: 37,314 lines of stories and 19,117 lines of `spec/` helpers in [bitwarden/clients](https://github.com/bitwarden/clients) `f9e3fc5c2a19`, 10,874 lines of stories in [element-hq/element-web](https://github.com/element-hq/element-web) `314f3662ca74`.
- Large data tables written as code: 121,600 lines of `un_locode`, IATA and ISO tables in NetBox.

### Other observations

- Go's `./...` pattern excludes `vendor/`, `testdata/` and folders starting with `_` or `.` ([`go help packages`](https://pkg.go.dev/cmd/go#hdr-Package_lists_and_patterns)); Locmetry's Go layout claims them.
- Language misrecognitions: compiled `.mo` translation catalogs as Modelica (NetBox, 11.2k lines), Firebase `*.rules` as Snakemake, `requirements.in` as Autoconf, Helm and code-generator `.tpl` templates as Pan in both Gin applications.
- Tracked files under an ignored path are not measured: Firefly III's 17 `resources/lang/en_US` files, because `resources/lang/.gitignore` contains `*`.
- Frameworks with `src/` are claimed but not finished: 3 of 8 React, Vue, Svelte and Angular applications were 42–53% unclaimed, mostly backends in `server/` or `backend/`, and the claimed part held 0.3–15% stories, mocks, helpers or generated code.

## Conclusions

- Frameworks whose source root is the package root or `app/`, and ecosystems without a detector, need the proposed rows to be measured usefully without configuration.
- Frameworks that keep code in `src/` have it claimed, but real applications also carry backends outside `src/` and non-production code inside it.
- Three ecosystems in the top 25 have no detector at all: .NET, PHP and Ruby. Django projects without packaging metadata miss the Python detector.
- Root tool configuration files such as `vite.config.ts` and `eslint.config.mjs` stay unclaimed in every JavaScript project; a convention must match tool names, because application code uses the `*.config.*` suffix too.
- Generated code in claimed layouts is the largest source of silent false claims across all ecosystems measured.
- The npm layout's `src/**` claims files of any language below a `package.json`. In eShop that made C# production by accident; no rule depends on it.

## Detector run

On 2026-10-23 the implemented detectors and conventions (commit `7d95a41`, release build, `--no-user-config`) measured the emulated projects and the precision-pass applications at the commits above; the shallow clones were removed afterwards. "Unclaimed" is the share of source code lines.

| Project | Unclaimed | Remaining unclaimed | Against the record |
| --- | ---: | --- | --- |
| vercel/commerce | 0% | none | as recorded |
| shadcn-ui/taxonomy | 8% | `config/`, `types/`, root `contentlayer.config.js` and `env.mjs`, Prisma SQL migrations | the record counted neither root files nor SQL |
| Turborepo `examples/basic` | 10% | `packages/eslint-config` | as recorded |
| nuxt-ui-templates/dashboard | 0% | none | as recorded |
| React Router `default` | 9% | the root `Dockerfile` | as recorded |
| jasontaylordev/CleanArchitecture | 5% | `build/`, item templates, root files of the two client apps | client-app root files beyond the record |
| laravel/laravel | 5% | `database/factories`, `database/seeders`, `public/` | as recorded |
| symfony/demo | 3% | root files and `public/` | as recorded |
| maybe-finance/maybe | 3% | `db/`, `public/`, `bin/`, vendored `vendor/javascript`, root files | as recorded, plus small folders |
| wagtail/bakerydemo | 1% | root files and `vagrant/` | as recorded |
| [dubinc/dub](https://github.com/dubinc/dub) `880c344b0223` | 3% | `apps/web/scripts` 129 files and `apps/web/playwright` 23 files | as recorded |
| [dotnet/eShop](https://github.com/dotnet/eShop) `913acea66708` | 26% | `wwwroot/` assets, 18 Blazor CSS isolation files (`*.razor.css`, 2,510 lines) and 25 MAUI XAML files (2,015 lines) | the accidental npm claim is gone; see below |

The false claims the precision pass found are absent: Snipe-IT's `resources/lang` stays unclaimed; Polar's `sentry.*.config.ts` stay unclaimed; ever-gauzy's plugin configurations are module source or unclaimed, while its 96 `jest.config.ts` files are tests; djangoproject.com's 3,893 lines of `tests.py` are tests; and eShop's 2,283 lines of EF Core designer and snapshot files are `production:generated`. No new false claim was found. One example became production as the record predicted for any example with a descriptor: Polar's Nuxt adapter playground, 38 lines.

Three gaps leave code unclaimed, which reports show for review:

- **.NET UI files:** Blazor CSS isolation files and MAUI or WPF XAML sit beside the compiled sources and are part of the application, but the `dotnet` layout claims only compile items and Razor files.
- **A Django app named `docs`:** djangoproject.com's documentation app, 3,968 lines, is excluded by the flat layout's list of non-package names, which `django` shares with `python`.
- **Root files of Next.js applications** that are neither documented conventions nor named tools, such as `env.mjs`.

On the same day the owner decided to close the first gap before the `dotnet` layout ships: it also claims `*.xaml` and `*.razor.css`, after Microsoft's [CSS isolation](https://learn.microsoft.com/aspnet/core/blazor/components/css-isolation) and [XAML compilation](https://learn.microsoft.com/dotnet/maui/xaml/xamlc) documentation, read directly. No counterexample was found: WPF, Workflow Foundation and Avalonia use XAML inside .NET projects as built UI or workflow definitions, and UiPath's XAML workflows sit in projects without a `.csproj`.
