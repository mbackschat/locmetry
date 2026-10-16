# On-demand Guides in the Locmetry Binary

Status: proposed for external review, 2026-10-16. The owner requested this design document; implementation is not approved. The inspected baseline is `4065e81` (`0.4.0-dev.1`). [PRODUCT](../PRODUCT.md) owns current behavior; [PLAN](../PLAN.md#proposals--under-discussion) tracks this proposal. All `--guide` commands below are proposed, not available commands.

## Owner Input and Intended Outcome

Locmetry should be suitable for, and where useful optimized for, coding agents. At least the initial configuration of a project should be created or evaluated by an agent. Agents need to discover not just commands, options and schemas, but their meanings, practical workflows, limitations and gotchas. That knowledge should be accessible on demand from the tool itself, without an additional installed skill.

The owner suggested authoring Markdown documents covering options and commands, schemas and semantics, use cases and gotchas, and exposing them either through stable GitHub links or by including them in the executable as extended help, for example `--help config-schema`. The request is for professional recommendations, alternatives and rationale; inexpensive sub-agent experiments are explicitly permitted to test usability. A separate request establishes visible retirement of implemented proposals into an archive; the [retirement procedure](README.md#retirement-procedure) owns that documentation practice.

## Recommendation and Scope

Embed a small set of task-oriented Markdown guides in the binary, exposed by `--guide TOPIC`. Keep ordinary help for invocation syntax and generated schemas for machine-readable contracts. The guide joins those surfaces into an actionable workflow. It serves humans as well as agents and requires no agent-specific integration.

| Scope | Recommendation and rationale |
| --- | --- |
| Required | A discoverable topic index; configuration creation and evaluation; selection diagnosis; report interpretation; saved-report comparison; examples and relevant caveats; offline, version-matched access. These cover the requested agent workflow without reading the repository. |
| Optional for the reader | Follow links to repository documentation, rationale or research. These deepen understanding but cannot be required to complete the documented core tasks. |
| Excluded | A separate skill, agent mode, automatic configuration editing, new classification policy, a documentation search engine, an all-guides dump, another JSON documentation protocol, browser/pager launch, network fetching, and handwritten copies of the full CLI or schemas. These add mechanisms without addressing the observed retrieval problem. |

The recommendation is to organize by task rather than put commands, semantics and gotchas into separate large documents. Keep each caveat next to the action it qualifies. Agents should retrieve the relevant topic, not assemble one task from several encyclopedic references.

## Proposed Public Interface

```sh
locmetry --help
locmetry --guide index
locmetry --guide configure
locmetry --guide selection
locmetry --guide reports
locmetry --guide compare
locmetry --schema config
```

`--guide` takes one required value from `index|configure|selection|reports|compare`. A missing or unknown topic exits 1, identifies accepted values, and produces no guide on stdout. Root help lists the option and directs first-time configuration work to `--guide configure`; topic names are case-sensitive, like schema names. No abbreviated topic names or filesystem paths are accepted.

A successful query prints one complete Markdown document to stdout and exits 0, without terminal styling or an automatic pager. It starts with the Locmetry version, topic and title so saved or pasted guidance identifies its producer. Output does not change when piped. It requires no network, project scan or configuration loading and works outside a repository, including when a local configuration is invalid. There is no separate guide version.

Recommend treating it as a standalone query: reject combinations with a positional root, measurement or presentation options, `--schema`, or a subcommand before any scan or write. Defaulted options do not count as explicit combinations; ordinary `--help` and `--version` retain their normal early-exit behavior. This avoids another case of accepting a requested output or action and silently ignoring it. Guide saving uses shell redirection, for example `locmetry --guide configure > configure.md`; `--output` remains a result-export option.

| Topic | Required contents and boundaries |
| --- | --- |
| `index` | Short descriptions of all topics; when to use command help versus a guide versus a schema; exact next commands. It is a navigation document, not the entire manual. |
| `configure` | Create or evaluate `.locmetryrc`; establish intended scope; measure first; review unclaimed and already classified source; inspect representative decisions with `explain`; make narrow evidence-based entries with reasons; verify on the unrestricted project. Include evaluating an existing config, personal-default effects and configuring workspace members through their own project configs and the workspace schema. Explain overrides versus rules and direct readers to the config schema for exact stages, defaults, built-ins and accepted values. |
| `selection` | Independent scan root versus `--subfolder`; root-relative paths and configuration discovery; user defaults, hidden and ignored paths, exclusions and symlinks; use `explain` to distinguish an established cause from unavailable evidence. Changing selection can change module context as well as measured files. |
| `reports` | Table, Markdown, full JSON and headline; destination defaults and protection; requesting file records; source versus data/documentation; additive breakdown versus overlapping language participation; documented zero omissions versus unavailable evidence; partial outcomes and stderr. Show narrow schema queries instead of requiring ingestion of the whole schema. |
| `compare` | Save before/after full measurements outside measured roots; compare without rescanning; matching restrictions; net change versus Git additions/deletions; policy, producer and coverage findings; workspace presence changes; limitations of aggregate evidence. Explain why a saved headline cannot reconstruct a missing baseline and why separate measurements need not capture identical filesystem state. |

Each task guide includes a short workflow, runnable synthetic examples, expected evidence, pitfalls and next discovery commands. Keep examples self-contained or explicitly name their prerequisites. Cross-topic navigation names the exact `locmetry --guide` or `--schema` command; relative repository links are supplemental and are not assumed to resolve when the text is printed from the binary. Essential warnings cannot be available only through a web link. The guides describe how to inspect and judge evidence; they do not claim that agents can infer project intent reliably from names alone.

For configuration review in particular: zero unclaimed source is not proof of correct classification; build modules can classify fixtures or examples as production; intentionally unclassified source is legitimate; unavailable counts are not zero; and unused entries in a subtree do not justify changing project-wide policy. The reader should surface unresolved scope judgments to the project owner rather than manufacture rules merely to make findings disappear. A successful measurement validates configuration loading and observed behavior, not the correctness of those judgments.

## Alternatives and Decisions for Review

| Decision or alternative | Recommendation and rationale |
| --- | --- |
| Embed Markdown or only link to GitHub | Embed the core guides. Offline availability and exact alignment with the installed binary outweigh the small additional text payload. GitHub remains useful for deeper context; version-sensitive external references should use a release tag or commit, not a moving `main` URL. |
| `--guide TOPIC` | Recommended. It parallels `--schema TOPIC`, adds no reserved subcommand name and requires no optional-value parsing. Use `index` explicitly to discover topics. |
| `locmetry help TOPIC` | Credible alternative: Git uses command and concept help. It would extend the existing help namespace and require distinguishing command help from guide topics. Prefer the separate option for this small interface; choose extended `help` if review establishes a substantial discovery advantage. |
| `--help TOPIC`, including `--help config-schema` | Do not overload the ordinary help flag. Preserve its existing invocation behavior and keep `--schema config` as the one way to obtain that schema. A guide can explain how to use the schema. |
| New `docs` or `guide` subcommand | Avoid reserving another bare directory name solely to retrieve static text. No benefit over the proposed option for the current scope. |
| Expand only schema descriptions or `--help` | Keep both accurate, but do not put every workflow there. Schemas already carry useful semantics; their size and structure make them cumbersome as the primary tutorial. Ordinary help should remain scannable. |
| Separate agent skill | Excluded by owner preference. A skill also requires installation and can diverge from the executable version. Useful procedure should be available through the same binary every agent can invoke. |
| Dedicated JSON index, search, or all-guides output | Exclude initially. A short fixed index and topic retrieval are sufficient for the observed tasks; these extra interfaces lack demonstrated demand. |
| Informational query mixed with measurement flags | Reject explicitly supplied unrelated flags. This is stricter than some existing schema combinations, but prevents silent ignored requests in a new interface with no existing callers. |

## Documentation Ownership and Drift

Author the guide text once as Markdown in `docs/guides/` and include that same text in the executable. This location is proposed; it does not yet contain supported guides. Avoid a generated second Markdown copy or a separate embedded rewrite. Version identification belongs to the command output, so editing a release number is not required in every source document.

[CLI definitions](../../src/main.rs) own option spelling and accepted values; generated schemas own structural and field-level contracts; [PRODUCT](../PRODUCT.md) owns supported guarantees. Guides own task sequences, examples and interpretation, and link to those owners instead of reproducing their complete references. README becomes the entry point to the same guides rather than another full copy of their workflows. Some concise explanation must be repeated for offline usability; behavioral changes must update affected guides and examples in the same checkpoint.

The existing configuration schema already contains setup advice. Retain enough semantic guidance there for schema-only consumers; deliberately share or reconcile overlapping workflow text rather than independently maintaining two tutorials. Dynamically defined language names and built-in lists remain in the generated schema. The guide teaches how to retrieve them instead of freezing another list.

Examples should be checked for their claimed result, not merely successful exit. Documentation checks cannot prove every prose statement current; review must still reconcile guidance with PRODUCT and schemas. The [repository ownership table](../../CLAUDE.md#documentation-ownership-and-checks) should name guides when the feature is implemented. Do not describe the proposed option as supported in PRODUCT or the README usage reference before then.

## Evidence and Limits

On 2026-10-16, inspection of `4065e81` found substantial existing guidance in [configuration schema descriptions](../../src/config.rs), [report contracts](../../src/contract.rs), [CLI help](../../src/main.rs) and the [README agent workflow](../../README.md#configure-with-an-agent). Emitted config and report schemas were 27,882 and 57,095 bytes respectively. These are observations of this baseline, not size targets or token measurements.

A single read-only discovery exercise used a fresh LOW-tier agent (`gpt-6-luna`, medium effort) with no source, repository documentation or real-project scans. Five shell calls inspected root/explain help, the config schema and selected report-schema definitions through `jq`. The agent recovered configuration precedence, the setup workflow, coverage-versus-zero distinctions and full-report export. It found workflow prose buried in schema text, a noisy language enumeration and a root-versus-subfolder distinction requiring synthesis across descriptions. It did not create or evaluate a configuration. This is qualitative retrieval evidence, not a configuration-quality benchmark, timing study, or demonstrated improvement from guides that do not yet exist.

[Git's official help documentation](https://git-scm.com/docs/git-help), retrieved 2026-10-16, documents discoverable concept guides alongside command help. That supports the separation of syntax and conceptual guidance, not the specific `--guide` spelling or any claim about agent performance. Existing [project classification trials](../research/real-project-trials.md#agent-configuration-trials) concern earlier workflows and populations; their agreement results do not establish a benefit from this proposal.

## Acceptance and External Review Focus

- Verify every topic is discoverable from root help or the index, prints version-identified plain Markdown, and works offline without reading project configuration or scanning. Unknown/missing topics and unsupported combinations fail before side effects.
- Verify terminal and pipe output contain the same guide content, no ANSI and no paging; examples use only supported commands of that build. No third-party agent runtime is required by the binary or public tests.
- Execute synthetic configuration examples and assert the expected classification and deciding rules, including a mistaken production classification that would escape an unclaimed-only review. Include an intentionally unclassified case that should not be “fixed.”
- Verify selection examples distinguish a nested independent root from `--subfolder`, and demonstrate an established exclusion cause as well as the limits of explanation evidence. Verify coverage examples preserve unavailable versus zero and partial status.
- Verify saved-report examples produce eligible full baselines, a known net change and the documented scope-mismatch rejection. Check destination/stdout expectations, not just exit codes.
- Repeat fresh-agent exercises with and without guide access on the same synthetic task set: initial configuration, evaluation of a flawed configuration, missing-file diagnosis and saved-report comparison. Assess substantive decisions and evidence, commands used, unnecessary retrieval and unresolved judgments. Record model, prompts and limits; a small trial is not a general accuracy claim. Do not set an arbitrary improvement percentage.
- Review whether five topics cover the core workflow without duplication, whether the proposed flag is easier to discover than extending `help`, and whether essential semantics remain accessible without a web visit or a giant schema dump. Recommended decisions and rationale are in the alternatives table; unresolved owner policy must not be converted into automatic classification advice.

No counting, selection, schema structure or output-routing changes are proposed. The material risks are prose drift, excessive guide length, and advice that mistakes a clean report for correct project policy. The ownership boundaries and outcome-based examples address those risks without introducing a new processing framework.
