# Guide Sections and Task Index

<!-- doc-status
{
  "status": "accepted",
  "date": "2026-10-24",
  "summary": "Addressable guide sections and schema definitions, direct routes from help and a task index, so that agents read only what a task needs.",
  "evidence": "[second review and owner acceptance](guide-sections.md#review-history)"
}
-->

The owner raised the idea on 2026-10-24 after the [category-guide correction run](../research/agent-regression-run-2026-10-24.md#correction-run-shorter-guide), and approved writing it as a proposal the same day. An independent review the same day accepted it with changes; this revision adopts them: self-contained routes, exit status 1 for unknown sections, a precise schema namespace, corrected evidence, direct help routes, a wider guard, a reproducible acceptance manifest, and uncached input kept as the primary cost gate.

## Problem

Guides and schemas can be retrieved natively only as whole documents; projections need external tools such as `jq` or Python. Configuration sessions therefore read whole documents in one batch, as the agents' own instructions encourage for independent reads, and the batch outgrows what the agent harness shows. Counted from the raw session logs of the 2026-10-24 runs:

- The first reference batch was truncated in 24 of 24 sessions of the first run and in 7 of 8 of the correction, in both arms, to about 40K characters of decoded text. It held the configure guide and `--schema config` together in 23 of the 24 first-run sessions; the categories guide was in it in 4 of 24, the candidate's task-12 sessions.
- The configure guide was never fetched again. The configuration schema was fetched again in 20 of 24 and 5 of 8 sessions, often as a projection: partly to recover what truncation cut, partly as legitimate follow-up for the semantics the guides route to.

Smaller native reads address the first cause and make the second cheaper. They also stop each addition to a guide or schema from costing every configuration session: the category guide added 6.5 KB to a batch that was already being cut off.

## Design

### Required

**Section ids.** Each `##` and `###` heading of an embedded guide has an id: its GitHub heading slug, as `scripts/doc-indexes.py` computes it for fragment links. Ids are unique across all heading levels of a guide, so GitHub never adds numeric suffixes. The supported heading form is plain text with punctuation; a heading containing Markdown links, emphasis, inline code or closing hashes is rejected by the guard. Lines inside fenced code blocks and HTML comments are never headings, so a shell comment such as `# baseline` in a fence starts no section.

Long headings are shortened before their ids are first published, such as "Refine a Category Without Changing Purpose" to a shorter title. From then on an id is stable: removing or renaming a published id is a **Breaking** change recorded in CHANGELOG.

**Section output.** `--section ID` requires exactly one `--guide` or `--schema` topic. It rejects positional roots, subcommands, measurement options and a repeated `--section`, and keeps `--no-color`, help and version behaviour.

- With `--guide TOPIC`, it prints the usual source line, naming the section, followed by the heading and its text up to the next heading of the same or a higher level, so a `##` section includes its `###` subsections. Relative links stay as in the guide.
- With `--schema TOPIC`, `ID` is matched exactly against three namespaces:
  - `overview` selects the root object, with `properties` and `$defs` reduced to lists of their names. It carries the root `description`, which guide recipes use for precedence and matching semantics, and doubles as the schema's contents.
  - The name of a member of the root `properties` selects that property's schema, such as `rules`.
  - The name of a `$defs` entry selects that definition, such as `Refinement`.

  The output is a fragment, not a self-contained schema: `$ref` values such as `#/$defs/Purpose` resolve against the full topic schema and name the next section to request. It is pretty-printed JSON without a source line, so it can be piped.
- An unknown id exits with status 1, as other invalid queries do ([PRODUCT](../PRODUCT.md#command-outcomes)), with empty stdout. Stderr names the unknown id and lists the topic's valid ids in document order, worded by the [human output wording rules](../help-design.md#human-output-wording).

```text
$ locmetry --guide categories --section choose-a-name
Source: docs/guides/categories.md#choose-a-name (locmetry 0.10.0)

## Choose a Name
...
$ locmetry --schema config --section Refinement
{
  "description": "...",
  "type": "object",
  ...
}
```

**Self-contained routes.** PRODUCT already promises that a guide's essential instructions are self-contained. Sections extend that promise: a section the index or help names states its own prerequisites, or the route names the additional reads they need. For example, the owner-naming rule in the categories guide's opening moves into the section that chooses names, and a diagnostic subsection that depends on its parent's fixture or working directory is routed through its parent.

**Help routes.** `--help` reaches every session first. The configuration route gains a few conditional direct section commands for the needs agents met in the recorded runs, such as refining conventional tests or naming categories, and names `locmetry --guide index` for the rest. Other whole-guide and schema routes in help are reconciled with them, so help does not offer two routes for one need. [Help-design](../help-design.md) prefers an example with its applicability over another navigation layer; direct routes follow that, and the index is the fallback. Text must fit the current [help budgets](../agent-regression-checks.md#help-size-guard), 288 bytes free in `--help` and 62 in `-h` when measured on 2026-10-24, by rewording existing routes; a raise is a reviewed decision, taken only if rewording loses applicability or verification guidance. Fixtures that preserve the wording of earlier experiments stay unchanged.

**Task index.** [docs/guides/README.md](../guides/README.md) becomes the routing table and is embedded verbatim as `locmetry --guide index`, so the GitHub page and the CLI are one document. Each row pairs a task with every command it needs, prerequisites included; it keeps one whole-document link to each guide, as the documentation checker requires.

```markdown
| When you want to | Read |
| --- | --- |
| Give conventional tests finer categories | `locmetry --guide configure --section refine-categories`, `locmetry --schema config --section Refinement` |
| Name a new category | `locmetry --guide categories --section choose-a-name` |
```

**Guard.** Integration tests fail when:

- a guide heading is outside the supported form, has an empty slug, or duplicates another id of the same guide;
- a schema has `overview` as a property or definition name, or a property and a definition share a name;
- a `--section` reference does not resolve, wherever it appears: rendered help, guides, schema descriptions, README, and the recorded guides under `docs/cli/` and `docs/showcase/`, including inline code and code blocks, which the link checker deliberately skips;
- a guide section is named by no index row, neither itself nor a parent section that prints it;
- an id in the committed list of published ids no longer resolves, unless CHANGELOG's `Unreleased` section records its removal as **Breaking**;
- the Rust slug rule disagrees with `anchors()` in `scripts/doc-indexes.py` for any guide heading.

Recorded Showboat output that changes is regenerated, never edited.

### Excluded

- Splitting guides into more topics. Self-contained sections give the same small reads without more routing choices. `reports` (19 KB, 9 sections) is the only candidate worth revisiting if its sections prove too coarse.
- A `TOPIC#ID` syntax. Measured on 2026-10-24: with zsh's `extendedglob`, `--guide configure#choose-a-name` fails with "no matches found", and an unquoted `config#/$defs/Refinement` loses `$defs` to variable expansion in POSIX shells. Quoting would avoid both, at a cost every routed command would carry.
- JSON pointers into nested schema values, and bundling a definition with the definitions it references.
- Changing how full guides and schemas print, and size budgets for guides or schemas.
- Trimming the categories guide's "Write a Reviewed Policy". It would confound the navigation measurement, and its `kind = ["source"]` advice has no other home yet; it is a separate change.

## Acceptance

- The guard passes, and help stays within its budgets or a raise is reviewed.
- An offline manifest pins the baseline revision, the exact commands of each covered route with all prerequisites and the index where a route uses it, and counts UTF-8 bytes of stdout including source lines. Against today's 59,552-byte batch of configure guide, categories guide and configuration schema, the task-12 route reads at most a third, `--help` excluded. The manifest is committed with the implementation.
- An agent run on a new frozen protocol finds no loss of correctness and no regression under the existing gates: tasks 3, 8 and 12, plus unrelated task 1 as the root-help reading guard that [help-design](../help-design.md) requires, four AB/BA pairs each, 32 sessions. Uncached input stays the primary cost gate. The protocol also reports, as a diagnostic, the context each session builds: the sum of the increases of per-request input after the first request, read from the session logs' per-request token counts, which does not depend on cache hits or on how much of the first request was cached; a request smaller than its predecessor, as after compaction, adds nothing, and the protocol reports such decreases. It measures whole-session growth, not the isolated cost of any one document. For a primary failure that the diagnostic does not reproduce, the protocol predeclares one confirmation sample: its size, the condition that activates it, whether it is pooled with the first sample, and the final decision rule. Trials are never added until a gate passes. The [agent regression checks](../agent-regression-checks.md) own gates and diagnostics, so the harness support lands there before the protocol is frozen.

## Risks

- **Calls.** A route through the index adds a call, which the overall calls gate does not allow. Direct help routes avoid it for the common needs; the index serves the rest.
- **Uptake.** Routing studies found explicit execution-first guidance effective; none tested navigation through an index. Agents never used `-h` in 310 sessions, so routes must sit in `--help`.
- **Cache noise.** Uncached input remains exposed to provider cache misses; the predeclared confirmation sample reduces that sensitivity without moving the gate, but cannot rule out that cache misses decide an outcome.
- **Evidence.** The configuration route in help is pinned by the agent-explorability fixture as measured evidence; changing it requires the run above.

Implementation updates PRODUCT, the affected README and guide routes, help-design's inventory and CHANGELOG together.

## Review History

A second independent review on 2026-10-24 accepted the revision and asked that the protocol fix the confirmation sample's rules and present context growth as whole-session growth; commit `23e0ce9` adopted both. The owner accepted the proposal for implementation the same day. During implementation the same day, two guard rules changed: a schema property and definition may differ only in case, as `scope` and its definition `Scope` do, because ids match exactly; and the index covers a subsection through its parent, so no exemption list was needed.
