# Tokei Fork Step 6: Each Language's Judge

Measured from 2026-10-29 on macOS arm64, for [step 6](../plans/tokei-fork.md#step-6-no-difference-from-each-languages-judge): every file of every corpus must count as the language's own lexer or parser counts it. A line is a comment when it holds nothing but comments; a blank line takes the class of the comment or literal it lies in; every other line is code. A file the judge itself rejects as lexically invalid has no correct count, so it is listed rather than matched.

The judges and the comparison tool live in the fork's [judges](https://github.com/mbackschat/tokei/tree/locmetry/judges) folder from `a2a9724`. Each judge reports the comment and literal ranges its lexer found, and `judgecmp` compares the lines they imply with the fork's `line_classes` (fork `238b0c5`), line by line. The fork's results below are those of `15.0.0+locmetry.13`, fork `a55e730`, for Python of `15.0.0+locmetry.14`, fork `701fdaa`, for C, C++, Objective-C, Go, C# and Ruby of `15.0.0+locmetry.15`, fork `40ed432`, for PHP, Swift, Dart and Scala of `15.0.0+locmetry.16`, fork `eb463fb`, for R of `15.0.0+locmetry.17`, fork `984e57b`, for Elixir and PowerShell of `15.0.0+locmetry.18`, fork `687cb25`, for Haskell of `15.0.0+locmetry.19`, fork `ffb6076`, for XML, SVG, CSS, SCSS, HTML and Vue of `15.0.0+locmetry.20`, fork `397da12`, for Svelte of `15.0.0+locmetry.21`, fork `9c85621`, for Lua and SQL of `15.0.0+locmetry.22`, fork `a6c2728`, for YAML of `15.0.0+locmetry.23`, fork `1806dc8`, and for Markdown of `15.0.0+locmetry.24`, fork `99502cb`. A line of whitespace takes the class of the comment or literal its own bytes lie in, as judgecmp reads it from fork `02345d9`; every earlier result was rechecked under that rule.

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

## Python

Judge: CPython 3.14.10's `tokenize`, which reads f-strings and t-strings as PEP 701 defines them: the text parts are literals, and each `{…}` replacement field is code with its own strings and comments. Docstrings are literals, as every string is.

| Repository | Commit | Python files |
| --- | --- | ---: |
| [django/django](https://github.com/django/django) | `817df1c` | 2,937 |
| [python/cpython](https://github.com/python/cpython) | `ef1fce0` | 2,379 |
| [numpy/numpy](https://github.com/numpy/numpy) | `39f66c0` | 496 |

Every file is valid and matches on every line. Before the fork's correction, one file differed on one line: in cpython's `Lib/test/test_fstring.py`, a line holding only a comment inside a multi-line replacement field counted as code. The correction declares Python's f-string and t-string prefixes, each with its replacement fields, which made Django's parse 24% slower until the fork tried prefixed openers only where their first two bytes can start one (fork `05c5eff`); Django then parsed 7% slower than with `15.0.0+locmetry.13`, and Tokio and Prettier no slower, as the minimum of interleaved runs on a loaded machine.

## C, C++, Objective-C and Objective-C++

Judge: clang's raw lexer, `clang -cc1 -dump-raw-tokens` from Apple clang 21.0.0, which lexes without preprocessing, as the counts read the text that was written. Headers (`.h`) are lexed as C++. A line splice, a backslash before a newline, belongs to no token, since translation phase 2 deletes it before lexing (C23 §5.1.1.2).

| Repository | Commit | Files |
| --- | --- | ---: |
| [godotengine/godot](https://github.com/godotengine/godot) | `5e5b0a3` | 8,444 |
| [php/php-src](https://github.com/php/php-src) | `b3c2d717` | 2,314 |
| [protocolbuffers/protobuf](https://github.com/protocolbuffers/protobuf) | `6ce965a` | 1,437 |
| [curl/curl](https://github.com/curl/curl) | `87c4c6d` | 1,062 |
| [git/git](https://github.com/git/git) | `b9e5cff` | 986 |
| [abseil/abseil-cpp](https://github.com/abseil/abseil-cpp) | `8e4c6fc` | 879 |
| [redis/redis](https://github.com/redis/redis) | `4188839` | 854 |
| [TextureGroup/Texture](https://github.com/TextureGroup/Texture) | `d8aa8c7` | 848 |
| [nlohmann/json](https://github.com/nlohmann/json) | `2aecd86` | 619 |
| [catchorg/Catch2](https://github.com/catchorg/Catch2) | `76dd8f5` | 418 |
| [nginx/nginx](https://github.com/nginx/nginx) | `67290bf` | 411 |
| [CocoaLumberjack/CocoaLumberjack](https://github.com/CocoaLumberjack/CocoaLumberjack) | `2106aaf` | 225 |
| [SDWebImage/SDWebImage](https://github.com/SDWebImage/SDWebImage) | `c3ad5e1` | 200 |
| [google/leveldb](https://github.com/google/leveldb) | `bb74ef7` | 133 |
| [AFNetworking/AFNetworking](https://github.com/AFNetworking/AFNetworking) | `d9f589c` | 80 |
| [fmtlib/fmt](https://github.com/fmtlib/fmt) | `f87a75e` | 74 |

Every file is valid and matches on every line. Before the fork's correction, 265 files differed on 2,218 lines, each at a line splice: a line holding comments and a splice, or only a splice, inside a macro. The repositories hold no Objective-C character literal holding a quote and no Objective-C++ raw string; fixtures checked by the same judge cover them.

## Go

Judge: the standard library's `go/scanner` from Go 1.25.14.

| Repository | Commit | Go files | Lexically invalid |
| --- | --- | ---: | ---: |
| [kubernetes/kubernetes](https://github.com/kubernetes/kubernetes) | `a972b672` | 18,096 | 0 |
| [golang/go](https://github.com/golang/go) | `9586e4b6` | 11,867 | 15 |
| [gohugoio/hugo](https://github.com/gohugoio/hugo) | `a1668da` | 912 | 0 |
| [prometheus/prometheus](https://github.com/prometheus/prometheus) | `a87694b` | 755 | 0 |

Every valid file matched on every line before any Go correction. The invalid files are the compiler's test data.

## C#

Judge: Roslyn's parser, `Microsoft.CodeAnalysis.CSharp` 5.9.0. Roslyn reads text in an inactive `#if` region as disabled text without comments or literals, so the judge parses each run of it on its own, as its author wrote it.

| Repository | Commit | C# files | Lexically invalid |
| --- | --- | ---: | ---: |
| [dotnet/roslyn](https://github.com/dotnet/roslyn) | `dcfa99c3` | 18,289 | 68 |
| [dotnet/aspnetcore](https://github.com/dotnet/aspnetcore) | `69e6bd52` | 10,729 | 16 |
| [JamesNK/Newtonsoft.Json](https://github.com/JamesNK/Newtonsoft.Json) | `52fa3ae` | 951 | 0 |
| [AutoMapper/AutoMapper](https://github.com/AutoMapper/AutoMapper) | `df65f2e` | 517 | 0 |

Every valid file matches on every line. Before the fork's correction, 2 files differed on 269 lines, each after a `{{{` inside a `$$"""…"""` string, whose innermost braces open the hole.

## Ruby

Judge: Prism, `Prism.lex` from Ruby 3.4.12, Ruby's default parser since 3.4.

| Repository | Commit | Ruby files |
| --- | --- | ---: |
| [discourse/discourse](https://github.com/discourse/discourse) | `3c9c7ba6` | 13,022 |
| [rails/rails](https://github.com/rails/rails) | `8ebac2f` | 3,481 |
| [Homebrew/brew](https://github.com/Homebrew/brew) | `9226bdf` | 2,419 |
| [rubocop/rubocop](https://github.com/rubocop/rubocop) | `bde5d42` | 1,791 |

Every file is valid and matches on every line. Before the fork's corrections, 17 files differed on 143 lines: regular expressions spanning lines, `%{…}` and `/…/` arguments after a method name, `<< /…/`, and `?/` character literals. Telling a local variable from a method name before ` /` or ` %{` takes a parser; the fork reads a method name, and no measured file holds the other case.

## PHP

Judge: PHP's `token_get_all` with `TOKEN_PARSE` from PHP 8.4.27, whose default enables short `<?` tags. Text outside the PHP tags is inline HTML, in no comment or literal, so it counts as code. Blade templates (`.blade.php`) are not PHP and were left out.

| Repository | Commit | PHP files | Lexically invalid |
| --- | --- | ---: | ---: |
| [symfony/symfony](https://github.com/symfony/symfony) | `92216211` | 12,949 | 1 |
| [nextcloud/server](https://github.com/nextcloud/server) | `c7e435c` | 5,928 | 0 |
| [laravel/framework](https://github.com/laravel/framework) | `5a787ee` | 3,043 | 1 |
| [WordPress/WordPress](https://github.com/WordPress/WordPress) | `702368b` | 2,083 | 0 |

Every valid file matches on every line. Before the fork's correction, 44 files differed on 228 lines: apostrophes and `#` in inline HTML opened strings and comments, and a `?>` inside a `//` comment did not end PHP. Inline `<script>` and `<style>` text in PHP output stays code, as PHP reads it.

## Swift

Judge: swift-syntax, the parser of the Swift compiler, as the Xcode toolchain of Apple Swift 6.4 ships it for macros. The compiler corpus is a sparse checkout of `stdlib`, `test/StringProcessing`, `test/Parse`, `test/Interpreter`, `test/stdlib`, `test/IDE`, `test/Macros` and `validation-test/stdlib`.

| Repository | Commit | Swift files | Lexically invalid |
| --- | --- | ---: | ---: |
| [swiftlang/swift](https://github.com/swiftlang/swift) | `6d943a0687` | 2,959 | 300 |
| [kickstarter/ios-oss](https://github.com/kickstarter/ios-oss) | `0c22209f2c` | 2,085 | 0 |
| [swiftlang/swift-package-manager](https://github.com/swiftlang/swift-package-manager) | `1df2f93f7e` | 1,889 | 0 |
| [pointfreeco/swift-composable-architecture](https://github.com/pointfreeco/swift-composable-architecture) | `bb175b544d` | 798 | 2 |
| [realm/SwiftLint](https://github.com/realm/SwiftLint) | `ec4691d9e8` | 778 | 0 |
| [swiftlang/swift-syntax](https://github.com/swiftlang/swift-syntax) | `237beba662` | 768 | 0 |
| [apple/swift-collections](https://github.com/apple/swift-collections) | `f96b909e1a` | 739 | 0 |
| [ReactiveX/RxSwift](https://github.com/ReactiveX/RxSwift) | `3e33f90c1b` | 588 | 0 |
| [apple/swift-nio](https://github.com/apple/swift-nio) | `108daca35d` | 556 | 0 |
| [swiftlang/swift-foundation](https://github.com/swiftlang/swift-foundation) | `8177931533` | 523 | 0 |
| [vapor/vapor](https://github.com/vapor/vapor) | `0f2a8ee9f3` | 278 | 0 |
| [swiftlang/swift-format](https://github.com/swiftlang/swift-format) | `77e88e4a03` | 264 | 0 |
| [Alamofire/Alamofire](https://github.com/Alamofire/Alamofire) | `bda9ed57d7` | 101 | 0 |

Every valid file matched on every line before any Swift correction. The invalid files are the parser's error tests and documentation snippets.

## Dart

Judge: the scanner of the Dart SDK's front end, `_fe_analyzer_shared` 111.0.0, run with Dart 3.13.7. An unterminated string or comment makes a file invalid. The SDK corpus is a sparse checkout of `pkg/analyzer/lib`, `pkg/front_end/lib`, `pkg/_fe_analyzer_shared/lib`, `sdk/lib` and `tests/language`, and Flutter's of `packages` and `dev`.

| Repository | Commit | Dart files | Lexically invalid |
| --- | --- | ---: | ---: |
| [dart-lang/sdk](https://github.com/dart-lang/sdk) | `113b1b454f` | 5,951 | 16 |
| [flutter/flutter](https://github.com/flutter/flutter) | `d555faf800` | 5,718 | 0 |
| [flutter/packages](https://github.com/flutter/packages) | `3258d45ac0` | 4,113 | 0 |
| [rrousselGit/riverpod](https://github.com/rrousselGit/riverpod) | `33af1e8122` | 1,311 | 0 |
| [felangel/bloc](https://github.com/felangel/bloc) | `b9be1e252e` | 616 | 0 |
| [dart-lang/http](https://github.com/dart-lang/http) | `c76dfb3a2a` | 340 | 0 |

Every valid file matches on every line. Before the fork's correction, 2 files differed on 9 lines, each inside a nested block comment. The invalid files are the language tests of lexical errors.

## Scala

Judge: the Scala 3.10.0 compiler's parser and scanner. The parser drives the scanner, since XML literals are read by the parser and are code; where the parser gives up, as on Scala 2's early definitions, the scanner reads on to the end of the file. Scala 2 code is read as Scala 3, whose lexical syntax differs from Scala 2's only in forms no measured file uses. The compiler corpora are sparse checkouts: Scala 3's `compiler`, `library`, `tests/pos` and `tests/run`, Scala 2's `src/library`, `src/compiler`, `test/files/pos` and `test/files/run`, and Spark's `core`, `sql/core/src/main`, `mllib/src/main` and `streaming`.

| Repository | Commit | Scala files | Lexically invalid |
| --- | --- | ---: | ---: |
| [scala/scala3](https://github.com/scala/scala3) | `5edef6d5ac` | 7,965 | 0 |
| [scala/scala](https://github.com/scala/scala) | `cee0672ed8` | 5,251 | 0 |
| [apache/pekko](https://github.com/apache/pekko) | `8f3d71b90e` | 2,838 | 0 |
| [apache/spark](https://github.com/apache/spark) | `a8e363406b` | 2,385 | 0 |
| [sbt/sbt](https://github.com/sbt/sbt) | `4bcdaa9f79` | 2,255 | 1 |
| [zio/zio](https://github.com/zio/zio) | `846696c625` | 976 | 0 |
| [playframework/playframework](https://github.com/playframework/playframework) | `d6f230d447` | 856 | 0 |
| [typelevel/cats](https://github.com/typelevel/cats) | `b6c4dbfbb3` | 835 | 0 |
| [scalameta/scalameta](https://github.com/scalameta/scalameta) | `dae6167bd1` | 761 | 0 |

Every valid file but one matches on every line. Before the fork's corrections, 144 files differed on 6,483 lines: nested block comments, triple-quoted strings ending in a run of more than three quotes, and interpolators other than `s`, `f` and `raw`, such as `em"…"`. The remaining file, Scala 3's `tests/run/dedented-string-literals.scala`, differs on 4 lines inside `'''` strings, which `import language.experimental.dedentedStringLiterals` enables; experimental features are not yet part of the language, so the fork does not read them.

## R

Judge: R's own parser, through the parse data that `parse(keep.source = TRUE)` records for every token, run with R 4.6.1. `COMMENT` tokens are comments and `STR_CONST` tokens, raw strings included, are literals; a file R cannot parse is invalid. The R corpus is a sparse checkout of `src/library` and `tests`.

| Repository | Commit | R files | Lexically invalid |
| --- | --- | ---: | ---: |
| [wch/r-source](https://github.com/wch/r-source) | `da12bf8030` | 940 | 3 |
| [tidyverse/ggplot2](https://github.com/tidyverse/ggplot2) | `b638d654c8` | 360 | 0 |
| [mlr-org/mlr3](https://github.com/mlr-org/mlr3) | `f0455c65ec` | 301 | 0 |
| [rstudio/shiny](https://github.com/rstudio/shiny) | `261cbe6e5e` | 278 | 2 |
| [r-lib/testthat](https://github.com/r-lib/testthat) | `54c0edc932` | 266 | 1 |
| [tidyverse/dplyr](https://github.com/tidyverse/dplyr) | `d5e94e7fa8` | 195 | 1 |
| [r-lib/rlang](https://github.com/r-lib/rlang) | `b4a2a057ea` | 160 | 0 |
| [Rdatatable/data.table](https://github.com/Rdatatable/data.table) | `086c6466d8` | 87 | 0 |
| [tidyverse/tidyr](https://github.com/tidyverse/tidyr) | `26f83e89a6` | 86 | 0 |

Every valid file matches on every line. Before the fork's correction, 13 files differed on 53 lines: Tokei's R declared no strings, so a `#` inside one started a comment, and a `#line` directive, which R's parser reads as code, counted as a comment. The invalid files are R's and testthat's tests of syntax errors, Shiny's app templates with `{{` placeholders, and a dplyr script whose `[` call in a pipe R 4.6.1 rejects.

## Elixir

Judge: Elixir's own tokenizer, `:elixir_tokenizer`, the one `Code.string_to_quoted_with_comments/2` runs, with Elixir 1.20.4 on OTP 29. Its tokens record where they start, so a literal ends where the next token or comment starts, less the whitespace before it; character literals keep their own text. A file the tokenizer rejects is invalid.

| Repository | Commit | Elixir files | Lexically invalid |
| --- | --- | ---: | ---: |
| [plausible/analytics](https://github.com/plausible/analytics) | `02cfac1517` | 1,315 | 0 |
| [ash-project/ash](https://github.com/ash-project/ash) | `2d0f8f52a3` | 1,118 | 0 |
| [elixir-lsp/elixir-ls](https://github.com/elixir-lsp/elixir-ls) | `328a1c08c4` | 1,014 | 2 |
| [hexpm/hexpm](https://github.com/hexpm/hexpm) | `9d810e6f6b` | 1,011 | 0 |
| [elixir-lang/elixir](https://github.com/elixir-lang/elixir) | `c17c7c66a7` | 568 | 0 |
| [absinthe-graphql/absinthe](https://github.com/absinthe-graphql/absinthe) | `68486bc958` | 552 | 0 |
| [rrrene/credo](https://github.com/rrrene/credo) | `b113eb2e5f` | 489 | 0 |
| [livebook-dev/livebook](https://github.com/livebook-dev/livebook) | `ef1b666f92` | 443 | 0 |
| [phoenixframework/phoenix_live_view](https://github.com/phoenixframework/phoenix_live_view) | `d700eec809` | 238 | 0 |
| [phoenixframework/phoenix](https://github.com/phoenixframework/phoenix) | `c9065a8956` | 207 | 0 |
| [elixir-nx/nx](https://github.com/elixir-nx/nx) | `ebe2b9fad5` | 176 | 0 |
| [elixir-ecto/ecto](https://github.com/elixir-ecto/ecto) | `4a1bedf1d2` | 126 | 0 |
| [sorentwo/oban](https://github.com/sorentwo/oban) | `c5aa59b352` | 124 | 0 |
| [elixir-plug/plug](https://github.com/elixir-plug/plug) | `575a8454a2` | 78 | 0 |
| [elixir-explorer/explorer](https://github.com/elixir-explorer/explorer) | `5e75adb6a7` | 50 | 0 |
| [dashbitco/broadway](https://github.com/dashbitco/broadway) | `8debff6f4e` | 33 | 0 |

Every valid file matches on every line. Before the fork's correction, 60 of the 3,001 files in the ten repositories measured first, all but Plausible, Ash, ElixirLS, Hexpm, Explorer and Broadway, differed on 3,322 lines: character literals such as `?"` opened strings, sigils such as `~S{…}` were code, and a heredoc inside a `#{…}` hole closed the heredoc around it. The invalid files are ElixirLS fixtures of lexical errors.

## PowerShell

Judge: PowerShell's own parser, `Parser.ParseFile`, with PowerShell 7.5.0. Comment tokens are comments and quoted strings and here-strings literals; bare words, which the parser also types as strings, are code, and the `$(…)` subexpressions and variables inside an expandable string are code. Only errors that leave a string or comment open make a file invalid. Its offsets count UTF-16 code units.

| Repository | Commit | PowerShell files | Lexically invalid |
| --- | --- | ---: | ---: |
| [dataplat/dbatools](https://github.com/dataplat/dbatools) | `ae7f09539a` | 1,795 | 0 |
| [PowerShell/PowerShell](https://github.com/PowerShell/PowerShell) | `2725d6aaa6` | 555 | 0 |
| [chocolatey/choco](https://github.com/chocolatey/choco) | `43c0b3ae12` | 355 | 0 |
| [pester/Pester](https://github.com/pester/Pester) | `98e56d65da` | 272 | 0 |
| [PowerShell/PSScriptAnalyzer](https://github.com/PowerShell/PSScriptAnalyzer) | `0151b9dafd` | 271 | 0 |
| [ScoopInstaller/Scoop](https://github.com/ScoopInstaller/Scoop) | `e6aa3b366b` | 82 | 0 |
| [microsoft/PowerShellForGitHub](https://github.com/microsoft/PowerShellForGitHub) | `fd4fa65f23` | 69 | 0 |
| [PowerShell/platyPS](https://github.com/PowerShell/platyPS) | `66dbe279b6` | 33 | 0 |
| [PowerShell/PSReadLine](https://github.com/PowerShell/PSReadLine) | `a2d143f789` | 10 | 0 |

The files are the `.ps1`, `.psm1` and `.psd1` files; Tokei also maps `.ps1xml` and `.cdxml`, which are XML. Every valid file matches on every line, except 17 PSScriptAnalyzer test files in UTF-16 or other encodings, which judgecmp does not read. Before the fork's correction, 233 files differed on 10,411 lines, nearly all because `"@(…)"` opened the backwards-declared here-string; the last difference was a string that a `“` closes.

## Haskell

Judge: GHC's own lexer, `lexTokenStream` from the `ghc` library of GHC 9.14.1, with GHC2021's extensions and those the file's LANGUAGE pragmas turn on or off. Comment tokens are comments, and string, character and quasi-quote tokens literals; pragmas that the lexer returns as tokens, such as `{-# INLINE f #-}`, are code, while it skips `{-# LANGUAGE … #-}` and unknown pragmas as comments. A file the lexer rejects is invalid; it does not run the C preprocessor, so files using CPP are invalid, and a project's default extensions from its Cabal file are not read.

| Repository | Commit | Haskell files | Lexically invalid |
| --- | --- | ---: | ---: |
| [haskell/cabal](https://github.com/haskell/cabal) | `07e003b333` | 2,186 | 19 |
| [haskell/haskell-language-server](https://github.com/haskell/haskell-language-server) | `bc2bbb8e54` | 1,462 | 27 |
| [commercialhaskell/stack](https://github.com/commercialhaskell/stack) | `11e40c9da1` | 700 | 9 |
| [jgm/pandoc](https://github.com/jgm/pandoc) | `75ebbfa7c8` | 366 | 3 |
| [hadolint/hadolint](https://github.com/hadolint/hadolint) | `1d901b9c0b` | 189 | 0 |
| [PostgREST/postgrest](https://github.com/PostgREST/postgrest) | `29a95f823b` | 186 | 2 |
| [haskell/aeson](https://github.com/haskell/aeson) | `1dd465c115` | 126 | 6 |
| [ekmett/lens](https://github.com/ekmett/lens) | `61eb79c3b4` | 120 | 3 |
| [haskell/text](https://github.com/haskell/text) | `9b7ff8a4cd` | 106 | 14 |
| [haskell/containers](https://github.com/haskell/containers) | `953428ad07` | 79 | 9 |
| [koalaman/shellcheck](https://github.com/koalaman/shellcheck) | `919d37c4d7` | 31 | 0 |
| [simonmar/happy](https://github.com/simonmar/happy) | `4613f8ba7e` | 24 | 2 |

Every valid file matches on every line. Before the fork's correction, 386 files differed on 3,955 lines: pragmas that GHC reads as code counted as comments, a string gap's closing backslash escaped the quote after it, and quasi-quotes such as `[r|…|]` were read as code. The 94 invalid files all use CPP.

## XML and SVG

Judge: expat 2.2.8 through Python's `xml.parsers.expat`, which reports where each comment and start tag begins; quoted attribute values of an accepted tag are literals, comments, those of the internal DTD subset included, comments, and character data, CDATA sections included, code. External DTDs are not read. A file expat rejects as not well formed is invalid.

| Repository | Commit | XML and SVG files | Lexically invalid |
| --- | --- | ---: | ---: |
| [FortAwesome/Font-Awesome](https://github.com/FortAwesome/Font-Awesome) | `14c65a3747` | 11,544 | 0 |
| [mastodon/mastodon](https://github.com/mastodon/mastodon) | `b4076a6671` | 4,294 | 0 |
| [apache/maven](https://github.com/apache/maven) | `b46f9353aa` | 3,272 | 5 |
| [hibernate/hibernate-orm](https://github.com/hibernate/hibernate-orm) | `d6c1a87f14` | 1,122 | 4 |
| [spring-projects/spring-framework](https://github.com/spring-projects/spring-framework) | `64861cb395` | 705 | 1 |
| [apache/ant](https://github.com/apache/ant) | `dbb01a2d52` | 516 | 6 |
| [mybatis/mybatis-3](https://github.com/mybatis/mybatis-3) | `bd5915a997` | 441 | 0 |
| [apache/tomcat](https://github.com/apache/tomcat) | `8c164da347` | 236 | 2 |
| [WordPress/WordPress](https://github.com/WordPress/WordPress) | `1e68f9f454` | 147 | 1 |
| [android/nowinandroid](https://github.com/android/nowinandroid) | `2982e63cb6` | 69 | 1 |
| [discourse/discourse](https://github.com/discourse/discourse) | `3b80490392` | 29 | 1 |
| [primer/css](https://github.com/primer/css) | `c990fb1e44` | 7 | 0 |
| [twbs/bootstrap](https://github.com/twbs/bootstrap) | `c967e2cbe9` | 3 | 0 |

Every valid file matches on every line. Before the fork's correction, 19 files differed on 197 lines, all inside CDATA sections: `<!--` there opened a comment, and the section read as a tag, so an apostrophe in it opened a string. The invalid files are empty, deliberately malformed test fixtures, templates with placeholders, and one ISO-2022-JP file expat cannot decode.

## CSS

Judge: tinycss2 1.5.1, an implementation of CSS Syntax Level 3. Comments are comments, and strings and unquoted `url(…)` tokens literals; a bad string or URL, or a comment, string or URL that the end of the file cuts off, makes a file invalid.

| Repository | Commit | CSS files | Lexically invalid |
| --- | --- | ---: | ---: |
| [WordPress/WordPress](https://github.com/WordPress/WordPress) | `1e68f9f454` | 1,073 | 0 |
| [picocss/pico](https://github.com/picocss/pico) | `a07563ddc8` | 238 | 0 |
| [animate-css/animate.css](https://github.com/animate-css/animate.css) | `3f8ab233db` | 106 | 0 |
| [foundation/foundation-sites](https://github.com/foundation/foundation-sites) | `a15a9775e1` | 69 | 0 |
| [jgthms/bulma](https://github.com/jgthms/bulma) | `c5cd04baf0` | 69 | 0 |
| [FortAwesome/Font-Awesome](https://github.com/FortAwesome/Font-Awesome) | `14c65a3747` | 41 | 0 |
| [twbs/bootstrap](https://github.com/twbs/bootstrap) | `c967e2cbe9` | 33 | 0 |
| [spring-projects/spring-framework](https://github.com/spring-projects/spring-framework) | `64861cb395` | 23 | 2 |
| [mastodon/mastodon](https://github.com/mastodon/mastodon) | `b4076a6671` | 11 | 0 |
| [apache/tomcat](https://github.com/apache/tomcat) | `8c164da347` | 6 | 0 |
| [mybatis/mybatis-3](https://github.com/mybatis/mybatis-3) | `bd5915a997` | 6 | 0 |
| [discourse/discourse](https://github.com/discourse/discourse) | `3b80490392` | 6 | 0 |

Every valid file matches on every line. Tokei declared `//` line comments for CSS, which has none; no file in the corpus starts a line with `//`, and the fork's correction is checked by a test. The invalid files are Spring's tests of an unclosed `url(`.

## SCSS

Judge: the tokenizer of postcss-scss 4.0.9, with which stylelint parses SCSS. `/* */` and `//` comments are comments and quoted strings literals; an unclosed string or comment makes a file invalid. The three `.sass` files of the indented syntax in Bulma are not judged.

| Repository | Commit | SCSS files | Lexically invalid |
| --- | --- | ---: | ---: |
| [discourse/discourse](https://github.com/discourse/discourse) | `3b80490392` | 731 | 1 |
| [jgthms/bulma](https://github.com/jgthms/bulma) | `c5cd04baf0` | 179 | 0 |
| [WordPress/WordPress](https://github.com/WordPress/WordPress) | `1e68f9f454` | 157 | 0 |
| [twbs/bootstrap](https://github.com/twbs/bootstrap) | `c967e2cbe9` | 138 | 0 |
| [mastodon/mastodon](https://github.com/mastodon/mastodon) | `b4076a6671` | 131 | 0 |
| [primer/css](https://github.com/primer/css) | `c990fb1e44` | 113 | 0 |
| [picocss/pico](https://github.com/picocss/pico) | `a07563ddc8` | 54 | 0 |
| [FortAwesome/Font-Awesome](https://github.com/FortAwesome/Font-Awesome) | `14c65a3747` | 40 | 0 |

Every valid file matches on every line; the invalid file is Discourse's broken-SCSS fixture.

## HTML

Judge: parse5 7, the HTML Standard's tokenizer, through its SAX parser. Comments, bogus ones such as `<?xml …?>` included, are comments, quoted attribute values literals, and text code. Scripts of a JavaScript type are read by Babel and styles by PostCSS's tokenizer, keeping their offsets in the file; scripts of other types are data blocks, whose text is code. The fork classes the lines of a script or style element as its own language counts them from fork `3eda0aa`; until then, files holding one could not be compared.

| Repository | Commit | HTML files | Lexically invalid |
| --- | --- | ---: | ---: |
| [GoogleChrome/samples](https://github.com/GoogleChrome/samples) | `8c3a755f75` | 882 | 2 |
| [jgthms/bulma](https://github.com/jgthms/bulma) | `c5cd04baf0` | 356 | 0 |
| [tastejs/todomvc](https://github.com/tastejs/todomvc) | `ff43b02e59` | 259 | 1 |
| [apache/ant](https://github.com/apache/ant) | `dbb01a2d52` | 232 | 0 |
| [mdn/dom-examples](https://github.com/mdn/dom-examples) | `03ae1ffb32` | 199 | 0 |
| [discourse/discourse](https://github.com/discourse/discourse) | `3b80490392` | 199 | 0 |
| [mdn/learning-area](https://github.com/mdn/learning-area) | `21973263ba` | 154 | 0 |
| [foundation/foundation-sites](https://github.com/foundation/foundation-sites) | `a15a9775e1` | 116 | 0 |
| [apache/tomcat](https://github.com/apache/tomcat) | `8c164da347` | 102 | 0 |
| [WordPress/WordPress](https://github.com/WordPress/WordPress) | `1e68f9f454` | 78 | 0 |
| [jquery/jquery](https://github.com/jquery/jquery) | `c2f27864e4` | 59 | 1 |
| [twbs/bootstrap](https://github.com/twbs/bootstrap) | `c967e2cbe9` | 16 | 0 |
| [mdn/webgl-examples](https://github.com/mdn/webgl-examples) | `17a7484b5b` | 9 | 0 |
| [mastodon/mastodon](https://github.com/mastodon/mastodon) | `b4076a6671` | 9 | 0 |
| [spring-projects/spring-framework](https://github.com/spring-projects/spring-framework) | `64861cb395` | 9 | 0 |
| [h5bp/html5-boilerplate](https://github.com/h5bp/html5-boilerplate) | `b6597338e6` | 4 | 0 |

Every valid file matches on every line. Before the fork's corrections, three files differed: an `<!-->` inside a conditional comment did not close it, because the engine skipped a comment opener inside a comment; a `<script type="text/ractive">` template was read as JavaScript; and an `<?xml …?>` declaration counted as code. The invalid files are Jekyll and Riot templates and a jQuery fixture whose scripts Babel rejects.

## Vue

Judge: Vue's own parser, `@vue/compiler-dom` 3 in its single-file-component mode. Comments at the top level and in the template are comments, attribute values, directives' included, literals, and template text and interpolations code. Script blocks are read by Babel, as TypeScript for `lang="ts"`, and style blocks by postcss-scss for `lang="scss"`, `"sass"` or `"less"` and by PostCSS otherwise.

| Repository | Commit | Vue files | Lexically invalid |
| --- | --- | ---: | ---: |
| [element-plus/element-plus](https://github.com/element-plus/element-plus) | `a8d102a91b` | 1,022 | 0 |
| [PanJiaChen/vue-element-admin](https://github.com/PanJiaChen/vue-element-admin) | `6858a9ad67` | 131 | 0 |
| [vuejs/docs](https://github.com/vuejs/docs) | `40aa88af00` | 60 | 0 |

Every valid file matches on every line. Once CSS lost its `//` comments, five files differed on 27 lines, because `<style lang="scss">` blocks were read as CSS; the fork now resolves a `lang` attribute by file extension too.

## Svelte

Judge: Svelte's own parser, `parse` from svelte 5.57.3 in its modern mode. Markup comments are comments, quoted attribute values literals, and text and `{…}` expressions code, except for the JavaScript comments the parser lists, those between a tag's attributes included. Script elements, in the markup too, are read by Babel, as TypeScript for `lang="ts"`, and style blocks by postcss-scss for `lang="scss"`, `"sass"` or `"less"` and by PostCSS otherwise; Svelte's parser reads style blocks only as CSS, so their text is blanked, keeping its offsets, before the component is parsed.

| Repository | Commit | Svelte files | Lexically invalid |
| --- | --- | ---: | ---: |
| [sveltejs/svelte](https://github.com/sveltejs/svelte) | `fe21fce019` | 4,611 | 107 |
| [sveltejs/kit](https://github.com/sveltejs/kit) | `04aac59bad` | 1,022 | 3 |

Every valid file matches on every line. Most invalid files are the compiler's tests of input it rejects. Fork `397da12` differed on 20 files and 75 lines, all JavaScript comments or apostrophes inside `{…}` expressions, which the fork now reads with TypeScript's syntax up to their closing brace, and on one more file of 15 lines, Svelte's test of the comments between attributes that Svelte 5.53.0 allows.

## Lua

Judge: Lua's own lexer, `luaX_next` from `llex.c`, linked with Lua 5.5.1's core, whose source was checked against the SHA-256 that lua.org publishes. The lexer skips comments without reporting them, so the judge reads them from the text between tokens, which holds nothing else, by `llex.c`'s rules; strings, long brackets included, are literals; a first line starting with `#` is a comment, as `luaL_loadfile` skips it (`lauxlib.c`, `skipcomment`).

| Repository | Commit | Lua files | Lexically invalid |
| --- | --- | ---: | ---: |
| [Kong/kong](https://github.com/Kong/kong) | `21032122ab` | 1,312 | 1 |
| [awesomeWM/awesome](https://github.com/awesomeWM/awesome) | `c463e19ef9` | 883 | 0 |
| [neovim/neovim](https://github.com/neovim/neovim) | `e0516a46e8` | 857 | 2 |
| [koreader/koreader](https://github.com/koreader/koreader) | `801277394b` | 601 | 0 |
| [LuaLS/lua-language-server](https://github.com/LuaLS/lua-language-server) | `a58fc7b01e` | 482 | 0 |
| [luarocks/luarocks](https://github.com/luarocks/luarocks) | `85bfe36c81` | 162 | 0 |
| [lunarmodules/Penlight](https://github.com/lunarmodules/Penlight) | `7f3351f226` | 115 | 0 |
| [nvim-treesitter/nvim-treesitter](https://github.com/nvim-treesitter/nvim-treesitter) | `a2f694478b` | 79 | 0 |
| [nvim-telescope/telescope.nvim](https://github.com/nvim-telescope/telescope.nvim) | `40aedd8a68` | 77 | 0 |
| [folke/lazy.nvim](https://github.com/folke/lazy.nvim) | `306a05526a` | 65 | 0 |

Every valid file matches on every line. The invalid files use LuaJIT's integer suffixes, such as `1ULL`, which Lua rejects. Fork `9c85621` differed on 27 files, one line each: first lines starting with `#`, such as `#!/usr/bin/env -S nvim -l` or `#./luajit`, which Lua skips as a comment.

## SQL

SQL has no single reference lexer; the judge names PostgreSQL's, `scan.l`, through pglast 8.7, as the most widely used open-source dialect's. It reads a script as psql runs it: a backslash outside literals and comments starts a meta-command, such as `\set x 'a\'b'`, which psql's own lexer reads to the end of the line, so the line's rest is code. String constants and quoted identifiers are literals, `--` and nested `/* */` comments comments. A dollar-quoted string whose text scans as SQL is read as SQL, as PostgreSQL reads the PL/pgSQL and SQL function bodies and `DO` blocks it mostly holds, with the core scanner (`pl_scanner.c` calls `core_yylex`); any other, such as a Perl function's body, is a literal. Other dialects' syntax, such as MySQL's `#` comments, backticks and `\'` escapes, reads as PostgreSQL reads it.

| Repository | Commit | SQL files | Lexically invalid |
| --- | --- | ---: | ---: |
| [citusdata/citus](https://github.com/citusdata/citus) | `843ef6f049` | 1,384 | 5 |
| [postgres/postgres](https://github.com/postgres/postgres) | `328a9a1731` | 847 | 4 |
| [timescale/timescaledb](https://github.com/timescale/timescaledb) | `21b65abd94` | 679 | 2 |
| [supabase/supabase](https://github.com/supabase/supabase) | `65217161af` | 111 | 0 |
| [pgvector/pgvector](https://github.com/pgvector/pgvector) | `44624c1f8d` | 57 | 0 |
| [jOOQ/sakila](https://github.com/jOOQ/sakila) | `e089a5b1ec` | 34 | 3 |
| [datacharmer/test_db](https://github.com/datacharmer/test_db) | `e324b56193` | 15 | 0 |
| [dbt-labs/jaffle-shop-classic](https://github.com/dbt-labs/jaffle-shop-classic) | `fd7bfacae4` | 5 | 0 |

Every valid file matches on every line. The invalid files hold `COPY … FROM stdin` data, which psql sends to the server unread, or are PostgreSQL's and its extensions' tests of input the scanner rejects, such as `123abc` or `""`. Against fork `9c85621`, 385 files differed on 5,121 lines while the judge read every dollar-quoted string as a literal, most of them comment lines inside function bodies; once it read bodies that scan as SQL, 32 files differed on 1,089 lines: apostrophes in `"…"` identifiers, `\'` in standard strings, psql meta-commands and dollar-quoted text that is not SQL.

## YAML

Judge: the lexer of the `yaml` package 2.8.1, a YAML 1.2 implementation that passes the yaml-test-suite; its parser's errors mark a file invalid. Comments are comments; plain, quoted and block scalars are literals, except the empty lines after a block scalar without content and without keep chomping, which the lexer reads as its body although chomping drops them from its value (YAML 1.2.2 §8.1.1.2).

| Repository | Commit | YAML files | Lexically invalid |
| --- | --- | ---: | ---: |
| [kubernetes/kubernetes](https://github.com/kubernetes/kubernetes) | `f2a57ac2b9` | 6,599 | 29 |
| [argoproj/argo-cd](https://github.com/argoproj/argo-cd) | `0a82eca9c8` | 2,744 | 32 |
| [ansible/ansible](https://github.com/ansible/ansible) | `efb06653bb` | 2,107 | 1 |
| [prometheus-community/helm-charts](https://github.com/prometheus-community/helm-charts) | `77d25d4bcd` | 953 | 573 |
| [github/docs](https://github.com/github/docs) | `59fde2de47` | 578 | 0 |
| [home-assistant/home-assistant.io](https://github.com/home-assistant/home-assistant.io) | `552c85771c` | 76 | 0 |

Every valid file matches on every line. Most invalid files are templates, Helm charts' among them, that are YAML only once their `{{ … }}` directives are rendered; the others are tests of input a strict parser rejects. Fork `a6c2728` differed on 4 files, one line each: a line of whitespace beyond a block scalar's indentation, part of its text (§8.1.2, `l-nb-literal-text`), which it counted as blank.

## Markdown

Markdown's text counts as comment lines and each fenced code block's content in the block's language, so the judge decides where the blocks are. Judge: commonmark.js 0.31.2, CommonMark's reference implementation, which reports each fenced block's lines, its info string and its content without the prefixes of the list items or block quotes holding it. `judgecmp markdown` counts each file from them: the fences as comment lines, a block's content as the fork counts its language, and every other line as text, or blank by CommonMark's definition, under which a line of no-break spaces is text (0.31.2, §2.1); it compares these counts with the fork's, file by file. Both take a block's language as Tokei does, from the first word of its info string matched against language names, so a ```` ```js ```` block counts as Markdown text.

| Repository | Commit | Markdown files |
| --- | --- | ---: |
| [github/docs](https://github.com/github/docs) | `59fde2de47` | 7,729 |
| [home-assistant/home-assistant.io](https://github.com/home-assistant/home-assistant.io) | `552c85771c` | 4,240 |
| [kubernetes/kubernetes](https://github.com/kubernetes/kubernetes) | `f2a57ac2b9` | 595 |
| [argoproj/argo-cd](https://github.com/argoproj/argo-cd) | `0a82eca9c8` | 505 |
| [sveltejs/kit](https://github.com/sveltejs/kit) | `04aac59bad` | 162 |
| [sveltejs/svelte](https://github.com/sveltejs/svelte) | `fe21fce019` | 138 |
| [vuejs/docs](https://github.com/vuejs/docs) | `40aa88af00` | 122 |
| [element-plus/element-plus](https://github.com/element-plus/element-plus) | `a8d102a91b` | 115 |
| [luarocks/luarocks](https://github.com/luarocks/luarocks) | `85bfe36c81` | 71 |
| [ansible/ansible](https://github.com/ansible/ansible) | `efb06653bb` | 65 |
| [prometheus-community/helm-charts](https://github.com/prometheus-community/helm-charts) | `77d25d4bcd` | 65 |
| The other nine Lua repositories above and [PanJiaChen/vue-element-admin](https://github.com/PanJiaChen/vue-element-admin) `6858a9ad67` | | 125 |

Every one of the 13,932 files matches. Fork `a6c2728` differed on 634: it found ```` ```lang ```` anywhere in a line, so a fence inside a paragraph opened a block, missed `~~~` fences and blocks in list items, dropped the blank lines at a block's start and end, and counted the words after the language in an info string, such as Docusaurus's `title="…"`, as the block's first code line too, adding a line to the file. Fork `bdc6aff` found the blocks with pulldown-cmark 0.13.5 in the whole file, since a list item before a fence decides whether it opens one, and also matched on every file; fork `72d47d8` ports commonmark.js's block pass instead, for speed, and matches on every file as well.

## Perl

Perl can only be parsed by running it: `perl -c` executes `BEGIN` blocks and `use` statements, so it cannot judge untrusted files. Judge: PPI 1.270, the static Perl parser that Perl::Critic is built on. Comments and POD are comments; strings, quote-like operators other than `<FH>`, regular expressions and heredoc bodies are literals, and data after `__END__` or `__DATA__` is in no range. PPI looks for POD after `__END__` but not after `__DATA__`, while Pod parsers read a block wherever a line starts with `=` and a letter ([perlpodspec, "Pod Definitions"](https://perldoc.perl.org/perlpodspec#Pod-Definitions)), and modules such as Opcode keep their documentation there, so the judge finds POD after `__DATA__` too.

| Repository | Commit | Perl files | Lexically invalid |
| --- | --- | ---: | ---: |
| [Perl/perl5](https://github.com/Perl/perl5) | `b3e8681e59` | 1,312 | 2 |
| [moose/Moose](https://github.com/moose/Moose) | `98ec7dda05` | 466 | 0 |
| [bestpractical/rt](https://github.com/bestpractical/rt) | `f1ba6d9250` | 371 | 0 |
| [exiftool/exiftool](https://github.com/exiftool/exiftool) | `980409314f` | 241 | 0 |
| [Perl-Critic/Perl-Critic](https://github.com/Perl-Critic/Perl-Critic) | `368a20326a` | 205 | 0 |
| [miyagawa/cpanminus](https://github.com/miyagawa/cpanminus) | `c034dd1d38` | 172 | 0 |
| [mojolicious/mojo](https://github.com/mojolicious/mojo) | `4146e40249` | 164 | 0 |
| [PerlDancer/Dancer2](https://github.com/PerlDancer/Dancer2) | `64218bec27` | 119 | 0 |
| [Perl-Toolchain-Gang/ExtUtils-MakeMaker](https://github.com/Perl-Toolchain-Gang/ExtUtils-MakeMaker) | `55834fc274` | 68 | 0 |
| [beyondgrep/ack3](https://github.com/beyondgrep/ack3) | `11d698f940` | 37 | 0 |

Every valid file but one matches on every line. In perl5's `ext/Devel-Peek/Peek.pm`, PPI reads `1<<index(…)` as a heredoc and the 509 lines after it as its body; `perl -MO=Deparse` on that expression alone reads a shift, as the fork does. The two invalid files are perl5's `Module-Metadata` byte order mark test, which PPI returns no document for, and `regen/mk_invlists.pl`, whose multi-line signature with comments PPI does not rebuild unchanged. Six files that are not UTF-8 are not judged. Fork `99502cb` differed on 57 files, on 7,166 lines, against the judge's first version: the punctuation variables `$'`, `$"`, `` $` `` and `$#` opened strings or comments, regular expressions ended at their first line, quote-like operators with `,`, `;` or other unusual delimiters, such as `m,/,`, were code, backtick commands were code, and a `/` after a regular expression opened another one rather than dividing. Fork `3738416` reads them as PPI does, with PPI's rule for when a quote-like word is a name instead, such as `$h{s}`, `->q(` or `y => 1`.

## Shell

A shell reads a script only as it runs it, and `bash -n` or `zsh -n` report only whether a script parses. Judge: mvdan.cc/sh v3.14.1, the parser shfmt runs, which reads POSIX sh, Bash, mksh and, experimentally, Zsh. Files ending in `.zsh` are parsed as Zsh and all others as Bash; a file one rejects is parsed as the other, since zsh scripts are often named `.sh` and completion scripts hold branches for both. Comments are comments; single-quoted, `$'…'` and double-quoted strings and heredoc bodies are literals, except the command substitutions a double-quoted string holds, which are code.

| Repository | Commit | Shell files | Lexically invalid |
| --- | --- | ---: | ---: |
| [termux/termux-packages](https://github.com/termux/termux-packages) | `34a8c84d04` | 4,074 | 1 |
| [git/git](https://github.com/git/git) | `b9e5cff180` | 1,306 | 7 |
| [ohmyzsh/ohmyzsh](https://github.com/ohmyzsh/ohmyzsh) | `17a2b4c3a5` | 411 | 47 |
| [Bash-it/bash-it](https://github.com/Bash-it/bash-it) | `72ea9e285f` | 345 | 0 |
| [kubernetes/kubernetes](https://github.com/kubernetes/kubernetes) | `f2a57ac2b9` | 298 | 0 |
| [acmesh-official/acme.sh](https://github.com/acmesh-official/acme.sh) | `37d064f7b5` | 294 | 0 |
| [bats-core/bats-core](https://github.com/bats-core/bats-core) | `5b61d8f22b` | 52 | 0 |
| [pyenv/pyenv](https://github.com/pyenv/pyenv) | `a2d0ea8903` | 11 | 0 |
| [nvm-sh/nvm](https://github.com/nvm-sh/nvm) | `5e38bc0cae` | 8 | 0 |
| [rbenv/rbenv](https://github.com/rbenv/rbenv) | `07e9b1e297` | 3 | 0 |

Every valid file matches on every line. Of the 55 invalid files, 6 are scripts no shell accepts: git-gui's Tcl programs behind a shell trampoline and a zunit test. The other 49 parse with `zsh -n` or `bash -n` but use zsh syntax the judge does not read yet, such as parameter expansion flags in some positions and floating-point arithmetic, so the reference cannot check them. Fork `2601ec3` differed on 2 files, on 40 lines: at the second `<` of a `<<<` here-string, which has no body ([Bash Reference Manual, "Here Strings"](https://www.gnu.org/software/bash/manual/html_node/Redirections.html#Here-Strings)), it found a heredoc with an empty word, whose body ended at the next blank line; and after an opening backquote, whose text is read as a script of its own (POSIX.1-2024 §2.6.3), it did not start a comment at `#`, so a quote in a comment of Kubernetes' `kube_codegen.sh` opened a string. Fork `213fd8e` reads both as shells do.

## Groovy

Judge: `GroovyLangLexer` from Groovy 5.1.3's Parrot parser, the lexer `groovyc` runs. It turns comments into newline tokens or drops them, so the judge reads them from those tokens and from the text between tokens; strings, slashy strings and the text of GStrings are literals, and their `$name` and `${…}` interpolations are code.

| Repository | Commit | Groovy files | Lexically invalid |
| --- | --- | ---: | ---: |
| [apache/grails-core](https://github.com/apache/grails-core) | `7047ca37b4` | 7,621 | 2 |
| [apache/groovy](https://github.com/apache/groovy) | `396e862eda` | 2,847 | 0 |
| [nextflow-io/nextflow](https://github.com/nextflow-io/nextflow) | `a8bcfb2f04` | 1,696 | 0 |
| [jenkinsci/job-dsl-plugin](https://github.com/jenkinsci/job-dsl-plugin) | `ffe27ca917` | 902 | 0 |
| [spockframework/spock](https://github.com/spockframework/spock) | `ce7e8afa89` | 609 | 0 |

Every valid file matches on every line. The two invalid files are a grails-core code-generation template and a script that the Groovy lexer rejects. Fork `2b00d58` differed on 1,375 files, on 51,280 lines: Groovy was declared with `"…"` strings alone, so `'…'`, `'''…'''` and `"""…"""` strings, slashy strings such as `/a'b/` and `$/…/$` dollar-slashy strings were read as code, and a quote or comment marker inside them changed the lines after it. Fork `a70c21f` declares Groovy's strings, its interpolations and its slashy strings where an expression may start ([Groovy Language Documentation, Syntax](https://groovy-lang.org/syntax.html)).

## JSON

Judge: the scanner of `jsonc-parser` 3.3.1, which VS Code reads both JSON and JSON with Comments with, since both share the `.json` extension; strict JSON is the subset without comments. Strings are literals and comments are comments.

| Repository | Commit | JSON files | Lexically invalid |
| --- | --- | ---: | ---: |
| [microsoft/vscode](https://github.com/microsoft/vscode) | `0d958c41dd` | 1,208 | 0 |
| [kubernetes/kubernetes](https://github.com/kubernetes/kubernetes) | `f2a57ac2b9` | 1,159 | 0 |
| [sveltejs/svelte](https://github.com/sveltejs/svelte) | `fe21fce019` | 463 | 0 |
| [github/docs](https://github.com/github/docs) | `59fde2de47` | 381 | 0 |
| [supabase/supabase](https://github.com/supabase/supabase) | `65217161af` | 235 | 0 |
| [sveltejs/kit](https://github.com/sveltejs/kit) | `04aac59bad` | 214 | 0 |
| [prettier/prettier](https://github.com/prettier/prettier) | `c7a823c4` | 111 | 26 |
| [ansible/ansible](https://github.com/ansible/ansible) | `efb06653bb` | 83 | 16 |
| 37 other checkouts of the corpora above | | 1,256 | 4 |

Every valid file matches on every line. The 46 invalid files are test fixtures of malformed JSON, such as Prettier's and Ansible's. Fork `2b00d58` differed on 102 files, on 679 lines: JSON declared no comments or strings, so the `//` and `/* */` comments of JSON with Comments, as in VS Code's settings and `tsconfig.json` files, counted as code. Fork `fc28619` declares them, with `"…"` strings so that a comment marker in a string stays text ([RFC 8259](https://www.rfc-editor.org/rfc/rfc8259), [VS Code, JSON with Comments](https://code.visualstudio.com/docs/languages/json#_json-with-comments)).

## TOML

Judge: the lexer of `toml_parser` 1.1.5, under the `toml` crate 0.9.12 that Cargo reads manifests with; a file `toml` does not parse is invalid. Strings are literals and comments are comments.

| Repository | Commit | TOML files | Lexically invalid |
| --- | --- | ---: | ---: |
| [toml-lang/toml-test](https://github.com/toml-lang/toml-test) | `96fd088448` | 776 | 499 |
| [helix-editor/helix](https://github.com/helix-editor/helix) | `ba40e54742` | 242 | 0 |
| [astral-sh/ruff](https://github.com/astral-sh/ruff) | `16cffe4a80` | 150 | 0 |
| 33 other checkouts of the corpora above | | 134 | 0 |

Every valid file matches on every line. The 499 invalid files are toml-test's invalid cases, which exist to be rejected. Fork `2b00d58` differed on 1 file, on 5 lines: a backslash in a `'…'` or `'''…'''` literal string, which has no escapes, hid its closing quote. Fork `485bcb3` declares literal strings as verbatim and lets one or two quotes before a multi-line string's closing delimiter belong to the string ([TOML 1.0, String](https://toml.io/en/v1.0.0#string)).

## Dockerfile

Judge: BuildKit's Dockerfile parser, `github.com/moby/buildkit/frontend/dockerfile/parser` v0.34.1, for instructions, heredoc bodies and validity; a line starting with `#` is a comment, as the [Dockerfile reference](https://docs.docker.com/reference/dockerfile/#format) defines, and heredoc bodies are literals.

| Repository | Commit | Dockerfiles | Lexically invalid |
| --- | --- | ---: | ---: |
| [docker-library/php](https://github.com/docker-library/php) | `5ec2a6a0da` | 66 | 0 |
| [moby/buildkit](https://github.com/moby/buildkit) | `c1cfdbd30b` | 60 | 4 |
| [kubernetes/kubernetes](https://github.com/kubernetes/kubernetes) | `f2a57ac2b9` | 40 | 0 |
| [docker-library/python](https://github.com/docker-library/python) | `f4c67ece20` | 40 | 0 |
| [docker/awesome-compose](https://github.com/docker/awesome-compose) | `30f4b7f6a6` | 35 | 0 |
| [jupyter/docker-stacks](https://github.com/jupyter/docker-stacks) | `22d92ea8f1` | 31 | 0 |
| [docker-library/postgres](https://github.com/docker-library/postgres) | `dfc76bb299` | 24 | 0 |
| [docker-library/golang](https://github.com/docker-library/golang) | `4b919ec6b2` | 20 | 0 |
| [nodejs/docker-node](https://github.com/nodejs/docker-node) | `14b1d124e2` | 18 | 0 |
| 21 other checkouts of the corpora above | | 100 | 0 |

Every valid file matches on every line. The four invalid files are BuildKit's negative parser tests. Fork `2b00d58` differed on 19 files, on 54 lines: a `#` anywhere started a comment, and quotes, which belong to the shell in `RUN` and to JSON in exec form, opened strings across lines, as the apostrophe in `echo it's` did. Fork `5ab59ff` reads `#` comments only at the start of a line, declares no strings, and reads `<<WORD` heredocs.

## Makefile

GNU make has no tokenizer to call. Judge: a port of GNU make 4.4.1's reading rules from its `src/read.c`, checked against `make` 4.4.1 on synthetic makefiles. It joins a line ending in an odd number of backslashes to the next; cuts a makefile line at the first `#` that no backslash escapes and that lies outside `$(…)` and `${…}` references, so a comment continues over the lines it joins; reads a tab-led line as a recipe once a rule line has been read; and reads recipes by the shell's rules (POSIX.1-2024 §2.3), where `#` comments out only the rest of its line, and make's `@`, `-` and `+` flags before a comment are part of it. The lines of a `define` body, which make keeps unread until `$(eval)` or a recipe reads them, count as comments only where both readings agree: when they start with `#`. Conditionals are not evaluated, so every branch is read; nmake files, which GNU make rejects, are invalid.

| Repository | Commit | Makefiles | Lexically invalid |
| --- | --- | ---: | ---: |
| [buildroot/buildroot](https://github.com/buildroot/buildroot) | `93156d5848` | 3,507 | 0 |
| [torvalds/linux](https://github.com/torvalds/linux), makefiles only | `d89e1d186e` | 3,244 | 0 |
| [postgres/postgres](https://github.com/postgres/postgres) | `328a9a1731` | 197 | 0 |
| [kubernetes/kubernetes](https://github.com/kubernetes/kubernetes) | `f2a57ac2b9` | 48 | 0 |
| [git/git](https://github.com/git/git) | `b9e5cff180` | 21 | 0 |
| 27 other checkouts of the corpora above | | 170 | 3 |

Every valid file matches on every line. The three invalid files are perl5's nmake `win32/Makefile` and two files that make rejects: Dancer2's, which indents recipes with spaces, and a prose file in ack3's test tree. Fork `2b00d58` differed on 13 files, on 45 lines: recipe lines such as `@# …`, most of them, were code, and a comment ending in a backslash did not continue on the next line, as in VS Code's makefile grammar fixture. Fork `9f25025` continues makefile comments over odd runs of backslashes, but not in a recipe or `define` body, whose comments the shell may read; reads `\#` as text and a `#` inside a reference as part of it; and counts `@#`, `-#` and `+#` lines as comments ([GNU make manual, 3.1 What Makefiles Contain](https://www.gnu.org/software/make/manual/html_node/Makefile-Contents.html), [5.1.1 Splitting Recipe Lines](https://www.gnu.org/software/make/manual/html_node/Splitting-Recipe-Lines.html)). Since that fork, an escaped line comment marker outside literals is text in every language, and, since fork `a022f12`, a comment after any run of make's `@`, `-` and `+` recipe flags with blanks among them, such as `@-#` or `+ #`, counts as one, as `start_job_command` in GNU make 4.4.1's `src/job.c` strips them all; a synthetic recipe found that gap, and no judged corpus changed.

## Batch

Judge: cmd.exe's reading rules, from Microsoft's documentation of [`rem`](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/rem), [`goto` labels](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/goto) and the `@` of [`echo`](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/echo), and, where it is silent, from Wine's reimplementation in `programs/cmd/wcmdmain.c`, since cmd.exe's source is closed and it runs only on Windows. A command starts after blanks and `@` (`lexer_strip_for_command_start`). `rem`, in any case, followed by a blank or the end of the line makes the line a remark (`WCMD_ReadAndParseLine`); followed by one of `+./(;=:`, it is still the `rem` command (`search_command`), which runs nothing unless an `&` or `|` outside quotes runs more, as Microsoft's `Rem/||(` example does. A line starting with `:` is a label, and a `::` label is a remark. A caret before the newline joins a code line to the next. Batch has no lexical errors.

| Repository | Commit | Batch files | Lexically invalid |
| --- | --- | ---: | ---: |
| [npocmaka/batch.scripts](https://github.com/npocmaka/batch.scripts) | `7e784303a5` | 197 | 0 |
| [microsoft/vscode](https://github.com/microsoft/vscode) | `0d958c41dd` | 24 | 0 |
| [apache/tomcat](https://github.com/apache/tomcat) | `8c164da347` | 12 | 0 |
| [buildroot/buildroot](https://github.com/buildroot/buildroot) | `93156d5848` | 12 | 0 |
| [microsoft/vcpkg](https://github.com/microsoft/vcpkg) | `d186940b2f` | 6 | 0 |
| 38 other checkouts of the corpora above | | 117 | 0 |

Every file matches on every line. Fork `fe46d48` differed on 120 files, on 2,071 lines: Batch declared `REM` and `::` as line comments, in upper case only and anywhere on a line, so `rem` and `@rem` remarks, such as the license headers of Gradle's and Maven's wrapper scripts, counted as code, and a line starting with a word such as `REMOTE` as a comment. Fork `ea14733` reads remarks as cmd.exe does. It does not join a line over a caret, so a remark-like line after a caret-continued command would count as a comment; none of the corpus's caret continuations is followed by one.

## INI

Judge: Python's `configparser`, run with CPython 3.13, since INI has no standard and Tokei's `;` and `#` comments are its default comment prefixes. A line is a comment when, stripped as `str.strip` strips it, it starts with a prefix, even inside a value continued over indented lines (`_Line._strip_full` in `Lib/configparser.py`); inline comments are off by default. A file that `RawConfigParser` rejects is invalid.

| Repository | Commit | INI files | Lexically invalid |
| --- | --- | ---: | ---: |
| [ansible/ansible](https://github.com/ansible/ansible) | `efb06653bb` | 28 | 6 |
| [miyagawa/cpanminus](https://github.com/miyagawa/cpanminus) | `c034dd1d38` | 5 | 2 |
| 10 other checkouts of the corpora above | | 15 | 5 |

Every file matches on every line, the invalid ones included: most hold keys before the first section, as Ansible inventories and Dist::Zilla's `dist.ini` files do, which `configparser` rejects while reading their comments as it reads any. Fork `fe46d48` differed on none. Fork `7598c1b` counts lines holding only the Unicode whitespace that `str.strip` removes, such as a no-break space, as blank; a comment after such whitespace still counts as code, as [DEFERRED](../DEFERRED.md#measurement-and-comparison) records.

## Speed

The owner decided on 2026-10-30 that the fork must parse within 10% of upstream Tokei 15.0.0's time on every repository below; the fork's [speed gate](https://github.com/mbackschat/tokei/tree/locmetry/judges#speed), `judges/speed/check.py`, measures it before each release is adopted. It parses every file Tokei recognizes from memory, three times, and compares the minimum user CPU time of 15 interleaved runs. Django, Tokio, Prettier, Swift NIO, bloc, Cats, ggplot2, Phoenix, Pester, pandoc, cabal, Tomcat, Bootstrap, Element Plus, SvelteKit, Svelte, KOReader, Citus, Prometheus's Helm charts, perl5, Moose, termux-packages, acme.sh, Buildroot and Git are whole checkouts, GitHub's docs are measured on their `content` folder, Linux on its makefiles, and the other trees hold the files of their judged languages.

Upstream's Haskell declares no strings, so it never analyses the lines holding them, a quarter of cabal's Haskell lines and a third of pandoc's; against it, fork `994df39` took 1.10 times as long on pandoc and 1.38 times on cabal, and upstream's own engine took 3.4 times its time on cabal's Haskell files once given the quotes. The owner decided on 2026-10-30 that the gate gives upstream the string quotes of languages whose upstream definition has none, listed in the gate's `baseline-quotes.json`, so it measures the fork's own overhead; the Haskell rows below are measured that way.

| Repository | Commit | Language | Upstream | Fork `f8826bd` | Ratio |
| --- | --- | --- | ---: | ---: | ---: |
| [symfony/symfony](https://github.com/symfony/symfony) | `92216211` | PHP | 4.479 s | 1.169 s | 0.26 |
| [koreader/koreader](https://github.com/koreader/koreader) | `80127739` | Lua | 0.375 s | 0.136 s | 0.36 |
| [django/django](https://github.com/django/django) | `817df1c4` | Python | 1.538 s | 0.695 s | 0.45 |
| [twbs/bootstrap](https://github.com/twbs/bootstrap) | `c967e2cb` | SCSS, CSS | 0.175 s | 0.081 s | 0.46 |
| [prometheus-community/helm-charts](https://github.com/prometheus-community/helm-charts) | `77d25d4b` | YAML | 0.345 s | 0.159 s | 0.46 |
| [element-plus/element-plus](https://github.com/element-plus/element-plus) | `a8d102a9` | Vue | 0.410 s | 0.187 s | 0.46 |
| [torvalds/linux](https://github.com/torvalds/linux) makefiles | `d89e1d18` | Makefile | 0.040 s | 0.019 s | 0.47 |
| [pester/Pester](https://github.com/pester/Pester) | `98e56d65` | PowerShell | 0.139 s | 0.069 s | 0.50 |
| [prettier/prettier](https://github.com/prettier/prettier) | `c7a823c4` | JavaScript, TypeScript | 0.424 s | 0.215 s | 0.51 |
| [phoenixframework/phoenix](https://github.com/phoenixframework/phoenix) | `c9065a89` | Elixir | 0.110 s | 0.059 s | 0.54 |
| [jgm/pandoc](https://github.com/jgm/pandoc) | `75ebbfa7` | Haskell | 0.184 s | 0.100 s | 0.54 |
| [felangel/bloc](https://github.com/felangel/bloc) | `b9be1e25` | Dart | 0.186 s | 0.103 s | 0.55 |
| [tidyverse/ggplot2](https://github.com/tidyverse/ggplot2) | `b638d654` | R | 0.162 s | 0.092 s | 0.57 |
| [npocmaka/batch.scripts](https://github.com/npocmaka/batch.scripts) | `7e784303` | Batch | 0.007 s | 0.004 s | 0.57 |
| [tokio-rs/tokio](https://github.com/tokio-rs/tokio) | `6470b5e4` | Rust | 0.101 s | 0.060 s | 0.59 |
| [moose/Moose](https://github.com/moose/Moose) | `98ec7dda` | Perl | 0.074 s | 0.044 s | 0.59 |
| [git/git](https://github.com/git/git) | `b9e5cff1` | C, Shell, Makefile | 0.966 s | 0.580 s | 0.60 |
| [termux/termux-packages](https://github.com/termux/termux-packages) | `34a8c84d` | Shell | 0.613 s | 0.374 s | 0.61 |
| [Perl/perl5](https://github.com/Perl/perl5) | `b3e8681e` | Perl | 1.292 s | 0.805 s | 0.62 |
| [acmesh-official/acme.sh](https://github.com/acmesh-official/acme.sh) | `37d064f7` | Shell | 0.114 s | 0.072 s | 0.63 |
| [rubocop/rubocop](https://github.com/rubocop/rubocop) | `bde5d420` | Ruby | 0.396 s | 0.249 s | 0.63 |
| [buildroot/buildroot](https://github.com/buildroot/buildroot) | `93156d58` | Makefile | 0.326 s | 0.208 s | 0.64 |
| [sveltejs/kit](https://github.com/sveltejs/kit) | `04aac59b` | Svelte | 0.177 s | 0.114 s | 0.64 |
| [gohugoio/hugo](https://github.com/gohugoio/hugo) | `a1668da3` | Go | 0.191 s | 0.128 s | 0.67 |
| [citusdata/citus](https://github.com/citusdata/citus) | `843ef6f0` | SQL, C | 0.390 s | 0.267 s | 0.68 |
| [apache/tomcat](https://github.com/apache/tomcat) | `8c164da3` | XML, HTML | 0.691 s | 0.474 s | 0.69 |
| [spring-projects/spring-framework](https://github.com/spring-projects/spring-framework) | `64861cb3` | Java, Kotlin | 1.106 s | 0.801 s | 0.72 |
| [github/docs](https://github.com/github/docs) `content` | `59fde2de` | Markdown | 0.221 s | 0.164 s | 0.74 |
| [apple/swift-nio](https://github.com/apple/swift-nio) | `108daca3` | Swift | 0.133 s | 0.102 s | 0.77 |
| [haskell/cabal](https://github.com/haskell/cabal) | `07e003b3` | Haskell | 0.185 s | 0.145 s | 0.78 |
| [curl/curl](https://github.com/curl/curl) | `87c4c6d6` | C | 0.265 s | 0.209 s | 0.79 |
| [AutoMapper/AutoMapper](https://github.com/AutoMapper/AutoMapper) | `df65f2e3` | C# | 0.043 s | 0.035 s | 0.81 |
| [typelevel/cats](https://github.com/typelevel/cats) | `b6c4dbfb` | Scala | 0.094 s | 0.078 s | 0.83 |
| [sveltejs/svelte](https://github.com/sveltejs/svelte) | `fe21fce0` | Svelte | 0.261 s | 0.231 s | 0.89 |

Measured on 2026-10-30 on a loaded machine, where the minimum of interleaved runs varied by about 3% between repetitions. Fork `1806dc8` found Markdown's code blocks with pulldown-cmark, whose first pass also scans inline syntax: on GitHub's `content`, 3,796 Markdown files, it took 1.05 and 1.07 times upstream's time in two runs, but 1.37 in a later run in which upstream ran faster, so it failed the gate, and its first pass alone took 28 of the 48 ms that upstream's whole count of those files took. Fork `72d47d8` reads only CommonMark's block structure, ported from commonmark.js, and took 0.79, but from 0.97 to above 1.10 over 30 runs once the machine was quiet; fork `6758800` counts Markdown's lines straight from the block list instead of the generic line loop, skips the block pass for files without three backticks or tildes in a row, 2,806 of the 3,796, and finds line ends with a vectorized search, and took 0.92 over 30 runs and 0.71 in an earlier run of the table, 0.74 in that of fork `f8826bd`. Cabal ranged from 0.64 to 0.95 between runs of unchanged Haskell code and measured 0.69 and 0.71 over 30 runs. Fork `a6c2728` measured from 0.27 to 0.83 on the first twenty-four. Fork `9c85621` measured from 0.26 to 0.83 on the first twenty-two. Swift NIO, bloc, Cats and ggplot2 were measured as files of their judged languages until fork `397da12`, which measured from 0.27 to 0.82 on the first twenty, and as whole checkouts since, after the gate was found passing roots that no longer existed; it fails them since fork `47e6301`. Fork `ffb6076` measured from 0.25 to 0.83 on the first seventeen. Fork `994df39` passes over runs of bytes that change nothing at once, which cut the ratios from 0.45–0.96 for fork `687cb25`; earlier measurements found ratios from 0.53 to 0.92 for fork `55b0c87` and from 0.42 to 0.94 for fork `984e57b`.
