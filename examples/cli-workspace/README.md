# Synthetic Acme Orders Workspace

This is the source for the [sample-project tour](../../docs/sample-project-tour.md), [README tables](../../README.md#what-you-see) and [recorded command guides](../../docs/cli/README.md). Its tiny source files demonstrate measurement; they are not a buildable application. Go/npm descriptors supply module context. No packages, interpreters or build tools are needed to measure them, and no dependencies should be installed here.

- `acme-orders/service`: Go production and test source, maintained Python tooling, an unresolved experiment, excluded vendor code, hidden source and ignored scratch work.
- `acme-orders/web`: TypeScript/TSX production and tests, HTML with embedded JavaScript, data, documentation and an unrecognized `.asset` file.
- `acme-orders/.locmetryrc`: policy for measuring the combined tree. Member configurations express equivalent policy relative to their own roots for workspace measurement.
- [workspace.toml](workspace.toml): names the two member roots relative to this file; member names are `service` and `web`.

The ignored and hidden files deliberately remain in Git. `.ignore` affects Locmetry's selection, not Git's inventory. The scope explicitly disables hidden files; `--hidden true` demonstrates an override. The experiment remains unclaimed and the asset remains unrecognized, without making successfully counted files partial.

Copy this whole directory to a temporary location before changing anything. Save reports beside `acme-orders`, outside both members. The recorded guides use [cli-demo-env.sh](../../scripts/cli-demo-env.sh) to do that automatically and remove the copy on shell exit. Root and workspace source totals agree; metadata, configuration provenance and member-relative paths differ.

To refresh the root README's measured table excerpts after an intentional fixture or renderer change:

```sh
cargo test --locked --test integration readme::refresh_readme_tables -- --ignored
cargo test --locked --test integration readme::
```

The public integration guard checks actual classifications, embedded-language contributions and independently known source totals. [Showboat verification](../../docs/cli/README.md#verify-and-regenerate) checks the recorded workflows and their output.
