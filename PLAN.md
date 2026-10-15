# Development Plan

[PRODUCT.md](PRODUCT.md) owns requirements, including optional and excluded scope; this file tracks remaining work and its completion evidence. Implemented work is recorded in Git history. Unchecked items are planned work, not supported features.

## Release for macOS and Linux — Next

Decided 2026-10-15: licence MIT OR Apache-2.0, repository `github.com/mbackschat/locmetry`, an own tap `mbackschat/homebrew-tap` before any homebrew-core submission, and targets macOS arm64 plus Linux x86_64 and aarch64. [Tokei integration and distribution](docs/tokei-integration.md#distribution) describes the release path.

- [x] Add licence files and Cargo metadata.
- [x] Format, lint, and test on all three targets in [CI](.github/workflows/ci.yml).
- [x] Build release archives on tags with an empty-environment measurement and, on Linux, a static-linkage check in the [release workflow](.github/workflows/release.yml).
- [x] Draft the [formula](packaging/homebrew/locmetry.rb): source build, Rust build-only, and a test that measures a synthetic project with an empty `PATH`. Its style check passes, and its test steps pass against a local build.
- [x] Create the GitHub repository, push, and confirm CI passes on both Linux targets (2026-10-15).
- [x] Tag `v0.1.0` and verify the release archives and `SHA256SUMS`; the downloaded macOS archive passes its checksum and measures with an empty environment (2026-10-15).
- [ ] Publish the rendered formula to the tap, then add Homebrew installation to the README.

**Completion evidence:** CI passes on every target, the release carries archives and checksums, `brew install` and `brew test` pass on macOS arm64 and Linux, and publication was explicitly authorized.

## Working Rules

Contributor commands and conventions are in [CLAUDE.md](CLAUDE.md). Run the [private regression](docs/measurement.md#private-regression) when measurement behavior changes, and keep this plan current as work lands.
