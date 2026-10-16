# Documentation Index Maintenance

Every shared documentation directory has a README listing its direct Markdown documents and linking to its child indexes. Start at the [documentation index](README.md). Indexes provide navigation; [PLAN](PLAN.md) owns open work and the [product contract](PRODUCT.md) owns supported behavior.

## Scope and Ownership

The checker discovers tracked and new, non-ignored Markdown files under `docs/`, including intermediate directories. Each direct document and child index needs exactly one whole-document link in its parent README. Additional links to specific sections are allowed. README files do not index themselves. Asset-only directories need no README. Private `docs/research/local/` is always excluded, even if accidentally tracked; never copy its names or evidence into shared indexes.

Ordinary indexes use handwritten task-oriented bullets with concise descriptions. Proposals, plans and releases use generated tables with one row per direct document: Document, Status, Date and Summary. Their introductions and child-directory links remain handwritten. Tables are bounded by `<!-- doc-index:start -->` and `<!-- doc-index:end -->`; do not edit their contents manually. Empty status directories retain both markers and show an explicit empty state.

## Status and Date Metadata

Each non-README document under `docs/proposals/`, `docs/plans/` or `docs/releases/` has exactly one `doc-status` JSON comment, immediately after its title. This metadata is the authoritative source for the index. The document's first level-one heading supplies its link label. Rendered READMEs expose the status and date to humans; the comment keeps the same fields easy for agents and the script to read.

```markdown
# Example Proposal

<!-- doc-status
{
  "status": "draft",
  "date": "2026-10-16",
  "summary": "A bounded proposed behavior change.",
  "evidence": ""
}
-->
```

All four fields are required strings. Use a nonempty, single-line summary and single-line field values. `evidence` contains regular Markdown links relative to the document, or an empty string when no decision has been made. Do not use counts or percentages as progress estimates.

| Document type | Allowed statuses and meanings |
| --- | --- |
| Proposal | `draft`: under discussion; `accepted`: owner-approved scope, implementation not yet verified; `implemented`: accepted scope verified; `rejected`: declined by the owner; `superseded`: replaced by a linked decision. |
| Plan | `planned`: agreed future work; `active`: execution underway; `deferred`: on hold or conditional; `completed`: the plan's entire accepted scope verified; `historical`: retired snapshot whose body can include unresolved historical items. |
| Release | `unreleased`: development release record; `published`: publication and artifact verification recorded. |

Dates use `YYYY-MM-DD` and record when the current status took effect. For `historical`, this is the snapshot retirement date; for `unreleased`, it is when that development release was established; for `published`, it is the verified publication date. An implementation date remains an implementation date even if publication happens later. Do not derive dates from filesystem timestamps or refresh them on wording changes. Backfill from dated records; if no reliable date exists, resolve and record that evidence gap before assigning one.

`accepted`, `implemented`, `rejected`, `superseded`, `completed`, `historical` and `published` require at least one decision or verification evidence link. Keep dates and claims consistent with that evidence. For historical snapshots, link the retirement/current-scope record. For superseded work, link its replacement and the decision. Other states should link their scope decision when available. A valid link does not prove approval, completion or publication: reviewers must assess its content. Existing historical notices remain contextual evidence; preserve their text, even when they describe an earlier state inside the archived body.

## Updating and Checking

1. Update the owning document's metadata and any live prose invalidated by the decision. Preserve archived bodies; only outer metadata, archival notices and necessary links may change. Follow the [proposal retirement procedure](proposals/README.md#retirement-procedure) when retiring a design.
2. Add handwritten entries for new ordinary documents or child directories. Add a README for a new documentation folder; status indexes need the managed markers above.
3. Run `python3 scripts/doc-indexes.py update` from the repository root. It validates metadata and navigation, then regenerates only managed table sections. Validation errors prevent writes; missing ordinary entries and missing READMEs require an author to supply meaningful navigation.
4. Run `python3 scripts/doc-indexes.py check` and `git diff --check`. Commit document status/date changes and refreshed indexes together. Update PLAN only when open scope, status or a decision gate changes.

The script uses Python 3.9+ standard library and Git for repository inventory; these are development/CI tools, not dependencies of Locmetry or public Cargo tests. No package installation is required. CI runs the checker and its regression suite (`python3 scripts/test_doc_indexes.py`). Changes to this tooling require red/green regression tests; documentation-only updates require the checker and diff validation, with history preservation checks when archives change.

Check mode is read-only. It rejects missing indexes, missing or duplicate direct entries, malformed metadata, invalid status/date values, missing required evidence links, stale tables, and broken local links or Markdown heading fragments. Local link checking covers the shared docs plus the root README and CLAUDE files. It skips fenced/indented code examples, inline code and HTML comments; metadata evidence links are checked explicitly. It never fetches external links.

Use regular inline Markdown links with relative destinations and ATX (`#`) headings. The checker supports angle-bracket destinations for spaces and GitHub-style duplicate-heading suffixes. Reference-style links, raw HTML anchors and other Markdown extensions are outside this navigation contract; add parser support and regression coverage before using them for navigation. The checker validates structure and synchronization; review remains responsible for the truth of status, date, summaries and evidence.
