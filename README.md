<img src="docs/assets/locmetry.webp" alt="locmetry logo" width="96">

# locmetry

`locmetry` measures codebases with embedded Tokei, adding configurable test/production classification, exclusions, language mappings for unrecognized files, folder breakdowns, agent-friendly JSON, and readable Markdown.

It is a single Rust executable with Tokei 15.0.0 compiled in, so it needs no Python, Git, separate Tokei installation, or Rust runtime. Releases target macOS arm64 and Linux x86_64 and aarch64.

## Build and run

```sh
cargo install --locked --path .   # or: cargo build --release --locked
locmetry --help
```

Development needs a Rust toolchain; [CLAUDE.md](CLAUDE.md) lists the test, format, and lint commands.

## Usage

```sh
locmetry --root path/to/project                 # uses path/to/project/.locmetryrc if present
locmetry --root . --config shared.toml          # explicit configuration; the root stays `.`
locmetry --exclude 'generated/**' --hidden false --format markdown
locmetry --output /tmp/measurement.json         # headline on stdout, full report in one file
```

Without a `.locmetryrc`, built-in exclusions, test and build conventions, and language mappings apply. Stdout is a compact JSON headline; `--output FILE` writes the full report with a review list and an additive breakdown table, and `--files` adds per-file records. Exit code 0 means a complete report, 2 a partial one with unreadable files, and 1 a failure. [PRODUCT.md](PRODUCT.md) specifies the configuration, options, and JSON schema.

## Documentation

- [Product vision and requirements](PRODUCT.md): scope, configuration contract, report schema, and guarantees.
- [Measurement semantics and limits](docs/measurement.md): selection and counting details, known limits, and the private regression.
- [Tokei integration and distribution](docs/tokei-integration.md): embedded backend, release binaries, and the Homebrew formula.
- [Research](docs/research/README.md): counter capabilities and classification findings.
- [Development plan](PLAN.md): remaining work and completion evidence.
- [Repository guidelines](CLAUDE.md): contributor conventions; `AGENTS.md` links to this file.

## License

Licensed under either the [Apache License, Version 2.0](LICENSE-APACHE) or the [MIT License](LICENSE-MIT), at your option.
