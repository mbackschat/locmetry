# Tokei Fork Step 5 Measurement

Measured 2026-10-29 on macOS arm64, for step 5 of the [Tokei fork plan](../plans/tokei-fork.md): quotes that open strings only inside tags or expressions, not in text. The fork is [mbackschat/tokei](https://github.com/mbackschat/tokei) at `95d1621`, version `15.0.0+locmetry.6`; its README cites the specification behind each declaration. The method and harness are those of [step 4](tokei-fork-step-4.md), with tree-sitter grammars for HTML 0.23.2, XML 0.7.0 and YAML 0.7.2 added, and YAML's quoted scalars and block scalars read as literals.

## Change

The engine gained four declarations:

| Declaration | Languages | Reading |
| --- | --- | --- |
| `quotes_in_tags` | HTML, XML, SVG, XAML, Vue, Svelte, Ruby HTML | Quotes open strings only inside a tag such as `<p class='a'>`; text between tags is text. |
| `jsx` | JavaScript, JSX, TSX | An element starts where an expression may start, when a closing tag or `/>` follows; its text is text, its attribute strings have no escapes, its `{…}` braces hold code, and its tag counts the angle brackets of TypeScript type arguments such as `<Item<T>>`. A type parameter such as `<T,>(x: T) => x` stays code. |
| `quotes_after` | YAML | A quote opens a string only at the start of a line or after `[`, `{`, `,`, `: `, `- ` or `? `, where a flow scalar can start; `''` inside `'…'` is a quote. |
| `block_scalars` | YAML | The lines after a `\|` or `>` indicator that are indented deeper than its node, and at least as deep as the first of them, are text; trailing blank lines are left out, as YAML's default chomping strips them. |

Lines inside a block scalar count as code, blank ones included, as Tokei counts lines inside other literals. Shell's word-start comment rule now takes its delimiters from `languages.json`.

The measurement found three of these rules: comments between JSX attributes, TypeScript type arguments in a tag, and a comment indented less than a block scalar's first line, which [YAML 1.2.2 §8.1.1.1](https://yaml.org/spec/1.2.2/#8111-block-indentation-indicator) ends the scalar at. Each has a regression file in the fork. The XML, YAML, JSX and TSX files match tree-sitter's code, comment and blank counts, and so does the HTML file, although tree-sitter-html reports a parse error in it.

## Corpora

The 37 repositories of steps 1 to 4 plus Bootstrap `0cc9057` and Hugo `c2f9cb8` for HTML, and Excalidraw `e7b5346` and Mastodon `0cdf6f8` for TSX, JSX and YAML.

## Results

Files whose counts differ from tree-sitter's, with the step-4 fork measured on the same repositories; the last column counts the fork's differences in files where tree-sitter reports a parse error:

| Language | Files | Tokei | Step 4 | Step 5 | Parse errors |
| --- | --- | --- | --- | --- | --- |
| HTML | 775 | 114 | 114 | 72 | 7 |
| XML | 4,938 | 6 | 6 | 0 | 0 |
| YAML | 3,398 | 252 | 252 | 51 | 4 |
| JavaScript | 9,768 | 180 | 102 | 91 | 83 |
| TSX | 1,063 | 3 | 2 | 0 | 0 |
| TypeScript | 2,951 | 35 | 2 | 2 | 0 |

Vue and Svelte have no grammar in the harness; their text follows the HTML rule.

## Remaining Disagreements

- **HTML (72):** 63 files with `<script>` or `<style>` elements, whose comments Tokei counts by the embedded language while tree-sitter-html reads element bodies as raw text, and in which a `<script>` element that closes on its opening line is counted twice, a defect upstream Tokei shares; two Prettier fixtures with template syntax; and seven parse errors.
- **YAML (51):** 44 files where tree-sitter counts the blank lines after a block scalar as part of it while the fork follows YAML's chomping; four parse errors; RuboCop's `.rubocop.yml`, whose plain scalars such as `["']` hold a quote after `[`, which the fork, like upstream, reads as a string; and two Prettier fixtures, an invalid document and a document-level scalar whose content starts in column 0.
- **JavaScript (91):** 83 parse errors; five files holding a block comment followed by a line comment, which Tokei's whole-line rule counts as code; two Prettier fixtures with HTML-like `<!--` comments; and one Closure typecast fixture.
- **TypeScript (2):** the Prettier fixtures of [step 3](tokei-fork-step-3.md#remaining-disagreements).

The fork differs from tree-sitter where upstream Tokei agreed in five files: the step-4 PHP file and one YAML file, both with tree-sitter parse errors; two acme.sh workflows whose last block scalar is followed by a blank line, where upstream matched because an apostrophe in the scalar left a string open to the end of the file; and Mastodon's `media_privacy_runner.rb`, whose `%r{/[^/]+\Z}` percent literal the step-3 regular expression rule reads as a regular expression starting at `/`, a [percent-literal defect](../tokei-defects.md#corrected-by-locmetry) present since step 3.

In the Immich v3.2.0 and v3.3.4 showcase, data code lines rise by 43 in both releases: GitHub workflow and issue-template YAML whose block scalars hold `#` lines and blank lines.

## Speed

Median instructions relative to upstream Tokei 15.0.0, from hyperfine on one thread, ten runs per repository, measured at `356c845`, before the block scalar indentation fix:

| Repository | Step 4 | Step 5 |
| --- | --- | --- |
| tokio | 0.97 | 0.98 |
| dotnet/aspire | 0.87 | 0.89 |
| Prettier | 0.98 | 0.99 |
| React | 0.90 | 0.99 |
| Excalidraw | 0.92 | 1.01 |
| Mastodon | 0.94 | 0.97 |
| Symfony | 0.92 | 0.94 |
| RuboCop | 0.97 | 0.99 |

JSX costs about a tenth more on JavaScript-heavy repositories than step 4, because every `<` is examined and a line inside an element cannot take the fast path. A first Mastodon run measured 1.04; two repeats measured 0.97 and 0.96, and no language filter of Mastodon exceeded 1.01.

## Assessment

Verified: on these repositories the fork matches tree-sitter for XML and TSX, removes four fifths of the YAML disagreements and a third of the HTML ones, and differs where upstream agreed only through tree-sitter parse errors, tree-sitter's block scalar boundary, or the Ruby percent-literal defect. Speed stays within run-to-run variation of upstream's.

Limits: one machine and one run of each corpus; the readings of the HTML and YAML remainders rest on inspection of samples; template languages such as Twig, Liquid, Blade, Mustache, Hamlet and Haml still declare quotes for the whole file; and a YAML quote after a flow indicator inside a plain scalar, as in `["']`, is still read as a string.
