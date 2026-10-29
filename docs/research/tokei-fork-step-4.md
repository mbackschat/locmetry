# Tokei Fork Step 4 Measurement

Measured 2026-10-29 on macOS arm64, for step 4 of the [Tokei fork plan](../plans/tokei-fork.md): heredocs, whose closing line names a word taken from the opener. The fork is [mbackschat/tokei](https://github.com/mbackschat/tokei) at `ecc48f4`, version `15.0.0+locmetry.4`; its README cites the specification or reference documentation behind each declaration. The method and harness are those of [step 3](tokei-fork-step-3.md), with the tree-sitter-perl grammar 2.0.3 added and Perl's `heredoc_content` nodes read as literals.

## Change

The engine gained a `heredocs` declaration. An opener is a start such as `<<`, an optional flag that lets the closing line be indented, and a word, bare or quoted; the body runs from the next line, or from the end of the bodies of earlier heredocs on the same line, to a line holding the word. An opener counts only when such a line follows in the file, so a shift such as `1 << 2` or `x << y` stays code. Body lines, blank ones included, and the closing line count as code, as Tokei counts lines inside other literals.

| Language | Openers | Closing line |
| --- | --- | --- |
| Shell, Bash, Zsh, Korn shell | `<<WORD`, `<<-WORD`, `<<'WORD'`, `<<"WORD"` and `<<\WORD`, with optional spaces before the word; `<<<` here-strings are not heredocs | The word alone, after leading whitespace with `-`. |
| Ruby | `<<ID`, `<<-ID` and `<<~ID`, with the identifier or a quoted word directly after the opener | The word alone, indented only with `-` or `~`. |
| PHP | `<<<ID`, `<<<"ID"` and `<<<'ID'`, with optional spaces | The word, optionally indented and followed by anything but an identifier character, as PHP 7.3 and later allow. |
| Perl | `<<ID`, `<<~ID`, and quoted words that may follow a space | The word alone, indented only with `~`. |

POSIX allows only tabs before the closing word of `<<-`; the fork accepts spaces too. The measurement also showed that a Ruby regular expression may start a line, since a line break ends the statement before it, which the fork now reads.

Each form has a regression file in the fork, which passes `bash -n` and `zsh -n`, `ruby -c` or `perl -c`; the PHP file matches tree-sitter-php line by line.

## Corpora

The 34 repositories of steps 1 to 3 plus Mojolicious `4146e40`, Dancer2 `64218be` and ExifTool `9804093` for Perl.

## Results

Files whose counts differ from tree-sitter's, with the step-3 fork for comparison; the last column counts the fork's differences in files where tree-sitter reports a parse error:

| Language | Files | Tokei | Step 3 | Step 4 | Parse errors |
| --- | --- | --- | --- | --- | --- |
| PHP | 16,118 | 3,378 | 244 | 25 | 1 |
| Ruby | 1,794 | 413 | 392 | 3 | 0 |
| Shell | 994 | 81 | 49 | 11 | 7 |
| Perl | 606 | 230 | not measured | 220 | 2 |

The only file that tree-sitter and upstream Tokei agree on while the fork differs is Symfony's `ImportMapConfigReader.php`, whose heredoc holds a PHP file with a doc comment; tree-sitter-php reports a parse error and reads the heredoc as comments, and the fork's string lines are correct on inspection.

## Remaining Disagreements

- **PHP (25):** 17 Laravel `.blade.php` templates, which Tokei counts as Blade while the harness parses them as PHP; six Symfony scripts whose `#!/usr/bin/env php` line Tokei counts as a comment and tree-sitter as text; a template whose `//` lines lie in an HTML `<script>` element; and the parse error above.
- **Ruby (3):** RuboCop's percent literals `%w[' "]` and `%r{…}`.
- **Shell (11):** seven files with tree-sitter parse errors or misreadings, four of which step 3 inspected; and four heredocs whose first body line is blank, which tree-sitter-bash leaves outside the heredoc node.
- **Perl (220):** constructs other than heredocs. 206 of the files hold POD sections opened by `=head1`, `=item` or similar, which Tokei does not declare, since it reads only `=pod` … `=cut` as a comment; the others hold quote-like operators such as `q{…}`, `qw(…)` and `s{…}{…}`, regular expressions, or `__END__` sections. One heredoc in curl's `symbols.pl` has a blank first body line, which tree-sitter-perl, like tree-sitter-bash, leaves outside the node.

## Speed

Median instructions relative to upstream Tokei 15.0.0, from hyperfine on one thread, ten runs per repository:

| Repository | Step 3 | Step 4 |
| --- | --- | --- |
| tokio | 0.97 | 0.97 |
| dotnet/aspire | 0.86 | 0.86 |
| Prettier | 0.98 | 0.98 |
| RuboCop | 0.94 | 0.97 |
| Symfony | 0.91 | 0.92 |
| Laravel framework | 0.83 | 0.84 |
| acme.sh | 0.96 | 0.98 |
| Bash-it | 0.98 | 0.99 |
| Mojolicious | 0.83 | 0.85 |
| ExifTool | 0.85 | 0.90 |

The first build sought the end of the line at every `<` before matching the opener and reached 1.00 on Symfony. The Symfony, RuboCop, acme.sh and Bash-it values are from the corrected build; the other rows are from the first build, which did strictly more work per `<`.

## Assessment

Verified: on these repositories heredocs no longer cause disagreements in shell, Ruby and PHP, apart from tree-sitter's boundary for a blank first body line; the fork never disagrees where upstream agreed except where tree-sitter fails to parse; and speed stays within upstream's. Perl needs more than heredocs: its POD sections, quote-like operators and regular expressions remain misread in a third of the measured files.

Limits: one machine and one run of each corpus; the readings of tree-sitter's errors rest on inspection; and the requirement of a later closing line means a heredoc whose closer is missing is read as code rather than as a literal running to the end of the file.
