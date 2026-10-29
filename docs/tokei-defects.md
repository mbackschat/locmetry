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
| JavaScript, TypeScript, Kotlin, Swift, C#, Scala, Dart, Ruby | Interpolation holes were read as string text, so a string or comment inside a hole, such as `` `${f("`")}` `` or `$"{(f ? "/*" : "")}"`, closed the outer string or opened a comment. | Undercount or overcount up to the next misread quote. | Fork `e778ec0` and `d77916d`. |
| JavaScript, TypeScript, Ruby | Regular expression literals such as `/"/` or `p.replace(/\/*$/, "")` were not literals. | Undercount or overcount. | Fork `e778ec0`, `d77916d` and `9c039dc`. |
| Ruby | The `$'`, `$"` and `` $` `` globals opened strings. | Overcount. | Fork `d77916d`. |
| Shell, Ruby, PHP, Perl | Heredoc bodies were read as code, so an apostrophe in text such as `Don't panic` opened a string and a `#` or `//` line counted as a comment. | Overcount or undercount up to the next misread quote. | Fork `785d277`. |
| Shell, Bash, Zsh, Korn shell | Quotes inside `"$(…)"` closed the outer string, a backslash before the closing `'` kept a `'…'` string open, and `#` inside a word, as in `${#a}`, opened a comment. | Undercount or overcount. | Fork `aaa4c32`. |
| All | A line whose text starts with a comment marker counted as a comment even when it started inside a literal, such as a `// …` line ending a raw string. | Undercount of one line per occurrence. | Fork `801aae6`. |

Locmetry adopted the fork's corrections in `1e8998c`, its raw-string corrections in `51d9664`, its interpolation, regular expression and shell corrections in `714f778`, and its heredoc corrections in `f9276a7`; the fork's [README](https://github.com/mbackschat/tokei#changes-from-upstream) cites the specification behind each declaration, and the [step 1](research/tokei-fork-step-1.md), [step 2](research/tokei-fork-step-2.md), [step 3](research/tokei-fork-step-3.md) and [step 4](research/tokei-fork-step-4.md) measurements compare the results with tree-sitter on public repositories. The Lean and Handlebars lexers keep Tokei's line classification, so only the lexing changes. One Tokei rule is deliberately kept: a line that starts inside a block comment is a comment even when code follows the comment's end.

## Uncorrected Defects

Found on 2026-10-28 with synthetic probes measured at `62dea4a`. Probes for Rust, Swift, C, C++, Java, JavaScript, Ruby and Shell were checked with the language's compiler or syntax checker; the others are unchecked against a toolchain. Where a row names several languages, those without a probe share a probed language's Tokei definition. In public repositories, the [step 3 measurement](research/tokei-fork-step-3.md) found apostrophes in JSX text, Ruby percent literals, JavaScript HTML-like comments and the line rule below, and the [step 4 measurement](research/tokei-fork-step-4.md) found Perl's POD sections and quote-like operators; the prevalence of the other rows has not been measured.

An undercount happens when a misread literal exposes a block-comment opener: the rest of the file, or everything up to the next closer, becomes comment. An overcount happens when a misread quote opens a string: comment and blank lines count as code up to the next matching quote. Either effect can follow from the same defect, depending on what comes next in the file.

### Literal forms Tokei does not declare

| Language | Construct | Probe | Effect |
| --- | --- | --- | --- |
| Ruby | Percent literals. | `%w[' "]`, `%r{/*}` | Undercount or overcount. |
| Perl | POD sections opened by a directive other than `=pod`, such as `=head1`; quote-like operators such as `q{…}`, `qw(…)` and `s{…}{…}`; regular expressions; `__END__` sections. | `=head1 Don't panic`, `qw(' ")` | POD lines count as code, and quotes inside the operators open strings. |
| JavaScript | HTML-like comments ([ECMA-262 Annex B.1.1](https://tc39.es/ecma262/2025/#sec-html-like-comments)). | `<!-- comment` | Comment lines count as code. |

### Quotes declared where the text is not code

These are the same class as the Handlebars defect: an apostrophe in prose opens a string.

| Language | Construct | Probe | Effect |
| --- | --- | --- | --- |
| JSX, TSX, HTML, Vue | Apostrophes in element text. | `<p>Don't panic</p>` | Overcount: comments up to the next apostrophe count as code. |
| YAML | Apostrophes in plain scalars. | `description: Don't panic` | Overcount. |

Python's nested f-string quotes rebalance within the line and were counted correctly.

### Line classification

| Language | Construct | Probe | Effect |
| --- | --- | --- | --- |
| Languages with block and line comments | A line holding only a block comment followed by a line comment counts as code, because Tokei's whole-line rule requires the line to end with the block comment's closer. | `/* c8 ignore next */ // @ts-expect-error` | Overcount of one line per occurrence. |

## Upstream

The open pull requests [#1389](https://github.com/XAMPPRocky/tokei/pull/1389) for character literals and [#1351](https://github.com/XAMPPRocky/tokei/pull/1351) for Go backtick strings had no review on 2026-10-29; the repository has its issue tracker disabled. [Counting backends](research/counting-backends.md) compares ways to correct the defects in Locmetry, and the fork's corrections are to be offered upstream as the [fork plan](plans/tokei-fork.md) describes.

No Tokei issue about Lean existed on 2026-10-28; the Lean definition on Tokei's master branch still declares no quotes. Character literals, and raw strings with any number of `#`, cannot be expressed in upstream Tokei's quote model at all, which is why the fork extends its engine.
