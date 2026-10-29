# Compare Two Immich Releases

*2026-10-21T03:17:02Z by Showboat 0.6.1*
<!-- showboat-id: 0b1f7748-841c-441a-b682-63cf1a407f87 -->

Run each Bash block from the Locmetry repository root after `cargo build --locked` and `bash scripts/fetch-showcase.sh`; the [showcase index](README.md) explains the environment. The comparison spans one minor release line of real development, from `v3.2.0` to `v3.3.4`, both measured with the same reviewed policy.

## Compare two releases

Save a full report per release outside the measured roots, then compare the saved files. `sed` removes only the two measurement-time lines.

```bash
source scripts/showcase-env.sh
locmetry --root v3.2.0/immich --output v3.2.0.json
locmetry --root v3.3.4/immich --output v3.3.4.json
locmetry compare v3.2.0.json v3.3.4.json --format table --no-color \
  | sed '/^Before measured at:/d; /^After measured at:/d' \
  | demo_wrap
```

```output
Net change: immich → immich

 ──────────────────────────────────────────────────────────────────────────────────────────────
                          Lines           │           Code            │         Change
 Kind            before    after   change │  before    after   change │ Comments  Blanks  Files
 ─────────────────────────────────────────┼───────────────────────────┼────────────────────────
 Source         322,048  333,302  +11,254 │ 273,477  282,671   +9,194 │     +416  +1,644    +12
 Data           231,043  245,954  +14,911 │ 224,465  239,213  +14,748 │       +4    +159     +3
 Documentation   12,293   12,532     +239 │     793      793        0 │     +150     +89     +3
 ─────────────────────────────────────────┼───────────────────────────┼────────────────────────
 All counted    565,384  591,788  +26,404 │ 498,735  522,677  +23,942 │     +570  +1,892    +18

 ──────────────────────────────────────────────────────────────────
                         Lines           │           Code
 Purpose        before    after   change │  before    after  change
 ────────────────────────────────────────┼─────────────────────────
 Production    207,996  210,697   +2,701 │ 177,406  179,495  +2,089
 Test           96,227  103,432   +7,205 │  81,161   87,011  +5,850
 Other          17,825   19,173   +1,348 │  14,910   16,165  +1,255
 ────────────────────────────────────────┼─────────────────────────
 Source total  322,048  333,302  +11,254 │ 273,477  282,671  +9,194

 Test/prod (Code): 0.46 → 0.48 (before → after)

 ───────────────────────────────────────────────────────────────────────────────
                                      Lines           │           Code
 Category                    before    after   change │  before    after  change
 ─────────────────────────────────────────────────────┼─────────────────────────
 other:generated             17,825   19,173   +1,348 │  14,910   16,165  +1,255
 production:build-tooling       864      886      +22 │     749      772     +23
 production:deployment          893      938      +45 │     707      743     +36
 production:distribution        337      695     +358 │     264      567    +303
 production:implementation  196,005  198,194   +2,189 │ 167,666  169,318  +1,652
 production:native            8,783    8,870      +87 │   7,131    7,206     +75
 test:test                   80,595   87,800   +7,205 │  67,860   73,710  +5,850
 ─────────────────────────────────────────────────────┼─────────────────────────
 Source total               322,048  333,302  +11,254 │ 273,477  282,671  +9,194

 ──────────────────────────────────────────────────────────────────
                         Lines           │           Code
 Language       before    after   change │  before    after  change
 ────────────────────────────────────────┼─────────────────────────
 C++               310      295      -15 │     266      251     -15
 CSS               797      818      +21 │     673      691     +18
 Dart           89,812   90,871   +1,059 │  75,239   75,726    +487
 Dockerfile        323      369      +46 │     230      267     +37
 Groovy            171      173       +2 │     137      139      +2
 HCL               264      262       -2 │     227      225      -2
 JavaScript     23,329   23,273      -56 │  19,804   19,746     -58
 Kotlin          4,549    4,604      +55 │   3,761    3,810     +49
 Python          4,649    6,184   +1,535 │   3,688    4,888  +1,200
 SQL             9,421   10,534   +1,113 │   8,617    9,699  +1,082
 Shell             564      565       +1 │     403      404      +1
 Svelte         18,909   19,014     +105 │  17,339   17,442    +103
 Swift           3,842    3,874      +32 │   3,077    3,103     +26
 TypeScript    164,598  171,956   +7,358 │ 139,620  145,884  +6,264
 ────────────────────────────────────────┼─────────────────────────
 Source total  322,048  333,302  +11,254 │ 273,477  282,671  +9,194

 Folded: BASH → Shell, TSX → TypeScript.

 ─────────────────────────────────────────────────────────────────────────────
                                    Lines           │           Code
 Folder                    before    after   change │  before    after  change
 ───────────────────────────────────────────────────┼─────────────────────────
 deployment (both)            264      262       -2 │     227      225      -2
 e2e (both)                15,485   15,491       +6 │  13,168   13,174      +6
 machine-learning (both)    5,132    6,698   +1,566 │   4,069    5,291  +1,222
 mobile (both)             98,947  100,095   +1,148 │  82,619   83,183    +564
 packages (both)           12,060   12,893     +833 │   9,388   10,055    +667
 server (both)            122,378  129,296   +6,918 │ 105,842  111,901  +6,059
 web (both)                67,048   67,833     +785 │  57,577   58,255    +678
 ───────────────────────────────────────────────────┼─────────────────────────
 Source total             322,048  333,302  +11,254 │ 273,477  282,671  +9,194

 Folders are recursive. Do not sum parents and children.

Only in one report: 2 folders only before, 16 folders only after.

▸ Measurement notes

- Net change = after − before, from the two saved reports.
- Unreadable files add nothing to the recorded counts: their lines are unknown, not zero.
- A folder seen in only one report may be newly visible, not newly created.
- Unrecognized files: same 437 paths in both reports. Not counted on either side.
- Folder totals containing these files cover counted files only. Per-folder detail is in the JSON field
'findings[].folder'.

▸ Findings

- presence_changed (before): Folder 'mobile/lib/interfaces' appears only in the before report. It may be deleted or no
longer measured.

- presence_changed (before): Folder 'mobile/lib/mixins' appears only in the before report. It may be deleted or no
longer measured.

- presence_changed (after): Folder 'mobile/lib/data/store' appears only in the after report, with 1 folder inside it. It
may be new or newly measured.

- presence_changed (after): Folder 'mobile/lib/presentation/widgets/slideshow' appears only in the after report. It may
be new or newly measured.

- presence_changed (after): Folder 'mobile/lib/presentation/widgets/sliver_app_bar' appears only in the after report. It
may be new or newly measured.

- presence_changed (after): Folder 'mobile/packages/ui/lib/extensions' appears only in the after report. It may be new
or newly measured.

- presence_changed (after): Folder 'mobile/packages/ui/lib/src/components/settings' appears only in the after report. It
may be new or newly measured.

- presence_changed (after): Folder 'mobile/test/data' appears only in the after report, with 2 folders inside it. It may
be new or newly measured.

- presence_changed (after): Folder 'mobile/test/medium/utils' appears only in the after report. It may be new or newly
measured.

- presence_changed (after): Folder 'mobile/test/presentation/pages' appears only in the after report. It may be new or
newly measured.

- presence_changed (after): Folder 'mobile/test/presentation/widgets/slideshow' appears only in the after report. It may
be new or newly measured.

- presence_changed (after): Folder 'mobile/test/widgets' appears only in the after report, with 2 folders inside it. It
may be new or newly measured.

- presence_changed (after): Folder 'web/src/lib/components/memories' appears only in the after report. It may be new or
newly measured.

No recorded differences in measurement settings. Reports do not fingerprint source, ignore files or executable builds.
This does not prove source-only growth.

```

Most of the added source is test code, so Test/prod rises. Data grows faster than source; the content-kind table keeps that growth out of the source change. Folder rows locate the work, mostly in the server. The findings list folders that exist in only one release; they prompt a check that the measured scope did not change, and they are not errors.

## Focus on one workspace member

Workspace reports can be compared member by member. `--member` selects one root by its exact name; totals and detail then cover that member, while input metadata still describes the whole workspace. The second `sed` expression omits the findings, which repeat the folder presence changes shown above.

```bash
source scripts/showcase-env.sh
locmetry workspace --workspace-file v3.2.0/immich/workspace.toml --output before.json
locmetry workspace --workspace-file v3.3.4/immich/workspace.toml --output after.json
locmetry compare before.json after.json --member mobile --format table --no-color \
  | sed '/^Before measured at:/d; /^After measured at:/d; /^▸ Findings/,$d' \
  | demo_wrap
```

```output
Net change: e2e + machine-learning + mobile + packages + server + web → e2e + machine-learning + mobile + packages +
server + web
Member: mobile (both). Counts and detail cover this member only.
Input metadata, measurement notes, report-level findings and partial status cover the whole workspace inputs.

 ──────────────────────────────────────────────────────────────────────────────────────────────
                          Lines           │           Code            │         Change
 Kind            before    after   change │  before    after   change │ Comments  Blanks  Files
 ─────────────────────────────────────────┼───────────────────────────┼────────────────────────
 Source          98,947  100,095   +1,148 │  82,619   83,183     +564 │     +236    +348     -6
 Data            32,795   43,669  +10,874 │  32,444   43,314  +10,870 │       +2      +2     +3
 Documentation      466      466        0 │       9        9        0 │        0       0      0
 ─────────────────────────────────────────┼───────────────────────────┼────────────────────────
 All counted    132,208  144,230  +12,022 │ 115,072  126,506  +11,434 │     +238    +350     -3

 ──────────────────────────────────────────────────────────────
                        Lines          │          Code
 Purpose       before    after  change │ before   after  change
 ──────────────────────────────────────┼───────────────────────
 Production    81,013   79,825  -1,188 │ 68,179  66,952  -1,227
 Test          17,934   20,270  +2,336 │ 14,440  16,231  +1,791
 ──────────────────────────────────────┼───────────────────────
 Source total  98,947  100,095  +1,148 │ 82,619  83,183    +564

 Test/prod (Code): 0.21 → 0.24 (before → after)

 ───────────────────────────────────────────────────────────────────────────
                                     Lines          │          Code
 Category                   before    after  change │ before   after  change
 ───────────────────────────────────────────────────┼───────────────────────
 production:build-tooling      229      231      +2 │    178     180      +2
 production:implementation  71,490   70,213  -1,277 │ 60,456  59,152  -1,304
 production:native           8,783    8,870     +87 │  7,131   7,206     +75
 test:test                  17,934   20,270  +2,336 │ 14,440  16,231  +1,791
 ───────────────────────────────────────────────────┼───────────────────────
 Source total               98,947  100,095  +1,148 │ 82,619  83,183    +564

 ──────────────────────────────────────────────────────────────
                        Lines          │          Code
 Language      before    after  change │ before   after  change
 ──────────────────────────────────────┼───────────────────────
 Dart          89,812   90,871  +1,059 │ 75,239  75,726    +487
 Groovy           171      173      +2 │    137     139      +2
 Kotlin         4,549    4,604     +55 │  3,761   3,810     +49
 Swift          3,842    3,874     +32 │  3,077   3,103     +26
 ──────────────────────────────────────┼───────────────────────
 Source total  98,947  100,095  +1,148 │ 82,619  83,183    +564

Member source lines: matched +1,148, entering 0, leaving 0. Total +1,148.
Member all-counted lines: matched +12,022, entering 0, leaving 0. Total +12,022.

Member source code: matched +564, entering 0, leaving 0. Total +564.
Member all-counted code: matched +11,434, entering 0, leaving 0. Total +11,434.

 ─────────────────────────────────────────────────────────────────────────
                                   Lines          │          Code
 Folder                   before    after  change │ before   after  change
 ─────────────────────────────────────────────────┼───────────────────────
 android (both)            5,124    5,181     +57 │  4,199   4,250     +51
 integration_test (both)     744      753      +9 │    543     551      +8
 ios (both)                3,935    3,967     +32 │  3,130   3,156     +26
 lib (both)               69,239   67,419  -1,820 │ 58,654  56,893  -1,761
 packages (both)           2,201    2,727    +526 │  1,770   2,218    +448
 pigeon (both)               397      414     +17 │    316     325      +9
 test (both)              16,843   19,170  +2,327 │ 13,613  15,396  +1,783
 ─────────────────────────────────────────────────┼───────────────────────
 Source total             98,947  100,095  +1,148 │ 82,619  83,183    +564

 Folders are recursive. Do not sum parents and children.

Only in one report: 2 folders only before, 15 folders only after.

▸ Measurement notes

- Net change = after − before, from the two saved reports.
- Unreadable files add nothing to the recorded counts: their lines are unknown, not zero.
- A folder seen in only one report may be newly visible, not newly created.
- Unrecognized files: same 230 paths in both reports. Not counted on either side.
- Folder totals containing these files cover counted files only. Per-folder detail is in the JSON field
'findings[].folder'.

```

In the mobile app, production code shrank while its tests grew, a change that whole-repository totals hide.
