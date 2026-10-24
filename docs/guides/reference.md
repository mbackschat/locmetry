# Reference

The facts every task shares: what exit statuses mean, which environment variables and files Locmetry reads, and how to install its man page and shell completions. [PRODUCT](../PRODUCT.md) owns these guarantees; this page states them for reading. `locmetry --generate man` renders its first three sections into the man page.

## Exit Status

- `0`: the command completed. A measurement holds a report whose counts are all available, though some files may be unrecognized. `compare` used inputs without recorded inaccuracy or read failures. `explain` found every target counted, excluded, not selected, unrecognized or a symlink. Exit 0 does not prove complete language recognition or identical settings.
- `1`: the command failed: invalid arguments, configuration or input, or a traversal, arithmetic or output failure. No usable document is promised; the reason is on stderr.
- `2`: the command wrote a partial result. A measurement report has unavailable or inaccurate counts and records `metadata.backend_inaccurate`, a comparison used such an input, or an `explain` target was unread or failed to parse. Keep that status in scripts; missing lines are never estimated.

Help, version, guide, schema and generation queries exit 0 when they succeed. Diagnostics always go to stderr, so a pipeline should preserve the status before it queries JSON.

## Environment

Locmetry itself reads these variables; variables that only its libraries read are not listed.

- `NO_COLOR`: when set and not empty, turns off colors and emphasis, as `--no-color` does. Share bars and Markdown emphasis stay.
- `TERM`: `dumb` turns off colors and emphasis.
- `COLUMNS`: a positive integer sets the width for share bars in tables, saved ones included, and the width help wraps to. Otherwise the width of a stdout terminal applies, else 80.
- `XDG_CONFIG_HOME`: when it is an absolute path, personal defaults are read from `$XDG_CONFIG_HOME/locmetry/config.toml`, on every platform. A relative value is ignored.
- `HOME`: on macOS and Linux, personal defaults are otherwise read from `$HOME/.config/locmetry/config.toml`.
- `APPDATA`: on Windows, personal defaults are otherwise read from `%APPDATA%\locmetry\config.toml`, when it is an absolute path.

## Files

- `.locmetryrc` in the scan root: the project configuration, TOML. Only the scan root's file is read, never one in a subfolder or a parent.
- `--config FILE`: replaces that discovery with this file; the scan root stays as selected.
- Personal defaults: the first of `$XDG_CONFIG_HOME/locmetry/config.toml`, then `~/.config/locmetry/config.toml` on macOS and Linux or `%APPDATA%\locmetry\config.toml` on Windows. They only narrow scope and teach languages; `--no-user-config` ignores them, and `locmetry --schema user-config` describes them.
- Workspace files: read only when named with `--workspace-file FILE`, never discovered; `locmetry --schema workspace` describes them.
- Reports are written only where `--output` names a file. A measurement's file must lie outside the scan root, or outside every member of a workspace, and must not overwrite the configuration or workspace file; a comparison's must not overwrite either input.

## Man Page and Completions

`locmetry --generate KIND` prints a man page or a shell completion script on stdout and exits; it reads no configuration. The kinds are `man`, `complete-bash`, `complete-zsh`, `complete-fish` and `complete-powershell`. Homebrew and the Linux packages install the man page and the bash, zsh and fish completions, and the release archives include them. Otherwise install them yourself:

```sh
locmetry --generate man > ~/.local/share/man/man1/locmetry.1
locmetry --generate complete-bash > ~/.local/share/bash-completion/completions/locmetry
locmetry --generate complete-fish > ~/.config/fish/completions/locmetry.fish
locmetry --generate complete-zsh > ~/.zfunc/_locmetry
```

For zsh, add `fpath=(~/.zfunc $fpath)` before `compinit` in `~/.zshrc`. If `man locmetry` then finds no entry, as on macOS, add `export MANPATH="$HOME/.local/share/man:"` to your shell profile; the trailing colon keeps the system's pages. Completions offer commands, options and the values Locmetry knows, such as `--format` and `--guide` topics, but not section ids or category names.

On Windows, PowerShell completes commands and options, not values. Scoop installs no profile changes; add this line to the file `$PROFILE` names:

```powershell
locmetry --generate complete-powershell | Out-String | Invoke-Expression
```
