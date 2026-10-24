# Manual Page, Completions and Reference Topic

<!-- doc-status
{
  "status": "draft",
  "date": "2026-10-24",
  "summary": "A man page and shell completions generated from the executable, and one reference topic for exit status, environment and files, so people and agents read the same facts in their usual places.",
  "evidence": ""
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
| cargo | `cargo help build` shows the man page, which is generated from the same Markdown as the web documentation | One source, several renderings |
| ripgrep | `rg --generate man` and `rg --generate complete-zsh` (also bash, fish, PowerShell); Homebrew's formula runs them at install | The installed binary produces its own man page and completions, always matching its version |
| gh | `gh help environment`, `gh help exit-codes`, `gh help formatting` are topics beside the commands | Cross-cutting reference gets one topic each |
| kubectl | `kubectl explain type.field` | Schema documentation by path; covered for Locmetry by `--schema … --section` |
| uv | A 3 KB `--help` and `uv help <command>` with a pager | Not adopted: agent experiments found that Locmetry's longer help with examples and routes pays off |

The design follows the common rule: **one source, several renderings, and a split by question.** "What does this option do?" is answered by `--help` and the man page, rendered from the same clap definitions. "How do I do this task?" is answered by the guides, through `locmetry --guide index`. The man page contains no workflows; it points to the guides.

## Design

### Required

**Reference topic.** A new guide, `locmetry --guide reference`, holds the facts every task shares, each in its own section, so people, agents and the man page read the same text:

- `exit-status`: what 0, 1 and 2 mean for measurement, `compare` and `explain`.
- `environment`: every environment variable Locmetry reads (`NO_COLOR`, `COLUMNS`, `TERM`, `XDG_CONFIG_HOME`, `HOME`, `APPDATA`) and what it changes.
- `files`: `.locmetryrc` in the scan root, `--config FILE`, the personal defaults file per platform, and workspace files.

PRODUCT stays the owner of these guarantees; the guide states them for readers and links to it, as the other guides do. The index routes to each section. The reference guide uses only headings, paragraphs, bullet lists, inline code and code blocks, so the man page can render it.

**Generation query.** `locmetry --generate KIND` prints a generated document on stdout and exits, as a standalone query like `--schema`:

| KIND | Output |
| --- | --- |
| `man` | The `locmetry(1)` man page in roff |
| `complete-bash`, `complete-zsh`, `complete-fish`, `complete-powershell` | A completion script for that shell |

It is hidden from `--help` and `-h`, so neither people nor agents pay reading cost for a packaging feature, and PRODUCT and the README's installation section document it. It follows the standalone-query rule of `--schema` and `--guide`: it accepts global `--no-color`, conflicts with `--guide`, `--schema` and `--section`, and rejects roots, measurement and presentation options and commands with exit status 1 before writing anything. Success exits 0 with the document on stdout and nothing on stderr. An unknown kind fails with exit status 1, empty stdout and the valid kinds.

**The man page.** One page, `locmetry(1)`, rendered from these sources only:

| Section | Source |
| --- | --- |
| NAME, SYNOPSIS, DESCRIPTION, OPTIONS | The clap definitions of `--help`, long help texts included |
| COMMANDS | Each of `workspace`, `compare` and `explain` with its synopsis and options, from its clap definition |
| EXAMPLES | The examples of `--help` |
| EXIT STATUS, ENVIRONMENT, FILES | The sections of the reference guide |
| SEE ALSO | `locmetry --guide index` and every guide topic with its description, from the guide list |

Nothing in the page is written only for it, so no separately maintained text can drift from `--help` or the guides. Shared sources do not stop a renderer from omitting or misrepresenting something; the guards check that.

**Completions.** Generated from the clap definitions: commands, options, and the values clap knows, such as `--format`, `--guide` and `--schema` topics. Section ids are not completed.

**Distribution.**

- Homebrew installs the man page and completions at install time from the installed binary, as its ripgrep formula does: `generate_completions_from_executable(bin/"locmetry", "--generate", shell_parameter_format: "complete-")` and `(man1/"locmetry.1").write Utils.safe_popen_read(bin/"locmetry", "--generate", "man")`.
- Release archives for macOS and Linux and the deb and rpm packages include `locmetry.1` and the completion scripts, generated once per release by a Linux build.
- Windows has no man page; the README names `locmetry --generate complete-powershell` for PowerShell profiles.

**Guards.**

- Every environment variable the source reads appears in the reference guide's `environment` section.
- The reference guide stays within the Markdown subset the man renderer supports.
- `--generate man` lists, within each command's own section, exactly that command's visible options with their value names and possible values, and every guide topic in SEE ALSO; it renders without roff warnings where `mandoc -T lint` is available, which public tests do not require.
- Each completion script offers, for each command, exactly that command's visible options and the possible values clap knows for them.
- `--generate` follows the standalone-query rule, with the exit statuses above.

### Excluded

- Guides as man pages, section 7 in git's manner (`man locmetry-configure`). Their tables, such as the category vocabulary and the index, need a Markdown-to-roff converter with table support; `--guide` remains their home. Reopen if people ask for `man` access to the guides.
- Separate man pages per command.
- Completing section ids, which depend on the topic given before them.
- Condensing option text in `--help` to one line each, with detail moved elsewhere. It would change what agents read and needs its own measured experiment.
- Changing the notes of `--help`. Its automation note keeps its short summary of exit status; the reference guide holds the full statement.

## Effect on Each Audience

| Change | People | Agents |
| --- | --- | --- |
| Man page | `man locmetry` works offline in the usual place | No change: no recorded session used `man`, and its text equals `--help` and the reference guide |
| Completions | Commands, options and values complete in bash, zsh, fish and PowerShell | No change |
| Reference topic | Exit status, environment and files in one place | One section to request instead of searching guides; routed from the index |
| `--help` | One more `--guide` topic line | About 70 bytes more per session, unless offset by rewording |

## Acceptance

- The guards pass, and help stays within its budgets, offsetting the topic line by rewording or through a reviewed raise.
- `man ./locmetry.1` renders on macOS and Linux, and the completion scripts load and complete a command, an option and a possible value in bash, zsh, fish and PowerShell. Checked by a documented manual procedure on each platform before release; CI runs the shells it has.
- A desk review under the [agent regression checks](../agent-regression-checks.md#desk-review) finds no correctness or routing regression. Help changes only by a topic line, so no agent run is planned; the desk review decides whether one is needed.

## Risks

- **Help budget.** `--help` is at 12,947 of 12,950 bytes after the guide-sections amendment, so the `reference` topic line needs rewording elsewhere or a reviewed raise.
- **Rendering.** `clap_mangen` renders one command per page; composing the commands into one page uses its section renderers directly. Long help texts with wrapped lists may need small adjustments to render cleanly in roff.
- **Platform text.** Help text differs by platform: it names `~/.config/locmetry/config.toml` on Unix and `%APPDATA%\locmetry\config.toml` on Windows. Every packaged page is generated on Linux, so it names the Unix path in its options; its FILES section, from the reference guide, states every platform's path. `--generate man` on Windows produces the Windows text, which no package installs.
- **Sequencing.** It changes neither guide sections nor configuration routes, so it does not confound the pending guide-sections rerun, but it should land after that run so the run measures one change.
