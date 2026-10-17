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

## Distribution

Releases target macOS arm64 and Linux x86_64 and aarch64, licensed MIT. The installed executable depends only on operating-system libraries; Cargo dependencies are compiled code, not separate runtime packages.

Use `python3 scripts/release.py VERSION --publish` from a clean `main` checkout to prepare and request a release, for example `python3 scripts/release.py 0.4.1 --publish`. The [release command](../scripts/release.py) updates the crate, lockfile and README versions, runs the release checks sequentially, commits the version update, creates an annotated tag and atomically pushes both `main` and the tag. It requires the existing development tools (Python 3, Cargo, Bash and jq), GitHub access, the project origin and `mbackschat@mail.de` as the Git email. It refuses unrelated edits, conflicting tags, downgrades and a remote main that has not been integrated locally. Omitting `--publish` prepares and checks locally without committing or pushing; publication then repeats the checks.

If a check or push fails, fix the cause and rerun the same command. An exact version-update diff or a committed release can be resumed; existing tags are never moved. An already-pushed release is a no-op, not proof that publication succeeded. Check the [Release workflow](https://github.com/mbackschat/locmetry/actions/workflows/release.yml) and published assets before updating Homebrew. A failed GitHub workflow must be rerun in GitHub after diagnosing the failure; the local command does not retry it or update the separate tap repository.

Pushing a `v<version>` tag runs the [release workflow](../.github/workflows/release.yml). It requires the tag to match the crate version, tests, builds `aarch64-apple-darwin` and static `x86_64-unknown-linux-musl` and `aarch64-unknown-linux-musl` executables, measures with an empty environment, checks static linkage on Linux, and attaches the archives with `SHA256SUMS` to a GitHub Release. Each archive carries `THIRD-PARTY-NOTICES.md`, generated by [cargo-about](https://github.com/EmbarkStudios/cargo-about) from [the accepted licences](../packaging/licenses/about.toml) for exactly the crates that ship on the release targets; CI generates it on every push, so a dependency with another licence fails before a release. musl avoids a glibc version floor, and every target-side dependency is pure Rust, so no C cross toolchain is needed.

The [Homebrew formula](../packaging/homebrew/locmetry.rb) installs the release archive for the user's platform, so users need no Rust toolchain and the formula has no dependencies; only `--HEAD` builds the main branch from source with Rust as a build-only dependency. Homebrew-core would require a source build, so a submission there needs a different formula. The formula test measures a synthetic project with an empty `PATH` and asserts per-purpose code totals, complementing the empty-`PATH` integration test. For each release, run `packaging/homebrew/render-formula.sh VERSION`, which fills the version and the three archive checksums from the release's `SHA256SUMS`, and commit its output to `Formula/locmetry.rb` in the `mbackschat/homebrew-tap` repository. [Homebrew taps](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap), [formula tests](https://docs.brew.sh/Formula-Cookbook#add-a-test-to-the-formula), [dependency declarations](https://docs.brew.sh/Formula-Cookbook#specifying-other-formulae-as-dependencies)
