# Measuring Locmetry Itself

The root [.locmetryrc](../.locmetryrc) defines how Locmetry measures its own implementation, verification code and supporting tools. It expresses repository intent where built-in conventions cannot know it. The [configuration guide](guides/configure.md) explains the review workflow; [PRODUCT](PRODUCT.md#matching-and-precedence) owns configuration semantics.

## Scope

Exclude `examples/cli-workspace/**` from repository-root measurements. This tree is a synthetic application used to demonstrate and test counting behavior; its artificial language mix and test/production composition should not contribute to Locmetry's own totals. The exclusion removes its counted files, module context and unrecognized entries. The actual regression tests and demonstration-verification scripts stay measured.

Keep the normal ignore policy and built-in exclusions. [.gitignore](../.gitignore) already owns generated build output, private analysis and Python caches; the configuration does not duplicate those paths. Caller overrides and personal defaults can change selection. Use `--no-user-config` when checking the repository policy in isolation.

The synthetic workspace remains independently measurable. Configuration discovery uses the selected root, without inheriting parent `.locmetryrc` files. A direct measurement of `examples/cli-workspace/acme-orders` therefore uses that fixture's own policy, while `--subfolder` retains the repository policy and cannot reopen the excluded tree. Recorded demonstrations use temporary copies; see the [fixture instructions](../examples/cli-workspace/README.md).

## Classification

| Area | Purpose and category | Design reason |
| --- | --- | --- |
| Rust implementation under `src/` | Production / `implementation`, through Cargo inference | Existing module detection already expresses its role. |
| `tests/` and conventional `scripts/test_*.py` files | Test / `test`, through built-in conventions | Preserve existing decisions without redundant rules or overrides. |
| Documentation-index maintenance and release preparation | Production / `tooling` | These maintained commands operate the repository and prepare distribution. |
| Source under `packaging/` | Production / `distribution` | Formula generation, installation definitions and notice templates are maintained distribution code. |
| Named documentation checks, demo support and agent-evaluation harnesses | Test / `test-infrastructure` | They execute, prepare or assess verification workflows. |
| Source under `examples/analytics/` | Other / `example` | Maintained user-facing recipes stay counted and visible without entering the implementation/test ratio. |

The configuration owns the exact path lists and category descriptions. Script entries name known responsibilities rather than assigning every future file under `scripts/` to production or test. New unrecognized responsibilities should surface as unclaimed source and prompt a review.

All added classification rules restrict `kind` to `source`. Data and documentation remain counted in their own content-kind totals; they are not pulled into these purpose decisions merely because they share a directory. Existing built-in conventions can still assign them a purpose. Ordinary rules suffice: no decision here needs to outrank a test or build convention.

Production includes operational tooling even when it is not shipped in the executable. Test includes the infrastructure used to verify the project. This is a role-based distinction, not a statement that production means shipped bytes or that every test file contains assertions.

## Deliberate Limits

Keep the default unclaimed fallback and existing language recognition. There is no catch-all production rule, no override to clear findings and no language mapping added solely to make the recognition count look complete. An unrecognized license or ignore file does not establish missing implementation source; inspect meaningful gaps before adding mappings.

The test/production ratio describes whole-file classifications. Rust unit tests embedded in production files under `src/` remain counted with those files. This policy cannot separate them, and the ratio is not a coverage or quality score. See [measurement limits](measurement.md#limits).

## Review and Verify

From the repository root, use the installed executable or the current local build at `target/debug/locmetry`:

```sh
locmetry --no-user-config --format table --verbose
locmetry explain scripts/release.py --no-user-config --format json
locmetry explain scripts/check-doc-examples.sh --no-user-config --format json
locmetry explain examples/cli-workspace/acme-orders/service/src/main.go --no-user-config --format json
locmetry --root examples/cli-workspace/acme-orders --no-user-config --format table
```

Expect release preparation to be production tooling, the documentation checker to be test infrastructure, and the fixture path to be explicitly excluded in the repository-root explanation. The final command should still measure the demonstration with its own policy. Inspect deciding rules and coverage, not just whether the unclaimed list is empty.

Before changing policy, capture full file detail and use the [configuration-transition recipe](guides/compare.md#inspect-configuration-transitions). Counts for retained files should stay unchanged when only classification changes; excluding the fixture changes the measured subset. Reports do not establish unchanged source contents. Keep self-measurement reports and any independent input-identity evidence in the ignored local-analysis directory, following [contributor instructions](../CLAUDE.md#sensitive-analysis).

One focused [configuration regression](../tests/integration/config.rs) loads the actual root policy into a synthetic repository and checks fixture exclusion and independent fixture discovery. Run `cargo test --locked --test integration config::repository_policy_excludes_the_demo_without_affecting_direct_demo_scans`; the [README fixture checks](../tests/integration/readme.rs) and [recorded-guide replay](cli/README.md#verify-and-regenerate) additionally protect the real demonstration. Review classification choices through the self-scan and targeted explanations above rather than maintaining a second path-by-path copy of the policy in tests.
