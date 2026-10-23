# Immich Showcase Recordings

These executable demonstrations measure [Immich](https://github.com/immich-app/immich), a popular self-hosted photo platform, at pinned releases. They show what Locmetry adds on a real multi-language project: content kinds separated from source, production and test told apart, and a reviewed policy that records what only the project knows. The [command guides](../cli/README.md) remain the option-by-option reference on the small checked-in fixture.

- [Tour: build a policy for Immich](tour.md): built-in conventions, review, a draft policy from evidence and the resulting transitions.
- [Measure Immich](measure.md): built-in conventions alone, the evidence behind a decision, the reviewed policy, explanations, a review of every policy entry, folder detail, a category selection and the mobile app.
- [Measure Immich as a workspace](workspace.md): six application components measured under the project's one policy.
- [Compare two Immich releases](compare.md): `v3.2.0 → v3.3.4` for the whole repository and for one member.

## Run an example

Build the current executable with `cargo build --locked` and fetch the pinned checkouts once with `bash scripts/fetch-showcase.sh`; the [showcase inputs](../../examples/showcase/immich/README.md) describe pinning, cache location and Locmetry's policy. From the repository root, run a recording's complete Bash block. Its first line sources [showcase-env.sh](../../scripts/showcase-env.sh), which verifies the checkouts, applies the policy overlay and links `immich` (the `v3.3.4` showcase release) and `<tag>/immich` for each pinned release into a new temporary directory. Personal defaults are isolated there, so the commands omit `--no-user-config`; add it to your own runs if your numbers differ from the recorded ones.

Commands and captured output are real. `jq` projections select fields, `sed` expressions omit measurement-time and location lines or select report sections, and `demo_wrap` wraps displayed text at 120 characters, as the [authoring guidelines](../showboat-authoring.md) require.

## Verify and regenerate

Replay only these recordings with:

```sh
cargo build --locked
bash scripts/fetch-showcase.sh
uv run --no-project --python 3.11 --with showboat==0.6.1 \
  python scripts/check-recorded-guides.py --set showcase
```

Without the pinned checkouts, each showcase document fails with the fetch command instead of replaying. `--set reference` replays only the fixture guides, and the default `--set all` replays both. Documentation CI replays the showcase in its own step after restoring the checkouts from a cache keyed by the pins. Moving a pin changes the recorded numbers; regenerate the recordings after reviewing the policy against the new tree, as the [showcase inputs](../../examples/showcase/immich/README.md#move-the-pin) describe.
