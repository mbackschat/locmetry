# Reviewable Policy Entries and Category Refinements

<!-- doc-status
{
  "status": "draft",
  "date": "2026-10-24",
  "summary": "Per-entry decision counts and findings in reports, `except` lists on entries, and a refinement stage that changes only a file's category.",
  "evidence": ""
}
-->

Three projects asked, on 2026-10-24, how to categorize their tests more finely, for example property laws apart from unit tests, and then reported what slowed them down: a12-rulekit, bpmn-lean-experiment and a private Lean project. Their figures stay in the Git-ignored local research folder. The [policy test drives](../research/policy-test-drives/README.md#findings) of 2026-10-22 and 2026-10-23 had already raised the `except` list.

## Problem

**A policy entry can be partly wrong without any finding.** `summary.unused_entries` reports only an entry that decides no file. All three projects lost at least one review round to an entry that decided some files but not the intended ones:

- In a12-rulekit, a case-sensitive glob, `**/src/*Test/**`, missed the lowercase `src/test` folder. Two entries decided a small fraction of their files, and nothing was reported; the author noticed only because a table looked wrong.
- In the private Lean project, an entry matched a whole folder but decided fewer files, because an earlier entry took the rest. That ordering dependency is invisible in reports.
- In bpmn-lean-experiment, a broad override turned production demo hosts into tests. A glob inside an entry that matches nothing is not reported either.

**Giving a test a finer category requires an override, the most powerful stage.** Built-in test conventions win over `[[rules]]`, so a rule with `purpose = "test"` over conventional tests decides nothing. The [configure guide](../guides/configure.md#refine-a-category-without-changing-purpose) therefore recommends a narrow override and asks the author to remeasure and confirm that purpose totals did not change. Overrides beat every stage, so a glob broader than intended moves code between production and test, which is how bpmn-lean-experiment's mistake happened. The author checks the invariant by hand; the tool could guarantee it.

**Exceptions exist only as entry order.** To keep `FieldAgreementTest` out of a law category, a12-rulekit narrowed `*AgreementTest` to a `laws/` folder. Ordering an earlier entry to catch the exception works too, but reports do not show it.

## Design

**Required: per-entry decision counts.** Reports gain `summary.policy_entries`, in headline and full JSON, with one object per override, rule, refinement and language correction, in configuration order:

```json
{
  "id": "demo-tests",
  "stage": "override",
  "files": 12,
  "code": 840,
  "preempted": [{ "by": "builtin:test-directory", "files": 3 }],
  "unused_patterns": ["src/Demo/**"],
  "displaced": [{ "decision": "production:implementation", "files": 9, "code": 610 }]
}
```

- `files` and `code` count the counted files of every kind the entry decided. A small count beside an expected large one is the under-matching signal; no threshold can judge it.
- `preempted` lists, per deciding entry or built-in stage, files the entry matches but an earlier entry or stage decided. It makes ordering dependencies reviewable.
- `unused_patterns` lists the entry's globs that decided no file, as `unused_entries` does for whole entries. `unused_exceptions` lists `except` globs that removed no file the entry otherwise matched.
- `displaced` groups the decided files by the `purpose:category` they would have received without the entry, for overrides and rules from the next matching stage and for refinements from the category before refinement. Files that would have reached the fallback are omitted, because claiming unclaimed code is a rule's ordinary job.
- Optional fields are omitted when empty. `unused_entries` stays unchanged as the finding list.
- `--verbose` table and Markdown output add a note naming entries whose `displaced` contains another purpose, such as `demo-tests: production → test, 610 code lines`. Such moves are often intended, such as test runners classified as production tooling, so the note asks for review rather than warning.
- Language corrections report `files`, `code`, `preempted` and `unused_patterns`; they decide language, not purpose, so they have no `displaced`.
- With `--subfolder`, counts cover the measured subfolder only; cleanup judgments still need an unrestricted measurement. In a workspace, each member's own policy reports its entries in `metadata.members`, beside `unused_entries`, and a shared project-folder policy reports once in `summary.policy_entries`, summed over its folders.

**Required: `except` on classification entries.** Overrides, rules and refinements accept an optional `except` list of globs. A file matching one of them is not matched by the entry and falls through to later entries and stages:

```toml
[[overrides]]
id = "agreement-laws"
match = ["**/*AgreementTest.java"]
except = ["**/FieldAgreementTest.java"]
purpose = "test"
category = "agreement-law"
```

**Required: a refinement stage that changes only the category.** `[[refinements]]` runs after every classification stage and after generated-code markers:

```toml
[[refinements]]
id = "property-laws"
purpose = "test"
match = ["**/laws/**/*PropertyTest.java"]
kind = ["source"]
category = "property-law"
reason = "Property-based laws of the rule engine"
```

- `id`, `purpose`, `match` and `category` are required; `kind`, `except` and `reason` are optional. IDs share one space with overrides, rules and corrections, and `category` must be declared when `[categories]` is present.
- A refinement applies only to files whose decided purpose equals its `purpose`, whichever stage decided them. It never applies to files that reached the fallback, which stay `unclaimed` so they are reported for review, or to files a generated-code marker refined, so documented evidence keeps `generated`.
- The first matching refinement sets the category. Purpose, totals, module credit and the file's `rule` stay the same; file records name the refinement in a new optional field, `refinement`, and `explain` shows it with its reason.
- A refinement that decides no file is listed in `summary.unused_entries`.
- The configure guide's category section recommends a refinement instead of a narrow override; overrides remain for decisions that change purpose.

The module credit proposal [excluded a table of category refinements](archive/module-credit-by-coverage.md#design) because category-only entries leave files in mixed folders unclaimed when they replace purpose-setting rules. A refinement does not replace a rule: it acts only on files whose purpose is already decided and claims nothing, so a mixed folder keeps its claimed and unclaimed files as before. Generated-code markers already refine categories this way.

All three parts are additive: existing configurations measure the same, and the new report fields are optional fields of schema 3, as [report compatibility](../report-compatibility.md) describes.

**Optional:** `explain` lists the other entries that match a file but do not decide it, as `decision.also_matched`, so a single file's ordering dependency can be inspected without a full measurement.

**Excluded:**

- Classifying purpose by file content, such as the share of `theorem` declarations in a Lean file. Paths keep decisions deterministic and explainable, and a content predicate is per-language policy inside the counter; a12-rulekit argued against it. A content predicate restricted to refinements is deferred in [DEFERRED](../DEFERRED.md#configuration-and-categories).
- Orthogonal tags beside the category. They break the guarantee that category rows add up to purpose totals; bpmn-lean-experiment asked for them only if a second axis recurs, which [DEFERRED](../DEFERRED.md#configuration-and-categories) tracks.
- Purpose warnings that fail a measurement. Intended purpose changes are common, so a note is the right strength.
- `except` on language corrections and scope exclusions. No project asked for them.

## Feasibility

Classification is first-match over overrides, built-in conventions, rules and the innermost module, and compiled entries already carry an exception matcher, which module layouts use for folders such as Go's `vendor/`. The audit evaluates every entry instead of stopping at the first match. The displaced decision of a file is the next match after the deciding entry, and the counts aggregate per entry. Refinements are one more first-match pass over decided files. The added cost is one glob evaluation per entry and file, without file content. `compare` does not compare `policy_entries`; differing policies are already a comparison finding.

## Acceptance

- A synthetic fixture shows each finding: an entry with a case-mismatched glob decides fewer files than its folder holds, an entry behind an earlier one lists it in `preempted`, an entry with a glob that matches nothing lists it in `unused_patterns`, an unused `except` glob is listed in `unused_exceptions`, and an override over module production lists `production:implementation` in `displaced` and appears in the `--verbose` note.
- An `except` glob makes a matched file fall through to the next stage, as `explain` shows.
- A refinement with `purpose = "test"` over conventional tests changes their category, leaves `rule`, purpose totals and module credit unchanged, does not touch production files under the same glob, unclaimed files or marker-refined files, and is named as `refinement` in file records and `explain`.
- Existing configurations produce identical totals and classifications; the release fixtures stay valid and comparable.
- The configuration and report schemas, PRODUCT, the configure guide and the changelog describe the new fields.
