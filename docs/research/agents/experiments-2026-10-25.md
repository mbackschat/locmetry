# Experiments on the Categorization Feedback, 2026-10-25

Three projects categorized their tests with Locmetry 0.11.0 and reported friction: a12-rulekit, bpmn-lean-experiment and a private Lean project. Their agents then answered follow-up questions from their session evidence. These experiments checked the claims before deciding. They are informal: Claude agents in this repository's session, not the Codex runner and frozen protocol of the [agent regression checks](../../agent-regression-checks.md), and the samples are small. Private projects' figures stay in the Git-ignored local folder.

## Table options in pipes

The agents had piped table options such as `--show-categories` into `head` or `sed` with stderr merged and received the JSON headline. On the [synthetic workspace](../../../examples/cli-workspace/README.md) with 0.11.0:

| Pattern | Result |
| --- | --- |
| `--show-categories --no-bars 2>&1 \| head` | Two warnings, then the headline as one JSON line of about 3 KB |
| `--table-category test:test --show-folders --tables-only 2>&1 \| sed -n '/Folder/,+14p'` | Empty output, exit 0 |
| `--show-folders --folder-rows 6 2>&1 \| sed -n '/Folder/,$p'`, an option that already fails | Empty output too; only the unseen exit status is 1 |
| `--show-categories --output b.json` | Full JSON saved, warning names `'--print table'` |

Every table-only option warned alone in a pipe; `--sort` is silent because Markdown uses it too. No environment variable or personal default sets the format.

A blind A/B then gave six fresh agents (Sonnet at `xhigh`, no terminal) the same task: print the category table and the test-only folder table of a copy of the workspace, learning only from Locmetry's own help. Control was `44c5c9f`, the candidate the [single-format option rule](../../proposals/archive/single-format-options.md).

| Arm | Agents | First table command without `--format` | Extra calls from it | Correct tables |
| --- | ---: | --- | ---: | --- |
| Control | 3 | 2 of 3, each answered with the headline and a warning | 2 | 3/3 |
| Candidate | 3 | 3 of 3, each answered with the table | 0 | 3/3 |

All six agents also flagged a language row with 0 files and 1 code line: JavaScript embedded in an HTML file. Nothing in the table or `--verbose` notes explains it, and two agents guessed wrong causes.

## Content predicates for categories

a12-rulekit defines test lanes by JUnit `@Tag` annotations; its agent found that a regex finds them exactly only when anchored to the start of a line, since unanchored matches also hit comments and code that mention a tag. bpmn-lean-experiment separates theorem files from definition files. A read-only measurement of its Lean sources, counting declaration lines, found that a large share of the code sits in files mixing theorems and definitions, so a predicate that tests for the presence of a pattern classifies only the files of one kind; theorem-dominant mixed files would need a count comparison, and files classified whole stay a majority guess.

Implication: a `content` regex on refinements that matches when any line matches covers the tag case. Counting declarations is a different feature, closer to per-declaration measurement.

## Content kind of mapped files

Mapping `.tsv` to Plain Text counts every line as a comment and the files as documentation. Mapping them to JSON, a data language without comment syntax, counts them as data with every non-blank line as code, at the cost of reporting the language as JSON; YAML would read `#` lines as comments. A `kind` field alone on mappings would fix the kind but keep Plain Text's all-comment split, so a display name on a mapping counted by another counter is the closer fit.

## Configuration errors

The owner asked whether an incompatible or invalid `.locmetryrc` gets feedback good enough to repair it. A corpus of 26 broken files, from TOML syntax through unknown fields, wrong types, unknown values, globs and IDs, showed that every error failed fast with exit 1, empty stdout and the file named, but in the parser's wording: `expected a sequence`, `expected u32`, double quotes in a message about TOML strings, and no hint for a likely fix. An unknown field listed the valid ones but did not say that `language_mappings` is written `[[languages.mappings]]`, that `exclude` in an entry is `except`, or that a field may come from a newer release; `[rules]` instead of `[[rules]]` produced `invalid type: map, expected a sequence`.

Two fresh agents (Sonnet at `xhigh`, no terminal) per arm then repaired ten broken configurations, each with a stated intent, using only Locmetry's output and help. The candidate added the hints listed in [PRODUCT](../../PRODUCT.md): synonyms and near typos for unknown fields and values, a newer-release note otherwise, case-insensitive language suggestions, the `[[table]]` hint, TOML wording for types, the configuration version Locmetry reads, and which entry and field an invalid glob or ID belongs to.

| Arm | Agents | Read the whole `--schema config` | Cases where the agent judged the message not enough or only mostly enough |
| --- | ---: | --- | --- |
| Control, without the hints | 2 | 2 of 2 | 6 of 10: the mappings table, the `[rules]` table, a lowercase language name, version 2, a duplicate ID, `exclude` in an entry |
| Candidate | 2 | 0 of 2 | 2 of 10, both semantic: whether version 2 means the same as version 1, and a content-tag rule this release cannot express |

Every configuration of all four agents loads with the candidate; whether each matched its stated intent was judged by reading, not graded. The remaining two need knowledge of a future format and a feature this release lacks, which no message can supply.
