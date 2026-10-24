# Measuring Locmetry Itself

The root [.locmetryrc](../.locmetryrc) defines how Locmetry measures its own implementation, verification code and supporting tools. It expresses repository intent where built-in conventions cannot know it, and it is meant as a model policy: each entry uses the weakest stage that expresses its decision, comments in the file say why, its categories come from the [category reference](guides/categories.md), and every entry's report figures are intended. The [configuration guide](guides/configure.md) explains the review workflow; [PRODUCT](PRODUCT.md#matching-and-precedence) owns configuration semantics.

## Scope

Exclude `examples/cli-workspace/**` from repository-root measurements. This tree is a synthetic application used to demonstrate and test counting behavior; its artificial language mix and test/production composition should not contribute to Locmetry's own totals. The exclusion removes its counted files, module context and unrecognized entries. The actual regression tests and demonstration-verification scripts stay measured.

Keep the normal ignore policy and built-in exclusions. [.gitignore](../.gitignore) already owns generated build output, private analysis and Python caches; the configuration does not duplicate those paths. Caller overrides and personal defaults can change selection. Use `--no-user-config` when checking the repository policy in isolation.

The synthetic workspace remains independently measurable. Configuration discovery uses the selected root, without inheriting parent `.locmetryrc` files. A direct measurement of `examples/cli-workspace/acme-orders` therefore uses that fixture's own policy, while `--subfolder` retains the repository policy and cannot reopen the excluded tree. Recorded demonstrations use temporary copies; see the [fixture instructions](../examples/cli-workspace/README.md).

## Classification

| Area | Purpose and category | Design reason |
| --- | --- | --- |
| Rust implementation under `src/` | Production / `implementation`, through Cargo inference | Existing module detection already expresses its role. |
| Rust unit test modules `src/**/tests.rs`, and the two script tests that import their module and mock every process | Test / `unit`, through the `rust-test-module` and Python test conventions and the `unit-tests` refinement | The conventions decide the purpose; the refinement only names the kind of test. |
| The integration-test binary under `tests/integration/`, and the script tests that run their script or Git as a process | Test / `integration`, through the test-directory and Python test conventions and the `integration-tests` refinement | These tests run the built executable, a script or Git as a process, which is a different kind of evidence from unit tests. |
| `tests/integration/common.rs` | Test / `test-support`, through the `integration-test-helpers` refinement | It holds helpers shared by the integration tests and no test of its own. |
| The documentation-example check | Test / `integration` | It runs documented examples with the built executable and fails on a mismatch, as an integration test does, but no test convention recognizes its name, so a rule decides it. |
| The comparison helper `scripts/compare-locmetry-files.sh` | Production / `implementation` | It ships in release archives and Homebrew installations, so it is part of the product although no build descriptor declares it. |
| Documentation maintenance (indexes, the README's Immich excerpts and demo GIF, and the agent-regression trend table) | Production / `tooling` | These maintained commands serve the people working on the repository. |
| Source under `packaging/` and the release command | Production / `distribution` | Formula generation, installation definitions, notice templates and the command that checks, commits, tags and pushes a release publish Locmetry. |
| Agent-evaluation harnesses and graders, the recorded-guide replay, the file-size gate, and demo and showcase environments | Production / `verification-tooling` | They are programs that run, prepare or score verification, and most have their own unit tests. |

The configuration owns the exact path lists and category descriptions. Script entries name known responsibilities rather than assigning every future file under `scripts/` to production or test. New unrecognized responsibilities should surface as unclaimed source and prompt a review.

All added rules and refinements restrict `kind` to `source`. Data and documentation remain counted in their own content-kind totals; they are not pulled into these purpose decisions merely because they share a directory. Existing built-in conventions can still assign them a purpose. Ordinary rules suffice: no decision here needs to outrank a test or build convention, so the policy has no overrides. Test categories come from refinements rather than overrides, because an override over `tests/**` would also beat the conventions and could move a file between purposes; a refinement cannot, so Test/prod stays exactly what the conventions decide.

Production includes operational tooling even when it is not shipped in the executable, including the programs that run verification. Test holds the code that verifies: test files, checks and support that exists only inside tests, such as fixtures. A program with its own unit tests is code under test, so classifying it as Test would count those unit tests as tests of tests and inflate Test/prod; until 2026-10-22 the verification programs were Test, which put Python's ratio at 5.62 where its tests of Python code give 0.79 (owner decision, 2026-10-22). This is a role-based distinction, not a statement that production means shipped bytes or that every test file contains assertions.

## Deliberate Limits

Keep the default unclaimed fallback and existing language recognition. There is no catch-all production rule, no override to clear findings and no language mapping added solely to make the recognition count look complete. An unrecognized license or ignore file does not establish missing implementation source; inspect meaningful gaps before adding mappings.

The test/production ratio describes whole-file classifications. Rust unit tests inline in production files under `src/` remain counted with those files, and the table's legend names their code, while a `tests.rs` module file is test through the built-in `rust-test-module` convention; move a large inline test module out of line rather than adding a project rule. The ratio is not a coverage or quality score. See [measurement limits](measurement.md#limits).

## File Sizes

CI gates source file size with this measurement, as the [file-size gate cookbook](cookbooks/file-size-gate.md) describes for any project: [check-file-sizes.py](../scripts/check-file-sizes.py) runs the built executable with `--files`, and [file-size-limits.json](../scripts/file-size-limits.json) holds the limits. Limits count code lines, so explanatory comments never count against a file, and they follow the purpose this policy decides: production files warn above 600 and fail above 1,000, test files warn above 1,000 and fail above 2,000, and Other is unchecked. A large file is a navigation and review cost before it is a design problem, so the soft limit asks for a split while it is still cheap.

Files that predated the limits received a ceiling at their size when the gate was introduced on 2026-10-21; splitting them retired every ceiling the same day. A ceiling replaces the hard limit for that file and may only shrink: the gate fails when the file grows past it, asks for a lower ceiling when the file shrinks, and fails once the file fits the hard limit until the ceiling is removed. Split an oversized file rather than raising its limit.

Inline Rust unit tests count against their production file's limits; moved into a `tests.rs` submodule, they follow the test limits. Run the gate locally after `cargo build --locked` with `python3 scripts/check-file-sizes.py`; [its regression suite](../scripts/test_file_sizes.py) covers limits, ceilings and stale entries.

## Review and Verify

From the repository root, use the installed executable or the current local build at `target/debug/locmetry`:

```sh
locmetry --no-user-config --format table --verbose
locmetry explain scripts/release.py --no-user-config --format json
locmetry explain scripts/check-doc-examples.sh --no-user-config --format json
locmetry explain examples/cli-workspace/acme-orders/service/src/main.go --no-user-config --format json
locmetry --root examples/cli-workspace/acme-orders --no-user-config --format table
```

Expect release preparation to be production tooling, the documentation checker to be test, and the fixture path to be explicitly excluded in the repository-root explanation. In `summary.policy_entries` of `locmetry --no-user-config --format json`, expect no `preempted`, `unused_patterns` or `unused_exceptions` on any entry, no `displaced` on the rules, which claim scripts nothing else decides, and only `test:test` as `displaced` on the three refinements. Anything else means the tree or the policy changed and needs a review. The final command should still measure the demonstration with its own policy. Inspect deciding rules and coverage, not just whether the unclaimed list is empty.

Before changing policy, capture full file detail and use the [configuration-transition recipe](guides/compare.md#inspect-configuration-transitions). Counts for retained files should stay unchanged when only classification changes; excluding the fixture changes the measured subset. Reports do not establish unchanged source contents. Keep self-measurement reports and any independent input-identity evidence in the ignored local-analysis directory, following [contributor instructions](../CLAUDE.md#sensitive-analysis).

One focused [configuration regression](../tests/integration/config.rs) loads the actual root policy into a synthetic repository and checks fixture exclusion and independent fixture discovery. Run `cargo test --locked --test integration config::repository_policy_excludes_the_demo_without_affecting_direct_demo_scans`; the [README fixture checks](../tests/integration/readme.rs) and [recorded-guide replay](cli/README.md#verify-and-regenerate) additionally protect the real demonstration. Review classification choices through the self-scan and targeted explanations above rather than maintaining a second path-by-path copy of the policy in tests.
