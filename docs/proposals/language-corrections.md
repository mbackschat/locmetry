# Language Corrections

<!-- doc-status
{
  "status": "draft",
  "date": "2026-10-22",
  "summary": "Policy entries that correct a language the counter recognized wrongly, naming another counter language or leaving the file unrecognized.",
  "evidence": ""
}
-->

The Immich showcase found this gap on 2026-10-22, as its [measurement recording](../showcase/measure.md#look-at-the-code-around-the-product) shows.

## Problem

The counter recognizes languages by file name and extension, and some extensions belong to more than one format. Immich's `mobile/android/app/proguard-rules.pro` holds ProGuard keep rules, but `.pro` is Prolog to the counter, so the showcase reports a Prolog row that does not exist. Qt project files use the same extension.

A policy cannot correct this. [Language mappings](../guides/configure.md#recognize-a-language-then-review-purpose) apply only to files the counter does not recognize. Classification entries set purpose and category, not language. An exclusion removes the file from every total and hides the question instead of answering it.

## Design

**Required: a `[[languages.corrections]]` entry.** It replaces the counter's language for the files it matches:

```toml
version = 1

[[languages.corrections]]
id = "proguard-rules"
match = ["mobile/android/app/proguard-rules.pro"]
language = "unrecognized"
reason = "ProGuard keep rules; the counter reads the .pro extension as Prolog"
```

- `id`, `match` and `reason` are required. A correction contradicts the counter, so it records its evidence, as a policy override does.
- `language` names a counter language, as a mapping does, or the reserved value `"unrecognized"`. ProGuard has no counter language, and counting its `#` comments with Prolog's syntax would produce wrong comment and code lines. An unrecognized file stays selected, counts nothing and is listed in `metadata.coverage.unrecognized`, which already means "selected but not counted".
- A file corrected to a language is counted with that language. Its content kind follows that language.
- Corrections apply before recognition, and the first matching correction decides. Mappings never apply to a corrected file.
- Corrections are allowed where mappings are allowed: in the project configuration and in personal defaults, with the project's corrections first.
- Reports name the deciding correction in a new per-file field, `language_correction`, beside `language_mapping`. `explain` shows the correction and its reason.
- A correction that decides no selected file is reported as unused, as unused classification entries and exclusions are. Unused mappings are not reported today, but a correction contradicts the counter, so a stale one should surface. Comparisons treat a change to corrections as a policy change, as they do for mappings.

**Optional:** built-in corrections for well-known names such as `proguard-rules.pro`, in a set selected by `version`, as the built-in mappings are. This proposal adds none: one observed case does not establish which names are safe everywhere.

**Excluded:**

- Changing mappings so that they also apply to recognized files. An existing broad mapping, such as a personal `**/*.foo`, would silently start overriding the counter.
- Mapping a misrecognized file to the nearest counter language. It would count its lines with the wrong comment syntax and report a language the file is not written in.
- Custom languages and custom content kinds. PRODUCT keeps custom content-kind overrides as separate optional scope.

## Feasibility

Measurement already counts unrecognized files through mappings after the counter's pass. A correction removes its matching paths from that pass and then sends them either through the same per-file counting with the named language, or into the unrecognized list. The counter's own recognition stays untouched. Schemas gain the configuration table and the file field. Older reports simply lack the field and remain valid.

## Acceptance

- In the Immich overlay, a correction to `"unrecognized"` removes the Prolog row, lists the ProGuard file as unrecognized, and leaves every other count unchanged. The showcase recordings and README excerpts are regenerated.
- A correction to a counter language counts a misrecognized fixture file with that language's comment syntax and content kind.
- A mapping matching a corrected file does not apply. The first of two matching corrections decides. An unused correction is reported.
- A personal correction applies after the project's corrections.
- `explain`, full JSON and both schemas show the correction. The configure guide and PRODUCT describe it next to mappings.
