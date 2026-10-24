# Agent Regression Desk Review: Fixes to the Man Page and Reference Guide

Evaluated 2026-10-25 under the [agent regression checks](../agent-regression-checks.md#desk-review) by one fresh MEDIUM agent (Claude Code `sonnet` at `xhigh`), acknowledged by the owner. Control: `28213f9`, the last ledger candidate and the subject of the [previous desk review](agent-regression-desk-review-2026-10-24.md). Candidate: `8d668ce`. The range is three commits and 18 files: the record of that review, the fixes for its findings (`c57cec3`) and the retirement of the [man page proposal](../proposals/archive/manual-and-reference.md). The fixes group positional arguments under their own heading before the options in the man page, reword the `explain --format` values, correct three passages of the [reference guide](../guides/reference.md) and lower the `compare --help` size budget. Both revisions were built with `cargo build --release --locked` from `git archive` into separate scratch directories and run with `NO_COLOR=1`, `COLUMNS=100` and an empty `XDG_CONFIG_HOME` on macOS, on the synthetic [CLI workspace](../../examples/cli-workspace/README.md). The reviewer also ran the 46 public tests that the `generate::`, `reference::`, `help::`, `queries::` and `guides::` filters select, in release mode on the candidate source. A desk review does not measure calls or tokens.

## Verdict

No regression. Of 39 captured texts, 34 are identical apart from timestamps: `-h` and `--help` of the root, `workspace` and `compare`, `explain -h`, six of the seven guides, all six schemas, and the table, folder-and-category table, `--verbose` table, Markdown, headline, full JSON, `--files`, workspace (both member styles, also with `--verbose` and as JSON), `explain` output and version for the synthetic fixture, plus the bash and PowerShell completion scripts. Exit statuses and stderr are identical in all 39. The five that differ are `explain --help`, the `reference` guide, the man page and the zsh and fish completion scripts, and each difference is an intended one: the new `explain --format` descriptions in four of them, the three reference-guide passages in two, and the argument headings in the man page. All 58 guide and schema routes that help, guides, the README and the recorded guides name run, every flag they name exists, and `--generate` is still absent from every help text, the man page and every completion script.

All six findings of the previous review are fixed. The man page now lists each command's arguments before its options, as `--help` does, and `mandoc -T lint -W warning` is silent. The Files bullet matches the behavior in all eleven output-destination cases tried, and the man path advice finds the page on macOS. The fixes leave one inaccurate fallback width, and the new `MANPATH` advice makes an older gap visible: the install recipes assume directories that a clean home lacks (N1 and N2 below). Two further points are wording only (N3 and N4).

No targeted agent run is needed. No text on an ordinary task's path changed: root, `workspace` and `compare` help, the index and every other guide are byte-identical, and `explain --help` shrank by three bytes. The `reference` guide grew 337 bytes, in sections that only index routes request (`environment` +29, `files` +141, `man-page-and-completions` +167), far below what provider cache misses already swing in a four-pair median.

## Reading Cost

| Text | Control lines / bytes | Candidate lines / bytes | Change |
| --- | ---: | ---: | ---: |
| `explain --help` | 82 / 3,339 | 82 / 3,336 | −0.1% |
| All 8 help texts | 771 / 37,058 | 771 / 37,055 | 0.0% |
| Guide `reference` | 51 / 4,235 | 51 / 4,572 | +8.0% |
| All 7 guides | 852 / 75,921 | 852 / 76,258 | +0.4% |
| All 6 schemas | 6,943 / 227,183 | identical | 0.0% |

No other help text, guide or schema changed: root `-h` stays at 71 / 4,406, root `--help` at 279 / 13,003, `workspace --help` at 147 / 6,766, `compare --help` at 96 / 4,689 and the guide `index` at 49 / 5,006. Generated documents, which no ordinary task reads: the man page grew from 822 lines (32,326 bytes) to 827 (32,552), +0.7%, from the argument headings and the longer Files and Environment text; the zsh and fish scripts lost three bytes each; bash (342 lines) and PowerShell (134) are identical.

The size budgets follow the stated rule, the measured size plus about 3% and never above the earlier budget: `compare --help` fell from 4,875 to 4,830 bytes (measured 4,689), and `workspace --help` (measured 6,766) and `explain --help` (measured 3,336) stay at their earlier 6,900 and 3,400. Root `--help` still sits 7 bytes under its 13,010 budget, so the next addition to it stops for a reviewed decision, as the guard intends.

## Findings

Findings of the [previous review](agent-regression-desk-review-2026-10-24.md#findings), checked against the candidate.

| Id | Result | Evidence |
| --- | --- | --- |
| F1 | Fixed | The Files bullet now says a measurement's file lies outside the scan root, or outside every member of a workspace, and does not overwrite the configuration or workspace file, and that a comparison's does not overwrite either input. Eleven cases agree: output inside the root, inside a member or beside the members of a folder workspace fails with exit 1; overwriting the `--config` file, the workspace file or either comparison input fails with exit 1; a comparison written inside a measured root succeeds. |
| F2 | Fixed | With a clean `HOME`, `man locmetry` finds nothing, and finds the page and still `ls(1)` once `MANPATH="$HOME/.local/share/man:"` is exported as the guide says. |
| F3 | Fixed | The man page has ARGUMENTS before OPTIONS for the root and an Arguments heading before Options under `compare` and `explain`, as `--help` orders them; `workspace` has no positional arguments and shows none. The guard test `the_man_page_lists_each_commands_positional_arguments_before_its_options` passes. |
| F4 | Fixed, with a caveat | `COLUMNS` is now documented as the help wrap width too; `--help` is 380 lines at `COLUMNS=60`, 279 at 100. See N1 for its fallback. |
| O1 | Fixed | `explain --help`, the zsh and fish scripts, the man page and the recorded explain guide say `Tables and evidence` and `Markdown tables and evidence for reviews`, as the other commands' `--format` does. No other text keeps `Human-readable tables`. |
| O2 | Fixed | The [feature inventory](../help-design.md#current-feature-inventory) has a row for exit statuses, environment variables and files; the index rows it names exist and the three sections resolve. |

New findings.

| Id | Severity | Finding | Evidence | Agent run |
| --- | --- | --- | --- | --- |
| N1 | Risk, low | The `COLUMNS` bullet ends "Otherwise the width of a stdout terminal applies, else 80", which was true for share bars and now reads as true for help as well. Help wraps at 100 columns when `COLUMNS` is unset and stdout is not a terminal, which is what an agent's pipe gives. The wording is also rendered in the man page. | Unset `COLUMNS`, piped: 279 lines, widest line 100; the 80 is the fallback of `bar_columns` in `src/app/destination.rs`, which sizes share bars | No |
| N2 | Improvement | The install recipes redirect into directories that a clean home lacks and the guide never creates them, so the first command of the section fails. This predates the range and the control has it, but the fixed `MANPATH` advice, which depends on that first step, now follows directly after it. | On a clean `HOME`, each of the four redirects (`~/.local/share/man/man1/locmetry.1`, the bash, fish and `~/.zfunc` targets) fails with "No such file or directory" before Locmetry runs, and nothing is written | No |
| N3 | Improvement | "If `man locmetry` then finds no entry" sits in the zsh sentence's paragraph, so "then" reads as a consequence of the `fpath` step instead of the man page install, and the macOS caveat is easy to miss for a reader who skips the zsh instruction. | `docs/guides/reference.md`, last paragraph before the PowerShell note | No |
| N4 | Improvement | A workspace made of project folders refuses a destination anywhere in the project root, beside the members as well, while the Files bullet says "outside every member". The error names the rule and the safe side is the same as before; it only makes the bullet less exact than the folder case. | `workspace --workspace-file folders.toml --output acme-orders/r2.json` fails with "must be outside the read-only scan root" | No |

Smaller notes needing no action: [DEFERRED](../DEFERRED.md#manual-page-and-completions) carries three of the retired proposal's six exclusions, the ones with reopening conditions; section-id and category-name completion, PowerShell value completion and the unchanged `--help` notes stay only in the archived record. The retired record's body is unchanged apart from link relocation, and its status, dates and evidence agree with the ledger, the PLAN entry and the archive index. `python3 scripts/doc-indexes.py check` reports no stale index, `git diff --check` is clean and no relative link or fragment in the 11 changed Markdown files is broken. `--generate man compare` still reports clap's missing-argument error before the standalone-query message, as in the control.

## Not Checked

Rendering by groff, which was absent, so the page was read through mandoc only; the `MANPATH` advice on Linux; completion behavior in every shell, since the candidate's bash script is identical to the control's and the zsh script had a syntax check only, while fish and PowerShell were not installed, so the retired record's PowerShell 7.6.8 check of 2026-10-25, which exists only in that archived record, was not reproduced; help wrapping on a real terminal, since no pseudo-terminal was available; the recorded command guides' own verification, as only the changed `explain` block was compared by text; release packaging; and the rest of the test suite beyond the 46 selected tests.
