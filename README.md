# locmetry

`locmetry` measures codebases using Tokei, adding configurable test/production classification, exclusions, folder totals, agent-friendly JSON, and readable Markdown reports.

It is written in Rust and embeds Tokei 15.0.0 as a library. The installed executable needs neither Python, Git, a separate Tokei installation, nor a Rust runtime installation. Homebrew needs Rust only when building from source; bottles contain the compiled executable.

## Build and run

```sh
cargo build --release --locked
./target/release/locmetry --help
cargo test --locked
cargo fmt --all --check
cargo clippy --locked --all-targets -- -D warnings
```

Use a Rust toolchain and a platform C build toolchain for development.

## Usage

```sh
locmetry --root path/to/project                 # uses path/to/project/.locmetryrc if present
locmetry --root . --config shared.toml          # explicit configuration; the root stays `.`
locmetry --exclude 'generated/**' --hidden false --format markdown
locmetry --output /tmp/measurement.json         # headline on stdout, full report in one file
```

The scan root defaults to the current directory. Configuration is the TOML `.locmetryrc` contract in [PRODUCT.md](PRODUCT.md); without one, built-in defaults apply, including default exclusions and built-in test conventions. `--exclude`, `--hidden`, and `--respect-ignore-files` override project settings. JSON is the default stdout format; `--format markdown` produces human-readable output, with `--depth N` folder levels. Stdout JSON is a small headline; `--output FILE` writes the full report (summary, review list, and an aggregated breakdown table) to one file outside the scan root, `--output -` prints it instead, and `--files` adds per-file records. Exit code 2 marks a partial report with unreadable files. [PRODUCT.md](PRODUCT.md#reporting-and-coverage) documents the JSON schema. [Reports, limits, and private regression](poc/README.md) describes what a measurement contains.

## Documentation

- [Product vision and requirements](PRODUCT.md): scope, configuration contract, and guarantees.
- [Tokei integration and Homebrew distribution](docs/tokei-integration.md): embedded backend, dependencies, and packaging.
- [Research](docs/research/README.md): counter capabilities and classification findings.
- [Reports, limits, and private regression](poc/README.md): report contents, known limits, and the private regression.
- [Development plan](PLAN.md): remaining work and acceptance evidence.
- [Repository guidelines](CLAUDE.md): contributor conventions; `AGENTS.md` links to this file.

## Sensitive measurements

Keep measured-project identities, paths, rules, counts, raw statistics, and reports exclusively in Git-ignored `docs/research/local/`. Public documentation and tests use synthetic examples. The CLI accepts general output paths, so contributors must select this private directory explicitly; never force-add it.
