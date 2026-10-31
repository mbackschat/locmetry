# Agent Regression Desk Review: 0.12.0

Evaluated 2026-10-31 under the [agent regression checks](../../agent-regression-checks.md#desk-review) by one fresh MEDIUM agent (Claude Code `sonnet` at `xhigh`), acknowledged by the owner. Control: `b8222e4`, the last ledger candidate. Candidate: `51d53b2`, `main` before release 0.12.0. Both revisions were exported with `git archive`, built with `cargo build --release --locked` one after the other and captured on macOS with `NO_COLOR=1`, `COLUMNS=100` and an empty `XDG_CONFIG_HOME`; the fixture `examples/cli-workspace` is byte-identical in both. The session used 205,602 tokens, 70 tool calls and 17 minutes.

## Verdict

No regression in anything agents rely on, and no targeted agent run is needed. Of 40 captured texts, 24 are byte-identical apart from version strings, paths and timestamps, and the other 16 differ only as intended: a 56-byte addition to the `--table-category` option text in root and `workspace --help`, one index row, one configure-guide example, one reports-guide recipe, three schema descriptions, a blank line before table legends and the new counter name. Exit statuses and stderr are identical in all 40. All 51 distinct `--guide` and `--schema` routes named in help, guides, README and PRODUCT run with exit 0 and empty stderr, no help or guide names a flag that does not exist, all 17 root-help examples run against the fixture, and the new file-size gate recipe, extracted from the served guide, exits 0 within its limits and 1 with a file list beyond them.

One risk was fixed before the release (D1): a wholly empty `--table-category` entry selected everything silently, where the control rejected it. The only text added on every agent's path is 56 bytes in root `--help`, far below the swing one provider cache miss causes in a four-pair median ([lesson 11](../../agent-lessons.md)).

## Reading Cost

Bytes are `wc -c`, the measure of `help_stays_within_its_reading_budget`.

| Text | Control lines / bytes | Candidate lines / bytes | Change |
| --- | ---: | ---: | ---: |
| `--help` (root) | 280 / 13,140 | 281 / 13,196 | +0.43% |
| `workspace --help` | 148 / 6,843 | 149 / 6,899 | +0.82% |
| Guide `index` | 49 / 5,006 | 49 / 5,027 | +0.42% |
| Guide `configure` | 205 / 17,703 | 205 / 17,657 | −0.26% |
| Guide `reports` | 216 / 19,098 | 228 / 19,746 | +3.39% |
| Section `reports --section continuous-integration` | 25 / 1,657 | 37 / 2,305 | +39.1% |
| Schema `config` | 872 / 36,997 | 872 / 37,008 | +0.03% |
| Schema `report` | 2,263 / 76,349 | 2,263 / 76,380 | +0.04% |
| Schema `comparison` | 2,520 / 80,699 | 2,520 / 80,742 | +0.05% |
| Schema `explanation` | 676 / 20,964 | 676 / 20,972 | +0.04% |
| All 8 help texts | 774 / 37,472 | 776 / 37,584 | +0.30% |
| All 7 guides | 855 / 78,355 | 867 / 78,978 | +0.80% |
| All 6 schemas | 6,945 / 229,052 | 6,945 / 229,145 | +0.04% |

Root `-h` (72 / 4,573), `workspace -h`, `compare -h`, `compare --help`, `explain -h`, `explain --help`, the `selection`, `compare`, `categories` and `reference` guides, the `user-config` and `workspace` schemas and the two sections that root help routes to are unchanged. Headroom against the budgets: root `--help` 344 bytes, `workspace --help` 101, `workspace -h` 22, the others unchanged.

## Findings

| Id | Severity | Evidence | Outcome |
| --- | --- | --- | --- |
| D1 | Risk | `--table-category ''`, `'test,'` and `',test'` exit 0 with the full table; the control exits 1 with `Unknown purpose ''`. An empty purpose part matches every purpose, so `CategoryPattern::parse` returned before its "An entry is empty" arm, which no test reached. A stray comma or an empty shell variable widened a filter without a warning, the silent no-op of lessons 1 and 2. | Fixed: a wholly empty entry is a usage error again, guarded by `table_category::empty_entries_are_rejected`, and PRODUCT states it. |
| D2 | Improvement | The unmatched-category warning names the measured categories; the control's pointer to `--show-categories` printed an empty table under the same filter. The list is uncapped, 1,552 bytes for 60 categories. | Recorded |
| D3 | Improvement | The new spellings `:tool*`, `:*tool*`, `test:`, `production:` and `:` select as documented in root and workspace tables, and the errors for `*law` and `x*:law` suggest spellings that run. | Recorded |
| D4 | Improvement, low | The bare-entry hint `write ':X'` also fires for purpose typos such as `Production`, `prod` or `tests`; the valid purposes come first in the same message, so the remedy stays correct. | Recorded, unchanged |
| D5 | Improvement | Only added blank lines separate tables from legends in the default, verbose, category, folder, workspace and comparison tables; `--tables-only`, `explain --format table` and Markdown are byte-identical. | Recorded |
| D6 | Risk, low, documented | Counts move with the fork; the fixture's counts are identical, so only a real project shows it. On Locmetry's own source, Rust comments fall from 2,950 to 2,417 and code rises from 34,158 to 34,687, and Rust lines now equal `wc -l` (38,803), which the control undercounted. `compare` reports `counter_changed` with its note. | Recorded; the Breaking changelog entry covers it |
| D7 | Improvement | The CI file-size gate is served by the executable and routed from `--guide index`; at limits of 3 it names the three files over them and exits 1. | Recorded |
| D8 | Improvement, low | The [feature inventory](../../help-design.md#current-feature-inventory) lagged: no mention of empty-part entries, the measured-categories warning or the file-size gate. | Fixed in the inventory rows |
| D9 | Improvement, low | The `workspace --help` budget was raised from 6,900 to 7,000 on 2026-10-27 without an agent check; this review is the growth review the guard asks for (+56 bytes, one sentence, no new route). | Budget kept; its comment points here |
| D10 | Improvement, low | The configure guide called the `.cls` misreading "Visual Basic 6", while the executable reports `VB6/VBA`. | Fixed |
| D11 | Note, pre-existing | `locmetry help -h` exits 1 with `unrecognized subcommand` on both sides; the reports guide links `../cookbooks/file-size-gate.md`, which an installed binary cannot serve, though its recipe is now self-contained. | Recorded, unchanged |

## Limits

All captures are macOS. The reviewer ran 106 public tests of the affected modules, not the full suite, and not the documentation scripts; the release command runs those. Counts were re-measured only on Locmetry's own source, shell completions were captured but not exercised, and the man page was not rendered.
