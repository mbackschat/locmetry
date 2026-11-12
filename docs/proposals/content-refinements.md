# Content Conditions on Refinements

<!-- doc-status
{
  "status": "draft",
  "date": "2026-11-12",
  "summary": "An optional `content` regex on refinements, tested against the lines the counter classifies as code, for categories that paths cannot express.",
  "evidence": "[DEFERRED](../DEFERRED.md#configuration-and-categories), [content predicates experiment](../research/agents/experiments-2026-10-25.md#content-predicates-for-categories)"
}
-->

Status: draft for the owner's decision. It takes up the deferred content predicates on refinements in the scope their [experiment](../research/agents/experiments-2026-10-25.md#content-predicates-for-categories) set, a regex that matches when any line matches, and adds one rule: only lines the counter classifies as code are tested. The evidence comes from a private project migrating TypeScript to Kotlin Multiplatform; its name, paths and figures stay in the Git-ignored local research folder. The two companion items [DEFERRED](../DEFERRED.md#configuration-and-categories) asks to propose together with this one, a filter on the deciding rule and entry references in `except`, are outlined under [Companion Items](#companion-items) for the owner to bundle or keep apart.

## Problem

**Some categories are a property of a file's content at a commit, not of its path.** The migrating project replaces each ported module's TypeScript implementation in place: the file keeps its path, its types and its doc comments, and its exports delegate to the Kotlin implementation. Its contributor documentation defines a port by one import route, every ported module reaching Kotlin through a `#kotlin/*` import map. The owners want to see how much TypeScript production code is such a façade, how much is still to be ported and how much the migration's own scaffolding adds, measured on every commit of the branch to show the shift over time.

- Paths cannot say it. A façade shares its folder with TypeScript still to be ported, and a glob list of façades is right only at the commit it was written for, since every port commit turns another file into one. Over a branch's history, such a list claims files before their port.
- An import alone claims too much. A few files in the project import a Kotlin piece and still hold their own TypeScript logic, one of them large; a test for the import alone would count them as façade.
- Whole-text matching claims wrongly through comments. Matching the same patterns against every line instead of code lines demoted a noticeable share of real façades, because their doc comments contain words such as `else`; unanchored keywords matched comment lines in almost every façade.

The deferred entry's earlier requests have the same shape at a smaller scale: a12-rulekit's test lanes are JUnit `@Tag` annotations, which its agent found exactly only with a regex anchored to the start of a line, since unanchored matches hit comments.

## Design

**Required: a `content` condition on refinements.** A refinement may carry `content`, one regular expression. A refinement with `content` matches a file when its other conditions match and at least one code line of the file matches the expression. Everything else about refinements stays: they apply after every classification stage, only to files whose decided purpose equals their `purpose`, the first matching refinement decides, and purpose, totals, `rule` and module credit never change.

```toml
version = 1

[categories]
facade = "TypeScript file whose exports delegate to the Kotlin implementation"

[[refinements]]
id = "typescript-logic"
purpose = "production"
match = ["src/jsMain/typescript/**"]
content = '^\s*(if|for|while|switch|do|try)\b|\belse\b|\?\?|\bcatch\b'
category = "implementation"
reason = "Control flow means the file still holds TypeScript logic, even beside a Kotlin import"

[[refinements]]
id = "kotlin-facades"
purpose = "production"
match = ["src/jsMain/typescript/**"]
content = "from [\"']#kotlin/"
category = "facade"
reason = "A ported module's TypeScript file reaches its Kotlin implementation through the #kotlin import map"
```

The first refinement keeps files with control flow in `implementation`, so the second claims only files that import Kotlin and do nothing else. On the migrating project's branch head, this order separated the façades from the mixed files without a miss in either direction among the files checked; the figures are local.

- **Code lines.** A line is tested when the counter classifies it as code, as `LanguageType::line_classes` in the Tokei fork reports it, the same classification the counts use and the [fork's judges](../research/tokei-fork-step-6.md) verify against each language's own lexer. Comment-only and blank lines are skipped. A line holding code and a trailing comment is tested whole, and string literals are not removed, so anchoring a pattern to the start of the line remains the author's tool against matches in strings.
- **Syntax.** Rust `regex` syntax, with `^` and `$` anchoring each line. It has no look-around or backreferences and matches in linear time, so a pattern cannot stall a measurement. An invalid pattern fails configuration loading with the parser's position and a hint, as other configuration errors do.
- **Reading.** Only files that match a refinement's purpose, `match`, `except` and `kind` are read, once per file, in parallel, as generated-code markers are. A file whose language has no line classes, a Jupyter notebook or a Markdown file whose lines a fenced language counts, does not match a `content` refinement.
- **Evidence.** `explain` names the matching line and its number under `decision.refinement`. `summary.policy_entries` reports for each content refinement, besides the files it relabeled, the files its globs matched but its content did not, so a pattern that silently misses some of its targets stays visible ([agent lesson 2](../agent-lessons.md)). A content refinement that relabels nothing is listed in `summary.unused_entries`, as any refinement is.

**Optional: an absent condition.** `content_absent`, a second expression that must match no code line, after GitHub Linguist's `negative_pattern` in its language heuristics. It would let one refinement claim façades and leave a separate category, such as partially ported files, to another, instead of the shielding refinement above. Include it only if the owner wants such a category; ordering covers the façade case.

**Excluded:**

- Content conditions on overrides and rules. They would let file contents move code between purposes, which the deferred scope rules out.
- Counting or comparing matches, such as "mostly `theorem` declarations". The experiment found it a different feature, closer to per-declaration measurement.
- Structural or semantic matching through syntax trees or query databases (tree-sitter, ast-grep, Semgrep, CodeQL). Tree-sitter stays a test-time reference, not a shipped dependency ([Tokei fork decisions](../plans/archive/tokei-fork.md#decisions)), and LOCmetry keeps no counter of its own ([REJECTED](../REJECTED.md#out-of-scope-by-decision)); CodeQL also needs a paid GitHub Code Security license on private repositories.
- Built-in content heuristics. `content` is the project's judgment, like a `.gitattributes` declaration; built-in checks remain the documented, position-bound [generated-code markers](archive/generated-code-markers.md), since every free-text heuristic measured there claimed hand-written code.
- A category attribute in `.gitattributes`, the alternative bpmn-lean-experiment raised. It goes stale silently and holds no answer for commits before its adoption.

## Companion Items

Both are deferred with their requests in [DEFERRED](../DEFERRED.md#configuration-and-categories), which asks to propose them together with content conditions, since all three narrow what an entry matches. Outlined here for the owner's decision; neither is needed for the migrating project's case.

- **Filter on the deciding rule:** `decided_by = ["builtin:test-directory"]` on a refinement, matching only files that rule or entry decided, which today needs the convention's suffix list copied into `except`. Asked for by a12-rulekit and bpmn-lean-experiment on 0.11.1.
- **Entry references in `except`:** `except_entries = ["observation-bundles"]`, leaving out the files another entry matches without copying its globs. Asked for by bpmn-lean-experiment and the private Lean project on 0.11.1.

## Feasibility

Refinements run in `Policy::refine` over the counted rows after markers, with root-relative paths; a content refinement needs the scan root and a file reader, as the generated-code marker pass already has, and can run in that parallel pass. `RefinementConfig` gains an optional field, the effective configuration records it, and `RefinementEvidence` gains the matched line. The fork already exports `line_classes` and `LineClass` publicly. The `regex` crate is today only a build- and test-time dependency and becomes a runtime one. Measurements without content refinements read no extra file; the cost of one is a second parse of its candidate files, to be measured before acceptance.

## Acceptance

- Test-first integration tests: a pattern on a code line matches; the same text in a comment-only line does not; `except`, `kind` and refinement order still decide; fallback and marker-refined files stay unchanged; purpose totals are equal with and without the refinement; an invalid pattern fails loading with a hint; a notebook does not match; `explain` shows the line; the policy audit reports files the content left out.
- The schema, the configure guide's [refine categories](../guides/configure.md#refine-categories) section, [PRODUCT](../PRODUCT.md#matching-and-precedence) and the CHANGELOG describe the field.
- On a large public repository, measurement time with a content refinement over all of its source is reported; without one, it is unchanged within run-to-run variation.
- The migrating project's façade categories match its local record at its measured commit.

## Open Questions for the Owner

- Whether the deferred entry's last condition, an agent A/B of false claims and cost on the field-sized project, applies here, where the consumers of the categories are people tracking a migration rather than agents reviewing tests.
- Whether to bundle the companion items, and whether to include `content_absent`.
