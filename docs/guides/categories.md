# Choose Category Names

Use this non-binding reference to name new categories of a `.locmetryrc` policy, so that projects, people and agents name the same kind of code alike. Locmetry accepts any declared name. Renaming a category a policy already declares is the owner's decision: propose the reference name instead of applying it.

## What the Reference Decides

It chooses a category inside a purpose that is already decided; `$defs.Purpose` in `locmetry --schema config`, the built-ins and the policy decide purposes. Following it never requires changing a purpose. Built-in labels are accepted defaults: `implementation` (module source), `build-tooling` (build conventions), `test` (test conventions), `generated` (marked generator output), `third-party-wrapper` and `unclaimed`. Refining one is optional and keeps its purpose. A file whose best fit belongs to another purpose takes the closest name of its own, such as `test-support` for a runner a test convention keeps.

## Choose a Name

Give each file one category. Take the first row of its purpose that fits, reading "Not for"; rows run from narrow roles to broad ones. Prefer a row's name over the names it replaces.

<!-- category-vocabulary -->
| Purpose | Name | Use for | Not for | Instead of |
| --- | --- | --- | --- | --- |
| production | `generated` | Generator output counted as product | | `generated-source` |
| production | `native` | Platform shell of a cross-platform or web app: iOS and Android runners, an Electron main process | A fully native app | |
| production | `example` | Examples, samples, demos | Examples in test files | `sample`, `demo` |
| production | `documentation-site` | Code and build scripts only for the documentation site | | `docs`, `site-config` |
| production | `implementation` | The product's functionality in any layer, including migrations and schemas | Code that builds, checks, ships or runs it | `frontend`, `core` |
| production | `distribution` | Publishing releases: package manifests, formulas, release scripts | | `packaging`, `release-tooling` |
| production | `deployment` | Running the product for its users: images, infrastructure, installers, start scripts | Development, CI and test environments | `infrastructure` |
| production | `build-tooling` | Building artifacts: build scripts, compiler, bundler and linter configuration, code generators the build runs | | |
| production | `verification-tooling` | Running, preparing or scoring tests and checks: runners, harnesses, graders, source gates | | `ci-tooling`, `runner` |
| production | `tooling` | Anything else for working on the project: development environments, setup and maintenance scripts | | `developer-tooling` |
| test | `generated` | Generated code only for tests, such as snapshot baselines | | |
| test | `test-support` | Test code without test cases: helpers, fixtures, mocks, stand-in services | | `fixture`, `test-helpers` |
| test | `fuzz` | Fuzz targets | | |
| test | `performance` | Benchmarks and load tests | | `benchmark`, `load-test` |
| test | `end-to-end` | Workflows through the whole system with its real services | Suites that mock the system's own services | `e2e` |
| test | `integration` | Components together, or the built artifact through its interface: a CLI run as a process, a UI against mocked APIs | | `integration-tests` |
| test | `unit` | Units in process, with temporary files or in-process mocks | Tests that start processes or use the network | `unit-tests` |
| test | `test` | Tests of undecided kind | | |
| other | `generated` | Generated code excluded from product totals | | |
| other | `vendored` | Third-party code kept and measured; prefer excluding it | | |
| other | `archived` | Retired code and frozen copies | | `historical` |
| other | `prototype` | Unmaintained experiments | | `research` |
| other | `example` | Unmaintained examples and snippets | | `snippet` |

`third-party-wrapper` and `unclaimed` are assigned by Locmetry, never by a policy. Leave undecided code unclaimed, so it stays on the review list. A test file spanning several kinds takes the widest.

## Worked Cases

| Case | Category |
| --- | --- |
| Release script that checks, tags and publishes | production `distribution` |
| CI script that only runs the tests | production `verification-tooling` |
| Development container or setup script | production `tooling` |
| Code generator the build runs / one run by hand | `build-tooling` / `tooling` |
| Benchmark asserting a threshold | test `performance` |
| Browser suite against mocked API responses | test `integration` |
| pytest `conftest.py` with only fixtures | test `test` by convention, refinable to `test-support` |

## Project-Specific Names

Use one when no row fits, such as proofs in a Lean project, or when a row would merge kinds the owner wants to see apart, such as differential tests against a reference engine (`engine-oracle`) among integration tests. Name the role relative to the product in lowercase kebab-case, without a purpose or content-kind suffix (`-test`, `-source`, `-code`, `-data`), location, layer, framework, language, the project's own name or a replaced name, and declare it in `[categories]` with its meaning. The same name may serve several purposes when it means the same kind of code.

## Write a Reviewed Policy

`locmetry --guide configure` owns the workflow and `locmetry --schema config` the semantics.

- Use the weakest stage: a rule to claim code or set its purpose, a refinement to change only a category, an override only to change a convention's purpose.
- Limit entries to `kind = ["source"]` unless they concern data or documentation, and give each a checkable `reason`.
- Name known files and folders, and state exceptions with `except` rather than entry order.
- Leave no unexplained item in `summary.policy_entries`.

Locmetry's own [policy](../../.locmetryrc) and the [Immich showcase policy](../../examples/showcase/immich/overlay/.locmetryrc) follow this reference.
