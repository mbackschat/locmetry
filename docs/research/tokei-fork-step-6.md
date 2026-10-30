# Tokei Fork Step 6: Each Language's Judge

Measured from 2026-10-29 on macOS arm64, for [step 6](../plans/tokei-fork.md#step-6-no-difference-from-each-languages-judge): every file of every corpus must count as the language's own lexer or parser counts it. A line is a comment when it holds nothing but comments; a blank line takes the class of the comment or literal it lies in; every other line is code. A file the judge itself rejects as lexically invalid has no correct count, so it is listed rather than matched.

The judges and the comparison tool live in the fork's [judges](https://github.com/mbackschat/tokei/tree/locmetry/judges) folder from `a2a9724`. Each judge reports the comment and literal ranges its lexer found, and `judgecmp` compares the lines they imply with the fork's `line_classes` (fork `238b0c5`), line by line. The fork's results below are those of `15.0.0+locmetry.13`, fork `a55e730`, for Python of `15.0.0+locmetry.14`, fork `701fdaa`, for C, C++, Objective-C, Go, C# and Ruby of `15.0.0+locmetry.15`, fork `40ed432`, and for PHP, Swift, Dart and Scala of `15.0.0+locmetry.16`, fork `eb463fb`. A line of whitespace takes the class of the comment or literal its own bytes lie in, as judgecmp reads it from fork `02345d9`; every earlier result was rechecked under that rule.

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
