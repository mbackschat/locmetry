# Measure Immich as a Workspace

*2026-10-21T03:17:22Z by Showboat 0.6.1*
<!-- showboat-id: 84efac68-5dbe-4a98-9f39-dbd87e4303ea -->

Run each Bash block from the Locmetry repository root after `cargo build --locked` and `bash scripts/fetch-showcase.sh`; the [showcase index](README.md) explains the environment. Immich's application components live side by side in one repository. A [workspace file](../../examples/showcase/immich/overlay/workspace.toml) names six of them as folders of the project, so each is measured as `--subfolder` would measure it, under the checkout's single `.locmetryrc`.

## Measure the components together

The workspace report keeps the regular shape and adds one row per repository. Folders outside the members, such as `docs/` and `deployment/`, are not part of this workspace.

```bash
source scripts/showcase-env.sh
locmetry workspace --workspace-file v3.3.4/immich/workspace.toml --format table --no-color --no-bars \
  | sed '/^Root:/d; /^Roots:/d; /^Project:/d; /^Measured at:/d'
```

```output
 ────────────────────────────────────────────────────────────────────────────────────────────
 Language    Files    Lines     Code │ Comments  Blanks │   Prod.    Test   Other │ Test/prod
 ────────────────────────────────────┼──────────────────┼─────────────────────────┼──────────
 TypeScript  1,083  171,885  145,882 │    4,633  21,370 │  70,800  68,616   6,466 │      0.97
 Dart          823   90,871   75,726 │    2,779  12,366 │  59,495  16,231       0 │      0.27
 JavaScript     11   22,954   19,487 │      382   3,085 │  19,425      62       0 │      0.00
 Svelte        412   19,014   17,442 │      200   1,372 │  17,420      22       0 │      0.00
 SQL            37   10,534    9,699 │      436     399 │       0       0   9,699 │         -
 Python         39    6,184    4,888 │       79   1,217 │   2,818   2,070       0 │      0.73
 Kotlin         35    4,604    3,810 │      177     617 │   3,810       0       0 │      0.00
 Swift          28    3,874    3,103 │      232     539 │   3,103       0       0 │      0.00
 CSS             1      692      589 │       18      85 │     589       0       0 │      0.00
 Dockerfile      4      369      267 │       12      90 │     257      10       0 │      0.04
 C++             1      295      251 │       11      33 │     251       0       0 │      0.00
 Shell          12      307      209 │       47      51 │     209       0       0 │      0.00
 C               2      225      168 │       30      27 │     168       0       0 │      0.00
 Groovy          3      173      139 │        7      27 │     139       0       0 │      0.00
 Java            1      166      124 │       25      17 │     124       0       0 │      0.00
 HTML            1       74       66 │        2       6 │      66       0       0 │      0.00
 BASH            2       40       30 │        2       8 │      30       0       0 │      0.00
 PostCSS         0       31       28 │        0       3 │      28       0       0 │      0.00
 CMake           1       13        9 │        0       4 │       9       0       0 │      0.00
 C Header        1        1        1 │        0       0 │       1       0       0 │      0.00
 ────────────────────────────────────┼──────────────────┼─────────────────────────┼──────────
 Total       2,497  332,306  281,918 │    9,072  41,316 │ 178,742  87,011  16,165 │      0.49
 Lines = Code + Comments + Blanks · Production + Test + Other = Code · Test/prod = Test / Production (Code)
 Tests written in another language than the code they test skew per-language Test/prod.
 Folded: TSX → TypeScript.

 ─────────────────────────────────────────────────────────────────────────────────
 Member              Lines     Code │   Prod.    Test   Other │ Test/prod │  Share
 ───────────────────────────────────┼─────────────────────────┼───────────┼───────
 server            129,296  111,950 │  53,296  48,955   9,699 │      0.92 │  39.7%
 mobile            100,095   83,183 │  66,952  16,231       0 │      0.24 │  29.5%
 web                67,833   58,255 │  52,867   5,388       0 │      0.10 │  20.7%
 e2e                15,491   13,174 │      61  13,113       0 │    214.97 │   4.7%
 packages           12,893   10,065 │   2,345   1,254   6,466 │      0.53 │   3.6%
 machine-learning    6,698    5,291 │   3,221   2,070       0 │      0.64 │   1.9%
 ───────────────────────────────────┼─────────────────────────┼───────────┼───────
 Total             332,306  281,918 │ 178,742  87,011  16,165 │      0.49 │ 100.0%
 Production + Test + Other = Code · Test/prod = Test / Production (Code) · Share = Code / workspace Code

```

The rows show each component's character at a glance: the server's Test/prod ratio is high, the web client's is low, and `e2e` is almost entirely test code. Other in `server` and `packages` is generated code. Workspace Test/prod divides summed test code by summed production code rather than averaging member ratios.

## Measure one member on its own

A folder member is the project's `--subfolder` run of that folder, so measuring the folder directly reproduces its workspace row.

```bash
source scripts/showcase-env.sh
locmetry --root immich --subfolder mobile --format json \
  | jq '.summary.source_purpose | map_values(.code)'
```

```output
{
  "production": 66952,
  "test": 16231
}
```
