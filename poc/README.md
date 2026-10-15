# Legacy Profiles and Reports

The TOML/glob configuration in [PRODUCT.md](../PRODUCT.md) is the product interface; [README](../README.md) shows its usage. This page documents the provisional JSON/regex profiles that preceded it, which `--config <file>.json` still loads until existing profiles are migrated, and the report behavior shared by both formats. The implementation is Rust with Tokei embedded. Build using `cargo build --release --locked`; the resulting executable needs no Python, Git, or external Tokei.

## Run from the repository root

Keep real-project configurations and outputs exclusively in Git-ignored `docs/research/local/`:

```sh
cargo build --release --locked
./target/release/locmetry --config docs/research/local/project-rules.json --root /path/to/project --output docs/research/local/measurement
cargo test --locked
```

The configuration path above is a placeholder, not a bundled project. `--root` overrides the configuration's local path. Each run replaces `.md`, `.json`, and `.tokei.json` files at the output prefix, appending each suffix to the full prefix name so `run.v1` and `run.v2` stay distinct: readable breakdowns, classified records/folder totals, and the raw backend response. The runner does not modify target source/configuration or run target builds/tests.

## Configuration

This synthetic example illustrates the PoC format. Rust regular expressions are provisional; they are not the chosen product configuration syntax.

```json
{
  "project": "example",
  "root": "/path/to/project",
  "exclude_prefixes": ["reference-material/"],
  "rules": [
    {
      "id": "test-trees",
      "pattern": "(^|/)(tests|commonTest)/",
      "purpose": "test",
      "category": "tests-and-support",
      "reason": "Explicit test source sets."
    },
    {
      "id": "application",
      "pattern": "^src/",
      "purpose": "production",
      "category": "implementation",
      "reason": "Project-owned application source after test exceptions."
    }
  ]
}
```

## Measurement Contract

- Profiles scan hidden and untracked nonignored files, exclude Git metadata, and preserve existing ignore behavior; TOML configurations make hidden-file and ignore-file selection configurable. Ignore behavior: `.ignore` and `.tokeignore` files always apply, `.gitignore` applies only inside a Git work tree, and symbolic links are not followed. Apply configured root-prefix exclusions before counting and check that they did not leak into the response.
- `exclude_prefixes` contains literal project-root-relative directories, not globs; each becomes the root-relative glob `<prefix>/**`. `reference-material/` does not match a nested directory of that name. CLI scope options apply to profiles as well.
- Match `rules` in order against root-relative POSIX paths using Rust `regex` matching; first match wins. Patterns are unanchored unless they use `^` or `$`, so `test` also matches `latest/`. Preserve the rule ID and leave unmatched files unclassified with rule `fallback`, which is reserved. A rule without `category` uses its purpose as the category. Path components come from the platform's path parser, so Unix file names may contain `:` or `\`.
- Keep purpose (`production`, `test`, `unclassified`) separate from content kind (`source`, `data/config`, `documentation`, `other`). Production means maintained non-test source, including separately categorized tooling/examples, not just shipped runtime code.
- Derive content kind from the Tokei language: reviewed data/configuration and documentation lists, with every other recognized language counted as source. `metadata.kinds` records the lists. A code snippet in Markdown remains documentation. Documentation embedded in source, such as Rust `///` doc comments and their doctests, counts as comment lines of the host language. Language names unknown to the embedded Tokei stay `other`.
- Reconstruct counters from original reports and nested blobs once. Do not add aggregate `Total` or `children` views. A reported path counts once, even when it contributes multiple languages.
- Expose direct and recursive folder totals. A recursive parent includes its children; displayed recursive rows cannot all be summed together.

## Evidence and Limits

Output uses schema `poc-3` with engine identity, Unix timestamp, effective scope, kind lists, coverage, rule snapshot, and backend inaccuracy flags. `metadata.coverage` counts selected and counted files and lists selected files the backend did not recognize separately from recognized files it could not read; the Markdown report summarizes both. Missing files are not proven zero LOC. Compared with `poc-2`, `metadata.source_languages` is replaced by `metadata.kinds`, the `data/config` kind is spelled `data`, and `metadata.scope` lists effective exclusion globs. Ambient Tokei configuration is not loaded; normal repository/parent/global ignore behavior remains. Git is not invoked, so reports do not claim a commit revision or unchanged working tree. Traversal/configuration errors fail the command; backend inaccuracy produces partial output and a nonzero exit whose message names unreadable files. The PoC does not split mixed production/test files or establish whether every source file is handwritten.

Without `--output`, stdout contains the full JSON report. With `--output`, JSON stdout is a compact summary and full records go to the report files. Use `--format markdown` for readable stdout. Invalid regex syntax, unknown configuration keys, duplicate or reserved rule IDs, and invalid purposes fail before scanning. An output prefix whose reports would overwrite the configuration is rejected before scanning. Rust regex does not support lookaround or backreferences. Relative scan roots resolve against the invocation directory. A profile's `root` is used only when `--root` is absent.

Synthetic tests guard precedence, fallback, root exclusions, source/data separation, embedded-language deduplication, folder rollups, and duplicate paths. Real-project rule guards and analyses belong in Git-ignored local documentation; they are not needed to run the public tests.

## Privacy

Never put measured-project names, paths, rule examples, counts, or reports in shared documentation or fixtures. For this repository’s sensitive analysis, choose `docs/research/local/` for every profile and report and verify destinations with `git check-ignore` before running. The portable CLI enforces outputs outside the scan root but does not enforce this repository-specific privacy directory. Keep the local directory ignored and never force-add it.

## Private migration regression

The optional Rust guard checks reviewed local examples and reconstructs saved historical measurements from raw backend statistics. It is skipped by ordinary `cargo test` runs:

```sh
LOCMETRY_PRIVATE_FIXTURES=docs/research/local/poc cargo test --locked --test private_regression -- --ignored
```

The directory must contain matching `*-rules.json`, `*-measurement.json`, and `*-measurement.tokei.json` files. Fresh scans can differ when a working copy changes; compare equivalent snapshots before attributing differences to the counter.
