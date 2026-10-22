# Comparison Presence Summary

<!-- doc-status
{
  "status": "draft",
  "date": "2026-10-22",
  "summary": "Bounded presence findings in human comparisons: contained folders and packages fold into their outermost changed folder or member, and a cap ends the list with a count.",
  "evidence": ""
}
-->

This proposal answers the [PLAN](../PLAN.md#comparison-presence-findings--follow-up) item to evaluate a bounded human summary for large restructures. The [comparison readability checkpoint](../plans/archive/measurement-quality-and-net-change.md#comparison-readability--2026-10-16) left it open as a separate decision.

## Problem

Human comparisons list one `presence_changed` finding per folder, package and member that appears in only one report. Moving a folder `old` with 40 npm packages to `new`, measured on 2026-10-22 with a synthetic fixture, prints 162 such findings: 324 of the table's 360 lines. The one `modules_changed` finding above them, and the settings caveat after them, are easy to miss. The `Only in one report:` line already counts presence by kind and side, and headline JSON keeps only `presence_summary`, but the table and Markdown keep every line.

## Design

The change affects the Findings section of table and Markdown comparisons only. JSON, full and headline, keeps every finding, field and message, so it remains the complete evidence.

**Required: fold contained presence.** A folder or package finding is not listed on its own when its path lies inside a folder, or inside a member, that has a presence finding on the same side. The outer finding names what it contains:

```text
- presence_changed (before): Folder 'old' appears only in the before report, with 40 folders and 40 packages inside it. It may be deleted or no longer measured.
- presence_changed (after): Member 'worker' joined the workspace measurement, with 12 folders and 1 package inside it.
```

Containment uses the recorded identities the findings already carry: folder paths, member-prefixed in workspaces, and a package's folder. A finding with nothing inside it keeps its present wording.

**Required: cap the remaining list.** After folding, at most 10 presence findings are listed per side, in the present order. One closing line per side counts the rest:

```text
- presence_changed (before): 31 more folders and 2 packages appear only in the before report. '--format json' lists each one.
```

The other findings are listed in full, before the presence findings. Wording follows the [human output wording rules](../help-design.md#human-output-wording).

**Optional:** a smaller cap in table output than in Markdown. The proposal uses 10 for both, so a saved Markdown review matches what the terminal showed.

**Excluded:**

- Detecting moves by pairing before-only and after-only entities with equal counts. This is a heuristic that could misreport a deletion plus an unrelated addition as a move.
- Any change to JSON, `presence_summary`, finding codes or exit statuses.
- A flag to show every presence line. JSON already lists them, and the closing line names it.

## Rationale

Folding handles the common case. Moving or deleting a subtree changes presence for every folder and package under it, but the reader needs only the subtree's root, and the counts keep the scale visible. The cap bounds what folding cannot reduce, such as 200 sibling packages renamed under a folder that exists on both sides. A cap without folding would list 10 children of the moved folder and hide the folder that explains them. Dropping presence lines entirely, as headline JSON does, would remove the names that tell a reader where the change happened.

## Acceptance

- The moved-subtree fixture above lists one folder finding per side, naming 40 folders and 40 packages, and the table shrinks from 360 lines to about 40.
- A workspace member that leaves or joins absorbs its folders and packages.
- 25 sibling folders that appear only after list 10 lines and a closing line that counts 15 more.
- Full JSON for each fixture is byte-identical before and after the change, apart from `measured_at`.
- The wording guard gains a scenario with folded and capped presence.
- [PRODUCT](../PRODUCT.md) and the [compare guide](../guides/compare.md) describe the bounded human list and point to JSON for every identity.
