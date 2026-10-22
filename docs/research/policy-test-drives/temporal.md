# Temporal policies and workspace (2026-10-22)

Created by a MEDIUM-tier agent (Claude Code `sonnet` at `xhigh`) with `target/debug/locmetry` 0.9.0, `--no-user-config`; saved by the main session from the handback. Measured trees, all clean: [temporalio/documentation](https://github.com/temporalio/documentation) at `16c1899a0380`, [temporalio/samples-typescript](https://github.com/temporalio/samples-typescript) at `fb0aa23d7539` and [temporalio/sdk-typescript](https://github.com/temporalio/sdk-typescript) at `2595d1b62cf5`. Created as untracked files: a `.locmetryrc` in `documentation` (6 entries), `samples-typescript` (4) and `sdk-typescript` (5 plus a scope exclusion), copied here as [temporal-documentation.policy.toml](policies/temporal-documentation.policy.toml), [temporal-samples-typescript.policy.toml](policies/temporal-samples-typescript.policy.toml) and [temporal-sdk-typescript.policy.toml](policies/temporal-sdk-typescript.policy.toml), and the workspace file, copied as [temporal.workspace.toml](policies/temporal.workspace.toml). Moved from the private research folder on 2026-10-23; the raw exports stayed there.

## Summary

| Repository | Source | Test/prod before → after | Unclaimed before → after |
|---|---:|---|---|
| documentation | 22,954 | 0.11 → 0.07 | 7,962 → 38 |
| samples-typescript | 18,365 | 0.28 → 0.19 | 4,830 → 0 |
| sdk-typescript | 213,668 → 91,708 | 0.60 → 0.99 | 16,706 → 54 |

sdk-typescript excludes the `packages/core-bridge/sdk-core` Git submodule (temporalio/sdk-rust, 121,960 lines, 57% of the checkout) and reclassifies the private `test-helpers` package as test support. Other notable entries: a generated `payload-visitor.generated.ts` (5,365 lines, production:generated), a vendored `gogo.proto`, package sources outside `src/` (`core-bridge/ts`, `create-project/cli.js`, `proto/protos`), documentation sample apps as `example`, `sidebars.js` and `docusaurus.config.js` as `site-config`, a test runner `tests/run-all.mjs` as tooling (override), 50 byte-identical `.eslintrc.js` copies as tooling.

Workspace: 133,027 code lines, Test/prod 0.60; meaningful as an inventory, not as a ratio. Equal category names from different members merge into one row.

## Finding 1

Sixteen files, all label splits, no true collision: extensionless Bash scripts count as BASH while `.sh` Bash scripts count as Shell (documentation 2, samples 1 vs 3, sdk 1 plus 4 in the submodule); in the submodule a plain C header counts as C Header beside C. The agent read the help as promising a C Header fold; the main session checked `src/backend/languages.rs`: C Header stays apart on purpose because Tokei assigns every `.h` to it, and the help promises only the C++ Header fold. Not a defect.

## Finding 2

Four entries take 104 module-claimed files, 8,326 lines: `docs-sample-apps` (python 1,299, go 814, maven 270: those modules drop out of the note), `generated-payload-visitor` 5,365, `test-helpers-package` 460 (purpose change), `packages/docs` 118. Category-only wanted in three. samples-typescript has zero cases only because sibling folders were enumerated instead of broad globs, which is brittle.

## Configuration issues

1. A scope exclusion cannot carry a reason.
2. An excluded tree leaves no trace in table or Markdown output (the submodule note disappears).
3. No negation and no "only where nothing else claims" option.
4. Category refinement erases module credit.
5. The table note hides zero-credit modules.
6. `--config FILE` stops excluding the project's `.locmetryrc` (also found in a private project).
7. The workspace category table merges equal names across members.
8. `.mermaid` files cannot be mapped (no Mermaid language).

## Open questions with the agent's recommendations

1. Exclude the `sdk-core` submodule: keep excluded; measure sdk-rust separately.
2. Generated code as production:generated: keep.
3. Documentation sample apps as production:example: keep.
4. `packages/docs` as tooling: keep.
5. `site-config` category: keep.
6. Two small unclaimed files (`set-tab-language.js`, a stale proto copy): leave until maintainers answer.
7. Load and performance programs in `packages/test/src` as test: keep.
8. No `sample` category in samples-typescript: stay with implementation, since it would erase all module credit.
