# Category Vocabulary

Measured 2026-10-24 with Locmetry built from commit `8a27b3c` (`--no-user-config`, no project policy) on shallow clones, removed afterwards. Question: which categories do real policies need beyond the built-in ones, and does one shared vocabulary cover them without ambiguity? The [category reference proposal](../proposals/category-reference.md) uses these results.

## Policies written so far

24 policy files were surveyed: the [policy test drives](policy-test-drives/README.md#records) of public GitHub projects, the [Immich overlay](../../examples/showcase/immich/overlay/.locmetryrc), this repository's [.locmetryrc](../../.locmetryrc), the [demonstration fixture](../../examples/cli-workspace/README.md), and private projects whose policies stay in the Git-ignored local folder. Counted from the entries of each file, each distinct `purpose:category` pair once, they assign about 70 pairs. Several files are agent and final snapshots, or earlier versions, of one project's policy, so how many files use a name overstates how many projects do. Only a handful recur across projects: `tooling`, `implementation`, `example`, `test-support`, `build-tooling` and `test`. The per-file inventory stays in the Git-ignored local folder, because most files are private.

The rest drift in four recognizable ways:

- Synonyms for one role: `performance-test`, `benchmark` and `benchmark-test`; `test-support`, `test-infrastructure`, `fixture` and `test-data`; `tooling`, `repository-tooling`, `ci-tooling` and `validation-tooling`; `example`, `example-support`, `playground` and `tutorial-example`; `generated`, `generated-implementation` and `generated-source`.
- Names that encode something other than the role: a purpose (`benchmark-test`, `tooling-test`), a content kind (`deployment-source`, `configuration-data`) or a location or layer (`frontend`, `library-source`, `web-asset`, `migration`), all of which are the product's implementation.
- Placeholders that park undecided code under purpose `other` (`unmapped-source`, `fixture-review`, `configuration-or-data`), which hides it from the unclaimed review list instead of reporting it.
- Domain-specific names with no general equivalent, such as machine-checked proofs in a Lean project.

These labels are observations of what authors chose, not established ground truth: older records include unresolved classifications and workarounds. Mapped onto the draft vocabulary of the proposal, most uses match a reference name directly or as a listed synonym. The remainder are the layer names, which map to `implementation`, the placeholders, which a convention forbids, and the domain-specific names, which the proposal keeps as declared project categories.

## Fresh projects

Four projects not measured before were measured without configuration to see which decisions a policy would need, and whether exactly one reference name fits each.

| Project | Commit | Decisions a policy needs | Reference categories that fit |
| --- | --- | --- | --- |
| [documenso/documenso](https://github.com/documenso/documenso), Next.js monorepo | `8707e33c124e` | Packages without a `src/` layout, a Playwright package `packages/app-tests`, `docker/`, `.devcontainer/`, `scripts/`, shared configuration packages | `implementation`, `end-to-end`, `deployment`, `tooling`, `build-tooling` |
| [fastapi/fastapi](https://github.com/fastapi/fastapi) | `7a0a3f0b286c` | `docs_src/`, examples the documentation embeds and the tests import, the documentation site's code under `docs/`, `scripts/` | `example`, `documentation-site`, `tooling` |
| [BurntSushi/ripgrep](https://github.com/BurntSushi/ripgrep) | `6892bc2b7efe` | The binary crate under `crates/core` that no `Cargo.toml` places, `benchsuite/` and `crates/*/benches`, `fuzz/fuzz_targets`, `crates/*/examples`, `pkg/brew`, `ci/` | `implementation`, `performance`, `fuzz`, `example`, `distribution`, `tooling` |
| [caddyserver/caddy](https://github.com/caddyserver/caddy) | `2b47a1849f86` | Source files under Go `testdata/` folders, `*_fuzz.go` files built only with the `gofuzz` tag, file-server templates | `test-support`, `fuzz`, `implementation` |

Every inspected decision found a reference name; this short inventory shows coverage of these cases, not of every project. Descriptions alone did not make the choice unique: development containers and CI scripts fit both `tooling` and `deployment`, documentation-site configuration fits `tooling`, `build-tooling` and `documentation-site`, and benchmark directories mix workloads with programs that run them. Two independent reviews of the drafts found further overlaps, such as a CI script that builds, checks and publishes, and suites against a locally started server. The proposal therefore orders names from narrow roles to broad activities and gives the broad ones explicit exclusions. Mappings under that order, with the alternatives it rejects; rows marked provisional rest on file names and layout, not on reading the code:

| Decision | Name | Rejected alternatives |
| --- | --- | --- |
| documenso `.devcontainer/`, ripgrep `ci/` | `tooling` | `deployment`, which excludes development and CI environments |
| fastapi documentation-site code under `docs/` | `documentation-site` | `tooling`, `build-tooling`, which come later in the order |
| fastapi `docs_src/` | `example` (production) | `test-support`: the files show how to use the product, and the tests import them |
| ripgrep `crates/*/benches` | `performance` (test) | `unit`, `integration` |
| ripgrep `benchsuite/` (provisional) | `verification-tooling` for a program that runs benchmarks, `performance` for workload definitions | One name for the whole directory |
| documenso `packages/app-tests` (provisional) | `end-to-end` if the Playwright suites run against the application with its real services, otherwise `integration` | Deciding by the framework |
| caddy `*_fuzz.go` | `fuzz` | `unit`, `integration`, which come later in the order |

Next.js projects need no category of their own. The [framework layouts record](framework-layouts.md) found their code in `app/`, `components/` and `lib/`, which is `implementation`; route handlers, UI and middleware are layers of it, which folders and workspace members already separate.

## Limits

The vocabulary is a naming aid, not a classifier, so these measurements test coverage and ambiguity, not classification accuracy. Few surveyed projects label test kinds at all, so the test-kind names rest on the primary documentation of build tools and test frameworks, read on 2026-10-24, rather than on recurring policy evidence:

- Cargo's [target reference](https://doc.rust-lang.org/cargo/reference/cargo-targets.html) calls files under `tests/` "integration tests", files under `benches/` benchmarks, and files under `examples/` "example uses of the functionality provided by the library".
- Go's [testing package](https://pkg.go.dev/testing) names benchmarks, fuzzing ("a testing technique") and examples as kinds of test code.
- Maven's [Failsafe plugin](https://maven.apache.org/surefire/maven-failsafe-plugin/) "is designed to run integration tests while the Surefire Plugin is designed to run unit tests".
- [Playwright](https://playwright.dev/docs/intro) describes Playwright Test as "an end-to-end test framework for modern web apps".
- [Locust](https://docs.locust.io/en/stable/what-is-locust.html) is "an open source performance/load testing tool", which is why the reference uses `performance` for benchmarks and load tests together.
