# Development Plan

[PRODUCT](PRODUCT.md) owns supported requirements. This file indexes open work and decision gates; it is not a checkpoint log. Read the relevant linked design only when working on that item. Deferred items do not authorize implementation.

## Windows Support — Follow-up

- [ ] Publish the Scoop manifest from [packaging/scoop](../packaging/scoop) in a bucket repository and update it per release like the Homebrew tap; 0.9.0 ships the Windows zip, and `render-manifest.sh 0.9.0` renders a valid manifest. Add the update step to the release handback in `scripts/release.py` once the bucket exists.
- [ ] Generate PowerShell tab completion, together with the other shells; deferred by the owner on 2026-10-21.
- Accept UTF-16 configuration and workspace files, and add PowerShell equivalents of the `jq` recipes, only when Windows users ask for them. Windows on ARM stays excluded while x86_64 emulation serves it. The [Windows support record](proposals/archive/windows-support.md) owns the rationale.

## Accepted Proposals — Implementation Order

Decided 2026-10-23 by the owner: implement the accepted proposals in this order, each with red/green TDD, and finish each before starting the next. Each step implements its proposal's required design and acceptance; optional parts stay in the proposal unless named here. A finished step retires its proposal by the [retirement procedure](proposals/README.md#retirement-procedure). Steps 1 to 3 change `--help`, guides or report fields, so the next release needs an [agent regression check](agent-regression-checks.md).

1. [x] Framework layouts, implemented and verified on 2026-10-23; the [archived record](proposals/archive/framework-layouts.md) holds the design and links the [detector run](research/framework-layouts.md#detector-run).
2. [x] Module credit by layout coverage, implemented and verified on 2026-10-23, including the `explain` note; the [archived record](proposals/archive/module-credit-by-coverage.md) holds the design.
3. [x] Language corrections, implemented and verified on 2026-10-23; the [archived record](proposals/archive/language-corrections.md) holds the design.
4. [x] Drafted the [generated-code markers proposal](proposals/generated-code-markers.md) on 2026-10-23, from the [marker measurements](research/generated-code-markers.md) and Linguist's checks as the [prior-art record](research/classification-prior-art.md) proposed; it awaits the owner's decision under Proposals — Under Discussion.

## Configuration — Follow-up

- [ ] Decide whether an unloaded `.locmetryrc` is project content. [PRODUCT](PRODUCT.md#project-configuration) excludes only the loaded configuration, so `--config OTHER` selects the scan root's `.locmetryrc` and reports it as an unrecognized file: coverage differs by one file from a discovery run of the same tree, which misleads anyone comparing a draft policy with the committed one. Two policy test drives hit this. Recommendation: exclude every file named `.locmetryrc` as policy, which also covers inactive ones in subfolders and nested projects, and record the coverage change as **Breaking**.
- [ ] Decide whether `django` keeps the flat layout's non-package names: djangoproject.com's `docs` app, 3,968 lines, stays unclaimed because setuptools never treats `docs` as a package. Recommendation: keep the shared list; the gap is visible, and an app named like a documentation folder is rare.
- Deferred framework conventions: Storybook stories and Jest manual mocks, deferred by the owner on 2026-10-23 because their purpose needs a decision and other ecosystems were not searched; and `requirements.txt` as a Python descriptor, which needs its own evidence because documentation and tool folders carry such files too. The [archived proposal](proposals/archive/framework-layouts.md) records the measurements.
- [ ] Make the Go layout skip `vendor/`, `testdata/` and folders starting with `_` or `.`, which `go help packages` excludes from `./...`; Loki's 3.16 million vendored lines count as Go production today. Changing an existing detector needs a new configuration `version` or an explicit compatibility decision.
- Candidates from the [policy test drives](research/policy-test-drives/README.md#findings), each seen in one to four projects and without a decision: Gradle `buildSrc` and Maven failsafe test conventions, reasons on scope exclusions, an `except` list for entries whose carved-out files fall through to a later stage, and rules that apply only to files no other stage decides. A built-in candidate needs the [research checks](../CLAUDE.md#research-and-experiments) before it is proposed.
- [ ] Say in the [measurement semantics](measurement.md) that tracked files under an ignored path are not measured, because selection reads ignore files, not the Git index; two projects lost tracked files this way.

## Known Display Follow-up

- [ ] Fold BASH into Shell in human language tables, as TSX folds into TypeScript; JSON keeps both names. Tokei counts extensionless Bash scripts as BASH and `.sh` scripts as Shell, which split one project's scripts over two rows in four projects measured on 2026-10-22 and 2026-10-23. A [language correction](proposals/archive/language-corrections.md) does not fit: neither name is wrong, and globs cannot select files by shebang.
- Folder-partition convergence remains deferred: adaptive folder detail and JSON `unclassified_folders` intentionally use different policies. Revisit only if a workflow demonstrates confusing divergence, with explicit compatibility review. The [adaptive-folder decision](proposals/archive/adaptive-folder-detail.md#existing-partition-and-feasibility) owns the rationale.

## Ideas — Not Scheduled

- [Analytics cookbooks](plans/analytics-cookbooks.md): removed 2026-10-21; recreate them, refocused on questions native output cannot answer, only on the owner's decision. The design names the recovery commit and every removed file and pin.
- [Conditional measurement/comparison capabilities](plans/measurement-quality-and-net-change.md#conditional-follow-ups-and-preserved-rationales): consumer-driven candidates and their triggers, including [per-file comparison](plans/measurement-quality-and-net-change.md#per-file-comparison), generated/minified evidence and duplicate evidence. Ancestor discovery remains outside accepted scope.
- Built-in language corrections for project-independent names, such as `proguard-rules.pro`, in a set selected by `version`: add one only when the name recurs across measured projects; the first was observed once ([language corrections](proposals/archive/language-corrections.md)).
- Revisit native file comparison only when recurring usage justifies expanding the CLI. The optional [file-transition helper](../scripts/compare-locmetry-files.sh) ships with releases and Homebrew, while its `jq` prerequisite remains a user-installed choice; avoiding that prerequisite alone does not activate native implementation.
- JSON finding grammar, such as “1 unrecognized files”, remains an optional separate cleanup. The comparison-readability changes intentionally preserved JSON message text.
- [Ineffective-option compatibility policy](proposals/archive/cli-option-clarity.md#deferred-compatibility-decision): blanket rejection remains deferred. Reopen only with a separate compatibility assessment of reusable option sets, terminal-to-pipe transitions and existing script expectations.

- [File-detail option naming](proposals/archive/cli-option-clarity.md#optional-and-excluded-scope): retain `--files`; revisit `--include-files` only if task evidence shows recurring confusion with input selection.

## Proposals — Under Discussion

- [Generated-code markers](proposals/generated-code-markers.md), draft of 2026-10-23: module-claimed production with a documented marker or a `.gitattributes` `linguist-generated` declaration becomes `production:generated`. The owner accepted Java's type-level `@Generated` as a required marker on 2026-10-23. Open decisions: accept the other required markers; turn the unreleased `dotnet-generated` convention into the Roslyn marker before release; defer the optional markers, Django's migration header, OpenAPI Generator's manifest, `@generated` and tool-specific headers.

The [proposal index](proposals/README.md) distinguishes active designs from historical decisions.

## Historical Evidence

- [CLI option clarity and applicability](proposals/archive/cli-option-clarity.md): reviewed naming and output-view contract, owner amendment and verification.
- [Windows support](proposals/archive/windows-support.md): reviewed platform contract and its verification.

- [Complete CLI help coverage](plans/archive/full-help-coverage.md): implementation scope and experiment evidence; [help design](help-design.md) owns the current feature inventory and contributor guidance.
- [Default report output decision](proposals/archive/default-report-output-proposal-redacted.md): reviewed design and acceptance evidence.
- [Development and release snapshot through 2026-10-16](plans/archive/development-plan-2026-10-16.md): completed work, published releases and their verification links.
- [Measurement quality and subsequent implementation checkpoints](plans/archive/measurement-quality-and-net-change.md): S1–S5, reviews, accepted decisions and completion evidence.
- [Test/production ratio design](plans/archive/test-production-ratios.md): research behind neutral, unbanded ratios.
- [Research index](research/README.md): dated findings and source provenance.

Contributor workflow and check selection belong in [CLAUDE.md](../CLAUDE.md). Update this index only for material scope/status changes; do not duplicate historical completion narratives here.
