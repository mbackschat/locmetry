# Loki experiment (2026-10-23)

Main session, `target/debug/locmetry` 0.9.0, `--no-user-config`, on a shallow clone of `grafana/loki` at `be73bce830abd53ea51e99ca28374f8772a00b50` (2026-10-22), removed afterwards. Purpose: test both proposals outside the JVM/TypeScript sample of the policy test drives. [loki.policy.toml](policies/loki.policy.toml) is the quick realistic policy with ordinary full rules; its `generated` rule lists the files whose header says `DO NOT EDIT`.

## Language corrections

- `.tpl` as Pan: 32 files, Helm and operator documentation Go templates. Go HTML shares the `{{/* */}}` comment syntax: one Helm `_helpers.tpl` counts 15 comment lines as Pan and 179 as Go HTML (code 1,170 to 1,006). So Helm templates seen earlier in a private project should also correct to Go HTML, not to unrecognized.
- `.y` as Happy: 5 goyacc grammars; Happy has no comment syntax. Correction to unrecognized.
- `.s` as GNU Style Assembly: 70 Go assembler files in vendor; comment syntax matches, no correction needed.
- BASH 7 files vs Shell 39: the split again (fourth project).
- `.tmpl` as Templ: one vendored Go text template.

## Module credit

The Go detector claims every `.go` below `go.mod`, including 3.16M lines in `vendor/`, 130,204 lines of generated code (42 `.pb.go`, 5 `.y.go`, deepcopy and others) and the Go tools. Policy outcome: Go credit 262,414 with current semantics; 396,844 when credit is the production source a layout covers (generated 130,204 plus tools 4,226 kept). Category-only entries would keep the same 134,430 only after splitting entries by coverage: `tools/` mixes module Go with scripts (2,200 lines would stay unclaimed), and deployment (7,793) and examples (669) have no module source at all. Conclusion: replace category-only entries by credit by coverage (proposal revised in commit `baca05b`).

## Other observations

- Go `vendor/` is toolchain-defined like `node_modules`, which is excluded by default, but is counted as Go module production; a built-in convention or default exclusion is a candidate.
- `npm` module `pkg/logql/internal/logqltest/syntax` has no `src/` and classifies nothing, but counts as a module.
