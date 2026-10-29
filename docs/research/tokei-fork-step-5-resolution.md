# Tokei Fork Step 5 Resolution

Measured 2026-10-29 on macOS arm64, after the [step 5 measurement](tokei-fork-step-5.md) left disagreements whose readings rested on samples. Every file whose fork counts differ from tree-sitter's is now either corrected in the fork, test-first, or explained file by file below, with a second reference wherever tree-sitter cannot parse the file. The fork is [mbackschat/tokei](https://github.com/mbackschat/tokei) at `5e86b70`, version `15.0.0+locmetry.10`; its README cites the specification behind each declaration.

## Corrections

Each correction has a regression file in the fork:

| Fork commit | Correction | Found in |
| --- | --- | --- |
| `5994462` | Rust `///` and `//!` doc comments are comment lines; Tokei re-parsed them as Markdown, which counted an empty `///` as blank and dropped a block's edge lines. | Rust files with doc comments |
| `5399062` | A line holding only comments, such as `/* a */ // b`, is a comment. | Five JavaScript files of step 5 |
| `418dd12`, `44f2a48`, `2220fce`, `61279ab` | An embedded `<script>` or `<style>` counts only the lines between its tags, and its tags are read as the HTML tokenizer reads them: case-insensitively, past a `>` inside a quoted attribute value, and with end tags such as `</style >` or `</script type="text/babel">`. | HTML files with one-line scripts, React's scheduler fixture, Immich's `TestWrapper.svelte` |
| `de1ab9e` | JavaScript's HTML-like comments, [ECMA-262 Annex B.1.1](https://tc39.es/ecma262/2025/#sec-html-like-comments). | Two Prettier fixtures |
| `a1a1886`, `13d1920` | YAML plain scalars that continue on later lines, quotes after `[`, `{` or `,` only inside flow collections, every line of a flow collection that spans lines, block scalars after `?`, `-` or at document level, keep chomping, and libyaml's indentation of leading empty lines. | RuboCop's `.rubocop.yml`, YAML test suites, and a flow collection found while profiling Mastodon's locale files |
| `d2b0180` | A markup tag that continues on later lines keeps its quoted attribute values. | HTML files |
| `ddbcd99`, `56379a6`, `daf1c1d` | Perl POD blocks, data after `__END__` and `__DATA__`, quote-like operators, regular expressions and `\"…"` string references; Ruby data after `__END__` and percent literals. | The 220 Perl files of step 4, three RuboCop files, one Mastodon file, Dancer2's skeleton `Entry.pm` |
| `598ce84` | Blade, Twig and Handlebars read quotes only in markup tags and skip template expressions inside a tag. | Nine Symfony Twig templates and one Prettier Handlebars file, checked against their blank-line counts because the harness has no template grammar |

`912b759` keeps the new code within Tokei's declared minimum Rust version, 1.71, and `d551ec3` keeps YAML text lines on the per-line fast path, as [Speed](#speed) records; neither changes a count in the corpora.

The reference was corrected too, so that the comparison judges the languages' definitions rather than tree-sitter's: `<script>` and `<style>` bodies, including `text/babel` and `text/jsx` scripts, are read with the JavaScript and CSS grammars; quoted attribute values are literals; POD runs to `=cut` or the end of the file; YAML block scalars follow their chomping indicator; a byte order mark is removed as Tokei's reader removes it; and Markdown children fold into comments, as Locmetry folds them. These are fork commits `d8a92bf`, `0150e2f`, `f10cc53` and `33710ed`.

## Corpora

The 40 repositories of the step 5 measurement, 69,295 files. That record counts 41, the 37 repositories of steps 1 to 4 plus four, but one of the four, Hugo `c2f9cb8`, is already among the 17 of [step 1](tokei-fork-step-1.md#method), so the corpus holds 40 distinct repositories. All 40 were cloned again at their recorded commits for the final fork, which counts every one of the 69,295 files as `296eae1` did.

## Second References

Where tree-sitter reports a parse error, or its grammar does not describe the file, another implementation of the language decided:

- JavaScript and TypeScript: [Babel](https://babeljs.io/docs/babel-parser) 7.28.5 with the Flow, JSX or TypeScript plugin, and [Hermes](https://github.com/facebook/hermes/tree/main/tools/hermes-parser/js) 0.36.1, React's Flow parser, for Flow syntax Babel rejects. Comments and string, template and regular expression tokens are painted, and lines classified by the harness's rules.
- Swift: the syntax map of the Xcode toolchain's `sourcekitd` (Apple Swift 6.4, `editor.open` with `enablesyntaxmap`, syntactic only), which the compiler's own parser produces. It equals the fork on all 773 Swift files of the corpora; tree-sitter equals it on 770.
- Shell: GNU bash 3.2.57. Each differing line is replaced by `&`, which cannot start a command; `bash -n` then reports a syntax error at that line exactly when the line is code rather than the text of a string or heredoc.
- YAML: the scanner of [PyYAML](https://pyyaml.org/) 6.0.2, which follows libyaml: scalar tokens are literals, block scalars are trimmed by their chomping, and `#` after whitespace outside scalars starts a comment.
- HTML: [parse5](https://github.com/inikulin/parse5) 7.3.0, an implementation of the WHATWG tokenizer and tree builder; comment nodes are comments and quoted attribute values literals. On the 533 script-free HTML files of the corpora it equals the fork in 526 and tree-sitter in 525. In the other seven, all Prettier fixtures, fork, upstream and tree-sitter agree: six hold CDATA sections or `<?…>` in HTML content, which the HTML standard turns into bogus comments while their authors write them as markup, and one repeats an attribute, whose location parse5 does not record.
- Minimal patches: the construct that stops the parser is rewritten without changing any line's class, and tree-sitter runs again. This settled Kotlin's soft keyword `set` and multi-dollar strings, Perl template placeholders and YAML 1.0 headers. The Perl patch exposed a fork defect, the reference backslash, rather than confirming the fork.

## Results

Files whose counts differ from tree-sitter's; the last column counts those among them in which tree-sitter reports a parse error:

| Language | Files | Upstream | Fork | Parse errors |
| --- | --- | --- | --- | --- |
| C | 1,285 | 10 | 1 | 1 |
| C# | 7,381 | 186 | 0 | 0 |
| C++ | 754 | 18 | 0 | 0 |
| Dart | 1,311 | 9 | 0 | 0 |
| F# | 146 | 8 | 0 | 0 |
| Go | 1,026 | 187 | 0 | 0 |
| Haskell | 366 | 2 | 2 | 1 |
| HTML | 775 | 70 | 1 | 1 |
| Java | 3,449 | 8 | 1 | 1 |
| JavaScript | 9,768 | 180 | 83 | 82 |
| Kotlin | 1,703 | 16 | 11 | 11 |
| Lua | 1,427 | 173 | 0 | 0 |
| OCaml | 1,896 | 31 | 0 | 0 |
| Perl | 606 | 389 | 0 | 0 |
| PHP | 16,118 | 3,378 | 25 | 1 |
| Python | 205 | 0 | 0 | 0 |
| Ruby | 5,149 | 461 | 0 | 0 |
| Rust | 1,053 | 542 | 0 | 0 |
| Scala | 835 | 0 | 0 | 0 |
| Shell | 995 | 81 | 6 | 6 |
| Swift | 773 | 27 | 3 | 2 |
| TSX | 1,063 | 3 | 0 | 0 |
| TypeScript | 2,951 | 35 | 2 | 0 |
| XML | 4,869 | 6 | 0 | 0 |
| YAML | 3,391 | 236 | 4 | 4 |

Three files count differently from upstream Tokei where upstream equals tree-sitter: Hugo's `code-toggle.html`, Symfony's `sfTests.yml` and `ImportMapConfigReader.php`. In all three tree-sitter reports a parse error, so its agreement with upstream is not evidence, and a reference that parses the file equals the fork:

- `code-toggle.html`: parse5 counts 110/0/9, as the fork does; upstream's 116/0/3 opens a string at the apostrophe of `draft='false` in text after `{{<`, which the HTML tokenizer reads as text.
- `sfTests.yml`: PyYAML, once the YAML 1.0 header `--- %YAML:1.0` is patched to `---`, reads its lines as the fork's 133/0/0; upstream's six blank lines are inside block scalars, whose text they are.
- `ImportMapConfigReader.php`: tree-sitter-php ends a heredoc body at a line opening with `/*`, which the [PHP manual](https://www.php.net/manual/en/language.types.string.php#language.types.string.syntax.heredoc) makes string text, since heredoc text behaves like a double-quoted string. With `/**` on line 118 patched to `/xx`, tree-sitter parses the file without error and counts 142/29/30, as the fork does; upstream counts the heredoc's 23-line doc comment as comments.

## Every Remaining File

Counts are code/comments/blanks.

| Language | File | Fork | tree-sitter | Upstream | Parse error | Verdict |
| --- | --- | --- | --- | --- | --- | --- |
| JavaScript | `prettier/tests/format/flow/flow-repo/ambient_declarations/use_variables.js` | 12/6/3 | 13/5/3 | 12/6/3 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `prettier/tests/format/flow/flow-repo/annot/annot.js` | 35/9/16 | 39/5/16 | 35/9/16 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `prettier/tests/format/flow/flow-repo/bounded_poly/test.js` | 20/2/3 | 22/0/3 | 20/2/3 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `prettier/tests/format/flow/flow-repo/function/rest.js` | 48/3/13 | 49/2/13 | 48/3/13 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `prettier/tests/format/flow/flow-repo/jsx_intrinsics.builtin/main.js` | 11/6/4 | 14/3/4 | 11/6/4 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `prettier/tests/format/flow/flow-repo/more_generics/poly.js` | 13/9/7 | 22/0/7 | 13/9/7 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `prettier/tests/format/flow/flow-repo/poly/implicit_bounded_instantiation.js` | 13/3/4 | 15/1/4 | 13/3/4 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `prettier/tests/format/flow/flow-repo/promises/promise.js` | 141/50/35 | 143/48/35 | 141/50/35 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `prettier/tests/format/flow/flow-repo/rec/test.js` | 10/1/5 | 11/0/5 | 10/1/5 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `prettier/tests/format/flow/flow-repo/type_param_scope/method_shadow.js` | 35/5/13 | 36/4/13 | 35/5/13 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `prettier/tests/format/flow/flow-repo/union/test.js` | 11/4/3 | 13/2/3 | 11/4/3 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `prettier/tests/format/flow/flow-repo/union/type-app.js` | 3/8/3 | 8/3/3 | 3/8/3 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `prettier/tests/format/flow/flow-repo/union_new/test10.js` | 20/29/23 | 21/28/23 | 20/29/23 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `prettier/tests/format/flow/flow-repo/union_new/test2.js` | 29/18/30 | 34/13/30 | 29/18/30 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `prettier/tests/format/flow/type-parameters/flow-only/const-type-params.js` | 8/2/1 | 9/1/1 | 8/2/1 | yes | Hermes equals the fork |
| JavaScript | `prettier/tests/format/js/comments-closure-typecast/no-semi/comments.js` | 11/3/4 | 10/4/4 | 11/3/4 |  | Fork correct: line 4 holds code between two comments, which the reference line rule calls a comment; Babel agrees except for that rule |
| JavaScript | `prettier/tests/format/js/cursor/comments-2.js` | 2/2/0 | 4/0/0 | 2/2/0 | yes | Prettier's `<\|>` cursor placeholder on line 1; no parser accepts the file; a non-blank, non-comment line is code |
| JavaScript | `prettier/tests/format/js/multiparser-graphql/invalid.js` | 18/2/2 | 17/2/3 | 18/2/2 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `prettier/tests/format/js/multiparser-invalid/text.js` | 28/0/3 | 31/0/0 | 28/0/3 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `prettier/tests/format/jsx/jsx-text/named-entries.jsx` | 6/1/1 | 7/0/1 | 6/1/1 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `react/flow-typed/environments/bom.js` | 2315/94/311 | 2329/80/311 | 2315/94/311 | yes | Hermes equals the fork |
| JavaScript | `react/flow-typed/environments/cssom.js` | 396/3/15 | 397/2/15 | 396/3/15 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `react/flow-typed/environments/dom.js` | 3150/307/222 | 3152/305/222 | 3150/307/222 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `react/flow-typed/environments/html.js` | 1470/71/169 | 1474/67/169 | 1470/71/169 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `react/flow-typed/environments/node.js` | 3745/269/275 | 3747/267/275 | 3745/269/275 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-client/src/ReactFlightClient.js` | 4721/797/260 | 4751/767/260 | 4721/797/260 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-client/src/ReactFlightReplyClient.js` | 1241/270/123 | 1300/224/110 | 1241/270/123 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-debug-tools/src/ReactDebugHooks.js` | 1170/162/120 | 1202/130/120 | 1170/162/120 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-devtools-cdt-mcp/src/DevToolsCdtMcp.js` | 341/37/18 | 363/15/18 | 341/37/18 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-devtools-extensions/src/contentScripts/hookSettingsInjector.js` | 57/5/8 | 58/4/8 | 57/5/8 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `react/packages/react-devtools-extensions/src/main/index.js` | 660/79/122 | 666/73/122 | 660/79/122 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `react/packages/react-devtools-facade/src/DevToolsFacadeTreeTools.js` | 642/139/56 | 667/114/56 | 642/139/56 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-devtools-shared/src/backend/fiber/renderer.js` | 6430/1076/576 | 6447/1059/576 | 6430/1076/576 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-devtools-shared/src/backend/legacy/renderer.js` | 1069/103/141 | 1074/98/141 | 1069/103/141 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-devtools-shared/src/backend/utils/index.js` | 205/34/26 | 209/30/26 | 205/34/26 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-devtools-shared/src/devtools/store.js` | 2021/216/348 | 2022/215/348 | 2021/216/348 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-devtools-shared/src/devtools/views/Components/TreeContext.js` | 914/118/99 | 916/116/99 | 914/118/99 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-devtools-shared/src/devtools/views/ErrorBoundary/cache.js` | 106/14/19 | 108/12/19 | 106/14/19 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-devtools-shared/src/devtools/views/Profiler/CommitTreeBuilder.js` | 431/41/84 | 457/36/63 | 431/41/84 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-devtools-shared/src/devtools/views/Profiler/ProfilerContext.js` | 424/64/63 | 426/62/63 | 424/64/63 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-devtools-shared/src/devtools/views/Profiler/SnapshotSelector.js` | 153/15/16 | 154/14/16 | 153/15/16 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-devtools-shared/src/dynamicImportCache.js` | 120/22/30 | 121/21/30 | 120/22/30 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-devtools-shared/src/hookNamesCache.js` | 180/22/32 | 181/21/32 | 180/22/32 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-devtools-shared/src/hooks/generateHookMap.js` | 57/56/12 | 63/50/12 | 57/56/12 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `react/packages/react-devtools-shared/src/hooks/getHookNameForLocation.js` | 177/69/28 | 184/62/28 | 177/69/28 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `react/packages/react-devtools-shared/src/inspectedElementCache.js` | 227/27/40 | 228/26/40 | 227/27/40 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-devtools-shared/src/utils.js` | 1094/115/117 | 1109/103/114 | 1094/115/117 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-dom-bindings/src/client/ReactFiberConfigDOM.js` | 5704/879/419 | 5784/834/384 | 5704/879/419 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-dom-bindings/src/events/ReactDOMEventReplaying.js` | 556/84/36 | 573/67/36 | 556/84/36 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-flight-server-fb/src/client/ReactFlightClientConfigBundlerFB.js` | 176/39/24 | 178/37/24 | 176/39/24 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-native-renderer/src/ReactFiberConfigFabric.js` | 756/89/115 | 759/86/115 | 756/89/115 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-native-renderer/src/legacy-events/ResponderTouchHistoryStore.js` | 190/27/20 | 194/23/20 | 190/27/20 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-reconciler/src/ReactChildFiber.js` | 1805/290/145 | 1807/288/145 | 1805/290/145 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-reconciler/src/ReactFiber.js` | 748/127/87 | 763/121/78 | 748/127/87 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `react/packages/react-reconciler/src/ReactFiberBeginWork.js` | 3380/815/313 | 3414/781/313 | 3380/815/313 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-reconciler/src/ReactFiberClassUpdateQueue.js` | 497/199/69 | 516/180/69 | 497/199/69 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-reconciler/src/ReactFiberHooks.js` | 4174/727/364 | 4210/691/364 | 4174/727/364 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-reconciler/src/ReactFiberLane.js` | 913/250/141 | 938/225/141 | 913/250/141 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `react/packages/react-reconciler/src/ReactFiberLegacyContext.js` | 251/38/35 | 257/32/35 | 251/38/35 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-reconciler/src/ReactFiberNewContext.js` | 454/121/52 | 495/80/52 | 454/121/52 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-reconciler/src/ReactFiberThenable.js` | 321/124/38 | 331/114/38 | 321/124/38 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-refresh/src/ReactFreshRuntime.js` | 543/133/57 | 552/124/57 | 543/133/57 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-server-dom-esm/src/ReactFlightESMNodeLoader.js` | 699/47/58 | 700/46/58 | 699/47/58 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `react/packages/react-server-dom-esm/src/client/ReactFlightClientConfigBundlerESM.js` | 163/35/16 | 164/34/16 | 163/35/16 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-server-dom-turbopack/src/ReactFlightTurbopackReferences.js` | 316/53/21 | 323/46/21 | 316/53/21 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-server-dom-unbundled/src/ReactFlightUnbundledNodeLoader.js` | 699/47/58 | 700/46/58 | 699/47/58 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `react/packages/react-server-dom-unbundled/src/ReactFlightUnbundledReferences.js` | 292/53/19 | 300/45/19 | 292/53/19 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-server-dom-webpack/src/ReactFlightWebpackNodeLoader.js` | 699/47/58 | 700/46/58 | 699/47/58 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `react/packages/react-server-dom-webpack/src/ReactFlightWebpackReferences.js` | 290/53/19 | 298/45/19 | 290/53/19 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-server/src/ReactFizzHooks.js` | 722/135/90 | 727/130/90 | 722/135/90 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-server/src/ReactFizzServer.js` | 5427/1023/394 | 5440/1010/394 | 5427/1023/394 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-server/src/ReactFizzThenable.js` | 220/84/19 | 228/76/19 | 220/84/19 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-server/src/ReactFlightHooks.js` | 133/20/17 | 134/19/17 | 133/20/17 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-server/src/ReactFlightReplyServer.js` | 1784/223/115 | 1807/200/115 | 1784/223/115 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-server/src/ReactFlightServer.js` | 5562/1077/387 | 5581/1058/387 | 5563/1076/387 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react-server/src/ReactFlightThenable.js` | 105/55/15 | 112/48/15 | 105/55/15 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react/src/ReactChildren.js` | 318/118/35 | 347/89/35 | 318/118/35 | yes | Hermes equals the fork |
| JavaScript | `react/packages/react/src/ReactLazy.js` | 192/53/16 | 195/50/16 | 192/53/16 | yes | Hermes equals the fork |
| JavaScript | `react/packages/scheduler/src/forks/Scheduler.js` | 446/112/55 | 449/109/55 | 446/112/55 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `react/packages/scheduler/src/forks/SchedulerMock.js` | 550/87/61 | 554/83/61 | 550/87/61 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `react/packages/scheduler/src/forks/SchedulerPostTask.js` | 175/32/32 | 176/31/32 | 175/32/32 | yes | Babel (Flow, JSX) equals the fork |
| JavaScript | `react/packages/shared/ReactTypes.js` | 375/38/66 | 384/29/66 | 375/38/66 | yes | Hermes equals the fork |
| JavaScript | `react/scripts/flow/environment.js` | 385/177/80 | 386/176/80 | 385/177/80 | yes | Hermes equals the fork |
| TypeScript | `prettier/tests/format/typescript/conditional-types/comments.ts` | 67/39/11 | 70/36/11 | 67/39/11 |  | Babel (TypeScript, JSX) equals the fork |
| TypeScript | `prettier/tests/format/typescript/union/consistent-with-flow/18647.ts` | 9/9/0 | 12/6/0 | 9/9/0 |  | Babel (TypeScript, JSX) equals the fork |
| PHP | `framework/src/Illuminate/Foundation/Exceptions/views/layout.blade.php` | 45/1/7 | 46/0/7 | 45/1/7 |  | Blade `{{-- --}}` and `<!-- -->` comment lines, which tree-sitter-php reads as text |
| PHP | `framework/src/Illuminate/Foundation/resources/exceptions/renderer/components/frame.blade.php` | 50/1/4 | 51/0/4 | 50/1/4 |  | Blade `{{-- --}}` and `<!-- -->` comment lines, which tree-sitter-php reads as text |
| PHP | `framework/src/Illuminate/Foundation/resources/exceptions/renderer/components/previous-exceptions.blade.php` | 81/4/6 | 85/0/6 | 81/4/6 |  | Blade `{{-- --}}` and `<!-- -->` comment lines, which tree-sitter-php reads as text |
| PHP | `framework/src/Illuminate/Foundation/resources/exceptions/renderer/components/query.blade.php` | 169/6/7 | 174/0/8 | 169/6/7 |  | Line 43 is a blank line inside a multi-line `x-data` attribute value, plus the Blade comment lines |
| PHP | `framework/src/Illuminate/Foundation/resources/health-up.blade.php` | 37/2/7 | 39/0/7 | 37/2/7 |  | Blade `{{-- --}}` and `<!-- -->` comment lines, which tree-sitter-php reads as text |
| PHP | `framework/src/Illuminate/Mail/resources/views/html/layout.blade.php` | 50/2/6 | 52/0/6 | 50/2/6 |  | Blade `{{-- --}}` and `<!-- -->` comment lines, which tree-sitter-php reads as text |
| PHP | `framework/src/Illuminate/Mail/resources/views/html/message.blade.php` | 20/4/3 | 24/0/3 | 20/4/3 |  | Blade `{{-- --}}` and `<!-- -->` comment lines, which tree-sitter-php reads as text |
| PHP | `framework/src/Illuminate/Mail/resources/views/text/message.blade.php` | 20/4/3 | 24/0/3 | 20/4/3 |  | Blade `{{-- --}}` and `<!-- -->` comment lines, which tree-sitter-php reads as text |
| PHP | `framework/src/Illuminate/Notifications/resources/views/email.blade.php` | 45/6/7 | 51/0/7 | 45/6/7 |  | Blade `{{-- --}}` and `<!-- -->` comment lines, which tree-sitter-php reads as text |
| PHP | `framework/src/Illuminate/Pagination/resources/views/bootstrap-3.blade.php` | 38/5/3 | 43/0/3 | 38/5/3 |  | Blade `{{-- --}}` and `<!-- -->` comment lines, which tree-sitter-php reads as text |
| PHP | `framework/src/Illuminate/Pagination/resources/views/bootstrap-4.blade.php` | 38/5/3 | 43/0/3 | 38/5/3 |  | Blade `{{-- --}}` and `<!-- -->` comment lines, which tree-sitter-php reads as text |
| PHP | `framework/src/Illuminate/Pagination/resources/views/bootstrap-5.blade.php` | 73/7/6 | 80/0/6 | 73/7/6 |  | Blade `{{-- --}}` and `<!-- -->` comment lines, which tree-sitter-php reads as text |
| PHP | `framework/src/Illuminate/Pagination/resources/views/semantic-ui.blade.php` | 28/5/3 | 33/0/3 | 28/5/3 |  | Blade `{{-- --}}` and `<!-- -->` comment lines, which tree-sitter-php reads as text |
| PHP | `framework/src/Illuminate/Pagination/resources/views/simple-bootstrap-3.blade.php` | 16/2/1 | 18/0/1 | 16/2/1 |  | Blade `{{-- --}}` and `<!-- -->` comment lines, which tree-sitter-php reads as text |
| PHP | `framework/src/Illuminate/Pagination/resources/views/simple-bootstrap-4.blade.php` | 24/2/1 | 26/0/1 | 24/2/1 |  | Blade `{{-- --}}` and `<!-- -->` comment lines, which tree-sitter-php reads as text |
| PHP | `framework/src/Illuminate/Pagination/resources/views/simple-bootstrap-5.blade.php` | 26/2/1 | 28/0/1 | 26/2/1 |  | Blade `{{-- --}}` and `<!-- -->` comment lines, which tree-sitter-php reads as text |
| PHP | `framework/src/Illuminate/Pagination/resources/views/tailwind.blade.php` | 95/5/11 | 100/0/11 | 95/5/11 |  | Blade `{{-- --}}` and `<!-- -->` comment lines, which tree-sitter-php reads as text |
| PHP | `symfony/.agents/skills/sync-translations/scripts/add_message.php` | 117/30/29 | 118/29/29 | 117/30/29 |  | Line 1 `#!/usr/bin/env php`, which the PHP CLI skips, counted as a `#` comment |
| PHP | `symfony/src/Symfony/Component/AssetMapper/ImportMap/ImportMapConfigReader.php` | 142/29/30 | 117/52/32 | 117/52/32 | yes | Lines 115-143 are a flexible heredoc whose doc comment is string content; with the `/**` that stops tree-sitter-php patched, tree-sitter equals the fork |
| PHP | `symfony/src/Symfony/Component/Emoji/Resources/bin/build.php` | 193/19/52 | 194/18/52 | 193/19/52 |  | Line 1 `#!/usr/bin/env php`, which the PHP CLI skips, counted as a `#` comment |
| PHP | `symfony/src/Symfony/Component/ErrorHandler/Resources/bin/extract-tentative-return-types.php` | 56/13/15 | 57/12/15 | 38/26/20 |  | Line 1 `#!/usr/bin/env php`, which the PHP CLI skips, counted as a `#` comment |
| PHP | `symfony/src/Symfony/Component/ErrorHandler/Resources/views/exception_full.html.php` | 67/2/9 | 69/0/9 | 67/2/9 |  | `//` comments inside an HTML `<script>` |
| PHP | `symfony/src/Symfony/Component/Mailer/Tests/Transport/Fixtures/fake-failing-sendmail.php` | 5/1/2 | 6/0/2 | 5/1/2 |  | Line 1 `#!/usr/bin/env php`, which the PHP CLI skips, counted as a `#` comment |
| PHP | `symfony/src/Symfony/Component/Mailer/Tests/Transport/Fixtures/fake-sendmail.php` | 3/1/1 | 4/0/1 | 3/1/1 |  | Line 1 `#!/usr/bin/env php`, which the PHP CLI skips, counted as a `#` comment |
| PHP | `symfony/src/Symfony/Component/Validator/Resources/bin/sync-iban-formats.php` | 107/30/44 | 108/29/44 | 107/30/44 |  | Line 1 `#!/usr/bin/env php`, which the PHP CLI skips, counted as a `#` comment |
| HTML | `hugo/docs/layouts/_shortcodes/code-toggle.html` | 110/0/9 | 116/0/3 | 116/0/3 | yes | parse5 equals the fork: `{{<` is text, so `draft='false` opens nothing |
| YAML | `symfony/src/Symfony/Component/Validator/Tests/Mapping/Loader/bad-format.yml` | 6/2/1 | 8/0/1 | 6/2/1 | yes | Invalid on purpose (tab indentation); `#` lines are comments |
| YAML | `symfony/src/Symfony/Component/Yaml/Tests/Fixtures/YtsFoldedScalars.yml` | 173/0/3 | 146/0/30 | 157/0/19 | yes | PyYAML equals the fork after `--- %YAML:1.0` is patched to `---` |
| YAML | `symfony/src/Symfony/Component/Yaml/Tests/Fixtures/YtsSpecificationExamples.yml` | 1624/14/28 | 1582/0/84 | 1553/69/44 | yes | PyYAML equals the fork after `--- %YAML:1.0` is patched to `---` |
| YAML | `symfony/src/Symfony/Component/Yaml/Tests/Fixtures/sfTests.yml` | 133/0/0 | 127/0/6 | 127/0/6 | yes | PyYAML equals the fork after `--- %YAML:1.0` is patched to `---` |
| Kotlin | `kotlinx.coroutines/kotlinx-coroutines-core/common/src/channels/Channel.kt` | 214/1261/53 | 781/711/36 | 214/1261/53 | yes | Every differing line is blank or a comment outside a raw string |
| Kotlin | `kotlinx.coroutines/kotlinx-coroutines-core/common/src/sync/Semaphore.kt` | 208/154/33 | 270/109/16 | 208/154/33 | yes | Every differing line is blank or a comment outside a raw string |
| Kotlin | `kotlinx.coroutines/kotlinx-coroutines-core/common/test/CancellableContinuationTest.kt` | 131/9/8 | 132/8/8 | 131/9/8 | yes | Every differing line is blank or a comment outside a raw string |
| Kotlin | `kotlinx.coroutines/kotlinx-coroutines-core/jvm/test/ThreadLocalsLeaksTest.kt` | 78/8/20 | 80/6/20 | 78/8/20 | yes | Every differing line is blank or a comment outside a raw string |
| Kotlin | `kotlinx.coroutines/kotlinx-coroutines-core/jvm/test/exceptions/StackTraceRecoveryTest.kt` | 226/4/32 | 231/4/27 | 226/4/32 | yes | Every differing line is blank or a comment outside a raw string |
| Kotlin | `kotlinx.coroutines/kotlinx-coroutines-test/common/src/TestBuilders.kt` | 259/332/19 | 260/331/19 | 259/332/19 | yes | Every differing line is blank or a comment outside a raw string |
| Kotlin | `kotlinx.coroutines/test-utils/jvm/src/FieldWalker.kt` | 154/15/11 | 159/12/9 | 154/15/11 | yes | Patched (`$$"…"` rewritten) tree-sitter equals the fork |
| Kotlin | `okhttp/okhttp/src/commonJvmAndroid/kotlin/okhttp3/internal/http2/Http2Stream.kt` | 505/150/87 | 590/72/80 | 505/150/87 | yes | Every differing line is blank or a comment outside a raw string |
| Kotlin | `okhttp/okhttp/src/commonJvmAndroid/kotlin/okhttp3/internal/http2/Settings.kt` | 69/41/23 | 88/22/23 | 69/41/23 | yes | Patched (`set` property renamed) tree-sitter equals the fork |
| Kotlin | `okhttp/okhttp/src/jvmTest/kotlin/okhttp3/CacheTest.kt` | 3813/190/257 | 4097/77/86 | 3813/190/257 | yes | Every differing line is blank or a comment outside a raw string |
| Kotlin | `okhttp/okhttp/src/jvmTest/kotlin/okhttp3/URLConnectionTest.kt` | 4009/193/342 | 4198/86/260 | 4009/193/342 | yes | Every differing line is blank or a comment outside a raw string |
| Shell | `acme.sh/deploy/windows_rdp.sh` | 113/30/15 | 111/30/17 | 92/44/22 | yes | `bash -n` supports the fork on every differing line |
| Shell | `acme.sh/dnsapi/dns_edgedns.sh` | 384/47/37 | 394/39/35 | 384/47/37 | yes | `bash -n` supports the fork on every differing line |
| Shell | `acme.sh/dnsapi/dns_servercow.sh` | 143/16/31 | 147/15/28 | 143/16/31 | yes | `bash -n` supports the fork on every differing line |
| Shell | `aspire/eng/build.sh` | 284/26/43 | 305/19/29 | 284/26/43 | yes | `bash -n` supports the fork on every differing line |
| Shell | `bash-it/plugins/available/colors.plugin.bash` | 162/2/38 | 194/2/6 | 162/2/38 | yes | `bash -n` supports the fork on every differing line |
| Shell | `nvm/nvm.sh` | 5825/418/499 | 5869/401/472 | 5850/406/486 | yes | `bash -n` supports the fork on every differing line |
| Swift | `swift-syntax/Tests/SwiftParserTest/ExpressionTests.swift` | 3870/51/256 | 3902/48/227 | 4037/32/108 | yes | The Swift toolchain syntax map equals the fork |
| Swift | `swift-syntax/Tests/SwiftParserTest/LexerTests.swift` | 1785/37/109 | 1861/31/39 | 1348/519/64 |  | The Swift toolchain syntax map equals the fork |
| Swift | `swift-syntax/Tests/SwiftSyntaxBuilderTest/StringLiteralExprSyntaxTests.swift` | 363/14/40 | 370/14/33 | 363/14/40 | yes | The Swift toolchain syntax map equals the fork |
| C | `fmt/include/fmt/format.h` | 3694/563/469 | 3696/562/468 | 3746/547/433 | yes | Error recovery around `FMT_PRAGMA_CLANG(...)` turns a blank line and a `///` comment into code |
| Java | `aspire/.github/workflows/polyglot-validation/Dockerfile.java` | 21/23/2 | 40/0/6 | 21/23/2 | yes | A Dockerfile named `.java`; Java lexing opens a comment at `lists/*` that runs to the end |
| Haskell | `pandoc/src/Text/Pandoc/Data.hs` | 216/32/8 | 218/30/8 | 216/32/8 |  | Haddock comments inside an `#else` branch, which tree-sitter-haskell reads as one CPP node |
| Haskell | `pandoc/src/Text/Pandoc/Readers/Markdown.hs` | 1984/221/210 | 1985/221/209 | 1984/221/210 | yes | tree-sitter-haskell lexes `'\n')` to `spnl'` as one character literal |

## Speed

hyperfine on one thread, ten runs per repository after two warm-up runs, upstream Tokei 15.0.0 against the fork at `d551ec3`, whose code `5e86b70` releases unchanged; wall time in milliseconds with its standard deviation, and median instructions relative to upstream:

| Repository | Upstream | Fork | Wall time | Instructions |
| --- | --- | --- | --- | --- |
| tokio | 48 ± 0 | 41 ± 0 | 0.84 | 0.88 |
| dotnet/aspire | 989 ± 41 | 836 ± 30 | 0.85 | 1.00 |
| Prettier | 303 ± 14 | 289 ± 2 | 0.95 | 1.01 |
| React | 397 ± 14 | 382 ± 27 | 0.96 | 1.03 |
| Excalidraw | 99 ± 3 | 90 ± 1 | 0.91 | 1.06 |
| Mastodon | 604 ± 19 | 544 ± 13 | 0.90 | 1.00 |
| Symfony | 1,541 ± 33 | 1,284 ± 34 | 0.83 | 0.94 |
| RuboCop | 131 ± 1 | 136 ± 9 | 1.04 | 1.06 |

With the commands run in reverse order, the wall-time ratios were 0.85, 0.85, 0.98, 0.92, 0.93, 0.89, 0.85 and 1.00, with instruction ratios within 0.04 of these. Thirty runs of RuboCop measured 131 ± 4 against upstream's 133 ± 6 and, reversed, 133 ± 7 against 134 ± 11, so its wall time is within upstream's variation; on every other repository the fork is faster.

Before, Mastodon executed 1.09 times upstream's instructions at `944191b`, and its 549 YAML files alone took 1,893M instructions at `296eae1` against upstream's 1,206M: a `[`, `{`, `|` or `>` anywhere in a line, as in a locale string holding `%{name}` or `<strong>`, sent the line from the per-line fast path to the per-byte loop, and the plain-scalar check scanned lines byte by byte. Since `d551ec3` a line leaves the fast path only where a flow collection or block scalar can start, and those lines are scanned with `memchr`; Mastodon's YAML now takes 1,225M instructions, with the same counts in every file of the corpora.

The remaining extra instructions on Excalidraw and RuboCop are the cost of reading what upstream misreads. Built without its JSX declaration, the fork takes 401M instructions on Excalidraw's TSX against upstream's 443M and its own 475M; built without Ruby's heredoc, percent literal, regular expression, interpolation and code token declarations, it takes 1,345M on RuboCop's Ruby against upstream's 1,337M and its own 1,410M. Wall time does not follow, because opening and reading files take most of a run: they were about two thirds of the samples in a profile of RuboCop's Ruby files.

## Showcase

In the Immich showcase, `web/src/lib/components/TestWrapper.svelte` loses one code line and one total line in both releases: version `.6` counted its script's opening line for Svelte and again for JavaScript. The comparison's deltas stay unchanged.

## Assessment

Verified: on these repositories every file whose fork counts differ from tree-sitter's has a verdict, from a second reference, a minimal patch, or the language's definition applied to the named lines; no file counts worse than upstream Tokei by any reference that parses it; and wall time stays within upstream's variation, with the extra instructions on Excalidraw and RuboCop attributed to the JSX and Ruby literals the fork reads.

Limits: one machine and one run of each corpus; the references' line rule calls a line that starts and ends in comments a comment even with code between them, which the closure typecast verdict corrects by inspection; template languages are checked by blank-line counts only; Liquid, Mustache, Hamlet and Haml still declare quotes for the whole file and are unmeasured; and a line such as `/* a */ x; /* b */` still counts as a comment, because Tokei's rule that a line opening with a block comment and ending with its closer is a comment stays for languages such as PlantUML, whose block comments are line-based.
