# Tokei Fork Step 3 Measurement

Measured 2026-10-29 on macOS arm64, for step 3 of the [Tokei fork plan](../plans/archive/tokei-fork.md): interpolation holes whose contents are code, and regular expression literals. The fork is [mbackschat/tokei](https://github.com/mbackschat/tokei) at `139b5d2`, version `15.0.0+locmetry.3`; its README cites the specification or reference documentation behind each declaration. The method and harness are those of [step 1](tokei-fork-step-1.md), with tree-sitter grammars for Dart 0.3.0, Ruby 0.23.1 and TSX added, and with interpolation holes in tree-sitter's string nodes read as code, as the fork reads them.

## Change

The engine gained `interpolations` and `regex_literals` declarations. A hole suspends its string, is read as code that can hold strings, comments and further holes, and resumes the string at its closer outside nested brackets; text that only looks like a hole, such as C#'s `{{`, stays in the string. A `/` starts a regular expression when the code before it, past whitespace, block comments and whole-line comments, ends in an operator, an opening bracket or a keyword such as `return`; after a value, a closing bracket or another identifier it is division.

| Language | Declared |
| --- | --- |
| JavaScript, TypeScript, JSX, TSX | `${…}` holes in template literals; regular expression literals. |
| Kotlin | `${…}` holes in `"…"` and `"""…"""` strings. |
| Swift | `\(…)` holes in `"…"` and `"""…"""` strings. |
| C# | `{…}` holes in `$"…"`, `$@"…"` and `@$"…"` strings with `{{` and `}}` as text, and in `$"""…"""` raw strings with as many braces as `$` signs. |
| Scala | `${…}` holes in `s`, `f` and `raw` interpolated strings, with `$$` as text. |
| Dart | `${…}` holes in single, double and triple-quoted strings; `r` raw strings. |
| Ruby | `#{…}` holes in double-quoted strings; regular expression literals; `$'`, `$"` and `` $` `` globals as code. |
| Shell, Bash, Zsh, Korn shell | `$(…)` holes in double-quoted strings; `'…'` as verbatim and `$'…'` as escaping strings; `#` comments only where a word can start. |

The shell declarations were added during the measurement, which found `"$(…)"` substitutions holding their own quotes in five files. Holes alone misread 22 files that upstream counted correctly, because `#` inside a word, as in `${#a}` or `${x#p}`, then opened a comment inside the hole; the word-start rule of POSIX §2.3 removed those. Ruby's quote-like globals and two engine defects, blank lines inside holes counted as code and a regular expression after a comment line read as division, were also found by the measurement.

Each form has a regression file in the fork. The JavaScript files pass `node --check`, the Ruby files `ruby -c`, the Swift file `swiftc -typecheck`, and the shell file `bash -n` and `zsh -n`; the Kotlin, C#, Scala, Dart and TypeScript files were checked against their tree-sitter grammars, which agree with the hand-derived counts line by line.

## Corpora

The 22 repositories of steps 1 and 2 plus shallow clones chosen for these constructs: lodash `2b5e6f7`, Prettier `5ef89cf`, Vue core `05db7e8`, React `16a1d5d`, Riverpod `33af1e8`, RuboCop `28b550d`, nvm `5e38bc0`, bats-core `5b61d8f`, asdf `ae14f6e`, rbenv `07e9b1e`, acme.sh `37d064f` and Bash-it `72ea9e2`.

## Results

Files whose counts differ from tree-sitter's; the last column counts the fork's differences in files where tree-sitter reports a parse error:

| Language | Files | Tokei | Fork | Parse errors |
| --- | --- | --- | --- | --- |
| C# | 7,381 | 186 | 0 | 0 |
| Dart | 1,311 | 9 | 4 | 0 |
| JavaScript | 9,431 | 171 | 101 | 85 |
| Kotlin | 1,703 | 16 | 11 | 11 |
| Ruby | 1,794 | 413 | 392 | 0 |
| Scala | 835 | 0 | 0 | 0 |
| Shell | 994 | 81 | 49 | 7 |
| Swift | 773 | 27 | 3 | 2 |
| TSX | 239 | 1 | 1 | 0 |
| TypeScript | 2,186 | 27 | 2 | 0 |

No file of any measured language counts differently from tree-sitter with the fork where upstream Tokei agreed with tree-sitter.

## Remaining Disagreements

- **Ruby (392):** 390 files with heredocs, which step 4 covers, and two RuboCop files with percent literals, `%r{…}` and `%w[' "]`.
- **Shell (49):** 45 files with heredocs; in the other four, tree-sitter misreads nested parameter expansions such as `${response#*{\"name\":…}`, and the fork's lines are correct on inspection.
- **JavaScript (101):** 85 files with tree-sitter parse errors, mostly React's Flow and Prettier's test fixtures, of which 84 count as upstream Tokei counts them. Of the other 16, eight hold apostrophes or comment markers in JSX text, which step 5 covers; six hold lines with a block comment followed by a line comment, such as `/* c8 ignore next */ // @ts-expect-error`, which Tokei's whole-line rule counts as code; and two are Prettier fixtures with HTML-like `<!--` comments.
- **Dart (4):** Riverpod snippets whose lines start with `/* SNIPPET START */ ///`, the same whole-line rule.
- **Kotlin (11):** all have tree-sitter parse errors and count as upstream Tokei counts them.
- **Swift (3):** the step-2 remainders: two parse errors and swift-syntax's `LexerTests.swift`.
- **TypeScript (2):** tree-sitter reads a `/**` comment after `instanceof B` as code in Prettier fixtures; the fork's comment lines are correct.
- **TSX (1):** `/*test*/` as JSX text, which step 5 covers.

In the Immich v3.3.4 showcase, two TypeScript files change: 59 code lines become 43 comment and 16 blank lines, and both files now match tree-sitter.

## Speed

Median instructions relative to upstream Tokei 15.0.0, from hyperfine on one thread, ten runs per repository:

| Repository | Step 2 | Step 3 |
| --- | --- | --- |
| tokio | 0.95 | 0.97 |
| swift-syntax | 0.92 | 0.96 |
| dotnet/aspire | 0.78 | 0.85 |
| lodash | 0.85 | 0.83 |
| Prettier | 0.94 | 0.83 |
| React | 0.89 | 1.01 |
| RuboCop | 0.94 | 0.94 |
| Riverpod | 1.14 | 0.95 |
| acme.sh | 0.87 | 0.97 |
| Bash-it | 0.95 | 0.99 |

The first step-3 build reached 1.21 on aspire and 1.16 on acme.sh: it checked for holes at every byte of a string, and it sought the line start for every `/`, which made aspire's minified JavaScript cost 2.87 times upstream. Gating holes by their first bytes and seeking a line start only after the look-back crosses a newline brought the costs to the table's values without changing any count.

## Assessment

Verified: on these repositories the fork matches tree-sitter for C# and Scala, it never disagrees where upstream agreed, and every remaining difference is a construct scheduled for steps 4 or 5, a tree-sitter misreading or parse error, or one of the uncorrected rules listed in [Tokei counting defects](../tokei-defects.md). Speed stays within upstream's, React's 1.01 within run-to-run variation of the other repositories.

Limits: one machine and one run of each corpus; the tree-sitter misreadings rest on inspection; and the regular expression rule is an approximation of ECMAScript's syntactic goal symbols and Ruby's lexer state, which a comment after code on the preceding line, or a regular expression after `}` or `<`, still defeats.
