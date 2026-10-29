# Tokei Fork Step 6: Each Language's Judge

Measured from 2026-10-29 on macOS arm64, for [step 6](../plans/tokei-fork.md#step-6-no-difference-from-each-languages-judge): every file of every corpus must count as the language's own lexer or parser counts it. A line is a comment when it holds nothing but comments; a blank line takes the class of the comment or literal it lies in; every other line is code. A file the judge itself rejects as lexically invalid has no correct count, so it is listed rather than matched.

The judges and the comparison tool live in the fork's [judges](https://github.com/mbackschat/tokei/tree/locmetry/judges) folder from `a2a9724`. Each judge reports the comment and literal ranges its lexer found, and `judgecmp` compares the lines they imply with the fork's `line_classes` (fork `238b0c5`), line by line. The fork's results below are those of `15.0.0+locmetry.13`, fork `a55e730`.

## Lean

Judge: Lean 4.31.0's own lexer functions, run from the toolchain: `whitespace` and `finishCommentBlock` for comments, `tokenFn` for string, character and raw string literals, and `interpolatedStr` with Lean's term parser after `s!`, `f!`, `m!`, `throwError`, `throwErrorAt`, `println!` and `trace[…]`, and for any string that only parses as interpolated, such as one holding `\{`. Counted: Locmetry's [Lean counter](../../src/backend/lean.rs), compared line by line.

| Repository | Commit | Lean files |
| --- | --- | --- |
| [leanprover-community/mathlib4](https://github.com/leanprover-community/mathlib4) | `06a6c7b` | 9,282 |
| [leanprover/lean4](https://github.com/leanprover/lean4) | `5fe62b1` | 7,823 |
| [leanprover/verso](https://github.com/leanprover/verso) | `8f508e9` | 431 |
| [leanprover-community/batteries](https://github.com/leanprover-community/batteries) | `6d971e7` | 275 |
| [leanprover-community/aesop](https://github.com/leanprover-community/aesop) | `7d11bfa` | 250 |
| [leanprover-community/ProofWidgets4](https://github.com/leanprover-community/ProofWidgets4) | `16259c0` | 49 |
| [leanprover/doc-gen4](https://github.com/leanprover/doc-gen4) | `20afd4d` | 49 |
| [leanprover/lean4-cli](https://github.com/leanprover/lean4-cli) | `c09cb1e` | 5 |

On these 18,164 files, 3,767,576 lines, the counter equals the judge on every line of every valid file. Seven lean4 test files are lexically invalid on purpose; three of them differ: `tests/elab_fail/string_gaps_err_newline.lean` and `tests/elab_fail/tokenErrors.lean`, which test the lexer's errors, and `tests/server_interactive/completionIStr.lean`, an editor completion test whose `s!"testing {c. "` leaves a term unfinished. The owner's two Lean projects, bpmn-lean-experiment and a private project, also match on every line; their records stay local.

## Rust

Judge: rustc's own lexer, `ra-ap-rustc_lexer` 0.179.0, run as rustc runs it on a file: the shebang stripped, frontmatter allowed, and a guarded-string prefix `#"` split into `#` and a string, as `rustc_parse` splits it before Rust 2024. A shebang counts as a comment, as rustc skips it as it skips comments and as the other judged lexers name it; a frontmatter counts as a literal, so its lines, blank ones included, are code.

| Repository | Commit | Rust files | Lexically invalid |
| --- | --- | ---: | ---: |
| [rust-lang/rust](https://github.com/rust-lang/rust) | `96a18bf5f` | 39,437 | 133 |
| [bevyengine/bevy](https://github.com/bevyengine/bevy) | `ae61521` | 1,964 | 0 |
| [rust-lang/rust-analyzer](https://github.com/rust-lang/rust-analyzer) | `c59cdaa` | 1,498 | 63 |
| [denoland/deno](https://github.com/denoland/deno) | `e7b39ebd` | 1,052 | 0 |
| [tokio-rs/tokio](https://github.com/tokio-rs/tokio) | `6470b5e` | 814 | 0 |
| Locmetry `src` and `tests` | `7b18854` | 125 | 0 |

Every valid file matches on every line. The lexically invalid files are rustc's and rust-analyzer's lexer-error tests and fuzz failures; 9 of them, unclosed or malformed frontmatter tests in `tests/ui/frontmatter`, differ. Before the fork's corrections, 20 valid files differed: cargo-script frontmatter read as Rust, upstream Tokei's `#"…"#` quote, which rustc never reads as one string, a blank line holding U+2028, shebangs, and a `#!` that a comment separates from `[`, which rustc reads as an inner attribute.

## Java

Judge: javac's scanner from JDK 27.0.1, through `ScannerFactory` with a diagnostic listener counting lexical errors. javac skips a `//` comment that ends the input without a line terminator but reports it only when more input follows (`JavaTokenizer.readToken`), so the judge restores it as the end-of-line comment JLS §3.7 defines.

| Repository | Commit | Java files |
| --- | --- | ---: |
| [spring-projects/spring-framework](https://github.com/spring-projects/spring-framework) | `64861cb` | 9,293 |
| [apache/kafka](https://github.com/apache/kafka) | `14ee92c` | 6,283 |
| [JetBrains/kotlin](https://github.com/JetBrains/kotlin) | `548305369` | 4,235 |
| [google/guava](https://github.com/google/guava) | `3d5be86` | 3,275 |

Every file matches on every line, Markdown `///` doc comments included, and none is lexically invalid. The fork needed no correction.

## Kotlin

Judge: `KotlinLexer` from `kotlin-compiler-embeddable` 2.4.0. String parts, quotes, escapes and character literals are literals; `${…}` template entries are code with their own strings and comments. `BAD_CHARACTER`, `DANGLING_NEWLINE` and unclosed comments or strings make a file invalid.

| Repository | Commit | Kotlin files | Lexically invalid |
| --- | --- | ---: | ---: |
| [JetBrains/kotlin](https://github.com/JetBrains/kotlin) | `548305369` | 70,671 | 129 |
| [spring-projects/spring-framework](https://github.com/spring-projects/spring-framework) | `64861cb` | 396 | 0 |
| [google/guava](https://github.com/google/guava) | `3d5be86` | 2 | 0 |

Every valid file matches on every line. The invalid files are the compiler's diagnostics and recovery tests and multi-file test containers whose C sections are not Kotlin. Before the fork's corrections, 42 valid files differed: raw strings ending in more than three quotes, which Kotlin closes at the last three (`THREE_OR_MORE_QUO` in Kotlin.flex), `$$"…"` strings, whose `${` is text, backtick identifiers holding quotes, and shebangs.

## JavaScript and TypeScript

Judges: for JavaScript, JSX and Flow, Babel 7.28.5 with JSX and Flow, then with the proposal plugins Prettier's tests use, then Hermes 0.36.1; for TypeScript and TSX, and for JavaScript no other parser accepts, the TypeScript 6.0.3 parser, whose comments are the trailing and leading trivia ranges of every token. TypeScript 7 is a native port of that scanner without a stable JavaScript API. A hashbang is a comment (ECMA-262 2025 §12.5). JSX text is not a literal, as JSX drops lines of whitespace, so a blank line in it is blank.

| Repository | Commit | JavaScript and TypeScript files | Lexically invalid |
| --- | --- | ---: | ---: |
| [microsoft/vscode](https://github.com/microsoft/vscode) | `e219e7c0` | 15,548 | 26 |
| [prettier/prettier](https://github.com/prettier/prettier) | `c7a823c` | 5,887 | 240 |
| [facebook/react](https://github.com/facebook/react) | `c5ec5c6` | 4,479 | 0 |

Every valid file matches on every line. The invalid files are Prettier's error tests and its range and cursor tests, whose `<<<PRETTIER_RANGE_START>>>` and `<|>` markers are not JavaScript, and VS Code test fixtures with deliberate syntax errors. Before the fork's correction, 84 valid files differed, each on its hashbang line.

The owner's a12-rulekit also matches on every line of its Java, Kotlin, JavaScript and TypeScript files; its record stays local. One of its JavaScript files does not parse: a template literal holding unescaped backticks ends early.
