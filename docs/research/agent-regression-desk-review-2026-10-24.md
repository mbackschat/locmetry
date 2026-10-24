# Agent Regression Desk Review: Man Page, Completions and Reference Guide

Evaluated 2026-10-24 under the [agent regression checks](../agent-regression-checks.md#desk-review) by one fresh MEDIUM agent (Claude Code `sonnet` at `xhigh`), acknowledged by the owner. Control: `47a1081`, the last ledger candidate. Candidate: `28213f9`. The range adds the hidden `--generate KIND` query (the locmetry(1) man page and bash, zsh, fish and PowerShell completions), the `reference` guide with its index rows, reworded `--format` value descriptions and a re-indented TOML example in `workspace --help`; the remaining commits change showcase policy, repository tooling and an internal policy-audit refactor. Both revisions were built with `cargo build --locked` from `git archive` into separate scratch directories and run with `NO_COLOR=1`, `COLUMNS=100` and an empty `XDG_CONFIG_HOME` on macOS, on the synthetic [CLI workspace](../../examples/cli-workspace/README.md) and on stress policies the reviewer wrote. A desk review does not measure calls or tokens.

## Verdict

No regression. Default, `--verbose`, Markdown, headline, full JSON and `--files` outputs are identical apart from timestamps, for single roots, workspaces and shared-policy workspaces, and the policy-audit refactor gave byte-identical output on four inputs, including one with five unused entries. All 51 guide and schema routes named in help, guides and README run; every flag a guide names exists. `--generate` appears in no help, no `help` output, the man page or any completion script, rejects every combination with a "standalone query" error and leaves stdout empty on failure. `mandoc -T lint` reports no warnings, and bash completion offers commands, per-command options and values as expected.

No targeted agent run is needed. The only change on every task's reading path is `, reference` in the `--guide` value list (+11 bytes in root `-h`, −2 in `--help`); the largest is the guide index (+429 bytes), far below what provider cache misses already swing in a four-pair median.

## Reading Cost

| Text | Control lines / bytes | Candidate lines / bytes | Change |
| --- | ---: | ---: | ---: |
| Root `-h` | 71 / 4,395 | 71 / 4,406 | +0.3% |
| Root `--help` | 278 / 13,005 | 279 / 13,003 | 0.0% |
| `workspace --help` | 147 / 6,844 | 147 / 6,766 | −1.1% |
| `compare --help` | 96 / 4,731 | 96 / 4,689 | −0.9% |
| All 8 help texts | 770 / 37,169 | 771 / 37,058 | −0.3% |
| Guide `index` | 45 / 4,577 | 49 / 5,006 | +9.4% |
| Guide `reference` | none | 51 / 4,235 | new |
| All guides | 797 / 71,257 | 852 / 75,921 | +6.5% |
| All 6 schemas | 6,943 / 227,183 | identical | 0.0% |

The other `-h` texts and `explain --help` are unchanged. Generated documents, which no ordinary task reads: man page 822 lines (32,326 bytes), bash 342, zsh 299, fish 141 and PowerShell 134 lines. Root `--help` sits 7 bytes under its size budget; the `workspace` and `compare` budgets were not lowered after their help shrank.

## Findings

| Id | Severity | Finding | Evidence |
| --- | --- | --- | --- |
| F1 | Risk, low | The reference guide's Files section says reports and comparisons may only be written outside every measured root, but a comparison may write inside a root and only refuses to overwrite its inputs ([PRODUCT](../PRODUCT.md)). The error is on the safe side: an agent would avoid a legal destination. | `docs/guides/reference.md`, last Files bullet, also rendered in the man page; `compare … --output web/inside-root.json` exits 0 |
| F2 | Risk, low | The guide's first man-page install recipe, `~/.local/share/man/man1/locmetry.1`, is not found by `man` on stock macOS without `MANPATH`; the guide notes zsh's `fpath` but not this. Homebrew and the Linux packages install system paths and are unaffected. | `man locmetry` with a clean HOME: "No manual entry for locmetry" |
| F3 | Improvement | The man page lists positional arguments (`[DIR]`, `<BEFORE>`, `<AFTER>`, `<PATH>...`) among the options, while `--help` gives them an Arguments heading; nothing is lost, and the synopsis shows them. | clap_mangen 0.3.3 groups by help heading |
| F4 | Improvement | The Environment section says `COLUMNS` sets the share-bar width but not that it also sets the help wrap width. | `--help` is 380 lines at `COLUMNS=60`, 279 at 100 |
| O1 | Improvement | `explain --help` keeps the old `--format` wording ("Human-readable tables and evidence"); the other commands use the new one. | `src/app/cli.rs`, `ExplainFormat` |
| O2 | Improvement | The [feature inventory](../help-design.md#current-feature-inventory) has no destination for the reference guide's exit-status, environment and files sections. | `docs/help-design.md` |

Smaller notes needing no action: `--generate man workspace` without `--root` reports clap's missing-argument error before the standalone-query message, as `--schema config workspace` does on the control; the page's date is that of the current Cargo.toml version and moves with the next release.

## Not Checked

zsh, fish and PowerShell completion behavior (zsh had a syntax check only; the [manual completion check](../tokei-integration.md#man-page-and-completion-check) covered zsh and fish on 2026-10-24); groff rendering, since groff was absent; release packaging paths; and the test suite, whose help-size guard the reviewer replaced by direct measurement.
