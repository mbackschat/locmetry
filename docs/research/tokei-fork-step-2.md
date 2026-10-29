# Tokei Fork Step 2 Measurement

Measured 2026-10-29 on macOS arm64, for step 2 of the [Tokei fork plan](../plans/tokei-fork.md): raw strings and comments whose closing delimiter repeats a count or label from the opener. The fork is [mbackschat/tokei](https://github.com/mbackschat/tokei) at `62ec728`, version `15.0.0+locmetry.2`; its README cites the specification or reference lexer behind each declaration. The method and harness are those of [step 1](tokei-fork-step-1.md), with tree-sitter grammars for Swift 0.7.4 and Lua 0.5.0 added.

## Change

The engine gained `raw_strings` and `raw_comments` declarations: an opener is a fixed start, a run of one repeated character or a label, and a fixed open text, and the closer is computed from what the file contains. Declared forms:

| Language | Forms |
| --- | --- |
| Rust | `r"…"`, `br"…"` and `cr"…"` with any number of `#`, replacing the step-1 declarations that stopped at three. |
| Swift | `#"…"#` and `#"""…"""#` with any number of `#` and their `\#` escapes, and `"""…"""`. A `#"` opener counts as a one-line string when the line closes it, which is how Swift's own lexer reads `#"""#`. |
| C# | `"""…"""` raw strings with three or more quotes. |
| C++ | `R"x(…)x"` with delimiters of up to 16 characters, replacing the declaration that accepted only an empty one. |
| Lua | `[==[…]==]` long strings and `--[==[…]==]` long comments of any level, replacing the level-0 comment declaration. |
| OCaml | `{id\|…\|id}` quoted strings. |

Each form has a regression file in the fork. The Rust, Swift, C++ and Lua files pass `rustc`, `swiftc -typecheck`, `clang++ -std=c++20 -fsyntax-only` and `luac -p`; the C# and OCaml files parse without errors in their tree-sitter grammars, which agree with the hand-derived counts.

## Corpora

The 17 step-1 repositories, which hold the OCaml, C++, C# and Rust code measured here, plus five shallow clones chosen for these constructs: swift-syntax `237beba`, Kong `2103212`, nlohmann/json `46bead6`, dotnet/aspire `818da5a` and lazy.nvim `306a055`. Together they hold 39,258 files of the measured languages.

## Results

Files whose counts differ from tree-sitter's:

| Language | Files | Tokei | Fork |
| --- | --- | --- | --- |
| C++ | 754 | 18 | 0 |
| C# | 7,381 | 186 | 2 |
| Lua | 1,427 | 173 | 0 |
| OCaml | 1,896 | 31 | 0 |
| Rust | 932 | 451 | 451 |
| Swift | 770 | 25 | 3 |

The first pass found 6 Swift files; three of them held `#"""#`, which led to the one-line rule above.

## Remaining Disagreements

- **Rust (451):** only `///` and `//!` lines, which Tokei deliberately reads as Markdown; two files start with a byte-order mark before `//!`.
- **C# (2):** a raw string inside the interpolation hole of an interpolated string, such as `$"… {Regex.Replace(p, """…""", "^$0")} "`, which step 3 covers.
- **Swift (3):** two files have tree-sitter parse errors, and in swift-syntax's `LexerTests.swift` tree-sitter reads part of a regex test as a literal, counting 70 blank and 6 comment lines as code; in all three the fork's lines are correct on inspection.

## Speed

Median instructions relative to upstream Tokei 15.0.0, from hyperfine on one thread, ten runs per repository:

| Repository | Step 1 | Step 2 |
| --- | --- | --- |
| tokio | 1.04 | 0.96 |
| ripgrep | 0.95 | 0.87 |
| swift-syntax | 0.91 | 0.92 |
| Kong | 0.81 | 0.84 |
| nlohmann/json | 0.80 | 0.82 |
| dune | 0.96 | 1.00 |
| dotnet/aspire | 0.79 | 0.80 |
| guava | 0.89 | 0.93 |

## Assessment

Verified: on these repositories the fork matches tree-sitter for C++, Lua and OCaml, and every remaining Rust, C# and Swift difference is Tokei's deliberate doc-comment rule, a construct scheduled for step 3, or a tree-sitter misreading. Speed stays at or below upstream.

Limits: one machine and one run of each corpus, and the Swift remainders rest on inspection. The Lua files contain 1,665 long-bracket openers of level 0, 22 of level 1, 4 of level 2 and 2 of level 5.
