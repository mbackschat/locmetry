# Correct Counting in a Tokei Fork

<!-- doc-status
{
  "status": "active",
  "date": "2026-10-29",
  "summary": "Fix Tokei's literal and comment defects in a fork, in five feature steps that each cover every language with the construct.",
  "evidence": "[counting backends](../research/counting-backends.md), [Tokei counting defects](../tokei-defects.md), [step 1](../research/tokei-fork-step-1.md), [step 2](../research/tokei-fork-step-2.md) and [step 3](../research/tokei-fork-step-3.md) measurements"
}
-->

[Tokei counting defects](../tokei-defects.md) lists what Tokei 15.0.0 miscounts, and the [counting backends spike](../research/counting-backends.md) chose the remedy. This plan owns the remaining work.

## Decisions

Taken by the owner on 2026-10-29:

- Correct counting in a fork of Tokei; use tree-sitter as a test-time reference on real repositories, not as a shipped dependency; drop the spike's hand-written lexers.
- Work in five feature steps, in the order below. Each feature is written once in the fork and switched on for every language that has the construct.
- The fork's working copy is `../tokei` next to this repository, on branch `locmetry` from `v15.0.0` (`c14f744`). Create its GitHub repository at the end of step 1, when Locmetry first needs a pinned git dependency; until then Locmetry may depend on the local path. Created on 2026-10-29 as [mbackschat/tokei](https://github.com/mbackschat/tokei), with `locmetry` as its default branch.
- Counts change for most languages, so the release that ships the fork carries a **Breaking** CHANGELOG entry.

## Every Step

Each step is complete when all of these hold for its languages:

- [ ] Each declaration cites the language's specification or reference lexer, as the [research rules](../../CLAUDE.md#research-and-experiments) require.
- [ ] Probe files with hand-derived counts cover each construct and pass the language's compiler or syntax checker where one is available; they become tests in the fork.
- [ ] Public repositories for each language, at recorded commits, are measured against tree-sitter, and every remaining disagreement is explained.
- [ ] Speed stays within Tokei's on the spike's benchmark set.
- [ ] The fork's README lists its changes from upstream, each with its commit and upstream pull request.
- [ ] [Tokei counting defects](../tokei-defects.md) moves the fixed entries to the corrected section, with the fork and Locmetry commits; [measurement semantics](../measurement.md#counting) states the new counting rules.

## Step 1: Character Literals and Missing Declarations

- [x] Character literals in the engine, from the spike prototype: an escape running to the closing quote on the same line, or exactly one character, so Rust lifetimes and Haskell primes stay code.
- [x] Declare character literals for C, C++, Java, Kotlin, C#, Go, Rust, Scala, OCaml and F#.
- [x] Declare quotes, with character literals where the language has them, for Haskell, Elm, PureScript, Agda, Racket, Scheme and Lisp.
- [x] Go backtick strings as verbatim quotes, with verbatim openers added to the per-line fast path.
- [x] Java text blocks.
- [x] Rust raw strings up to the delimiter counts step 2 generalizes.
- [x] Exceptions to comment markers: PHP `#[` attributes and the F# `(*)` operator.
- [x] `""` escapes in C# and F# `@"…"` strings, found by the measurement.
- [x] Decide whether to replace Tokei's text-based rule that counts a line starting with a comment marker as a comment even inside a literal: replaced for lines that start inside a literal, which the measurement showed in Go, C# and PHP files.
- [x] Add repositories for Kotlin, C++, C#, Scala, OCaml, Haskell and PHP to the measurement.
- [x] Preserve the tree-sitter reference harness and probes in the fork as development tools.
- [x] Switch Locmetry to the fork and keep its Lean, Handlebars and notebook counters. Record the fork, its pinning and its upgrade and upstreaming policy in [Tokei integration](../tokei-integration.md#counting-boundary), adjust the embedded-Tokei statement in [PRODUCT](../PRODUCT.md), and add the CHANGELOG entry.
- [x] Create the fork's GitHub repository and pin Locmetry's dependency to a commit.
- [ ] Verify the Linux musl, Windows and Homebrew builds.
- [ ] Offer the character-literal and Go backtick changes upstream, referring to [#1389](https://github.com/XAMPPRocky/tokei/pull/1389) and [#1351](https://github.com/XAMPPRocky/tokei/pull/1351).

## Step 2: Raw Strings With Repeated Delimiters

- [x] An engine feature for raw strings whose closing delimiter repeats the opening count or names a label.
- [x] Rust `r#…#"` with any count, Swift `#"…"#`, C# `"""…"""`, C++ `R"x(…)x"`, Lua `[==[…]==]` and OCaml `{id|…|id}`.
- [x] Lua `--[==[…]==]` long comments and Swift's one-line reading of `#"""#`, found while implementing and measuring.
- [x] Measured against tree-sitter, adopted in Locmetry `51d9664` with fork `15.0.0+locmetry.2`; see the [step 2 measurement](../research/tokei-fork-step-2.md).

## Step 3: Interpolation and Regular Expressions

- [x] Interpolation whose terms are code and can hold their own strings: JS and TS `` `${…}` ``, Kotlin `"${…}"`, Swift `"\(…)"`, C# `$"{…}"`, Scala `s"${…}"`, Dart `"${…}"` and Ruby `"#{…}"`.
- [x] Regular expression literals, recognized by the previous token: JS, TS and Ruby.
- [x] Shell `"$(…)"` substitutions, verbatim `'…'` strings and word-start `#` comments, and Ruby's `$'`, `$"` and `` $` `` globals, found by the measurement.
- [x] Adopted in Locmetry `714f778` with fork `15.0.0+locmetry.3`; see the [step 3 measurement](../research/tokei-fork-step-3.md).

## Step 4: Heredocs

- [ ] An engine change for closing delimiters taken from the file, since Tokei stores them as static strings.
- [ ] Shell, Ruby, PHP and Perl heredocs, including quoted and indented delimiters and here-strings.

## Step 5: Apostrophes in Markup Text

- [ ] Quotes that apply only inside tags or expressions, not in text: HTML, JSX, TSX, Vue, Svelte, XML and YAML plain scalars.

## Out of Scope

Python, which counted correctly in the spike; Tokei's rule that reads Rust doc comments as Markdown; and Perl and Ruby quote operators such as `q{}` and `%q()`, unless measurements show them.
