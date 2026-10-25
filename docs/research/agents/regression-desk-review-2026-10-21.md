# Agent Regression Desk Review: 0.7.0 to 0.8.0

Evaluated 2026-10-21 under the [agent regression checks](../../agent-regression-checks.md#desk-review) by one fresh MEDIUM agent (Claude Code `sonnet` at `xhigh`). Control: tag `v0.7.0`. Candidate: `dc2bf33`, release 0.8.0 plus the inline-test legend. Both were built from `git archive` and run with `NO_COLOR=1`, `COLUMNS=100` and empty personal defaults against the synthetic [CLI workspace](../../../examples/cli-workspace/README.md) and the benchmark's task fixtures. A desk review does not measure calls or tokens; sizes are bytes.

## Verdict

No correctness regression was found. Every flag, value and command that help, guides, schemas, examples and error remedies name exists in the candidate; all 25 commands adapted from help examples and 4 commands for new features exit 0; no stale reference to removed or renamed text remains. The execution-first routing text adopted from the [full-help study](explorability-full-help-study.md) is unchanged.

Reading cost grew, mostly in the guides, and two changes land on tasks the benchmark measures. They justify a targeted agent run; nothing found blocks use.

## Reading Cost

| Text or route | v0.7.0 bytes | dc2bf33 bytes | Change |
| --- | ---: | ---: | ---: |
| Root `--help` | 12,585 | 13,582 | +7.9% |
| Root `-h` | 6,394 | 7,354 | +15.0% |
| `workspace -h` | 3,721 | 4,673 | +25.6% |
| `compare -h` | 1,090 | 1,560 | +43.1% |
| All help texts | 38,543 | 43,576 | +13.1% |
| Compare guide | 8,710 | 13,930 | +59.9% |
| Configure guide | 7,552 | 12,250 | +62.2% |
| All guides | 43,790 | 55,384 | +26.5% |
| All schemas | 185,477 | 191,299 | +3.1% |
| Comparison route: root and compare help plus compare guide | 25,545 | 32,196 | +26.0% |
| Configuration route: root help plus configure guide | 20,137 | 25,832 | +28.3% |

## Findings

| ID | Severity | Finding | Agent run |
| --- | --- | --- | --- |
| F1 | Risk | `--verbose` now lists the largest module-claimed production files as needing an entry if generated or vendored; it appears for 13 of 15 benchmark project roots and lists exactly the inferred production that configuration tasks 3 and 8 ask agents to retain. | Tasks 3 and 8 |
| F2 | Risk | The compare guide gained the file-transition section and the configure guide a verified review workflow; in the full-help study, comparison task 4 had failed the call gate before its correction and all candidate agents read the compare guide. | Task 4, with tasks 3 and 8 |
| F3 | Risk, low | Help texts grew 13.1% overall while routing text stayed the same. | Measured within F1 and F2 |
| F5 | Risk | Tables fold TSX into TypeScript while JSON keeps TSX, which changes a per-language Test/prod; no benchmark fixture has a language variant, so existing tasks cannot detect a misreading. | Optional, needs a new task variant |
| F8 | Risk, low | The [feature inventory](../../help-design.md#current-feature-inventory) lacks entries for several new features, such as `--table-language`, `--tables-only`, `inline_tests`, `largest_module_files` and project-folder workspaces; no test ties the inventory to help. | No |
| F9 | Risk, low | Some routes are thin: `explain -h` still says "one existing root-relative file" although it takes several paths, the workspace summary names only roots with their own `.locmetryrc`, and guides link the cookbook and helper by repository paths a binary-only agent cannot open. | No |
| F10 | Risk, low | v0.7.0 rejects candidate reports with `inline_tests`; reports are not forward compatible, as the schema version already states. | No |
| F11 | Pre-existing | Clap's tips for the 0.7.0 renames mislead: `--purpose` suggests `--verbose`, and `--folders` suggests `--folder-rows`. | No |
| F4, F6, F7, F12 | Improvement | Links resolve; the inline-test legend, the `rust-test-module` convention and "Not shown" notes with excluded code add information; `--tables-only` conflicts are reported clearly. | No |

## Limits

The control v0.7.0 was itself never checked: the last measured revision, `c17fece`, predates the 0.7.0 option renames. Fixtures are small and synthetic, and Windows-specific text was read but not exercised. Captures and the full report remain in the session scratch space, not in the repository; the [procedure](../../agent-regression-checks.md#desk-review) reproduces them from the two revisions.
