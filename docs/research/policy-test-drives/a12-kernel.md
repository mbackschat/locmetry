# Locmetry policy for a12-kernel and workspace of four sibling projects

Measured 2026-10-22 with the development build `target/debug/locmetry` (0.9.0), always with `--no-user-config`, by a MEDIUM-tier agent (Claude Code `sonnet` at `xhigh`). Saved by the main session from the agent's handback, because the agent's write tool refused report files. Measured trees: [mgm-tp/a12-kernel](https://github.com/mgm-tp/a12-kernel) at `cb66e51fa7ab` and [a12-kernel-mm](https://github.com/mgm-tp/a12-kernel-mm) at `d4ec70fcd80f`, both with the owner's uncommitted changes, so counts can differ from those commits; [a12-kernel-antlr4](https://github.com/mgm-tp/a12-kernel-antlr4) at `9ef99fa551b7` and [a12-kernel-bootstrap](https://github.com/mbackschat/a12-kernel-bootstrap) at `507ff0b345e7`, clean. Moved from the private research folder on 2026-10-23; the raw exports stayed there.

Created, and nothing else outside this directory: `a12-kernel/.locmetryrc` and `a12-kernel-workspace.toml` (one level above the four projects). Both are untracked and uncommitted. No `.locmetryrc` exists in the other three projects. The work in progress in a12-kernel and a12-kernel-mm was only read.

## 1. a12-kernel: built-in conventions versus policy

This repository is the open-source mirror, and its Java and TypeScript test suite is deliberately absent (`ANALYSIS-TESTS.md`; the Java `test` source set is empty on purpose in `buildSrc`). Test/prod is therefore not a quality signal. The 482 test lines are the codemod's Jest tests, fixtures and helpers.

Source totals are identical before and after: 2,205 files, 154,511 code lines. Data and documentation are unchanged (109 data files, 32 documentation files).

| Purpose | Built-in only (files / code) | With policy (files / code) |
| --- | ---: | ---: |
| Production | 2,139 / 152,132 | 2,157 / 152,492 |
| Test | 30 / 431 | 32 / 482 |
| Other | 36 / 1,948 (all `unclaimed`) | 16 / 1,537 (all `doc-snippet`) |
| Test/prod | 0.0028 | 0.0032 |
| Unclaimed source | 36 files, 1,948 code lines in 8 folders | none |

| Category | Built-in only (code) | With policy (code) |
| --- | ---: | ---: |
| production:implementation | 150,723 | 148,848 |
| production:build-tooling | 1,409 | 1,722 |
| production:tooling | 0 | 411 |
| production:embedded-data | 0 | 1,511 |
| test:test | 431 | 12 |
| test:fixture | 0 | 419 |
| test:test-support | 0 | 51 |
| other:doc-snippet | 0 | 1,537 |
| other:unclaimed | 1,948 | 0 |

Built-in-only observations:

- Module detection: 50 `gradle` modules (149,840 code lines) and 8 `npm` modules (883). All Java and TypeScript product code sits under `src/main/**` of Gradle modules; the layouts were right except for the cases in section 2.
- Test conventions: only `kernel-ts/codemod/**/__tests__/**` matches (30 files, test-directory convention). There is no `src/test` tree and no `*Test.java`.
- Generated code: none checked in. The ANTLR parsers are generated at build time: `kernel-core-parser` into `build/`, and `kernel-core-parser-web` into `src/main/js/internal/gen/**`, which its `.gitignore` excludes. `kernel-core-parser/src/main/java/.../grammar/gen/` holds six hand-written classes (1,117 lines) that extend the generated `Base_ConditionLexer`. `git grep` for generator headers found none.
- Vendored code: none. Every Java, TypeScript, JavaScript, Groovy and Batch file outside Gradle files, lint configs, the two `generateJSDoku.bat`, `jest.config.ts`, `scripts/*.js` and the codemod fixtures carries the project's own license header.
- Not countable: 630 unrecognized files: 434 `.st` and 54 `.stg` StringTemplate code-generation templates (4,827 lines by `wc -l`), 4 `.g4` ANTLR grammars, 104 `.png`. See issue 5.
- All 2,962 tracked files are selected; no tracked file is hidden by `.gitignore`.

## 2. Policy entries and evidence

Nine entries, all `kind = ["source"]`, verified by remeasure and by `explain` on one file per entry. Per-file transitions from `scripts/compare-locmetry-files.sh`: exactly 71 changed rows with zero code delta.

| Entry (stage) | Matches | Files / code moved | Evidence |
| --- | --- | ---: | --- |
| `gradle-convention-plugins` (rule), production/build-tooling | `buildSrc/src/main/**` | 4 / 313 from implementation | `buildSrc` is Gradle's convention-plugin build; the module build scripts apply its plugins. The gradle-module layout counted them as kernel implementation. |
| `ci-pipeline` (rule), production/tooling | `build-kernel/jenkins/**` | 2 / 109 from unclaimed | The pipeline runs `gradle build`, publishes `TEST*.xml` and loads `utils.groovy`: a CI gate. |
| `npm-pack-scripts` (rule), production/tooling | `scripts/**` | 2 / 49 from unclaimed | `prepack`/`postpack` of six published packages call these scripts to copy license files. |
| `lint-format-config` (rule), production/tooling | `kernel-ts/eslint-config/**`, `**/eslint.config.mjs`, `**/.prettierrc.js`, `kernel-ts/codemod/scripts/copyright.js` | 15 / 237 from unclaimed | Shared ESLint package imported by each package's config; the Gradle `npm_lint` task runs ESLint with `--max-warnings 0`. `copyright.js` is the `templateFile` of the notice rule. |
| `codemod-jest-config` (rule), production/tooling | `kernel-ts/codemod/jest.config.ts` | 1 / 16 from unclaimed | Runner configuration: tooling that runs tests. |
| `codemod-test-support` (rule), test/test-support | `kernel-ts/codemod/src/core/test-runner.ts`, `.../src/snapshotSerializer.ts` | 2 / 51 from implementation (npm module) | Used only by the Jest tests and `jest.config.ts`. |
| `embedded-tld-list` (rule), production/embedded-data | `kernel-rt/kernel-core-customfieldtype-impl/src/main/js/internal/utils/EmailVerifierUtils.ts` | 1 / 1,511 from implementation (gradle module) | Lines 41 to 1539 are an IANA TLD list (2021) in one string literal; about ten lines of logic. |
| `documentation-snippets` (rule), other/doc-snippet | `documentation/dev/assets/snippets/**`, `**/src/snippets/**` | 16 / 1,537 from unclaimed | Included by AsciiDoc tags and Javadoc `{@snippet}`; the conventions say they cannot be compiled or run in this repository. |
| `codemod-fixtures` (override), test/fixture | `kernel-ts/codemod/src/transforms/*/__tests__/__fixtures__/**` | 28 / 419 from test:test | Inputs read by `test-runner.ts`, not tests. An override because a rule loses to `builtin:test-directory` (probed, issue 2). |

Considered and left to the default: `kernel-tool/kernel-core-ontheflytesting` (a product feature), `RuntimeController.{java,ts}`, `TypedAccessorGenerator.java`, `FormatDefinitionZahl.ts` (hand-written), the six hand-written classes in `grammar/gen/`, `generateJSDoku.bat` (legacy, see open questions), and the extensionless `kernel-ts/codemod/bin/kernel-codemod` (2 lines; a mapping would need a second entry for purpose).

## 3. Verification performed

- Baseline `--files` export before editing, policy export after, and `scripts/compare-locmetry-files.sh`: 71 rows (documentation-snippets 16, lint-format-config 15, codemod-fixtures 28, gradle-convention-plugins 4, ci-pipeline 2, codemod-test-support 2, npm-pack-scripts 2, embedded-tld-list 1, codemod-jest-config 1). Source and all-counted totals identical.
- `explain --format json` on one file per entry: stages `rule` or `override`, `matched_patterns` and `reason` as expected; also through `--workspace-file`.
- Negative controls from a scratchpad `--config`: a rule matching no file is listed under "Entries that decided no file"; a rule on the fixtures is listed there too, and the file stays `builtin:test-directory`.
- "Production from build modules": gradle 149,840 to 148,016 (minus 313 + 1,511), npm 883 to 832 (minus 51).

## 4. Workspace result

```toml
version = 1
roots = [
    "a12-kernel",
    "a12-kernel-antlr4",
    "a12-kernel-bootstrap",
    "a12-kernel-mm",
]
```

`locmetry workspace --no-user-config --workspace-file ... --format table --show-categories --verbose` exits 0: 3,866 files, 301,830 source code lines.

| Member | Code | Production | Test | Other | Test/prod | Share |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| a12-kernel | 154,511 | 152,492 | 482 | 1,537 | 0.00 | 51.2% |
| a12-kernel-antlr4 | 145,624 | 35,373 | 31,846 | 78,405 | 0.90 | 48.2% |
| a12-kernel-mm | 1,464 | 1,247 | 217 | 0 | 0.17 | 0.5% |
| a12-kernel-bootstrap | 231 | 0 | 0 | 231 | - | 0.1% |
| Total | 301,830 | 189,112 | 32,545 | 80,173 | 0.17 | 100% |

Only a12-kernel runs with a project policy (`metadata.members[].config_path`). The workspace Test/prod and the 80,173 Other lines are almost entirely an artifact of antlr4 having no policy (78,405 of 78,636 unclaimed lines).

## 5. The other three projects

Drafts were run from a scratchpad with `--config`; nothing was saved in the projects.

**a12-kernel-antlr4 (ANTLR fork): yes, the largest improvement.** 78,405 of 145,624 code lines (54%, 1,091 files) are unclaimed: `runtime/CSharp/src` 17,878, `tool/src` 16,909, `runtime/Cpp` 16,664, `runtime/Java/src` 14,243, `runtime/Swift` 11,509, JS specs 788, `scripts` 201, Maven site 122. Cause: layouts the module detection cannot see (`<sourceDirectory>src</sourceDirectory>` in `tool/pom.xml` and `runtime/Java/pom.xml`; the Swift package at the root pointing into `runtime/Swift/Sources/Antlr4`; no matching descriptor for C++ and C#). Java Test/prod reads 29.10 and the project 0.90. Eight checked-in generated files (1,137 lines, `Generated from ... by ANTLR 4.x`). Twelve `*Runner.java` (1,084) and `states/**` (131) in `runtime-testsuite/test/**` are test harness, production tooling by the guide, needing an override. An 11-entry draft leaves 334 unclaimed lines (runtime/Cpp 169, runtime/Swift 165), Test/prod 0.28, Java Test/prod 0.51, no unused entries; it needs review by someone who knows the fork. Not countable: 40 `.stg`, 35 `.g4`, 23 `.dot`; 357 `runtime-testsuite/.../descriptors/*.txt` (9,666 lines of grammars and inputs) are Plain Text, so much real test content sits outside source totals. Hosting: a workspace root cannot take an external config (issue 8).

**a12-kernel-mm: yes, small.** All 1,033 "implementation" lines are `build-logic/src/main/kotlin/**` (8 files of Gradle build logic), the same pattern as a12-kernel's `buildSrc`. The three `test-mm*/build.gradle.kts` (58 lines) are build files of test-only modules, test support by role, needing an override. Draft: Production 1,189 build-tooling, Test 275 (Test/prod 0.17 to 0.23); the whole "Production from build modules" line disappears. No generated code or language collisions; 60 JSON models (100,842 lines) are data.

**a12-kernel-bootstrap: yes, trivial.** 231 code lines in 6 files, all unclaimed: `bootstrap.sh` (158), `gradle/public-repos.init.gradle` (9), a stub Gradle plugin under `patches/build-utilities-stub/src/main/groovy/**` (64). One tooling rule covers all; whether the stub should be `other` (patch payload) is the one judgment call.

## 6. Open questions for the owner

1. Documentation snippets (1,537 lines): `other`/`doc-snippet`, `test`, or `production`/`example`? Recommendation: keep `other`; nothing compiles or runs them here.
2. `test-runner.ts` and `snapshotSerializer.ts` as `test-support` versus production tooling? Recommendation: keep test-support; both work only inside Jest.
3. `EmailVerifierUtils.ts` as `embedded-data`? Recommendation: keep; 1,511 lines of data would otherwise read as logic.
4. Is the Jenkins pipeline still used by the mirror? Recommendation: keep as tooling; make it `other` if it is an internal leftover.
5. Is `generateJSDoku.bat` maintained? Recommendation: leave as is; add an `other` entry if confirmed dead.
6. Where should the antlr4 policy live? Recommendation: in the fork as a locally excluded file.
7. Cite the workspace before antlr4 has a policy? Recommendation: no.
8. StringTemplate (`.st`, `.stg`: 488 files, 4,827 lines) and `.g4` are not measured. Recommendation: accept and state it; it needs a Locmetry feature (issue 5).

## 7. Findings

### Finding 1: misrecognized language through an extension collision

Four groups, 179 files, 5,276 code lines, all in a12-kernel-antlr4; zero in the other three. Full list in [a12-kernel-antlr4-misrecognized-files.txt](policies/a12-kernel-antlr4-misrecognized-files.txt).

| # | Files / code | Recognized as | Really | Paths |
| ---: | ---: | --- | --- | --- |
| 1 | 172 / 5,026 | C Header | C++ headers (169 declare `namespace antlr4`, classes, templates or use `std::`); the backend uses "C++ Header" only for `.hpp` | `runtime/Cpp/runtime/src/**/*.h`, `runtime/Cpp/runtime/antlrcpp-ios/antlrcpp_ios.h` |
| 2 | 2 / 126 | Autoconf | CMake `configure_file` templates | `runtime/Cpp/cmake/antlr4-generator.cmake.in`, `runtime/Cpp/cmake/antlr4-runtime.cmake.in` |
| 3 | 1 / 2 | Autoconf | Python sdist `MANIFEST.in` | `runtime/Python3/MANIFEST.in` |
| 4 | 4 / 122 | Apache Velocity | Maven APT documentation processed by Velocity (arguable) | `antlr4-maven-plugin/src/site/apt/**/*.apt.vm` |

Case 1 splits the C++ runtime across a "C Header" row (5,026) and a "C++" row (11,027). Not counted as cases: Plain Text test descriptors, `.d.ts` files, and `.properties` labelled INI by Locmetry's built-in mapping.

### Finding 2: refining code a convention or module already decided replaces the whole decision

Mechanism as measured; precedence is overrides, built-in conventions, rules, build modules, fallback. Code decided by a build module can be refined with a plain rule; any entry that takes a module's file removes it from "Production from build modules", from `summary.build_modules[*].code` and from the largest-files-by-layout list, while `metadata.modules` still lists the module without a code figure. Code decided by a built-in convention (`builtin:test-directory`, `builtin:build-script`) needs an override. An entry sets purpose and category together; a category-only refinement cannot be expressed.

Applied in a12-kernel:

| Case | Entry | Beats | Files / code | Result |
| --- | --- | --- | ---: | --- |
| K1 `buildSrc` convention plugins | rule | `builtin:gradle-module` | 4 / 313 | implementation to build-tooling |
| K2 `EmailVerifierUtils.ts` | rule | `builtin:gradle-module` | 1 / 1,511 | implementation to embedded-data |
| K3 codemod test helpers | rule | `builtin:npm-module` | 2 / 51 | production to test/test-support |
| K4 codemod fixtures | override | `builtin:test-directory` | 28 / 419 | test:test to test:fixture |

Total 35 files, 2,294 code lines (1.5% of source). Module credit: gradle minus 1.2%, npm minus 5.8%.

Wanted but not applied (scratchpad drafts):

| Case | Project | Would beat | Files / code | Note |
| --- | --- | --- | ---: | --- |
| M1 `build-logic/src/main/kotlin/**` as build-tooling | a12-kernel-mm | `builtin:gradle-module` | 8 / 1,033 | Removes the entire "Production from build modules" line although five modules stay detected. |
| M2 `test-mm*/build.gradle.kts` as test-support | a12-kernel-mm | `builtin:build-script` | 3 / 58 | Override. |
| A1 generated `XPathLexer.py` | a12-kernel-antlr4 | `builtin:python-module` | 1 / 72 | Rule suffices; python credit 6,195 to 6,123. |
| A2 generated Python test parsers | a12-kernel-antlr4 | `builtin:test-directory` | 3 / 739 | Override for test/generated. |
| A3 other generated files | a12-kernel-antlr4 | unclaimed fallback | 5 / 326 | Plain rule, no credit lost. |
| A4 `*Runner.java` and `states/**` as production/tooling | a12-kernel-antlr4 | `builtin:test-directory` | 18 / 1,215 | Override; purpose change per the guide. |

Considered and default fine: `kernel-core-ontheflytesting`, the six hand-written `grammar/gen/` classes, the large hand-written files, `MigratorCli.java` and `kernel-rewrite/**`, the codemod sources apart from K3.

Suggested improvements: a refinement that keeps the deciding stage (category-only entry); or count module credit by layout regardless of a later entry; or a built-in convention for `buildSrc/**` and Gradle included builds such as `build-logic` (solves K1 and M1 without policy).

## 8. Policy configuration issues and missing features

1. Guides and schemas reject `--no-user-config` (low): `error: '--guide' is a standalone query and cannot be combined with '--no-user-config'`. Suggestion: accept it as a no-op for standalone queries, or name the exception.
2. The unused-entries note does not say why an entry decided nothing, and its absence is the only positive signal (medium): a rule preempted by `builtin:test-directory` and a rule matching nothing look alike. Suggestion: "matched N files, all decided earlier by X" versus "matched no file", and a positive "Unused entries: none" under `--verbose`.
3. A zero-code file is reported as unclaimed source (low): `Unclaimed source: 0 code lines in 1 file`. Suggestion: leave zero-code files out or label them.
4. Native `compare` shows only purpose-level change (low): `--show-categories` is rejected; category and rule moves needed the jq helper. Suggestion: category and rule transitions in native `compare`.
5. `[[languages.mappings]]` cannot name a language outside the backend list (medium): StringTemplate (488 files) and ANTLR `.g4` stay unrecognized; nearest names mislabel. Suggestion: a free-form language name with an explicit `kind`.
6. Any refinement of module-decided code removes its module credit (medium); see finding 2.
7. The workspace table does not show which members have a project policy (low): only JSON `config_path`. Suggestion: a verbose note per member.
8. A workspace member cannot take an external policy file (medium): `--config` is rejected with `workspace`, and `roots` are strings. Suggestion: `roots` entries with an optional `config` path.
9. A `reason` belongs to the entry, not to a matched pattern (low): `explain` prints a sentence about a different file. Suggestion: the guide could advise separate entries for unrelated files.

## Appendix: created `.locmetryrc`

[a12-kernel.policy.toml](policies/a12-kernel.policy.toml) copies the created `.locmetryrc`, and [a12-kernel.workspace.toml](policies/a12-kernel.workspace.toml) the workspace file, both on 2026-10-23; it declares the categories `tooling`, `doc-snippet`, `embedded-data`, `test-support` and `fixture`, eight rules and one override as listed in section 2.
