# Immich Reference Showcase

<!-- doc-status
{
  "status": "implemented",
  "date": "2026-10-21",
  "summary": "Immich at pinned release tags became the public showcase; Acme stays the exact test reference.",
  "evidence": "[Showcase recordings](../../showcase/README.md)"
}
-->

Status: historical decision record — implemented. The showcase was implemented and verified locally on 2026-10-21: all eleven recorded guides replay against the pinned checkouts and the public Cargo suite passes; the documentation CI replay had not yet run, because the commits were not pushed. Archived on 2026-10-21 under the [retirement procedure](../README.md#retirement-procedure). The [showcase index](../../showcase/README.md) and the [showcase inputs](../../../examples/showcase/immich/README.md) own the current behavior.

Two outcomes differ from the design below. Project-folder workspaces replaced the six member policies, as the update note under Policy Overlay records. The owner reversed the decision that the DuckDB cookbook capture stays self-contained: on 2026-10-21 both [analytics cookbooks](../../duckdb-cookbook.md) moved to recorded Immich releases, with a synthetic test for the cases Immich cannot show.

The sections below preserve the reviewed design and acceptance criteria. Their imperative wording, examples and stated limitations are historical evidence, not current guarantees. Do not update this body to track behavior; this record carries no live scope.

## Owner Input and Intended Outcome

The synthetic [Acme Orders workspace](../../../examples/cli-workspace/README.md) and the [self-measurement](../../self-measurement.md) are too small to convince a reader of Locmetry's value. The owner asked to measure a popular open-source project at a fixed commit, give it a dedicated Locmetry configuration, showcase workspace measurement with it and use it consistently. The README matters most: a first-time reader should see the tool's power there.

## Decisions — 2026-10-21

Owner decisions taken in the design discussion:

| Decision | Choice | Reason |
| --- | --- | --- |
| Role of Acme | Hybrid: Acme stays the exact reference; Immich becomes the showcase | Tests and edge-case explanations need independently known totals and deliberately placed cases (hidden, ignored, unrecognized, embedded, unclaimed); a real project only provides observed totals. |
| Reference project | [Immich](https://github.com/immich-app/immich) | Several languages, a clear production/test split, sibling sub-projects suitable as workspace members, and a showcase-sized policy story. |
| Pinning | Release tags, verified by commit | A tag is recognizable to readers; the commit makes the content exact. |
| Member policies | Locmetry-owned files copied into the checkout | No product change; it mirrors what a user measuring a third-party repository does. |
| Checkout location | Platform cache folder, outside the repository | Reused across clones and worktrees, excluded from Time Machine on macOS; see [Checkout Location and Fetch Contract](#checkout-location-and-fetch-contract). |
| Narrative | Line counters, then unconfigured Locmetry, then a reviewed policy | Shows what each step adds; see [Showcase Narrative](#showcase-narrative). |
| Acme improvements | Recommended policy shape plus the uncovered classification stages and cleanup findings | Acme is the reference readers copy and the guides' option coverage; see [Acme Reference Improvements](#acme-reference-improvements). |
| README verification | No test gate | The README is the showcase and need not be part of the public test harness. |
| Public naming | Allowed | Recorded as an exception in [CLAUDE.md](../../../CLAUDE.md#sensitive-analysis). |

## Scope

| Scope | Content |
| --- | --- |
| Required | Pinned Immich inputs and a fetch contract; Locmetry-owned root and member policies plus a workspace file; README showcase sections measured on Immich; recorded showcase guides for root, workspace and compare; the [analytics demo](../../analytics-demo.md) on the Immich comparison; showcase replay in documentation CI; a README refresh script; the [Acme reference improvements](#acme-reference-improvements). |
| Optional | None identified. |
| Excluded | Removing Acme or its tests; vendoring Immich files; network access from `cargo test`, Locmetry or the recordings; a per-member configuration field in the workspace contract; Acme demonstrations of Git submodule hints, notebooks or nested modules. |

Acme keeps the integration tests, the option-coverage [recorded command guides](../../cli/README.md) and the explanation edge cases.

## Pinned Inputs

| Role | Tag | Commit | Upstream date |
| --- | --- | --- | --- |
| Showcase | `v3.3.4` | `69e0cb8606137c4f8dc0e7207a672ff8be39f358` | 2026-10-20 |
| Comparison baseline | `v3.2.0` | `1b6098c9dbfffe978bec2d414606ed7a4c8e019a` | 2026-09-09 |

`v3.3.4` was the latest stable release when decided; `v3.2.0` begins the previous minor line, so the comparison spans one minor release of real development. Upgrading the pin is a deliberate documentation change that regenerates all showcase output.

## Checkout Location and Fetch Contract

Location: `<showcase>/<tag>/immich` outside the repository, where `<showcase>` is `LOCMETRY_SHOWCASE_DIR` when set, otherwise the platform cache folder followed by `locmetry/showcase`: `~/Library/Caches` on macOS, `${XDG_CACHE_HOME:-$HOME/.cache}` elsewhere.

- It survives `cargo clean` and `git clean -fdx`, and one download serves every clone and worktree on a machine.
- macOS excludes `~/Library/Caches` from Time Machine but includes `~/.cache`; Spotlight indexed neither (checked 2026-10-21 with `tmutil isexcluded` and `mdfind`). Each checkout occupies about 164 MB.
- It cannot enter Locmetry's self-measurement, the Showboat Git-state check or the documentation checker.
- The folder name supplies the report's project name, and a measured symlink reports its canonical folder. Every release folder is therefore named `immich`: comparison flags differing project names as an identity change, which tag-named folders triggered.

A fetch script performs a shallow, single-tag clone and verifies `HEAD` against the pinned commit. An existing checkout with the right commit is reused; a commit mismatch fails rather than refetching silently. Recordings and the README refresh only check this precondition and print the fetch command when it fails; they never fetch. Git is a development tool for this script, not a Locmetry dependency.

## Policy Overlay

Locmetry-owned files live under `examples/showcase/immich/`: a `.locmetryrc` for the combined root, a `.locmetryrc` per workspace member and a `workspace.toml` naming sibling members such as `server`, `web`, `mobile`, `machine-learning` and `e2e`. The workspace file sits at the checkout root, so member roots stay relative to it. Environment setup copies the overlay on every use, so policy edits take effect without refetching. The copied files are the only permitted untracked paths in the checkout; anything else is reported as a mutated checkout.

Documentation states that the policy is Locmetry's, not upstream's. The policy is written from inspection evidence, not from expected numbers.


Update 2026-10-21: workspace files gained [project folders](../../PRODUCT.md#workspace-mode), which measure members under the project's single policy. The overlay now holds only the root `.locmetryrc` and a `workspace.toml` naming the six folders; the six member policies, which repeated the root policy's decisions, were removed.

## Showcase Narrative

The README tells one story in three steps, each answering a question the previous step cannot. Every number below was measured on `v3.3.4` on 2026-10-21 without personal defaults, using Locmetry 0.7.3 with the Dart and `*.e2e-spec` conventions described below, Tokei 15.0.0, cloc 2.10 and scc 4.1.0, all with default options; the showcase regenerates them rather than copying these values.

### 1. A line counter answers "how much text, per language"

| Tool | Reported code | JSON share | Production/test split |
| --- | --- | --- | --- |
| Tokei | 518,917 | 209,607 (40%) | None |
| cloc | 530,427 | 209,931 (40%) | None |
| scc | 510,072 | 209,867 (41%) | None |

Most of the JSON is data: Drift database schemas (41,829 lines), the OpenAPI specification (34,045) and translation files. YAML adds the 22,484-line `pnpm-lock.yaml`. The tools also split embedded languages differently: cloc and scc count 36.8k Svelte lines, while Tokei reports 17,442 Svelte lines and attributes the script and style blocks inside Svelte files to JavaScript (18,966), CSS and PostCSS (421), the same total.

The comparison must stay fair. These tools are good at what they do, and Locmetry counts with Tokei. scc adds complexity and cost estimates and cloc compares directories; Locmetry offers neither. The claim is about the question answered, not counting accuracy.

### 2. Locmetry without configuration answers "how much source, and how much of it is tests"

Locmetry separates data and documentation from source without configuration: 282,770 source code lines, split into 193,662 production, 83,485 test and 5,623 other (0.43 test/prod). Module descriptors find the npm, Gradle, Python and Dart/Flutter modules; test conventions find unit, Flutter integration and `*.e2e-spec.ts` suites. What only the project knows remains visible rather than guessed:

- Generated code counts as hand-written production: the oazapfts SDK client (`packages/sdk/src/fetch-client.ts`, header "DO NOT MODIFY") and 37 query snapshots under `server/src/queries/` (header "auto generated by ./sql-generator").
- End-to-end support code in `e2e/src` (fixtures, generators, utilities; 3,118 lines) counts as production through the npm module.
- iOS app code under `mobile/ios` (3,102 lines) is unclaimed, because no descriptor declares it; it dominates the 5,623 unclaimed lines listed for review.

### 3. Locmetry with a reviewed policy answers "what does this project consist of"

A draft policy of two overrides and six rules, each with a recorded reason, was tested against the same tree:

| Entry | Decision |
| --- | --- |
| Generated SDK client and SQL snapshots | Other / `generated` (override, beats the module convention) |
| `e2e/src/**`, `packages/e2e-auth-server/**` | Test / `end-to-end`, `test-infrastructure` |
| Pigeon platform-channel definitions | Production |
| `mobile/ios` | Production / `native` |
| Deployment files, developer commands, build and lint configuration | Production / `deployment`, `tooling` |

| Measure | Unconfigured | Draft policy |
| --- | --- | --- |
| Production | 193,662 | 179,552 |
| Test | 83,485 | 86,837 |
| Other: generated + unclaimed | 5,623 (0 + 5,623) | 16,381 (16,165 + 216) |
| Test/prod | 0.43 | 0.48 |

Every entry was used, and 216 lines remain unclaimed for review. Folder detail, `explain` on a generated file, the workspace summary across members and the `v3.2.0 → v3.3.4` comparison (+9,204 unconfigured source code lines with Locmetry 0.7.3) complete the story; `explain` shows the rule and reason behind each decision.

The README shows these three steps; the former Acme sample-project tour becomes the [Immich policy tour](../../showcase/tour.md). The [DuckDB cookbook](../../duckdb-cookbook.md) capture stays self-contained so readers can run it without a download.

The spike first ran Locmetry 0.7.3, which had no Dart module detector and no `*.e2e-spec` convention: 56,893 lines of Flutter code were unclaimed, `mobile` reported an implausible test/prod ratio of 3.69, and all 13,003 `e2e/src` lines counted as production. Both conventions were added before recording the showcase, so it demonstrates shipped behavior; the draft policy then needed two fewer rules for the same configured result.

### Personal defaults in examples

Showcase commands omit `--no-user-config`, so readers do not mistake it for a required flag. Recordings already isolate personal defaults through their environment script. One note beside the first README command says that personal defaults in `~/.config/locmetry/config.toml` can explain numbers that differ from a local run and that `--no-user-config` removes them. The flag stays where reproducibility is the point: CI recipes, baseline captures for comparison and policy verification.

## Recordings and README Refresh

Showcase recordings live apart from the option-coverage guides. A separate environment script links the pinned checkouts into a temporary directory under their tag folder names, applies the overlay and writes reports outside every measured root; it never copies the checkout. Outputs stay reviewable after counting changes: bounded `--folder-rows`, jq projections of stable fields and the existing footer filters.

A refresh script regenerates the README's marked Immich excerpts from the pinned checkout. Contributors run it after a pin move or a measurement or presentation change that affects them; release verification does not, because it must not change its inputs or depend on the checkouts. The existing `readme::` table guard over Acme retires with those tables; the Common commands guard is unaffected.

## Acme Reference Improvements

Acme stays small, and every addition must explain something no recording shows today. Its policies become the recommended shape, because readers copy fixture files:

| Change | Demonstrates |
| --- | --- |
| Rewrite the combined-root and member `.locmetryrc` files with `kind = ["source"]`, a `reason` on every entry and a `[categories]` table, without changing totals | The configuration shape recommended by the configure guide and [self-measurement](../../self-measurement.md); `explain` shows the reason |
| Add a few-line generated client under `web/src/` with a "DO NOT EDIT" header, classified Other / `generated` by an `[[overrides]]` entry | An override beating the npm module convention, and intentional Other without a proposed policy |
| Add a two-line `service/Makefile` | The `build-script` convention: Production / `build-tooling` |
| Map one invented template extension to HTML through `languages.mappings`, while `web/docs/wireframe.asset` stays unrecognized | Mapped and unrecognized files side by side in coverage |
| In a recording, measure with a deliberately stale `--config` variant | `unused_entries` and `unused_exclusions` findings, while the fixture's own measurement stays clean |

These changes land after the README tables move to Immich, as their own commit. Acme's independently known totals then change only in the reference recordings, the integration tests that check them and the fixture README, so the change stays reviewable apart from the showcase.

## Risks

- A deleted or moved upstream tag fails the commit check; the pinned commit remains the recovery input.
- Locmetry counting or Tokei changes rewrite many showcase numbers; reviewers must read those diffs as measurement changes.
- Immich is AGPL-3.0; Locmetry only measures it and redistributes no upstream files.

## Open Decisions

None. Decisions taken 2026-10-21 after the narrative spike: the README names Tokei, cloc and scc with their versions so readers can reproduce the comparison; documentation CI fetches the pinned tags, cached by commit, and replays the showcase recordings in a step separate from the Acme replay; the [analytics demo](../../analytics-demo.md) moves to the `v3.2.0 → v3.3.4` comparison.
