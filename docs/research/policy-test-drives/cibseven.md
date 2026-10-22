# CIB seven policy (2026-10-22)

Created by a MEDIUM-tier agent (Claude Code `sonnet` at `xhigh`) with `target/debug/locmetry` 0.9.0, `--no-user-config`; saved by the main session from the handback. Measured tree: [cibseven/cibseven](https://github.com/cibseven/cibseven) at `5a45b47ea226`, clean. The policy was created as an untracked `.locmetryrc`: 2 mappings, 7 categories, 14 rules; [cibseven.policy.toml](policies/cibseven.policy.toml) copies it after the main session's fix of 2026-10-23 below. Moved from the private research folder on 2026-10-23; the raw exports stayed there.

## Summary

| | Built-ins only | Policy |
|---|---|---|
| Production | 335,601 / 6,006 files | 390,380 / 6,741 |
| Test | 516,948 / 4,792 | 528,309 / 4,944 |
| Other | 69,295 / 894 | 4,605 / 12 |
| Test/prod | 1.54 | 1.35 |
| Unclaimed | 892 files / 68,912 | 7 files / 3,759 |
| Module note | 335,601 in 198 maven modules | 287,855 in 198 maven modules |

The AngularJS frontend (55,801 lines) has `package.json` but no `src/`, so no module claims it; a `frontend` rule makes it production. Mappings count BPMN, CMMN, DMN and XSD as XML data (1,284 files) and five Jenkinsfiles as Groovy. Other entries: test-runner config as tooling, frontend unit-test support, frontend build, client integration tests under `src/it` (failsafe), webapp integration tests under `src/main`, QA test support, QA perf framework and index gate as tooling, examples and demos, distribution launchers, CI pipelines, docs and CI helpers, OpenAPI FreeMarker templates as `api-docs` (37,747), a copied Apache Commons Codec as vendored (463).

## Finding 1

Zero confirmed cases. `.properties` count as INI through the built-in mapping; a user mapping wins over the built-in one, but switched the kind to documentation, so it was not applied.

## Finding 2

Six entries take 856 module-claimed files, 47,746 lines (module note minus 14.2%). Category-only: `openapi-templates` 37,747, `qa-test-tooling` 3,081, `examples-and-demos` 2,379 (43,207 lines). Modules with credit fell from 80 to 60 while the note keeps saying 198.

## Configuration issues

1. Category-only refinement forfeits module credit (43,207 lines).
2. No glob negation: six sibling globs to leave two files unclaimed.
3. `kind = ["source"]` leaves fixture data in the same folder with another purpose; guidance on when to omit `kind` would help.
4. Missing built-in test conventions for Maven failsafe: `**/src/it/**` and `*IT.java` (25 `*IT.java` files plus 10 support files were not test).
5. `--files` with `--format headline` is rejected (clear message; `--files --output FILE --print headline` works).

## Open questions with the agent's recommendations

1. `kitchen-sink.html` and `test-styles.less` (3,754 lines, the main unclaimed residue): `other` if orphaned, `frontend` if maintained. Main session: neither is referenced anywhere in the repository (`git grep`), which supports `other`.
2. `juel/` (a renamed JUEL fork, 3,931 lines): keep as production unless frozen upstream code.
3. Packages named `test` under `src/main` (66 files, 5,325 lines) counted as test by `builtin:test-directory`, some imported by production code: keep as test.
4. `qa/test-db-util`: test-support.
5. BPMN, CMMN, DMN and XSD as XML data: keep.
6. `PerfTest.java` (18 lines) test by name: no override.
7. Five unclaimed test-runtime lines: leave.
8. Duplicate variant trees (`run`/`run4`, `wildfly`/`wildfly28`, `invoice`/`invoice-jakarta`): leave measured, they differ.

Main-session spot check (2026-10-23): Test/prod 1.35, unclaimed 3,759 lines in 7 files, no entries that decided no file, module note as reported.

Applied 2026-10-23 (uncommitted in cibseven): a rule `unreferenced-style-demo` (other, new category `unreferenced`) placed before `frontend`, whose six sibling globs collapsed into `camunda-commons-ui/**`. Only those two files changed; unclaimed source fell from 3,759 to 5 lines.
