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
| Checkout location | Platform cache folder, outside the repository | Reused across clones and worktrees, excluded from Time Machine on macOS; see [Checkout Location and Fetch Contract](#checkout-location-and-fetch-contract). |
| Narrative | Line counters, then unconfigured Locmetry, then a reviewed policy | Shows what each step adds; see [Showcase Narrative](#showcase-narrative). |
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

Location: `<showcase>/immich-<tag>` outside the repository, where `<showcase>` is `LOCMETRY_SHOWCASE_DIR` when set, otherwise the platform cache folder followed by `locmetry/showcase`: `~/Library/Caches` on macOS, `${XDG_CACHE_HOME:-$HOME/.cache}` elsewhere.

- It survives `cargo clean` and `git clean -fdx`, and one download serves every clone and worktree on a machine.
- macOS excludes `~/Library/Caches` from Time Machine but includes `~/.cache`; Spotlight indexed neither (checked 2026-10-21 with `tmutil isexcluded` and `mdfind`). Each checkout occupies about 164 MB.
- It cannot enter Locmetry's self-measurement, the Showboat Git-state check or the documentation checker.
- The folder name supplies the report's project name: a measured symlink reports its canonical folder, so `immich-v3.3.4` appears in reports and compare titles.

A fetch script performs a shallow, single-tag clone and verifies `HEAD` against the pinned commit. An existing checkout with the right commit is reused; a commit mismatch fails rather than refetching silently. Recordings and the README refresh only check this precondition and print the fetch command when it fails; they never fetch. Git is a development tool for this script, not a Locmetry dependency.

## Policy Overlay

Locmetry-owned files live under `examples/showcase/immich/`: a `.locmetryrc` for the combined root, a `.locmetryrc` per workspace member and a `workspace.toml` naming sibling members such as `server`, `web`, `mobile`, `machine-learning` and `e2e`. The workspace file sits at the checkout root, so member roots stay relative to it. Environment setup copies the overlay on every use, so policy edits take effect without refetching. The copied files are the only permitted untracked paths in the checkout; anything else is reported as a mutated checkout.

Documentation states that the policy is Locmetry's, not upstream's. The policy is written from inspection evidence, not from expected numbers.

## Showcase Narrative

The README tells one story in three steps, each answering a question the previous step cannot. Every number below was measured on `v3.3.4` on 2026-10-21 with Locmetry 0.7.3 (`--no-user-config`), Tokei 15.0.0, cloc 2.10 and scc 4.1.0, all with default options; the showcase regenerates them rather than copying these values.

### 1. A line counter answers "how much text, per language"

| Tool | Reported code | JSON share | Production/test split |
| --- | --- | --- | --- |
| Tokei | 518,917 | 209,607 (40%) | None |
| cloc | 530,427 | 209,931 (40%) | None |
| scc | 510,072 | 209,867 (41%) | None |

Most of the JSON is data: Drift database schemas (41,829 lines), the OpenAPI specification (34,045) and translation files. YAML adds the 22,484-line `pnpm-lock.yaml`. The tools also split embedded languages differently: cloc and scc count 36.8k Svelte lines, while Tokei reports 17,442 Svelte lines and attributes the script and style blocks inside Svelte files to JavaScript (18,966), CSS and PostCSS (421), the same total.

The comparison must stay fair. These tools are good at what they do, and Locmetry counts with Tokei. scc adds complexity and cost estimates and cloc compares directories; Locmetry offers neither. The claim is about the question answered, not counting accuracy.

### 2. Locmetry without configuration answers "how much source, and how much of it is tests"

Locmetry separates data and documentation from source without configuration: 282,770 source code lines, split into 144,377 production, 73,049 test and 65,344 other (0.51 test/prod). Built-in conventions and module descriptors find the npm, Gradle and Python modules and the conventional tests. They also report, rather than guess, what they cannot decide:

- `mobile/lib`, 56,893 lines of Flutter code, is unclaimed; Locmetry has no Dart module convention.
- `mobile` therefore reports 4,250 production against 15,680 test lines, an implausible test/prod ratio of 3.69.
- `e2e/src` (13,003 lines) counts as production: its `*.e2e-spec.ts` names do not match the `*.spec.ts` convention, and the npm module claims the rest.
- Generated code counts as hand-written production: the oazapfts SDK client (`packages/sdk/src/fetch-client.ts`, header "DO NOT MODIFY") and 37 query snapshots under `server/src/queries/` (header "auto generated by ./sql-generator").

### 3. Locmetry with a reviewed policy answers "what does this project consist of"

A draft policy of two overrides and seven rules, each with a recorded reason, was tested against the same tree:

| Entry | Decision |
| --- | --- |
| Generated SDK client and SQL snapshots | Other / `generated` (override, beats the module convention) |
| `e2e/src/**`, `packages/e2e-auth-server/**` | Test / `end-to-end`, `test-infrastructure` |
| `mobile/lib`, local package `lib` folders, Pigeon definitions | Production; `mobile/integration_test` as test |
| `mobile/ios` | Production / `native` |
| Deployment files, developer commands, build and lint configuration | Production / `deployment`, `tooling` |

| Measure | Unconfigured | Draft policy |
| --- | --- | --- |
| Production | 144,377 | 179,552 |
| Test | 73,049 | 86,837 |
| Other (generated + unclaimed) | 65,344 (0 + 65,344) | 16,381 (16,165 + 216) |
| `mobile` test/prod | 3.69 | 0.24 |

Every entry was used, and 216 lines remain unclaimed for review. Folder detail, `explain` on a generated file, the workspace summary across members and the `v3.2.0 → v3.3.4` comparison (+9,204 unconfigured source code lines) complete the story; `explain` shows the rule and reason behind each decision.

The README shows these three steps; the [sample-project tour](../sample-project-tour.md) becomes the longer Immich walkthrough. The [DuckDB cookbook](../duckdb-cookbook.md) capture stays self-contained so readers can run it without a download.

The Flutter and `*.e2e-spec.ts` gaps are possible built-in convention additions; each is a separate decision. Adding one would shorten step 3 without invalidating the narrative.

## Recordings and README Refresh

Showcase recordings live apart from the option-coverage guides. A separate environment script links the pinned checkouts into a temporary directory under their tag folder names, applies the overlay and writes reports outside every measured root; it never copies the checkout. Outputs stay reviewable after counting changes: bounded `--folder-rows`, jq projections of stable fields and the existing footer filters.

A refresh script regenerates the README's marked Immich excerpts from the pinned checkout; release preparation runs it. The existing `readme::` table guard over Acme retires with those tables; the Common commands guard is unaffected.

## Risks

- A deleted or moved upstream tag fails the commit check; the pinned commit remains the recovery input.
- Locmetry counting or Tokei changes rewrite many showcase numbers; reviewers must read those diffs as measurement changes.
- Immich is AGPL-3.0; Locmetry only measures it and redistributes no upstream files.

## Open Decisions

1. Whether documentation CI fetches the pinned tags (cached by commit) and replays the showcase recordings; recommended, as a step separate from the Acme replay.
2. Whether the analytics demo moves to the Immich comparison; recommended, since it adds no download beyond the comparison baseline.
3. Whether the README names the other tools or describes them generically ("line counters such as Tokei, cloc and scc"); recommended to name them with versions, because a reader can then reproduce the comparison.
