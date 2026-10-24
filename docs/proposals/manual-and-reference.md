# Manual Page, Completions and Reference Topic

<!-- doc-status
{
  "status": "accepted",
  "date": "2026-10-24",
  "summary": "A man page and shell completions generated from the executable, and one reference topic for exit status, environment and files, so people and agents read the same facts in their usual places.",
  "evidence": "[second review and owner acceptance](manual-and-reference.md#review-history)"
}
-->

The owner raised the idea on 2026-10-24, comparing Locmetry's help with Homebrew's `tree`, and asked for a survey of what other tools do. People and agents both use Locmetry and count equally: a change must not make either one's reading worse to improve the other's.

## Problem

People on macOS and Linux look for `man locmetry` and shell completion; Locmetry ships neither. No recorded agent session used `man`, while agents read `--help`, and often guides, in every session. Some reference facts have no single home for either audience: exit status, the environment variables Locmetry reads and the files it looks for are spread over notes in `--help`, the guides and [PRODUCT](../PRODUCT.md#command-outcomes).

A man page written by hand would also be a third body of text beside `--help` and the guides, and a reader could not tell which one is current or which to read.

## Survey

Checked on 2026-10-24 with the tools installed on the owner's machine:

| Tool | Pattern | What it shows |
| --- | --- | --- |
| git | `git help -g` lists concept guides; commands are man section 1, guides section 7, and `git help X` shows either | One system answers both "what does this command do" and "how do I work with it" |
| cargo | `cargo help build` shows the man page, which is generated from the same Markdown as the web documentation; the same source also yields embedded plain text for platforms without `man` | One source, several renderings, and a portable text form beside `man` |
| ripgrep | `rg --generate man` and `rg --generate complete-zsh` (also bash, fish, PowerShell), listed in its help; Homebrew's formula runs them at install | The installed binary produces its own man page and completions, always matching its version |
| gh | `gh help environment`, `gh help exit-codes`, `gh help formatting` are topics beside the commands; `gh completion` is a visible command | Cross-cutting reference gets one topic each |
| kubectl | `kubectl explain type.field`, with nested paths and schema from the server | Schema documentation by path; Locmetry's `--schema … --section` is analogous for top-level properties and definitions |
| uv | `uv -h` and `uv --help` print the same 3 KB summary; `uv help` prints longer help, in a pager unless `--no-pager` | A short form under the flags and a longer one under a help command, the split Locmetry makes with `-h` and `--help`; nothing here shows that a pager would help or hurt Locmetry's readers |

The design follows the common rule: **one source, several renderings, and a split by question.** "What does this option do, and when does it apply?" is answered by `--help` and the man page, rendered from the same clap definitions. "How do I do this task?" is answered by the guides, through `locmetry --guide index`. The man page reuses the examples and notes `--help` already has, and no other workflow; it points to the guides for those. Ripgrep and gh show their generators in help; hiding Locmetry's is a deliberate departure, explained below.

## Design

### Required

**Reference topic.** A new guide, `locmetry --guide reference`, holds the facts every task shares, each in its own section, so people, agents and the man page read the same text:

- `exit-status`: what 0, 1 and 2 mean for measurement, `compare` and `explain`.
- `environment`: each environment variable Locmetry itself reads and what it changes: nonempty `NO_COLOR` turns off colors and emphasis; `TERM=dumb` does too; a positive integer in `COLUMNS` sets the width for share bars; `XDG_CONFIG_HOME`, `HOME` and `APPDATA` locate personal defaults. Variables read only inside dependencies are not listed.
- `files`: `.locmetryrc` in the scan root, `--config FILE`, and the personal defaults file: an absolute `XDG_CONFIG_HOME` wins on every platform, Windows included, followed by `~/.config` elsewhere and `%APPDATA%` on Windows; a relative `XDG_CONFIG_HOME` is ignored. Workspace files are read only when named with `--workspace-file`, never discovered.
- `man-page-and-completions`: the `--generate` kinds, and how to load each shell's completion, including a PowerShell profile line for Scoop users.

PRODUCT stays the owner of these guarantees; the guide is a concise reader's view, not a second exhaustive contract, and links to it as the other guides do. The `--help` automation note keeps its short summary of exit status; the reference guide's `exit-status` section is the full statement, and a guard checks that the two agree. The index routes to each section. The reference guide uses only headings, paragraphs, bullet lists, inline code, code blocks and links; the man page renders a link as its text.

**Generation query.** `locmetry --generate KIND` prints a generated document on stdout and exits:

| KIND | Output |
| --- | --- |
| `man` | The `locmetry(1)` man page in roff |
| `complete-bash`, `complete-zsh`, `complete-fish`, `complete-powershell` | A completion script for that shell |

It is a standalone query like `--schema`, with these rules:

- It accepts global `--no-color` and nothing else: roots, measurement and presentation options, commands, `--guide`, `--schema` and `--section` fail with exit status 1 before anything is written. `--section` is rejected explicitly, because the current standalone check accepts it for every query.
- It reads no configuration, personal defaults or files and scans nothing, so an invalid `.locmetryrc` or defaults file cannot make it fail.
- `--help` and `--version` keep their early exits, as they do for `--schema` and `--guide`.
- Success exits 0 with the document on stdout and nothing on stderr. A missing or unknown kind fails with exit status 1, empty stdout and the valid kinds. A failure to write stdout behaves as it does for the other queries.

It is hidden from `--help` and `-h`, so neither people nor agents pay reading cost for a packaging feature. Hidden does not mean unsupported: the kinds, streams and statuses are public, documented in PRODUCT, the README's installation section and the reference guide's `man-page-and-completions` section, which the index routes to for offline discovery. Removing or renaming a kind is a breaking change, and the feature gets an `Unreleased` entry.

**The man page.** One page, `locmetry(1)`, rendered from these sources only:

| Section | Source |
| --- | --- |
| NAME, SYNOPSIS, DESCRIPTION, OPTIONS | The root command's clap definitions: description, options with long help, value names and possible values |
| COMMANDS | `workspace`, `compare` and `explain`, each with its description, synopsis and options from its clap definition |
| EXAMPLES AND NOTES | Each command's long-help examples and notes, the `after_long_help` text, under that command: recipes, applicability and interpretation notes such as comparison input compatibility and output protection |
| EXIT STATUS, ENVIRONMENT, FILES | The matching sections of the reference guide |
| SEE ALSO | `locmetry --guide index` and every guide topic with its description, from the guide list |

Nothing in the page is written only for it, so no separately maintained text can drift from `--help` or the guides. `clap_mangen` renders one command per page and points to separate pages for subcommands; the combined page needs Locmetry's own composition, which replaces those pointers with the command sections and formats example blocks as roff examples. Shared sources do not stop a renderer from omitting or misrepresenting something; the guards and a rendering review check that.

**Completions.** Generated by `clap_complete` from the clap definitions. Hidden options, `--generate` among them, are left out of every script, filtered by Locmetry because the generators do not. What each shell completes:

| Shell | Commands | Options | Possible values (`--format`, `--guide`, `--schema` …) |
| --- | --- | --- | --- |
| bash, zsh, fish | Yes | Yes, per command | Yes |
| PowerShell | Yes | Yes, per command | No, as generated by `clap_complete`; documented in the reference guide |

Section ids and category names are not completed: section ids depend on the topic before them, and category names on the project's configuration.

**Distribution.** Each release job generates the files with the binary it packages; all release runners execute their own targets natively, and the Windows job's PowerShell script then carries the Windows help text.

| Channel | Files and destinations |
| --- | --- |
| macOS and Linux archives | `locmetry.1`, `complete/locmetry.bash`, `complete/_locmetry`, `complete/locmetry.fish` |
| Windows archive | `complete/_locmetry.ps1` |
| deb and rpm | `/usr/share/man/man1/locmetry.1`, `/usr/share/bash-completion/completions/locmetry`, `/usr/share/zsh/vendor-completions/_locmetry` (deb) or `/usr/share/zsh/site-functions/_locmetry` (rpm), `/usr/share/fish/vendor_completions.d/locmetry.fish` |
| Homebrew | At install, from the installed binary: `generate_completions_from_executable(bin/"locmetry", "--generate", shell_parameter_format: "complete-")` for bash, zsh and fish, and `(man1/"locmetry.1").write Utils.safe_popen_read(bin/"locmetry", "--generate", "man")` |
| Scoop | The archive's PowerShell script; the manifest's `notes` names the profile line in the reference guide. Nothing edits the user's profile. |

Homebrew installs no PowerShell completion: its helper would ask for `complete-pwsh`.

**Guards.** Public tests, run without external executables:

- For the man page and each completion script, the options listed per command equal that command's visible options in the clap tree, inherited globals included, with their value names and possible values where the shell's row in the completion table promises them; a command-scoped negative case, an option of one command absent from another's, is part of each check. The man page also shows each command's positional arguments with their syntax and descriptions; completion scripts are not required to name positional arguments their generators do not emit. SEE ALSO lists every guide topic.
- The reference guide's `environment` section names every variable Locmetry reads directly and through the defaults-file lookup on both platform branches, and nothing else.
- The reference guide stays within the Markdown subset above.
- The `--help` automation note and the `exit-status` section state the same statuses.
- `--generate` follows the query rules above: missing and unknown kinds, each conflict in either order, an invalid configuration and personal defaults in place, and `--help`'s precedence.
- The existing guide guards cover the new topic: its published ids, the index routes to its sections, and resolving references.

A designated Linux CI job also runs `mandoc -T lint` on the generated page and fails on any diagnostic; locally the check is optional.

### Excluded

- Guides as man pages, section 7 in git's manner (`man locmetry-configure`). Their tables, such as the category vocabulary and the index, need a Markdown-to-roff converter with table support; `--guide` remains their home. Reopen if people ask for `man` access to the guides, with a table-capable renderer checked on narrow terminals.
- Separate man pages per command; reopen if commands gain nested subcommands.
- Completing section ids or category names.
- Value completion in PowerShell beyond what `clap_complete` generates.
- Condensing option text in `--help` to one line each, with detail moved elsewhere. It would change what agents read and needs its own measured experiment.
- Changing the notes of `--help`, apart from rewording that offsets the new topic line.

## Effect on Each Audience

| Change | People | Agents |
| --- | --- | --- |
| Man page | `man locmetry` works offline in the usual place | No demonstrated benefit or regression: no recorded session used `man` |
| Completions | Commands, options and, outside PowerShell, values complete | No demonstrated benefit or regression |
| Reference topic | Exit status, environment, files and setup in one place | One section to request instead of searching guides; routed from the index |
| `--help` and the index | One more `--guide` topic line, offset by rewording | The topic line, any rewording and the new index rows change what they read |

## Acceptance

- The guards pass, and help stays within its budgets, offsetting the topic line by rewording; a reviewed raise only if rewording loses meaning. The first candidates are the `--format` value descriptions, repeated in both root listings; routing, automation and verification notes stay.
- The rendered page is reviewed on macOS and Linux with `man ./locmetry.1` for command grouping, examples and wrapped lists.
- A documented manual procedure, run before release and recorded with each shell's version, loads each completion script and completes a command, an option of that command, a possible value (except in PowerShell), and checks that an option of another command is not offered. CI runs the shells it has.
- A desk review under the [agent regression checks](../agent-regression-checks.md#desk-review) covers every help surface, the index rows and the reference guide and finds no correctness or routing regression. It decides whether targeted discovery or report tasks are needed; it does not establish that agent cost is unchanged.

## Risks

- **Help budget.** `--help` is at 13,005 of 13,010 bytes after the conditional index pointer was restored on 2026-10-24, so the `reference` topic line needs rewording elsewhere or a reviewed raise.
- **Rendering.** The combined page depends on Locmetry's composition of `clap_mangen`'s section renderers; long help with wrapped lists and example blocks may need adjustments to read well in roff, which presence guards cannot detect.
- **Platform text.** Help text differs by platform: it names `~/.config/locmetry/config.toml` on Unix and `%APPDATA%\locmetry\config.toml` on Windows. Man pages are packaged only for macOS and Linux, whose text agrees; FILES states every platform's path. Per-platform pages become necessary only if Unix platforms diverge.
- **Sequencing.** It changes neither guide sections nor configuration routes, but the reference topic and index rows change what agents read, so it lands after the guide-sections rerun and the owner's adoption decision establish the baseline.

## Review History

An independent review on 2026-10-24 accepted the draft with changes, and this revision adopts them: a per-shell completion matrix and an explicit policy for hidden options; man-page reuse of each command's notes, not only its examples, with the composition `clap_mangen` lacks; explicit `--section` rejection, help precedence, no configuration loading and a public contract for `--generate`, with offline discovery from the index; file names and destinations per channel and a PowerShell profile line instead of a README note; the defaults-file precedence on Windows and the conditions on `NO_COLOR` and `COLUMNS`; executable guards and a required `mandoc` check in CI; and narrower survey and audience claims. It confirmed that `clap_mangen` 0.3.3 and `clap_complete` 4.6.11 accept the locked clap 4.6.7 and fit the license allowlist. Release jobs generate their own files rather than one Linux job, because every runner executes its target natively and no artifact transfer is needed.

A second review the same day accepted the revision and asked for two wording corrections, both adopted: the uv row, whose `-h` and `--help` are identical, and the guards' separation of positional arguments, which the man page shows and completion scripts need not name. The owner accepted the proposal the same day; implementation follows the agents' context isolation, the guide-sections Amendment 1 rerun and the owner's adoption decision.

The implementation on 2026-10-24 departs from the text above in two details. `--generate` exists only in the command that parses the command line, not in the definitions the documents are generated from, so no filter is needed; the generation guards compare every script and the page with `--help` and would reject any hidden option that reached them. The CI lint fails on mandoc warnings and errors, not on its style notes, because `clap_mangen` writes every help text as one source line and mandoc notes each line longer than 80 bytes; the page repairs the two warnings the renderers produced. The manual checks then found two defects that the presence guards could not: zsh took a command name for the root's optional directory and completed root options after it, and groff hyphenated option names at line ends. The zsh script is therefore generated without the root positional, so the first word is the command, with a dispatch branch that completes root options after a directory; the page turns hyphenation off. Guards cover both.
