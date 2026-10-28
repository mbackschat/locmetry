# Tokei Counting Defects

Locmetry counts with the embedded Tokei 15.0.0 ([Tokei integration](tokei-integration.md)). Tokei lexes every language with one generic model from its `languages.json`: line comments, block comments, and quote pairs that protect comment markers. When a language's definition omits a literal form, or declares quotes where the text is not code, comment markers and quotes are misread, and the error can carry on to the end of the file. This page is the register of defects found, which ones Locmetry corrects, and the evidence for the rest. [Tokei integration](tokei-integration.md#counting-boundary) owns how corrections replace Tokei's parsing, and [measurement semantics](measurement.md#counting) owns the resulting counting rules.

## Corrected by Locmetry

| Language | Tokei defect | Effect before the fix | Fix |
| --- | --- | --- | --- |
| Lean | No string quotes declared, so `/-` inside a string opened a nested block comment. | Every following line counted as comment: real files lost up to 83% of their code lines. | `62dea4a`, local lexer in [lean.rs](../src/backend/lean.rs) that follows Lean's own token rules. |
| Handlebars | Quotes declared for the whole file, so an apostrophe in template text opened a string. | Later comment and blank lines counted as code. | `f88b39a`, local lexer in [handlebars.rs](../src/backend/handlebars.rs). |
| Jupyter | The notebook parser overlaps parent and cell statistics and replaces parse failures with zeros. | Double counting, and silent zero counts. | `3e2684f`, local decoder in [notebook.rs](../src/backend/notebook.rs). |

The Lean and Handlebars lexers keep Tokei's line classification, so only the lexing changes. One Tokei rule is deliberately kept: a line that starts inside a block comment is a comment even when code follows the comment's end.

## Uncorrected Defects

Found on 2026-10-28 with synthetic probes measured at `62dea4a`. Probes for Rust, Swift, C, C++, Java, JavaScript, Ruby and Shell were checked with the language's compiler or syntax checker; the others are unchecked against a toolchain. Where a row names several languages, those without a probe share a probed language's Tokei definition. Prevalence in real projects has not been measured, apart from this repository: Tokei counts [lean.rs](../src/backend/lean.rs) at 189 code, 18 comment and 6 blank lines, while the file has 22 comment-only and 7 blank lines, because of its `'"'` character literals.

An undercount happens when a misread literal exposes a block-comment opener: the rest of the file, or everything up to the next closer, becomes comment. An overcount happens when a misread quote opens a string: comment and blank lines count as code up to the next matching quote. Either effect can follow from the same defect, depending on what comes next in the file.

### Literal forms Tokei does not declare

| Language | Construct | Probe | Effect |
| --- | --- | --- | --- |
| Haskell, Elm, Agda, PureScript | No quotes declared at all. | `main = putStrLn "{-"` | Undercount: the rest of the file is comment. |
| Racket, Scheme, Lisp | No quotes declared at all. | `(define s "#\|")` | Undercount. |
| Rust, C, C++, Java, Kotlin, Go, OCaml | A character literal holding a double quote. | `let q = '"';` then `let s = "/*";` | Undercount: the misread string ends at the next `"`, which exposes `/*`. |
| Rust | Raw strings with three or more `#`, or ending in a backslash. | `r###"a "## /* "###`, `r"C:\"` | Undercount. |
| Swift | Raw strings, and lone quotes in multi-line strings. | `#"a "/*" b"#` | Undercount. |
| C++ | Raw strings with a custom delimiter. | `R"sql(a "/*" b)sql"` | Undercount. |
| C# | Raw string literals and nested strings in interpolation. | `"""a "/*" b"""`, `$"{(f ? "/*" : "")}"` | Undercount. |
| Go | Backtick raw strings. | `` s := `/*` `` | Undercount. |
| Java | Lone quotes in text blocks. | `"""` block holding `He said "hi` | Undercount. |
| JavaScript, TypeScript | Regular expression literals. | `p.replace(/\/*$/, "")`, `/"/` | Undercount. |
| OCaml | Quoted strings. | `{\|(*\|}` | Undercount. |

### Comment markers that are code

| Language | Construct | Probe | Effect |
| --- | --- | --- | --- |
| PHP | Attributes start with the `#` line comment marker. | `#[Attribute]` on its own line | Undercount: each attribute line is a comment. |
| F# | The multiplication operator section starts a block comment. | `List.fold (*) 1 xs` | Undercount. |

### Quotes declared where the text is not code

These are the same class as the Handlebars defect: an apostrophe in prose opens a string.

| Language | Construct | Probe | Effect |
| --- | --- | --- | --- |
| JSX, TSX, HTML, Vue | Apostrophes in element text. | `<p>Don't panic</p>` | Overcount: comments up to the next apostrophe count as code. |
| Ruby, Shell | Apostrophes in heredocs. | `Don't panic` inside `<<EOS` | Overcount. |
| YAML | Apostrophes in plain scalars. | `description: Don't panic` | Overcount. |

Python's nested f-string quotes rebalance within the line and were counted correctly.

## Upstream

The open pull requests [#1389](https://github.com/XAMPPRocky/tokei/pull/1389) for character literals and [#1351](https://github.com/XAMPPRocky/tokei/pull/1351) for Go backtick strings had no review on 2026-10-29; the repository has its issue tracker disabled. [Counting backends](research/counting-backends.md) compares ways to correct the defects in Locmetry.


No Tokei issue about Lean existed on 2026-10-28; the Lean definition on Tokei's master branch still declares no quotes. Locmetry pins Tokei exactly, so an upstream fix reaches Locmetry only with a dependency update, and several defects above, such as character literals and raw strings with any number of `#`, cannot be expressed in Tokei's quote model at all.
