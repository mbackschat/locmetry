# A tour of a sample project

Follow `acme-orders` from its first measurement through configuration, classification checks, folder exploration and saved reports. This is the hypothetical monorepo behind the [README tables](../README.md#what-you-see), with Kotlin backend modules, a TypeScript/TSX frontend, Go workers, Python tooling and unresolved experiments.

The report numbers are illustrative and come from one [checked data fixture](../tests/fixtures/readme/README.md). There is no downloadable source checkout. The paths below sketch the project; commands demonstrate the real CLI and can be adapted to an existing checkout. Run them from the directory containing `acme-orders`; report exports go beside that directory so they cannot become part of the next measurement. `--no-user-config` isolates the examples from personal defaults.

## Start with the layout and local intent

These representative paths explain the different roles within the project; this is not a complete file inventory:

```text
acme-orders/
├── .locmetryrc
├── gradlew
├── backend/
│   ├── orders/src/main/kotlin/domain/Order.kt
│   ├── orders/src/test/kotlin/OrderTest.kt
│   └── billing/, catalog/, customers/, inventory/, shipping/
├── web/src/orders/{Order.ts,OrderView.tsx,Order.test.ts}
├── workers/{billing,inventory,shipping}/
├── scripts/imports/orders/order.py
├── scripts/maintenance/
├── prototypes/{pricing,forecasting}/
├── docs/{architecture.md,order-flow.png}
└── third_party/payment-sdk/
```

Assume the backend modules have selected Gradle descriptors, the frontend has `package.json`, and the workers have Go module descriptors. Locmetry recognizes those filenames and their conventional layouts; it does not parse build targets or execute a build. The innermost module supplies the context for a file. The [module contract](PRODUCT.md#module-detection) defines each supported layout.

The project owner has established that the SDK is vendored and outside the intended scope, while `scripts/` is maintained tooling. The experiments have no settled ownership yet. These are project decisions, not facts Locmetry can infer from the amount of code.

Save that policy in `acme-orders/.locmetryrc`:

```toml
version = 1

[scope]
exclude = ["third_party/**"]

[[rules]]
id = "tooling"
match = ["scripts/**"]
purpose = "production"
category = "tooling"
```

The exclusion removes the SDK from the measurement. The rule retains maintained scripts and labels their role within production. No broad production rule is needed for conventional module source. Use the [configuration guide](guides/configure.md) to establish this evidence for a real repository, including already-classified files that might be wrong.

## Read the first measurement

```sh
locmetry --root acme-orders --no-user-config --format table
```

Read the [language table](../README.md#what-you-see) from left to right: physical lines split into code, comments and blanks; code then splits into production, test and unclassified. Tooling is a category within production, rather than a fourth purpose.

The frontend tests and Kotlin tests contribute to Test even when their enclosing module is production. The Python experiments and shell wrapper both contribute to Unclassified, but require different interpretation. JSON configuration and Markdown documentation are counted separately from source; an unrecognized image has unavailable counts, not an invented zero.

The illustrative dataset has no embedded-language overlap, so its language file counts also reconcile. In general, a file can participate in more than one language: adding language file counts does not necessarily give the number of physical files. [Report interpretation](guides/reports.md#interpret-the-numbers) explains that distinction.

## Find where the code resides

```sh
locmetry --root acme-orders --no-user-config --format table --folder-rows 30
```

The [second README table](../README.md#what-you-see) keeps the same measurement but distributes its source across folder rows. Larger backend areas receive more detail, while some smaller areas remain grouped. Parent and child contributions are not counted twice. Direct files can have their own row when a parent expands.

Compare the mixed production/test rows under `web/src/` and `workers/` with the separate main/test folders under `backend/`. The folder layout changes where the two purposes appear, not their meaning. `Test/prod` divides test code by production code; a dash means there is no production denominator. It does not measure test coverage, and the total ratio is computed from total counts rather than the average of row ratios.

To inspect only the frontend while retaining the root's configuration and module context:

```sh
locmetry --root acme-orders --subfolder web/src --no-user-config --format table --folders
```

Changing `--root` to `acme-orders/web/src` would establish a different scan root and could lose the project policy or enclosing module descriptor. [Selection diagnostics](guides/selection.md#common-issues-and-useful-queries) explains that failure mode. [Folder reporting](guides/reports.md#look-inside-large-folders) covers the row budget, saved tables and deeper expansion.

## Trace a classification decision

```sh
locmetry explain web/src/orders/Order.test.ts --root acme-orders --no-user-config --format json
```

For that path in the assumed layout, the JavaScript test-file convention decides `test` / `test` with rule `builtin:js-test-file`. The explanation exposes the matching convention and available module evidence. It lets you distinguish a deliberate decision from a surprising total before changing configuration.

The same policy produces several different kinds of decision:

| Representative path | Decision | Why |
| --- | --- | --- |
| `backend/orders/src/main/kotlin/domain/Order.kt` | production / implementation | A selected Gradle descriptor supplies the conventional source layout. |
| `web/src/orders/Order.test.ts` | test / test | Test conventions run before ordinary rules and module inference. |
| `scripts/imports/orders/order.py` | production / tooling | The project rule supplies the local intent. |
| `gradlew` | unclassified / third-party-wrapper | The build-wrapper convention deliberately classifies it. |
| `prototypes/pricing/candidate_1.py` | unclassified / unclassified | No earlier stage claims it, so it reaches fallback. |

Overrides run first, followed by test/build conventions, ordinary rules, module inference and fallback. Consequently, an ordinary tooling rule does not override a matching test convention. Use an override only when evidence justifies changing that precedence for a specific path. See [matching and precedence](PRODUCT.md#matching-and-precedence) for the full contract and wrapper patterns.

## Review what remains unresolved

```sh
locmetry --root acme-orders --no-user-config --format table --verbose
```

The experiments belong in the unclaimed review list; the wrapper does not. Both remain in Unclassified counts. If the owner later confirms that an experiment should remain deliberately unclassified, a narrow rule with an explicit category and reason records that decision and removes it from the unclaimed list without excluding its code. If its role is still unknown, leave the question open. A clean review list alone does not establish correct classification.

Verbose output also exposes non-source totals, inferred modules, unused policy and unrecognized extensions. A scope exclusion that matches nothing can reveal a misspelling. Actual read/parse failures remain visible without verbose mode and produce a partial measurement with exit status 2. Unclaimed code and an unrecognized image alone do not make the report partial. [Coverage interpretation](guides/reports.md#unavailable-is-not-zero) distinguishes these states.

## Save the evidence and ask a narrower question

```sh
locmetry --root acme-orders --no-user-config --files --output acme-orders-report.json
```

This saves full JSON outside the project, with per-file counts and deciding rules, and emits nothing on stdout. The full report also includes the review list and additive breakdown rows by direct folder, kind, purpose, category and language. The compact headline produced by piping a default measurement does not contain all of that evidence.

For example, use the optional external `jq` tool to sum source test code by language across all folders:

```sh
jq '[.breakdown[] | select(.kind == "source" and .purpose == "test")]
    | group_by(.language)
    | map({language: .[0].language, code: (map(.code) | add)})' acme-orders-report.json
```

Grouping before summing matters: one language can have many folder rows. Other useful questions include which files remain unclaimed and how production/test totals split beneath `web/`. The [saved-report queries](guides/reports.md#query-a-saved-report) demonstrate those operations without a second scan.

## Share a report and follow changes

```sh
locmetry --root acme-orders --no-user-config --format markdown --output acme-orders-report.md
```

Markdown includes purpose/category summaries, languages, recursive folder detail, review findings and coverage. It suits a review or wiki where readers need more than the terminal overview. The explicit format matters: `--output acme-orders-report.md` alone would still save JSON. This command performs another measurement; it is not a conversion of the previously saved JSON.

To follow changes, capture full reports on either side of the work:

```sh
locmetry --root acme-orders --no-user-config --output acme-orders-before.json
# After changing the source or its measurement policy:
locmetry --root acme-orders --no-user-config --output acme-orders-after.json
locmetry compare acme-orders-before.json acme-orders-after.json
```

Interpret net change together with coverage and policy differences. Reports describe working copies, not Git commits, and a changed classification policy can move code between purposes without changing source text. The [comparison guide](guides/compare.md) covers those boundaries. For automated checks, use the [CI recipe](guides/reports.md#continuous-integration), which preserves measurement failure and partial-result status before applying a policy threshold.
