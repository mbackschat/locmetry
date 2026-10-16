# Proposals and Historical Decisions

Active designs live directly in this directory. Retired records live in `archive/`; their status notices identify the outcome and current contract owner. Directory placement does not establish approval: each document's status is authoritative, and [PLAN](../PLAN.md) tracks accepted work and deferred scope.

## Active Proposals

- [On-demand guides in the binary](embedded-guides.md) — proposed for external review; no implementation approval.

## Archive

- [Net change](archive/net-change.md) — aggregate/topology comparison implemented; [PRODUCT](../PRODUCT.md#net-change) owns behavior. Per-file comparison remains in the [conditional follow-up](../plans/measurement-quality-and-net-change.md#per-file-comparison).
- [Subtree measurement](archive/subtree-measurement.md) — `--subfolder` implemented; [PRODUCT](../PRODUCT.md#subfolder-measurement) owns behavior.
- [Adaptive folder detail](archive/adaptive-folder-detail.md) — `--folder-rows` implemented; [PRODUCT](../PRODUCT.md#adaptive-folder-detail) owns behavior. Partition convergence remains in [PLAN](../PLAN.md#known-display-follow-up).

## Retirement Procedure

Retire a proposal when its accepted scope is implemented and verified, or when the owner rejects or supersedes it. Approval alone is not completion. A partly implemented proposal remains active unless the remaining scope is explicitly transferred to an active plan or replacement proposal. Publication is separate: verified development work can be retired without claiming a release.

1. Check implementation and acceptance evidence against the accepted scope. Identify the supported contract owner and completion checkpoint; record only verified versions, dates and outcomes. For a rejected or superseded design, link the decision or replacement instead of implying implementation.
2. Transfer every live follow-up, unresolved decision or deferred design into PLAN, the active work plan or a replacement proposal. Preserve its rationale and activation condition, and link it from the retired record. An archived proposal must carry no live implementation scope.
3. Move the full record to `docs/proposals/archive/` with its existing basename. Preserve headings, acceptance criteria, rationale and review history. Update relative links inside the moved file and all tracked repository references, including fragment links. Do not maintain a second full copy at the old path. Pinned historical Git URLs still address their original revisions; unpinned external links to the old path are not redirects.
4. Replace the opening status with “historical decision record — implemented,” “rejected,” or “superseded,” the dated outcome and links to the current owner and evidence. Explicitly state that imperative wording, examples and limitations below describe the reviewed design, not current guarantees. Later corrections to retirement metadata or links are allowed; do not revise the reviewed body to track evolving behavior.
5. Move the index entry from active to archive and the PLAN entry from discussion to completed decisions, or record rejection/supersession there. Update contributor guidance only when the procedure itself changes; it should link here rather than copy this checklist.
6. Verify inbound and outbound relative links and fragments, unchanged historical headings and acceptance text apart from link relocation, and the absence of unsupported release claims. Record the verification in the work plan and commit the retirement with the implementing checkpoint where possible. A documentation-only retirement needs link/history checks, not unrelated counting tests.

## Audit — 2026-10-16

All three pre-existing proposals were already marked historical and indexed as completed. The audit added consistent archival placement and notices, and corrected stale retirement metadata rather than reclassifying unfinished work as done. Source at tags `v0.2.0` and `v0.3.0`, the [release records](../PLAN.md#minor-release-v030--published) and the linked implementation checkpoints establish net change/subfolder availability in v0.2.0 and adaptive detail in v0.3.0. Historical body text remains intentionally frozen, including output rules superseded by the [current routing contract](../PRODUCT.md#output-routing).
