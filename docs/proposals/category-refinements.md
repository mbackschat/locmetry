# Category Refinements

<!-- doc-status
{
  "status": "draft",
  "date": "2026-10-23",
  "summary": "Policy entries that change only the category of files a built-in convention or build module decided, keeping their purpose, provenance and module credit; build_modules counts contributing modules.",
  "evidence": ""
}
-->

The Immich showcase found this gap on 2026-10-22. Policy test drives on six further projects on 2026-10-22 and 2026-10-23 found it in each of them.

## Problem

A classification entry sets purpose and category together and becomes the deciding stage. Refining only the category of module source therefore takes the file away from its build module:

- Immich's Android runner is Gradle module source. Labelling it `native` like the iOS code needs an entry, which moves all 4,102 code lines from `builtin:gradle-module` to the entry. The human note "Production from build modules" then drops Gradle, while `summary.build_modules.gradle` still reports 2 modules with 0 code lines.
- Nothing in the reports says that an entry took the credit. The module count keeps counting every detected module, including those that now classify nothing.

The entry stated nothing about modules; it only wanted a different category name. The loss is a side effect of the mechanism, not of the policy's intent.

## Evidence

| Project | Category wanted for module source | Code lines | Module credit |
| --- | --- | ---: | --- |
| Immich | Android runner as `native` | 4,102 | Gradle 4,102 to 0; JSON still 2 modules |
| A Java workflow engine | OpenAPI templates as `api-docs`, QA tooling, examples | 43,207 | Maven modules with code 80 to 60; the note still says 198 |
| A TypeScript UI library | Demo apps and documentation snippets as `example` | 18,204 | npm 37,506 to 19,265; the note still says 9 modules, of which 3 now classify nothing |
| A TypeScript SDK | Generated code, documentation sample apps, docs tooling | 7,866 | Python, Go and Maven sample modules drop out of the note |
| A Java data service | Build logic, tooling and examples | 6,029 | Gradle modules with code 42 to 31; the note still says 96 |

About 79,000 code lines in five projects were moved from build modules only to rename their category. The other two projects inspected lost credit for the same reason in smaller amounts. One project kept its credit only by listing sibling folders explicitly instead of writing a broad glob.

## Design

**Required: a `[[refinements]]` entry.** It replaces the category of files a built-in stage decided and changes nothing else:

```toml
version = 1

[categories]
native = "Platform code of the iOS and Android apps"

[[refinements]]
id = "android-app"
match = ["mobile/android/app/src/main/**"]
kind = ["source"]
purpose = "production"
category = "native"
reason = "Kotlin, Java and C code of the Android runner, labelled like the iOS code"
```

- A refinement applies only to files that a built-in convention or a build module decided. Overrides, rules and the fallback are never refined: an entry already states its category, and an unclaimed file still needs a decision.
- `purpose` is a condition, not an assignment: the refinement applies only where the decided purpose equals it. A `production` refinement of a module tree leaves that tree's conventional tests alone. An entry therefore reads like the rule it replaces, while never changing purpose.
- `id`, `match`, `purpose` and `category` are required; `kind` and `reason` are optional, as for rules. IDs are unique across overrides, rules and refinements. A declared `[categories]` table also governs refinement categories.
- The first matching refinement decides. Refinements are allowed only in the project configuration, which is the only file that classifies.
- A refined file keeps its `rule`, such as `builtin:gradle-module`, so module credit, `summary.build_modules`, `summary.largest_module_files` and the module note are unchanged. Reports name the refinement in a new per-file field, `refinement`. `explain` shows the deciding stage, then the refinement and its reason.
- A refinement that refines no file is listed in `summary.unused_entries`: its globs match nothing, or every matched file has another purpose or was decided by an entry.

**Required: `summary.build_modules` counts contributing modules.** `modules` becomes the number of modules that classified at least one measured source file, as it already is with subfolder selection, and a build system with none is omitted. `metadata.modules` keeps every detected module, and the Markdown module list keeps reporting modules with 0 code lines as hints. In the evidence above, 45 of one project's 96 Gradle "modules" were folders holding only a build file. This changes reported values, so it is **Breaking**.

**Optional:** report the module code that rules and overrides decided, for example as a verbose note per build system. With refinements available, the remaining cases change purpose on purpose, such as generated code or test helpers. Add it only if reviewers still miss such moves.

**Excluded:**

- Making `purpose` optional on rules. A rule decides before build modules, so a category-only rule would need a purpose for files no module claims, and it would still take the files' provenance.
- Refining files decided by an override or rule. The entry already owns that category, and two entries would own one decision.
- Rules that apply only to files no other stage decides. One project asked for this to fill gaps next to module source; it is separate scope.
- Changing purpose. That remains the job of rules and overrides.

## Feasibility

Classification already returns the deciding stage. A refinement lookup runs only when that stage is a convention or a module, and replaces the category in the returned classification. Module shares are keyed by the `rule` field, which a refinement does not touch, so credit is kept without further changes. Unused-entry tracking works per entry ID and extends to refinements. Schemas gain the configuration table, the per-file field and the `explain` field; older reports lack the field and stay valid. Subfolder runs already keep only module shares with classified files; the summary count can apply the same filter while the Markdown module list keeps every share.

## Acceptance

- In the Immich overlay, the `android-app` override becomes a refinement. The Android code stays `production:native` with 4,102 code lines, the module note shows Gradle again, and every other count is unchanged. The showcase recordings and README excerpts are regenerated.
- A fixture's module-decided file refined to another category keeps its `rule`, sets `refinement`, and leaves `summary.build_modules` unchanged.
- A `production` refinement matching a module tree leaves the tree's conventional test files unchanged.
- A refinement matching only files decided by a rule, or only unclaimed files, is reported unused. The first of two matching refinements decides. Duplicate IDs and undeclared categories are rejected.
- `summary.build_modules` counts only contributing modules and omits a build system with none; `metadata.modules` is unchanged; the human note's count matches.
- `explain`, full JSON and both schemas show the refinement. The configure guide and PRODUCT describe refinements next to rules, and the changelog marks the module count as **Breaking**.
