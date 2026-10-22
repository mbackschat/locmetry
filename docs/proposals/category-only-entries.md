# Category-Only Entries

<!-- doc-status
{
  "status": "draft",
  "date": "2026-10-23",
  "summary": "Classification entries that omit purpose set only the category, so purpose, provenance and build-module credit stay with the stage that decides them.",
  "evidence": ""
}
-->

The Immich showcase found this gap on 2026-10-22. Policy test drives on six further projects on 2026-10-22 and 2026-10-23 found it in each of them.

## Problem

A classification entry sets purpose and category together and becomes the deciding stage. Renaming only the category of module source therefore takes the file away from its build module:

- Immich's Android runner is Gradle module source. Labelling it `native` like the iOS code needs an entry, which moves all 4,102 code lines from `builtin:gradle-module` to the entry. The human note "Production from build modules" then drops Gradle, while `summary.build_modules.gradle` still reports 2 modules with 0 code lines.
- Nothing in the reports says that an entry took the credit.

The entry stated nothing about modules or purpose; it only wanted a different category name. The loss is a side effect of coupling the two attributes, not of the policy's intent.

## Evidence

| Project | Category wanted for module source | Code lines | Module credit |
| --- | --- | ---: | --- |
| Immich | Android runner as `native` | 4,102 | Gradle 4,102 to 0 |
| A Java workflow engine | OpenAPI templates as `api-docs`, QA tooling, examples | 43,207 | Maven modules with code 80 to 60 |
| A TypeScript UI library | Demo apps and documentation snippets as `example` | 18,204 | npm 37,506 to 19,265; 3 of 9 modules now classify nothing |
| A TypeScript SDK | Generated code, documentation sample apps, docs tooling | 7,866 | Python, Go and Maven sample modules drop out of the note |
| A Java data service | Build logic, tooling and examples | 6,029 | Gradle modules with code 42 to 31 |

About 79,000 code lines in five projects were moved from build modules only to rename their category. Each row counts only entries whose purpose equals the one the module decided. The other two projects inspected lost credit the same way in smaller amounts. One project kept its credit only by listing sibling folders instead of writing a broad glob.

## Design

**Required: `purpose` becomes optional on overrides and rules.** An entry without it sets only the category:

```toml
version = 1

[categories]
native = "Platform code of the iOS and Android apps"

[[rules]]
id = "android-app"
match = ["mobile/android/app/src/main/**"]
kind = ["source"]
category = "native"
reason = "Kotlin, Java and C code of the Android runner, labelled like the iOS code"
```

- Purpose and category are resolved separately, each by the first match in the existing order: overrides, conventions, rules, build modules, fallback. An entry with `purpose` sets both, its category defaulting to the purpose as today. An entry without `purpose` must name a `category` and sets only that; the purpose comes from the next stage that decides it.
- The existing order gives the expected results without further conditions. Conventions run before rules, so a category-only rule never relabels conventional tests, build scripts or wrappers, while a category-only override does, as overrides beat conventions today. Build modules decide only `production` and run after rules, so module source takes the rule's category and keeps its purpose.
- A category is not applied to a file whose purpose comes from the fallback. The file stays `other:unclaimed` and is still reported as needing a decision.
- `rule` keeps naming the stage that decided the purpose, such as `builtin:gradle-module`. Module credit, `summary.build_modules`, `summary.largest_module_files` and unclaimed findings therefore keep their meaning. A new per-file field, `category_rule`, names the entry that set the category when it differs from `rule`. `explain` shows both entries and their reasons.
- An entry that set neither attribute of any file is listed in `summary.unused_entries`, as today. A `[categories]` table also governs category-only entries.
- Existing configurations classify identically, because every entry they contain sets a purpose.

**Optional:** report the module code that purpose-setting entries took, for example as a verbose note per build system. With category-only entries available, the remaining cases change purpose on purpose, such as generated code or test helpers. Add it only if reviewers still miss such moves.

**Excluded:**

- A separate table of refinements limited to built-in decisions, with `purpose` as a filter. The stage order already yields the same results, so the extra concept would only add rules to learn.
- Purpose-only entries that keep a later stage's category. A module's `implementation` would be wrong for a file the entry makes `test` or `other`.
- Rules that apply only to files no other stage decides. One project asked for them to fill gaps next to module source; that is separate scope.
- Counting only contributing modules in `summary.build_modules`. It has its own evidence and is tracked in [PLAN](../PLAN.md).

## Feasibility

Classification currently returns the first matching entry across stages. It becomes two lookups over the same ordered entries: the first entry that sets a purpose, and the first that sets a category, which is never later than the purpose decider because every purpose-setting entry also sets a category. Module shares are keyed by `rule`, which category-only entries do not touch, so credit is kept without further changes. Unused-entry tracking extends from "decided" to "set an attribute". Schemas gain the optional `purpose`, the per-file field and the `explain` field; older reports lack the field and stay valid.

## Acceptance

- In the Immich overlay, the `android-app` override becomes a category-only rule. The Android code stays `production:native` with 4,102 code lines, the module note shows Gradle again, and every other count is unchanged. The showcase recordings and README excerpts are regenerated.
- A fixture's module-decided file matched by a category-only rule takes the category, keeps its `rule`, sets `category_rule`, and leaves `summary.build_modules` unchanged.
- A category-only rule matching a tree leaves its conventional test files unchanged; a category-only override relabels them while their purpose stays `test`.
- A category-only rule matching only unclaimed files leaves them `other:unclaimed` and is reported unused.
- A category-only rule followed by a matching full rule yields the first entry's category and the second entry's purpose.
- An entry without `purpose` and without `category` is rejected. Existing configurations produce identical reports apart from the new field.
- `explain`, full JSON and both schemas show both deciders. The configure guide and PRODUCT describe category-only entries next to rules.
