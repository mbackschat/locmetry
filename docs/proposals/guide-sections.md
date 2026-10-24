# Guide Sections and Task Index

<!-- doc-status
{
  "status": "draft",
  "date": "2026-10-24",
  "summary": "Addressable guide sections and schema definitions, and a task index that routes to them, so that agents read only what a task needs.",
  "evidence": ""
}
-->

The owner raised the idea on 2026-10-24 after the [category-guide correction run](../research/agent-regression-run-2026-10-24.md#correction-run-shorter-guide), and approved writing it as a proposal the same day.

## Problem

Configuration needs knowledge from guides and the schema, and the only way to get it is to read whole documents. In all 24 sessions of the first 2026-10-24 run and in 7 of 8 sessions of its correction, in both arms, the agent's first configuration read fetched the configure guide and `--schema config`, and usually the categories guide, in one parallel call. Codex cut the result to about 44–45K characters, roughly 11K tokens, from 12.7–20K tokens. The agents never saw the whole read and fetched the missing parts again, mostly as schema projections. Every addition to a guide or the schema therefore costs every configuration session, even when the task needs none of it: the category guide added 6.5 KB to a read that was already being cut off.

A task-12 agent needs about 12 KB of that read: the refinement section of the configure guide, the vocabulary of the categories guide, the `Refinement` definition and the verification section. It reads 59.5 KB.

## Design

### Required

**Section ids.** Each `##` and `###` heading of an embedded guide has an id: its GitHub heading slug, the fragment that repository links already use. The text is lowercased; characters other than letters, digits, spaces, hyphens and underscores are removed; spaces become hyphens. Ids are unique within a guide, without GitHub's numeric suffixes. One id serves a GitHub link and the CLI:

```text
docs/guides/configure.md#refine-a-category-without-changing-purpose
locmetry --guide configure --section refine-a-category-without-changing-purpose
```

**Section output.** `--section ID` is valid only with `--guide` or `--schema`, and stays a standalone query.

- With `--guide TOPIC`, it prints the usual source line, naming the section, followed by the heading and its text up to the next heading of the same or a higher level, so a `##` section includes its `###` subsections. Relative links stay as in the guide.
- With `--schema TOPIC`, `ID` is a top-level property name or a `$defs` name, matched exactly, and the output is that value as pretty-printed JSON, without a source line, so it can still be piped. `$ref` values remain document pointers such as `#/$defs/Entry`, which name the next section to request.
- An unknown id fails with exit status 2 and lists the topic's valid ids, so a mistyped id costs one call and returns the topic's contents.

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

**Task index.** [docs/guides/README.md](../guides/README.md) becomes the routing table and is embedded verbatim as `locmetry --guide index`, so the GitHub page and the CLI are one document. Each row pairs a task with the exact commands that answer it; a task that needs a whole guide names the guide.

```markdown
| When you want to | Read |
| --- | --- |
| Give conventional tests finer categories | `locmetry --guide configure --section refine-a-category-without-changing-purpose`, `locmetry --schema config --section Refinement` |
| Name a new category | `locmetry --guide categories --section choose-a-name` |
| Diagnose why one file is missing | `locmetry --guide selection --section a-file-is-absent-from-the-counts` |
```

**Routing from help.** `--help` already reaches every session first, and the [help-routing study](../research/agent-explorability-help-routing-study.md) found explicit conditional routes effective. The configuration route's sentence "Use --guide configure or --schema config for exclusions, overrides, intentional Other, or semantics not shown here" names `locmetry --guide index` as the way to find the section for a need not shown there. The `--guide` description lists `index` among the topics.

**Guard.** An integration test fails when:

- a guide has duplicate ids, or a heading whose slug is empty;
- a `--section` reference in help, guides, schemas or README does not resolve;
- the index omits a guide section or names a section that does not exist, except sections the index marks as covered by a whole-guide row;
- the Rust slug rule disagrees with the fragment rule of the documentation link checker for any guide heading.

Headings become a public contract: renaming one changes its id, which the guard reports in every reference.

### Optional

- Removing what the categories guide repeats from the configure guide: "Write a Reviewed Policy" (699 bytes) restates the configure workflow.
- Shortening long headings when their ids are first published, such as `refine-a-category-without-changing-purpose`, while every reference is still local to this repository.

### Excluded

- Splitting guides into more topics. Sections give the same small reads without more routing choices, and every extra topic is another place to look. The categories guide is already its own topic; `reports` (19 KB, 9 sections) is the only candidate worth revisiting if its sections prove too coarse.
- A `#` fragment syntax on the command line. Measured on 2026-10-24: with zsh's `extendedglob`, `--guide configure#choose-a-name` fails with "no matches found", and an unquoted JSON pointer such as `config#/$defs/Refinement` loses `$defs` to variable expansion in every POSIX shell. `--section` avoids both.
- JSON pointers into nested schema values; a definition is the unit an agent needs.
- Changing how full guides and schemas print.
- Size budgets for guides or schemas. Sections make document size matter less; [help budgets](../agent-regression-checks.md#help-size-guard) remain.

## Acceptance

- The guard passes; help stays within its budget, or a raise is reviewed as the help-size guard requires.
- Following the index for task 12 reads, by an offline count, at most a third of today's 59.5 KB configuration read, `--help` excluded.
- An agent run on tasks 3, 8 and 12, four AB/BA pairs each, on a new frozen protocol, finds no loss of correctness and no regression under the cost gates below.

The protocol gates primary cost on the context each session builds, its last request's input less the instructions all sessions share from the provider's cache, read from the per-request token counts in the session logs. Uncached input stays a reported secondary figure: the correction run showed that one provider cache miss can decide a four-pair median of uncached input, which the binary does not influence. The calls gates are unchanged. This gate change applies to new protocols only; earlier results stand as recorded. The [agent regression checks](../agent-regression-checks.md) own the gates, so the change lands there with the harness support, before the protocol is frozen.

## Risks

- **Calls.** Reading the index adds a call before the sections. The per-task gate allows one more call, but the overall gate allows none, so the index has to send agents to the right section on the first try, and sections have to replace rereads rather than add to them.
- **Use.** Agents ignore options they are not told about: 310 sessions made 535 help calls and none used `-h`. Without the help route, the index would go unread.
- **Help budget.** `--help` is at 12,662 of 12,950 bytes. The new option's description and the `index` topic must fit by rewording the configuration route, or need a reviewed raise.
- **Evidence.** The help route is pinned by the agent-explorability fixture as measured evidence, so changing it requires the agent run above.
