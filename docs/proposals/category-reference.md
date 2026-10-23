# Category Reference

<!-- doc-status
{
  "status": "draft",
  "date": "2026-10-24",
  "summary": "A non-binding reference of category names, naming rules and policy conventions, embedded as a guide and enforced on Locmetry's own documents.",
  "evidence": ""
}
-->

The owner raised the idea on 2026-10-24 after policies for more than 12 projects, and chose its shape the same day: an embedded guide, vocabulary plus policy conventions, a test guard, and aligned model policies. The [category vocabulary record](../research/category-vocabulary.md) holds the evidence.

## Problem

Categories are free-form, so every author and every agent invents names. 24 surveyed policies use about 75 names, mostly once, with synonyms for the same role (`benchmark`, `performance-test`), names that repeat the purpose or content kind (`tooling-test`, `deployment-source`), layer names that are just implementation (`frontend`), and placeholder categories that hide undecided code. Reports from different projects cannot be compared by category, and an agent asked twice for the same policy may name a category differently each time. Locmetry's own documents drift too: examples use names no reference defines.

## Design

**Required: an embedded reference guide.** `locmetry --guide categories` prints `docs/guides/categories.md`, embedded verbatim like the other guides; its help line reads "Reference category names, naming rules and policy conventions". The guide is non-binding: classification and validation do not change, and a project may use any name. It has four parts.

*Naming rules.* A category names the role code plays relative to the product. It is lowercase kebab-case. It does not repeat the purpose (`test:performance`, not `test:performance-test`) or the content kind (no `-source`, `-code`, `-data`), and names no location, layer, framework, language or project (`frontend` and `migration` are `implementation`). The same name may appear under several purposes when it means the same kind of code; whether that code counts as product is the purpose's job, as with `production:generated` and `other:generated`.

*Vocabulary.* Use a reference name when its description fits; the "instead of" column lists the synonyms it replaces. Built-in names are marked; Locmetry assigns them itself.

| Purpose | Name | Use for | Instead of |
| --- | --- | --- | --- |
| production | `implementation` (built-in) | The product's hand-written code, whatever its layer: UI, API, server, database migrations, templates and assets in source form | `frontend`, `core`, `library-source`, `migration`, `web-asset` |
| production | `generated` (built-in) | Committed generator output that the product uses | `generated-implementation`, `generated-source` |
| production | `native` | Platform code of a mobile or desktop app beside its main code base, such as iOS and Android runners | |
| production | `build-tooling` (built-in) | Build scripts and the configuration of build, bundling, lint and test tools | |
| production | `tooling` | Commands and environments for working on the project: scripts, code-generation drivers, CI scripts, development containers | `repository-tooling`, `ci-tooling`, `developer-tooling` |
| production | `verification-tooling` | Programs that run, prepare or score tests and checks: runners, harnesses, graders, CI gates, benchmark drivers | `validation-tooling`, `runner` |
| production | `deployment` | Building, shipping and running the product: container images, infrastructure definitions, installers, start scripts | `deployment-source`, `infrastructure` |
| production | `distribution` | Packaging for package managers and release channels: formulas, manifests, notice generation | `packaging`, `plugin-distribution` |
| production | `example` | Maintained examples, samples and demos, including documentation examples that tests run | `sample`, `demo`, `playground`, `example-support` |
| production | `documentation-site` | Code and configuration that exist only to build the documentation site | `api-docs`, `site-config`, `docs` |
| test | `test` (built-in) | Tests without a finer label | |
| test | `unit` | Tests of one unit in isolation, in process and without external services | |
| test | `integration` | Tests of several components together, or of the built artifact through its interface, without a deployed system | `consumer-test` |
| test | `end-to-end` | Suites that drive the running system through its user-facing interfaces, such as a browser or public API | |
| test | `performance` | Benchmarks and load tests | `benchmark`, `performance-test`, `load-test` |
| test | `fuzz` | Fuzz targets | |
| test | `test-support` | Code that exists only for tests: helpers, fixtures, test data in source form, stand-in services, mocks | `fixture`, `test-data`, `test-infrastructure`, `mock` |
| other | `unclaimed` (built-in) | Code no stage decided; never assign it | |
| other | `generated` | Generated code the project does not count as part of its product | |
| other | `vendored` | Third-party code kept in the tree and measured; prefer a scope exclusion | |
| other | `third-party-wrapper` (built-in) | Build-tool wrappers such as `gradlew` | |
| other | `example` | Examples and snippets nobody maintains | `doc-snippet`, `tutorial-example` |
| other | `prototype` | Experiments nobody maintains yet | `research`, `incubator` |
| other | `archived` | Retired code and frozen copies kept for reference | `archived-evidence`, `frozen-snapshot` |

Tie-breaks: development containers and CI scripts are `tooling`, because they serve the people who work on the project rather than running the product. Code that only builds the documentation site is `documentation-site`, not `tooling` or `build-tooling`. Benchmark definitions are test `performance`, while a program that only runs or scores them is production `verification-tooling`. A test that fits several kinds takes the widest scope it exercises: `end-to-end` over `integration` over `unit`.

*Project-specific categories.* Use one only when no reference name fits, such as machine-checked proofs in a Lean project; declare it in `[categories]` with its meaning, and never use a listed synonym.

*Policy conventions.* Each entry uses the weakest stage that expresses its decision: a rule to claim code or decide its purpose, a refinement to change only a category, an override only to change the purpose a built-in convention gives. Entries restrict `kind` to `source` unless the decision concerns data or documentation. Every entry records checkable evidence as `reason`; comments in the file say why an entry uses its stage or exception. Globs name known files and folders, so new code surfaces as unclaimed instead of being swept in, and `except` states an exception instead of relying on entry order. Undecided code stays unclaimed and becomes an open question; a placeholder category under `other` hides it. Entry IDs name the code they cover, such as `ios-app`, not their category. A reviewed policy has no unused entries, patterns or exceptions in `summary.policy_entries`, and each `preempted` and `displaced` item is intended.

The [configure guide](../guides/configure.md) keeps the workflow and links to the reference for names and conventions, so the rules have one owner. The schema's `category` and `[categories]` descriptions point to the reference.

The [README](../../README.md) introduces the reference where readers meet categories: its Guides and reference table gains a row "Name categories and follow policy conventions" for `locmetry --guide categories` with a link to the repository copy, the Category term points to it, and the Configuration section recommends reference names before declaring custom categories.

**Required: a guard over Locmetry's own documents.** An integration test reads the vocabulary table from the embedded guide and checks every `category` in the TOML blocks of the guides, README, recorded walkthroughs and schema examples, in the help examples, and in the checked-in policies (this repository's, the Immich overlay and the demonstration fixture). Each must be a reference name for its purpose or be declared in the same TOML's `[categories]`. The guard fails in CI and changes no runtime behavior.

**Required: aligned model policies and documents.** In the same change:

- Immich overlay: `test-infrastructure` becomes `test-support`, `performance-test` becomes `performance`, `.devcontainer/**` moves from `deployment` to `tooling`, and the documentation site's `docs/docusaurus.config.js` and `docs/sidebars.js` become `documentation-site`. The showcase recordings and README excerpts are regenerated.
- This repository: `tooling-test` becomes `unit` and `documentation-check` becomes `integration`; the self-measurement record is updated.
- Documents: names the guard rejects are replaced, such as the explain walkthrough's `order-rules`.

**Optional:** reference names validate without being declared in `[categories]`, and reports record their reference meaning. This would save boilerplate but make the reference part of validation; it is left for a later decision.

**Excluded:**

- Framework profiles. Framework layers such as UI, API or route handlers are implementation; folders and workspace members already separate them. Framework notes join the guide only with measured evidence.
- Hints in reports when a policy uses a listed synonym. They wait until the vocabulary has settled.
- Renaming built-in categories.

## Feasibility

The guide is one Markdown file, one `Guide` variant, one help line and one README table row; [guides.rs](../../src/guides.rs) already embeds the others the same way. The guard parses one Markdown table and TOML blocks the existing `guides::` tests already extract. Alignment changes classification only by category, so purpose totals and module credit stay; the showcase's category rows change and are regenerated. The guide's length adds reading cost for agents only when they request it.

## Acceptance

- `locmetry --guide categories` prints the guide, and `--help` lists the topic.
- The guard passes on the repository and fails on a fixture document that uses an unlisted, undeclared name.
- This repository and the Immich overlay use only reference names; on both pinned Immich tags, purpose totals and module credit are unchanged and the recordings replay.
- The README's guide table, Category term and Configuration section link to the reference; the configure guide and schema descriptions link to it instead of restating its rules, and the changelog announces the guide.
