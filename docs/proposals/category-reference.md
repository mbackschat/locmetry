# Category Reference

<!-- doc-status
{
  "status": "draft",
  "date": "2026-10-24",
  "summary": "A non-binding reference of category names, a decision order and naming conventions, embedded as a guide and enforced on Locmetry's own documents and policies.",
  "evidence": ""
}
-->

The owner raised the idea on 2026-10-24 after policies for more than 12 projects, and chose its shape the same day: an embedded guide, vocabulary plus conventions, a test guard, and aligned model policies. The [category vocabulary record](../research/category-vocabulary.md) holds the evidence. An independent review of the first draft on 2026-10-24 found overlapping names, an incomplete decision order and a guard that declarations could defeat; this revision addresses its findings.

## Problem

Categories are free-form, so every author and every agent invents names. 24 surveyed policy files assign about 70 distinct `purpose:category` pairs, most of them once, with synonyms for the same role (`benchmark`, `performance-test`), names that repeat the purpose or content kind (`tooling-test`, `deployment-source`), layer names that are just implementation (`frontend`), and placeholder categories that hide undecided code. Reports from different projects cannot be compared by category, and an agent asked twice for the same policy may name a category differently each time. Locmetry's own documents drift too: PRODUCT's example declares a catch-all placeholder, and walkthroughs use names no reference defines.

## Design

### Required: an embedded reference guide

`locmetry --guide categories` prints `docs/guides/categories.md`, embedded verbatim like the other guides; its help line reads "Reference category names, a decision order and naming conventions". The guide is non-binding: classification and validation do not change, and a project may use any name. It never requires an override to follow it: built-in decisions stand, and the guide describes them.

*Choosing a name.* Agents and authors follow this order, so the same file gets the same name:

1. Decide the purpose first, by the owner's measurement policy: is the code part of the product, a test of it, or neither? A file that exercises the product and fails when the product misbehaves is a test, whatever runs it. Generated code and examples may legitimately be production or other; the owner decides, the category does not.
2. Choose one category for the whole file. Categories apply to files: scripts inside `package.json` and cells of a notebook cannot be labeled separately.
3. Take the first name in the purpose's order below whose description fits. Each order ends with a residual name, so every file has one.

*Vocabulary.* Rows are in decision order within each purpose. Built-in names are marked; Locmetry assigns them itself. The "instead of" column lists synonyms the reference replaces.

| Purpose | Name | Use for | Instead of |
| --- | --- | --- | --- |
| production | `generated` (built-in) | Committed generator output that the owner counts as product | `generated-implementation`, `generated-source` |
| production | `native` | The platform shell of a cross-platform app: iOS and Android runners and platform channels beside a Flutter or React Native code base. A fully native app's code is `implementation` | |
| production | `deployment` | Installing, provisioning and running the product in an environment: container images, infrastructure definitions, installers, start scripts. A script that builds, checks and ships the product is `deployment` | `deployment-source`, `infrastructure` |
| production | `distribution` | Publishing the product to package managers and release channels: formulas, manifests, notice generation | `packaging`, `plugin-distribution` |
| production | `build-tooling` (built-in) | Build scripts and the configuration of compilers, bundlers and linters | |
| production | `verification-tooling` | Programs that run, prepare or score tests, and gates that check the repository's source rather than the product's behavior, such as size, format or index checks | `validation-tooling`, `runner` |
| production | `documentation-site` | Code and configuration that exist only to build the documentation site | `api-docs`, `site-config`, `docs` |
| production | `example` | Maintained examples, samples and demos outside test files | `sample`, `demo`, `playground`, `example-support` |
| production | `implementation` (built-in) | The product's hand-written code, whatever its layer: UI, API, server, database migrations, schemas and interface definitions, templates, assets in source form | `frontend`, `core`, `library-source`, `migration`, `web-asset` |
| production | `tooling` | Everything else for working on the project: maintenance and code-generation scripts, CI orchestration, development containers | `repository-tooling`, `ci-tooling`, `developer-tooling` |
| test | `generated` | Generated code that exists only for tests | |
| test | `test-support` | Test code without test cases: helpers, executable fixtures, factories, stand-in services, mocks, test data in source form | `fixture`, `test-data`, `test-infrastructure`, `mock` |
| test | `fuzz` | Fuzz targets | |
| test | `performance` | Benchmarks and load tests | `benchmark`, `performance-test`, `load-test` |
| test | `end-to-end` | A complete intended workflow through the user-facing interfaces of the whole system with its real services | `acceptance` |
| test | `integration` | Several components together, or the built artifact through its interface, short of the whole system: a CLI run as a subprocess, a server started locally with stubbed dependencies | `consumer-test` |
| test | `unit` | One unit in isolation, in process, without external services or subprocesses | |
| test | `test` (built-in) | Tests whose kind is not decided; built-in test conventions assign it, including to test-runner configuration | |
| other | `generated` | Generated code that the owner excludes from product totals | |
| other | `vendored` | Third-party code kept in the tree and measured; prefer a scope exclusion | |
| other | `third-party-wrapper` (built-in) | Build-tool wrappers such as `gradlew` | |
| other | `archived` | Retired code, historical demonstrations and frozen copies kept for reference | `archived-evidence`, `frozen-snapshot`, `historical` |
| other | `prototype` | Experiments nobody maintains yet | `research`, `incubator` |
| other | `example` | Examples and snippets nobody maintains | `doc-snippet`, `tutorial-example` |
| other | `unclaimed` (built-in) | Code no stage decided; never assign it | `unmapped-source` |

The order settles the cases a single description cannot: a load test against the deployed system is `performance`, not `end-to-end`; a fuzz target is `fuzz` whatever it exercises; a CI script that builds an image, smoke-tests it and publishes it is `deployment`. Framework and tool names do not decide a category: a Playwright suite against a locally started server with stubbed services is `integration`.

*Naming rules for project-specific names.* Use one only when no reference name fits, such as proofs of properties of the implementation in a Lean project. A category names the role code plays relative to the product. It is lowercase kebab-case. It does not repeat the purpose or the content kind (no `-test`, `-source`, `-code`, `-data`), names no location, layer, framework, language or project, and is never a listed synonym. `native` is the reference's one platform name, because it names the role of a platform shell beside a cross-platform code base. Declare the name in `[categories]` with its meaning; the same name may appear under several purposes when it means the same kind of code.

*Policy conventions.* These are style conventions for reviewed policies; the [product contract](../PRODUCT.md) owns the semantics they rely on, and the [configure guide](../guides/configure.md) the workflow:

- Each entry uses the weakest stage that expresses its decision: a rule to claim code or decide its purpose, a refinement to change only a category, an override only to change the purpose a built-in convention gives.
- Entries restrict `kind` to `source` unless the decision concerns data or documentation.
- Every entry records checkable evidence as `reason`; comments say why an entry uses its stage or an exception.
- Globs name known files and folders, so new code surfaces as unclaimed instead of being swept in, and `except` states an exception instead of relying on entry order.
- Undecided code stays unclaimed and becomes an open question; a placeholder category hides it from the review list.
- A reviewed policy leaves no unexplained item in `summary.policy_entries`: each unused entry, pattern or exception, and each `preempted` and `displaced` item, is intended, such as an exception kept for a second release tree.

### Required: one owner per rule

The reference owns names, the decision order, naming rules and the style conventions above. PRODUCT and the schema keep stage semantics, refinement limits and the audit fields; the configure guide keeps the workflow. In the configure guide, the tooling boundary of "Test Support or Verification Tooling" moves to the reference and the section keeps its Test/prod rationale with a link. The schema's `category` and `[categories]` descriptions point to the reference.

The [README](../../README.md) introduces the reference where readers meet categories: its Guides and reference table gains a row "Name categories and follow policy conventions" for `locmetry --guide categories` with a link to the repository copy, the Category term points to it, and the Configuration section recommends reference names before declaring custom categories. PRODUCT's guide enumeration lists the new topic.

### Required: a guard over Locmetry's own documents and policies

Two separate checks:

- *Validity.* Every complete policy found below loads through the policy parser, as the existing guide and schema tests do for their blocks.
- *Conformity.* An integration test reads the vocabulary table from the embedded guide. Each `category` value must be a reference name for its purpose, or appear in the guard's reviewed exception list, which records source, `purpose:category` and the reason no reference name fits. Declaring a name in `[categories]` does not waive the check. The exception list may not contain a listed synonym or `unclaimed`.

The guard parses TOML; it never searches raw text, so reasons and captured output cannot trigger it. Its sources, each of which must yield at least one policy so that a lost extraction fails:

| Form | Where |
| --- | --- |
| TOML fences | Embedded guides, README, PRODUCT, schema examples |
| `<<'TOML'` heredocs inside shell fences | Recorded walkthroughs under `docs/cli` and `docs/showcase`, guides |
| TOML in rendered help | `locmetry --help` |
| Policy files | This repository's `.locmetryrc`, the Immich overlay, the demonstration fixture's policies |

Research records and archived documents are frozen evidence and stay outside the guard. Negative fixtures show that it rejects an undeclared unlisted name, a declared synonym, a reference name under the wrong purpose, and a violation inside a walkthrough heredoc.

### Required: aligned model policies and documents

Audited file by file against the decision order, at Immich v3.2.0 and v3.3.4 and on this repository:

- Immich overlay: the auth-server stand-in becomes `test-support` and the load test `performance`; `.devcontainer/**` moves to a `tooling` rule and into the `except` list of the `deployment` rule, whose `**/Dockerfile` would otherwise match it; `docs/docusaurus.config.js` and `docs/sidebars.js` become `documentation-site`; `packages/*/esbuild.js`, which call `esbuild.build`, become `build-tooling`. The end-to-end refinement splits: specs under `e2e/src/specs/**` and `e2e/src/ui/specs/**` stay `end-to-end`, while `e2e/src/*.ts`, `e2e/src/ui/mock-network/**` and `e2e/src/ui/generators/**` become `test-support`. Category descriptions take the reference meanings.
- This repository: `scripts/test_*.py` become `integration`, because they run their script as a subprocess or against a real Git repository, except `scripts/test_agent_regression_trend.py`, which imports its module and is `unit`. `documentation-check` becomes `integration`. The `unit` description stops saying Rust.
- Documents: the configure guide's `historical` and PRODUCT's `evidence` become `archived`; PRODUCT's `unmapped-source` catch-all is removed, because it contradicts README's rule that entries without evidence belong in the review list as unclaimed source; the explain walkthrough's `order-rules` becomes a reference name. Any further name the guard rejects is aligned or added to the exception list with its reason.

### Excluded

- Accepting reference names without a declaration in `[categories]`, and recording their reference meaning in reports. It changes validation and the effective-policy snapshot; reconsider when the vocabulary has held unchanged for a release and a contract defines which meaning wins when a project declares a reference name differently. On acceptance it moves to [DEFERRED](../DEFERRED.md) with that condition.
- Changing built-in decisions, such as the `test:test` of test-runner configuration; that is a separate decision.
- Framework profiles. Framework layers are implementation; framework notes join the guide only with measured evidence.
- Hints in reports when a policy uses a listed synonym, until the vocabulary has settled.
- Entry ID conventions; IDs stay project-local.

## Feasibility

The guide is one Markdown file, one `Guide` variant, one help line and one README table row; [guides.rs](../../src/guides.rs) already embeds the others the same way. The guard needs a heredoc extractor and a rendered-help extractor beside the fence extraction the `guides::` tests already use. Alignment changes categories and, where entries split, rule provenance; selection, kinds, purposes and module credit stay. The guide's length adds reading cost for agents only when they request it.

## Acceptance

- `locmetry --guide categories` prints the guide; `--help` and PRODUCT's guide enumeration list the topic.
- The guard passes on the repository, every listed source yields a policy, and each negative fixture fails it.
- On both pinned Immich tags and on this repository (old and new policy on the same tree), file-level reports compared with `compare-locmetry-files.sh` show no change of selection, kind, purpose or module; only categories and the named rule provenance differ. Recordings, README excerpts and the demo GIF are regenerated where their category rows change.
- The README's guide table, Category term and Configuration section link to the reference; the configure guide and schema descriptions link to it instead of restating its rules; the changelog announces the guide.
