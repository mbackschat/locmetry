# Tokei Integration and Homebrew Distribution

Decision: implement Locmetry in Rust and embed the Tokei library to minimize installation dependencies. The PoC implements this boundary; the TOML configuration in [PRODUCT.md](../PRODUCT.md) remains a separate product requirement. Upstream references checked on 2026-10-15.

## Counting Boundary

```text
Locmetry profile + scan root
  → Locmetry selects files using root-relative scope and ignore rules
  → embedded Tokei recognizes languages and counts each selected file
  → Locmetry classifies, aggregates, and emits JSON or Markdown
```

[Cargo.toml](../Cargo.toml) pins Tokei 15.0.0 with `default-features = false`, disabling its default CLI feature. [Cargo.lock](../Cargo.lock) locks transitive dependencies. Cargo compiles the counter into Locmetry; there is no Tokei subprocess or runtime executable discovery. [Upstream crate manifest](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/Cargo.toml)

The adapter uses Tokei's `Languages::get_statistics` API. Locmetry first selects paths using the `ignore` crate with normal ignore-file behavior and a scan-root-based exclusion matcher. This avoids Tokei 15's exclusion builder being anchored to the current working directory. Tokei receives the selected paths with further ignore filtering disabled. [Backend implementation](../src/backend.rs), [upstream traversal](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/src/utils/fs.rs)

Use `Config::default()` with explicit counting settings, never `Config::from_config_files()`. Ambient `tokei.toml` and `.tokeirc` settings therefore cannot silently change counting. Repository, parent, and global ignore rules still participate in scope; this is not a hermetic filesystem snapshot. [Tokei configuration](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/src/config.rs)

Normalize original reports to root-relative paths and count direct statistics plus nested language blobs once. Aggregate `children` and `Total` views are not additional files. Backend identity, effective scope, classification rules, and the inaccuracy flag accompany the results. Traversal/configuration errors fail the command; a backend inaccuracy flag produces marked partial output and a nonzero exit. Unsupported file types remain a coverage limitation.

## Homebrew Dependencies

The intended formula has no runtime formula dependencies. Its build dependency and installation body are:

```ruby
depends_on "rust" => :build

def install
  system "cargo", "install", "--locked", "--path", ".", "--root", prefix
end
```

A source build requires the Rust and platform build toolchains. A bottle installation uses the compiled executable without installing Rust, Python, or Tokei for Locmetry. Cargo dependencies are compiled code, not separate Homebrew runtime packages; platform system libraries still apply. [Homebrew dependency declarations](https://docs.brew.sh/Formula-Cookbook#specifying-other-formulae-as-dependencies)

```text
Homebrew bottle: locmetry executable → operating-system libraries
Source build:   Rust/C toolchains + locked Cargo crates → same executable
```

Before distribution, inspect each platform binary's linkage and run a formula test that measures a synthetic source tree and asserts JSON counts/classification. The integration tests already exercise measurement with an empty `PATH`. A Tokei upgrade requires rebuilding and releasing Locmetry and rerunning counting regressions; it cannot arrive through an independent runtime Tokei upgrade.

Use a maintained tap for an initial release. A release URL/checksum, project license, supported-platform bottles, and complete formula are still needed. No tap or formula has been published. [Homebrew taps](https://docs.brew.sh/How-to-Create-and-Maintain-a-Tap), [formula tests](https://docs.brew.sh/Formula-Cookbook#add-a-test-to-the-formula)

## Migration Contract

The TOML/glob configuration replaced the earlier JSON/regex profiles after strict translations reproduced every reviewed classification; file and folder report fields are unchanged. Invalid configurations fail before counting. Content kind comes from reviewed data/configuration and documentation language lists for Tokei 15.0.0; every other recognized language counts as source.

JSON metadata is versioned as `poc-3`: it identifies the embedded engine, records a Unix timestamp, explicit scope, kind lists, and file coverage, and removes subprocess commands, Git revision/status, and captured subprocess stderr. Locmetry does not invoke Git. Private validation may compare working-tree status externally. Historical reports stay available in ignored local documentation.
