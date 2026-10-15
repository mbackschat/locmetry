# Trials on Real Projects

Conducted on 2026-10-15. This page publishes the method, results, and resulting changes of trials on real repositories. Only a12-rulekit and bpmn-lean-experiment are named; other projects stay anonymous, and paths, sizes, and configurations stay in Git-ignored `docs/research/local/`, as [CLAUDE.md](../../CLAUDE.md#sensitive-analysis) requires.

## Agent Configuration Trials

**Question:** can a coding agent that works in a repository write a correct `.locmetryrc` from the binary alone?

**Method:**

- Each trial gives a fresh agent a one-line task, "create a locmetry configuration for this project", and nothing else: no locmetry source, no documentation, and no mention of `--schema`. The agent may read the repository but not change it, and passes its configuration with `--config` from outside.
- The oracle is a human-reviewed configuration per repository. A score is the share of source code lines whose purpose, `production`, `test`, or `unclassified`, matches the reference, file by file. From round 3 on, the reference is measured again right before scoring, because the repositories kept changing.
- a12-rulekit is a Kotlin Multiplatform library with nonstandard Gradle test source sets and test corpora, repository B a TypeScript research monorepo with Git submodules of other teams' code, and repository C a TypeScript monorepo with archived evidence whose files are named like tests.

**Results:**

| Round | Binary | Model | a12-rulekit | B | C |
| --- | --- | --- | ---: | ---: | ---: |
| No configuration | | | 94.7% | 21.8% | 95.9% |
| 1 | `ed89e2f` | Sonnet | 97.2% | 98.4% | 92.5% |
| 2 | `3ce20ad`, purposes defined in the schema | Sonnet | 99.9% | 99.0% | 99.5% |
| 3 | `86ec058` and `76aa744` | Haiku | 99.6% | 99.7% | 99.4% |
| 4 | 0.1.3 | Sonnet | | 99.0% | 99.4% |

- Every agent found `--schema config` from `--help` and followed the setup steps its description states, in 9 to 17 tool calls and under two minutes.
- All agents of C used an override for the archived evidence, because test conventions beat rules, and all agents of B excluded the submodules.
- Remaining differences are judgment calls, such as test-support helpers inside a production source tree or reference material excluded rather than counted.
- In [bpmn-lean-experiment](https://github.com/mbackschat/bpmn-lean-experiment), a Lean project with a Lake build and no reference configuration, two models agreed on 95.0% of source; nearly all of the rest was whether machine-checked conformance theorems are tests or production.

**Open question without configuration:** asked "how much test code compared to production code?" in B, an agent without configuration counted the submodules as production and answered 0.58 test lines per production line instead of about 1.0. After `2703a79` pointed at Git submodules, a Haiku agent excluded them and answered about 1.05.

## What the Trials Changed

| Finding | Change |
| --- | --- |
| Round 1 agents parked maintained tooling and examples as `unclassified`, reading production as shipped code. | The schema defines production as maintained non-test source (`3ce20ad`); round 2 rose to 99.0–99.9%. |
| Agents guessed the report's shape and missed an absent `unclassified_folders`. | `--schema report`, and the folder list is always present (`86ec058`). |
| A question-answering agent counted other teams' submodules as production. | `summary.submodules` and a table note name them (`2703a79`, `fa5a76d`). |
| Entries whose globs matched nothing went unnoticed. | `summary.unused_entries` (`2703a79`). |
| Agents invented inconsistent category names. | `--categories` and an optional `[categories]` declaration (`3327cc2`). |
| The review list mixed deliberately parked code with unclaimed code. | Only unclaimed source is listed for review (`abb5973`). |
| Round 4 agents reviewed only unclaimed code, missed misclassified code that build modules claimed, widened rules over build-module source, and guessed the entry shape. | The schema states the precedence, the review step, and a validated example entry (`75e4bae`). |
| An exclusion of a folder without `/**` silently matched nothing. | `summary.unused_exclusions` (`c4abd02`). |

Still open: `metadata.root` reports the path with symbolic links resolved, `locmetry .` is rejected because the root is `--root`, and the language list makes `--schema config` large.

## Workspace Run

Four sibling repositories, each with its reviewed configuration, were measured as one workspace. The merged purpose totals equalled the sum of the individual runs, each repository's totals equalled its own run, and the run took under a second. Every repository had unrecognized files, mostly images and files without an extension, so a per-repository note that listed them named every repository; it now lists only unclaimed and unreadable files (`030b3ce`).
