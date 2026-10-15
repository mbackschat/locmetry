# Measurement Semantics and Limits

[PRODUCT.md](../PRODUCT.md) owns the configuration, measurement model, and report schema, and the [README](../README.md) shows CLI usage. This page records selection and counting details, known limits, and this repository's private regression.

## Selection and Paths

- Untracked nonignored files are included. With ignore files respected, `.ignore` and `.tokeignore` files always apply and `.gitignore` applies only inside a Git work tree. Symbolic links are not followed.
- Exclusions are checked again against the backend response, so excluded files cannot leak into counts.
- Rules match root-relative `/`-separated paths. Components come from the platform's path parser, so Unix file names may contain `:` or `\`.
- An output path inside the scan root or equal to the configuration is rejected before scanning, so a report never measures itself or replaces its policy; in workspace mode the same holds for every member and the workspace file.

## Counting

- Content kind comes from the Tokei language through reviewed data and documentation lists, recorded in the report file as `metadata.kinds`; every other recognized language is source. A code snippet in Markdown remains documentation. Language names unknown to the embedded Tokei stay `other`.
- Docstrings count as code, as in Tokei's default: locmetry does not set Tokei's `treat_doc_strings_as_comments`, so every line inside a Python docstring, blank ones included, is a code line. This affects the languages with doc strings in Tokei 15.0.0: Python, Cython, Mojo, Raku, Bazel, Snakemake, Pony, and Roc. Doc comments such as Rust `///`, Javadoc, or TSDoc are comments as usual. Counting docstrings as comments would change totals and therefore requires a new configuration `version`.
- Counters are reconstructed once from original reports and nested blobs; aggregate `Total` or `children` views are not added again. A reported path counts once, even when it contributes multiple languages.
- `breakdown` rows in the report file are additive. So are the top-level folders of `summary.top_folders` and the terminal table: each covers its whole subtree, and together with the root files they cover every file once. The Markdown folder table lists nested levels, where a parent includes its subfolders, so its rows cannot all be summed together.

## Limits

- Ambient Tokei configuration is not loaded. Git is not invoked, so reports do not claim a commit revision or unchanged working tree.
- Selected files that neither the backend nor a language mapping recognizes are listed in `metadata.coverage`, not proven zero LOC.
- Classification is per file, so mixed production/test files are not split, and Locmetry does not establish whether a source file is handwritten.
- Unreadable files produce partial output and exit code 2 with a message naming them.

## Private Regression

Keep real-project configurations and reports exclusively in Git-ignored `docs/research/local/`, as [CLAUDE.md](../CLAUDE.md#sensitive-analysis) requires, and verify destinations with `git check-ignore` before running. The CLI requires outputs outside the scan root but does not enforce this repository-specific directory:

```sh
cargo build --release --locked
./target/release/locmetry --root /path/to/project --config docs/research/local/project.locmetryrc --output docs/research/local/measurement.json
```

The optional guard replays saved raw backend statistics through each reviewed TOML configuration, requires the saved file records and totals, and checks reviewed example paths. It is skipped by ordinary `cargo test` runs:

```sh
LOCMETRY_PRIVATE_FIXTURES=docs/research/local/poc cargo test --locked --test private_regression -- --ignored
```

For each `<name>.locmetry.toml`, the directory must contain `<name>-measurement.json`, `<name>-measurement.tokei.json`, and `<name>.examples.json` with the checkout `root` and reviewed `examples`. The fixtures are frozen: Locmetry no longer writes raw backend statistics, so they cannot be regenerated, and the guard normalizes their older serialization instead. They predate language mappings, so files they never counted are absent from both sides. Fresh scans can differ when a working copy changes; compare equivalent snapshots before attributing differences to the counter.
