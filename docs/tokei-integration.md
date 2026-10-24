# Tokei Integration and Distribution

Decision: implement Locmetry in Rust and embed the Tokei library to minimize installation dependencies. Upstream references checked on 2026-10-15.

## Counting Boundary

```text
Configuration + scan root
  → Locmetry selects files using root-relative scope and ignore rules
  → embedded Tokei recognizes languages; supported counters measure readable content
  → Locmetry maps unrecognized files, classifies, aggregates, and emits a terminal table, JSON, or Markdown
```

[Cargo.toml](../Cargo.toml) pins Tokei 15.0.0 with `default-features = false`, disabling its default CLI feature. [Cargo.lock](../Cargo.lock) locks transitive dependencies. Cargo compiles the counter into Locmetry; there is no Tokei subprocess or runtime executable discovery. [Upstream crate manifest](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/Cargo.toml)

The adapter recognizes selected files with Tokei's `LanguageType::from_path` and counts them through its per-file `LanguageType::parse` API, retaining the local corrections described below. Locmetry first selects paths using the `ignore` crate with normal ignore-file behavior and a scan-root-based exclusion matcher. This avoids Tokei 15's exclusion builder being anchored to the current working directory. Selected files are parsed in parallel using the Rayon pool already used by Tokei, without the second filesystem traversal performed by `Languages::get_statistics`. Language totals and embedded reports retain Tokei's aggregation semantics. [Backend implementation](../src/backend.rs), [upstream traversal](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/src/utils/fs.rs)

Handlebars is counted by a [local lexical correction](../src/backend/handlebars.rs) before reports are normalized. Tokei 15.0.0's global quote handling mistakes apostrophes in template text for strings, corrupting subsequent blank and comment counts. The correction keeps Tokei's language recognition, report shape, BOM-aware decoding, and unreadable-file reporting; mapped Handlebars files use it too. Its syntax boundaries follow the [reference Handlebars lexer](https://github.com/handlebars-lang/handlebars.js/blob/v4.7.8/src/handlebars.l), with HTML comments and quoted attributes retained for HTML templates. [Measurement semantics](measurement.md#counting) defines the line categories. The decoder dependency is already in Tokei's dependency graph; no runtime executable is added.

Notebooks use a [local container decoder](../src/backend/notebook.rs), with Tokei counting code-cell text. Tokei 15's private notebook parser both overlaps parent/blob totals and discards decoding failures. Locmetry constructs disjoint cell contributions and reports malformed or unsupported content as unavailable. Recognized and mapped notebook paths share this boundary; [notebook semantics](measurement.md#notebooks) owns supported shapes, cell treatment and language selection.

Use `Config::default()` with explicit counting settings, never `Config::from_config_files()`. Ambient `tokei.toml` and `.tokeirc` settings therefore cannot silently change counting. Repository, parent, and global ignore rules still participate in scope; this is not a hermetic filesystem snapshot. [Tokei configuration](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/src/config.rs)

Normalize original reports to root-relative paths and count direct statistics plus nested language blobs once. Aggregate `children` and `Total` views are not additional files. Selected files Tokei does not recognize are counted with the parser of a matching language mapping, or else listed as coverage gaps. Traversal/configuration failures and backend inaccuracy follow the shared [command-outcome contract](PRODUCT.md#command-outcomes), including inaccuracy without identified paths.

A Tokei upgrade requires rebuilding and releasing Locmetry and rerunning counting regressions; it cannot arrive through an independent runtime Tokei upgrade.

## Installation

Install the prebuilt binary with `brew install mbackschat/tap/locmetry`, or download an archive and its `SHA256SUMS` from the [latest release](https://github.com/mbackschat/locmetry/releases/latest). Supported archive targets are `aarch64-apple-darwin`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl` and `x86_64-pc-windows-msvc` (a `.zip`). Linux binaries are static, and the Windows binary links the C runtime statically, so it needs no Visual C++ redistributable; macOS binaries use operating-system libraries. No separately installed runtime, Git or Tokei executable is needed to measure files.

For a manual download, replace `VERSION` with the release number without its `v` prefix and select your target. Extract the archive, then install the executable into a directory on your `PATH`:

```sh
version=VERSION
target=aarch64-apple-darwin
curl -fsSL "https://github.com/mbackschat/locmetry/releases/download/v$version/locmetry-v$version-$target.tar.gz" | tar -xz
mkdir -p ~/.local/bin
install "locmetry-v$version-$target/locmetry" ~/.local/bin/
install "locmetry-v$version-$target/compare-locmetry-files.sh" ~/.local/bin/
```

The archive also holds `locmetry.1` and, under `complete/`, the bash, zsh and fish completion scripts (`_locmetry.ps1` for PowerShell in the Windows zip); the [reference guide](guides/reference.md#man-page-and-completions) shows where each shell loads them from.

On Windows, `scoop bucket add mbackschat https://github.com/mbackschat/scoop-bucket` and then `scoop install mbackschat/locmetry` install it with [Scoop](https://scoop.sh), starting with 0.10.0; `scoop update locmetry` updates it. Without Scoop, in PowerShell, download and extract the zip, then add the folder holding `locmetry.exe` to your user `Path` once and open a new terminal:

```powershell
$version = "VERSION"
$name = "locmetry-v$version-x86_64-pc-windows-msvc"
Invoke-WebRequest "https://github.com/mbackschat/locmetry/releases/download/v$version/$name.zip" -OutFile "$name.zip"
Expand-Archive "$name.zip" -DestinationPath "$env:LOCALAPPDATA\Programs"
& "$env:LOCALAPPDATA\Programs\$name\locmetry.exe" --version
```

On Debian, Ubuntu, Fedora and other Linux distributions, releases after 0.9.0 also include `.deb` and `.rpm` packages for x86_64 (`amd64`) and aarch64 (`arm64`). They install `locmetry` and the helper into `/usr/bin`, the `locmetry(1)` man page and bash, zsh and fish completions, declare no dependencies because the binary is static, and suggest `jq` for the helper. They come from the release page, not a package repository, so install each new release the same way; `sudo apt remove locmetry` or `sudo dnf remove locmetry` uninstalls them.

```sh
version=VERSION
curl -fsSLO "https://github.com/mbackschat/locmetry/releases/download/v$version/locmetry_$version-1_amd64.deb"
sudo apt install "./locmetry_$version-1_amd64.deb"
# Fedora, RHEL and other RPM distributions:
sudo dnf install "https://github.com/mbackschat/locmetry/releases/download/v$version/locmetry-$version-1.x86_64.rpm"
```

Archives and Homebrew installations starting with 0.7.3 also supply `compare-locmetry-files.sh`, an optional executable Bash script for [reviewing file-level changes](guides/compare.md#inspect-configuration-transitions). Only this helper needs `jq`; install it separately with `brew install jq` on macOS, your system package manager on Linux, or `winget install jqlang.jq` on Windows, where the helper also needs Bash, for example from Git for Windows. If it is missing, the helper prints that remedy and exits 1. Homebrew installs the helper without declaring a `jq` dependency.

With a Rust toolchain, build a specific release from source by replacing `VERSION` in `cargo install --locked --git https://github.com/mbackschat/locmetry --tag vVERSION`. To build or install the current local checkout:

```sh
cargo build --locked
cargo install --locked --path .
locmetry --version
```

`cargo install` installs only the native binary. To install the helper from the checkout, run `install -m 755 scripts/compare-locmetry-files.sh ~/.cargo/bin/compare-locmetry-files.sh` (adjust the destination if you use a custom Cargo installation directory).

Checkout documentation describes that checkout; `locmetry --guide configure` and the other embedded guides describe the installed executable. Installing a checkout does not publish a release.

On Linux distributions with an older toolchain, such as Ubuntu 22.04, Homebrew also installs its own gcc and glibc for this formula; Locmetry itself is a static binary and uses neither.

## Distribution

Releases target macOS arm64, Linux x86_64 and aarch64, and Windows x86_64, licensed MIT. The native executable depends only on operating-system libraries; Cargo dependencies are compiled code, not separate runtime packages. Each archive also carries `CHANGELOG.md` and the optional Bash helper, covered by the archive checksum. The Linux `.deb` and `.rpm` packages hold the same static binary and files; their metadata is in [Cargo.toml](../Cargo.toml), read by [cargo-deb](https://github.com/kornelski/cargo-deb) and [cargo-generate-rpm](https://github.com/cat-in-136/cargo-generate-rpm). Release verification exercises it with the packaged binary and checks its missing-`jq` diagnostic.

Use `python3 scripts/release.py VERSION --publish` from a clean `main` checkout to prepare and request a release, for example `python3 scripts/release.py 0.4.1 --publish`. The [release command](../scripts/release.py) updates the crate and lockfile versions and the version output recorded in the [root command guide](cli/root.md) without editing README, moves the `Unreleased` entries of the [changelog](../CHANGELOG.md) under the new version and date, refusing a release without entries, runs the release checks sequentially, including the pinned replay of the [command guides](cli/README.md), commits the version update, creates an annotated tag and atomically pushes both `main` and the tag. It requires the existing development tools (Python 3, Cargo, Bash, jq and uv), GitHub access, the project origin and `mbackschat@mail.de` as the Git email. It refuses unrelated edits, conflicting tags, downgrades and a remote main that has not been integrated locally. Omitting `--publish` prepares and checks locally without committing or pushing; publication then repeats the checks.

If a check or push fails, fix the cause and rerun the same command. An exact version-update diff or a committed release can be resumed; existing tags are never moved. An already-pushed release is a no-op, not proof that publication succeeded. Check the [Release workflow](https://github.com/mbackschat/locmetry/actions/workflows/release.yml) and published assets before updating Homebrew. A failed GitHub workflow must be rerun in GitHub after diagnosing the failure; the local command does not retry it or update the separate tap repository.

Pushing a `v<version>` tag runs the [release workflow](../.github/workflows/release.yml). It requires the tag to match the crate version, tests, builds `aarch64-apple-darwin`, static `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl`, and `x86_64-pc-windows-msvc` executables, generates the man page and completion scripts with each packaged executable (bash, zsh and fish beside the Unix executables, PowerShell beside the Windows one), measures with an empty environment, checks static linkage on Linux and a static C runtime on Windows, builds the Linux packages and installs, runs and removes each in a clean Debian or Fedora container with [verify-packages.sh](../packaging/linux/verify-packages.sh), checks that the packages carry the man page and completions, and attaches the archives and packages with `SHA256SUMS` to a GitHub Release whose text is the version's changelog section, with repository links pinned to the tag. [.cargo/config.toml](../.cargo/config.toml) selects the static C runtime for Windows builds, including source builds. A pull request that changes the workflow builds and verifies every archive without publishing. Each archive carries `THIRD-PARTY-NOTICES.md`, generated by [cargo-about](https://github.com/EmbarkStudios/cargo-about) from [the accepted licences](../packaging/licenses/about.toml) for exactly the crates that ship on the release targets; CI generates it on every push, so a dependency with another licence fails before a release. musl avoids a glibc version floor, and every target-side dependency is pure Rust, so no C cross toolchain is needed.

The [Homebrew formula](../packaging/homebrew/locmetry.rb) installs the release archive for the user's platform, so users need no Rust toolchain and the formula has no dependencies; only `--HEAD` builds the main branch from source with Rust as a build-only dependency. Homebrew-core would require a source build, so a submission there needs a different formula. The formula test measures a synthetic project with an empty `PATH` and asserts per-purpose code totals, complementing the empty-`PATH` integration test. For each release, run `packaging/homebrew/render-formula.sh VERSION`, which fills the version and the three archive checksums from the release's `SHA256SUMS`, and commit its output to `Formula/locmetry.rb` in the `mbackschat/homebrew-tap` repository. Likewise, run `packaging/scoop/render-manifest.sh VERSION` and commit its output to `bucket/locmetry.json` in the [`mbackschat/scoop-bucket`](https://github.com/mbackschat/scoop-bucket) repository; the manifest's `checkver` and `autoupdate` entries also let Scoop's own tools find and hash a newer release. [Homebrew taps](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap), [formula tests](https://docs.brew.sh/Formula-Cookbook#add-a-test-to-the-formula), [dependency declarations](https://docs.brew.sh/Formula-Cookbook#specifying-other-formulae-as-dependencies)

### Man Page and Completion Check

Public tests check that the man page and every completion script list each command's options as `--help` does, and CI lints the page with `mandoc -T lint -W warning`. Whether a shell actually completes is checked by hand before a release whose commands, options or values changed, and recorded in the release's handback with each shell's version. Load each script from `locmetry --generate complete-SHELL` as the [reference guide](guides/reference.md#man-page-and-completions) describes, open a new shell and check four completions:

- `locmetry com` completes the command `compare`.
- `locmetry compare --m` offers `--member`, an option of that command.
- `locmetry compare --format ` offers `table`, `json`, `headline` and `markdown`; PowerShell completes no values.
- `locmetry compare --show-f` offers nothing, because `--show-folders` belongs to the root alone.

Also read the page with `man ./locmetry.1` on macOS and on Linux, where groff renders it rather than mandoc, for command grouping, example blocks and wrapped lists.
