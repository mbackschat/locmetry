# Agent Regression Desk Review: Framework Layouts, Generated Markers and Module Credit

Evaluated 2026-10-23 under the [agent regression checks](../agent-regression-checks.md#desk-review) by one fresh MEDIUM agent (Claude Code `sonnet` at `xhigh`), acknowledged by the owner. Control: `3bec0a3`, the last ledger candidate. Candidate: `be32810`, 48 commits later. The range adds framework layouts, .NET XAML and Blazor scoped CSS, module credit by layout coverage, `[[languages.corrections]]`, generated-code markers, Go layouts limited to what `./...` builds, an always-on `**/.locmetryrc` exclusion, the BASH to Shell fold in tables, folded and capped presence findings in human comparisons, the `--table-category` unmatched-entry warning and several error rewordings. Both revisions were built with `cargo build --release --locked` from `git archive` into separate scratch directories and run with `NO_COLOR=1`, `COLUMNS=100`, `LC_ALL=C` and an empty `XDG_CONFIG_HOME` on macOS. Fixtures were a copy of the synthetic [CLI workspace](../../examples/cli-workspace/README.md) and synthetic fixtures the reviewer created: a Go module with `vendor/`, `testdata/`, `_x/`, `.x/`, a generated `.pb.go`, a `.sh` script, an extensionless bash script, a nested `.locmetryrc` and a draft policy passed with `--config`; a mixed Next.js, Java, .NET, Django and Go tree with and without a `.locmetryrc`; a Helm and ProGuard tree for corrections; and the 12 frozen task fixtures of the benchmark. A desk review does not measure calls or tokens; sizes are bytes and token figures are labelled estimates.

## Verdict

No correctness regression was found. On the CLI workspace the default, `--verbose`, category-and-folder, split-language, workspace-roots and workspace-folders tables are byte-identical (Markdown differs by one note sentence about module credit), and on the 12 frozen task fixtures (15 project roots) counts, purposes, categories, coverage and per-file decisions are identical; the only differences are the new `**/.locmetryrc` entry in the effective exclusions and, on one fixture, a module credit of 8 code lines instead of 6. Exit statuses are identical for 81 of 88 invocations; the other seven use `[[languages.corrections]]`, which the control rejects. The classification changes (Go vendor code, framework layouts, generated markers, BASH fold) behave as the changelog and README say and change figures only where the new rules apply. Every flag, value, topic and command that help, guides, schemas and error remedies name exists in the candidate.

The cost that matters is reading, not behavior. Help is unchanged (+0.3%, all budgets hold), but the schemas grew 28% (`--schema config` +57.9%) and the configure guide 12.7%, and 41% of the schema growth is one language-name list that the new `LanguageCorrection` repeats in three schemas. Earlier campaigns read `--schema config` in every session of tasks 3, 7 and 8, so replaying them would add an estimated 5k to 16k input tokens per session, which exceeds the 110% median gate on paper.

A targeted agent run is warranted for tasks 3 and 8 (and optionally 2 and 7), and only the cost gates can settle it. Recommended order: remove the duplicated language list from the schemas first, since that is behavior-neutral and cuts the exposure by about 22 KB, then run the tasks on the resulting candidate. No run is needed for the Go, framework, marker, fold or `.locmetryrc` changes: no frozen task contains those inputs, and the behavior is deterministic and documented.

## Reading Cost

| Text | Control bytes | Candidate bytes | Change | Lines |
| --- | ---: | ---: | ---: | ---: |
| Root `-h` | 4,323 | 4,376 | +1.2% | 70 → 71 |
| Root `--help` | 12,560 | 12,572 | +0.1% | 270 → 270 |
| `workspace -h` | 2,425 | 2,478 | +2.2% | 40 → 41 |
| `workspace --help` | 6,844 | 6,844 | 0.0% | 147 → 147 |
| `compare -h` | 959 | 959 | 0.0% | 25 → 25 |
| `compare --help` | 4,731 | 4,731 | 0.0% | 96 → 96 |
| `explain -h` | 1,418 | 1,418 | 0.0% | 30 → 30 |
| `explain --help` | 3,339 | 3,339 | 0.0% | 82 → 82 |
| All 8 help texts | 36,599 | 36,717 | +0.3% | 760 → 762 |
| Guide `configure` | 12,914 | 14,552 | +12.7% | 179 → 201 |
| Guide `selection` | 10,382 | 10,382 | 0.0% | 135 → 135 |
| Guide `reports` | 18,842 | 18,842 | 0.0% | 216 → 216 |
| Guide `compare` | 13,931 | 14,142 | +1.5% | 129 → 129 |
| All 4 guides | 56,069 | 57,918 | +3.3% | 659 → 681 |
| Schema `config` | 28,857 | 45,568 | +57.9% | 748 → 1,267 |
| Schema `user-config` | 13,677 | 13,677 | 0.0% | 559 → 559 |
| Schema `report` | 62,499 | 80,256 | +28.4% | 1,961 → 2,499 |
| Schema `comparison` | 68,343 | 85,389 | +24.9% | 2,237 → 2,767 |
| Schema `explanation` | 17,106 | 19,663 | +14.9% | 554 → 633 |
| Schema `workspace` | 1,988 | 1,988 | 0.0% | 52 → 52 |
| All 6 schemas | 192,470 | 246,541 | +28.1% | 6,111 → 7,777 |

The control column equals the previous review's candidate column for help, guides and schemas, which confirms the revisions were built and captured the same way. Reading that tasks actually do:

| Typical read | Control bytes | Candidate bytes | Change |
| --- | ---: | ---: | ---: |
| Root `--help` + `compare --help` | 17,291 | 17,303 | +0.1% |
| Root `--help` + `explain --help` + `explanation` schema | 33,005 | 35,574 | +7.8% |
| Root `--help` + `report` schema | 75,059 | 92,828 | +23.7% |
| Root `--help` + `configure` guide + `config` schema | 54,331 | 72,692 | +33.8% |

Before ordinary measurement, comparison and explanation use an agent still reads only root `--help` plus one command's `--help`, and neither changed. The growth sits in the references that configuration, JSON and discovery tasks open. Output sizes also grew a little: a full JSON report gains about 1.2 KB of constant metadata per root (the conventions and the generated-marker list; 9,601 → 10,790 bytes without `--files` on the CLI workspace, 18,693 → 21,279 for its two-member workspace with `--files`), plus about 100 to 250 bytes per credited file; the headline grew by 3 bytes.

Where the schema growth comes from, measured on the candidate schemas: the `LanguageCorrection.language` list is the same 7,482-byte enum as `LanguageMapping.language` (7,264 bytes) plus `unrecognized`, repeated in the `config`, `report` and `comparison` schemas, 22.4 KB of the 54.1 KB total growth. The rest is the generated-marker and correction descriptions, 10 new module detectors and 5 new conventions listed in the descriptions, and the `module` and `language_correction` file fields.

## Exposure in Earlier Campaigns

The Git-ignored call logs of the eleven non-quarantined campaigns from 2026-10-20 to 2026-10-22 (Codex `gpt-6.1-sol` at `xhigh`, 326 session records, 310 of which start with root `--help`) were counted for `--schema` and `--guide` calls. The extra output is the candidate's growth in bytes for each call actually made, assuming agents behave the same; the token estimate is bytes divided by 4. The ledger medians of uncached input come from the trend table and the 0.8 check.

| Task | Records | Mean extra output per session | Estimated extra tokens | Reads `--schema config` | Control median uncached input (ledger) |
| --- | ---: | ---: | ---: | --- | --- |
| 7 discovery | 16 | +63.3 KB | ≈ 16,200 | 16 of 16 | 43,706 to 50,830 |
| 2 JSON and CI | 14 | +36.9 KB | ≈ 9,400 | 3 of 14, `report` schema 14 of 14 | 35,099 to 37,620 |
| 3 broader configuration | 30 | +24.5 KB | ≈ 6,300 | 30 of 30 | 32,572 |
| 8 config layout | 64 | +20.7 KB | ≈ 5,300 | 64 of 64 | 29,886 |
| 11 advanced policy | 16 | +17.1 KB | ≈ 4,400 | 8 of 16 | 38,420 to 38,581 |
| 9 simple configuration | 28 | +10.9 KB | ≈ 2,800 | 17 of 28 | 28,942 to 31,352 |
| 5 workspaces | 22 | +7.1 KB | ≈ 1,800 | 3 of 22 | 37,458 to 60,008 |
| 6 explanation | 14 | +5.6 KB | ≈ 1,400 | 2 of 14 | 27,486 to 29,273 |
| 4 saved comparison | 38 | +2.3 KB | ≈ 600 | 0 | 32,830 to 44,296 |
| 10 scope and views | 16 | +1.1 KB | ≈ 300 | 0 | 35,114 to 35,127 |
| 1 report interpretation | 62 | 0 | 0 | 0 | 20,068 to 20,805 |

The 110% gate allows +10% of the median. On paper tasks 3 and 8 would add about 19% and 18%, task 2 about 25% to 27%, task 7 about 32% to 37% and task 11 about 11%. These are arithmetic from past call patterns, not measurements: later turns hit the cache, agents may read less, and tool-output truncation in a harness could change what is actually read. Discovery (task 7) reads every schema and guide by design, so its growth is dominated by what the task asks for.

## Findings

| ID | Severity | Finding | Agent run |
| --- | --- | --- | --- |
| D1 | Risk, medium | Schemas grew 28% and `--schema config` 58%; 41% of the growth is a duplicated language-name list | Yes: tasks 3 and 8; optional 2 and 7 |
| D2 | Risk, low | New features have no inventory entry and no route from root help; language corrections are invisible in tables and coverage | No |
| D3 | Improvement | Go modules no longer claim vendored, test-data and underscore or dot paths; the shift is large and the default table does not say why | No |
| D4 | Risk, low | `explain` shows the new Go fallback without a reason and, with `hidden = false`, gives nested `.locmetryrc` files no cause | No |
| D5 | Improvement | Generated markers, module credit and framework layouts behave as documented and reclassify ordinary projects | No |
| D6 | Improvement | `.locmetryrc` files are always excluded; coverage and unrecognized figures drop | No |
| D7 | Improvement | BASH folds into Shell in tables and Markdown; JSON keeps both | No |
| D8 | Improvement | Human comparison findings fold and cap; JSON keeps all | Optional: task 5 |
| D9 | Improvement | `--table-category` and root help fixes after the previous review are in and verified | No |
| D10 | Neutral | A control executable rejects reports of the candidate; the candidate reads the control's | No |
| D11 | Neutral | Help budgets hold; `workspace -h` has 22 bytes of headroom | No |
| D12 | Improvement | Error remedies name existing options; four messages are more specific | No |

### D1 Schema and configure-guide growth

Severity: risk, medium. Evidence: the table above. `--schema config` 28,857 → 45,568 bytes, `report` +17,757, `comparison` +17,046, `explanation` +2,557, `--guide configure` +1,638 (a new section on `[[languages.corrections]]`, 22 lines, placed before the category section, so every configuration reader pays for it). In the candidate schemas `LanguageCorrection.language` repeats `LanguageMapping.language` entry for entry plus `unrecognized`; as a shared definition with one extra alternative it would remove about 7.5 KB from each of the three schemas that embed the configuration. Verified: sizes and the enum comparison. Plausible, not measured: the effect on calls and uncached input, per the exposure table. Agent run: yes, tasks 3 and 8 first (both read `--schema config` in every recorded session and have a ledger control from the 0.8 check), then 2 (reads the `report` schema in every session) and 7 (reads everything) if budget allows. Run after the list is shared, so the run measures the schema that will ship.

### D2 Inventory and routing gaps for new features

Severity: risk, low. Evidence: `git diff 3bec0a3 be32810` changes no line of [help-design.md](../help-design.md), so the feature inventory has no entry for language corrections, generated-code markers (`production:generated`, `builtin_generated`, explain's `Generated marker`) or module credit (`module`, explain's `Module credit`). The row "Understand inferred context" still says `largest_module_files` is for "spotting generated or vendored code", but files that carry a documented marker no longer appear there; the schema description of `largest_module_files` is unchanged and only the configure guide says so. Root `--help` changed by one phrase ("not JSON or Markdown"); its "Broader policy" paragraph names only "language mappings for unrecognized extensions". An agent reaches corrections through `--guide configure` (the verbose unclaimed note points there) and `--schema config`, where they are documented, and the guide's TOML example ran correctly: a `.tpl` file was counted as Go HTML and a `.pro` file left unrecognized, and `explain` printed `Language correction: helm-templates (…)`. Verified: the wording of help and guides, the example. Not visible in human output: a corrected file shows only as its new language in the table, `coverage.mapped_files` stays 0 and no coverage field counts corrections, so a reader of the table or Markdown cannot tell policy recounted it; `--files` and `explain` show it. Without configuration a ProGuard `.pro` file counts as Prolog, other and unclaimed in both revisions with no hint. Suggestion if the owner wants a route: one clause in "Broader policy" would cost about 100 bytes of the 378 left in the root `--help` budget. Agent run: no; a run would need a new task, and the guide route is documented.

### D3 Go layout narrowing

Severity: improvement, with an interpretation shift. Evidence on the Go fixture (`--format table`): Go production 23 → 12 code lines, Other 0 → 11, Test/prod 0.30 → 0.58; the unclaimed note went from one folder to six (`vendor/github.com/acme/lib`, root files, `.x`, `_x`, `testdata`, `sub`); the per-file records give `other` / `unclaimed` / `fallback` to `vendor/…`, `testdata/…`, `_x/…`, `.x/…` and `_skip.go`, and the Go module's credit fell from 23 to 12 lines. The default table shows the larger Other column and the lower production figure with no accompanying note; the unclaimed list needs `--verbose`. The route works once followed: the verbose note points to `--guide configure`, whose text asks whether vendored dependencies belong in the measurement. Better than before: hand-written code and vendored code no longer share one figure and vendored code appears in the review list instead of the "largest files" list. The change is **Breaking** in the changelog and in the README module table, so an agent comparing against a pre-change report sees Production −11 and Other +11 with total source unchanged and only a generic `policy_changed (both)`; both revisions report version 0.9.0, so the `counter_changed` finding that a version difference would add could not be exercised. Agent run: no frozen task has Go inputs.

### D4 Explanation gaps

Severity: risk, low. Evidence: `locmetry explain vendor/github.com/acme/lib/lib.go --format table` prints `Purpose: other`, `Category: unclaimed`, `Rule: fallback`, `Stage: fallback` and no `Module` line; nothing says that a Go module's layout no longer covers `vendor/` or `testdata/`. That fact is in the config schema's `builtin:go-module` text, the README table and the built-in detection reference. Second: with `hidden = false`, as the CLI workspace sets it, `explain service/.locmetryrc` now returns `not_selected` / `reason_not_recorded` ("The available evidence does not establish a cause.") where the control returned `hidden_policy` ("Enabling hidden paths selects this path."). The control's remedy would now be false, because enabling hidden paths still leaves the file excluded by `**/.locmetryrc` (verified: `--hidden true` gives `State: excluded`, `Patterns: **/.locmetryrc`), but the new answer is vaguer than the cause the tool could name; the control already answered the root `.locmetryrc` this way. This is the limit [PRODUCT](../PRODUCT.md#subfolder-measurement) states for single-setting probes: overlapping filters may leave the cause unknown. Agent run: no.

### D5 Markers, module credit and framework layouts

Severity: improvement. Evidence on the mixed fixture without configuration: unclaimed code 20 → 4 lines, Test 0 → 2, Production 16 → 30; `production:generated` appears in the category table for a Go `// Code generated` header, a Java top-level `@Generated`, a `*.designer.cs` name, an `<auto-generated>` comment and a `.gitattributes` `linguist-generated` line; `vitest.config.ts` is test, `vite.config.ts` and `next.config.ts` build tooling, Django `tests.py` test. `explain` prints `Generated marker: builtin:go-generated-header, from '// Code generated by protoc-gen-go. DO NOT EDIT.'` and, for a file an entry decided, `Module credit: scan root (go), whose layout covers this file`. Module credit behaves as documented: a rule that keeps `production` and names category `native` keeps the credit, a rule that makes the file `other` removes it, an npm module with no covered file drops out of `summary.build_modules` (control: `npm` with 0 code), and on the frozen task 10 fixture `summary.build_modules.npm.code` is 8 instead of 6 because a Python tool under `src/` is now credited. `classification.builtin_generated = false` turns the markers off and an unused correction is listed in `summary.unused_entries`. Two limits: the default Production total still includes marker-generated code (totals are unchanged by design), visible only with `--show-categories` or JSON; and the new `production:generated` shares its category name with `other:generated`, the policy-set-aside meaning the README terms table uses, so a reader needs the purpose prefix. Agent run: no; reclassifications are deterministic and documented, and no frozen task has such inputs. Not evaluated: false-claim rates of the new rules, which belong to the research records, not to this review.

### D6 `.locmetryrc` always excluded

Severity: improvement. Evidence on the Go fixture: `coverage.selected_files` 16 → 14 and unrecognized files 3 → 1 (only `go.mod`); with `--config draft.toml` the control listed `.locmetryrc`, `drafts/.locmetryrc` and `sub/.locmetryrc` as unrecognized and the candidate lists none; with `--config` pointing at a nested `.locmetryrc` the control again listed the unloaded root file. The effective exclusions show `**/.locmetryrc` in every full and headline JSON, also for trees without any `.locmetryrc` (the task fixtures without a project policy show five entries instead of four); no frozen scorer reads that list. A comparison of reports across the change reports a folder that held only a `.locmetryrc` as `presence_changed`, hedged as "may be deleted or no longer measured". Agent run: no.

### D7 BASH folded into Shell

Severity: improvement. Evidence: the control splits the Go fixture's scripts over `BASH` (two extensionless files with bash shebangs) and `Shell` (a `.sh` file with a `#!/bin/bash` shebang); the candidate shows one Shell row of 3 files with the note `Folded: BASH → Shell.` in the default table and in Markdown. `--split-languages` restores `BASH`, `--table-language bash` and `shell` both select the merged row, and JSON `source_languages` keeps both names. The `--split-languages` help names "TSX, JSX and C++ Header" as examples and does not mention BASH; the `Folded:` line carries the information. Task 8's fixture has only `.sh` files without shebang and task 9's has none, so their rows do not change. Agent run: no.

### D8 Comparison findings folded and capped

Severity: improvement. Evidence: comparing a workspace with a departing member prints one finding, "Member 'web' left the workspace measurement, with 6 folders and 1 package inside it. Its source may still exist.", where the control printed eight lines (six folders, the member, the package); the JSON `findings` array is identical (15 entries each). With 14 folders added, the candidate lists ten and one line "4 more folders appear only in the after report. '--format json' lists each one."; the control listed all 14. `compare --member web` folds the same way. The compare guide documents the folding; `compare --help` (unchanged, 4,731 bytes) does not. The new text matches task 5's departure oracle ("a departing member does not prove source deletion"). An agent that counts folders from the human table now needs the JSON. Agent run: optional, task 5 only, as insurance for the departure interpretation; task 4 has no presence findings.

### D9 Fixes carried over from the previous review

Severity: improvement. Evidence: the `--table-category` line in `-h` now says "comma-separated"; the root footer says `--table-*` options filter tables "not JSON or Markdown", and both formats warn `'--table-category' has no effect here. It applies only to table output. Use '--format table'.`; an entry that matches nothing warns `'--table-category' entry 'production:nope' matches no category in this measurement. '--show-categories' lists the measured ones.` and the table still lists the real categories under `Not shown (categories)`. Agent run: no.

### D10 Report compatibility

Severity: neutral. Evidence: the control executable rejects a full report the candidate produced (`unknown field 'builtin_generated', expected one of 'builtin_tests', 'builtin_build', 'builtin_modules', 'fallback'`); the candidate compares reports of both revisions in both orders. `schema_version` stays "3". [Report compatibility](../report-compatibility.md) states this rule ("read reports with an executable at least as new as their producer"). An agent uses one executable, so it is not exposed. Agent run: no.

### D11 Help budgets

Severity: neutral. Evidence: `cargo test --locked --test integration help::` in the candidate tree: 10 passed, including `help_stays_within_its_reading_budget`. Sizes against budgets: root `-h` 4,376 / 4,450; root `--help` 12,572 / 12,950; `workspace -h` 2,478 / 2,500; `workspace --help` 6,844 / 6,900; `compare -h` 959 / 990; `compare --help` 4,731 / 4,875; `explain -h` 1,418 / 1,460; `explain --help` 3,339 / 3,400. `workspace -h` has 22 bytes of headroom (99.1%), so its next factual edit trips the guard. Agent run: no.

### D12 Error remedies and wording

Severity: improvement. Evidence: all flags, values and topics named in help, guides, schemas and the 24 non-empty captured stderr texts exist (scan of `--option`, `--guide TOPIC`, `--schema TOPIC` and `--format VALUE` mentions; the only non-options are the glob `--table-*` and prose). Changed messages: `'--guide' is a standalone query and cannot be combined with a directory to measure`; `'--schema' … combined with '--verbose'`; `Subfolder 'service/Makefile' must be a directory`; `Output 'ws/acme-orders/out.json' must be outside the read-only scan root`; `workspace --show-folders` now tips `'--show-folders' is only available without a command, not with 'workspace'` instead of suggesting `--show-bars`. The 17 root-help examples ran against the CLI workspace in both revisions with identical exit statuses (all 0) and identical output apart from timestamps, the `**/.locmetryrc` entry, the new metadata and `module` fields in JSON, and the larger config schema; the executable-example tests for help pass, and `scripts/check-doc-examples.sh` passes against both release binaries. Agent run: no.

## Limits

Release builds on macOS only; colour, Windows, other widths than 100 columns and debug builds were not exercised. Fixtures are synthetic and small; the framework and marker checks show that the documented rules fire and what they change, not their false-claim rates. Both revisions report version 0.9.0, so the version-difference findings of a real release (`counter_changed`) were not seen. The reviewer ran the filtered tests `help::`, `guides::`, `readme::`, `wording::`, `schema::` and `builtin_detection::` in the candidate tree and the documentation-example script against both binaries, not the full suite, the recorded-guide replay or the private regression; those ran for the candidate when its commits were made. The exposure table is replay arithmetic over Codex `gpt-6.1-sol` call logs; it assumes the same reads, uses four bytes per token and ignores caching, so the cost conclusions in D1 are plausible, not measured. That the frozen scorers do not read the effective exclusion list is an inference from a search of the scoring scripts, not from running them. D2, D4 and the second limit of D5 are inferences from help text and one fixture each. Reports were compared per revision, so `compare` across the two executables was exercised only for pairs of reports both could read. The coordinating session reproduced the schema sizes of D1 and both `explain` answers of D4's second case independently.
