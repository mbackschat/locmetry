# Tokei Fork Step 6: Each Language's Judge

Measured from 2026-10-29 on macOS arm64, for [step 6](../plans/tokei-fork.md#step-6-no-difference-from-each-languages-judge): every file of every corpus must count as the language's own lexer or parser counts it. A line is a comment when it holds nothing but comments; a blank line takes the class of the comment or literal it lies in; every other line is code. A file the judge itself rejects as lexically invalid has no correct count, so it is listed rather than matched.

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
