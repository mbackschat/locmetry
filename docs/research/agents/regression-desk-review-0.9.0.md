# Agent Regression Desk Review: 0.9.0

Evaluated 2026-10-22 under the [agent regression checks](../../agent-regression-checks.md#desk-review) by one fresh MEDIUM agent (Claude Code `sonnet` at `xhigh`), acknowledged by the owner. Control: `99bf4d0`, the last ledger entry. Candidate: `9a46ad3`, which adds notes that quote flags only, errors that name the scan root and workspace file, `error:` and `warning:` stderr labels, and the workspace vocabulary "member" in place of "repository". Both were built from `git archive` and run with `NO_COLOR=1`, `COLUMNS=100` and empty personal defaults against a copy of the synthetic [CLI workspace](../../../examples/cli-workspace/README.md) and fixtures the reviewer created, covering Rust inline tests, an unreadable file, a broken notebook, a symlink and an ignored file. A desk review does not measure calls or tokens; sizes are bytes.

## Verdict

No correctness regression was found. 139 invocations per revision have identical exit statuses, and 70 comparison invocations agree in both reading directions. 107 JSON documents are identical apart from timestamps; the only schema difference is the `largest_module_files` description. Every flag, guide topic and schema name in help, guides, notes and errors exists. The candidate's help, wording, guides and README test filters and the release-script tests pass.

An agent run is not needed. Tasks 3 and 8 could give behavioral evidence for B2, but in the 0.8.0 run agents in those tasks ran `--verbose` once and did not act on the notes.

## Reading Cost

| Text or output | Control bytes | Candidate bytes | Change |
| --- | ---: | ---: | ---: |
| All 8 help texts | 43,456 | 43,448 | −0.02% |
| All 4 guides | 55,427 | 55,451 | +0.04% |
| All 6 schemas | 191,886 | 191,882 | 0 |
| Verbose table, CLI workspace project | 2,842 | 2,815 | −0.9% |
| Workspace verbose table, independent roots | 4,069 | 3,999 | −1.7% |
| Workspace Markdown | 6,034 | 6,004 | −0.5% |
| JSON headline, root and workspace | 3,395 and 7,164 | 3,395 and 7,164 | 0 |
| Comparison tables, root and workspace | 4,907 and 5,151 | 4,907 and 5,151 | 0 |

Help line counts are unchanged; the only help edits are the workspace example title and "per-member totals".

## Findings

| ID | Severity | Finding | Resolution |
| --- | --- | --- | --- |
| B1 | Improvement | Notes quote flags only, as in `'--files --output -' lists each file's kind`, which resolves [A2](regression-desk-review-2026-10-22.md#findings). | — |
| B2 | Risk, low | Flags-only notes leave the scope to the reader: `--files --output -` run alone in another directory measures that directory, and appending the note's `'--guide configure'` to a workspace command fails with `error: unexpected argument '--guide' found`, because `--guide` is a root-level query. Each costs one recoverable call. | Accepted; optional tasks 3 and 8 |
| B3 | Improvement | Workspace output says `Member`, `## Members`, "by member", `Members to review:` and `Unclaimed source by member:`, matching JSON `summary.members` and comparisons; figures, columns and order are unchanged. | — |
| B4 | Risk, documentation | "Repository" remained beside "member" in the reports guide's workspace section and default-output paragraph, PRODUCT's table paragraphs and the recorded workspace guide's prose, and the guide never said a member is a root or a project folder. | Fixed after the review |
| B5 | Neutral | Stderr labels changed from `locmetry:` to `error:` or `warning:` with unchanged exit statuses; the CHANGELOG entry was not marked **Breaking** although a script matching `locmetry:` would stop matching. | Marked **Breaking** after the review |
| B6 | Improvement | Errors name the typed root, workspace file and exclusion pattern, as in `error: Cannot open scan root 'nonexistent': …`. | — |
| B7 | Neutral | Residual gaps unchanged by the candidate: some workspace member errors show canonical absolute paths, `Subfolder target must be a directory` and `Output must be outside the read-only scan root` name no input, and clap suggests `'--show-bars'` for `workspace --show-folders`. | Fixed after the review |
| B8 | Improvement | The note, configure guide and report schema now share the terms for files counted as production only by a build layout, resolving A3's terminology. | — |
| B9 | Neutral | Comparison composition lines state the total as its own sentence; finding messages and JSON are unchanged. | — |
| B10 | Neutral | The release script matches the guide header `Source: … (locmetry X)`; its tests pass. | — |
| B11 | Neutral | Workspace test names still said "repository". | Renamed after the review |

## Limits

Debug builds on macOS only; Windows text and styled tables were not exercised, and styling was checked only for the `error:` label. Fixtures are small and synthetic, and the Immich showcase was not rerun by the reviewer. The reviewer ran filtered tests, not the full suite or the recorded-guide replay. B2 and B4 are inferences from output, not observed behavior. The resolutions after `9a46ad3` were verified by the public suite and the recorded-guide replay, not by a second review. Captures remain in the session scratch space; the [procedure](../../agent-regression-checks.md#desk-review) reproduces them from the two revisions.
