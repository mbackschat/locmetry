# Choose Category Names

Use this guide to name the categories of a `.locmetryrc` policy: which reference name fits a file, when a project-specific name is justified, and how reviewed policies are written. The reference is non-binding: Locmetry accepts any declared name, and validation and classification do not depend on this guide. It exists so that projects, people and agents choose the same name for the same kind of code, and reports stay comparable.

## What the Reference Decides

A category is a finer label inside a purpose, written `purpose:category`. This reference chooses the category; it does not decide purposes. Whether code is production, test or other follows the purpose definitions in `locmetry --schema config` (`$defs.Purpose`) and the project's built-ins and policy. Adopting the reference never requires changing a purpose.

Built-in labels are accepted defaults. Locmetry assigns `implementation` to module source, `build-tooling` to build conventions, `test` to test conventions, `generated` to marked generator output, `third-party-wrapper` to build wrappers, and `unclaimed` to fallback code. A policy may keep any of them. Refining one is optional and uses a name of the same purpose, through a refinement as `locmetry --guide configure` shows. When the best-fitting description belongs to another purpose, such as a test runner that a test convention keeps under `tests/`, the file takes the closest name of its decided purpose, here `test-support`; changing its purpose is a separate policy decision that needs its own evidence.

## Choose a Name

Choose one category per file. Locmetry classifies whole files, so scripts inside `package.json` and cells of a notebook cannot be labeled separately.

Take the first row of the file's purpose whose description fits, read together with its exclusions. Rows run from narrow roles to broad activities, so a specific role is never swallowed by a general one. When no row fits, use a project-specific name. The last column lists names the reference replaces; prefer the reference name over them.

<!-- category-vocabulary -->
| Purpose | Name | Use for | Not for | Instead of |
| --- | --- | --- | --- | --- |
| production | `generated` | Committed generator output that the owner counts as product | | `generated-implementation`, `generated-source` |
| production | `native` | The platform shell around a cross-platform or web code base: iOS and Android runners of a Flutter or React Native app, the main process of an Electron or Tauri app | A fully native app's code, which is `implementation` | |
| production | `example` | Code whose job is to show how to use the product: examples, samples, demos | Examples in test files | `sample`, `demo`, `playground`, `example-support` |
| production | `documentation-site` | Code, configuration and build scripts that exist only for the documentation site | | `api-docs`, `site-config`, `docs` |
| production | `implementation` | The product's functionality, whatever its layer: UI, API, server, migrations, schemas and interface definitions, data the product applies, templates, assets in source form | Code that builds, checks, ships, installs or runs the product | `frontend`, `core`, `library-source`, `migration`, `web-asset` |
| production | `distribution` | Publishing releases to users through package managers and release channels: formulas, manifests, release scripts, notice generation, including the build and checks such a script runs on the way | | `packaging`, `plugin-distribution` |
| production | `deployment` | Running the product for its users: images, infrastructure, installers and start scripts of production, staging or self-hosted installations, including the build and checks such a script runs on the way | Development, CI and test environments | `deployment-source`, `infrastructure` |
| production | `build-tooling` | Turning the product's source into artifacts: build scripts, compiler, bundler and linter configuration, code generators the build invokes | | |
| production | `verification-tooling` | Running, preparing or scoring tests and checks without being one: runners, harnesses, graders, and gates on properties of the source such as size, format or index freshness | | `validation-tooling`, `runner` |
| production | `tooling` | Serving the people who work on the project: development environments and containers, development-database seeds, CI orchestration, maintenance and hand-run code-generation scripts | | `repository-tooling`, `ci-tooling`, `developer-tooling` |
| test | `generated` | Generated code that exists only for tests, such as generated snapshot baselines | | |
| test | `test-support` | Test code without test cases: helpers, executable fixtures, factories, stand-in services, mocks, runners kept as tests, hand-written baselines and test data in source form | | `fixture`, `test-data`, `test-infrastructure`, `mock` |
| test | `fuzz` | Fuzz targets | | |
| test | `performance` | Benchmark and load workloads, with or without threshold assertions | | `benchmark`, `performance-test`, `load-test` |
| test | `end-to-end` | Workflows through the user-facing interfaces of the whole system with its real services | Suites that stub or mock any of the system's own services | `acceptance` |
| test | `integration` | Several components together, or the built artifact through its interface, short of the whole system: a CLI run as a subprocess, a browser against mocked API responses, contract tests | | `consumer-test` |
| test | `unit` | Units in process; temporary files and in-process mocks are fine | Tests that start processes or use network or external services | |
| test | `test` | Tests whose kind is not decided | | |
| other | `generated` | Generated code that the owner excludes from product totals | | |
| other | `vendored` | Third-party code kept in the tree and measured; prefer a scope exclusion | | |
| other | `archived` | Retired code, historical demonstrations and frozen copies kept for reference | | `archived-evidence`, `frozen-snapshot`, `historical` |
| other | `prototype` | Experiments nobody maintains yet | | `research`, `incubator` |
| other | `example` | Examples and snippets nobody maintains | | `doc-snippet`, `tutorial-example` |

`third-party-wrapper` and `unclaimed` are labels Locmetry assigns, not names to assign. Leave undecided code unclaimed instead of inventing a placeholder such as `unmapped-source`: unclaimed code stays on the review list until someone decides it. A test file whose cases span several scopes takes the widest of them.

## Worked Cases

These cases show how the order settles a choice. Check a reading against them before naming a new kind of file:

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

## Project-Specific Names

Use a project-specific name only when no row fits, such as proofs of properties of the implementation in a Lean project. Such a name:

- names the role code plays relative to the product;
- is lowercase kebab-case;
- does not end in a purpose or content kind, such as `-test`, `-source`, `-code` or `-data`;
- names no location, layer, framework, language or project;
- is not one of the replaced names listed above;
- is declared in `[categories]` with its meaning.

`native` is the reference's one platform name, because it names the role of a platform shell rather than a platform. The same name may appear under several purposes when it means the same kind of code, as `generated` does.

## Write a Reviewed Policy

These conventions keep a policy reviewable. `locmetry --schema config` owns the semantics they rely on, and `locmetry --guide configure` the workflow for building and verifying a policy.

- Each entry uses the weakest stage that expresses its decision: a rule to claim code or decide its purpose, a refinement to change only a category, an override only to change the purpose a built-in convention gives.
- Entries restrict `kind` to `source` unless the decision concerns data or documentation.
- Every entry records checkable evidence as `reason`. TOML comments say why an entry uses its stage or an exception.
- Globs name known files and folders, so new code surfaces as unclaimed instead of being swept in. `except` states an exception instead of relying on entry order.
- Undecided code stays unclaimed and becomes an open question for the owner.
- A reviewed policy leaves no unexplained item in `summary.policy_entries`: each unused entry, pattern or exception, and each `preempted` and `displaced` item, is intended, such as an exception kept for a second release tree.

Locmetry's own [policy](../../.locmetryrc) and the [Immich showcase policy](../../examples/showcase/immich/overlay/.locmetryrc) follow these conventions and are checked against this reference.
