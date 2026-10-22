# Prior Art for Purpose Classification

Researched 2026-10-23 from the repositories' sources at the commits listed. Question: does an existing tool or library classify files by purpose (production, test, generated, vendored) and infer source layouts from build files, so that Locmetry could reuse it the way it reuses Tokei for counting?

## Findings

| Project | Commit | License | What it classifies | What it does not |
| --- | --- | --- | --- | --- |
| [GitHub Linguist](https://github.com/github-linguist/linguist) | `73ee9a3b277f` | MIT | Vendored and documentation paths by regex tables (`vendor.yml`, `documentation.yml`); generated files through 74 checks in [`generated.rb`](https://github.com/github-linguist/linguist/blob/73ee9a3b277f/lib/linguist/generated.rb), many reading the first lines of the file; per-repository overrides with the `linguist-generated`, `linguist-vendored` and `linguist-documentation` attributes in `.gitattributes`. | No test or production purpose and no source layouts; the files it marks are only left out of language statistics. |
| [go-enry](https://github.com/go-enry/go-enry) | `71719e4bdde5` | Apache-2.0 | A Go port of Linguist's data with an API: `IsVendor`, `IsGenerated(path, content)`, `IsDocumentation`, `IsConfiguration` and `IsTest`. | `IsTest` is nine hand-written regexes, such as `*_test.go` and `*.spec.ts`, a subset of Locmetry's test conventions. |
| [gengo](https://github.com/spenserblack/gengo) | `655cbe5c78c9` | Apache-2.0 | A Rust language classifier modelled on Linguist, with `gengo-generated`, `gengo-vendored` and `gengo-documentation` attributes. | Small (30 stars), and as a language classifier it overlaps Tokei; generated and vendored files are excluded from its statistics, not reported. |
| [@vercel/frameworks](https://github.com/vercel/vercel/tree/c628be7835e0/packages/frameworks) and Netlify's framework detection | `c628be7835e0` | Apache-2.0 | Which framework a project uses, by marker files, dependencies and file content. | Returns build commands and output folders, not which files are source. |
| SonarQube, Codecov, Code Climate | | | Separate tests from production. | Users configure the globs, such as `sonar.tests`; nothing is inferred. |

No project combines a production, test and tooling split with source layouts inferred from build descriptors and per-file provenance. That layer remains Locmetry's own; the framework layout table has the same shape as Vercel's detectors, marker file to behaviour, but Vercel stops at naming the framework.

## Generated code

Linguist's `generated.rb` is the most mature public catalogue of generated-file markers, refined over years of false-claim reports. Against the generators found in the [framework layout precision pass](framework-layouts.md#shared-false-claims-not-specific-to-a-row):

- **Covered:** Go files with a `// Code generated … DO NOT EDIT.` line in their first 40 lines, which includes swag, Wire, MockGen and the protobuf and parser output in Loki; protocol buffer output for several languages; ANTLR; jOOQ; .NET designer files.
- **Not covered:** OpenAPI Generator clients, MyBatis Generator output and Django migrations. Each carries its own header comment, so detecting them needs markers beyond Linguist's.

## Implications

- Reusing a library is not possible without a new runtime: Linguist is Ruby and go-enry is Go, and gengo would duplicate Tokei. Linguist's checks can be adapted under its MIT license with attribution, and serve as cited evidence for each marker.
- Linguist's `.gitattributes` overrides are declarations that projects already maintain for GitHub. Reading them would apply a project's own judgment without a Locmetry policy, but how often real projects set them is unmeasured.
- Linguist excludes generated files from statistics; Locmetry's [deferred generated/minified evidence](../plans/measurement-quality-and-net-change.md#conditional-follow-ups-and-preserved-rationales) holds that generated does not mean disposable, so a design would classify them, for example as `production:generated`, rather than drop them.
