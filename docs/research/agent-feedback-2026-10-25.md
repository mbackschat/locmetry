# Agent Feedback on Two Workspaces, 2026-10-25

After the changes that followed the [categorization feedback](agent-experiments-2026-10-25.md), the owner asked for Locmetry 0.11.1 to be checked by Codex agents updating real workspace policies, and for their feedback. This is a feedback round, not a frozen benchmark: the sessions ran in the owner's normal Codex home on real repositories, without controls for cache or order, and its findings are leads that were each verified before anything was recorded. Private projects' names and counts stay in the Git-ignored local folder.

## Sessions

Two fresh Codex sessions on `gpt-6.1-sol` at `xhigh`, the MEDIUM tier, each worked on one workspace: a private workspace of three repositories, and the public [a12-kernel](policy-test-drives/a12-kernel.md) workspace of four repositories at the commits of its 2025.06-ext5 release. The prompts were cold: the first turn asked the agent to update and review the workspace file and the members' `.locmetryrc` files so they make the best use of what the installed Locmetry offers and classify the code as accurately as the repositories' evidence supports, changing only those files; the second turn, in the same session, asked whether Locmetry could be improved and how well its help, guides, errors and reports served an agent new to it. No prompt named a feature.

Two setup failures preceded the valid sessions. Codex refused a writable root reached through a symbolic link, so the private workspace was given by its resolved path. Both first sessions then ran the installed 0.11.0: Codex runs each command in a login shell, which reloads the profile and put the Homebrew directory ahead of the PATH entry that named 0.11.1. Their results are kept as a 0.11.0 control. The valid sessions ran with the Homebrew link pointed at the 0.11.1 build, and an audit of their transcripts and saved reports confirmed 0.11.1 in every command, printed version and report producer.

## Outcome

Both agents produced policies that load, claim all counted source and leave no unused entries or exclusions; they used declared category meanings, refinements, `except` lists, `kind` limits, reasons, language mappings and corrections. Only policy and workspace files changed. Both praised the precedence explanation, the policy audit's `preempted` and `unused_patterns` evidence, which exposed a dead rule in an original policy, refinements together with `explain`, and saving full evidence while printing a table from one scan. Neither met a configuration error, so the round does not evaluate the new error messages. Both said they read whole guides or schemas before using sections.

## Findings, as Verified

| Finding | Verification | Result |
| --- | --- | --- |
| A glob with an empty alternative, such as `run{,.cmd}`, silently misses `run`; the audit stays quiet because the glob still matches a file | Reproduced on a synthetic project; globset drops empty alternatives unless enabled, and PRODUCT promises that `{a,b}` matches any listed alternative | Fixed in 0.11.1 |
| Workspace policy evidence is not where the configure guide points | The configure guide named only `summary`; the reports guide and PRODUCT document `metadata.members[]` for independent members. All three sessions that gave feedback, the control included, first searched `summary` or the member totals | Configure guide fixed in 0.11.1 |
| Per-file transitions are missing for workspace comparisons | The file-transition helper accepts single-root reports only; all three sessions that reviewed changes, the control included, wrote their own join | [DEFERRED](../DEFERRED.md#measurement-and-comparison): trigger met, the owner decides |
| Comparisons are noisy about unrecognized files | Human output summarizes them in one line; only JSON lists one `coverage_gap` per folder, worded as unavailable counts | Weaker than reported; [DEFERRED](../DEFERRED.md#display) wording item |
| Rstest has no runner-configuration convention | The built-in conventions cover Vitest, Jest, Playwright and Cypress only | [DEFERRED](../DEFERRED.md#built-in-detection) candidate, pending the research checks |
| Language mappings have no `reason` | The configuration schema lists `id`, `match` and `language` only | [DEFERRED](../DEFERRED.md#configuration-and-categories) |
| ANTLR and StringTemplate files cannot be counted | No counter language exists for either; grammars could map to a C-family counter by their comment syntax, under that counter's name, while templates match no counter | Evidence for the deferred display name of mappings and corrections |

The operator's own check found one more gap: five category names that the category reference replaces, inherited from the original policies, survived the update, since nothing in Locmetry named the replacement ([DEFERRED](../DEFERRED.md#configuration-and-categories)).

## Limits

Two workspaces, one session each, so the findings are leads rather than rates. The control sessions ran another version and are not paired with the valid ones. The feedback turn invited criticism, which favors complaints over praise. Raw sessions, the original policies and the binary audit are kept in Git-ignored `docs/research/local/agent-feedback-2026-10-25/`.
