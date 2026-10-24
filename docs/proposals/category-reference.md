# Category Reference

<!-- doc-status
{
  "status": "draft",
  "date": "2026-10-24",
  "summary": "A non-binding reference of category names, an order from narrow roles to broad ones, worked cases and naming conventions, embedded as a guide and enforced on Locmetry's own documents and policies.",
  "evidence": ""
}
-->

The owner raised the idea on 2026-10-24 after policies for more than 12 projects, and chose its shape the same day: an embedded guide, vocabulary plus conventions, a test guard, and aligned model policies. The [category vocabulary record](../research/category-vocabulary.md) holds the evidence. Two independent reviews on 2026-10-24 found that names defined by overlapping activities still let broad rows swallow narrow cases, that the reference competed with built-in labels and with the purpose definitions, and that the guard and the alignment were incomplete. This revision changes the mechanism: the reference only chooses a category inside a purpose already decided, orders names from narrow roles to broad activities with explicit exclusions, and checks itself against worked cases.

## Problem

Categories are free-form, so every author and every agent invents names. 24 surveyed policy files assign about 70 distinct `purpose:category` pairs, most of them once, with synonyms for the same role (`benchmark`, `performance-test`), names that repeat the purpose or content kind (`tooling-test`, `deployment-source`), layer names that are just implementation (`frontend`), and placeholder categories that hide undecided code. Reports from different projects cannot be compared by category, and an agent asked twice for the same policy may name a category differently each time. Locmetry's own documents drift too: PRODUCT's example declares a catch-all placeholder, and walkthroughs use names no reference defines.

## Design

### Required: an embedded reference guide

`locmetry --guide categories` prints `docs/guides/categories.md`, embedded verbatim like the other guides; its help line reads "Reference category names, worked cases and naming conventions". The guide is non-binding: classification and validation do not change, and a project may use any name.

*Scope.* The reference chooses a category; it does not decide purposes. The [purpose definitions](../PRODUCT.md) and the schema's `Purpose` descriptions own the boundary between production, test and other, and a project's built-ins and policy apply it. Adopting the reference never requires changing a purpose.

*Built-in labels are accepted defaults.* Locmetry assigns `implementation` to module source, `build-tooling` to build conventions, `test` to test conventions, `generated` to marked generator output, `third-party-wrapper` to build wrappers, and `unclaimed` to fallback code under the fallback purpose. A policy may keep any of them. Refining one is optional and uses a name of the same purpose. When the best-fitting description belongs to another purpose, such as a test runner a convention keeps under `tests/`, the file takes the closest name of its decided purpose (`test-support`); changing its purpose is a separate policy decision with its own evidence.

*Choosing a name.* Choose one category per file; scripts inside `package.json` and cells of a notebook cannot be labeled separately. Take the first row of the file's purpose whose description fits, read with its exclusions. Rows run from narrow roles to broad activities, so a specific role is never swallowed by a general one. When no row fits, use a project-specific name.

| Purpose | Name | Use for | Not for | Instead of |
| --- | --- | --- | --- | --- |
| production | `generated` (built-in) | Committed generator output that the owner counts as product | | `generated-implementation`, `generated-source` |
| production | `native` | The platform shell around a cross-platform or web code base: iOS and Android runners of a Flutter or React Native app, the main process of an Electron or Tauri app | A fully native app's code, which is `implementation` | |
| production | `example` | Code whose job is to show how to use the product: examples, samples, demos | Examples in test files | `sample`, `demo`, `playground`, `example-support` |
| production | `documentation-site` | Code, configuration and build scripts that exist only for the documentation site | | `api-docs`, `site-config`, `docs` |
| production | `implementation` (built-in) | The product's functionality, whatever its layer: UI, API, server, migrations, schemas and interface definitions, data the product applies, templates, assets in source form | Code that builds, checks, ships, installs or runs the product | `frontend`, `core`, `library-source`, `migration`, `web-asset` |
| production | `distribution` | Publishing releases to users through package managers and release channels: formulas, manifests, release scripts, notice generation, including the build and checks such a script runs on the way | | `packaging`, `plugin-distribution` |
| production | `deployment` | Running the product for its users: images, infrastructure, installers and start scripts of production, staging or self-hosted installations, including the build and checks such a script runs on the way | Development, CI and test environments | `deployment-source`, `infrastructure` |
| production | `build-tooling` (built-in) | Turning the product's source into artifacts: build scripts, compiler, bundler and linter configuration, code generators the build invokes | | |
| production | `verification-tooling` | Running, preparing or scoring tests and checks without being one: runners, harnesses, graders, and gates on properties of the source such as size, format or index freshness | | `validation-tooling`, `runner` |
| production | `tooling` | Serving the people who work on the project: development environments and containers, development-database seeds, CI orchestration, maintenance and hand-run code-generation scripts | | `repository-tooling`, `ci-tooling`, `developer-tooling` |
| test | `generated` | Generated code that exists only for tests, such as generated snapshot baselines | | |
| test | `test-support` | Test code without test cases: helpers, executable fixtures, factories, stand-in services, mocks, runners kept as tests, hand-written baselines and test data in source form | | `fixture`, `test-data`, `test-infrastructure`, `mock` |
| test | `fuzz` | Fuzz targets | | |
| test | `performance` | Benchmark and load workloads, with or without threshold assertions | | `benchmark`, `performance-test`, `load-test` |
| test | `end-to-end` | Workflows through the user-facing interfaces of the whole system with its real services | Suites that stub or mock any of the system's own services | `acceptance` |
| test | `integration` | Several components together, or the built artifact through its interface, short of the whole system: a CLI run as a subprocess, a browser against mocked API responses, contract tests | | `consumer-test` |
| test | `unit` | Units in process; temporary files and in-process mocks are fine | Tests that start processes or use network or external services | |
| test | `test` (built-in) | Tests whose kind is not decided | | |
| other | `generated` | Generated code that the owner excludes from product totals | | |
| other | `vendored` | Third-party code kept in the tree and measured; prefer a scope exclusion | | |
| other | `archived` | Retired code, historical demonstrations and frozen copies kept for reference | | `archived-evidence`, `frozen-snapshot`, `historical` |
| other | `prototype` | Experiments nobody maintains yet | | `research`, `incubator` |
| other | `example` | Examples and snippets nobody maintains | | `doc-snippet`, `tutorial-example` |

`third-party-wrapper` and `unclaimed` are labels Locmetry assigns, not names to assign; `unclaimed` replaces placeholders such as `unmapped-source`. A test file whose cases span several scopes takes the widest of them.

*Worked cases.* The guide lists cases that the order settles, so readers and agents can check their reading:

| Case | Category |
| --- | --- |
| CI script that only runs the test suites | production `verification-tooling` |
| Code generator the build invokes / one run by hand | production `build-tooling` / `tooling` |
| Script that builds an image, smoke-tests it and pushes it to a production registry | production `deployment` |
| Release script that checks, tags and publishes | production `distribution` |
| Development container, development-database seed | production `tooling` |
| Terraform for production / for CI workers | production `deployment` / `tooling` |
| Build script of the documentation site | production `documentation-site` |
| Mock server shipped for API consumers | production `implementation` |
| Storybook stories without assertions / Rust `examples/` | production `example` |
| Main process of an Electron app around a web UI | production `native` |
| pytest `conftest.py` holding only fixtures | test `test` by convention, refinable to `test-support` |
| Go `example_test.go` whose examples check output | test `test` by convention, refinable to `unit` |
| Pact consumer test, Playwright suite against mocked API responses | test `integration` |
| Benchmark asserting a threshold / generic program that runs benchmarks | test `performance` / production `verification-tooling` |

*Naming rules for project-specific names.* Use one only when no row fits, such as proofs of properties of the implementation in a Lean project. It names the role code plays relative to the product, is lowercase kebab-case, does not end in a purpose or content kind (`-test`, `-source`, `-code`, `-data`), names no location, layer, framework, language or project, is not a listed synonym, and is declared in `[categories]` with its meaning. `native` is the reference's one platform name, because it names the role of a platform shell. The same name may appear under several purposes when it means the same kind of code.

*Policy conventions.* Style conventions for reviewed policies; PRODUCT owns the semantics they rely on and the [configure guide](../guides/configure.md) the workflow:

- Each entry uses the weakest stage that expresses its decision: a rule to claim code or decide its purpose, a refinement to change only a category, an override only to change the purpose a built-in convention gives.
- Entries restrict `kind` to `source` unless the decision concerns data or documentation.
- Every entry records checkable evidence as `reason`; comments say why an entry uses its stage or an exception.
- Globs name known files and folders, so new code surfaces as unclaimed instead of being swept in, and `except` states an exception instead of relying on entry order.
- Undecided code stays unclaimed and becomes an open question; a placeholder category hides it from the review list.
- A reviewed policy leaves no unexplained item in `summary.policy_entries`: each unused entry, pattern or exception, and each `preempted` and `displaced` item, is intended, such as an exception kept for a second release tree.

### Required: one owner per rule

The reference owns names, their order and exclusions, worked cases, naming rules and the style conventions above. PRODUCT and the schema keep purpose definitions, stage semantics, refinement limits and audit fields; the configure guide keeps the workflow.

In configure's "Test Support or Verification Tooling" section, the sentence listing what to classify as production with "a category such as `tooling`" is replaced by a link to the schema's purpose definitions and to the reference, which also removes its conflict with the built-in `test` label of test-runner configuration. The sentences "A program with its own tests is code under test" and the tests-of-tests inference are removed, because having tests does not decide a purpose. The ratio explanation stays. The schema's `category` and `[categories]` descriptions point to the reference.

The [README](../../README.md) introduces the reference where readers meet categories: its Guides and reference table gains a row "Name categories and follow policy conventions" for `locmetry --guide categories` with a link to the repository copy, the Category term points to it, and the Configuration section recommends reference names before declaring custom categories. PRODUCT's guide enumeration, the [guides README](../guides/README.md) and the guide-topic loop in the [root walkthrough](../cli/root.md) list the new topic.

### Required: guards over Locmetry's own documents and policies

*Example inventory.* An integration test extracts every TOML example by form and checks it against a checked-in inventory that names each example's source, position, kind (project policy, workspace definition or deliberately invalid) and expected validity. An example that disappears, moves to another kind or appears without an inventory entry fails the test, as does a source that the inventory marks as having no examples but gains one. Policies load through the policy parser, workspace definitions through the workspace parser, and deliberately invalid examples must be rejected. A mutation test removes one of two examples from a source and expects a failure.

| Form | Where |
| --- | --- |
| TOML fences | Embedded guides, README, PRODUCT, schema examples |
| `<<'TOML'` heredocs inside shell fences | Recorded walkthroughs under `docs/cli` and `docs/showcase`, guides |
| TOML in rendered help | `locmetry --help` |
| Policy files | This repository's `.locmetryrc`, the Immich overlay, the demonstration fixture's policies |

*Conformity.* The test reads the vocabulary table from the embedded guide. Each `category` value of a valid policy must be an assignable reference name for its purpose, or match an entry of the reviewed exception list. An exception names one source, one entry ID and its `purpose:category`, records why no row fits, must satisfy the naming rules mechanically (kebab-case, no forbidden suffix, not a synonym) and must be declared in that policy's `[categories]`. Explicit `unclaimed` and `third-party-wrapper` fail. Unused exceptions fail, so the list cannot accumulate.

*Category selectors.* Each `--table-category` pattern in a guide, README or recorded walkthrough must match at least one assignable reference name or exception for its purpose, so a selector cannot silently stop selecting after a rename.

Research records and archived documents are frozen evidence and stay outside the guards. Negative fixtures show that conformity rejects an undeclared unlisted name, a declared synonym, a reference name under the wrong purpose, an explicit `unclaimed`, and a violation inside a walkthrough heredoc.

### Required: aligned model policies and documents

Audited file by file at Immich v3.2.0 and v3.3.4 and on this repository; these are the expected categories after alignment:

- Immich, end-to-end tree: `e2e/src/specs/**` stays `end-to-end`, because those suites drive the running server and web client without mocks. `e2e/src/ui/specs/**/*.e2e-spec.ts` become `integration`, because all of them mock API responses through `e2e/src/ui/mock-network`. `e2e/src/*.ts`, `e2e/src/ui/mock-network/**`, `e2e/src/ui/generators/**` and `e2e/src/ui/specs/**/utils.ts` become `test-support`. On both pins these sets partition the 72 source files of `e2e/src`.
- Immich, other entries: the auth-server stand-in becomes `test-support` and the load test `performance`. `.devcontainer/**` moves from the `deployment` rule to a `tooling` rule. `docs/docusaurus.config.js` and `docs/sidebars.js` become `documentation-site`. `packages/*/esbuild.js`, `web/lint-env.js` and `mobile/ios/scripts/xcode_flutter_build.sh` become `build-tooling`, because they configure the bundler or linter or run the Flutter build. Category descriptions take the reference meanings.
- This repository: `scripts/test_*.py` become `integration`, because they run their script or Git as a process, except `scripts/test_agent_regression_trend.py` and `scripts/test_score_full_help.py`, which run in process with mocks and become `unit`. `tests/integration/common.rs`, which holds helpers without tests, becomes `test-support`. `documentation-check` becomes `integration`. `scripts/release.py`, which checks, commits, tags and pushes a release, becomes `distribution`. The `unit` description stops saying Rust.
- Documents: the configure guide's `historical` and PRODUCT's `evidence` become `archived`; PRODUCT's `unmapped-source` catch-all is removed, because it contradicts README's rule that entries without evidence belong in the review list as unclaimed source; the explain walkthrough's `order-rules` becomes a reference name. The [self-measurement record](../self-measurement.md), the [Immich showcase README](../../examples/showcase/immich/README.md) and the showcase's `test:*infra*` selector in [measure.md](../showcase/measure.md) follow the new names, and the showcase prose keeps describing what its selections show. Any further name the guards reject is aligned or added to the exception list with its reason.

### Excluded

- Accepting reference names without a declaration in `[categories]`, and recording their reference meaning in reports. It changes validation and the effective-policy snapshot; reconsider when the vocabulary has held unchanged for a release and a contract defines which meaning wins when a project declares a reference name differently. On acceptance it moves to [DEFERRED](../DEFERRED.md) with that condition.
- Changing built-in decisions, such as the `test` label of test-runner configuration; that is a separate decision.
- Framework profiles. Framework layers are implementation; framework notes join the guide only with measured evidence.
- Hints in reports when a policy uses a listed synonym, until the vocabulary has settled.
- Entry ID conventions; IDs stay project-local.

## Feasibility

The guide is one Markdown file, one `Guide` variant, one help line and one README table row; [guides.rs](../../src/guides.rs) already embeds the others the same way. The inventory guard extends the fence extraction of the `guides::` tests and the workspace distinction of the `schema::` tests with heredoc and rendered-help extractors. A reviewer applied the policy changes of the previous draft in memory on both Immich pins and this repository on 2026-10-24: file identities, kinds, purposes, modules, languages and counts stayed unchanged. Alignment changes categories and, where entries split or move, rule and refinement provenance. The guide's length adds reading cost for agents only when they request it.

## Acceptance

- `locmetry --guide categories` prints the guide; `--help`, PRODUCT's guide enumeration, the guides README and the root walkthrough's topic loop list it.
- The guards pass on the repository; each negative fixture and the mutation test fail them.
- On both pinned Immich tags and on this repository (old and new policy on the same tree), file-level reports compared with `compare-locmetry-files.sh` show no change of selection, kind, purpose, module or counts; categories change exactly as listed under alignment, and only the named entries change provenance.
- Recordings, README excerpts and the demo GIF are regenerated where their category rows change, and the selector guard passes on them.
- The README's guide table, Category term and Configuration section link to the reference; the configure guide and schema descriptions link to it instead of restating its rules; the changelog's Unreleased section announces the guide.
