# README Structure, Tone and Documentation Boundaries

<!-- doc-status
{
  "status": "draft",
  "date": "2026-10-17",
  "summary": "Shorten the README around runnable examples, precise classification explanations and linked task guides.",
  "evidence": ""
}
-->

This proposal is for external review. It records the owner's requests, the documentation comparison, alternatives and recommendations; it does not approve a README rewrite or change supported CLI behavior. Findings describe the repository at `e31eeb9`, reviewed on 2026-10-17. Recommendations remain open for discussion.

## Decision Requested

Adopt a shorter README that explains what Locmetry measures, makes installation easy to find, demonstrates language and folder reports, and gives enough classification and configuration context to interpret them. Use the existing task guides for detailed workflows and the product contract for exhaustive semantics.

The recommended approach is a focused restructuring, rather than a wording-only edit or a minimal landing page. It preserves substantive examples while removing repeated explanations and reference material from the entry point. No fixed word or line budget is proposed: the acceptance criteria below define what must remain understandable and discoverable.

## Owner Input

The following rephrases all input from the discussion. “Requested fix” and “idea” preserve the owner's distinction; neither authorizes implementation during this proposal stage.

| ID | Input type | Request or concern |
| --- | --- | --- |
| U1 | Question | Explain why the opening begins with a current stable-release announcement, then says the examples describe the current checkout and links to output-routing semantics. This material seems misplaced before the tool's purpose is introduced. |
| U2 | Idea | Keep current release numbers out of README to reduce release maintenance. |
| U3 | Requested fix | Replace the marketing-style introduction about lines of code split into production, tests, tooling and what nobody claims with more factual language. |
| U4 | Idea | Extend “What you see” with a folder table using a value of 30, together with useful commands, an options list and an explanation of why configuration is needed. The proposed concrete command is `locmetry --folder-rows 30`. |
| U5 | Requested fix | Expand “Compared with line counters” to mention Tokei and ALOC explicitly. |
| U6 | Requested fix | Replace the incomplete wrapper example with the complete supported wrapper list and a link to details. |
| U7 | Idea | Make the depth of the codebase analysis clear through technical substance, without marketing language. |
| U8 | Concern and proposal | README is too long and text is too dense. Move material out or link to guides and follow-up documents; creating or extracting documentation is allowed where useful. |
| U9 | Research request | Analyze README structure, style and tone, and compare it with Tokei, SLOC, cloc and ALOC before deciding on the rewrite. The owner confirmed the intended repositories as `XAMPPRocky/tokei`, `flosse/sloc`, `AlDanial/cloc` and `modern-tooling/aloc`. |
| U10 | Process | Discuss the issues before editing README. Capture the complete discussion as a Markdown proposal for external review, including options, alternatives, recommendations and their rationales. |

## Evidence and Diagnosis

### Local Structure

The [current README](../../README.md) has 685 lines, approximately 6,600 whitespace-delimited words including tables and code, and 32 fenced blocks containing 233 content lines. These are source measurements at the reviewed revision, not rendered reading-time estimates. Installation begins at line 186 and Quick Start at line 225. Before installation, the page presents seven introductory capability bullets, ten use cases and twelve feature subsections. The sample-project tour occupies 267 lines, approximately 39% of the file.

The main structural problem is repetition and reading order. Folder views recur in the first example, use cases, options, terminal-table explanation, adaptive detail and output formats. Configuration, workspaces, agent guidance and JSON similarly appear in several places. A reader encounters a feature catalogue before reaching installation, then encounters a second explanation of many capabilities in the tour.

Density is also local: several paragraphs combine a command, its semantics, exceptions and links to deeper contracts. Breaking these into distinct concepts will help, but shortening sentences alone will leave the repeated material and competing reading paths intact.

### Tone and Precision

The opening slogan confuses dimensions: `production`, `test` and `unclassified` are purposes; `tooling` is a category. “What nobody claims” also conflates deliberately unclassified code with unresolved code that needs review. The [configuration guide](../guides/configure.md#decide-purpose-not-just-coverage) explains that distinction.

Phrases such as “Size a codebase honestly,” “Guard quality in CI,” “Honest coverage” and “Give coding agents a reliable answer” assert broad benefits where a narrower operation or guarantee would be more precise. A test-code ratio can support a policy check; it does not establish software quality. The early agent-trial agreement statistic has qualifications that require context before the reader even understands the tool. Preserve that evidence in the [research record](../research/real-project-trials.md#agent-configuration-trials), rather than making it part of the introduction.

### Comparison with Other READMEs

This is an editorial comparison of official documentation reviewed on 2026-10-17, not an independent accuracy, speed or feature-completeness benchmark. The external links address changing default-branch documents. In particular, the ALOC page available to the review was a cached GitHub view marked as crawled three weeks earlier; its raw README could not be retrieved, so no precise length comparison is asserted for it.

| Project and source | Structure and tone observed | What to borrow | What not to copy |
| --- | --- | --- | --- |
| [Tokei README](https://github.com/XAMPPRocky/tokei/blob/master/README.md) | A factual opening leads immediately to representative output, then contents, features, installation, configuration and examples. Stronger speed and accuracy claims appear in the feature discussion. | Show the result early; make examples discrete and navigable. | Exhaustive options and language inventories make the page long; promotional claims are not needed to communicate Locmetry's mechanisms. |
| [SLOC README](https://github.com/flosse/sloc#readme) | Concrete line-count types and formats precede installation and command/output examples. The relatively compact technical page also includes library use, integrations and a language inventory. | Pair commands with output and use short factual descriptions. | Mixing CLI, library and contributor audiences weakens the reading path; inventories do little to help a new CLI user start. |
| [cloc README](https://github.com/AlDanial/cloc/blob/master/README.md) | A precise definition leads into release information, an early Quick Start and worked examples, then a substantial manual with mechanism explanations. | Explain how meaningful results are produced, using concrete examples. | The full manual and release-specific detail are poor templates for shortening README or reducing release maintenance. |
| [ALOC README](https://github.com/modern-tooling/aloc#readme) | A descriptive sentence leads directly to installation and annotated commands, then tables, configuration, role classification and longer effort-model discussion. It describes the main reports without an early full report example. | Reach runnable commands quickly and keep common options compact. | Do not let specialist interpretation become a second manual within README; retain Locmetry's actual output demonstration. |

The comparison supports early examples and straightforward language, not copying any peer's entire structure. ALOC already advertises role classification, and cloc documents comparison and filtering workflows. Locmetry must not present all peers as counters that only produce language totals or imply that classification is unique to it.

## Proposed Scope

### Required in the Recommended Design

- Replace the opening slogan and release-status paragraph with a factual description.
- Keep installation, a language report and a folder report easy to reach.
- Add the `--folder-rows 30` demonstration, a compact command/options overview and configuration rationale.
- Explain analysis depth through representative decisions and inspectable evidence.
- Include Tokei and ALOC in the related-tools comparison and provide a complete wrapper list with a detail link.
- Consolidate repeated explanations and retain discoverability through the existing guides and documentation index.
- Remove the need to rewrite README's current-release identity on every release, while retaining valid examples and installation paths.

### Optional

- Preserve the full sample-project narrative in one dedicated walkthrough if reviewers consider its continuity valuable beyond the shorter README examples.
- Add a short contents list if the resulting README still requires navigation beyond its visibly distinct sections.

### Excluded

- Changes to counting, classification, precedence, output routing, folder selection or CLI defaults.
- New performance, accuracy, productivity or quality claims; competitor benchmarking.
- A new documentation platform, a second general manual, or a comprehensive CLI reference duplicated from `--help`.
- Rewriting historical research or reopening deferred folder-partition behavior.

## Editorial Decisions and Alternatives

### 1. Overall Reading Path

| Option | Assessment |
| --- | --- |
| Wording-only cleanup | Smallest change, but leaves repeated capabilities, late installation and dense reference material. Not recommended because it does not resolve U8. |
| Focused restructuring with examples and guide links | **Recommended.** Keeps enough evidence to understand and evaluate the tool while giving detailed workflows a clear owner. |
| Minimal landing page with almost everything linked away | Produces the shortest README, but makes readers navigate elsewhere before they can understand classification or judge the reports. Prefer only if extreme brevity becomes the primary requirement. |

Recommended section order:

1. Factual introduction.
2. Installation.
3. “What you see”: language output followed by folder output.
4. Common commands and selected options.
5. How classification works and why configuration exists.
6. Comparison with related tools.
7. Guides and reference links.
8. License.

Putting the first report before installation is a reasonable alternative, as Tokei does. The recommendation is installation first because it can be brief and immediately actionable; the first report still arrives before any feature catalogue. A rendered review should settle this tradeoff if the demonstration becomes visually distant.

### 2. Introduction and Technical Credibility

Recommended draft introduction:

> Locmetry counts code, comments and blank lines, classifies files as production, test or unclassified, and records the rule behind each classification. Reports separate source code from data and documentation and show results by language and folder.

An optional following sentence can introduce categories: “Categories distinguish implementation, build tooling and project-specific roles within those totals.” Prefer explaining this alongside the worked decisions if the introduction becomes crowded.

Keeping a revised slogan is an alternative, but offers little explanatory value and risks repeating the same purpose/category ambiguity. A longer technical introduction would establish more detail at the cost of delaying the first result. The recommended two-sentence opening names the actual model without requiring a vocabulary lesson first.

Communicate analytical depth through a compact example rather than a longer feature list:

| Example decision | What it demonstrates |
| --- | --- |
| Application source becomes production implementation through module inference. | Recognized build-descriptor names and conventional layouts supply context. |
| A colocated `cart.test.ts` remains test code inside an application's source tree. | Test conventions take precedence over ordinary rules and module inference. |
| A maintained script receives the project's tooling classification. | Configuration supplies repository-specific intent. |
| A build wrapper remains deliberately unclassified with a known category. | Unclassified does not necessarily mean unresolved. |
| An uncategorized prototype falls through for review. | Missing intent remains visible rather than silently becoming production. |

Pair one result with `locmetry explain` to expose its deciding rule and evidence. Briefly connect nested module context, explicit overrides, separate purpose/category/content-kind dimensions, reconciled folder totals and visible unavailable counts to the decisions readers can inspect. These are supported mechanisms, not a claim of semantic understanding of arbitrary code.

Keep the material boundary visible: module inference uses selected filenames and conventional layouts; it does not parse build configuration or execute builds. Custom layouts need policy, and whole-file classification cannot separate inline tests from production within a file. The [module contract](../PRODUCT.md#module-detection), [precedence contract](../PRODUCT.md#matching-and-precedence) and [measurement limits](../measurement.md#limits) own the details.

### 3. Folder Demonstration, Commands and Configuration Rationale

Extend the language example with actual output from the same synthetic project using:

```sh
locmetry --folder-rows 30
```

The recommendation is to retain both views because they answer different questions: language composition and where code resides. Replacing the language view with folders would hide the familiar counting entry point; adding a large folder report elsewhere would reproduce the present navigation problem.

Explain that 30 is a soft display-row budget, not an exact result count, depth limit, default or cap on measured files. The option enables the folder table automatically and leaves the measurement unchanged. Initial top-level rows can exceed the budget; a small project can produce fewer rows. Do not pad the example or invent output. The [adaptive-folder contract](../PRODUCT.md#adaptive-folder-detail) owns expansion and reconciliation details. The example is terminal table output; saved or piped table examples require explicit table format. Workspace mode does not accept this option.

Use a compact task/command table for folder detail, verbose notes, explaining a file, saving JSON, comparing reports and measuring a workspace. Include the selected options readers need to understand those commands and link to `--help` for the exhaustive list. A separate full options table is an alternative, but would duplicate the binary's reference and increase density; omitting options entirely would underserve U4.

Follow with a short explanation of configuration: conventions recognize common layouts, while project rules declare exclusions and local intent such as maintained tooling, vendored code or experiments. Show a minimal rule and its observable effect. Do not imply vendored or generated content is automatically recognized. Keeping all configuration detail inline would preserve self-containment but recreate the manual; linking everything away would fail to explain why configuration matters. The [configure guide](../guides/configure.md) is the recommended detailed destination.

### 4. Related Tools and Complete Wrapper Coverage

Rename the comparison section to “Compared with related tools” if helpful to avoid suggesting every peer is only a language counter. Include Tokei, SLOC, cloc and ALOC for consistency with this review; retain the existing scc reference where it remains useful. Explicitly say that Locmetry embeds Tokei for counting and adds its own policy, classification and reporting behavior.

Recommend a short factual paragraph or compact table of orientation, with a link to the dated [capability comparison](../research/tool-capability-comparison.md). An exhaustive competitive matrix would require a separate evidence refresh and is outside this proposal. Do not use that existing research as evidence for peer capabilities it did not examine. Describe Locmetry's combination of guarantees without treating unverified competitor absences as facts.

The wrapper sentence must enumerate `gradlew`, `gradlew.bat`, `mvnw`, `mvnw.cmd`, and files under `gradle/wrapper/` or `.mvn/wrapper/`, at any matching repository depth. When enabled, the built-in wrapper convention assigns purpose `unclassified` and category `third-party-wrapper` unless an earlier classification stage or convention wins. Overrides and test conventions take precedence, and `classification.builtin_build = false` disables the wrapper convention. For example, `tests/gradlew` matches the test-directory convention first under default policy. Existing selection and counting boundaries also apply; wrapper binaries do not become countable merely because their paths match.

Recommend one complete sentence in README, linked to a human-readable explanation under [matching and precedence](../PRODUCT.md#matching-and-precedence), where the built-in definition is already referenced. The [source definition](../../src/config.rs) remains the executable authority. Linking only to source minimizes prose duplication but is less helpful to users; creating a separate wrapper document adds navigation without sufficient scope. Maintain the short enumeration and detailed explanation together when the supported set changes.

### 5. Release Numbers and Opening Caveats

Remove the opening release announcement. Move the output-routing link beside the report commands, where destination and format are relevant. If the distinction between checkout documentation and an installed release remains necessary, place a short version-independent note near installation or guide discovery; the installed binary's guides describe that binary's behavior.

| Option | Assessment |
| --- | --- |
| Keep current release numbers and automate all replacements | Avoids manual edits but retains editorial coupling to releases and the current script's exact README markers. Not recommended. |
| Remove current-release claims; use stable links and illustrative version data only where needed | **Recommended.** Removes release churn without making valid report examples or pinned-install guidance misleading. |
| Remove every version-shaped value, including schema/configuration versions | Not recommended. Configuration and schema versions describe compatibility, and JSON examples may need valid producer-version data. These are different from a claim about the latest release. |

Keep the Homebrew command as the primary copyable installation example and link to the latest release for binary selection. For optional pinned source installation, use an explicitly explained version placeholder rather than a hardcoded current tag; do not present a placeholder as a command that runs unchanged. Moving platform-specific download recipes to the [distribution documentation](../tokei-integration.md#distribution) is preferable to adding automatic release-discovery machinery just to keep a long README recipe current.

If a JSON example retains a concrete producer version, label it as illustrative rather than updating it every release. Retain `version = 1` and other configuration/schema values wherever the public contract requires them. Remove the prose assertion that the current checkout identifies as a particular release; `locmetry --version` can demonstrate how to inspect the executable instead.

### 6. Documentation Destinations

| Current material | Recommended disposition and rationale |
| --- | --- |
| Opening capability bullets, use cases and feature catalogue | Consolidate into the introduction, two outputs, command table and concrete classification decisions. These are overlapping explanations of the same capabilities. |
| Installation and first invocation | Keep near the top; move lengthy platform/source detail behind the existing distribution link. |
| Configuration, custom mappings and personal defaults | Keep rationale and a minimal example; link to the [configure guide](../guides/configure.md) and [configuration contract](../PRODUCT.md#project-configuration) for details. |
| Build-system layout inventory and precedence edge cases | Link to [module detection](../PRODUCT.md#module-detection) and [matching and precedence](../PRODUCT.md#matching-and-precedence); retain only the evidence needed for the README example. |
| Explanation variants and selection diagnostics | Keep one representative explanation; link detailed workflows to the [selection guide](../guides/selection.md). |
| JSON and Markdown dumps, queries, output routing and CI recipes | Keep a short export entry point; move any unique recipes to the [reports guide](../guides/reports.md), which already owns interpretation. Preserve partial-result and exit-status meaning. |
| Adaptive-folder algorithm details | Keep the requested command/result and soft-budget interpretation; link deeper rules to the [adaptive-folder contract](../PRODUCT.md#adaptive-folder-detail). |
| Workspace setup and member behavior | Keep a command entry; link to the [workspace contract](../PRODUCT.md#workspace-mode) and reports guide for policy and interpretation. |
| Saved-measurement comparison | Keep a short entry point; put detailed recipes and evidence limits in the [compare guide](../guides/compare.md). |
| Agent configuration | Keep guide/schema discovery and a compact task prompt; retain the full process in the configure guide. |
| Agent-trial statistics, parser-specific caveats and historical implementation evidence | Link to [research](../research/README.md), [measurement semantics](../measurement.md) and existing historical records rather than reproducing them in the introduction. |
| Large documentation catalogue and contributor index policy | Replace with user-facing guide links plus the [documentation index](../README.md); contributor maintenance rules already have an owner. |

A dedicated sample-project walkthrough is the main optional extraction. It would preserve the tour's continuous narrative, but it would also create another maintained explanation alongside four existing guides. Recommend using existing guides first, moving only unique material and keeping representative examples in README. Choose a new walkthrough only if review establishes that the end-to-end narrative is independently useful. Do not split the tour mechanically across documents or copy material already owned elsewhere.

## Compatibility and Review Constraints

The proposed editorial change has material dependencies in release preparation, executable recipes and Rust regression guards:

- The [release script](../../scripts/release.py) currently requires seven README version markers and rewrites them during release preparation.
- The [documentation-example check](../../scripts/check-doc-examples.sh) selects some examples by README heading and block position.
- The `producer_identity_and_docs_distinguish_the_checkout_from_stable_installs` guard in [comparison tests](../../tests/compare.rs) requires the stable-release announcement, installation heading/tag and current producer-version example. Replace the obsolete README identity assertions while retaining executable producer-version and comparison-finding coverage.
- The `documented_examples_satisfy_their_schemas` guard in [schema tests](../../tests/schema.rs) reads TOML blocks from README, PRODUCT and the configure guide, and requires a workspace example currently supplied by README. If an example moves to another document, include its new location in schema validation and retain workspace-example coverage; do not merely remove the assertion.

A version-free or reorganized README must preserve these guarantees at the examples' final locations. Merely deleting text would break the existing guards or leave relocated examples unchecked.

Existing guide links to the sample tour and CI section must continue to reach an appropriate explanation after any extraction. The [documentation maintenance rules](../documentation-maintenance.md) govern indexes and links. Guide edits also change embedded `--guide` content because the binary includes those Markdown files directly. These constraints do not justify retaining an overloaded README, but they make the documentation boundary a behavior-preserving change rather than a cosmetic deletion.

No change to the README, runtime, release tooling or guides is made by adopting this draft for review. Approval of a later rewrite should cover its defined documentation scope; runtime feature work remains excluded.

## Acceptance Criteria for a Later Rewrite

- The opening accurately distinguishes purposes and categories and makes no current-release announcement or unsupported quality claim.
- Installation and representative output appear before detailed capability discussion.
- The language and folder examples use the same synthetic project, show actual output and explain `--folder-rows 30` without implying an exact row count or changed measurement.
- A reader can find common commands, selected options, configuration rationale and one classification explanation without leaving README.
- The comparison includes Tokei and ALOC, credits the embedded counting backend, and does not imply unsupported uniqueness or accuracy rankings.
- The wrapper enumeration is complete, links to its detail owner and describes the enabled convention subject to classification precedence, selection and counting boundaries.
- Each relocated workflow remains discoverable, unique evidence is preserved, and historical bodies are not rewritten to describe current behavior.
- Current-release identity changes no longer require README edits; illustrative producer versions and contract versions retain their distinct meanings.
- Obsolete README release-identity assertions are replaced without weakening executable producer-version or comparison-finding coverage; relocated TOML examples, including the workspace example, remain schema-checked at their final locations.
- Local links and managed indexes remain valid; affected executable examples, release-preparation guards and embedded-guide checks still establish their existing guarantees.
- The rendered page has a clear reading path without repeating the same detailed explanation across an introductory list, a feature section and a tour.

## Questions for External Review

1. Does the proposed reading path retain enough substance to evaluate Locmetry? **Recommendation:** approve focused restructuring, not wording-only cleanup or an almost-empty landing page, because both hands-on output and classification rationale are needed.
2. Should installation precede the first output? **Recommendation:** yes, with a compact installation section; prefer output first only if rendered review shows that the example loses its early prominence.
3. Does the neutral introduction and decision example communicate analysis depth accurately? **Recommendation:** use the proposed model and mechanisms, keeping the filename-inference and whole-file limits visible rather than adding promotional language.
4. Is a separate sample walkthrough worth maintaining? **Recommendation:** use existing guides unless reviewers identify a specific end-to-end learning need that those guides cannot meet without fragmenting the explanation.
5. How much command and option reference belongs in README? **Recommendation:** common tasks and their relevant options, with full help and contract links; an exhaustive duplicate increases maintenance without improving the initial workflow.
6. Should all apparent version numbers disappear? **Recommendation:** remove current-release identity, retain compatibility values and clearly illustrative report data, and use stable installation links plus explicitly explained placeholders where pinning matters.
7. Is the related-tools comparison adequately supported and fair? **Recommendation:** keep it short, credit Tokei and acknowledge ALOC's role classification; require fresh evidence before asserting competitor limitations beyond this editorial review.
8. Does the proposal preserve every owner request without expanding into runtime work? **Recommendation:** assess against U1–U10 and the exclusions above; request a separate proposal for any new CLI or measurement behavior.
