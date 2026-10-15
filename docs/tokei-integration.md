# Tokei Integration and Distribution

Decision: implement Locmetry in Rust and embed the Tokei library to minimize installation dependencies. Upstream references checked on 2026-10-15.

## Counting Boundary

```text
Configuration + scan root
  → Locmetry selects files using root-relative scope and ignore rules
  → embedded Tokei recognizes languages and counts each selected file
  → Locmetry maps unrecognized files, classifies, aggregates, and emits a terminal table, JSON, or Markdown
```

[Cargo.toml](../Cargo.toml) pins Tokei 15.0.0 with `default-features = false`, disabling its default CLI feature. [Cargo.lock](../Cargo.lock) locks transitive dependencies. Cargo compiles the counter into Locmetry; there is no Tokei subprocess or runtime executable discovery. [Upstream crate manifest](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/Cargo.toml)

The adapter uses Tokei's `Languages::get_statistics` API. Locmetry first selects paths using the `ignore` crate with normal ignore-file behavior and a scan-root-based exclusion matcher. This avoids Tokei 15's exclusion builder being anchored to the current working directory. Tokei receives the selected paths with further ignore filtering disabled. [Backend implementation](../src/backend.rs), [upstream traversal](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/src/utils/fs.rs)

Use `Config::default()` with explicit counting settings, never `Config::from_config_files()`. Ambient `tokei.toml` and `.tokeirc` settings therefore cannot silently change counting. Repository, parent, and global ignore rules still participate in scope; this is not a hermetic filesystem snapshot. [Tokei configuration](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/src/config.rs)

Normalize original reports to root-relative paths and count direct statistics plus nested language blobs once. Aggregate `children` and `Total` views are not additional files. Selected files Tokei does not recognize are counted with the parser of a matching language mapping, or else listed as coverage gaps. Traversal and configuration errors fail with exit code 1; a backend inaccuracy flag produces marked partial output and exit code 2.

A Tokei upgrade requires rebuilding and releasing Locmetry and rerunning counting regressions; it cannot arrive through an independent runtime Tokei upgrade.

## Distribution

Releases target macOS arm64 and Linux x86_64 and aarch64, licensed MIT OR Apache-2.0. The installed executable depends only on operating-system libraries; Cargo dependencies are compiled code, not separate runtime packages.

Pushing a `v<version>` tag runs the [release workflow](../.github/workflows/release.yml). It requires the tag to match the crate version, tests, builds `aarch64-apple-darwin` and static `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` executables, measures with an empty environment, checks static linkage on Linux, and attaches the archives with `SHA256SUMS` to a GitHub Release. musl avoids a glibc version floor, and every target-side dependency is pure Rust, so no C cross toolchain is needed.

The [Homebrew formula](../packaging/homebrew/locmetry.rb) installs the release archive for the user's platform, so users need no Rust toolchain and the formula has no dependencies; only `--HEAD` builds the main branch from source with Rust as a build-only dependency. Homebrew-core would require a source build, so a submission there needs a different formula. The formula test measures a synthetic project with an empty `PATH` and asserts per-purpose code totals, complementing the empty-`PATH` integration test. For each release, run `packaging/homebrew/render-formula.sh VERSION`, which fills the version and the three archive checksums from the release's `SHA256SUMS`, and commit its output to `Formula/locmetry.rb` in the `mbackschat/homebrew-tap` repository. [Homebrew taps](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap), [formula tests](https://docs.brew.sh/Formula-Cookbook#add-a-test-to-the-formula), [dependency declarations](https://docs.brew.sh/Formula-Cookbook#specifying-other-formulae-as-dependencies)
