# Changelog

User-visible changes to Locmetry, in the [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) format. Versions follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html); before 1.0, a minor release may break commands, reports or configuration, and such entries are marked **Breaking**. [PRODUCT](docs/PRODUCT.md) owns the current contract. Releases before 0.7.0 are listed only as [tags](https://github.com/mbackschat/locmetry/tags).

## [Unreleased]

### Changed

- `--table-category` accepts an empty part as matching anything: `:*law` selects categories ending in `law` under any purpose, and `test:` selects all of Test. An entry whose purpose part can match no purpose, such as `*law` or `x*:law`, is now an error that suggests the `:CATEGORY` spelling, where a wildcard previously passed and emptied the tables. An entry that matches no category warns with the measured category names instead of pointing to `--show-categories`, whose table the same entry had emptied.
- Terminal tables, including comparison tables, leave a blank line between their last row and their legend.
- `locmetry --guide reports --section continuous-integration` includes a file-size gate by purpose, which was previously only in the repository's cookbook.
- **Breaking:** code, comment and blank counts change wherever Tokei misread a literal, because Locmetry now counts with its own fork of Tokei. A quote in a character literal such as `'"'`, a Go backtick string, a Java text block, `""` inside a C# or F# `@"…"` string, a raw string such as Rust `r##"…"##`, Swift `#"…"#`, C# `"""…"""`, C++ `R"x(…)x"`, Lua `[==[…]==]` or OCaml `{id|…|id}`, or a string in Haskell, Elm, PureScript, Agda, Racket, Scheme or Lisp no longer turns later comments into code or code into comments; Lua `--[==[…]==]` long comments of any level count as comments; PHP `#[` attribute lines count as code; and a line that starts inside a string counts as code even when it begins with `//`. Code inside string interpolation, such as JavaScript and TypeScript `${…}`, Kotlin, Scala and Dart `${…}`, Swift `\(…)`, C# `{…}`, Ruby `#{…}` and shell `"$(…)"`, is read as code with its own strings and comments, and JavaScript, TypeScript and Ruby regular expressions such as `/"/` no longer open strings. In shell scripts, `'…'` strings no longer end at a backslash, and `#` starts a comment only at the start of a word, so `${#a}` and `$#` are code. Shell, Ruby, PHP and Perl heredocs, such as `<<EOF`, `<<~EOS` or `<<<'EOT'`, are literals, so apostrophes and comment markers in their text no longer change the lines after them, and their lines count as code. Apostrophes and comment markers in the text of HTML, XML, SVG, XAML, Vue, Svelte and ERB files, between the tags of JSX elements in JavaScript, JSX and TSX files, and in YAML plain scalars and `|` or `>` block scalars no longer open strings or comments; lines inside a YAML block scalar count as code. Rust `///` and `//!` doc comments, doctests included, are comment lines, so an empty `///` line is no longer blank and Rust files no longer report a Markdown language. A line holding only comments, such as `/* a */ // b`, counts as a comment. A `<script>` or `<style>` element counts each of its lines once, and also ends at `</style >`, `</SCRIPT>` or an end tag with attributes. JavaScript's HTML-like `<!--` comments, and `-->` comments after nothing but comments on their line, are comments. YAML plain scalars that continue on later lines, and block scalars after `?` or at the document level, are text. Perl POD blocks such as `=head1`, data after `__END__` or `__DATA__`, quote-like operators such as `qw(…)` and `s{…}{…}`, regular expressions and string references such as `\"…"`, and Ruby percent literals such as `%w[…]` and data after `__END__`, are read as the languages define them. In Blade and Twig templates, apostrophes in text no longer open strings. Reports name the backend `tokei 15.0.0+locmetry.14 (embedded)`, so `compare` notes the changed counter between older and newer reports.
- Counting takes less time: measured on eight public repositories, from 4% to 32% less than before.
- A line holding code after or between comments, such as `/* a */ x; /* b */` or Lean's `-/ def x := 1`, counts as code; only lines that hold nothing but comments count as comments, except in PlantUML, whose block comments are whole lines. Not Quite Perl POD blocks and WenYan `注曰`, `疏曰` and `批曰` comments count as comments.
- A shebang line such as `#!/usr/bin/env node` counts as a comment in Rust, Kotlin, JavaScript, JSX, TypeScript and TSX files. Rust cargo-script frontmatter between `---` fences counts as code instead of being read as Rust, `#"` no longer opens a string, and Rust lines holding only Unicode whitespace such as U+2028 count as blank. Kotlin raw strings ending in more than three quotes, `$$"…"` strings and backtick identifiers holding quotes no longer turn the lines after them into code.
- Python code inside the `{…}` replacement fields of f-strings and t-strings is read as code with its own strings and comments, so a line in a multi-line replacement field that holds only a comment counts as a comment.

### Fixed

- Lean files no longer lose code lines to comment markers inside literals: a `/-` in a string such as `"packet/-receipt.json"` used to turn the rest of the file into comment lines. Lean lines inside strings, blank ones included, now count as code, and a Lean file whose bytes cannot be decoded is reported as an `invalid_encoding` parse failure instead of being counted.

## [0.11.2] - 2026-10-26

### Changed

- Root help names the two reports-guide sections that interpret counts and coverage, so reading the whole guide is no longer needed for that.
- **Breaking:** a project's language mappings share the ID space of overrides, rules, refinements and corrections, because reports now list them together; a mapping whose ID repeats one of theirs fails with the repeated-ID error. Personal mappings are unaffected.

### Fixed

- A language mapping that matches files the counter already recognizes, such as `*.spec` mapped to Plain Text while the counter reads RPM Specfile, no longer does nothing silently. `--verbose` and the Markdown report name it and point to `[[languages.corrections]]`, `summary.policy_entries` reports each project mapping with the files it counted and those the counter recognized (`builtin:recognition`), and a mapping that counts no file appears in `summary.unused_entries`.

## [0.11.1] - 2026-10-25

### Added

- The Markdown report, and `--verbose` category tables, list the meanings the policy declares in `[categories]` for the categories shown, so a shared report explains its own names. A workspace lists a meaning only when every member declaring the name gives the same one.
- Language tables explain rows with 0 files: their code is embedded in files of other languages, such as a script in an HTML file, and a legend line now names those rows.
- `locmetry compare` table and Markdown output add a Category table of the source categories whose lines or code changed, so a reclassification that moves code between categories within a purpose, such as a new refinement, is visible. Before, it left every human table unchanged.
- `--verbose` tables and the Markdown report name the `match` and `except` globs that did nothing in entries that still decide files, and the entries that lost matching files to an earlier entry of the policy, which an `except` list can make explicit. Before, a mistyped glob in a partly working entry showed only in the JSON `summary.policy_entries`.

### Changed

- The configure guide names where a workspace of independent repositories keeps each member's policy findings: `metadata.members[]`, not `summary`.
- Configuration errors name the likely fix: the field, value or language name meant (`json` → `JSON`, `tests` → `test`, `exclude` → `except`, `language_mappings` → `[[languages.mappings]]`), `[[rules]]` for a single `[rules]` table, the configuration version this release reads, and, for an unknown field, that a newer release may define it. They use TOML's words and single quotes, say whether an entry ID is empty, repeated or reserved, and name an invalid glob once with its entry.
- **Breaking:** without `--format` and `--output`, an option that shapes only one format selects it, on a terminal and in pipes: `--show-categories`, `--show-folders`, `--folder-rows`, `--verbose`, the bar options, the `--table-*` filters and `--tables-only` print a table, `--markdown-depth` Markdown and `--files` full JSON. Before, such an option in a pipe printed the JSON headline with a warning, which agents' output filters often hid. Options of different formats select none, and `--output` still saves full JSON.
- The category table's legend says it counts source files only, so data such as test fixtures is not mistaken for missing from a category.
- A configuration that uses a reference category name from `--guide categories` without declaring it in `[categories]` now fails with an error that says it is a reference name, what the reference uses it for and that it needs a declaration.
- The configure guide ends its refine-categories section with the checks of the whole policy, and states that the three purposes are fixed while categories carry the detail.

### Fixed

- A policy glob with an empty alternative, such as `run{,.cmd}`, now also matches the path without it, `run`, as `{a,b}` matching any listed alternative promises. Before, it silently matched only `run.cmd`, and the policy audit raised nothing because the glob still matched a file.
- The man page lists the possible values of an option on consecutive lines, as `--help` does, instead of with an empty line between each.

## [0.11.0] - 2026-10-25

### Added

- A `locmetry(1)` man page and shell completions: Homebrew and the Linux packages install the man page and the bash, zsh and fish completions, release archives include them, and `locmetry --generate KIND` prints each one (`man`, `complete-bash`, `complete-zsh`, `complete-fish`, `complete-powershell`) for other installations. The page shows every command's options and help examples, exit statuses, environment variables and files. Completions offer commands, options and, except in PowerShell, values such as `--format` and `--guide` topics.
- `locmetry --guide reference` states exit statuses, the environment variables and files Locmetry reads, and how to install the man page and completions, each as its own section.
- `--section ID` prints one section of a guide or schema instead of the whole document: `locmetry --guide configure --section refine-categories` prints that heading's section, and `locmetry --schema config --section Refinement` one definition as a JSON object keyed by the id, with `overview` for the schema's root description and section names. Repeat `--section` to read several sections of one topic in one command. Ids ignore case. `locmetry --guide index` routes each task to the sections and definitions it needs, and `--help` names the sections for purposes, rules and scope, for test or script categories and for category names. A section id is its heading's GitHub anchor and stays stable.
- Overrides and rules accept an `except` list of globs: files it names are left out of the entry and fall through to later entries and stages, so an exception such as `FieldAgreementTest` among `*AgreementTest` laws no longer depends on entry order.
- Reports list every override, rule and language correction in `summary.policy_entries` with the files and code it decided, the earlier entries or conventions that took some of its files, globs and exceptions that did nothing, and the decisions it replaced. An entry that decides only a few of its intended files, or moves production code into tests, now shows in the report instead of only an entry that decides nothing; `--verbose` notes entries that change a purpose a later stage would give.
- `[[refinements]]` in the project configuration give files a finer category without touching their purpose: a refinement applies to files whose decided purpose it names, such as conventional tests relabeled `property-law`, keeps purpose totals, module credit and `rule`, and leaves unclaimed and generated files alone. File records and `explain` name the refinement, and the configure guide now recommends refinements instead of overrides for category-only changes.
- `locmetry --guide categories` prints a non-binding category reference, so that projects, people and agents choose the same name for the same kind of code: names for each purpose in the order to try them, with what each excludes and the ad hoc names it replaces, worked cases, rules for project-specific names and conventions for reviewed policies. The configure guide, schema descriptions and README link to it.

## [0.10.0] - 2026-10-23

### Added

- Windows users can install and update Locmetry with Scoop from the `mbackschat/scoop-bucket` bucket; see [Windows and PowerShell](README.md#windows-and-powershell).
- Releases include `.deb` and `.rpm` packages for Linux x86_64 and aarch64, installed with `apt` or `dnf`; see [installation](docs/tokei-integration.md#installation).
- `--table-category` narrows tables to categories, as `--table-purpose` narrows them to purposes: `--table-category test,production:tooling` takes whole purposes or exact `purpose:category` keys, and `'production:*tooling*'` matches with wildcards. A `Not shown (categories)` note names what it leaves out, an entry that matches no category in the measurement warns, and Test/prod gives way to Comment % when only part of Production or Test is shown.

- Popular frameworks are classified without configuration: Next.js, Nuxt, React Router, .NET, Composer, Symfony, Laravel, Rails, RubyGems and Django projects are detected as build modules with the folders their documentation names. .NET test projects, RSpec files, Django `tests.py` and test-runner configuration such as `vitest.config.ts` are tests; and configuration of named JavaScript tools such as `vite.config.ts` is build tooling. Comparisons with earlier reports may show these files reclassified.
- Generated code inside build modules is recognized by documented markers and reported as `production:generated` instead of implementation: Go's `// Code generated … DO NOT EDIT.` line, .NET designer and generated file names and `<auto-generated>` comments, Java's `@Generated` on a top-level class, and `linguist-generated` declarations in `.gitattributes`. Totals, purpose and module credit stay the same, the largest-module-files review lists only hand-written files, `explain` shows the matched marker, and `classification.builtin_generated = false` turns the markers off. Comparisons with earlier reports may show these files' category changed.
- `[[languages.corrections]]` in the project configuration corrects a language the counter recognized wrongly, such as ProGuard rules read as Prolog or Helm templates read as Pan: a counter language counts the file with that language's syntax, and `"unrecognized"` leaves it uncounted and listed as unrecognized. File records name the correction as `language_correction`, `explain` shows it with its reason, and an unused correction is reported with the unused entries.

### Changed

- Six guide headings are shorter, so their section ids are too, such as `refine-categories` in the configure guide; links to their old GitHub anchors no longer jump to the heading.
- `--schema config`, `report` and `comparison` list the counter's language names once, shared by language mappings and corrections, which makes them 12.7 KB shorter each; they accept the same files as before.
- Tables and Markdown show BASH scripts in the Shell row, with a `Folded: BASH → Shell.` note, because the counter names extensionless bash scripts BASH and `.sh` scripts Shell whatever their shebang, which split one project's scripts over two rows. `--split-languages` shows them apart, and JSON keeps both names.
- Every `.locmetryrc` in the measured tree is excluded as policy, not only the loaded one, so trying a draft policy with `--config` or measuring a tree with nested projects no longer lists `.locmetryrc` files as unrecognized. The effective exclusions show `**/.locmetryrc` instead of `.locmetryrc`.
- **Breaking:** Go modules no longer count `vendor/` and `testdata/` folders or folders and files starting with `_` or `.` as production, because `go build ./...` skips them; vendored dependencies and test inputs are now reported as unclaimed. Projects with a committed `vendor/` folder show much less Go production code than before.
- **Breaking:** build modules are credited with the production source their layout covers, whichever stage decided it. An entry that keeps `production` and only names a category, such as `native` or `generated`, no longer removes the module's credit; one that changes the purpose still does. `summary.build_modules` and the `--verbose` note count only modules with credit, while the Markdown module list still shows every module. File records name the credited module as `module`, and `explain` shows it as `Module credit` when an entry decided the file. Comparisons with earlier reports show credit differences that come from this definition.
- `-h` gives each option and command a one-line summary, and `--help` keeps the full text: `locmetry -h` is 41% shorter and points to `--help` for more. The command list and the help footer no longer repeat what the commands' and options' own help says.
- `--guide configure` and `--schema config` explain when to classify verification code as test and when as production tooling: test runners, harnesses, graders and CI gates are tooling, and Test/prod depends on purpose, not category.
- Comparison tables and Markdown no longer list every folder and package of a moved, added or removed subtree: one finding names the outermost folder or workspace member with how many folders and packages it holds, and a side with more than 11 presence findings lists 10 and counts the rest. JSON still lists every finding.

### Fixed

- Errors for a `--guide` or `--schema` query combined with other arguments quote both flags, as other errors do, and name a directory argument in words.
- Errors about an unusable path name it as typed rather than as a canonical absolute path: workspace files, member roots and folders, a `--subfolder` that is not a directory, and an `--output` inside the scan root or over the workspace file. A root option given to a command, such as `workspace --show-folders`, now says it is only available without a command instead of suggesting an unrelated option.

## [0.9.0] - 2026-10-22

### Added

- Comparison JSON names its projection with `detail: "full"` or `"headline"`, as measurement JSON already does, so scripts can tell a full comparison from a headline and treat both document types alike.

### Changed

- Default tables name Rust `#[cfg(test)]` code counted with production in a legend line below the language table, such as “Production in 'Rust' includes 145 code lines of inline #[cfg(test)] tests in 3 files”, so it is visible where Test/prod is read. `--verbose` no longer repeats it as an `Inline tests:` note; Markdown keeps that note.
- Table notes and legends are easier to scan: languages, modules, folders, members and flags inside sentences are single-quoted, as in “'--files --output -' lists each file's kind”, separate facts are separate sentences instead of being joined by semicolons, and formula legends separate their formulas with “·”. The verbose note on module-claimed files now reads “Largest files counted as Production only by a build layout”, with each file's module named as “web (npm)”.
- Comparison notes, `explain` tables, warnings and errors follow the same wording: values and paths are single-quoted, the composition lines show an unchanged figure as `0` instead of `+0` or `-0`, comparison findings name their side as “(after)” rather than “(After)”, and `explain` gives a filter or parse-failure reason its own `Detail:` line. The human `message` texts of comparison findings and of notebook parse failures in JSON follow the same wording; finding codes, fields and exit statuses are unchanged.
- **Breaking:** errors and warnings on stderr start with `error:` or `warning:` instead of `locmetry:`, as argument errors already did, so their severity is readable without color.
- **Breaking:** workspace tables, Markdown reports and notes call each measured root or folder a member instead of a repository, since a workspace can also measure folders of one project: the table column reads “Member”, the Markdown section “Members” and its grids “Source categories by member” and “Source languages by member”.

### Fixed

- `explain` tables no longer print “1 lines” or “1 blanks”, and comparison coverage findings no longer say “1 unreadable files”: each count agrees with its unit.
- Comparison notes no longer say unreadable files "count as 0": they add nothing to the recorded counts, and their lines are unknown, not zero.
- Errors for a missing `explain` target, `--subfolder` folder or workspace member name the path or member, so a failure among several `explain` paths shows which one; in a workspace the error names the `MEMBER/PATH` as typed. Errors for a missing or unreadable scan root or workspace file name it too.
- An option spelling retired in 0.7.0, such as `--purpose` or `workspace --file`, is still rejected, but the error now names the option that replaced it instead of an unrelated one such as `--verbose` or `--files`.
- Command summaries in `locmetry --help` mention workspaces of one project's folders and that `explain` takes several paths.

## [0.8.0] - 2026-10-21

### Added

- Windows x86_64 support: releases include a zip with `locmetry.exe`, which needs no Visual C++ runtime, for use from PowerShell. Personal defaults live in `%APPDATA%\locmetry\config.toml`, roots show as ordinary drive paths, and colors work in the Windows PowerShell console or fall back to plain text. See [Windows and PowerShell](README.md#windows-and-powershell).
- `compare` reads reports saved with a UTF-8 or UTF-16 byte-order mark, as Windows PowerShell 5.1 writes them with `>`.
- `--tables-only` prints the tables without legends, notes or footer, for transcripts and scripts.
- Table and Markdown views count language variants, such as TSX, JSX and C++ headers, within their family's row, and a `Folded:` line names each fold. `--split-languages` shows them separately; JSON keeps every backend language.
- Reports name Rust `#[cfg(test)]` code inside production files as `inline_tests`, per file and per purpose, without moving it out of production. `--verbose` tables and Markdown reports add a note.
- The `--verbose` note on files that are not source code adds the code lines in their embedded snippets, such as fenced code in Markdown.
- A [cookbook](docs/cookbooks/file-size-gate.md) shows how to gate source file sizes by purpose with Locmetry's own measurement.

### Changed

- **Breaking:** table filter notes name only what they leave out, as `Not shown (purposes): …` and `Not shown (languages): …`, with the excluded code. Table footers no longer end with `; purposes: …` or `; languages: …`.
- The built-in `rust-test-module` convention classifies Rust `tests.rs` modules as tests. Comparisons with earlier reports may show these files reclassified.

### Fixed

- Comparison table group headers no longer end with trailing blanks.

## [0.7.6] - 2026-10-21

### Added

- Category rows in tables carry share bars, like language and folder rows.

### Removed

- The analytics cookbooks. Their [recreation design](docs/plans/analytics-cookbooks.md) names the commit that restores them.

## [0.7.5] - 2026-10-21

### Added

- `--table-language` filters tables by language, like `--table-purpose`. Each filter's note states the code it excludes, so the shown total plus the exclusions equals the measurement; JSON and Markdown stay complete.

## [0.7.4] - 2026-10-21

### Added

- Workspace files can name folders of one project with `project` and `folders`. Each folder is measured as `--subfolder` would measure it, under the project's single policy.
- Full reports name the five largest files that build-module layouts classified as production, as `summary.largest_module_files`, in the `--verbose` note and in Markdown, so generated or vendored code inside a module becomes visible.
- `explain` accepts several paths and prints one JSON line per path; a single path keeps its document.
- Dart packages (`pubspec.yaml`) are detected as build modules, and `*.e2e-spec.*` suites are classified as tests. Comparisons with earlier reports may show these files reclassified.

### Changed

- **Breaking:** measurement, comparison and explanation documents share `schema_version` `"3"`; comparison and explanation documents previously carried `"2"`. Schema-1 and schema-2 measurements are rejected as invalid reports.

## [0.7.3] - 2026-10-20

### Added

- Release archives and Homebrew install `compare-locmetry-files.sh`, an optional Bash helper that lists file-level changes between two full reports. It needs `jq`; Locmetry itself does not.

## [0.7.0] - 2026-10-20

### Changed

- **Breaking:** presentation options are named for what they show: `--folders`, `--categories`, `--bars`, `--purpose` and `--depth` became `--show-folders`, `--show-categories`, `--show-bars`, `--table-purpose` and `--markdown-depth`. `workspace --file` and `explain --workspace` became `--workspace-file`. The former spellings are rejected.
- `--files` adds per-file records to every full JSON view, whether saved or printed.

[Unreleased]: https://github.com/mbackschat/locmetry/compare/v0.11.2...HEAD
[0.11.2]: https://github.com/mbackschat/locmetry/compare/v0.11.1...v0.11.2
[0.11.1]: https://github.com/mbackschat/locmetry/compare/v0.11.0...v0.11.1
[0.11.0]: https://github.com/mbackschat/locmetry/compare/v0.10.0...v0.11.0
[0.10.0]: https://github.com/mbackschat/locmetry/compare/v0.9.0...v0.10.0
[0.9.0]: https://github.com/mbackschat/locmetry/compare/v0.8.0...v0.9.0
[0.8.0]: https://github.com/mbackschat/locmetry/compare/v0.7.6...v0.8.0
[0.7.6]: https://github.com/mbackschat/locmetry/compare/v0.7.5...v0.7.6
[0.7.5]: https://github.com/mbackschat/locmetry/compare/v0.7.4...v0.7.5
[0.7.4]: https://github.com/mbackschat/locmetry/compare/v0.7.3...v0.7.4
[0.7.3]: https://github.com/mbackschat/locmetry/compare/v0.7.0...v0.7.3
[0.7.0]: https://github.com/mbackschat/locmetry/compare/v0.6.0...v0.7.0
