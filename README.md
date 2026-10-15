# locmetry

`locmetry` measures codebases using Tokei, adding configurable test/production classification, exclusions, folder totals, agent-friendly JSON, and readable Markdown reports.

The PoC is written in Rust and embeds Tokei 15.0.0 as a library. The installed executable needs neither Python, Git, a separate Tokei installation, nor a Rust runtime installation. Homebrew needs Rust only when building from source; bottles contain the compiled executable.

## Build and run

```sh
cargo build --release --locked
./target/release/locmetry --help
cargo test --locked
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
```

Use a Rust toolchain and a platform C build toolchain for development. [PoC usage](poc/README.md) explains configuration and measurement commands. JSON is the default stdout format; `--format markdown` produces human-readable output. `--output PREFIX` saves JSON, Markdown, and backend statistics outside the scan root.

The current profiles use JSON and regular expressions. The agreed TOML/glob configuration, built-in test conventions, and broader default exclusions in [PRODUCT.md](PRODUCT.md) remain product requirements, not implemented CLI features.

## Documentation

- [Product vision and requirements](PRODUCT.md): scope, configuration contract, and guarantees.
- [Tokei integration and Homebrew distribution](docs/tokei-integration.md): embedded backend, dependencies, and packaging.
- [Research](docs/research/README.md): counter capabilities and classification findings.
- [Run the PoC](poc/README.md): profiles, reports, limitations, and private regression checks.
- [Repository guidelines](CLAUDE.md): contributor conventions; `AGENTS.md` links to this file.

## Sensitive measurements

Keep measured-project identities, paths, rules, counts, raw statistics, and reports exclusively in Git-ignored `docs/research/local/`. Public documentation and tests use synthetic examples. The CLI accepts general output paths, so contributors must select this private directory explicitly; never force-add it.
