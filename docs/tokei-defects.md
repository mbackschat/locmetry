# Tokei Counting Defects

Locmetry counts with its fork of Tokei 15.0.0 ([Tokei integration](tokei-integration.md)). Tokei lexes every language with one generic model from its `languages.json`: line comments, block comments, and quote pairs that protect comment markers. When a language's definition omits a literal form, or declares quotes where the text is not code, comment markers and quotes are misread, and the error can carry on to the end of the file. This page is the register of defects found, which ones Locmetry corrects, and the evidence for the rest. [Tokei integration](tokei-integration.md#counting-boundary) owns how corrections replace Tokei's parsing, and [measurement semantics](measurement.md#counting) owns the resulting counting rules.

## Corrected by Locmetry

| Language | Tokei defect | Effect before the fix | Fix |
| --- | --- | --- | --- |
| Lean | No string quotes declared, so `/-` inside a string opened a nested block comment. | Every following line counted as comment: real files lost up to 83% of their code lines. | `62dea4a`, local lexer in [lean.rs](../src/backend/lean.rs) that follows Lean's own token rules. |
| Handlebars | Quotes declared for the whole file, so an apostrophe in template text opened a string. | Later comment and blank lines counted as code. | `f88b39a`, local lexer in [handlebars.rs](../src/backend/handlebars.rs). |
| Jupyter | The notebook parser overlaps parent and cell statistics and replaces parse failures with zeros. | Double counting, and silent zero counts. | `3e2684f`, local decoder in [notebook.rs](../src/backend/notebook.rs). |
| C, C++, Java, Kotlin, C#, Go, Rust, Scala, OCaml, F# | A character literal holding a double quote, such as `'"'`, opened a string. | Undercount: the misread string ended at the next `"`, which exposed `/*`. | Fork `9e65ec4` and `6798eb4`. |
| Haskell, Elm, Agda, PureScript, Racket, Scheme, Lisp | No quotes declared, so `{-` or `#\|` in a string opened a comment. | Undercount: the rest of the file was comment. | Fork `6798eb4`, with character literals. |
| Go | Backtick raw strings were not literals. | Undercount. | Fork `2c95b8f`. |
| Java | A lone quote in a text block closed it. | Undercount. | Fork `2c95b8f`. |
| Rust | Raw strings ending in a backslash, such as `r"C:\"`, or with three or more `#`. | Undercount. | Fork `2c95b8f`, and `1926802` for any number of `#`. |
| Swift | Raw strings such as `#"a "/*" b"#`, and lone quotes in `"""` strings. | Undercount. | Fork `1926802` and `39610e6`. |
| C++ | Raw strings with a delimiter, such as `R"sql(a "/*" b)sql"`. | Undercount. | Fork `1926802`. |
| C# | Raw string literals such as `"""a "/*" b"""`. | Undercount. | Fork `1926802`. |
| Lua | No long strings declared, and long comments only at level 0, so `[[ -- text ]]` held a comment and `--[==[ … ]] … ]==]` ended early. | Comments counted as code, or code as comments, up to the next misread bracket. | Fork `1926802`. |
| OCaml | Quoted strings such as `{\|(*\|}`. | Undercount. | Fork `1926802`. |
| C#, F# | A `""` escape inside a `@"…"` string closed it, so a preceding backslash flipped string state. | Comments counted as code, or code as comments, up to the next misread quote. | Fork `b773e13`. |
| PHP, F# | `#[` attributes and the `(*)` operator section start like comments. | Undercount: each such line was a comment. | Fork `a23b225`. |
| All | A line whose text starts with a comment marker counted as a comment even when it started inside a literal, such as a `// …` line ending a raw string. | Undercount of one line per occurrence. | Fork `801aae6`. |

Locmetry adopted the fork's corrections in `1e8998c` and its raw-string corrections in `51d9664`; the fork's [README](https://github.com/mbackschat/tokei#changes-from-upstream) cites the specification behind each declaration, and the [step 1](research/tokei-fork-step-1.md) and [step 2](research/tokei-fork-step-2.md) measurements compare the results with tree-sitter on public repositories. The Lean and Handlebars lexers keep Tokei's line classification, so only the lexing changes. One Tokei rule is deliberately kept: a line that starts inside a block comment is a comment even when code follows the comment's end.

## Uncorrected Defects

Found on 2026-10-28 with synthetic probes measured at `62dea4a`. Probes for Rust, Swift, C, C++, Java, JavaScript, Ruby and Shell were checked with the language's compiler or syntax checker; the others are unchecked against a toolchain. Where a row names several languages, those without a probe share a probed language's Tokei definition. The [step 1 measurement](research/tokei-fork-step-1.md) found PHP heredocs and the [step 2 measurement](research/tokei-fork-step-2.md) found C# raw strings nested in interpolation in public repositories; the prevalence of the other rows has not been measured.

An undercount happens when a misread literal exposes a block-comment opener: the rest of the file, or everything up to the next closer, becomes comment. An overcount happens when a misread quote opens a string: comment and blank lines count as code up to the next matching quote. Either effect can follow from the same defect, depending on what comes next in the file.

### Literal forms Tokei does not declare

| Language | Construct | Probe | Effect |
| --- | --- | --- | --- |
| C# | Strings nested in interpolation holes. | `$"{(f ? "/*" : "")}"` | Undercount. |
| JavaScript, TypeScript | Regular expression literals. | `p.replace(/\/*$/, "")`, `/"/` | Undercount. |

### Quotes declared where the text is not code

These are the same class as the Handlebars defect: an apostrophe in prose opens a string.

| Language | Construct | Probe | Effect |
| --- | --- | --- | --- |
| JSX, TSX, HTML, Vue | Apostrophes in element text. | `<p>Don't panic</p>` | Overcount: comments up to the next apostrophe count as code. |
| Ruby, Shell, PHP | Apostrophes in heredocs. | `Don't panic` inside `<<EOS` | Overcount. |
| YAML | Apostrophes in plain scalars. | `description: Don't panic` | Overcount. |

Python's nested f-string quotes rebalance within the line and were counted correctly.

## Upstream

The open pull requests [#1389](https://github.com/XAMPPRocky/tokei/pull/1389) for character literals and [#1351](https://github.com/XAMPPRocky/tokei/pull/1351) for Go backtick strings had no review on 2026-10-29; the repository has its issue tracker disabled. [Counting backends](research/counting-backends.md) compares ways to correct the defects in Locmetry, and the fork's corrections are to be offered upstream as the [fork plan](plans/tokei-fork.md) describes.

No Tokei issue about Lean existed on 2026-10-28; the Lean definition on Tokei's master branch still declares no quotes. Character literals, and raw strings with any number of `#`, cannot be expressed in upstream Tokei's quote model at all, which is why the fork extends its engine.
