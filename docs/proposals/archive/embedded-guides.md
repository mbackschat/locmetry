# On-demand Guides in the Locmetry Binary

<!-- doc-status
{
  "status": "implemented",
  "date": "2026-10-16",
  "summary": "Embedded task guides; implementation does not imply publication.",
  "evidence": "[Acceptance evidence](../../plans/archive/measurement-quality-and-net-change.md#embedded-guidance-implementation--2026-10-16)"
}
-->

Status: historical decision record — implemented and verified on 2026-10-16 in the unreleased `0.4.0-dev.1` checkout. [PRODUCT](../../PRODUCT.md#embedded-task-guides) owns the supported contract; the [work plan](../../plans/archive/measurement-quality-and-net-change.md#embedded-guidance-implementation--2026-10-16) records acceptance evidence. Imperative wording, examples and limitations below describe the reviewed design, not current guarantees. Preserve this rationale and acceptance record; do not update it to track behavior.

## Owner Input and Intended Outcome

Locmetry should be suitable for, and where useful optimized for, coding agents. At least initial project configuration should be created or evaluated by an agent. Commands, options and schemas must be discoverable together with their semantics, use cases, workflows and gotchas, on demand from the tool itself, without an additional installed skill.

The owner suggested Markdown documents covering those subjects, either linked through stable GitHub URLs or included in the executable as extended help such as `--help config-schema`. The requested advice includes alternatives, recommendations and rationale; inexpensive sub-agent experiments are permitted. Subsequent feedback also permits research into comparable tools.

## Recommendation and Scope

Embed four task-oriented Markdown guides through `--guide TOPIC`. Ordinary help owns invocation syntax; generated schemas expose structure and field semantics; guides explain how to use and interpret them. Keep pitfalls next to the actions they qualify, rather than separating commands, semantics and gotchas into different encyclopedias.

| Scope | Recommendation and rationale |
| --- | --- |
| Required | Configuration creation/evaluation, selection diagnosis, report interpretation and saved-report comparison; examples and essential caveats; offline access to the installed version's guidance. |
| Optional for readers | Follow supplementary repository or research links for deeper rationale. |
| Excluded | Separate skill, agent mode, automatic config editing, new classification policy, index topic, search, all-guides dump, documentation JSON protocol, network fetching, browser/pager launch and handwritten full option/schema references. |

## Public Interface

`--guide` requires one case-sensitive topic from `configure|selection|reports|compare`. Root `--help` lists each with a one-line description generated from the topic enum, and explains help versus guides versus schemas. Root `--help`, `-h` and `locmetry help` must all include this explicit starting instruction:

> Start project configuration with `locmetry --guide configure`.

Short help may omit topic descriptions but must retain that starting command. Missing/unknown topics exit 1 with accepted names, a `--help` remedy and empty stdout; errors need not repeat descriptions. Subcommand help, in both short and long forms, points to executable root queries: `explain` to `locmetry --guide selection`, `compare` to `locmetry --guide compare`, and `workspace` to `locmetry --guide configure` and `locmetry --guide reports`. Guides are not subcommand options.

### Fresh-agent Discovery Path

The configuration guide must supply this usable starting sequence, not merely name the available subjects. Here `.` is the intended project root; the agent first inspects any existing `.locmetryrc` and retains its policy unless evidence justifies a change.

```sh
locmetry --help
locmetry --guide configure
locmetry --schema config
locmetry --root . --format headline
```

The first measurement precedes configuration edits. Every guide supplies exact next commands and explains when to use them: `configure` directs readers to the config schema and initial measurement; `selection` to `explain --help` and `--schema explanation`; `reports` to `--schema report`; and `compare` to `compare --help` and `--schema comparison`. Configuration guidance also names `--schema user-config` and `--schema workspace` when relevant. Cross-topic commands always start with `locmetry`, so no checkout, inferred URL or remembered flag placement is needed.

Start with compact evidence; use a specific file explanation or a narrow full-report query when needed. Do not instruct agents to read every guide, every schema or all file records before acting. Examples using `jq` must identify it as an optional external query tool; Locmetry itself does not require it.

Successful guide queries exit 0 with empty stderr and print a header identifying source path and executable version, for example `Source: docs/guides/configure.md; locmetry 0.4.0-dev.1`, followed by the Markdown source unchanged. No styling, paging, configuration loading, scanning or network access occurs. Queries work outside repositories and beside invalid configuration. The header is produced by the executable; source documents contain no maintained release number or separate guide version.

### One Standalone-query Rule

Both `--guide` and `--schema` reject each other, explicit positional roots, measurement/presentation options and subcommands, before scanning or writing. Only explicit unrelated arguments conflict: `--root .`, `--config FILE` and `--no-user-config` are included, as are explicitly supplied default values such as `--sort code` and `--depth 2`. Defaulted values do not conflict; normal `--help`/`--version` early exits remain. Save either query by shell redirection; `--output` remains a result-export option.

Recommend tightening existing `--schema` validation as a separate preceding change, then using that same argument-validation boundary for guides. Today schema queries silently ignore roots, subfolders, exclusions and sort options, while explicit categories/depth only warn. Under this proposal those combinations exit 1, including `--schema config --depth 3`. This changes existing behavior; it is justified by avoiding silently ignored requests and two query policies, not by assuming that a pre-1.0 change cannot affect callers. Ordinary measurement's ineffective-option warnings remain unchanged. The work plan records this prerequisite separately; the owner has approved both checkpoints.

## Topics and Configuration Outcomes

| Topic | Required content |
| --- | --- |
| `configure` | Create or evaluate `.locmetryrc`; establish scope, including submodules; measure first; inspect unclaimed and already classified source; use `explain`; write narrow entries with reasons; validate unrestricted measurements and unused-policy findings. Cover overrides/rules, categories, personal defaults and independently configured workspace members. Retrieve exact defaults, built-ins and languages from schemas rather than copying lists. |
| `selection` | Extend the existing troubleshooting workflow: root versus subfolder, configuration/path bases, ignores, hidden files, exclusions, symlinks and explanation evidence limits. Changing selection can change module context. |
| `reports` | Human formats, full JSON/headline, destination defaults/protection, per-file records, source/data/documentation, additive breakdown versus overlapping language participation, documented zero omissions versus unavailable evidence, partial outcomes and narrow schema queries. |
| `compare` | Save full baselines outside measured roots; compare without rescanning; matching restrictions, net change versus Git additions/deletions, policy/producer/coverage findings and workspace presence. A headline cannot reconstruct a baseline; separate measurements need not capture the same filesystem state. |

For source needing a classification decision, the configuration workflow has three legitimate outcomes:

1. Classify it through an evidence-based entry.
2. Leave it intentionally unclassified through an entry with `purpose = "unclassified"` and a `reason`. This records a decision and removes it from the unclaimed review list.
3. Leave it unclaimed and report the unresolved question to the owner. Do not invent an entry to remove a finding.

Already-correct built-in or module decisions do not need redundant entries. Zero unclaimed source is not proof of correctness: module-inferred production also needs representative review. Unavailable counts are not zero; subtree-only unused findings do not justify deleting project policy. A successful measurement establishes configuration loading and observed behavior, not agreement with the owner's intent. The configuration guide ends with a handback: resolved decisions and their evidence, intentional unclassified entries and reasons, remaining unclaimed questions, coverage limitations and unused-policy findings. Remaining questions do not prohibit delivering a review; they prohibit claiming that configuration is fully settled. Separate unresolved intent from invalid configuration or partial counts.

In the guide implementation checkpoint, replace the config schema's blanket instruction to add entries for unclassified folders and the README's “until no folder is left undecided” instruction with these outcomes. Correct the schema's name-based implication that tests/fixtures/examples necessarily need an entry: names prompt inspection, not a classification verdict. These are required guidance corrections, not optional reconciliation or new classifier behavior.

## Sources, Ownership and Length

Use one Markdown source per topic in `docs/guides/`, embedded verbatim. Move [troubleshooting.md](../../guides/selection.md) to `docs/guides/selection.md` and extend it; do not author another overlapping selection guide. Update its inbound links, relative links, the embedding test and contributor ownership entry while preserving existing heading anchors where practical. Retarget [the executable documentation checks](../../../scripts/check-doc-examples.sh) to recipes at their new source locations, preserving their existing outcome assertions; moving a recipe is not a reason to stop executing it. Keep shell/`jq` recipe execution in that development/CI check and public Cargo tests free of external executable dependencies.

| Existing entry point | Required treatment |
| --- | --- |
| README “Configure with an agent” | Retain the task prompt and a short guide/schema pointer; move the detailed workflow to `configure`. |
| README “Common issues and useful queries” | Retain a compact issue-to-guide map; move selection remedies to `selection` and aggregate query recipes to `reports`. |
| README “Compare saved measurements” | Retain a minimal before/after/compare example and a guide pointer; move detailed variants, caveats and partial-result recipes to `compare`. |
| Config schema descriptions | Keep field semantics, exact conventions and a concise accurate setup summary with the three outcomes. Avoid a second full tutorial. |

[PRODUCT](../../PRODUCT.md) continues to own supported guarantees; [CLI definitions](../../../src/main.rs) own syntax; schemas own structural and field contracts. Guides own workflows and examples. README and [contributor guidance](../../../CLAUDE.md#documentation-ownership-and-checks) must point to those owners when implemented.

Write normal relative Markdown links for supplementary repository references and print them unchanged. The source-path/version header supplies their context; it does not make relative links automatically clickable in a terminal or identify an exact development commit. Essential instructions stay self-contained. Cross-guide navigation uses `locmetry --guide TOPIC` commands. No link rewriting, generated GitHub URL or release-number maintenance is needed; this replaces the initial proposal's pinned-link recommendation.

Target roughly 4–8 KB of UTF-8 Markdown per topic, using the existing 7,078-byte troubleshooting guide as a reference. This is an editorial review target, not a minimum to fill or a test limit. Explain justified excess; remove duplicated reference material before splitting topics. Each guide needs a short workflow, synthetic examples, expected evidence and relevant caveats. Synthetic examples isolate personal defaults with `--no-user-config` on measurement/explanation commands and use temporary fixtures; real-project guidance inspects effective policy rather than silently disabling it.

## Alternatives and Rationale

| Alternative | Recommendation |
| --- | --- |
| GitHub links only | Embed core guidance for version matching and offline use. Web links supplement it; they cannot be required for the documented tasks. |
| `--guide TOPIC` versus `help TOPIC` | Prefer the option beside `--schema`, with a required value and no new reserved directory name. Extended `help` is a credible precedent, but adds command/topic dispatch without a demonstrated advantage here. |
| Overload `--help TOPIC` or add `docs`/`guide` subcommands | Preserve ordinary help behavior and avoid another reserved command name. Keep schemas at `--schema`; guides explain their use. |
| More schema/help prose only | Preserve useful descriptions, but separate task guidance from large schemas and keep ordinary help scannable. |
| Separate skill | Excluded by the owner. A separately installed procedure also risks version drift. |
| Index topic or search protocol | Omit: four described values in root help cover discovery. Reconsider an index only if routing explanations outgrow a few lines. |

## Evidence and Comparable Tools

At `4065e81`, config/report schemas measured 27,882/57,095 bytes. A fresh LOW-tier agent (`gpt-6-luna`, medium effort) used five shell calls to inspect help and schemas, without repository docs or real-project scans. It recovered precedence, setup, coverage and export semantics, but reported buried workflow prose and noisy enumerations. It never configured a project: this is retrieval evidence, not a benchmark or demonstrated guide benefit. Earlier [classification trials](../../research/real-project-trials.md#agent-configuration-trials) do not measure this proposal either.

The external review of `2b28641` and local checks confirmed the existing marked-result example in [explanation tests](../../../tests/explain.rs), ignored schema arguments, schema success beside invalid configuration, and the packaging feasibility of embedding repository Markdown. A local unknown-schema probe lists names without descriptions, so the interface above does not promise descriptions in errors.

Official documentation inspected online on 2026-10-16 provides these bounded precedents; peer binaries were not tested:

| Source | Observed behavior | Lesson for this proposal |
| --- | --- | --- |
| [Go command documentation](https://pkg.go.dev/cmd/go) | Root documentation lists additional conceptual topics with descriptions and directs readers to `go help <topic>`. | Discoverable topics need not require a separate index request. This supports the structure, not the choice of option spelling. |
| [Git help manual](https://git-scm.com/docs/git-help) | Command and conceptual manuals share help; `--guides` lists guides. Its default viewer is `man`, with other viewers available. | Conceptual guidance complements syntax. Locmetry should keep direct stdout instead of adopting viewer dependencies. |
| [rustc command-line reference](https://doc.rust-lang.org/rustc/command-line-arguments.html#--explain-provide-a-detailed-explanation-of-an-error-message) | `--explain CODE` prints a longer explanation of a diagnostic. | A focused on-demand explanation flag is familiar; it does not establish Locmetry's guide semantics or agent effectiveness. |

These examples do not justify adding a skill, protocol, search engine or broader documentation platform.

## Acceptance

- Assert the explicit starting instruction in root `--help`, `-h` and `help` output, allowing normal wrapping. Check all topic descriptions in long help and the specified root-query pointers in short/long subcommand help. Do not settle for checking that the word “guide” appears.
- Exercise the fresh-agent path on a synthetic project using the documented commands and marked expected results. Verify the next-step commands resolve to existing guide topics, schemas and command help; the chain must not rely on repository or web access. This checks discovery and observable results, not agent reasoning quality.
- Missing/unknown topics and standalone-query conflicts fail with exit 1 and empty stdout before side effects. Check both queries, both argument orders, explicit default-valued options, and config/user-config flags as well as positional roots. Defaults remain accepted; normal help/version early exits remain. Errors identify the offending argument; bad topics supply the help remedy.
- Guide stdout equals the source/version header plus the embedded Markdown bytes, without ANSI or paging, even beside invalid configuration. A byte-level assertion is sufficient; public tests need no pseudo-terminal, network or agent runtime.
- Use the existing marked-result-block pattern: tests construct synthetic fixtures and compare actual JSON results or stable projections with marked expected blocks in the guide. Preserve the selection example and cover the three configuration outcomes, mistaken production classification, nested-root/subfolder differences, unavailable evidence and a known net change with scope-mismatch rejection. Assert statuses and destination behavior as well as counts.
- Update schema/README guidance, documentation ownership, moved links and executable example references in the same checkpoint. Preserve existing recipe assertions when their source moves, and verify synthetic examples are independent of personal defaults. Review the handback instructions, self-contained task coverage, absence of duplicate tutorials and size target; a schema assertion alone cannot establish their quality.

No counting, selection, schema structure or measurement output-routing change is proposed. Guide prose and standalone-query validation are the scope; the [work plan](../../plans/archive/measurement-quality-and-net-change.md#embedded-guidance-review--2026-10-16) owns sequencing, review disposition and any later usability experiments.
