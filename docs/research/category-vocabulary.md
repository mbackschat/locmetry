# Category Vocabulary

Measured 2026-10-24 with the development build after Locmetry 0.10.0 (`--no-user-config`, no project policy) on shallow clones, removed afterwards. Question: which categories do real policies need beyond the built-in ones, and does one shared vocabulary cover them without ambiguity? The [category reference proposal](../proposals/category-reference.md) uses these results.

## Policies written so far

24 policy files were surveyed: the [policy test drives](policy-test-drives/README.md#records) of public GitHub projects, the [Immich overlay](../../examples/showcase/immich/overlay/.locmetryrc), this repository's [.locmetryrc](../../.locmetryrc), the [demonstration fixture](../../examples/cli-workspace/README.md), and private projects whose policies stay in the Git-ignored local folder. They use about 75 distinct `purpose:category` names, and only a handful recur across projects: `tooling`, `implementation`, `example`, `test-support`, `build-tooling` and `test`.

The rest drift in four recognizable ways:

- Synonyms for one role: `performance-test`, `benchmark` and `benchmark-test`; `test-support`, `test-infrastructure`, `fixture` and `test-data`; `tooling`, `repository-tooling`, `ci-tooling` and `validation-tooling`; `example`, `example-support`, `playground` and `tutorial-example`; `generated`, `generated-implementation` and `generated-source`.
- Names that encode something other than the role: a purpose (`benchmark-test`, `tooling-test`), a content kind (`deployment-source`, `configuration-data`) or a location or layer (`frontend`, `library-source`, `web-asset`, `migration`), all of which are the product's implementation.
- Placeholders that park undecided code under purpose `other` (`unmapped-source`, `fixture-review`, `configuration-or-data`), which hides it from the unclaimed review list instead of reporting it.
- Domain-specific names with no general equivalent, such as machine-checked proofs in a Lean project.

Mapped onto the draft vocabulary of the proposal, most uses match a reference name directly or as a listed synonym. The remainder are the layer names, which map to `implementation`, the placeholders, which a convention forbids, and the domain-specific names, which the proposal keeps as declared project categories.

## Fresh projects

Four projects not measured before were measured without configuration to see which decisions a policy would need, and whether exactly one reference name fits each.

| Project | Commit | Decisions a policy needs | Reference categories that fit |
| --- | --- | --- | --- |
| [documenso/documenso](https://github.com/documenso/documenso), Next.js monorepo | `8707e33c124e` | Packages without a `src/` layout, a Playwright package `packages/app-tests`, `docker/`, `.devcontainer/`, `scripts/`, shared configuration packages | `implementation`, `end-to-end`, `deployment`, `tooling`, `build-tooling` |
| [fastapi/fastapi](https://github.com/fastapi/fastapi) | `7a0a3f0b286c` | `docs_src/`, about 6,000 code lines of examples the documentation embeds and the tests import, the documentation site's code under `docs/`, `scripts/` | `example`, `documentation-site`, `tooling` |
| [BurntSushi/ripgrep](https://github.com/BurntSushi/ripgrep) | `6892bc2b7efe` | The binary crate under `crates/core` that no `Cargo.toml` places, `benchsuite/` and `crates/*/benches`, `fuzz/fuzz_targets`, `crates/*/examples`, `pkg/brew`, `ci/` | `implementation`, `performance`, `fuzz`, `example`, `distribution`, `tooling` |
| [caddyserver/caddy](https://github.com/caddyserver/caddy) | `2b47a1849f86` | Source files under Go `testdata/` folders, `*_fuzz.go` files built only with the `gofuzz` tag, file-server templates | `test-support`, `fuzz`, `implementation` |

Each decision fits one name once three tie-breaks are stated: development containers and CI scripts are `tooling`, not `deployment`, because they serve the people who work on the project rather than running the product; code that exists only to build the documentation site is `documentation-site`, not `tooling` or `build-tooling`; and benchmark definitions are test `performance`, while a program that only runs or scores benchmarks is production `verification-tooling`, as the schema already says of test runners.

Next.js projects need no category of their own. The [framework layouts record](framework-layouts.md) found their code in `app/`, `components/` and `lib/`, which is `implementation`; route handlers, UI and middleware are layers of it, which folders and workspace members already separate.

## Limits

The vocabulary is a naming aid, not a classifier, so these measurements test coverage and ambiguity, not classification accuracy. Few surveyed projects label test kinds at all, so the test-kind names rest on the primary documentation of build tools and test frameworks, read on 2026-10-24, rather than on recurring policy evidence:

- Cargo's [target reference](https://doc.rust-lang.org/cargo/reference/cargo-targets.html) calls files under `tests/` "integration tests", files under `benches/` benchmarks, and files under `examples/` "example uses of the functionality provided by the library".
- Go's [testing package](https://pkg.go.dev/testing) names benchmarks, fuzzing ("a testing technique") and examples as kinds of test code.
- Maven's [Failsafe plugin](https://maven.apache.org/surefire/maven-failsafe-plugin/) "is designed to run integration tests while the Surefire Plugin is designed to run unit tests".
- [Playwright](https://playwright.dev/docs/intro) describes Playwright Test as "an end-to-end test framework for modern web apps".
- [Locust](https://docs.locust.io/en/stable/what-is-locust.html) is "an open source performance/load testing tool", which is why the reference uses `performance` for benchmarks and load tests together.
