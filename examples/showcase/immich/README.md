# Immich Showcase Inputs

Locmetry's public showcase measures [Immich](https://github.com/immich-app/immich), a popular self-hosted photo platform with a TypeScript server, a Svelte web client, a Flutter app and a Python machine-learning service. The [showcase proposal](../../../docs/proposals/archive/immich-showcase.md) records why it was chosen and how it is used. This folder holds everything Locmetry owns about it; no upstream file is copied into this repository.

- [pins.tsv](pins.tsv): the showcase release and the comparison baseline, each as tag and commit.
- [overlay/.locmetryrc](overlay/.locmetryrc): Locmetry's reviewed policy for the whole checkout.
- [overlay/workspace.toml](overlay/workspace.toml): the six application components as folders of the checkout, measured under that same policy.

## Fetch the checkouts

```sh
bash scripts/fetch-showcase.sh
```

The script makes one shallow clone per pinned tag and verifies its commit. It reuses a checkout already at the pinned commit and refuses one at any other commit. Checkouts live outside the repository as `<tag>/immich` in `~/Library/Caches/locmetry/showcase/` on macOS, which Time Machine excludes, and in `${XDG_CACHE_HOME:-~/.cache}/locmetry/showcase/` elsewhere; set `LOCMETRY_SHOWCASE_DIR` to choose another directory. Each checkout takes about 164 MB. `--check` verifies the checkouts without network access.

Recorded examples source [showcase-env.sh](../../../scripts/showcase-env.sh). It copies the overlay into each checkout, fails if anything else differs from the pinned commit, and links `immich` (the showcase release) and `<tag>/immich` for each pinned release into a temporary directory. Every checkout folder is named `immich` because reports take the project name from it, and comparison flags differing names.

## Measure by hand

After fetching, copy the overlay into the showcase checkout and measure it:

```sh
showcase=$(bash scripts/fetch-showcase.sh --check)
cp -R examples/showcase/immich/overlay/. "$showcase/v3.3.4/immich/"
locmetry --root "$showcase/v3.3.4/immich"
locmetry workspace --workspace-file "$showcase/v3.3.4/immich/workspace.toml"
```

Personal defaults in `~/.config/locmetry/config.toml` (on Windows `%APPDATA%\locmetry\config.toml`) can add exclusions or language mappings; add `--no-user-config` if your numbers differ from the published ones.

## How the policy is built

The overlay is meant as a model policy. Each entry uses the weakest stage that expresses its decision, comments in the file say why, its categories come from the [category reference](../../../docs/guides/categories.md), and the report's `summary.policy_entries` shows only intended effects. Built-in detection decides most of the checkout; the policy adds what only the project knows.

| Entries | Stage | Decision | Why this stage |
| --- | --- | --- | --- |
| `generated-sdk-client`, `generated-query-snapshots` | rule | Other / `generated` | Rules apply before build modules, so they take these files from the npm layout; no convention claims generated sources, so an override is not needed. |
| `end-to-end-suites`, `end-to-end-auth-server`, `load-tests` | rule | Test; the auth-server stand-in `test-support`, the load test `performance`, the suites by purpose only | The npm layouts would count this test code as production, or nothing claims it. The suites' `*.e2e-spec.ts` files are tests by convention first, which the report lists as `preempted`. |
| `pigeon-definitions`, `plugin-sdk-command`, `ios-app` | rule | Production | No build descriptor declares this maintained code, so it would stay unclaimed. |
| `deployment` | rule with `except` | Production / `deployment` | Images, start scripts, infrastructure and the installer of self-hosted installations. The exception keeps the end-to-end auth server's Dockerfile with its test support, whatever the entry order. Compose files under `docker/` are data, which source rules do not classify. |
| `build-scripts` | rule | Production / `build-tooling` | Bundler and linter configuration and the Xcode build phase that runs the Flutter build, which the built-in tool-configuration conventions do not recognize by name. |
| `documentation-site` | rule | Production / `documentation-site` | The Docusaurus configuration and sidebars exist only for the documentation site. |
| `developer-tooling` | rule with `except` | Production / `tooling` | Development containers and maintained developer commands. The exception leaves the Flutter build phase in `mobile/ios/scripts/` to `build-scripts`. |
| `android-app` | refinement | Production / `native` | The Gradle layout already counts the Android runner as production; a refinement relabels it without touching purpose or module credit. |
| `release-scripts`, `documentation-site-pages` | refinements | Production / `distribution`, `documentation-site` | The npm layouts count the release command in `packages/scripts/` and the documentation site's pages and components as production, which they are, but label them `implementation`, which the category reference reserves for product functionality. |
| `end-to-end`, `web-client-integration`, `end-to-end-support` | refinements | Test / `end-to-end`, `integration`, `test-support` | The suites under `e2e/src/specs/` drive the running server and web client without mocks; every web-client spec under `e2e/src/ui/specs/` mocks the API, so it tests short of the whole system; the remaining files hold no test cases. The refinements apply whether the convention or the `end-to-end-suites` rule made a file a test, and `except` states the split instead of entry order. |
| `release-notes-template` | language correction | Markdown documentation | The counter reads the Markdown release-notes template in `misc/release/` as Templ source by its `.tmpl` extension. |

There are no overrides: no decision has to beat a test or build convention.

## Review the policy

Every entry restricts itself to source and records the evidence behind it, such as a generator header or a package's declared entry point. Measure without the overlay to see what built-in conventions decide alone; `locmetry explain PATH --root CHECKOUT` shows the deciding rule of any file. A policy change must leave no unused entries or exclusions and no unclaimed source without a recorded reason, on both pinned tags.

Then read `summary.policy_entries` on both tags. Expect no `unused_patterns` or `unused_exceptions` anywhere, `preempted` only on `end-to-end-suites` (the e2e-spec convention), and `displaced` only where the table above explains it: production implementation on the generated, end-to-end, developer-tooling, Android, release-scripts and documentation-site-pages entries, and `test:test` on the end-to-end refinements. `--verbose` names the entries that change a purpose, which should be the two generated-code rules and `end-to-end-suites`.

## Move the pin

Update [pins.tsv](pins.tsv) with a newer release tag and its commit, fetch, and review the policy against the new tree. Then regenerate the showcase recordings and the README excerpts; their numbers describe the pinned trees, so moving a pin is a deliberate documentation change.
