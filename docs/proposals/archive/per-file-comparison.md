# Per-file Comparison

<!-- doc-status
{
  "status": "rejected",
  "date": "2026-10-26",
  "summary": "compare --files would report which files changed classification, counts or recorded state; rejected because Codex kept its own join and failed the cost gate.",
  "evidence": "[agent run](../../research/agents/regression-run-2026-10-25-per-file.md), [REJECTED](../../REJECTED.md#tested-and-rejected)"
}
-->

Status: historical decision record — rejected. Accepted on 2026-10-25 under the owner's delegation of the agent-efficiency work and implemented test-first on the branch `per-file-comparison`, it was rejected on 2026-10-26 under the same delegation and never merged: in the [agent run](../../research/agents/regression-run-2026-10-25-per-file.md), Codex agents kept their own join of the two reports and cross-checked it against `compare --files`, failing the uncached-input gate in both tasks at level correctness and calls. [REJECTED](../../REJECTED.md#tested-and-rejected) keeps the reopening condition. The design below describes the reviewed proposal, not current behavior.

## Problem

Reviewing a policy change means asking which files moved between categories, and saved-report comparison answers only in aggregates. Every agent that reviewed a policy change wrote its own join of two `--files` reports: three Codex sessions on workspaces, where the [file-transition helper](../../../scripts/compare-locmetry-files.sh) does not apply, and all seven Codex and Sonnet sessions on a single root, where it does ([workspace round](../../research/agents/feedback-2026-10-25.md#findings-as-verified), [field-sized experiment](../../research/agents/feedback-2026-10-25-b.md#experiment-2-a-field-sized-project)).

## Required

`locmetry compare BEFORE AFTER --files` adds per-file evidence. Both reports need file records; otherwise the command fails, naming the report without them and the capture command `locmetry --files --output FILE`. Without `--files`, comparison is unchanged and works without file records.

**Matching.** Paths match over the union of both reports' recorded paths: counted files and the coverage lists `unrecognized`, `unread` and `parse_failures`. Workspace paths keep their member prefix, so matching a path also matches its member; `--member` keeps the member's paths. A subfolder restriction must match, as for aggregate comparison.

**States.** Each side of a path is one of `counted`, `unrecognized`, `unread`, `parse_failed` or `not_recorded`. A counted side carries `language`, `kind`, `purpose`, `category`, `rule`, the optional `refinement`, `language_mapping` and `language_correction`, and `counts`; other states carry no counts, because unavailable counts are not zero. A path recorded on one side only shows that side's evidence, without claiming the file was created or deleted. Renames are two unrelated paths, and identical counts do not prove identical content.

**JSON.** Full JSON adds:

```json
{
  "file_summary": {
    "unchanged": 290,
    "changed": 30,
    "transitions": [
      {"before": "test:test", "after": "test:law", "files": 24, "code": 1032}
    ],
    "count_changed": {"files": 3, "code": 12},
    "counted_before_only": {"files": 0, "code": 0},
    "counted_after_only": {"files": 2, "code": 40},
    "state_changed": 1
  },
  "files": [
    {
      "path": "core/src/test/java/LedgerLawsTest.java",
      "before": {"state": "counted", "language": "Java", "kind": "source", "purpose": "test", "category": "test", "rule": "builtin:gradle-test-source-set",
                 "counts": {"code": 43, "comments": 0, "blanks": 9, "lines": 52}},
      "after": {"state": "counted", "language": "Java", "kind": "source", "purpose": "test", "category": "law", "rule": "builtin:gradle-test-source-set", "refinement": "law-tests",
                "counts": {"code": 43, "comments": 0, "blanks": 9, "lines": 52}},
      "delta": {"code": 0, "comments": 0, "blanks": 0, "lines": 0}
    }
  ]
}
```

- `files` lists only paths whose two sides differ, sorted by path; `delta` is present when both sides are counted.
- `file_summary.transitions` groups paths counted on both sides whose `purpose:category` changed, with their after code, largest code first. `count_changed` covers paths counted on both sides whose counts differ, with the net code change. `counted_before_only` and `counted_after_only` give the files and code of paths counted on one side and not recorded on the other. `state_changed` counts paths whose state changed otherwise, such as from `unrecognized` to `counted`. `unchanged` counts paths recorded identically, and `changed` the paths in `files`, each once.
- The headline keeps `file_summary` and omits `files`.

**Human output.** Table and Markdown add a File transitions grid, rows `before → after` with Files and Code, followed by one line: unchanged files, files with count changes and their code change, files counted on one side only, and a pointer to `--format json --files` for the per-file list.

**Format selection.** Like the measurement's `--files`, `compare --files` selects full JSON when neither `--format` nor `--output` decides; with an explicit table or Markdown format it adds the grid.

## Excluded

- Rename and content-change detection, and Git.
- Per-file lists in human output, which grow with the project; JSON carries them.
- Changes to the file-transition helper, which keeps working; retiring it is a separate decision.

## Acceptance

- Missing file records fail with the named report and remedy; aggregate comparison without `--files` is unchanged.
- States on both sides, including unavailable ones, with `null`-free JSON: absent fields rather than invented zeros.
- Classification-only changes appear with a zero delta; transitions, one-sided counts and count changes reconcile with the aggregate source and category changes.
- Workspace inputs match by member-prefixed path, and `--member` restricts files and summary.
- The compare guide's configuration-transition recipe and the configure guide route to `compare --files`; a targeted agent regression check with a per-file review task precedes the release.
