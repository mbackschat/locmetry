# Agent Regression Desk Review: Human Output Wording

Evaluated 2026-10-22 under the [agent regression checks](../agent-regression-checks.md#desk-review) by one fresh MEDIUM agent (Claude Code `sonnet` at `xhigh`), acknowledged by the owner in advance. Control: `142645e`, the last ledger entry. Candidate: `99bf4d0`, which adds explain errors that name their target (`97a022c`), the corrected comparison caveat (`da4be9e`) and the [human output wording rules](../help-design.md#human-output-wording) (`0b921ea`, `99bf4d0`). Both were built from `git archive` and run with `NO_COLOR=1`, `COLUMNS=100` and empty personal defaults against the synthetic [CLI workspace](../../examples/cli-workspace/README.md) and fixtures the reviewer created, covering inline tests, unreadable, unrecognized, ignored, excluded and symlinked files and broken notebooks. A desk review does not measure calls or tokens; sizes are bytes.

## Verdict

No correctness regression was found. All 8 help texts and all 6 schemas are byte-identical; every flag that help, guides, notes, warnings and errors name exists; 147 invocations have identical exit statuses in both revisions. Measurement, workspace and explain JSON are identical apart from notebook parse-failure `message` texts; comparison JSON differs only in finding `message` texts and in the order of tied `presence_changed` findings, which PRODUCT promises only to keep deterministic. Reports remain cross-readable in both directions.

An agent run is not needed for correctness. A targeted run of tasks 3 and 8 would give behavioral evidence for A2 and A3 and becomes more valuable if the owner changes the quoted-command rule.

## Reading Cost

| Text or output | Control bytes | Candidate bytes | Change |
| --- | ---: | ---: | ---: |
| All 8 help texts | 43,456 | 43,456 | 0 |
| All guides | 55,384 | 55,427 | +0.08% |
| All schemas | 191,886 | 191,886 | 0 |
| Default table, CLI workspace project | 1,942 | 1,945 | +0.15% |
| Verbose table, CLI workspace project | 2,837 | 2,919 | +2.9% |
| Workspace verbose table | 4,092 | 4,182 | +2.2% |
| JSON headline, root and workspace | 3,479 and 7,288 | 3,479 and 7,288 | 0 |
| Comparison table, single root | 4,767 | 4,840 | +1.5% |
| Comparison table, workspace with a member leaving | 6,197 | 6,262 | +1.0% |

The only new output lines are the second comparison caveat and the `Detail:` row of `explain`.

## Findings

| ID | Severity | Finding | Resolution |
| --- | --- | --- | --- |
| A1 | Risk, low | Commands and flags are single-quoted; an agent copying the quotes into a shell fails once, recoverably. Clap's own tips already quote flags this way. | Accepted |
| A2 | Risk, low to medium | Quoted whole commands such as `'locmetry --files --output -'` look runnable but omit the invocation's directory, `--subfolder`, `--config` and the `workspace` subcommand; run literally in a workspace's directory they measured a different scope and exited 0. | Fixed after the review by owner decision: notes quote the flags only, as in `'--files --output -'`, and the wording guard rejects quoted `locmetry` commands |
| A3 | Risk, low | The verbose note on files counted as production only by a build layout changed its advice, not only its punctuation, while the configure guide, PRODUCT and the report schema kept the old terms. | Terms aligned in `c02e19a`; behavior unmeasured |
| A4 | Risk, documentation | PRODUCT still said comparison finding messages were unchanged, and the CHANGELOG described only tables. | Fixed in `c02e19a` |
| A5 | Risk, release tooling | The release script no longer matched the new guide header `Source: … (locmetry X)` and would have left four stale versions in the recorded root guide. | Fixed test-first in `4799169` |
| A6 | Improvement | Remaining rule gaps: backticks in report-decoding errors, a `.:` seam in the user-defaults error, unquoted exclusion patterns and directory explain targets, and a workspace explain error naming both the typed and the member-relative path. | Fixed in `c02e19a`, with guard scenarios and a `.:` rule; the double naming is the deliberate behavior of `97a022c` and remains |
| A7 | Risk, low (inferred) | With commas, "total" read like a fourth composition component. | Fixed in `c02e19a`: the total is its own sentence |
| A8 | Improvement | The comparison caveat no longer says unreadable files count as 0. | — |
| A9 | Improvement | `explain` tables put the stable code in `Reason:` and the sentence in `Detail:`, counts agree in number, and errors name the target. | — |
| A10 | Improvement | Ineffective-option warnings and errors follow one pattern with an exact remedy; every remedy flag exists and exit statuses are unchanged. | — |
| A11 | Neutral | Sixteen comparison finding message patterns changed, and tied `presence_changed` findings reordered because the sort key includes the serialized message. | — |

## Limits

Debug builds only; Windows text and styled terminal output were not exercised. Fixtures are small and synthetic, and the Immich showcase was not rerun by the reviewer. The resolutions after `99bf4d0` were verified by the public suite, the wording guard and the recorded guides, not by a second review. Captures and the full report remain in the session scratch space; the [procedure](../agent-regression-checks.md#desk-review) reproduces them from the two revisions.
