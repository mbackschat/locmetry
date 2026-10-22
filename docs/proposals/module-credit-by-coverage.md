# Module Credit by Layout Coverage

<!-- doc-status
{
  "status": "accepted",
  "date": "2026-10-23",
  "summary": "Credit a build module with the production source its layout covers, whichever stage decided it, and count only modules with credit.",
  "evidence": "[Owner decision in PLAN](../PLAN.md#accepted-proposals--implementation-order)"
}
-->

The Immich showcase found this gap on 2026-10-22. The [policy test drives](../research/policy-test-drives/README.md) of 2026-10-22 and 2026-10-23 found it in every project.

## Problem

`summary.build_modules`, the human note "Production from build modules" and the Markdown module list credit a module only with files its layout decided. Any entry that matches a module's file takes the credit, even when it confirms the module's purpose and only names a more specific category:

- Immich's Android runner is Gradle module source. Labelling it `native` like the iOS code needs an entry, which moves all 4,102 code lines from `builtin:gradle-module` to the entry. The note then drops Gradle, while `summary.build_modules.gradle` still reports 2 modules with 0 code lines.
- Nothing in the reports says that an entry took the credit, and the module count keeps counting modules that are left with none.

The credit answers "how much production source do the build files account for", but it is computed from "which stage decided the file". A policy that only agrees with a module therefore makes the module look smaller.

## Evidence

| Project | Entries that keep `production` and set another category | Code lines | Module credit |
| --- | --- | ---: | --- |
| Immich | Android runner as `native` | 4,102 | Gradle 4,102 to 0 |
| [CIB seven](../research/policy-test-drives/cibseven.md) | OpenAPI templates, QA tooling, examples | 43,207 | Maven modules with code 80 to 60, while 198 are reported |
| [Temporal](../research/policy-test-drives/temporal.md) | Generated code, documentation sample apps, docs tooling | 7,866 | Python, Go and Maven sample modules drop out of the note |
| [Loki](../research/policy-test-drives/loki.md) | Generated protobuf and parser code, developer tools | 134,430 | Go 396,844 to 262,414 |
| [axum](../research/policy-test-drives/axum.md) | Example crates as `example` | 4,795 | Cargo 27,364 to 22,569, while 61 crates are reported |

Two private projects showed the same pattern for demo apps, documentation snippets, build logic and tooling. In every project, entries that change purpose also took module files, as they should: vendored Go modules, test fakes, generated diagrams. Separately, the count included modules without any source: in a private project almost half of the Gradle "modules" were folders holding only a build file, and several npm packages had no `src/`.

## Design

**Required: a module's credit is the production source its layout covers.** A measured source file counts for the innermost module whose layout covers it, chosen as in [Module Detection](../PRODUCT.md#module-detection), when its final purpose is `production`, whichever stage decided it:

- An entry that keeps `production` and sets a category, such as `native`, `generated` or `tooling`, keeps the module's credit. An entry that changes the purpose to `test` or `other`, such as for vendored or generated-and-unmaintained code, removes it.
- `rule` keeps naming the deciding stage. `summary.largest_module_files` keeps listing only files a layout decided, because its purpose is to surface production that nobody reviewed; a file an entry decided has been reviewed.
- Reports name the covering module in a new optional per-file field, `module`, with its `path` and `detector`, present on production source a layout covers. Readers and the report contract recompute the credit from it, since `rule` no longer identifies the module.

**Required: `summary.build_modules` counts only modules with credit.** `modules` becomes the number of modules covering at least one measured production source file, as it already is with subfolder selection, and a build system with none is omitted. `metadata.modules` keeps every detected module, and the Markdown module list keeps showing modules without credit, where 0 still hints at a misleading build file.

Both parts change reported values, so they ship together and are **Breaking**. No configuration changes: existing policies get the corrected credit as they are.

**Optional:** a note in `explain` that a file decided by an entry lies inside a module's layout.

**Excluded:**

- Category-only entries, an entry without `purpose` that sets only the category and leaves the purpose, and with it the credit, to a later stage. This was the earlier draft of this proposal. On Loki it kept the same 134,430 lines only after its entries were split by module coverage: a tools folder mixed module source with scripts no stage claims, so a category-only entry left 2,200 lines unclaimed, and deployment and example folders without module source left 8,462 more. A policy author would have to know which files a module covers to choose the entry form, while credit by coverage needs no new syntax.
- A separate table of category refinements limited to built-in decisions. It has the same mixed-folder problem and adds a concept.
- Rules that apply only to files no other stage decides. One project asked for them; that is separate scope.

## Feasibility

Classification already finds the innermost covering module for files no earlier stage decides; the same lookup runs for every production source file to fill `module`. Module shares then sum by that field instead of by `rule`. The largest-module-files list keeps its `rule`-based selection. The report contract check switches from `rule` to `module`; reports of earlier producers lack the field and keep their old meaning, so comparisons across this change report credit differences that come from the definition, not from the code. The schemas and PRODUCT's [report content](../PRODUCT.md#report-content) description change with it.

## Acceptance

- The Immich overlay, unchanged, credits Gradle with the Android runner's 4,102 code lines again; the note shows Gradle, and every other count is unchanged. The README excerpts and showcase recordings are regenerated.
- A fixture's module file decided by a rule that keeps `production` counts for the module and sets `module`; one decided by a rule with `purpose = "other"` does not count and has no `module`.
- `summary.largest_module_files` lists only files with a module `rule`.
- A module whose layout covers no measured production source is omitted from `summary.build_modules` and the note, but stays in `metadata.modules` and the Markdown module list.
- The report contract check recomputes credit from `module`. Both schemas, PRODUCT and the changelog, marked **Breaking**, describe the new definition.
