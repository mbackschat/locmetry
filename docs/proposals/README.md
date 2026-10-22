# Proposals and Historical Decisions

Active designs live directly in this directory. Retired records live in `archive/`; their status notices identify the outcome and current contract owner. Directory placement does not establish approval: each document’s `doc-status` metadata is authoritative, and [PLAN](../PLAN.md) tracks accepted work and deferred scope.

## Active Proposals

<!-- doc-index:start -->

| Document | Status | Date | Summary |
| --- | --- | --- | --- |
| [Category Refinements](category-refinements.md) | draft | 2026-10-23 | Policy entries that change only the category of files a built-in convention or build module decided, keeping their purpose, provenance and module credit; build_modules counts contributing modules. |
| [Language Corrections](language-corrections.md) | draft | 2026-10-23 | Policy entries that correct a language the counter recognized wrongly, naming another counter language or leaving the file unrecognized. |

<!-- doc-index:end -->

## Archive

- [Historical proposal decisions](archive/README.md): one row per retired design, with outcome and date.

## Retirement Procedure

Retire a proposal when its accepted scope is implemented and verified, or when the owner rejects or supersedes it. Approval alone is not completion. A partly implemented proposal remains active unless the remaining scope is explicitly transferred to an active plan or replacement proposal. Publication is separate: verified development work can be retired without claiming a release.

1. Check implementation and acceptance evidence against the accepted scope. Identify the supported contract owner and completion checkpoint; record only verified versions, dates and outcomes. For a rejected or superseded design, link the decision or replacement instead of implying implementation.
2. Transfer every live follow-up, unresolved decision or deferred design into PLAN, the active work plan or a replacement proposal. Preserve its rationale and activation condition, and link it from the retired record. An archived proposal must carry no live implementation scope.
3. Move the full record to `docs/proposals/archive/` with its existing basename. Preserve headings, acceptance criteria, rationale and review history. Update relative links inside the moved file and all tracked repository references, including fragment links. Do not maintain a second full copy at the old path. Pinned historical Git URLs still address their original revisions; unpinned external links to the old path are not redirects.
4. Replace the opening status with “historical decision record — implemented,” “rejected,” or “superseded,” the dated outcome and links to the current owner and evidence. Explicitly state that imperative wording, examples and limitations below describe the reviewed design, not current guarantees. Later corrections to retirement metadata or links are allowed; do not revise the reviewed body to track evolving behavior.
5. Set the document metadata outcome, date and evidence according to the [maintenance rules](../documentation-maintenance.md), then regenerate the active and archive indexes. Remove resolved discussion from PLAN, retain only live follow-ups and a historical navigation link, and preserve the decision in the retired record. Update contributor guidance only when the procedure itself changes.
6. Verify inbound and outbound relative links and fragments, unchanged historical headings and acceptance text apart from link relocation, and the absence of unsupported release claims. Record routine verification in the commit handback, update plans only for material scope/status changes, and commit the retirement with the implementing checkpoint where possible. A documentation-only retirement needs link/history checks, not unrelated counting tests.

## Audit — 2026-10-16

All three pre-existing proposals were already marked historical and indexed as completed. The audit added consistent archival placement and notices, and corrected stale retirement metadata rather than reclassifying unfinished work as done. Source at tags `v0.2.0` and `v0.3.0`, the [release records](../plans/archive/development-plan-2026-10-16.md#minor-release-v030--published) and the linked implementation checkpoints establish net change/subfolder availability in v0.2.0 and adaptive detail in v0.3.0. Historical body text remains intentionally frozen, including output rules superseded by the [current routing contract](../PRODUCT.md#output-routing).
