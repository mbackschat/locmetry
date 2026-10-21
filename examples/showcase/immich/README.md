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

## Review the policy

Every entry restricts itself to source and records the evidence behind it, such as a generator header or a package's declared entry point. Measure without the overlay to see what built-in conventions decide alone; `locmetry explain PATH --root CHECKOUT` shows the deciding rule of any file. A policy change must leave no unused entries or exclusions and no unclaimed source without a recorded reason, on both pinned tags.

## Move the pin

Update [pins.tsv](pins.tsv) with a newer release tag and its commit, fetch, and review the policy against the new tree. Then regenerate the showcase recordings and the README excerpts; their numbers describe the pinned trees, so moving a pin is a deliberate documentation change.
