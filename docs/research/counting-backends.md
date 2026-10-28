# Counting Backends Beyond Tokei

Evaluated 2026-10-28 and 2026-10-29 on macOS arm64. [Tokei counting defects](../tokei-defects.md) records the defects that prompted this spike: Tokei 15.0.0 lexes every language with one model of line comments, block comments and quote pairs, so character literals, raw strings, regular expressions, heredocs and interpolation turn comments into code or code into comments.

## Question

Which backend should correct those defects for mainstream languages, judged by accuracy on real code, speed, binary size and maintenance?

| Candidate | What was built |
| --- | --- |
| Tokei 15.0.0 | The embedded counter, as the baseline. |
| tree-sitter | tree-sitter 0.27.1 with the crates.io grammars for Rust 0.24.2, C 0.24.2, Go 0.25.0, Java 0.23.5, JavaScript 0.25.0, TypeScript 0.23.2, Python 0.25.0 and Bash 0.25.1; comment and literal nodes give byte ranges. |
| Family lexers | Two hand-written lexers for the spike: a C-like one for C, Java, Go, Rust and JS/TS, and one for Python and Shell. Their rules were written from general knowledge, not from cited language specifications. |
| Tokei fork | Tokei v15.0.0 (`c14f744`) with three additions: character literals in the engine, declared for C, C++, Rust, Java, Go, Kotlin, C# and OCaml; Go backtick strings and Rust `r"`, `br"`, `cr"` and `r###"` raw strings as verbatim quotes; and Java text blocks as a quote. Verbatim-quote openers were added to the engine's per-line fast path, without which a line holding only a backtick skips analysis. |

## Method

Every candidate except the fork reports comment and literal byte ranges, and one classifier applies Tokei's own line rules to them, so differences come from lexing alone: a line inside a literal is code, a line that starts inside a comment is a comment, and otherwise a line is a comment when its first and last non-blank characters are. The fork uses Tokei's own classification.

- **Probes:** 25 synthetic files with hand-derived counts, one per literal or comment construct. The Rust, C, Java, JavaScript and Shell probes and two of the three Python probes passed their compilers or syntax checkers. The nested f-string probe needs Python 3.12 (PEP 701) and the local Python was 3.9; the Go and TypeScript probes were not checked.
- **Real code:** 32,045 files (326 MB) from 14 public repositories, shallow clones of their default branches on 2026-10-28: ripgrep `91f5d22`, tokio `e0f829f`, cobra `adbc881`, hugo `c2f9cb8`, redis `a76a368`, curl `af34974`, guava `ae0fa4b`, prettier `a08e865`, lodash `2b5e6f7`, vscode `de09766`, django `2d9d619`, requests `3c7baaa`, nvm `5e38bc0` and ohmyzsh `17a2b4c`. Files were selected by extension: `.rs`, `.c`, `.h`, `.go`, `.java`, `.js`, `.mjs`, `.cjs`, `.ts`, `.mts`, `.cts`, `.py`, `.sh` and `.bash`.
- **Adjudication:** for each file where counters disagree, Tokei's per-line classes were recovered from the counts of growing prefixes, and the disagreeing lines were bucketed and sampled by hand.
- **Speed:** one thread, timing each counter on text already in memory, over curl, django, guava, hugo, prettier and tokio (81.5 MB).

## Results

### Probes

Tokei counted 10 of 25 correctly, tree-sitter and the family lexers 25. The fork counted 19: every probe its additions cover, and none of the six JS regular expression, template literal and Shell heredoc probes it does not cover.

### Real code

Files whose counts differ from tree-sitter's:

| Language | Files | Tokei | Family lexers | Tokei fork |
| --- | --- | --- | --- | --- |
| C | 1,929 | 21 | 0 | 0 |
| Go | 952 | 196 | 0 | 1 |
| Java | 3,282 | 4 | 0 | 0 |
| JavaScript | 5,497 | 69 | 27 | 69 |
| Python | 3,164 | 0 | 0 | 0 |
| Rust | 1,037 | 493 | 0 | 490 |
| Shell | 234 | 27 | 7 | 27 |
| TypeScript | 15,950 | 253 | 5 | 253 |

The fork's JavaScript, TypeScript and Shell rows equal Tokei's because none of its additions applies to them.

Verified by sampled lines:

- Where Tokei alone disagrees, Tokei was wrong in every sample: character literals such as `'"'` in curl, guava, hugo and a vscode Rust file; a raw string ending in a backslash, `r"\"`, in ripgrep; Go backtick strings in hugo tests; and JS and TS files in lodash and vscode in which a misread quote turned comment and blank lines into code.
- All 490 Rust files where the fork differs from tree-sitter differ only in `///` and `//!` lines: Tokei reads Rust doc comments as Markdown, so an empty `///` is blank and code examples are code. This is Tokei's deliberate rule, not a lexing defect.
- The one Go file is a line starting with `//` inside a raw string that closes on that line. Tokei classifies a line as a comment when its text starts with a comment marker, regardless of lexical state.
- tree-sitter is not a perfect reference on code its grammar cannot parse. It reported parse errors in 1,426 JavaScript and 688 C files. The C files still matched the lexers exactly, but in JavaScript, Flow type annotations in prettier's fixtures made it count a real `/* … */` comment as code, and in Shell, zsh syntax in `.sh` files made it count comments as code. The spike's tree-sitter classifier also did not descend into template literals, so a comment inside `${ … }` counted as literal.
- The family lexers' remaining differences are those tree-sitter errors, plus a Shell parser fixture with an unterminated quote, where no count is correct. The lexers also had a here-string bug, `<<<` read as a heredoc, that turned the rest of a vscode script into a literal until fixed: an instance of the runaway these candidates are meant to remove.

### Speed and size

Single-thread throughput was 104 MB/s for Tokei, 98 MB/s for the fork, 181 MB/s for the family lexers and 7.2 MB/s for tree-sitter. Parallel runs varied between runs and are not reported.

The fork adds no measurable size, since it keeps Tokei's engine and generated tables. A separate measurement on 2026-10-28 with Locmetry's release profile (stripped, thin LTO) found the tree-sitter runtime at 0.3 MB, 14 lighter grammars (Rust, C, JS, TS, TSX, Go, Java, Python, Ruby, PHP, Bash, HTML, CSS and YAML) at 10.9 MB, and those plus C++, C#, Swift, Kotlin, Haskell and OCaml at 35.0 MB, against Locmetry's 7.5 MB.

## Assessment

Verified: Tokei's literal defects affect real repositories, and the fork's three additions remove them for C, Java, Go and Rust without measurable cost in speed or size, matching tree-sitter in every adjudicated line apart from Tokei's deliberate Rust doc-comment rule and its text-based comment rule.

Plausible but not built: regular expressions and template literal nesting for JS and TS, heredocs for Shell, Ruby, PHP and Perl, raw strings with any number of delimiters for Rust, Swift, C++ and C#, and apostrophes in markup text. Heredocs need an engine change, because Tokei stores the closing quote as a static string and a heredoc's delimiter comes from the file. The family lexers show that regular expressions and template nesting work with a previous-token rule and a brace stack, on the evidence of the probes and the JS and TS corpora.

Recommendation: correct counting in a Tokei fork, adding one feature at a time with each language's declarations cited from its specification, and use tree-sitter as a test-time reference on real repositories rather than shipping it. tree-sitter costs 10.9 to 35 MB and runs about 14 times slower than Tokei, and its error recovery misclassifies code its grammar cannot parse. The family lexers are fast and accurate on this sample, but they duplicate Tokei's language table and detection, and each corner case has to be found and fixed by hand.

## Limits

- The family lexers and fork declarations were written for the spike without cited sources; shipping any of them requires the research rules' primary-documentation grounding.
- Speed and size were measured on one macOS arm64 machine. Linux musl, Windows and Homebrew builds with tree-sitter's C code were not tried.
- The harness and fork lived in a session scratchpad and are not preserved; the commits above and the method allow the measurement to be repeated.
