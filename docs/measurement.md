# Reports, Limits, and Private Regression

[README](../README.md) shows CLI usage, and [PRODUCT.md](../PRODUCT.md) defines the TOML configuration. This page describes what a measurement contains, its known limits, and the opt-in regression against private reference projects. The earlier JSON/regex profiles are retired: strict TOML translations reproduced every reviewed classification, and `--config` rejects `.json` files with a migration hint.

## Run from the repository root

Keep real-project configurations and outputs exclusively in Git-ignored `docs/research/local/`:

```sh
cargo build --release --locked
./target/release/locmetry --root /path/to/project --config docs/research/local/project.locmetryrc --output docs/research/local/measurement.json
```

The configuration path above is a placeholder, not a bundled project. Each run writes exactly one report file at the given path, replacing it; `--files` adds per-file records, and `--format markdown` prints the human report to stdout. Locmetry does not modify the measured project or run its builds or tests.

## Measurement Contract

- Selection: untracked nonignored files are included. With ignore files respected, `.ignore` and `.tokeignore` files always apply and `.gitignore` applies only inside a Git work tree. Symbolic links are not followed. Exclusions are checked again against the backend response so excluded files cannot leak into counts.
- Paths: rules match root-relative `/`-separated paths. Components come from the platform's path parser, so Unix file names may contain `:` or `\`.
- Purpose (`production`, `test`, `unclassified`) stays separate from content kind (`source`, `data`, `documentation`, `other`). Production means maintained non-test source, including separately categorized tooling and examples, not just shipped runtime code.
- Content kind comes from the Tokei language: reviewed data/configuration and documentation lists, with every other recognized language counted as source. `metadata.kinds` records the lists. A code snippet in Markdown remains documentation. Documentation embedded in source, such as Rust `///` doc comments and their doctests, counts as comment lines of the host language. Language names unknown to the embedded Tokei stay `other`.
- Counters are reconstructed once from original reports and nested blobs; aggregate `Total` or `children` views are not added again. A reported path counts once, even when it contributes multiple languages.
- Folders expose direct and recursive totals. A recursive parent includes its children; displayed recursive rows cannot all be summed together.

## Evidence and Limits

Output uses JSON schema `1` (field reference in [PRODUCT.md](../PRODUCT.md#reporting-and-coverage)) with engine identity, Unix timestamp, effective scope, kind lists, coverage, configuration path and snapshot, and backend inaccuracy flags. `metadata.coverage` counts selected, counted, and language-mapped files and lists selected files that neither the backend nor a mapping recognizes, separately from recognized files that could not be read; the Markdown report summarizes both. Missing files are not proven zero LOC. Per-file records (`--files`) of files counted through a language mapping carry `language_mapping`. Raw backend statistics are no longer written; the private regression replays previously saved ones. Compared with the earlier `poc-2` schema, `metadata.source_languages` is replaced by `metadata.kinds`, and the `data/config` kind is spelled `data`.

Ambient Tokei configuration is not loaded. Git is not invoked, so reports do not claim a commit revision or unchanged working tree. Configuration and traversal errors fail before output with exit code 1; unreadable files produce partial output and exit code 2 with a message naming them. Classification is per file, so mixed production/test files are not split, and Locmetry does not establish whether a source file is handwritten.

Stdout is a compact JSON headline, and the full report goes to `--output FILE` (or stdout with `--output -`), as specified in [PRODUCT.md](../PRODUCT.md#reporting-and-coverage). Use `--format markdown` for readable stdout and `--depth N` to deepen its folder table. An output path inside the scan root or equal to the configuration is rejected before scanning.

## Privacy

Never put measured-project names, paths, rule examples, counts, or reports in shared documentation or fixtures. For this repository's sensitive analysis, choose `docs/research/local/` for every configuration and report and verify destinations with `git check-ignore` before running. The portable CLI enforces outputs outside the scan root but does not enforce this repository-specific privacy directory. Keep the local directory ignored and never force-add it.

## Private regression

The optional guard replays saved raw backend statistics through each reviewed TOML configuration and requires the saved file records and totals; it also checks reviewed example paths. It is skipped by ordinary `cargo test` runs:

```sh
LOCMETRY_PRIVATE_FIXTURES=docs/research/local/poc cargo test --locked --test private_regression -- --ignored
```

For each `<name>.locmetry.toml`, the directory must contain `<name>-measurement.json`, `<name>-measurement.tokei.json`, and `<name>.examples.json` with the checkout `root` and reviewed `examples`. Saved measurements predate language mappings, so files they never counted are absent from both sides. Fresh scans can differ when a working copy changes; compare equivalent snapshots before attributing differences to the counter.
