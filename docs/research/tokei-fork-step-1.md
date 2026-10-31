# Tokei Fork Step 1 Measurement

Measured 2026-10-29 on macOS arm64, for step 1 of the [Tokei fork plan](../plans/archive/tokei-fork.md): character literals, missing quote declarations, Go backtick strings, Java text blocks, Rust raw strings with up to three `#`, comment-marker exceptions, and two defects this measurement found. The fork is [mbackschat/tokei](https://github.com/mbackschat/tokei) at `23bde34`, version `15.0.0+locmetry.1`; its README lists each change with the specification or reference lexer it follows. [Counting backends](counting-backends.md) describes the harness and why tree-sitter serves as the reference.

## Method

The spike harness, preserved in the fork's `locmetry-spike/` folder, counted every file selected by extension with Tokei 15.0.0, tree-sitter and the fork, applying Tokei's line rules to tree-sitter's comment and literal ranges. For each file where the fork and tree-sitter differ, the first disagreeing line and the lines before it were inspected.

Corpora were shallow clones of 17 public repositories: ripgrep `91f5d22`, tokio `e0f829f`, hugo `c2f9cb8`, curl `a7023ca`, guava `ae0fa4b`, kotlinx.coroutines `7e0d1b3`, okhttp `a11e55f`, fmt `f87a75e`, leveldb `bb74ef7`, Newtonsoft.Json `52fa3ae`, AutoMapper `63f41ac`, cats `b055e31`, dune `7e41f64`, pandoc `8605c4a`, laravel/framework `6065d21`, symfony `5c0e37b` and FSharpPlus `b5a650a`. The added tree-sitter grammars were Scala 0.26.3, F# 0.3.12, Kotlin (kotlin-ng) 1.1.0, C++ 0.23.4, C# 0.23.5, OCaml 0.26.0, Haskell 0.24.1 and PHP 0.25.1, beside the spike's.

## Results

Files whose counts differ from tree-sitter's, out of 29,399:

| Language | Files | Tokei | Fork |
| --- | --- | --- | --- |
| C | 1,268 | 9 | 1 |
| C++ | 136 | 1 | 0 |
| C# | 1,468 | 30 | 0 |
| F# | 146 | 8 | 0 |
| Go | 912 | 186 | 0 |
| Haskell | 366 | 2 | 2 |
| Java | 3,368 | 4 | 0 |
| Kotlin | 1,703 | 16 | 11 |
| OCaml | 1,896 | 31 | 23 |
| PHP | 16,118 | 3,378 | 244 |
| Rust | 916 | 448 | 448 |
| Scala | 835 | 0 | 0 |

Python, JavaScript, TypeScript and Shell files in these repositories counted as with Tokei; steps 3 and 4 of the plan cover the last three.

The first pass found two defects the plan had not listed, both fixed in the fork before the final run:

- **C# and F# verbatim strings:** Tokei closed `@"…"` at the first quote of a `""` escape and reopened a regular string at the second. Where a backslash precedes the pair, as in `@"x\""y\""z"`, the regular string read `\"` as an escape and string state flipped. This accounted for 12 of the 18 remaining C# files.
- **Lines that start inside a literal:** Tokei called a line a comment whenever its trimmed text began with a comment marker, regardless of lexical state, so a raw-string line such as `//# sourceMappingURL=…` ending a Go backtick string, or `//comment 1 ";` ending a C# verbatim string, counted as a comment. This accounted for the other 6 C# files, the Go file and a PHP file.

## Remaining Disagreements

Each remaining file was inspected at its first disagreeing line.

- **Rust (448):** only `///` and `//!` lines, which Tokei deliberately reads as Markdown, as in the spike.
- **Kotlin (11), C (1):** every file has a tree-sitter parse error, for example Kotlin's `$$"…"` multi-dollar strings; the first disagreeing line is a comment or blank line that the fork counts correctly.
- **Haskell (2):** in pandoc, `-- |` comment lines after CPP `#ifdef` blocks and a blank line after `char '\n'`, which tree-sitter counts as code; the fork is correct.
- **OCaml (23):** every file has a quoted string `{|…|}` or `{id|…|}` before its first disagreement, which step 2 covers.
- **PHP (244):** 220 files have a heredoc or nowdoc before their first disagreement, which step 4 covers. 17 are Blade templates whose `{{-- … --}}` comments the fork counts as comments and tree-sitter as text. 6 start with a `#!/usr/bin/env php` line, which the fork counts as a comment and tree-sitter as text; PHP skips it as an interpreter line. One is a JavaScript `//` comment in the HTML part of a PHP template.

## Speed

Upstream Tokei 15.0.0 and the fork were built as release binaries and timed with hyperfine on one thread, ten runs per repository. Instruction counts were stable where per-file wall-clock timings varied severalfold between identical runs.

| Repository | Before the opener gate | Fork |
| --- | --- | --- |
| tokio | 1.30 | 1.05 |
| ripgrep | 1.40 | 0.95 |
| pandoc | 1.41 | 1.19 |
| curl | 1.03 | 0.90 |
| guava | 1.05 | 0.90 |
| hugo | 1.08 | 0.93 |
| symfony | 1.05 | 0.80 |
| dune | 1.02 | 0.96 |
| Newtonsoft.Json | 1.01 | 0.88 |

The figures are median instructions relative to upstream. Before the gate, every byte of an analysed line was matched against each opener, including Rust's twelve raw-string openers. The gate skips bytes that cannot start a token and left the fork's counts unchanged on all 29,399 files. Haskell stays slower because its strings are now lexed; Tokei skipped them.

## Assessment

Verified: on these repositories the fork matches tree-sitter for C++, C#, F#, Go, Java and Scala, and every remaining difference in the other step-1 languages is a tree-sitter parse error, Tokei's deliberate Rust doc-comment rule, a construct scheduled for a later step, or a PHP interpreter or template line where the fork's comment count is defensible. Speed is at or below upstream except for Haskell and tokio.

Limits: one machine and one run of each corpus; the Kotlin, C and Haskell remainders rest on inspection, because tree-sitter misparsed those files; and the corpora hold no Elm, PureScript, Agda or Lisp-family code, whose declarations rest on their test files and cited lexers.
