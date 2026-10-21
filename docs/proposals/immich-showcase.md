# Immich Reference Showcase

<!-- doc-status
{
  "status": "draft",
  "date": "2026-10-21",
  "summary": "Showcase Locmetry on Immich pinned at a release tag, keeping Acme as the exact test reference.",
  "evidence": ""
}
-->

## Owner Input and Intended Outcome

The synthetic [Acme Orders workspace](../../examples/cli-workspace/README.md) and the [self-measurement](../self-measurement.md) are too small to convince a reader of Locmetry's value. The owner asked to measure a popular open-source project at a fixed commit, give it a dedicated Locmetry configuration, showcase workspace measurement with it and use it consistently. The README matters most: a first-time reader should see the tool's power there.

## Decisions — 2026-10-21

Owner decisions taken in the design discussion:

| Decision | Choice | Reason |
| --- | --- | --- |
| Role of Acme | Hybrid: Acme stays the exact reference; Immich becomes the showcase | Tests and edge-case explanations need independently known totals and deliberately placed cases (hidden, ignored, unrecognized, embedded, unclaimed); a real project only provides observed totals. |
| Reference project | [Immich](https://github.com/immich-app/immich) | Several languages, a clear production/test split, sibling sub-projects suitable as workspace members, and a showcase-sized policy story. |
| Pinning | Release tags, verified by commit | A tag is recognizable to readers; the commit makes the content exact. |
| Member policies | Locmetry-owned files copied into the checkout | No product change; it mirrors what a user measuring a third-party repository does. |
| README verification | No test gate | The README is the showcase and need not be part of the public test harness. |
| Public naming | Allowed | Recorded as an exception in [CLAUDE.md](../../CLAUDE.md#sensitive-analysis). |

## Scope

| Scope | Content |
| --- | --- |
| Required | Pinned Immich inputs and a fetch contract; Locmetry-owned root and member policies plus a workspace file; README showcase sections measured on Immich; recorded showcase guides for root, workspace and compare; a README refresh script. |
| Optional | Moving the [analytics demo](../analytics-demo.md) to the Immich comparison; showcase replay in documentation CI. |
| Excluded | Removing Acme or its tests; vendoring Immich files; network access from `cargo test`, Locmetry or the recordings; a per-member configuration field in the workspace contract; new language or module conventions motivated by Immich findings (separate decisions). |

Acme keeps the integration tests, the option-coverage [recorded command guides](../cli/README.md) and the explanation edge cases.

## Pinned Inputs

| Role | Tag | Commit | Upstream date |
| --- | --- | --- | --- |
| Showcase | `v3.3.4` | `69e0cb8606137c4f8dc0e7207a672ff8be39f358` | 2026-10-20 |
| Comparison baseline | `v3.2.0` | `1b6098c9dbfffe978bec2d414606ed7a4c8e019a` | 2026-09-09 |

`v3.3.4` was the latest stable release when decided; `v3.2.0` begins the previous minor line, so the comparison spans one minor release of real development. Upgrading the pin is a deliberate documentation change that regenerates all showcase output.

## Checkout Location and Fetch Contract

Proposed location: `<showcase>/immich-<tag>` outside the repository, where `<showcase>` is `LOCMETRY_SHOWCASE_DIR` when set, otherwise the platform cache folder followed by `locmetry/showcase`: `~/Library/Caches` on macOS, `${XDG_CACHE_HOME:-$HOME/.cache}` elsewhere.

- It survives `cargo clean` and `git clean -fdx`, and one download serves every clone and worktree on a machine.
- macOS excludes `~/Library/Caches` from Time Machine but includes `~/.cache`; Spotlight indexed neither (checked 2026-10-21 with `tmutil isexcluded` and `mdfind`). Each checkout occupies about 164 MB.
- It cannot enter Locmetry's self-measurement, the Showboat Git-state check or the documentation checker.
- The folder name supplies the report's project name: a measured symlink reports its canonical folder, so `immich-v3.3.4` appears in reports and compare titles.

A fetch script performs a shallow, single-tag clone and verifies `HEAD` against the pinned commit. An existing checkout with the right commit is reused; a commit mismatch fails rather than refetching silently. Recordings and the README refresh only check this precondition and print the fetch command when it fails; they never fetch. Git is a development tool for this script, not a Locmetry dependency.

## Policy Overlay

Locmetry-owned files live under `examples/showcase/immich/`: a `.locmetryrc` for the combined root, a `.locmetryrc` per workspace member and a `workspace.toml` naming sibling members such as `server`, `web`, `mobile`, `machine-learning` and `e2e`. The workspace file sits at the checkout root, so member roots stay relative to it. Environment setup copies the overlay on every use, so policy edits take effect without refetching. The copied files are the only permitted untracked paths in the checkout; anything else is reported as a mutated checkout.

Documentation states that the policy is Locmetry's, not upstream's. The policy is written from inspection evidence, not from expected numbers.

## Showcase Content

Unconfigured measurement of `v3.3.4` (2026-10-21, Locmetry 0.7.3, `--no-user-config`) reports 2,529 source files and 282,770 source code lines across 24 languages, with these review findings for the policy story:

- `e2e` (13,003 lines) counts as production; nothing marks it as an end-to-end test suite.
- `mobile/lib` (56,893 lines of Dart) is Other/unclaimed; no Flutter module convention is applied.
- `mobile` reports a Test/prod ratio of 3.69; the cause has not been investigated.

The README shows the configured measurement, the before/after effect of configuration, folder detail, a workspace summary across members and the `v3.2.0 → v3.3.4` comparison; unconfigured comparison on the same day measured +9,204 source code lines. The [sample-project tour](../sample-project-tour.md) moves to Immich; the [DuckDB cookbook](../duckdb-cookbook.md) capture stays self-contained so readers can run it without a download.

## Recordings and README Refresh

Showcase recordings live apart from the option-coverage guides. A separate environment script links the pinned checkouts into a temporary directory under their tag folder names, applies the overlay and writes reports outside every measured root; it never copies the checkout. Outputs stay reviewable after counting changes: bounded `--folder-rows`, jq projections of stable fields and the existing footer filters.

A refresh script regenerates the README's marked Immich excerpts from the pinned checkout; release preparation runs it. The existing `readme::` table guard over Acme retires with those tables; the Common commands guard is unaffected.

## Risks

- A deleted or moved upstream tag fails the commit check; the pinned commit remains the recovery input.
- Locmetry counting or Tokei changes rewrite many showcase numbers; reviewers must read those diffs as measurement changes.
- Immich is AGPL-3.0; Locmetry only measures it and redistributes no upstream files.

## Open Decisions

1. Checkout location: user cache as above (recommended) or a Git-ignored folder inside the repository.
2. Whether documentation CI fetches the pinned tags (cached by commit) and replays the showcase recordings; recommended, as a step separate from the Acme replay.
3. Whether the analytics demo moves to the Immich comparison; recommended, since it adds no download beyond the comparison baseline.
