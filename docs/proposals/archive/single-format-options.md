# Single-Format Options Select Their Format

<!-- doc-status
{
  "status": "implemented",
  "date": "2026-10-25",
  "summary": "Without --format and --output, an option that shapes only one format selects that format instead of warning in pipes.",
  "evidence": "[PRODUCT](../../PRODUCT.md#output-routing), [regressions](../../../tests/integration/cli_options.rs), [experiments](../../research/agent-experiments-2026-10-25.md)"
}
-->

Status: historical decision record — implemented. The owner delegated the decision on 2026-10-25, after asking for the earlier exclusion to be re-examined and for experiments; the design below was implemented test-first and verified the same day. [PRODUCT](../../PRODUCT.md#output-routing) owns the current behavior; the wording below describes the reviewed design, not current guarantees.

## Problem

Agents run without a terminal, so their commands get the compact JSON headline unless they pass `--format`. On 2026-10-25 the agents of all three projects categorizing their tests passed table options such as `--show-categories`, `--show-folders` and `--table-category` without `--format`, in four incidents. Each wanted a table, none wanted JSON, and each had read both "a JSON headline in pipes" and "Table format only" in help beforehand. They merged stderr into stdout and filtered it with `head` or `sed`; under `sed` the warning disappeared and the run exited 0 with empty output, and one agent never got the table. Failing with exit 1 instead would not have helped: the filter hides the error too, and a pipe without `pipefail` drops the status.

The [CLI option clarity record](cli-option-clarity.md) excluded "automatically choosing table format when a display flag appears" and stated that `--show-*` "never selects a format implicitly". It gave no argument specific to this; its only basis was the hypothesis that agents benefit from predictable formats, which it called unmeasured. The review evidence it cited, `--no-bars --output baseline.json` and `--sort code` in a pipe, concerned failing on ineffective options and stays valid under this design.

## Design

When neither `--format` nor `--output` is given, an option that shapes only one format selects that format, on a terminal and in a pipe alike:

| Selects | Options |
| --- | --- |
| Table | `--show-folders`, `--folder-rows`, `--show-categories`, `--show-bars`, `--no-bars`, `--verbose`, `--table-purpose`, `--table-category`, `--table-language`, `--tables-only` (measurement, workspace and compare) |
| Markdown | `--markdown-depth` |
| Full JSON | `--files` |

- Options of different formats select none; the default and today's warnings or errors apply.
- `--split-languages` and `--sort` shape tables and Markdown, so they select neither.
- `--output`, including `--output -`, keeps full JSON, and table options there still warn with the `--print table` remedy, so saved baselines never change format.
- Personal defaults and environment variables cannot set a format, so no hidden setting interacts with the rule.

One rule for every single-format option, rather than a table-only special case or an exception for `--verbose`, keeps the format predictable from the command line alone.

## Compatibility

The break: a command that passes one of these options without `--format` or `--output` and parses stdout as JSON now receives a table, Markdown or full JSON. No such use was observed in the four incidents or in the experiments; such a command passed an option that did nothing. `--files` without `--format`, which failed, now prints full JSON. Breaking changes are accepted before 1.0 (owner, 2026-10-21).

## Verification

- Regressions: [cli_options](../../../tests/integration/cli_options.rs) locks every option's selected format, the `--output` boundary, disagreeing options and `--split-languages`; unit tests lock the selection rule.
- Help stays within its byte budgets; two help examples dropped their now redundant `--format table`.
- [Experiments](../../research/agent-experiments-2026-10-25.md): the agents' filters reproduced on the synthetic workspace, and a blind A/B of six fresh agents on one table task. Two of three control agents first received the headline and spent a call; all three candidate agents got their tables on the first try.
- An [agent regression check](../../agent-regression-checks.md) before the next release remains required, since flags' defaults changed.
