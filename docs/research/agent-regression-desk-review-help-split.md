# Agent Regression Desk Review: Help Split and Category Filter

Evaluated 2026-10-22 under the [agent regression checks](../agent-regression-checks.md#desk-review) by one fresh MEDIUM agent (Claude Code `sonnet` at `xhigh`), acknowledged by the owner. Control: `9a46ad3`, the last ledger candidate. Candidate: `3bec0a3`, which adds `--table-category`, one-line `-h` summaries with the full text kept in `--help` and lowered help budgets, the corrected `test` purpose description in the schemas, the configure guide's section on test support and verification tooling, and "categories" in counted notes. Both were built from `git archive` and run with `NO_COLOR=1`, `COLUMNS=100` and empty personal defaults against a copy of the synthetic [CLI workspace](../../examples/cli-workspace/README.md) and a partial fixture the reviewer created. A desk review does not measure calls or tokens; sizes are bytes.

## Verdict

No correctness regression was found. All 161 of 235 invocations that do not use the new option have identical exit statuses, and their outputs other than help, guides and schemas are identical once timestamps and versions are normalised, including headline and full JSON, Markdown, both workspace forms, comparisons and `explain`. Reports remain cross-readable in both directions. Every flag, value and command named in help, guides, notes, warnings and remedies exists. No default output changed.

No agent run is needed. Across 310 admissible subject sessions of the earlier campaigns, agents made 535 help calls and none used `-h`, so the 40% shorter `-h` does not reach that population; only `--help`, which kept its full text, does. A run of tasks 5, 7 and 10 would be optional insurance for the shorter command list and footer.

## Reading Cost

| Text | Control bytes | Candidate bytes | Change |
| --- | ---: | ---: | ---: |
| Root `-h` | 7,383 | 4,323 | −41.4% |
| Root `--help` | 13,611 | 12,560 | −7.7% |
| `workspace -h` / `--help` | 4,705 / 6,687 | 2,425 / 6,844 | −48.5% / +2.3% |
| `compare -h` / `--help` | 1,560 / 4,684 | 959 / 4,731 | −38.5% / +1.0% |
| `explain -h` / `--help` | 1,526 / 3,292 | 1,418 / 3,339 | −7.1% / +1.4% |
| All 8 help texts | 43,448 | 36,599 | −15.8% |
| All 4 guides | 55,451 | 56,069 | +1.1% |
| All 6 schemas | 191,882 | 192,470 | +0.3% |

A typical session reads root `--help` and one command's `--help`; that read shrank 4.4% to 5.9%. The control column measures `9a46ad3`, so it differs slightly from sizes quoted before `--table-category` was added.

## Help Usage in Earlier Campaigns

The Git-ignored session records of eleven admissible campaigns from 2026-10-20 to 2026-10-22, all Codex `gpt-6.1-sol` at `xhigh`, were counted from the launcher's call logs.

| Invocation | Calls | Sessions |
| --- | ---: | ---: |
| `locmetry --help` | 310 | 310, always the first call |
| `locmetry -h` | 0 | 0 |
| `workspace --help` | 60 | 60 |
| `compare --help` | 85 | 85 |
| `explain --help` | 80 | 79 |
| Any command's `-h`, or `locmetry help` | 0 | 0 |

## Findings

| ID | Severity | Finding | Resolution |
| --- | --- | --- | --- |
| C1 | Neutral | Outside the intended changes, exit statuses and outputs are unchanged. | — |
| C2 | Improvement | Every `-h` keeps the pointer `Print help (see more with '--help')`, "standalone query" on `--schema` and `--guide`, and the root output defaults. | — |
| C3 | Risk, low | `-h` drops "Table format only" from six table options and "headline omits them" from `--files`. Each misuse warns or fails naming the remedy, and `--help` keeps the text. | Accepted: no `-h` use observed |
| C4 | Risk, low | The `-h` line of `--table-category` did not say "comma-separated", unlike its siblings. | Fixed after the review |
| C5 | Risk, low | `workspace -h` no longer says that roots in a workspace file are relative to it or that member folders prefix paths; `--help` does. | Accepted |
| C6 | Neutral | The root command list is one line per command; each command's `--help` keeps the details. | Accepted |
| C7 | Neutral | The `--help` footer's filter sentence named JSON but not Markdown. | Fixed after the review |
| C8 | Neutral | `--schema` long help lost "do not combine it with measurement options"; the combination still fails with a clear error. | Accepted |
| C9 | Improvement | `--table-category` behaves as documented in every form and combination tried, including workspaces, singular and plural notes, and the Test/prod replacement. | — |
| C10 | Risk, low | An unmatched or mistyped category yields an all-zero table without a warning; the note lists the real keys. This was the documented design, as for an unmatched language. | Fixed after the review: such an entry now warns on stderr |
| C11 | Neutral | Option errors are specific; a repeated `--table-category` is unioned, as repeated values of the other table filters are. | — |
| C12 | Neutral | Non-table formats warn about `--table-category` with the shared wording. | — |
| C13 | Improvement | The README option table did not list `--table-category`. | Fixed after the review |
| C14 | Improvement | Schemas and the configure guide agree that runners, harnesses, graders and CI gates are production tooling. | — |
| C15 | Neutral | Only `category` changed plural form. | — |
| C16 | Neutral | The gaps of the [0.9.0 review](agent-regression-desk-review-0.9.0.md#findings), B2 and B7, are unchanged and tracked in [PLAN](../PLAN.md); standalone-query errors such as `--guide is a standalone query and cannot be combined with --verbose` left both flags unquoted. | Quoting and B7 fixed after the review; B2 stays accepted |
| C17 | Neutral | `locmetry help -h` is rejected by clap in both revisions. | — |

## Limits

Debug builds on macOS only; colour, Windows text and widths other than 100 columns were not exercised. Fixtures are synthetic; the reviewer ran filtered tests and the documentation examples, not the full suite or the recorded-guide replay. The help-usage counts cover one model family whose prompts left discovery open, so they say nothing about agents of other families, which might use `-h`. C3, C4, C5 and C10 are inferences from text, not observed behavior. The resolutions after `3bec0a3` were verified by the public suite and the recorded-guide replay, not by a second review.
