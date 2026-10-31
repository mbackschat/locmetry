# Tokei Fork Step 6: Each Language's Judge

Measured from 2026-10-29 on macOS arm64, for [step 6](../plans/archive/tokei-fork.md#step-6-no-difference-from-each-languages-judge): every file of every corpus must count as the language's own lexer or parser counts it. A line is a comment when it holds nothing but comments; a blank line takes the class of the comment or literal it lies in; every other line is code. A file the judge itself rejects as lexically invalid has no correct count, so it is listed rather than matched.

The judges and the comparison tool live in the fork's [judges](https://github.com/mbackschat/tokei/tree/locmetry/judges) folder from `a2a9724`. Each judge reports the comment and literal ranges its lexer found, and `judgecmp` compares the lines they imply with the fork's `line_classes` (fork `238b0c5`), line by line. The fork's results below are those of `15.0.0+locmetry.13`, fork `a55e730`, for Python of `15.0.0+locmetry.14`, fork `701fdaa`, for C, C++, Objective-C, Go, C# and Ruby of `15.0.0+locmetry.15`, fork `40ed432`, for PHP, Swift, Dart and Scala of `15.0.0+locmetry.16`, fork `eb463fb`, for R of `15.0.0+locmetry.17`, fork `984e57b`, for Elixir and PowerShell of `15.0.0+locmetry.18`, fork `687cb25`, for Haskell of `15.0.0+locmetry.19`, fork `ffb6076`, for XML, SVG, CSS, SCSS, HTML and Vue of `15.0.0+locmetry.20`, fork `397da12`, for Svelte of `15.0.0+locmetry.21`, fork `9c85621`, for Lua and SQL of `15.0.0+locmetry.22`, fork `a6c2728`, for YAML of `15.0.0+locmetry.23`, fork `1806dc8`, and for Markdown of `15.0.0+locmetry.24`, fork `99502cb`. A line of whitespace takes the class of the comment or literal its own bytes lie in, as judgecmp reads it from fork `02345d9`; every earlier result was rechecked under that rule. File counts count each file of a repository at one commit once, however many checkouts of that commit the corpora hold; on 2026-10-30 that recount corrected the TOML, Dockerfile, Makefile and INI tables.

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
| 42 other checkouts of the corpora above | | 128 | 0 |

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
| 45 other checkouts of the corpora above | | 93 | 0 |

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
| 46 other checkouts of the corpora above | | 142 | 3 |

Every valid file matches on every line. The three invalid files are perl5's nmake `win32/Makefile` and two files that make rejects: Dancer2's, which indents recipes with spaces, and a prose file in ack3's test tree. Fork `2b00d58` differed on 10 files, on 40 lines: recipe lines such as `@# …`, most of them, were code, and a comment ending in a backslash did not continue on the next line, as in VS Code's makefile grammar fixture. Fork `9f25025` continues makefile comments over odd runs of backslashes, but not in a recipe or `define` body, whose comments the shell may read; reads `\#` as text and a `#` inside a reference as part of it; and counts `@#`, `-#` and `+#` lines as comments ([GNU make manual, 3.1 What Makefiles Contain](https://www.gnu.org/software/make/manual/html_node/Makefile-Contents.html), [5.1.1 Splitting Recipe Lines](https://www.gnu.org/software/make/manual/html_node/Splitting-Recipe-Lines.html)). Since that fork, an escaped line comment marker outside literals is text in every language, and, since fork `a022f12`, a comment after any run of make's `@`, `-` and `+` recipe flags with blanks among them, such as `@-#` or `+ #`, counts as one, as `start_job_command` in GNU make 4.4.1's `src/job.c` strips them all; a synthetic recipe found that gap, and no judged corpus changed.

## Batch

Judge: cmd.exe's reading rules, from Microsoft's documentation of [`rem`](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/rem), [`goto` labels](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/goto) and the `@` of [`echo`](https://learn.microsoft.com/en-us/windows-server/administration/windows-commands/echo), and, where it is silent, from Wine's reimplementation in `programs/cmd/wcmdmain.c`, since cmd.exe's source is closed and it runs only on Windows. A command starts after blanks and `@` (`lexer_strip_for_command_start`). `rem`, in any case, followed by a blank or the end of the line makes the line a remark (`WCMD_ReadAndParseLine`); followed by one of `+./(;=:`, it is still the `rem` command (`search_command`), which runs nothing unless an `&` or `|` outside quotes runs more, as Microsoft's `Rem/||(` example does. A line starting with `:` is a label, and a `::` label is a remark. A caret before the newline joins a code line to the next. Batch has no lexical errors.

| Repository | Commit | Batch files | Lexically invalid |
| --- | --- | ---: | ---: |
| [npocmaka/batch.scripts](https://github.com/npocmaka/batch.scripts) | `7e784303a5` | 197 | 0 |
| [microsoft/vscode](https://github.com/microsoft/vscode) | `0d958c41dd` | 24 | 0 |
| [apache/tomcat](https://github.com/apache/tomcat) | `8c164da347` | 12 | 0 |
| [buildroot/buildroot](https://github.com/buildroot/buildroot) | `93156d5848` | 12 | 0 |
| [microsoft/vcpkg](https://github.com/microsoft/vcpkg) | `d186940b2f` | 6 | 0 |
| 37 other checkouts of the corpora above | | 110 | 0 |

Every file matches on every line. Fork `fe46d48` differed on 120 files, on 2,071 lines: Batch declared `REM` and `::` as line comments, in upper case only and anywhere on a line, so `rem` and `@rem` remarks, such as the license headers of Gradle's and Maven's wrapper scripts, counted as code, and a line starting with a word such as `REMOTE` as a comment. Fork `ea14733` reads remarks as cmd.exe does. It does not join a line over a caret, so a remark-like line after a caret-continued command would count as a comment; none of the corpus's caret continuations is followed by one.

## INI

Judge: Python's `configparser`, run with CPython 3.13, since INI has no standard and Tokei's `;` and `#` comments are its default comment prefixes. A line is a comment when, stripped as `str.strip` strips it, it starts with a prefix, even inside a value continued over indented lines (`_Line._strip_full` in `Lib/configparser.py`); inline comments are off by default. A file that `RawConfigParser` rejects is invalid.

| Repository | Commit | INI files | Lexically invalid |
| --- | --- | ---: | ---: |
| [ansible/ansible](https://github.com/ansible/ansible) | `efb06653bb` | 27 | 6 |
| [miyagawa/cpanminus](https://github.com/miyagawa/cpanminus) | `c034dd1d38` | 5 | 2 |
| 10 other checkouts of the corpora above | | 15 | 5 |

Every file matches on every line, the invalid ones included: most hold keys before the first section, as Ansible inventories and Dist::Zilla's `dist.ini` files do, which `configparser` rejects while reading their comments as it reads any. Fork `fe46d48` differed on none. Fork `7598c1b` counts lines holding only the Unicode whitespace that `str.strip` removes, such as a no-break space, as blank; a comment after such whitespace still counts as code, as [DEFERRED](../DEFERRED.md#measurement-and-comparison) records.

## XML-Based Languages

Judge: expat, as for [XML and SVG](#xml-and-svg), on the languages whose files are XML documents: MSBuild projects, Visual Studio C++ projects, NuGet configuration, .NET resources, XAML, AXAML and XSL.

| Repository | Commit | Files | Lexically invalid |
| --- | --- | ---: | ---: |
| [PowerShell/PowerShell](https://github.com/PowerShell/PowerShell) | `2725d6aaa6` | 197 | 1 |
| [apache/ant](https://github.com/apache/ant) | `dbb01a2d52` | 39 | 0 |
| [koreader/koreader](https://github.com/koreader/koreader) | `801277394b` | 18 | 0 |
| [PowerShell/PSScriptAnalyzer](https://github.com/PowerShell/PSScriptAnalyzer) | `0151b9dafd` | 8 | 0 |
| [spring-projects/spring-framework](https://github.com/spring-projects/spring-framework) | `64861cb395` | 8 | 1 |
| 10 other checkouts of the corpora above | | 39 | 0 |

Every valid file matches on every line. The two invalid files are XSL stylesheets that expat rejects, one with a comment before its XML declaration. Fork `f8826bd` differed on 7 files, on 62 lines: only XML, SVG and XAML were declared with XML's reading of quotes, so an apostrophe in element text, as in a project's `<Description>PowerShell's …</Description>`, opened a string over the lines after it. Fork `3fc5f6b` declares the other XML-based languages as XML is declared.

## Protocol Buffers

Judge: protocompile v0.14.1, the pure-Go compiler that the buf CLI compiles with in place of protoc; its lexer keeps every comment between the tokens, and string tokens are literals.

| Repository | Commit | Protocol Buffers files | Lexically invalid |
| --- | --- | ---: | ---: |
| [kubernetes/kubernetes](https://github.com/kubernetes/kubernetes) | `f2a57ac2b9` | 126 | 0 |
| [moby/buildkit](https://github.com/moby/buildkit) | `c1cfdbd30b` | 65 | 0 |
| [Kong/kong](https://github.com/Kong/kong) | `21032122ab` | 39 | 1 |
| [apache/pekko](https://github.com/apache/pekko) | `8f3d71b90e` | 24 | 0 |
| [argoproj/argo-cd](https://github.com/argoproj/argo-cd) | `0a82eca9c8` | 19 | 0 |
| 9 other checkouts of the corpora above | | 15 | 0 |

Every valid file matches on every line; the invalid file is a test fixture that protocompile rejects. Fork `f8826bd` differed on 164 files, on 2,536 lines: Protocol Buffers declared only `//` comments, so `/* */` comments counted as code. Fork `1091d87` declares the block comments and the strings of the [language specification](https://protobuf.dev/reference/protobuf/proto3-spec/).

## CMake

Judge: CMake's own list-file lexer, `cmListFileLexer` from CMake 4.4.5's source; bracket comments are tokens, the `#` line comments it skips are read from the text between tokens, and quoted and bracket arguments are literals.

| Repository | Commit | CMake files | Lexically invalid |
| --- | --- | ---: | ---: |
| [microsoft/vcpkg](https://github.com/microsoft/vcpkg) | `d186940b2f` | 4,054 | 0 |
| [flutter/packages](https://github.com/flutter/packages) | `3258d45ac0` | 137 | 0 |
| [swiftlang/swift](https://github.com/swiftlang/swift) | `6d943a0687` | 64 | 0 |
| [swiftlang/swift-package-manager](https://github.com/swiftlang/swift-package-manager) | `1df2f93f7e` | 54 | 0 |
| [flutter/flutter](https://github.com/flutter/flutter) | `d555faf800` | 52 | 0 |
| 15 other checkouts of the corpora above | | 191 | 0 |

Every file matches on every line. Fork `f8826bd` differed on 58 files, on 606 lines: CMake declared neither the `#[[…]]` bracket comments nor the `[[…]]` bracket arguments of [cmake-language(7)](https://cmake.org/cmake/help/latest/manual/cmake-language.7.html#bracket-comment), so a bracket comment's later lines counted as code and a `#` inside a bracket argument started a comment. Fork `3db24ae` declares both, with as many `=` in the closer as in the opener, as Lua's long brackets are declared.

## Cabal

Judge: the comment rules of Cabal-syntax 3.18.2.0's lexer, `Distribution/Fields/Lexer.x`, ported since the lexer drops comments: a line whose first non-blank characters are `--` is a comment, also among a field's continuation lines, and there are no block comments.

| Repository | Commit | Cabal files | Lexically invalid |
| --- | --- | ---: | ---: |
| [haskell/cabal](https://github.com/haskell/cabal) | `07e003b333` | 961 | 0 |
| [haskell/haskell-language-server](https://github.com/haskell/haskell-language-server) | `bc2bbb8e54` | 80 | 0 |
| [commercialhaskell/stack](https://github.com/commercialhaskell/stack) | `11e40c9da1` | 15 | 0 |
| 9 other checkouts of the corpora above | | 22 | 0 |

Every file matches on every line, and fork `f8826bd` already did: none holds `{-`. Cabal was declared with Haskell's nested `{- -}` comments, so a field value holding `{-` would have turned the lines after it into a comment; fork `ec7f406` declares only `--` comments.

## PO File

Judge: GNU gettext's tokenizer, `gettext-tools/src/read-po-lex.c`, ported: outside strings, `#` starts a comment that runs to the end of the line, and `"…"` strings are literals. The lexer reads the `#~` lines of obsolete entries and the `#|` lines of previous strings to check them, but the gettext manual calls obsolete entries commented out and lists `#|` lines among an entry's comments, and msgfmt compiles neither, so they are comments.

| Repository | Commit | PO files | Lexically invalid |
| --- | --- | ---: | ---: |
| [django/django](https://github.com/django/django) | `817df1c4d6` | 1,274 | 0 |
| [wch/r-source](https://github.com/wch/r-source) | `da12bf8030` | 394 | 0 |
| [postgres/postgres](https://github.com/postgres/postgres) | `328a9a1731` | 76 | 0 |
| [git/git](https://github.com/git/git) | `b9e5cff180` | 60 | 0 |
| 4 other checkouts of the corpora above | | 21 | 0 |

Every file matches on every line, and fork `f8826bd` already did.

## Build-Language File Names

Before the build languages below were judged, fork `e42f092` stopped Tokei's claims on shared extensions: every `.in` file had been Autoconf, every `.inc` file Bitbake, every `.spec` file an RPM spec and every `.rules` file Snakemake, and Bazel's `BUILD`, `WORKSPACE` and `MODULE` matched in any case. In the corpora measured here those claims took 3,836 templates, such as Kconfig `Config.in` and `*.cmake.in`, 620 PHP, C and Pascal includes, 237 PostgreSQL isolation test specs and 20 udev rule files. Autoconf now claims `configure.ac` and `configure.in`, Bazel its names in their own case, and a shared extension only files whose text has the language's syntax, following [GitHub Linguist's heuristics](https://github.com/github-linguist/linguist/blob/73ee9a3b27/lib/linguist/heuristics.yml) where it has them. The tables below count the files the fork attributes to each language.

## Meson

Judge: Meson's own lexer, `mesonbuild.mparser.Lexer` from Meson 1.9.1, which `meson setup` reads build files with; strings, f-strings and their `'''` forms are literals, and `#` comments are comments.

| Repository | Commit | Meson files | Lexically invalid |
| --- | --- | ---: | ---: |
| [mesonbuild/meson](https://github.com/mesonbuild/meson) | `0ef4bf221d` | 1,660 | 5 |
| [postgres/postgres](https://github.com/postgres/postgres) | `328a9a1731` | 183 | 0 |
| [git/git](https://github.com/git/git) | `b9e5cff180` | 40 | 0 |
| 2 other checkouts of the corpora above | | 2 | 0 |

Every valid file matches on every line. The five invalid files are Meson's tests of build files it must reject. Fork `e42f092` differed on 1 file, on 4 lines: a backslash before the closing `'''` of a multi-line string, which processes no escapes ([Meson, Syntax](https://mesonbuild.com/Syntax.html#strings)), kept the string open, as in Meson's own string-operation test. Fork `61f32a1` declares the multi-line strings verbatim and recognizes `meson.options`, the options file since Meson 1.1.

## Nextflow

Judge: the [Groovy](#groovy) judge, since Nextflow scripts are Groovy.

| Repository | Commit | Nextflow files | Lexically invalid |
| --- | --- | ---: | ---: |
| [nextflow-io/nextflow](https://github.com/nextflow-io/nextflow) | `a8bcfb2f04` | 350 | 0 |

Every file matches on every line. Fork `61f32a1` differed on 184 files, on 277 lines: Nextflow declared only `"…"` strings, so shebangs, `'…'` strings holding `#` or `//`, and slashy strings were misread. Fork `5312a2e` reads Nextflow with Groovy's syntax, which a language now names in `syntax_of` instead of copying its fields.

## Snakemake

Judge: the [Python](#python) judge, since Snakemake's parser reads Snakefiles with Python's `tokenize` (measured with CPython 3.14).

| Repository | Commit | Snakemake files | Lexically invalid |
| --- | --- | ---: | ---: |
| [snakemake/snakemake](https://github.com/snakemake/snakemake) | `184ad191db` | 527 | 1 |
| [nvim-treesitter/nvim-treesitter](https://github.com/nvim-treesitter/nvim-treesitter) | `a2f694478b` | 2 | 0 |

Every valid file matches on every line; the invalid file is Snakemake's lint test of mixed tabs, which `tokenize` rejects. Fork `61f32a1` already matched every file, but declared no prefixed or f-strings, so a comment in an f-string's replacement field would have been misread; fork `5312a2e` reads Snakefiles with Python's syntax.

## Automake

Judge: the [Makefile](#makefile) judge, since `automake` passes a `Makefile.am` on to make; in `.am` files it also reads Automake's `if COND` conditionals and bare `@NAME@` substitutions as automake and `configure` do ([GNU Automake manual](https://www.gnu.org/software/automake/manual/automake.html#Conditionals)).

| Repository | Commit | Automake files | Lexically invalid |
| --- | --- | ---: | ---: |
| [util-linux/util-linux](https://github.com/util-linux/util-linux) | `becdb8ba9f` | 46 | 0 |
| [strace/strace](https://github.com/strace/strace) | `be6eb508a5` | 6 | 0 |
| [mesonbuild/meson](https://github.com/mesonbuild/meson) | `0ef4bf221d` | 5 | 2 |
| [openembedded-core](https://git.openembedded.org/openembedded-core) | `b62d5e6817` | 1 | 0 |

Every valid file matches on every line. The two invalid files are C++ templates in Meson's tests that only share the `.am` extension. Fork `5312a2e` differed on none of the 53 files the judge could read before it read Automake's conditionals, but declared only `#` comments; fork `5df7a24` reads `.am` files with make's syntax, so comments continued by a backslash and recipe comments after make's `@`, `-` and `+` flags count as they do in makefiles.

## Bazel

Judge: starlark-go `4f4267fb140d`, the Go implementation of [Starlark](https://github.com/bazelbuild/starlark/blob/master/spec.md), the language of BUILD and `.bzl` files; its parser with every dialect option on and comments retained; string and bytes literals are literals.

| Repository | Commit | Bazel files | Lexically invalid |
| --- | --- | ---: | ---: |
| [bazelbuild/rules_python](https://github.com/bazelbuild/rules_python) | `e9734b5ff9` | 840 | 1 |
| [bazelbuild/bazel-skylib](https://github.com/bazelbuild/bazel-skylib) | `5f88ea649f` | 137 | 0 |
| [Kong/kong](https://github.com/Kong/kong) | `21032122ab` | 66 | 0 |
| [kubernetes/kubernetes](https://github.com/kubernetes/kubernetes) | `f2a57ac2b9` | 33 | 0 |
| 3 other checkouts of the corpora above | | 12 | 0 |

Every valid file matches on every line, and fork `5df7a24` already did; the invalid file is a template that holds placeholders.

## RPM Specfile

rpm has no tokenizer to call. Judge: rpmbuild's reading rules, ported from rpm 6's `build/parseSpec.cc`: in the preamble and text sections a line whose first non-blank character is `#` is a comment, and the build scripts and scriptlets, without the `%if` and `%include` lines rpm consumes, are read by the [Shell](#shell) judge; a scriptlet for another interpreter makes the file invalid ([RPM spec file format](https://rpm-software-management.github.io/rpm/manual/spec.html)).

| Repository | Commit | Spec files | Lexically invalid |
| --- | --- | ---: | ---: |
| [terrapkg/packages](https://github.com/terrapkg/packages) | `b1f34f6415` | 1,308 | 9 |
| [rpm-software-management/rpm](https://github.com/rpm-software-management/rpm) | `4f7be32ef2` | 116 | 8 |
| 2 other checkouts of the corpora above | | 2 | 0 |

Every valid file matches on every line. The invalid files hold a scriptlet for another interpreter, such as Lua, or a script the Shell judge rejects; eight are rpm's own test specs. Fork `d72a599` differed on 31 files, on 93 lines: the whole spec was read with one set of rules, so `#` lines in the heredocs and multi-line strings of build scripts, such as a generated script's shebang, counted as comments. Fork `f26b942` splits a spec at rpm's section headers and reads each script by its interpreter.

## Autoconf and M4

Judge: GNU m4 1.4.19's tokenizer, ported from `next_token` in `src/input.c`, since m4 expands as it reads: `#` comments and `dnl` remarks run to the end of their line, and quoted strings nest, with Autoconf's `[` and `]` or m4's `` ` `` and `'` ([GNU m4 manual, Quoted strings](https://www.gnu.org/software/m4/manual/m4.html#Quoted-strings)); text a later expansion rescans is not followed, and a file that changes its quotes or comment delimiters is invalid.

| Repository | Commit | Autoconf files | Lexically invalid |
| --- | --- | ---: | ---: |
| [autoconf-archive/autoconf-archive](https://github.com/autoconf-archive/autoconf-archive) | `92c33ce5c3` | 587 | 19 |
| [strace/strace](https://github.com/strace/strace) | `be6eb508a5` | 15 | 0 |
| [postgres/postgres](https://github.com/postgres/postgres) | `328a9a1731` | 13 | 0 |
| [util-linux/util-linux](https://github.com/util-linux/util-linux) | `becdb8ba9f` | 9 | 0 |
| 6 other checkouts of the corpora above | | 13 | 1 |

| Repository | Commit | M4 files | Lexically invalid |
| --- | --- | ---: | ---: |
| [freebsd/freebsd-src](https://github.com/freebsd/freebsd-src) `contrib/sendmail/cf` | `53dd6b7760` | 151 | 1 |
| [autotools-mirror/m4](https://github.com/autotools-mirror/m4) | `e0e0f6eacc` | 46 | 4 |

Every valid file matches on every line. Of the 25 invalid files, 24 change their quotes with `changequote` or their comments with `changecom`, which the judge does not follow, and m4's tokenizer rejects the other. Fork `f26b942` differed on 364 of the 617 valid Autoconf files, on 6,134 lines: Autoconf declared no quotes and M4's quotes did not nest, so `#` lines, `dnl` remarks and blank lines in quoted macro bodies counted as comments and blanks. Fork `ea47c6a` reads m4's nesting quotes, and counts a `.m4` file as Autoconf when it uses Autoconf or m4sugar macros, as Linguist decides; 625 of the 637 Autoconf files are `.m4` files, which Tokei had counted as M4.

## Visual Basic

Visual Basic is the first language judged beyond the 39 of the [plan](../plans/archive/tokei-fork.md). Judge: Roslyn's parser, `Microsoft.CodeAnalysis.VisualBasic` 5.9.0, as for [C#](#c); `'`, `REM` and `'''` documentation comments are comments; strings, characters, dates and the text of interpolated strings are literals; XML literals are code; and text in inactive `#If` regions is parsed on its own.

| Repository | Commit | Visual Basic files | Lexically invalid |
| --- | --- | ---: | ---: |
| [dotnet/roslyn](https://github.com/dotnet/roslyn) | `dcfa99c3da` | 3,751 | 10 |
| [dotnet/docs](https://github.com/dotnet/docs) | `bcbe167f3f` | 1,686 | 1 |
| [dotnet/samples](https://github.com/dotnet/samples) | `ab42c8a425` | 469 | 0 |
| [icsharpcode/CodeConverter](https://github.com/icsharpcode/CodeConverter) | `a166046e1d` | 70 | 0 |

Every valid file matches on every line. Eight invalid files are Roslyn SDK templates whose names are `$placeholders$`, and the parser rejects the others, two Roslyn test sources and a documentation snippet. Fork `85e6ed1` differed on 317 files, on 12,301 lines. Visual Basic was declared with escaping `"…"` strings, so `"C:\"` kept its string open; `REM` opened no comment; the text of XML literals, in which Roslyn's tests embed whole programs inside CDATA sections, was read as Visual Basic, so its apostrophes started comments; and a backslash, the integer division operator, escaped the quote after it. Fork `b11aa64` declares `"…"` strings whose only escape is `""`, `REM` in any case where a token starts, as Roslyn's `PeekStartComment` reads it, XML literals where an expression may start, read to their end as text, and the backslash as an operator ([Visual Basic lexical grammar](https://learn.microsoft.com/en-us/dotnet/visual-basic/reference/language-specification/lexical-grammar)). On the Visual Basic files of these repositories it takes 0.62 times upstream's time.

## F#

Judge: the F# compiler's tokenizer, `FSharp.Compiler.Service` 43.12.403, with errors from its parser reading each file as a program without strict indentation; comment tokens are comments, joined over the blank lines of a block comment, string tokens are literals, and text in inactive `#if` regions is tokenized on its own. Version 43.9.300 still read the ML compatibility markers `(*F#` and `(*IF-FSHARP` as markers, which the compiler dropped in [dotnet/fsharp#19143](https://github.com/dotnet/fsharp/pull/19143) in January 2026.

| Repository | Commit | F# files | Lexically invalid |
| --- | --- | ---: | ---: |
| [dotnet/fsharp](https://github.com/dotnet/fsharp) | `fbc9be3aa8` | 6,796 | 757 |
| [fsprojects/fantomas](https://github.com/fsprojects/fantomas) | `e88da5853a` | 6,165 | 98 |
| [fable-compiler/Fable](https://github.com/fable-compiler/Fable) | `7319a994a9` | 1,032 | 1 |
| [ionide/FsAutoComplete](https://github.com/ionide/FsAutoComplete) | `4c33a2a722` | 386 | 3 |
| [fsprojects/Paket](https://github.com/fsprojects/Paket) | `af2ea6c09d` | 286 | 0 |
| [fsprojects/FSharpPlus](https://github.com/fsprojects/FSharpPlus) | `b5a650a483` | 217 | 1 |
| [giraffe-fsharp/Giraffe](https://github.com/giraffe-fsharp/Giraffe) | `9826d17fd6` | 49 | 0 |

Every valid file but two matches on every line; the invalid files are mostly the compiler's and Fantomas's tests of code the parser must reject. The two that differ, on 8 lines, are the compiler's conformance tests `InComment01.fs` and `InStringLiteral03.fs`, which leave a comment or a string open inside an `#if NOTDEFINED` region; the compiler skips such a region without reading it, while a counter that evaluates no preprocessor symbols reads it, as it must wherever a symbol is defined. Fork `9043f73` differed on 18 files, on 1,651 lines: a `"` inside a ``` ``…`` ``` identifier or an `@$"…"` string opened an escaping string, a `#!` first line was code, and inside block comments `(*)` opened a nested comment and a `*)` inside a string closed the comment. Fork `8ac5216` declares ``` `` ``` and `@$"` as verbatim quotes and the shebang, skips code tokens inside comments, and reads strings and character literals inside block comments as the lexer's `comment` rule does (src/Compiler/lex.fsl at `fbc9be3aa8`). On the F# files of dotnet/fsharp it takes 0.60 times upstream's time.

## Julia

Judge: JuliaSyntax, the tokenizer and parser in Julia's `Base` (measured with Julia 1.13.3); comments are comments, the delimiters and text of strings, commands and characters are literals, and string macro prefixes and `$` interpolations are code with their own literals.

| Repository | Commit | Julia files | Lexically invalid |
| --- | --- | ---: | ---: |
| [SciML/OrdinaryDiffEq.jl](https://github.com/SciML/OrdinaryDiffEq.jl) | `766596bc06` | 1,175 | 0 |
| [JuliaLang/julia](https://github.com/JuliaLang/julia) | `3a0616961a` | 948 | 79 |
| [MakieOrg/Makie.jl](https://github.com/MakieOrg/Makie.jl) | `21e2fcbecc` | 370 | 0 |
| [JuliaLang/Pkg.jl](https://github.com/JuliaLang/Pkg.jl) | `21e9b9b147` | 234 | 0 |
| [jump-dev/JuMP.jl](https://github.com/jump-dev/JuMP.jl) | `af70092219` | 163 | 0 |
| [JuliaPlots/Plots.jl](https://github.com/JuliaPlots/Plots.jl) | `115e28f23b` | 146 | 0 |
| [FluxML/Flux.jl](https://github.com/FluxML/Flux.jl) | `3b93324564` | 127 | 0 |
| [JuliaData/DataFrames.jl](https://github.com/JuliaData/DataFrames.jl) | `e867c34551` | 73 | 0 |

Every valid file matches on every line. The invalid files are sources of Julia's development branch that use syntax newer than 1.13, such as the wrapping operators `+%` and `-%`. Fork `9043f73` differed on 24 files, on 1,846 lines: Julia was declared without character literals, so `'"'` opened a string, and without commands, raw string macros or `$(…)` holes. Fork `92d542b` declares character literals that open only where no operand ends, as JuliaSyntax's `lex_prime` tells a character from the adjoint `x'`, backtick commands, `$(…)` holes in strings and commands, and a `"` directly after an identifier as a raw string without holes (src/julia/tokenize.jl at `3a0616961a`). On the Julia files of JuliaLang/julia it takes 0.60 times upstream's time.

## Erlang

Judge: `erl_scan`, the scanner the compiler's preprocessor runs (measured with OTP 29.1.2); comments are comments, and strings, triple-quoted strings, sigils, `$` character literals and quoted atoms are literals.

| Repository | Commit | Erlang files | Lexically invalid |
| --- | --- | ---: | ---: |
| [erlang/otp](https://github.com/erlang/otp) | `4ba5448abb` | 4,133 | 3 |
| [emqx/emqx](https://github.com/emqx/emqx) | `50eb8e5ae2` | 2,466 | 0 |
| [rabbitmq/rabbitmq-server](https://github.com/rabbitmq/rabbitmq-server) | `d38028c9c3` | 1,392 | 0 |
| [processone/ejabberd](https://github.com/processone/ejabberd) | `846822b00a` | 398 | 0 |
| [erlang/rebar3](https://github.com/erlang/rebar3) | `15f488ef1c` | 271 | 0 |
| [ninenines/cowboy](https://github.com/ninenines/cowboy) | `6475600966` | 189 | 0 |

Every valid file matches on every line. The three invalid files are snippets that wx's API generator inserts into the modules it writes. Not judged are eight OTP files encoded in Latin-1, which judgecmp cannot compare, and one whose directory, named with an emoji, Docker's file sharing does not show to the container. Fork `9043f73` differed on 847 files, on 31,006 lines, as Erlang was declared without strings, so a `%` in a string, such as a format string's `50%`, started a comment. Fork `dcbd729` declares `"…"` strings and `'…'` quoted atoms with escapes, `$` character literals such as `$"`, triple-quoted strings that are verbatim and close only at their quotes starting a line ([EEP 64](https://www.erlang.org/eeps/eep-0064)), and sigils as `erl_scan` reads them ([EEP 66](https://www.erlang.org/eeps/eep-0066)); the speed gate gives upstream the same quotes. On the Erlang files of erlang/otp it takes 0.61 times upstream's time.

## OCaml

Judge: the compiler's lexer, compiler-libs' `Lexer` (measured with OCaml 5.5.1), without documentation-comment handling; comments, documentation comments included, are comments, and strings, quoted strings such as `{id|…|id}` and `{%ext|…|}`, and character literals are literals.

| Repository | Commit | OCaml files | Lexically invalid |
| --- | --- | ---: | ---: |
| [ocaml/ocaml](https://github.com/ocaml/ocaml) | `a76fb1de94` | 3,085 | 5 |
| [ocaml/dune](https://github.com/ocaml/dune) | `89fc7a1daa` | 2,833 | 0 |
| [ocaml/merlin](https://github.com/ocaml/merlin) | `58f43bff7d` | 1,824 | 0 |
| [semgrep/semgrep](https://github.com/semgrep/semgrep) | `f472beb2b5` | 1,603 | 0 |
| [rocq-prover/rocq](https://github.com/rocq-prover/rocq) | `42343d9bba` | 1,325 | 0 |
| [janestreet/core](https://github.com/janestreet/core) | `dbceee5dda` | 591 | 0 |
| [mirage/irmin](https://github.com/mirage/irmin) | `7a09a06fff` | 539 | 0 |
| [janestreet/base](https://github.com/janestreet/base) | `28d466710a` | 291 | 0 |
| [ocaml/opam](https://github.com/ocaml/opam) | `ac518a2fa8` | 276 | 0 |

Every valid file matches on every line; the invalid files are the compiler's tests of input its lexer must reject. Fork `3343fdb` differed on 3 files, on 11 lines, as inside a block comment it did not read strings, quoted strings and character literals as the lexer's `comment` rule does, so a `*)` or `(*` within one closed or opened a comment. Fork `3a5b9a4` reads them there by that rule (parsing/lexer.mll at `a76fb1de94`). On the OCaml files of ocaml/ocaml it takes 0.84 times upstream's time.

## Fortran

Judge: fparser's reader (fparser 0.2.5), the one fparser's Fortran 2008 parser and PSyclone read sources with, as no Fortran compiler exposes its tokens; comments are comments, and strings need no ranges, as none holds a blank line. The judge sets each file's source form itself: free for `.f90`, `.f95`, `.f03` and `.f08`, and otherwise by fparser's content rule over lines other than preprocessor directives and `*` comment lines, since fparser reads a `#if` line, gone before the compiler reads the source, as free form, and LAPACK's `*` comment lines that end in `&` too.

| Repository | Commit | Fortran files | Lexically invalid |
| --- | --- | ---: | ---: |
| [Reference-LAPACK/lapack](https://github.com/Reference-LAPACK/lapack) | `472b3a0fcb` | 3,683 | 0 |
| [QEF/q-e](https://github.com/QEF/q-e) | `61569eb480` | 1,878 | 0 |
| [wrf-model/WRF](https://github.com/wrf-model/WRF) | `06d4240ae9` | 1,763 | 2 |
| [cp2k/cp2k](https://github.com/cp2k/cp2k) | `d077e66fe0` | 1,515 | 0 |
| [fortran-lang/stdlib](https://github.com/fortran-lang/stdlib) | `1903780b7c` | 421 | 0 |
| [fortran-lang/fpm](https://github.com/fortran-lang/fpm) | `d51563ea0c` | 221 | 0 |

Every valid file matches on every line. The two invalid files are makefiles that WRF's KPP names `Makefile.f` and `Makefile.f90`; not judged are one stdlib and two WRF files that are not UTF-8. Fork `3343fdb` differed on 7,568 files, on 429,004 lines: upstream read Fortran strings with backslash escapes instead of doubled quotes and matched comment markers on untrimmed lines, so an indented `!` line was code, and it read every `.F` file in fixed form, so a `C` opening a line of free-form code, such as cp2k's `CONTAINS`, started a comment. Fork `fa4eb14` reads `"…"` and `'…'` strings with doubled quotes, `!` comments wherever they start, and each file with a legacy extension in fixed source form unless its content is free form by the judge's rule, with `C`, `c` and `*` comments in the first column and an `!` in the sixth column marking a continuation (Fortran 2018, J3/18-007r1, §6.3.3.2 and §6.3.3.3). Since both apply one rule, the form was checked apart from it, against each file's trailing `&` and sixth-column continuation marks: no file read in fixed form shows more free-form continuations than fixed-form ones, and the free-form files that show more fixed-form marks, in WRF, carry both kinds and are compiled in free form, as WRF's build preprocesses each `.F` file into a `.f90` file before compiling it (`arch/postamble` at `06d4240ae9`, rule `.F.o`). On the Fortran files of LAPACK it takes 0.81 times upstream's time.

## Pascal

Judge: Free Pascal's own scanner, `TPascalScanner` of fcl-passrc (`packages/fcl-passrc/src/pscanner.pp` at FPCSource `fb4143a1ef`), built with Free Pascal 3.2.2, without include files or macro expansion and with the text of inactive `{$if}` branches read too; it reads each file in FPC mode, and in Delphi mode when FPC mode fails. Comments are comments, and strings are literals.

| Repository | Commit | Pascal files | Lexically invalid |
| --- | --- | ---: | ---: |
| [fpc/FPCSource](https://github.com/fpc/FPCSource) | `fb4143a1ef` | 4,608 | 8 |
| [fpc/Lazarus](https://github.com/fpc/Lazarus) | `709a56c0c0` | 3,210 | 4 |
| [castle-engine/castle-engine](https://github.com/castle-engine/castle-engine) | `af6e5cb725` | 1,215 | 0 |
| [doublecmd/doublecmd](https://github.com/doublecmd/doublecmd) | `1b6c0090b9` | 839 | 1 |
| [danieleteti/delphimvcframework](https://github.com/danieleteti/delphimvcframework) | `614dce2426` | 757 | 0 |
| [synopse/mORMot2](https://github.com/synopse/mORMot2) | `0186cd0223` | 461 | 0 |
| [HeidiSQL/HeidiSQL](https://github.com/HeidiSQL/HeidiSQL) | `2b4ec13c19` | 151 | 0 |

Every valid file matches on every line; 387 further files are not UTF-8 and were not judged. Fork `c18e585` differed on 507 files, on 169,238 lines: upstream read Pascal strings with backslash escapes, so `'C:\'` left a string open, and nested every comment kind in every other. Fork `5a72cb6` reads `'…'` strings with `''` as their only escape, nests `{…}` only in `{…}` and `(*…*)` only in `(*…*)`, as Free Pascal's scanner does, and reads a file whose comments would be left open by nesting without it, as a file written for Delphi, which does not nest, needs. The same change makes every declared nested comment nest only in its own kind, which D's specification requires of `/+ … +/` inside `/* … */` ([D Language Specification, Lexical, Comments](https://dlang.org/spec/lex.html#comment)); D was checked against that rule with a fixture, not measured on repositories. On the Pascal files of Lazarus and of FPCSource it takes 0.56 and 0.51 times upstream's time.

## Ada

Judge: a reference judge by the Ada reference manual's lexical rules (RM 2.5–2.7 and J.2), telling an apostrophe's tick from a character literal by the token before it as GNAT's scanner does (`gcc/ada/scng.adb` at `3f35c3d2d1`). Comments are comments, and strings and character literals are literals.

| Repository | Commit | Ada files | Lexically invalid |
| --- | --- | ---: | ---: |
| [AdaCore/spark2014](https://github.com/AdaCore/spark2014) | `b121a46798` | 13,346 | 2 |
| [AdaCore/gnatstudio](https://github.com/AdaCore/gnatstudio) | `54f541177a` | 2,332 | 6 |
| [AdaCore/ada_language_server](https://github.com/AdaCore/ada_language_server) | `ff71c07c77` | 1,745 | 7 |
| [AdaCore/Ada_Drivers_Library](https://github.com/AdaCore/Ada_Drivers_Library) | `d331ed4f6d` | 1,024 | 0 |
| [AdaCore/gnatcoll-core](https://github.com/AdaCore/gnatcoll-core) | `85615cb568` | 496 | 0 |
| [alire-project/alire](https://github.com/alire-project/alire) | `144e4f8abc` | 475 | 0 |
| [gcc-mirror/gcc](https://github.com/gcc-mirror/gcc) Ada files | `3f35c3d2d1` | 473 | 0 |

Fork `c18e585` matches every valid file on every line, as no Ada string or character literal can span a line; 22 files are not UTF-8 and were not judged.

## GraphQL

Judge: graphql-js 17.0.2, the reference implementation, whose lexer keeps `#` comments in its token chain; comments are comments, and strings and block strings are literals. Only the lexer's errors make a file invalid, so a schema fragment the parser rejects is judged all the same.

| Repository | Commit | GraphQL files | Lexically invalid |
| --- | --- | ---: | ---: |
| [hasura/graphql-engine](https://github.com/hasura/graphql-engine) | `61a09f382e` | 407 | 12 |
| [ardatan/graphql-tools](https://github.com/ardatan/graphql-tools) | `03f8963fe2` | 255 | 0 |
| [wundergraph/cosmo](https://github.com/wundergraph/cosmo) | `9f06b87ad4` | 224 | 4 |
| [aws-amplify/amplify-category-api](https://github.com/aws-amplify/amplify-category-api) | `74e5be84fd` | 99 | 0 |
| [dotansimha/graphql-code-generator](https://github.com/dotansimha/graphql-code-generator) | `0c19df5d61` | 73 | 0 |
| [neo4j/graphql](https://github.com/neo4j/graphql) | `887cbc6c28` | 19 | 0 |
| [gatsbyjs/gatsby](https://github.com/gatsbyjs/gatsby) | `516700a1dc` | 4 | 0 |
| [graphql/graphql-js](https://github.com/graphql/graphql-js) | `805fadd286` | 2 | 0 |
| [apollographql/apollo-client](https://github.com/apollographql/apollo-client) | `9462324835` | 2 | 0 |
| [octokit/graphql-schema](https://github.com/octokit/graphql-schema) | `82ff2d4780` | 1 | 0 |
| [saleor/saleor](https://github.com/saleor/saleor) | `cc4aea76e3` | 1 | 0 |

Fork `c18e585` matches every valid file on every line.

## Solidity

Judge: solc 0.8.30's parser, stopping after parsing, whose syntax tree gives each string, hex and unicode literal, in Solidity and in inline assembly, with its byte range; comments are the `//` and `/* */` that open outside them. Version pragmas are blanked out first, as solc rejects a file that asks for another compiler version.

| Repository | Commit | Solidity files | Lexically invalid |
| --- | --- | ---: | ---: |
| [ethereum/solidity](https://github.com/ethereum/solidity) | `765afa0602` | 7,361 | 585 |
| [OpenZeppelin/openzeppelin-contracts](https://github.com/OpenZeppelin/openzeppelin-contracts) | `b7f748eae2` | 430 | 0 |
| [Uniswap/v4-core](https://github.com/Uniswap/v4-core) | `bea7f6d1c1` | 132 | 0 |
| [aave/aave-v3-core](https://github.com/aave/aave-v3-core) | `782f519170` | 132 | 0 |
| [Uniswap/v3-core](https://github.com/Uniswap/v3-core) | `5ea68a77cb` | 69 | 0 |
| [transmissions11/solmate](https://github.com/transmissions11/solmate) | `89365b880c` | 59 | 0 |
| [foundry-rs/forge-std](https://github.com/foundry-rs/forge-std) | `bd8c870873` | 53 | 0 |
| [smartcontractkit/chainlink](https://github.com/smartcontractkit/chainlink) | `8c20b1c4d9` | 2 | 0 |

Every valid file matches on every line; the invalid files are all in the compiler's own test suite. Fork `c18e585` differed on one file, on one line, as it read no single-quoted strings, which Solidity allows beside double-quoted ones ([Solidity documentation, String Literals and Types](https://docs.soliditylang.org/en/latest/types.html#string-literals-and-types)); fork `55e3ec7` reads them. On the Solidity files of all eight repositories it takes 0.73 times upstream's time.

## Nim

Judge: the Nim 2.2.14 compiler's own lexer, `compiler/lexer.nim`, built with `-d:nimpretty` so that it records each token's byte range. Strings, raw, triple-quoted and generalized strings and character literals are literals; as the lexer merges neighbouring comments and the whitespace between them into one token, the comments are found between the literals by its own rules.

| Repository | Commit | Nim files | Lexically invalid |
| --- | --- | ---: | ---: |
| [nim-lang/Nim](https://github.com/nim-lang/Nim) | `9a4192cc4c` | 4,319 | 10 |
| [status-im/nimbus-eth2](https://github.com/status-im/nimbus-eth2) | `0abc0dd0fc` | 379 | 0 |
| [nim-lang/nimble](https://github.com/nim-lang/nimble) | `99ac3534b0` | 291 | 0 |
| [treeform/pixie](https://github.com/treeform/pixie) | `30bb510877` | 110 | 0 |
| [status-im/nim-chronos](https://github.com/status-im/nim-chronos) | `df2fd65f9a` | 99 | 0 |
| [karaxnim/karax](https://github.com/karaxnim/karax) | `fa5fdef42b` | 65 | 0 |

Every valid file matches on every line; the invalid files are compiler and nimsuggest tests, most of them of input the lexer must reject, and a source-code filter, and one file is not UTF-8. Fork `c18e585` differed on 260 files, on 4,862 lines, as it read neither `#[ … ]#` comments nor character literals, so a `'"'` opened a string, and read raw strings with escapes. Fork `131696b` reads `#[ … ]#` and `##[ … ]##` comments, each nesting on its own opener as the lexer's `skipMultiLineComment` does, raw and generalized raw strings without escapes and with `""` for a quote, triple-quoted strings closing at the end of a run of quotes, and character literals ([Nim Manual, Lexical Analysis](https://nim-lang.org/docs/manual.html#lexical-analysis)). On the Nim files of nim-lang/Nim it takes 0.60 times upstream's time.

## Crystal

Judge: the Crystal 1.22.2 compiler's own parser, which drives its lexer, asking for string pieces and heredoc bodies as it goes; the lexer is reopened to record each comment that `skip_comment` passes over, each string piece and each string's whole span, and each heredoc line. In a macro body, where the lexer reads text as macro literals, a `#` outside a string is a comment to the end of its line, as in the code the macro expands to.

| Repository | Commit | Crystal files | Lexically invalid |
| --- | --- | ---: | ---: |
| [crystal-lang/crystal](https://github.com/crystal-lang/crystal) | `9400d45c42` | 2,427 | 0 |
| [athena-framework/athena](https://github.com/athena-framework/athena) | `c673b78bbf` | 1,358 | 0 |
| [mint-lang/mint](https://github.com/mint-lang/mint) | `7aed761732` | 754 | 0 |
| [luckyframework/lucky](https://github.com/luckyframework/lucky) | `ddd7318e29` | 251 | 0 |
| [amberframework/amber](https://github.com/amberframework/amber) | `b29f257d22` | 204 | 0 |
| [crystal-lang/shards](https://github.com/crystal-lang/shards) | `843c886f7a` | 98 | 0 |
| [kemalcr/kemal](https://github.com/kemalcr/kemal) | `3d847ee37f` | 79 | 0 |

Fork `c18e585` differed on 433 files, on 12,425 lines: it read `'` as a string quote, though Crystal's `'` opens only a character literal, and read neither heredocs, percent literals nor regular expression literals. Fork `cc13f4f` reads Crystal with Ruby's percent and regular expression rules, `<<-` heredocs closing on a line that holds only their word after any indentation (`check_heredoc_end` in `src/compiler/crystal/syntax/lexer.cr`), `#{…}` holes in strings and commands, character literals, and `` def ` `` as code rather than a command. Nine blank lines in four files still differ, each inside a multi-line string within a macro body: an `asm("…")` string in three files of `src/fiber/context` and a `%(…)` string in `spec/compiler/interpreter/primitives_spec.cr`. The macro lexer tracks such strings only in local state, so the judge cannot see that the lines lie in a literal; read by hand, each lies in one, so the fork's code count is right by the rule that a whitespace line in a literal is code. On the Crystal files of crystal-lang/crystal it takes 0.60 times upstream's time.

## Clojure

Judge: a reference judge by the Clojure reader's lexical rules (`src/jvm/clojure/lang/LispReader.java` at clojure `33fb064dee`): `;` and, at the start of a token, `#!` open comments to the end of the line; strings and `#"…"` regular expressions, in which a backslash takes the next character, are literals; a backslash outside them takes the next character as a character literal.

| Repository | Commit | Clojure files | ClojureScript files | ClojureC files | Lexically invalid |
| --- | --- | ---: | ---: | ---: | ---: |
| [metabase/metabase](https://github.com/metabase/metabase) | `829cac87a6` | 4,033 | 31 | 369 | 0 |
| [babashka/babashka](https://github.com/babashka/babashka) | `b11a48af0c` | 532 | 15 | 116 | 0 |
| [clj-kondo/clj-kondo](https://github.com/clj-kondo/clj-kondo) | `47dedc92a3` | 417 | 26 | 19 | 0 |
| [technomancy/leiningen](https://github.com/technomancy/leiningen) | `d35aef2988` | 189 | 0 | 2 | 1 |
| [clojure/clojure](https://github.com/clojure/clojure) | `33fb064dee` | 142 | 0 | 1 | 0 |
| [clojure/clojurescript](https://github.com/clojure/clojurescript) | `164d88d43d` | 88 | 178 | 30 | 0 |
| [ring-clojure/ring](https://github.com/ring-clojure/ring) | `6238fe0e2e` | 81 | 0 | 0 | 0 |

Every valid file matches on every line; the invalid file is a source file of a Leiningen test project that is meant to fail to compile, and one Clojure file is a symbolic link. Fork `c18e585` differed on 81 files, on one line each: babashka and planck shebangs, which the reader's dispatch table reads as comments wherever a token starts, `dispatchMacros['!']`. Fork `56b9d63` reads `#!` as a line comment. A backslash already escaped the next character outside literals, so `\"` and `\;` opened nothing before.

## COBOL

Judge: GnuCOBOL 3.2's preprocessor, `cobc -E`, which applies the reference format and leaves a line empty when it held no program text; continuation lines and compiler directives, which it also empties, are code. Each file starts in the form GnuCOBOL's development branch chooses by default (`ppopen_get_file` in `cobc/pplex.l` at gnucobol `eda8905e44`), and `>>SOURCE FORMAT` or `$SET SOURCEFORMAT` switches it; conditional compilation directives are blanked and `COPY` and `REPLACE` renamed first, so that every branch counts and no copybook is needed.

| Repository | Commit | COBOL files | Lexically invalid |
| --- | --- | ---: | ---: |
| [uwol/proleap-cobol-parser](https://github.com/uwol/proleap-cobol-parser) | `d1bfe75bdd` | 829 | 220 |
| [meyfa/CobolCraft](https://github.com/meyfa/CobolCraft) | `e8c420df7a` | 268 | 9 |
| [Apress/beg-cobol-for-programmers](https://github.com/Apress/beg-cobol-for-programmers) | `3d2656fadd` | 118 | 1 |
| [aws-samples/aws-mainframe-modernization-carddemo](https://github.com/aws-samples/aws-mainframe-modernization-carddemo) | `59cc6c2fd7` | 106 | 4 |
| [openmainframeproject/cobol-programming-course](https://github.com/openmainframeproject/cobol-programming-course) | `780b1c9624` | 32 | 3 |
| [Martinfx/Cobol](https://github.com/Martinfx/Cobol) | `f1bb3163d6` | 28 | 0 |
| [neopragma/cobol-unit-test](https://github.com/neopragma/cobol-unit-test) | `8e2c04fc8c` | 27 | 0 |
| [azac/cobol-on-wheelchair](https://github.com/azac/cobol-on-wheelchair) | `9eac2bcb0e` | 6 | 2 |
| [IBM/cobol-is-fun](https://github.com/IBM/cobol-is-fun) | `d9244e6292` | 2 | 0 |

Every valid file matches on every line. Of the invalid files, 213 are NIST CCVS85 test programs, whose column-7 markers are for the suite's own preprocessor; the others hold embedded SQL, which needs a precompiler, or are code-generation templates, copybook fragments or deliberately non-compliant parser inputs. Read as valid anyway, 231 of them match; the eight others differ on 52 lines that cobc's error recovery drops after an `EXEC SQL` error, which the fork reads as code. Two files are not UTF-8. Upstream's only rule, `*` as a line comment, misses the reference format's columns: fork `721d752` differed on 482 of the 1,175 valid files, on 18,124 lines. Fork `2834e6c` reads each line as GnuCOBOL's reference format does, and fork `901313f` reads code words and the starting form without copies; on these repositories it takes 0.69 times upstream's time. No SQL comment occurs inside an `EXEC SQL` block in the corpus, so the fork does not read them.

## GNU Style Assembly

Judge: LLVM 19.1.7's assembler lexer, the one `llvm-mc --as-lex` runs, for the processor the nearest directory or file name word names, such as `arch/arm64/` or `aesv8-armx-apple.S`; a `.S` file is first lexed by clang 19's lexer in assembler mode, whose comments are comments, whose directives are code and whose splices join lines. `# 4` is no line marker in an assembly file (clang `Preprocessor::HandleDirective`), and a lone `#` is the null directive. Files for processors LLVM does not assemble, such as m68k, Alpha, SuperH and PA-RISC, 593 of them, and Go's assembly files are left out. Go's are judged with the tokenizer `cmd/asm` configures, `text/scanner` in the mode of `NewTokenizer` in `cmd/asm/internal/lex/tokenizer.go`.

| Repository | Commit | GNU-style files | Go assembly files |
| --- | --- | ---: | ---: |
| [bminor/glibc](https://github.com/bminor/glibc) | `04e750e75b` | 1,958 | 0 |
| [torvalds/linux](https://github.com/torvalds/linux) | `36aad21d50` | 1,025 | 0 |
| [FFmpeg/FFmpeg](https://github.com/FFmpeg/FFmpeg) | `df2926615e` | 236 | 0 |
| [google/boringssl](https://github.com/google/boringssl) | `ed9209cff8` | 135 | 0 |
| [videolan/dav1d](https://github.com/videolan/dav1d) | `4e3ab21fbb` | 66 | 0 |
| [mirror/x264](https://github.com/mirror/x264) | `c24e06c2e1` | 39 | 0 |
| [libjpeg-turbo/libjpeg-turbo](https://github.com/libjpeg-turbo/libjpeg-turbo) | `fa224ebee4` | 1 | 0 |
| [golang/go](https://github.com/golang/go) | `67ced03108` | 0 | 636 |

No file is lexically invalid. Upstream read only `//` and `/* */`, so the `#`, `@` and `!` comment lines of x86, Arm and SPARC files were code: fork `901313f` differed on 407 of 3,447 GNU-style files, on 8,302 lines, and on 16 of 636 Go files, on 128 lines. Fork `203d357` reads `#`, `@` and `!` as comments at the start of a line, except a `#` that starts a preprocessor directive, in its own case, so `# If nonzero` stays a comment, and except on a line a splice continues; `;`, a comment only in Apple's AArch64 syntax, and m68k's `|` stay code, since lines of Linux AArch64 files and of C expressions start with them. Three lines still differ. Two are a lone `#` in plain `.s` files of BoringSSL's delocate test data, an assembler comment, as no preprocessor reads `.s`; the fork lowercases extensions and reads every file as `.S`. One follows a `//` comment ending in a backslash in `src/internal/bytealg/index_ppc64x.s`, which Go's assembler does not splice. An unbalanced quote in a mid-line `#` comment, which both the preprocessor and the assembler end with the line, occurs in no file, so the fork does not read it.

## Assembly

Judge: a reference judge by NASM's lexical rules, from the NASM 2.16 manual (sections 3.1 and 3.4.2) and probes of `nasm -E` 2.16.03: `;` comments, verbatim `'…'` and `"…"` strings, backquote strings with escapes, a string ending with its line when it does not close, and a backslash directly before the newline joining the next line, also in a comment; and MASM's `COMMENT` blocks (MASM 6.1 Programmer's Guide). On the 5,340 files `nasm -E` accepts, every one of the 524,938 lines it prints agrees with the judge, and the 3,674 lines it consumes are directives and macro invocations, code for the judge. Eleven AMD GPU shader sources in Linux, with `//` comments, are another syntax and are left out.

| Repository | Commit | `.asm` files |
| --- | --- | ---: |
| [netwide-assembler/nasm](https://github.com/netwide-assembler/nasm) | `fbdc88565c` | 5,337 |
| [microsoft/MS-DOS](https://github.com/microsoft/MS-DOS) | `2d04cacc53` | 629 |
| [FFmpeg/FFmpeg](https://github.com/FFmpeg/FFmpeg) | `df2926615e` | 189 |
| [libjpeg-turbo/libjpeg-turbo](https://github.com/libjpeg-turbo/libjpeg-turbo) | `fa224ebee4` | 91 |
| [tgrysztar/fasmg](https://github.com/tgrysztar/fasmg) | `905f6791f6` | 62 |
| [videolan/dav1d](https://github.com/videolan/dav1d) | `4e3ab21fbb` | 49 |
| [google/boringssl](https://github.com/google/boringssl) | `ed9209cff8` | 34 |
| [mirror/x264](https://github.com/mirror/x264) | `c24e06c2e1` | 20 |

Upstream let a backslash escape the quote in `"\"`, which NASM and MASM read verbatim, and kept an unclosed quote, as in `title DOS entry to the KWC's 'C' programs`, open into the following lines: fork `901313f` differed on 79 of 6,411 files, on 9,412 lines. Fork `347319e` closes strings as NASM does and reads MASM's `COMMENT` blocks. Fifteen lines in 11 MS-DOS files still differ, each after a comment ending in a backslash, which NASM continues onto the next line; the MASM releases that built MS-DOS had no line continuation, and no NASM file in the corpus ends a comment so, so the fork does not continue them. On the assembly repositories, GNU-style and NASM together, the fork takes 0.95 times upstream's time. Sixty-five MS-DOS files and eight of NASM's test files are not UTF-8.

## Bitbake

BitBake has no tokenizer to call. Judge: BitBake 2.16's reading rules, ported from `lib/bb/parse/parse_py/BBHandler.py` and `ConfHandler.py` at bitbake `7412853430`, which `lib/bb/parse/ast.py` also applies to `.bbappend` files: a statement, its lines joined by trailing backslashes, whose first character is `#` is a comment; an assignment's quoted value, which must end its statement, is a literal; and the bodies of `name() {` and `def` functions are read by the [Shell](#shell) and [Python](#python) judges, a braced function as Python when `python` comes first or, as `MethodNode.eval` in `ast.py` makes it, when it has no name ([BitBake User Manual, Syntax and Operators](https://docs.yoctoproject.org/bitbake/bitbake-user-manual/bitbake-user-manual-metadata.html)). A statement BitBake does not know makes a file invalid. `.inc` files count when Tokei's content rule claims them for BitBake.

| Repository | Commit | BitBake files | Lexically invalid |
| --- | --- | ---: | ---: |
| [openembedded/meta-openembedded](https://github.com/openembedded/meta-openembedded) | `eaa6d03a72` | 2,646 | 0 |
| [openembedded/openembedded-core](https://github.com/openembedded/openembedded-core) | `5ffd9e0554` | 1,639 | 1 |
| [lgirdk/meta-virtualization](https://github.com/lgirdk/meta-virtualization), a mirror of the Yocto Project's layer | `b12fb49843` | 228 | 0 |
| [agherzan/meta-raspberrypi](https://github.com/agherzan/meta-raspberrypi) | `96d0b6dadf` | 93 | 0 |

Every valid file matches on every line, 240,608 lines in all. The invalid file, `kernel.bbclass`, appends a body to `do_create_spdx`, which another class declares as a Python function, so the file alone does not tell its interpreter; the judge reads it as shell, which rejects it. Fork `cb12c3b` differed on 45 files, on 217 lines: every line was read with BitBake's top-level `#` and quote rules, so heredocs and nested quoting in shell functions, and triple-quoted strings holding apostrophes in Python functions, turned comments into code and code into comments. Fork `f50b63c` reads BitBake's statements as BBHandler does and each function body as its interpreter does; on these repositories it takes 0.32 times upstream's time.

## Just

Judge: just 1.59.0's lexer, `src/lexer.rs`, vendored verbatim since just exports none, with just's recipe runner (`Recipe::run_linewise` and `Recipe::run_script` in `src/recipe.rs`; [Just Programming Language manual](https://just.systems/man/en/)). Outside recipe bodies, comment tokens are comments and strings, backticks and the text of format strings are literals. just hands each body line to the shell after its `-`, `@` and `?` sigils, so a line whose command starts with `#` is a comment, and so is the line a comment's trailing backslash continues. A script recipe runs its body as one script, without the recipe's indentation, for the command of its `[script]` attribute or the `script-interpreter` setting, or for the interpreter its `#!` line names; the [Shell](#shell) or [Python](#python) judge reads it, with each `{{…}}` interpolation replaced by `x` characters, and a script for another interpreter makes the file invalid.

| Repository | Commit | Justfiles | Lexically invalid |
| --- | --- | ---: | ---: |
| [ublue-os/bazzite](https://github.com/ublue-os/bazzite) | `3a46cd99bc` | 27 | 0 |
| [casey/just](https://github.com/casey/just) | `2638e9c094` | 8 | 3 |
| [pop-os/cosmic-files](https://github.com/pop-os/cosmic-files) | `09bdd8251d` | 2 | 0 |
| [pop-os/cosmic-comp](https://github.com/pop-os/cosmic-comp) | `eeca8b09cb` | 2 | 0 |
| [ublue-os/aurora](https://github.com/ublue-os/aurora) | `71dbbd7716` | 1 | 0 |
| [ublue-os/bluefin](https://github.com/ublue-os/bluefin) | `af07fd46dc` | 1 | 0 |
| 1,889 repositories that GitHub code search returned for justfile names on 2026-10-31, each file at the commit the search reported, [listed with their paths](tokei-fork-step-6-just-search.tsv) | | 2,230 | 78 |

The 2,271 files hold 2,100 distinct contents. Every valid file matches on every line, 170,241 lines in all. Of the 81 invalid files, 59 run a script recipe with an interpreter no judge reads, such as PowerShell, `cmd.exe`, Bun, Node.js, Perl or Nushell; 17 are not files just's lexer accepts, such as cookiecutter and Liquid templates; and 5 hold a script the Shell judge rejects. Fork `cb12c3b` differed on 56 files, on 215 lines: Just declared no strings, so `#` lines in strings that span lines counted as comments, and script recipes were read line by line, so heredocs and quoting that span lines turned code into comments and blank lines. Fork `8dfaf47` reads strings as just's lexer does, the lines of a body as just runs them, and a script recipe's body as the language of its `[script]` command, `script-interpreter` setting or `#!` line. Fork `a069406` counts `.justfile` files, which just reads as it reads `justfile` (`JUSTFILE_NAMES` in `src/search.rs`) and Tokei did not count; 768 of the files above are named so. Upstream declares no strings for Just, so the speed gate gives upstream Just's quotes, as for Haskell and JSON; on these files the fork takes 0.65 times its time.

## HCL

Judge: HCL v2.25.0's `hclsyntax.LexConfig`, the scanner HashiCorp's parser and Terraform run ([HCL native syntax specification](https://github.com/hashicorp/hcl/blob/v2.25.0/hclsyntax/spec.md)): comment tokens are comments, and quoted strings and heredocs, delimiters and text, are literals, while their `${…}` and `%{…}` holes are code.

| Repository | Commit | HCL files | Lexically invalid |
| --- | --- | ---: | ---: |
| [hashicorp/terraform-provider-aws](https://github.com/hashicorp/terraform-provider-aws) | `6979502b51` | 5,308 | 0 |
| [gruntwork-io/terragrunt](https://github.com/gruntwork-io/terragrunt) | `6987602270` | 3,511 | 2 |
| [GoogleCloudPlatform/cloud-foundation-fabric](https://github.com/GoogleCloudPlatform/cloud-foundation-fabric) | `9e616b751f` | 1,055 | 0 |
| [hashicorp/nomad](https://github.com/hashicorp/nomad) | `64cdf9f9a0` | 323 | 0 |
| [terraform-aws-modules/terraform-aws-rds](https://github.com/terraform-aws-modules/terraform-aws-rds) | `175da04342` | 104 | 0 |
| [terraform-aws-modules/terraform-aws-eks](https://github.com/terraform-aws-modules/terraform-aws-eks) | `91b8a8d944` | 90 | 0 |
| [terraform-aws-modules/terraform-aws-vpc](https://github.com/terraform-aws-modules/terraform-aws-vpc) | `b3abd6df2e` | 77 | 0 |

Every valid file matches on every line, 474,083 lines in all; the two invalid files are Terragrunt fixtures holding invalid characters on purpose. Release `15.0.0+locmetry.38`, fork `e483fc8`, differed on 66 files, on 232 lines: HCL declared no heredocs or holes, so `#` and blank lines in `<<EOF` bodies counted as comments and blanks, and a `/*` inside a string nested in `${…}` opened a comment that swallowed the following lines. Fork `1281560` reads `<<` and `<<-` heredocs and holes, with `$${` and `%%{` as literal text; on these repositories it takes 0.66 times upstream's time.

## Nix

Nix exports no tokenizer. Judge: Nix 2.35.4's lexer rules, ported from `src/libexpr/lexer.l` at nix `21286d6058` with flex's longest match ([Nix language manual, String literals](https://nix.dev/manual/nix/2.35/language/string-literals)): `#` and `/* */` comments, and `"…"` and `''…''` strings, whose `${…}` holes are code, with the escapes `''$`, `'''` and `''\` of indented strings.

| Repository | Commit | Nix files | Lexically invalid |
| --- | --- | ---: | ---: |
| [NixOS/nixpkgs](https://github.com/NixOS/nixpkgs), `lib`, `nixos/modules/services`, `pkgs/development/python-modules`, `pkgs/build-support`, `pkgs/top-level` and `pkgs/stdenv` | `42bb121685` | 12,642 | 0 |
| [nix-community/home-manager](https://github.com/nix-community/home-manager) | `095fef2f26` | 1,393 | 0 |
| [NixOS/nix](https://github.com/NixOS/nix), release 2.35.4 | `21286d6058` | 710 | 3 |
| [nix-darwin/nix-darwin](https://github.com/nix-darwin/nix-darwin) | `ff0e21a414` | 216 | 0 |
| [nix-community/disko](https://github.com/nix-community/disko) | `cedd626e11` | 127 | 0 |

The three invalid files are Nix's parse-failure tests. Release `15.0.0+locmetry.38`, fork `e483fc8`, differed on 3,244 files, on 64,335 lines: Nix knew no `''…''` strings, so `#` lines and blank lines in them counted as comments and blanks. Fork `559d1a4` reads indented strings with their escapes, such as `'''`, which start with the closer and so are checked before it, holes in both kinds of string, and `'` inside identifiers, so `x''` opens no string. Two lines in Nix's own tests still differ, `eval-fail-eol-2.nix` and `parse-okay-crlf.nix`, each after a lone carriage return, which Nix ends a line with and Tokei does not. On these repositories the fork takes 0.59 times upstream's time.

## Jsonnet

Judge: go-jsonnet v0.22.0's lexer, `internal/parser/lexer.go`, vendored with only its errors package's import path changed, since go-jsonnet keeps it internal ([Jsonnet specification, Lexing](https://jsonnet.org/ref/spec.html#lexing)): `#`, `//` and `/* */` comments, and `"…"`, `'…'`, verbatim `@"…"` and `@'…'` strings, which double a quote to escape it, and `|||` text blocks.

| Repository | Commit | Jsonnet files | Lexically invalid |
| --- | --- | ---: | ---: |
| [grafana/grafonnet](https://github.com/grafana/grafonnet) | `8de2f3c83a` | 1,324 | 0 |
| [grafana/jsonnet-libs](https://github.com/grafana/jsonnet-libs) | `5d1c9cccf1` | 1,183 | 0 |
| [google/go-jsonnet](https://github.com/google/go-jsonnet), v0.22.0 | `567b61ac4a` | 811 | 0 |
| [google/jsonnet](https://github.com/google/jsonnet) | `2ef32abf3d` | 301 | 10 |
| [grafana/mimir](https://github.com/grafana/mimir) | `1870541584` | 224 | 0 |
| [prometheus-operator/kube-prometheus](https://github.com/prometheus-operator/kube-prometheus) | `f71fb6c77d` | 108 | 0 |

Every valid file matches on every line, 558,543 lines in all; the ten invalid files are the parse-error tests of Jsonnet's test suite. Release `15.0.0+locmetry.38`, fork `e483fc8`, differed on 176 files, on 1,161 lines: without `|||` text blocks, `#` and blank lines in them counted as comments and blanks, and a backslash before the closer of an `@'…'` string, which is text there, kept the string open. Fork `215dc41` reads both; on these repositories it takes 0.36 times upstream's time.

## CUE

Judge: cuelang.org/go v0.17.1's `cue/scanner`, the scanner the `cue` command parses with: comment tokens are comments, and strings are literals, while their `\(…)` interpolations are code.

| Repository | Commit | CUE files |
| --- | --- | ---: |
| [kubevela/kubevela](https://github.com/kubevela/kubevela) | `c4dbb1a43a` | 622 |
| [cue-lang/cuelang.org](https://github.com/cue-lang/cuelang.org) | `957f004c34` | 575 |
| [holos-run/holos](https://github.com/holos-run/holos) | `85ab933437` | 423 |
| [stefanprodan/timoni](https://github.com/stefanprodan/timoni) | `40e1054b47` | 332 |
| [cue-labs/cue-by-example](https://github.com/cue-labs/cue-by-example) | `775df47533` | 144 |
| [cue-lang/cue](https://github.com/cue-lang/cue), v0.17.1 | `fc6c0b2ecd` | 121 |

Every file is valid and matches on every line, 234,526 lines in all, with fork `e483fc8` already: 212 files hold interpolations and 25 raw `#"…"#` strings, which the fork's existing declarations read as CUE does.

## Bicep

Judge: the lexer of the Bicep compiler, `Bicep.Core.Parsing.Lexer` in `Azure.Bicep.Core` 0.48.1 ([Bicep file structure and syntax](https://learn.microsoft.com/azure/azure-resource-manager/bicep/file)): comment trivia are comments, and string tokens, `'''` strings and the pieces of interpolated strings are literals, while their `${…}` holes are code.

| Repository | Commit | Bicep files | Lexically invalid |
| --- | --- | ---: | ---: |
| [Azure/bicep-registry-modules](https://github.com/Azure/bicep-registry-modules), `avm/res` | `82950838e7` | 1,878 | 0 |
| [Azure/azure-quickstart-templates](https://github.com/Azure/azure-quickstart-templates) | `59eb6101d3` | 829 | 0 |
| [Azure/bicep](https://github.com/Azure/bicep), v0.48.1, `docs` and `Bicep.Core.Samples` | `cec7951cf6` | 769 | 47 |

Every valid file matches on every line, 880,865 lines in all; the invalid files are the compiler's samples of invalid syntax. Release `15.0.0+locmetry.38`, fork `e483fc8`, differed on 6 files, on 250 lines: Bicep tried `'` before `'''`, so a multi-line string was read as an empty string followed by a single-quoted one, which a backslash before the closer or extra quotes, as in `''''abc''''`, left open (`ScanMultilineString` in `src/Bicep.Core/Parsing/Lexer.cs`). Fork `ecf3ba1` reads `'''…'''` strings verbatim, closing at the last quote of a run, as Bicep does; on these repositories it takes 0.47 times upstream's time.

## Device Tree

dtc exports no tokenizer. Judge: dtc 1.8.1's lexer rules, ported from `dtc-lexer.l` at dtc `8f48565e5c`, with the C preprocessor's handling of `//` comments that Linux's build applies first: `"…"` strings and `'…'` character literals, `/* */` and `//` comments, and `&{…}` path references, inside which `/*` opens no comment.

| Repository | Commit | Device Tree files |
| --- | --- | ---: |
| [zephyrproject-rtos/zephyr](https://github.com/zephyrproject-rtos/zephyr) | `0534e997ea` | 6,207 |
| [torvalds/linux](https://github.com/torvalds/linux), v6.17, `arch/arm64/boot/dts`, `arch/arm/boot/dts/st` and `arch/riscv/boot/dts` | `e5f0a698b3` | 2,737 |
| [dgibson/dtc](https://github.com/dgibson/dtc), v1.8.1 | `8f48565e5c` | 161 |

Every file is valid and matches on every line, 1,786,050 lines in all, with fork `e483fc8` already; 72 files hold path references.

## Module-Definition

Judge: the lexer of LLVM 21.1.0's `llvm/lib/Object/COFFModuleDefinition.cpp`, which `lld-link` and `llvm-dlltool` read `.def` files with: outside a quoted name, `;` starts a comment to the end of the line.

| Repository | Commit | `.def` files |
| --- | --- | ---: |
| [mingw-w64/mingw-w64](https://github.com/mingw-w64/mingw-w64) | `c47c7b4d68` | 2,841 |
| [llvm/llvm-project](https://github.com/llvm/llvm-project), 21.1.0, the `.def` files of the lld, `llvm-dlltool` and `llvm-lib` tests | `3623fe661a` | 15 |

Every file is valid and matches on every line, 200,689 lines in all, with fork `e483fc8` already.

## LD Script

Judge: the C preprocessor, which the builds measured run every linker script through before `ld` reads it, then GNU ld 2.45's lexer, `ld/ldlex.l`: the preprocessor's `/* */` and `//` comments, the latter continued by a final backslash, and `ld`'s `"…"` names, which take no escapes.

| Repository | Commit | Linker scripts |
| --- | --- | ---: |
| [STMicroelectronics/STM32CubeF4](https://github.com/STMicroelectronics/STM32CubeF4) | `60c8abe0ae` | 1,171 |
| [apache/nuttx](https://github.com/apache/nuttx) | `1be6287c5e` | 639 |
| [zephyrproject-rtos/zephyr](https://github.com/zephyrproject-rtos/zephyr) | `0534e997ea` | 443 |
| [tock/tock](https://github.com/tock/tock) | `d4979a4c09` | 84 |
| [u-boot/u-boot](https://github.com/u-boot/u-boot) | `ece349ade2` | 60 |
| [raspberrypi/pico-sdk](https://github.com/raspberrypi/pico-sdk) | `079c6f3902` | 15 |

Every file is valid and matches on every line, 378,049 lines in all; every `#` line in them is a preprocessor directive. Release `15.0.0+locmetry.38`, fork `e483fc8`, differed on 4 files, on 126 lines, LD Script knew only `/* */`, so a `//*lib.a:…` line, a commented-out input section in NuttX's scripts, opened a block comment that ran on. Fork `a7acd1c` reads `//` comments; on these repositories it takes 0.87 times upstream's time.

## Jinja2

Judge: Jinja2 3.1.6's lexer, `Lexer.tokeniter`, in the default syntax that Ansible and Salt use: `{# … #}` comments are comments, and strings in `{{ … }}` and `{% … %}` are literals.

| Repository | Commit | Jinja2 files | Lexically invalid |
| --- | --- | ---: | ---: |
| [debops/debops](https://github.com/debops/debops) | `c7b0813f44` | 798 | 1 |
| [openstack/kolla-ansible](https://github.com/openstack/kolla-ansible) | `297e16ef3b` | 355 | 0 |
| [ansible/ansible](https://github.com/ansible/ansible) | `025bb4d508` | 130 | 0 |
| [saltstack/salt](https://github.com/saltstack/salt) | `0fa7bec684` | 43 | 7 |

Fork `e483fc8` already reads no strings in Jinja2, so quotes in template text open none. One line of the 62,884 differs: a blank line inside a string that spans lines in a debops `{% … %}` tag, which counts as blank instead of code. Of the invalid files, seven are Salt templates with their own `<% %>` delimiters, and one has an unbalanced parenthesis.

## Mustache

Mustache's implementations differ in nothing that decides comments. Judge: the Mustache specification v1.4.3, `specs/comments.yml` and `specs/delimiters.yml` at mustache/spec `e8ec001db7`, and the mustache(5) manual: `{{! … }}` comments, across lines if need be, and `{{=<% %>=}}` delimiter changes; Mustache has no strings.

| Repository | Commit | Mustache files | Lexically invalid |
| --- | --- | ---: | ---: |
| [OpenAPITools/openapi-generator](https://github.com/OpenAPITools/openapi-generator) | `eef27d9229` | 4,429 | 4 |

Every valid file matches on every line, 350,306 lines in all; 91 files change their delimiters, and the invalid files hold a tag that does not close. Release `15.0.0+locmetry.38`, fork `e483fc8`, differed on 407 files, on 5,145 lines: Mustache declared quotes for the whole template, so an apostrophe in its text, as in `don't`, turned later comment and blank lines into code. Fork `afe278c` reads no strings; on these files it takes 0.20 times upstream's time.

## FreeMarker

Judge: FreeMarker 2.3.34's own parser, with the tag syntax detected from a template's first FreeMarker tag as FreeMarker detects it, and with HTML auto-escaping for templates, such as Keycloak's, that use `?no_esc`: the comment elements of the parsed template are comments; string literals in expressions are not reported.

| Repository | Commit | FreeMarker files | Lexically invalid |
| --- | --- | ---: | ---: |
| [apache/ofbiz-framework](https://github.com/apache/ofbiz-framework) | `165c5da1c1` | 525 | 0 |
| [apache/freemarker](https://github.com/apache/freemarker) | `7926e97719` | 196 | 1 |
| [keycloak/keycloak](https://github.com/keycloak/keycloak) | `a79e665763` | 143 | 0 |

Release `15.0.0+locmetry.38`, fork `e483fc8`, differed on 6 files, on 100 lines: the `[#-- … --]` comments of FreeMarker's square-bracket syntax were code. Fork `9d4eb34` reads them. One file of 84,043 lines differs, on 6 lines: FreeMarker's own `comment.ftl` lexer test, which uses the legacy `<comment>` directive of `<#ftl strict_syntax=false>` and a `<#-- … --]` comment inside an expression, which the fork does not read. The invalid file is an exception test. On these repositories the fork takes 1.03 times upstream's time, about 45 ms, where runs vary by several percent; on the corpus's templates replicated into files of 392 MB, it took 1.02 times.

## Apache Velocity

Judge: Velocity 2.4.1's own parser, `RuntimeInstance.parse`, read through its token chain since its tree keeps no comment positions: `##`, `#* … *#` and `#** … *#` comments are comments, from their `##` or `#*`, and `STRING_LITERAL` tokens, which only directive and reference arguments lex, are literals. XWiki's `#try` block directive, which XWiki registers with Velocity, is registered as a stand-in, and a template that does not parse otherwise is parsed again with hyphens allowed in identifiers, as Velocity 1.x read Sakai's `${form-id}`; that changes no comment in the 1,025 files that parse both ways.

| Repository | Commit | Velocity files | Lexically invalid |
| --- | --- | ---: | ---: |
| [sakaiproject/sakai](https://github.com/sakaiproject/sakai) | `4467a0c906` | 344 | 2 |
| [xwiki/xwiki-platform](https://github.com/xwiki/xwiki-platform) | `a3de4aa544` | 323 | 0 |
| [apache/velocity-engine](https://github.com/apache/velocity-engine) | `6f6290c32d` | 139 | 1 |
| [apache/roller](https://github.com/apache/roller) | `cd6ed2778b` | 83 | 0 |
| [apache/maven-fluido-skin](https://github.com/apache/maven-fluido-skin) | `60478c84f6` | 47 | 0 |
| [apache/velocity-tools](https://github.com/apache/velocity-tools) | `03bd638e66` | 45 | 0 |
| [apache/cxf](https://github.com/apache/cxf) | `16c63e353d` | 20 | 0 |
| [apache/cayenne](https://github.com/apache/cayenne) | `64722aceda` | 15 | 0 |
| [apache/turbine-core](https://github.com/apache/turbine-core) | `746f07340b` | 12 | 0 |

Two of the 1,028 files are encoding tests that are not UTF-8 and are left out. Release `15.0.0+locmetry.38`, fork `e483fc8`, differed on 59 files, on 992 lines: Velocity declared quotes for the whole template, so an apostrophe in its text opened a string that hid later comments and counted blank lines as code. Fork `4932872` reads no strings. One file of 105,741 lines differs, on 3 lines, blank lines inside a string that spans lines in velocity-tools' `xml.vm`; of the 19,865 strings, 6 hold `##` and 3 `#*`, which the fork reads as comments inside lines that are code either way. The invalid files are two Sakai templates with `#if ({$rb.…})` and Velocity's own parse-error test. On these repositories the fork takes 0.18 times upstream's time.

## PlantUML

PlantUML has no tokenizer to call; its preprocessor drops comments line by line. Judge: PlantUML v1.2026.9's rules, ported from `TContext`'s `CodeIteratorLongComment`, `CodeIteratorShortComment` and `CodeIteratorInnerComment` over `TLineType`'s line patterns: a line whose first text is `'`, or that holds nothing but `/' … '/`, is a comment; a line whose first text is `/'` and that holds no `'/` opens a comment of whole lines, which the first line ending in `'/` closes; other `/' … '/` share their line with code; PlantUML reads no string across lines.

| Repository | Commit | `.puml` files |
| --- | --- | ---: |
| [tupadr3/plantuml-icon-font-sprites](https://github.com/tupadr3/plantuml-icon-font-sprites) | `1826ccda77` | 6,852 |
| [awslabs/aws-icons-for-plantuml](https://github.com/awslabs/aws-icons-for-plantuml) | `e26e2c05da` | 902 |
| [plantuml/plantuml](https://github.com/plantuml/plantuml), its test diagrams | `a6420f610e` | 725 |
| [RicardoNiepel/Azure-PlantUML](https://github.com/RicardoNiepel/Azure-PlantUML) | `f360835f87` | 302 |
| [plantuml-stdlib/C4-PlantUML](https://github.com/plantuml-stdlib/C4-PlantUML) | `f998c46833` | 103 |
| [hyperledger/aries-rfcs](https://github.com/hyperledger/aries-rfcs) | `e34eec9e23` | 39 |
| [eclipse-tractusx/tractusx-edc](https://github.com/eclipse-tractusx/tractusx-edc) | `86706d6748` | 5 |
| [eclipse-edc/Connector](https://github.com/eclipse-edc/Connector) | `e45e1b1df9` | 3 |

Every file matches on every line, 490,195 lines in all. Release `15.0.0+locmetry.38`, fork `e483fc8`, differed on 3 files, on 9 lines: PlantUML declared `"` strings with backslash escapes, so `"\"` and an unbalanced quote in a label held the following lines in a string, and it opened block comments at a `/'` anywhere. Fork `9b24e4d` classifies PlantUML's lines by the preprocessor's rules instead of lexing them, so a `/'` inside a label opens no comment and a block comment closes only at a line ending in `'/`; on these repositories it takes 0.23 times upstream's time.

## Zig

Judge: Zig 0.16.0's own tokenizer, `std.zig.Tokenizer`, the one the compiler parses with, run from Alpine's packaged Zig in a container: `///` and `//!` doc comment tokens, and the `//` comments between tokens, are comments; string, character and `\\` multiline string line tokens are literals; a file holding an invalid token is invalid.

| Repository | Commit | Zig files | Lexically invalid |
| --- | --- | ---: | ---: |
| [ziglang/zig](https://codeberg.org/ziglang/zig), 0.16.0 | `24fdd5b7a4` | 2,967 | 5 |
| [ghostty-org/ghostty](https://github.com/ghostty-org/ghostty) | `ae0dc1b713` | 814 | 0 |
| [tigerbeetle/tigerbeetle](https://github.com/tigerbeetle/tigerbeetle) | `0450c32b2c` | 248 | 0 |
| [hexops/mach](https://github.com/hexops/mach) | `4be8e50fc8` | 132 | 0 |
| [zigtools/zls](https://github.com/zigtools/zls) | `25df3a3057` | 90 | 0 |
| [riverwm/river](https://github.com/riverwm/river) | `65a908430d` | 57 | 0 |

Every valid file matches on every line, 2,082,237 lines in all counting the invalid files; the invalid files are the compiler's tests of lexical errors, such as a tab inside a comment. Release `15.0.0+locmetry.39`, fork `9dcc6ea`, differed on 48 files, on 4,667 lines: Zig declared no character literals, so `'"'` opened a string, which ran across lines ([Zig Language Reference 0.16.0, String Literals and Unicode Code Point Literals](https://ziglang.org/documentation/0.16.0/#String-Literals-and-Unicode-Code-Point-Literals)). Fork `6069a9f` reads character literals and ends every literal with its line, since Zig's tokenizer rejects a newline in any literal; a `\\` multiline string line is then code whatever it holds, so it needs no declaration. On ziglang/zig, Ghostty and TigerBeetle it takes from 0.57 to 0.69 times upstream's time.

## Liquid

Judge: Liquid 5.14.0's own tokenizer, `Liquid::Tokenizer`, and tag patterns, `BlockBody::FullToken`, `FullTokenPossiblyInvalid` and `LiquidTagToken`, run in a container from the official Ruby image ([Liquid tags, Template](https://shopify.github.io/liquid/tags/template/)): a `comment` block, nesting and skipping `raw` bodies as `Liquid::Comment#parse_body` does, a `doc` block, an inline `{% # … %}` tag, and inside a `{% liquid %}` tag a `#` line or the lines from `comment` to `endcomment`, are comments; `QuotedString`s in tags and variables are literals; Liquid reads text as text, so a `<!--` in text opens a comment that the next `-->` closes, as an HTML parser reads the rendered page, as for the other templates whose text is HTML; a template whose tag or variable does not end, or whose block never closes, is invalid. Tags are not checked further, so the custom tags of Shopify themes and of Jekyll and Eleventy plugins need no definitions.

| Repository | Commit | Liquid files | Lexically invalid |
| --- | --- | ---: | ---: |
| [Shopify/horizon](https://github.com/Shopify/horizon) | `dfad90d87b` | 282 | 0 |
| [Shopify/dawn](https://github.com/Shopify/dawn) | `05222537f6` | 88 | 0 |
| [cfjedimaster/raymondcamden2020](https://github.com/cfjedimaster/raymondcamden2020), Eleventy | `d33864a51c` | 39 | 0 |
| [Shopify/skeleton-theme](https://github.com/Shopify/skeleton-theme) | `a7a655e79b` | 22 | 0 |
| [awesomephant/blog](https://github.com/awesomephant/blog) | `ada2c42a24` | 13 | 0 |
| [cobalt-org/cobalt-org.github.io](https://github.com/cobalt-org/cobalt-org.github.io), Cobalt | `52fac09d2d` | 12 | 0 |
| [stride3d/stride-website](https://github.com/stride3d/stride-website) | `fc2ed1d825` | 4 | 0 |
| [Igalia/wpewebkit.org](https://github.com/Igalia/wpewebkit.org) | `d72b0fd45d` | 2 | 0 |

Every file matches on every line, 96,667 lines in all. Release `15.0.0+locmetry.40`, fork `572c903`, differed on 217 of the 392 files of the three Shopify themes, on 3,029 lines: Liquid declared quotes for the whole template, so an apostrophe in text opened a string, and only `{% comment %}` with single spaces opened a comment, so `{%- comment -%}`, `doc` blocks, inline `{% # %}` tags and `#` lines in `{% liquid %}` tags were code. Whitespace control and nesting leave the comment forms open-ended, so fork `8e8cd77` reads Liquid with its tokenizer's rules in a module of its own; on these repositories it takes 0.31 times upstream's time.

## Haml

Judge: Haml 7.5.1's own parser, `Haml::Parser`, run in a container from the official Ruby image ([Haml reference, Comments](https://haml.info/docs/yardoc/file.REFERENCE.html#comments)). Its tree has a node for every line it reads on its own and folds the rest into the node before them, such as the lines nested under a `-#` comment or a filter, whose text it reads raw, ` |` multiline sequences, script lines a trailing comma continues and attribute hashes. A `-#` comment, and a `/` HTML comment but for a revealed `/![…]` one, is a comment from its marker to the end of the last line before the next node outside it that holds text, so blank lines after it are not in it. Ruby code and filter bodies are not lexed, so the judge reports no literals, and comments inside `:javascript` or `:ruby` bodies are text, as is an HTML comment in text such as a `:plain` body, since Haml writes HTML comments as `/`.

| Repository | Commit | Haml files | Lexically invalid |
| --- | --- | ---: | ---: |
| [gitlabhq/gitlabhq](https://github.com/gitlabhq/gitlabhq) | `d7b9a6176a` | 1,622 | 0 |
| [mastodon/mastodon](https://github.com/mastodon/mastodon) | `147bfc1f17` | 331 | 0 |
| [diaspora/diaspora](https://github.com/diaspora/diaspora) | `f96527862d` | 192 | 0 |
| [haml/haml](https://github.com/haml/haml) | `c06aad5df0` | 53 | 0 |

Every file matches on every line, 47,758 lines in all. Release `15.0.0+locmetry.40`, fork `572c903`, differed on 34 files, on 236 lines: Haml declared quotes for the whole template and `-#` as a line comment, so the lines nested under `-#` and `/` comments, such as Kaminari's documentation blocks in Mastodon, were code, and a `-#` inside a `:preserve` filter's text, as in Haml's own benchmark sample, was a comment. Fork `5ff737a` reads Haml by line and indentation; upstream's fixture had recorded the nested HTML comment as not detected. The fork reads lines by indentation alone, so a continued line that starts like a comment or a filter would open a block; none of these files holds one. On these repositories it takes 0.31 times upstream's time.

## Hamlet

Judge: shakespeare 2.1.7.1's line rules, ported from `Text.Hamlet.Parse.parseLine` since the module exports no function that keeps line positions ([Yesod book, Shakespearean Templates](https://www.yesodweb.com/book/shakespearean-templates)): a line whose text after its indentation opens with `$#` is a comment, and a `<!-- … -->` comment opening a line, which may span lines, is dropped with each further one on its line; an HTML comment later in a line, which Hamlet passes to the page as text, is read as the comment the page holds; tabs in indentation make a file invalid; attribute values end with their line, so a file holds no literals.

| Repository | Commit | Hamlet files | Lexically invalid |
| --- | --- | ---: | ---: |
| [jamesdabbs/pi-base.hs](https://github.com/jamesdabbs/pi-base.hs) | `1c7cb1f87c` | 50 | 0 |
| [ahushh/monaba](https://github.com/ahushh/monaba) | `9fbfffcc97` | 49 | 0 |
| [commercialhaskell/haskellers](https://github.com/commercialhaskell/haskellers) | `4b8fca2a7e` | 29 | 0 |
| [naruto-unison/naruto-unison](https://github.com/naruto-unison/naruto-unison) | `3f81bcd0de` | 27 | 0 |
| [ruHaskell/ruhaskell-yesod](https://github.com/ruHaskell/ruhaskell-yesod) | `f180b30116` | 20 | 0 |
| [commercialhaskell/stackage-server](https://github.com/commercialhaskell/stackage-server) | `948ca5c9ef` | 16 | 0 |
| [astro/bitlove-ui](https://github.com/astro/bitlove-ui) | `e445a2addd` | 13 | 2 |
| [jonschoning/espial](https://github.com/jonschoning/espial) | `27f1d4bcdc` | 13 | 0 |
| [purescript/pursuit](https://github.com/purescript/pursuit) | `b22fa73865` | 9 | 0 |
| [yesodweb/shakespeare](https://github.com/yesodweb/shakespeare) | `7b72d980c2` | 9 | 0 |
| [yesodweb/yesodweb.com](https://github.com/yesodweb/yesodweb.com) | `76e1733926` | 9 | 0 |
| [maoe/tkyprof](https://github.com/maoe/tkyprof) | `82194cdb2b` | 5 | 0 |
| [jgm/gitit2](https://github.com/jgm/gitit2) | `db61eae0e1` | 2 | 0 |

Every valid file matches on every line, 7,254 lines in all counting the invalid files, which indent with tabs. Release `15.0.0+locmetry.40`, fork `572c903`, differed on 11 files, on 35 lines: Hamlet declared quotes for the whole template, so an apostrophe in text opened a string, and `$#` comments were unknown. Fork `637e8b0` reads `$#` at a line's start as a comment and declares no strings; on these repositories it takes 0.23 times upstream's time.

## Extensions shared by several languages

After the build languages, every extension that Tokei gave to one language and [GitHub Linguist's heuristics](https://github.com/github-linguist/linguist/blob/73ee9a3b27/lib/linguist/heuristics.yml) assign to several was checked on public repositories of each language that writes it. Eleven extensions put files of one language into another's counts, so release `15.0.0+locmetry.42`, forks `819319f` and `e065129`, decides them by content: a file goes to a language only when its text matches that language's pattern, and otherwise to the language that still lists the extension plainly, if any. The patterns start from Linguist's rules and were narrowed until no measured file of another language matched: Linguist's Prolog rule, a `:-` on a line without a `#` before it, claimed six Perl files, and a bare fact such as `otherwise.` claimed 30 perl5 files through lines of POD prose, so a Prolog fact needs arguments; an Objective-C file needs an `@` directive or an `#import`, of any file, as protobuf's unity builds import only `.m` files; Phix needs `include`, `constant`, `namespace`, `enum` or `end function` in its own form, without lookahead, which the `regex` crate lacks.

| Extension | Repository | Commit | Written in | Files | Release `.42` gives |
| --- | --- | --- | --- | ---: | --- |
| `.v` | [rocq-prover/rocq](https://github.com/rocq-prover/rocq) | `a1f37202b9` | Coq | 2,931 | Coq 2,872, unrecognized 59, of which 16 are empty |
| `.v` | [math-comp/math-comp](https://github.com/math-comp/math-comp) | `cc79906a94` | Coq | 147 | Coq 147 |
| `.v` | [alexforencich/verilog-ethernet](https://github.com/alexforencich/verilog-ethernet) | `77320a9471` | Verilog | 461 | Verilog 461 |
| `.v` | [openrisc/mor1kx](https://github.com/openrisc/mor1kx) | `f46074a885` | Verilog | 55 | Verilog 55 |
| `.v` | [ZipCPU/zipcpu](https://github.com/ZipCPU/zipcpu) | `42606d2d6e` | Verilog | 80 | Verilog 80 |
| `.v` | [vlang/v](https://github.com/vlang/v) | `92cf0f51a5` | V | 9,675 | unrecognized 9,675 |
| `.fs` | [forthy42/gforth](https://github.com/forthy42/gforth) | `1c5764501d` | Forth | 546 | Forth 544, F# 2 |
| `.fs` | [freedreno-zz/freedreno](https://github.com/freedreno-zz/freedreno) | `f300d3c0cc` | GLSL | 54 | GLSL 54 |
| `.fs` | [tangrams/tangram-es](https://github.com/tangrams/tangram-es) | `ebe6e4df79` | GLSL | 9 | GLSL 9 |
| `.fs` | [dotnet/fsharp](https://github.com/dotnet/fsharp) | `fbc9be3aa8` | F# | 5,639 | F# 5,639 |
| `.e` | [petelomax/Phix](https://github.com/petelomax/Phix) | `a4fba6d9a5` | Phix | 483 | Phix 476, unrecognized 7 |
| `.e` | [OpenEuphoria/euphoria](https://github.com/OpenEuphoria/euphoria) | `fdac8fd9a5` | Euphoria | 369 | Phix 366, unrecognized 3 |
| `.e` | [gobo-eiffel/gobo](https://github.com/gobo-eiffel/gobo) | `0f70feb6df` | Eiffel | 6,457 | unrecognized 6,457 |
| `.mc` | [freebsd/freebsd-src](https://github.com/freebsd/freebsd-src) | `1fb5d8265b` | M4 | 37 | M4 37 |
| `.mc` | [warmsound/crystal-face](https://github.com/warmsound/crystal-face) | `bcc340484b` | Monkey C | 14 | Monkey C 14 |
| `.mc` | [house-of-abbey/GarminHomeAssistant](https://github.com/house-of-abbey/GarminHomeAssistant) | `35d7adf69c` | Monkey C | 28 | Monkey C 28 |
| `.mc` | [memen45/SubMusic](https://github.com/memen45/SubMusic) | `3f6830d6cd` | Monkey C | 75 | Monkey C 75 |
| `.mc` | [microsoft/Windows-driver-samples](https://github.com/microsoft/Windows-driver-samples) | `f4c1ff842b` | Windows message texts | 4 | unrecognized 4 |
| `.pro` | [qt/qtbase](https://github.com/qt/qtbase) | `c68297f665` | QMake | 1,766 | unrecognized 1,766 |
| `.pro` | [android/nowinandroid](https://github.com/android/nowinandroid) | `2982e63cb6` | ProGuard | 3 | unrecognized 3 |
| `.pro` | [signalapp/Signal-Android](https://github.com/signalapp/Signal-Android) | `515db93fb4` | ProGuard | 30 | unrecognized 30 |
| `.pl` | [SWI-Prolog/swipl-devel](https://github.com/SWI-Prolog/swipl-devel) | `08abe65e9d` | Prolog | 538 | Prolog 535, Perl 3 |
| `.pl` | Fifteen Perl repositories, listed below | | Perl | 908 | Perl 908 |
| `.ts` | [keepassxreboot/keepassxc](https://github.com/keepassxreboot/keepassxc) | `0491ec6f06` | Qt translations | 48 | XML 48 |
| `.ts` | [qbittorrent/qBittorrent](https://github.com/qbittorrent/qBittorrent) | `62259b84b3` | Qt translations | 122 | XML 122 |
| `.ts` | Seven TypeScript repositories, listed below | | TypeScript | 18,456 | TypeScript 18,456 |
| `.pan` | [quattor/template-library-core](https://github.com/quattor/template-library-core) | `d6825c6375` | Pan | 618 | Pan 618 |
| `.tpl` | [PrestaShop/PrestaShop](https://github.com/PrestaShop/PrestaShop) | `f706d3cf2d` | Smarty | 366 | unrecognized 366 |
| `.tpl` | [prometheus-community/helm-charts](https://github.com/prometheus-community/helm-charts) | `77d25d4bcd` | Go templates | 47 | unrecognized 47 |
| `.m` | [AFNetworking/AFNetworking](https://github.com/AFNetworking/AFNetworking) | `d9f589cc2c` | Objective-C | 49 | Objective-C 49 |
| `.m` | [SDWebImage/SDWebImage](https://github.com/SDWebImage/SDWebImage) | `c3ad5e1a9b` | Objective-C | 104 | Objective-C 104 |
| `.m` | [protocolbuffers/protobuf](https://github.com/protocolbuffers/protobuf) | `6ce965aab3` | Objective-C | 85 | Objective-C 85 |
| `.m` | [gnu-octave/octave](https://github.com/gnu-octave/octave) | `9335a54f30` | MATLAB | 1,593 | unrecognized 1,593 |
| `.m` | [Psychtoolbox-3/Psychtoolbox-3](https://github.com/Psychtoolbox-3/Psychtoolbox-3) | `8f952b4cae` | MATLAB | 4,265 | unrecognized 4,265 |
| `.m` | [Mercury-Language/mercury](https://github.com/Mercury-Language/mercury) | `3377edc909` | Mercury | 4,185 | unrecognized 4,185 |
| `.pp` | [fpc/FPCSource](https://github.com/fpc/FPCSource) | `bd63f90e11` | Pascal | 12,643 | Pascal 12,634, Puppet 9 |
| `.pp` | [puppetlabs/puppetlabs-apache](https://github.com/puppetlabs/puppetlabs-apache) | `688f56b2b6` | Puppet | 145 | Puppet 145 |
| `.pp` | [puppetlabs/puppetlabs-stdlib](https://github.com/puppetlabs/puppetlabs-stdlib) | `388def1a9a` | Puppet | 69 | Puppet 69 |
| `.srt` | [emacs-mirror/emacs](https://github.com/emacs-mirror/emacs) | `553e6e2d1f` | SRecode | 18 | SRecode 18 |

The Perl repositories are [ack3](https://github.com/beyondgrep/ack3) `11d698f940`, [cpanminus](https://github.com/miyagawa/cpanminus) `c034dd1d38`, [Dancer2](https://github.com/PerlDancer/Dancer2) `64218bec27`, [exiftool](https://github.com/exiftool/exiftool) `980409314f`, [ExtUtils-MakeMaker](https://github.com/Perl-Toolchain-Gang/ExtUtils-MakeMaker) `55834fc274`, [mojo](https://github.com/mojolicious/mojo) `4146e40249`, [Moose](https://github.com/moose/Moose) `98ec7dda05`, [Perl-Critic](https://github.com/Perl-Critic/Perl-Critic) `368a20326a`, [perl5](https://github.com/Perl/perl5) `b3e8681e59`, [rt](https://github.com/bestpractical/rt) `f1ba6d9250`, [postgres](https://github.com/postgres/postgres) `328a9a1731`, [citus](https://github.com/citusdata/citus) `843ef6f049`, [pgvector](https://github.com/pgvector/pgvector) `44624c1f8d`, [timescaledb](https://github.com/timescale/timescaledb) `21b65abd94` and [git](https://github.com/git/git) `b9e5cff180`. The TypeScript repositories are [vscode](https://github.com/microsoft/vscode) `e219e7c0b2`, [react](https://github.com/facebook/react) `c5ec5c6d18`, [supabase](https://github.com/supabase/supabase) `65217161af`, [element-plus](https://github.com/element-plus/element-plus) `a8d102a91b`, [kit](https://github.com/sveltejs/kit) `04aac59bad`, [svelte](https://github.com/sveltejs/svelte) `fe21fce019` and [prettier](https://github.com/prettier/prettier) `c7a823c43b`.

The release's own binary produced every row except the Coq and V rows, which the same patterns gave in a Python harness before those checkouts were removed; the fork checks Coq before Verilog, and no Verilog file above matches the Coq pattern. Before the release, Tokei gave each extension's files to the one language that listed it, so every row whose language differs from that one was miscounted, as [Tokei defects](../tokei-defects.md) records. The files the release misses are small or hold no defining construct: the two Forth files are an opcode table and a constant table without a colon definition or `\` comment, the ten Phix and Euphoria files are empty, comments only, tests of a line or two or inline assembly, the three Prolog files hold only bare facts or a comment, and the nine Pascal files are compiler tests of a statement or two, several meant to fail. Phix reads Euphoria, its ancestor, with the same comments and strings. Specman e is now reachable, as `.e` files that open a `<'` code block, but no public corpus was found to measure it; SubRip subtitles were checked only with synthetic files. Reading a file whose extension carries a check costs a second read of its text: VS Code, with 15,059 TypeScript files, took 2.134 s against upstream's 5.761 s.

Linguist also disambiguates extensions Tokei still gives to one language, such as `.cls`, `.cl`, `.re`, `.ftl`, `.l`, `.x`, `.p`, `.scm` and `.ml`. None of the corpora measured for step 6 held a file of another language with them, except 21 Tree-sitter query files read as Scheme, whose `;` comments Scheme shares, and seven Standard ML files read as OCaml.

## Speed

The owner decided on 2026-10-30 that the fork must parse within 10% of upstream Tokei 15.0.0's time on every repository below; the fork's [speed gate](https://github.com/mbackschat/tokei/tree/locmetry/judges#speed), `judges/speed/check.py`, measures it before each release is adopted. It parses every file Tokei recognizes from memory, three times, and compares the minimum user CPU time of 15 interleaved runs. Django, Tokio, Prettier, Swift NIO, bloc, Cats, ggplot2, Phoenix, Pester, pandoc, cabal, Tomcat, Bootstrap, Element Plus, SvelteKit, Svelte, KOReader, Citus, Prometheus's Helm charts, perl5, Moose, termux-packages, acme.sh, Buildroot and Git are whole checkouts, GitHub's docs are measured on their `content` folder, Linux on its makefiles, and the other trees hold the files of their judged languages.

Upstream's Haskell declares no strings, so it never analyses the lines holding them, a quarter of cabal's Haskell lines and a third of pandoc's; against it, fork `994df39` took 1.10 times as long on pandoc and 1.38 times on cabal, and upstream's own engine took 3.4 times its time on cabal's Haskell files once given the quotes. The owner decided on 2026-10-30 that the gate gives upstream the string quotes of languages whose upstream definition has none, listed in the gate's `baseline-quotes.json`, so it measures the fork's own overhead; the Haskell rows below are measured that way. Upstream's JSON declares no strings either, and since fork `fc28619` reads the comments of JSON with Comments, the fork must read JSON's strings too: vcpkg, whose 6,200 JSON files dominate its time, took 1.41 times upstream's time, and 0.81 times once upstream was given JSON's quotes (fork `69cc49b`, measured 2026-10-30); the vcpkg row is measured that way. Upstream's Just declares no strings either; the Just row is measured with Just's quotes given to upstream, and against upstream without them the fork took 1.47 times its time on those files (fork `8dfaf47`, measured 2026-10-31).

| Repository | Commit | Language | Upstream | Fork `819319f` | Ratio |
| --- | --- | --- | ---: | ---: | ---: |
| The nine repositories of the [Apache Velocity](#apache-velocity) section, Velocity files | as listed there | Apache Velocity | 0.251 s | 0.042 s | 0.17 |
| The eight repositories of the [Liquid](#liquid) section, Liquid files | as listed there | Liquid | 0.142 s | 0.031 s | 0.22 |
| [OpenAPITools/openapi-generator](https://github.com/OpenAPITools/openapi-generator) Mustache files | as listed there | Mustache | 0.426 s | 0.100 s | 0.23 |
| The two repositories of the [Module-Definition](#module-definition) section, `.def` files | as listed there | Module-Definition | 0.084 s | 0.021 s | 0.25 |
| The thirteen repositories of the [Hamlet](#hamlet) section, Hamlet files | as listed there | Hamlet | 0.018 s | 0.005 s | 0.26 |
| [symfony/symfony](https://github.com/symfony/symfony) | `92216211` | PHP | 4.011 s | 1.176 s | 0.29 |
| The eight repositories of the [PlantUML](#plantuml) section, `.puml` files | as listed there | PlantUML | 0.260 s | 0.091 s | 0.35 |
| The six repositories of the [Jsonnet](#jsonnet) section, Jsonnet files | as listed there | Jsonnet | 1.013 s | 0.367 s | 0.36 |
| The four repositories of the [Bitbake](#bitbake) section, BitBake files | as listed there | Bitbake | 0.449 s | 0.169 s | 0.38 |
| The four repositories of the [Haml](#haml) section, Haml files | as listed there | Haml | 0.135 s | 0.052 s | 0.39 |
| [koreader/koreader](https://github.com/koreader/koreader) | `80127739` | Lua | 0.347 s | 0.134 s | 0.39 |
| [prometheus-community/helm-charts](https://github.com/prometheus-community/helm-charts) | `77d25d4b` | YAML | 0.331 s | 0.142 s | 0.43 |
| [pester/Pester](https://github.com/pester/Pester) | `98e56d65` | PowerShell | 0.146 s | 0.063 s | 0.43 |
| [twbs/bootstrap](https://github.com/twbs/bootstrap) | `c967e2cb` | SCSS, CSS | 0.197 s | 0.087 s | 0.44 |
| [torvalds/linux](https://github.com/torvalds/linux) makefiles | `d89e1d18` | Makefile | 0.046 s | 0.020 s | 0.45 |
| [element-plus/element-plus](https://github.com/element-plus/element-plus) | `a8d102a9` | Vue | 0.376 s | 0.169 s | 0.45 |
| [git/git](https://github.com/git/git) | `b9e5cff1` | C, Shell, Makefile | 1.050 s | 0.478 s | 0.46 |
| [buildroot/buildroot](https://github.com/buildroot/buildroot) | `93156d58` | Makefile | 0.325 s | 0.148 s | 0.46 |
| The three repositories of the [Bicep](#bicep) section, Bicep files | as listed there | Bicep | 0.520 s | 0.238 s | 0.46 |
| [crystal-lang/crystal](https://github.com/crystal-lang/crystal) Crystal files | `9400d45c` | Crystal | 0.395 s | 0.187 s | 0.47 |
| [erlang/otp](https://github.com/erlang/otp) Erlang files | `4ba5448a` | Erlang | 1.623 s | 0.773 s | 0.48 |
| [npocmaka/batch.scripts](https://github.com/npocmaka/batch.scripts) | `7e784303` | Batch | 0.007 s | 0.003 s | 0.48 |
| [tokio-rs/tokio](https://github.com/tokio-rs/tokio) | `6470b5e4` | Rust | 0.193 s | 0.093 s | 0.48 |
| [felangel/bloc](https://github.com/felangel/bloc) | `b9be1e25` | Dart | 0.214 s | 0.105 s | 0.49 |
| [microsoft/vcpkg](https://github.com/microsoft/vcpkg) | `d186940b` | CMake, JSON | 0.796 s | 0.391 s | 0.49 |
| [django/django](https://github.com/django/django) | `817df1c4` | Python | 1.434 s | 0.707 s | 0.49 |
| [phoenixframework/phoenix](https://github.com/phoenixframework/phoenix) | `c9065a89` | Elixir | 0.120 s | 0.061 s | 0.51 |
| [ziglang/zig](https://codeberg.org/ziglang/zig) Zig files | `24fdd5b7` | Zig | 0.645 s | 0.336 s | 0.52 |
| [fpc/FPCSource](https://github.com/fpc/FPCSource) Pascal files | `fb4143a1` | Pascal | 4.126 s | 2.168 s | 0.53 |
| [acmesh-official/acme.sh](https://github.com/acmesh-official/acme.sh) | `37d064f7` | Shell | 0.102 s | 0.053 s | 0.53 |
| [prettier/prettier](https://github.com/prettier/prettier) | `c7a823c4` | JavaScript, TypeScript | 0.435 s | 0.231 s | 0.53 |
| [jgm/pandoc](https://github.com/jgm/pandoc) | `75ebbfa7` | Haskell | 0.179 s | 0.097 s | 0.54 |
| [tidyverse/ggplot2](https://github.com/tidyverse/ggplot2) | `b638d654` | R | 0.161 s | 0.090 s | 0.56 |
| [fpc/Lazarus](https://github.com/fpc/Lazarus) Pascal files | `709a56c0` | Pascal | 2.116 s | 1.214 s | 0.57 |
| [sveltejs/svelte](https://github.com/sveltejs/svelte) | `fe21fce0` | Svelte | 0.371 s | 0.215 s | 0.58 |
| The justfiles of the [Just](#just) section | as listed there | Just | 0.159 s | 0.093 s | 0.58 |
| The seven repositories of the [HCL](#hcl) section, HCL files | as listed there | HCL | 0.647 s | 0.387 s | 0.60 |
| [dotnet/roslyn](https://github.com/dotnet/roslyn), [dotnet/docs](https://github.com/dotnet/docs), [dotnet/samples](https://github.com/dotnet/samples) and [icsharpcode/CodeConverter](https://github.com/icsharpcode/CodeConverter) Visual Basic files | `dcfa99c3`, `bcbe167f`, `ab42c8a4`, `a166046e` | Visual Basic | 1.460 s | 0.884 s | 0.60 |
| [JuliaLang/julia](https://github.com/JuliaLang/julia) Julia files | `3a061696` | Julia | 0.470 s | 0.287 s | 0.61 |
| [nim-lang/Nim](https://github.com/nim-lang/Nim) Nim files | `9a4192cc` | Nim | 0.267 s | 0.163 s | 0.61 |
| [ghostty-org/ghostty](https://github.com/ghostty-org/ghostty) Zig files | `ae0dc1b7` | Zig | 0.146 s | 0.089 s | 0.61 |
| [sveltejs/kit](https://github.com/sveltejs/kit) | `04aac59b` | Svelte | 0.176 s | 0.108 s | 0.61 |
| The six repositories of the [CUE](#cue) section, CUE files | as listed there | CUE | 0.183 s | 0.114 s | 0.62 |
| The three repositories of the [Device Tree](#device-tree) section, Device Tree files | as listed there | Device Tree | 1.031 s | 0.648 s | 0.63 |
| [Perl/perl5](https://github.com/Perl/perl5) | `b3e8681e` | Perl | 1.248 s | 0.802 s | 0.64 |
| [gohugoio/hugo](https://github.com/gohugoio/hugo) | `a1668da3` | Go | 0.201 s | 0.129 s | 0.64 |
| [termux/termux-packages](https://github.com/termux/termux-packages) | `34a8c84d` | Shell | 0.478 s | 0.307 s | 0.64 |
| [haskell/cabal](https://github.com/haskell/cabal) | `07e003b3` | Haskell | 0.306 s | 0.200 s | 0.65 |
| [moose/Moose](https://github.com/moose/Moose) | `98ec7dda` | Perl | 0.075 s | 0.050 s | 0.66 |
| [rubocop/rubocop](https://github.com/rubocop/rubocop) | `bde5d420` | Ruby | 0.308 s | 0.205 s | 0.67 |
| The nine repositories of the [COBOL](#cobol) section, COBOL files | as listed there | COBOL | 0.321 s | 0.214 s | 0.67 |
| [apache/tomcat](https://github.com/apache/tomcat) | `8c164da3` | XML, HTML | 0.690 s | 0.478 s | 0.69 |
| [tigerbeetle/tigerbeetle](https://github.com/tigerbeetle/tigerbeetle) Zig files | `0450c32b` | Zig | 0.074 s | 0.052 s | 0.71 |
| [AutoMapper/AutoMapper](https://github.com/AutoMapper/AutoMapper) | `df65f2e3` | C# | 0.043 s | 0.030 s | 0.71 |
| [citusdata/citus](https://github.com/citusdata/citus) | `843ef6f0` | SQL, C | 0.442 s | 0.314 s | 0.71 |
| The five repositories of the [Nix](#nix) section, Nix files | as listed there | Nix | 1.405 s | 0.998 s | 0.71 |
| [Reference-LAPACK/lapack](https://github.com/Reference-LAPACK/lapack) Fortran files | `472b3a0f` | FORTRAN Legacy, FORTRAN Modern | 0.691 s | 0.513 s | 0.74 |
| [spring-projects/spring-framework](https://github.com/spring-projects/spring-framework) | `64861cb3` | Java, Kotlin | 1.120 s | 0.837 s | 0.75 |
| [apple/swift-nio](https://github.com/apple/swift-nio) | `108daca3` | Swift | 0.114 s | 0.086 s | 0.76 |
| The eight repositories of the [Solidity](#solidity) section, Solidity files | as listed there | Solidity | 0.225 s | 0.171 s | 0.76 |
| [curl/curl](https://github.com/curl/curl) | `87c4c6d6` | C | 0.261 s | 0.206 s | 0.79 |
| [typelevel/cats](https://github.com/typelevel/cats) | `b6c4dbfb` | Scala | 0.104 s | 0.088 s | 0.84 |
| [github/docs](https://github.com/github/docs) `content` | `59fde2de` | Markdown | 0.167 s | 0.143 s | 0.85 |
| [ocaml/ocaml](https://github.com/ocaml/ocaml) OCaml files | `a76fb1de` | OCaml | 0.392 s | 0.338 s | 0.86 |
| The six repositories of the [LD Script](#ld-script) section, linker scripts | as listed there | LD Script | 0.273 s | 0.239 s | 0.87 |
| [dotnet/fsharp](https://github.com/dotnet/fsharp) F# files | `fbc9be3a` | F# | 1.184 s | 1.035 s | 0.87 |
| The eleven repositories of the [GNU Style Assembly](#gnu-style-assembly) and [Assembly](#assembly) sections, assembly files | as listed there | GNU Style Assembly, Assembly | 0.882 s | 0.849 s | 0.96 |
| The debops, Ansible and Salt repositories of the [Jinja2](#jinja2) section, Jinja2 files | as listed there | Jinja2 | 0.021 s | 0.021 s | 1.01 |
| The three repositories of the [FreeMarker](#freemarker) section, FreeMarker files | as listed there | FreeMarker | 0.046 s | 0.047 s | 1.01 |

Measured on 2026-10-31, in one run of the gate, from 0.17 to 1.01. Fork `f1b0b53` measured FreeMarker at 1.13 earlier that day and again above 1.10 in a rerun, then 1.07 and 1.05 in two more and 1.04 over 45 runs, while the load average stayed near 8 on 8 cores; the code a FreeMarker file runs differs from fork `572c903`'s only by two comparisons per file. Its Liquid row was a rerun, since Liquid files were added to that root during its run. Fork `572c903` measured from 0.18 to 1.04 earlier that day, before Liquid, Haml and Hamlet were added. Fork `9dcc6ea` measured from 0.18 to 1.03 earlier that day, before Zig was added, while Spotlight indexing and two other processes kept the load average between 8 and 15 on 8 cores. Fork `9b24e4d` measured Jinja2 at 1.15, FreeMarker at 1.13 and Solidity at 1.15 earlier that day; a tree of one Jinja2 file took 3.4 times upstream's time, since the fork's first detection of a file with an extension compiled the patterns of every content check, about 4 ms, the whole count of those small corpora. Fork `6486e48` compiles a check's patterns only when its extension comes up, and measured that tree at 1.04, Jinja2 at 1.00, FreeMarker at 1.05 and Solidity at 1.00; Solidity had also measured 0.89 in a rerun before the change. Fork `cb12c3b` measured from 0.28 to 0.95 earlier that day, before BitBake and Just were added; its run measured all Solidity repositories at 1.48 while two Lean builds and Spotlight indexing kept the machine busy, and three reruns of that root alone measured 1.03, 0.84 and 0.77. Fork `8ca1543` measured from 0.30 to 0.86 earlier that day, before COBOL and assembly were added. Earlier, on ethereum/solidity alone, about 0.1 s of work, the ratio ranged from 0.90 to 1.22 over eight runs, four of them above 1.10, while Spotlight indexing kept the load average near 8 on 8 cores and upstream's minimum moved from 0.107 s to 0.165 s between runs; the Solidity row therefore measures all eight Solidity repositories together, which measured 0.94, 0.94 and 1.09 in three separate runs before this table's. Fork `c18e585` measured from 0.23 to 0.87 earlier that day, before Pascal, Nim, Crystal and Solidity were added, and fork `3343fdb` measured from 0.27 to 0.88 earlier that day, before OCaml and Fortran were added, and fork `9043f73` from 0.28 to 0.90 on a lightly loaded machine, with a load average of about 3 on 8 cores, before F#, Julia and Erlang were added, and fork `85e6ed1` from 0.26 to 0.89, before Visual Basic was added. Fork `59523f1` measured from 0.28 to 0.92 on 2026-10-30 on a loaded machine, where the minimum of interleaved runs varied by about 3% between repetitions. Fork `1806dc8` found Markdown's code blocks with pulldown-cmark, whose first pass also scans inline syntax: on GitHub's `content`, 3,796 Markdown files, it took 1.05 and 1.07 times upstream's time in two runs, but 1.37 in a later run in which upstream ran faster, so it failed the gate, and its first pass alone took 28 of the 48 ms that upstream's whole count of those files took. Fork `72d47d8` reads only CommonMark's block structure, ported from commonmark.js, and took 0.79, but from 0.97 to above 1.10 over 30 runs once the machine was quiet; fork `6758800` counts Markdown's lines straight from the block list instead of the generic line loop, skips the block pass for files without three backticks or tildes in a row, 2,806 of the 3,796, and finds line ends with a vectorized search, and took 0.92 over 30 runs and 0.71 in an earlier run of the table, 0.74 in that of fork `f8826bd`, 0.87 in that of fork `59523f1`, 0.67 in that of fork `85e6ed1`, 0.90 in that of fork `9043f73`, 0.78 in that of fork `3343fdb`, 0.67 in that of fork `c18e585` and 0.69 in that of fork `8ca1543`. Cabal ranged from 0.64 to 0.95 between runs of unchanged Haskell code and measured 0.69 and 0.71 over 30 runs. Fork `a6c2728` measured from 0.27 to 0.83 on the first twenty-four. Fork `9c85621` measured from 0.26 to 0.83 on the first twenty-two. Swift NIO, bloc, Cats and ggplot2 were measured as files of their judged languages until fork `397da12`, which measured from 0.27 to 0.82 on the first twenty, and as whole checkouts since, after the gate was found passing roots that no longer existed; it fails them since fork `47e6301`. Fork `ffb6076` measured from 0.25 to 0.83 on the first seventeen. Fork `994df39` passes over runs of bytes that change nothing at once, which cut the ratios from 0.45–0.96 for fork `687cb25`; earlier measurements found ratios from 0.53 to 0.92 for fork `55b0c87` and from 0.42 to 0.94 for fork `984e57b`.
