# Go Layout: What `./...` Builds

Measured 2026-10-23 with the development build of Locmetry 0.9.0 at commit `94b2a78` and the change (release builds), on shallow clones removed afterwards. Question: which Go files should the `go` module detector stop claiming as production, so that vendored dependencies and test inputs no longer count silently as hand-written implementation? A [policy test drive](policy-test-drives/README.md#findings) found Loki's vendored dependencies, 3.16 million lines, counted as Go production.

## Rule and Source

The detector claimed every `*.go` file below `go.mod`. The Go tool's package patterns define what a module builds; [`go help packages`](https://pkg.go.dev/cmd/go#hdr-Package_lists_and_patterns), read in `src/cmd/go/internal/help/helpdoc.go` on the Go `master` branch on 2026-10-23, says:

> File system patterns expanded with the "..." wildcard exclude the following:
>
> - Directories named "vendor"
> - Directories named "testdata"
> - Files and directories with names beginning with "_" or "."
> - Directories that contain a go.mod file
> - Directories matching an ignore directive in a module's go.mod file

The detector now leaves out the first three at any depth. Nested `go.mod` files were already left to the innermost module. `ignore` directives in `go.mod` are not read: detection reads file names, not descriptor contents.

## Measurements

A script listed, for every `*.go` file, its innermost `go.mod` and the first excluded path element below it. Locmetry measured the same clones with the build before the change and with the change.

| Repository | Commit | Files no longer claimed | Production code before → after |
| --- | --- | --- | --- |
| [grafana/loki](https://github.com/grafana/loki) | `3c975bd5b7b5` | 10,640 in `vendor/`, 1 in `testdata/` | 3,533,017 → 369,550 |
| [kubernetes/kubernetes](https://github.com/kubernetes/kubernetes) | `674b5161f064` | 4,337 in `vendor/`, 51 in `testdata/`, 2 in `_codegenignore/` | 2,217,635 → 1,490,415 |
| [gin-gonic/gin](https://github.com/gin-gonic/gin) | `0f09c3a9b462` | 1 in `testdata/` | not measured |
| [etcd-io/etcd](https://github.com/etcd-io/etcd) | `3aaab3479b6c` | none | 76,978, unchanged |
| [gohugoio/hugo](https://github.com/gohugoio/hugo) | `ed22b85c192d` | none | 93,267, unchanged |
| [prometheus/prometheus](https://github.com/prometheus/prometheus) | `d6851c403316` | none | not measured |

The file counts come from the script; Locmetry moves fewer files, because Loki's ignore files already leave many vendored files unselected and test conventions decide others first. Every moved line went to `other:unclaimed`; test and other totals were otherwise unchanged.

## False Claims and Lost Claims

The change only removes claims, so it cannot add a false claim. Every file it stops claiming was checked for a lost correct claim:

- `vendor/` in Loki and Kubernetes holds `go mod vendor` output, with `vendor/modules.txt` beside it.
- Loki's `pkg/chunkenc/testdata` package is imported only by `_test.go` files in its parent; Kubernetes' `testdata` packages are inputs of tool tests such as `cmd/import-boss`; gin's is a generated protocol buffer test message.
- Kubernetes' `_codegenignore` folders are code-generator test inputs.

No file the change stops claiming is production code of the measured project. Names that merely contain the words, such as `vendored/` or `my_data/`, stay claimed, as the regression test checks.

## Performance

A first implementation listed one glob per built folder; on Kubernetes it took 1.9 times the memory. The detector now keeps `**/*.go` and excludes `**/{vendor,testdata,_*,.*}/**` and `**/[._]*`: hyperfine over six runs on Kubernetes measured 160.7 MiB against 154.5 MiB before the change, and no slower wall time.
