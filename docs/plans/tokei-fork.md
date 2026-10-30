# Correct Counting in a Tokei Fork

<!-- doc-status
{
  "status": "active",
  "date": "2026-10-29",
  "summary": "Fix Tokei's literal and comment defects in a fork, in five feature steps that each cover every language with the construct.",
  "evidence": "[counting backends](../research/counting-backends.md), [Tokei counting defects](../tokei-defects.md), [step 1](../research/tokei-fork-step-1.md), [step 2](../research/tokei-fork-step-2.md), [step 3](../research/tokei-fork-step-3.md), [step 4](../research/tokei-fork-step-4.md) and [step 5](../research/tokei-fork-step-5.md) measurements, and the [step 5 resolution](../research/tokei-fork-step-5-resolution.md)"
}
-->

[Tokei counting defects](../tokei-defects.md) lists what Tokei 15.0.0 miscounts, and the [counting backends spike](../research/counting-backends.md) chose the remedy. This plan owns the remaining work.

## Decisions

Taken by the owner on 2026-10-29:

- Correct counting in a fork of Tokei; use tree-sitter as a test-time reference on real repositories, not as a shipped dependency; drop the spike's hand-written lexers.
- Work in five feature steps, in the order below. Each feature is written once in the fork and switched on for every language that has the construct.
- The fork's working copy is `../tokei` next to this repository, on branch `locmetry` from `v15.0.0` (`c14f744`). Create its GitHub repository at the end of step 1, when Locmetry first needs a pinned git dependency; until then Locmetry may depend on the local path. Created on 2026-10-29 as [mbackschat/tokei](https://github.com/mbackschat/tokei), with `locmetry` as its default branch.
- Counts change for most languages, so the release that ships the fork carries a **Breaking** CHANGELOG entry.

Taken by the owner later on 2026-10-29, for step 6:

- Every popular language counts with no difference from its judge: the language's own lexer or parser wherever one can run, and tree-sitter only where none can and it parses the file without error. A difference is a defect to fix, not to explain.
- The languages are the 39 top-ranked ones, Python, JavaScript, TypeScript, TSX and JSX, Java, C, C++, C#, Go, Rust, Kotlin, Swift, PHP, Ruby, Shell, PowerShell, SQL, HTML, CSS, SCSS, Markdown, YAML, JSON, XML, TOML, Dart, Scala, Lua, Perl, R, Objective-C, Haskell, Elixir, Groovy, Vue, Svelte, Dockerfile, Makefile and Lean, together with every other language Locmetry reports for the measured repositories.
- A line counts as a comment only when it holds nothing but comments, so `/* a */ x = 1; /* b */` is code; PlantUML, whose block comments are line-based, keeps Tokei's rule.

## Every Step

Each step is complete when all of these hold for its languages:

- [ ] Each declaration cites the language's specification or reference lexer, as the [research rules](../../CLAUDE.md#research-and-experiments) require.
- [ ] Probe files with hand-derived counts cover each construct and pass the language's compiler or syntax checker where one is available; they become tests in the fork.
- [ ] Public repositories for each language, at recorded commits, are measured against tree-sitter, and every remaining disagreement is explained.
- [ ] Parsing stays within 10% of upstream Tokei 15.0.0's time on every [benchmark repository](../research/tokei-fork-step-6.md#speed), by the fork's speed gate, `judges/speed/check.py`, before the release is adopted; a correction that needs more must first win back the time elsewhere. Decided 2026-10-30 by the owner.
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
- [x] Verify the Linux musl and Homebrew builds with fork `15.0.0+locmetry.11`, on 2026-10-29: static `x86_64` and `aarch64` musl binaries report the same measurement as the macOS build in Debian and Alpine, the public tests pass in Alpine from a clean clone that fetches the fork from GitHub, and the formula's `--HEAD` install and `brew test` pass in the `homebrew/brew` image.
- [ ] Link the Windows build: Locmetry and the fork compile for `x86_64-pc-windows-msvc`, but linking needs the MSVC toolchain of the release workflow's Windows runner.
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

- [x] An engine change for closing delimiters taken from the file, since Tokei stores them as static strings.
- [x] Shell, Ruby, PHP and Perl heredocs, including quoted and indented delimiters and here-strings.
- [x] Ruby regular expressions at the start of a line, found by the measurement.
- [x] Adopted in Locmetry `f9276a7` with fork `15.0.0+locmetry.4`; see the [step 4 measurement](../research/tokei-fork-step-4.md).

## Step 5: Apostrophes in Markup Text

- [x] Quotes that apply only inside tags or expressions, not in text: HTML, XML, SVG, XAML, Vue, Svelte and ERB tags, JSX elements in JavaScript, JSX and TSX, and YAML plain scalars.
- [x] YAML block scalars, `''` escapes in `'…'` strings, comments between JSX attributes and TypeScript type arguments in JSX tags, found by the measurement.
- [x] Adopted in Locmetry `530deed` with fork `15.0.0+locmetry.6`; see the [step 5 measurement](../research/tokei-fork-step-5.md).
- [x] Every remaining disagreement with tree-sitter corrected in the fork or decided file by file with a second reference, which also corrected Rust doc comments, comment-only lines, embedded script and style tags, JavaScript HTML-like comments, YAML plain scalars and flow collections, Perl POD, data sections, quote-like operators and references, Ruby percent literals, and Blade, Twig and Handlebars text; adopted in Locmetry `b8ca6f4` with fork `15.0.0+locmetry.9`, with YAML back at upstream speed in `db13933` with fork `15.0.0+locmetry.10`, and faster than upstream in `1b69b69` with fork `15.0.0+locmetry.11`; see the [step 5 resolution](../research/tokei-fork-step-5-resolution.md).

## Step 6: No Difference From Each Language's Judge

- [x] Count a line holding code between comments as code, in the fork, in Locmetry's Lean counter and in the judges' line rules: fork `655d44c`, released as `15.0.0+locmetry.12` with the NQP and WenYan corrections that the rule exposed, and Locmetry `a287b10`, on 2026-10-29; the fork's speed stayed within run-to-run variation on Prettier, Tokio and Lean 4.
- [ ] A judge per language, recorded with its version: Lean's lexer for Lean; Babel and Hermes for JavaScript and Flow; the TypeScript scanner for TypeScript and TSX; `javac`'s tokenizer for Java; the Kotlin compiler's lexer for Kotlin; `rustc_lexer` for Rust; clang's raw lexer for C, C++ and Objective-C; Roslyn for C#; `go/scanner` for Go; swift-syntax for Swift; PHP's `token_get_all`; Prism for Ruby; Python's `tokenize`; the PowerShell tokenizer; parse5 for HTML, Vue and Svelte markup; a CSS Syntax Level 3 tokenizer for CSS and `postcss-scss` for SCSS; CommonMark for Markdown fences; PyYAML for YAML; expat for XML; R's parse data; Elixir's `Code.string_to_quoted_with_comments`; the Dart scanner; the Scala 3 scanner; the Groovy parser; PPI for Perl; GHC's lexer for Haskell; and, where no lexer can run, tree-sitter with checks by the language's own syntax checker. Done, with no difference on any valid file, for Lean, Rust, Java, Kotlin, JavaScript and TypeScript on 2026-10-29 with fork `15.0.0+locmetry.13`, for Python with fork `15.0.0+locmetry.14`, for C, C++, Objective-C, Go, C# and Ruby with fork `15.0.0+locmetry.15`, for PHP, Swift, Dart and Scala with fork `15.0.0+locmetry.16`, for R with fork `15.0.0+locmetry.17`, for Elixir and PowerShell with fork `15.0.0+locmetry.18`, and for Haskell with fork `15.0.0+locmetry.19`, as the [step 6 record](../research/tokei-fork-step-6.md) shows.
- [ ] Corpora: the 40 repositories of step 5, large public repositories for each language, such as rust-lang/rust, JetBrains/kotlin, microsoft/vscode, spring-framework and mathlib4, Locmetry itself, and a12-rulekit, bpmn-lean-experiment and a private Lean project, whose records stay in `docs/research/local/`.
- [ ] Every file of every corpus counts as its judge counts it, each correction test-first in the fork or in Locmetry's Lean counter.
- [ ] Where a language has no official lexer, as for Shell, Perl, Lua, SQL, Markdown and YAML, the judge is a named reference, and agreement with it is the criterion; distinctions only a parser can make, such as a Ruby local variable before ` /`, are listed in [Tokei counting defects](../tokei-defects.md) instead of being approximated further. Decided 2026-10-30 by the owner.
- [ ] Before Perl and Shell, move the language-specific lexing rules that the declarations now switch on, such as Ruby's, out of the generic engine into per-language modules, as Locmetry's Lean and Handlebars counters are, as a change of its own. Decided 2026-10-30 by the owner.

## Found by the Measurements

Not yet scheduled; each needs the owner's decision before it becomes a step.

- [ ] Template languages that still declare quotes for the whole file, Liquid, Mustache, Hamlet and Haml, where an apostrophe in text opens a string; none occurs in the measured repositories.

The Perl, Ruby percent-literal, Twig and Blade, and comment-only-line items found by the step 3 to 5 measurements were corrected by the [step 5 resolution](../research/tokei-fork-step-5-resolution.md).

## Out of Scope

Python, which counted correctly in the spike. Tokei's rule that read Rust doc comments as Markdown was out of scope until the step 5 resolution, where tree-sitter showed it miscounting empty doc lines; the fork now counts doc comments as comments.
