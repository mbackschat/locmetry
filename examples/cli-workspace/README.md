# Synthetic Acme Orders Workspace

This is the source for the [recorded command guides](../../docs/cli/README.md) and for integration tests that check independently known totals. The README and tour measure the [Immich showcase](../showcase/immich/README.md) instead. Its tiny source files demonstrate measurement; they are not a buildable application. Go/npm descriptors supply module context. No packages, interpreters or build tools are needed to measure them, and no dependencies should be installed here.

- `acme-orders/service`: Go production and test source, a Makefile, maintained Python tooling, an unresolved experiment, excluded vendor code, hidden source and ignored scratch work.
- `acme-orders/web`: TypeScript/TSX production and tests, a generated API client, HTML with embedded JavaScript, a template counted as HTML through a language mapping, data, documentation and an unrecognized `.asset` file.
- `acme-orders/.locmetryrc`: policy for measuring the combined tree, in the recommended shape: declared categories, entries restricted to source and a reason on each. Member configurations express equivalent policy relative to their own roots for workspace measurement.
- [workspace.toml](workspace.toml): names the two member roots relative to this file; member names are `service` and `web`.

The ignored and hidden files deliberately remain in Git. `.ignore` affects Locmetry's selection, not Git's inventory. The scope explicitly disables hidden files; `--hidden true` demonstrates an override. The experiment remains unclaimed and the asset remains unrecognized, without making successfully counted files partial. Each classification stage has one example: an override makes the generated client Other despite the npm module layout, the Makefile is build tooling by convention, the tooling rule claims the check script, module layouts claim the rest of production, and the template shows a language mapping.

Copy this whole directory to a temporary location before changing anything. Save reports beside `acme-orders`, outside both members. The recorded guides use [cli-demo-env.sh](../../scripts/cli-demo-env.sh) to do that automatically and remove the copy on shell exit. Root and workspace source totals agree; metadata, configuration provenance and member-relative paths differ.

The public integration guard checks actual classifications, embedded-language contributions and independently known source totals. [Showboat verification](../../docs/cli/README.md#verify-and-regenerate) checks the recorded workflows and their output.
