# Recorded CLI Guides

These guides are executable demonstrations made with [Showboat](https://github.com/simonw/showboat), following its command-and-captured-output workflow. They all use the checked-in [Acme Orders workspace](../../examples/cli-workspace/README.md), also used by the [sample tour](../sample-project-tour.md) and root README. No application build or fixture dependency installation is needed.

- [Measure a project](root.md): root selection, classification, folder views, reporting and discovery.
- [Measure a workspace](workspace.md): member policy, combined reports and export choices.
- [Compare saved measurements](compare.md): a visible source change, output formats, member selection and membership changes.
- [Explain a file](explain.md): deciding rules, selection states and policy experiments.

Contributors and agents should read the [authoring guidelines](../showboat-authoring.md) before adding or changing a recording.

## Run an example

Build the current executable with `cargo build --locked`. From the repository root, run a guide's complete Bash block. Its first line sources [cli-demo-env.sh](../../scripts/cli-demo-env.sh), which copies the fixture into a new temporary directory, makes `target/debug/locmetry` available, fixes terminal width and isolates personal configuration. The copy is deleted on exit. Set `LOCMETRY_BIN` to an absolute executable path to test another build. Every block is independent; reports are outside the measured roots.

Commands and captured output are real. `jq` projections select meaningful JSON fields, and visible `sed` expressions omit variable location/time footers or select report sections. `demo_wrap` wraps displayed text at whitespace to 120 characters; JSON is pretty-printed with jq. The verifier enforces a 120-character limit on fenced commands and output. Complete exported files retain their original metadata. Comparison table examples also trim trailing padding. These projections are part of the recorded command, not edits to its output. Explicit formats avoid depending on whether stdout is a terminal. Styling suppression is demonstrated; the guides do not verify visual palettes.

Bash and jq are needed to execute the examples. Showboat is needed only for recording or replay, and DuckDB's Python package only for the separately recorded [analytics demo](../analytics-demo.md). Use the pinned commands below to obtain these optional development tools through uv's external cache. Locmetry's Rust dependencies, runtime and releases do not include them. No virtual environment, installed package, Python dependency manifest or lockfile belongs in Git.

## Option coverage

Each row routes an option to its primary explanation. The workspace guide exercises all of its shared options too. The verification script checks that every long option in each command's current help occurs in that guide's executable blocks; this catches newly added options needing a demonstration. It does not claim to test every value or combination. Short aliases `-h`/`-V` retain their normal help/version meanings.

| Command | Options or arguments | Example |
| --- | --- | --- |
| Root | `--root`, positional directory, `--no-user-config`, `--format`, `--no-color`, `--no-bars` | [Whole tree](root.md#measure-the-whole-tree) |
| Root | `--folder-rows`, `--sort`, `--categories`, `--verbose` | [Folders and decisions](root.md#explore-folders-and-decisions) |
| Root | `--subfolder`, `--folders`, `--purpose`, `--bars=comments` | [Production subtree](root.md#focus-on-production-in-a-subtree) |
| Root | `--config`, `--hidden`, `--respect-ignore-files`, `--exclude` | [Selection overrides](root.md#override-selection) |
| Root | `--files`, `--output`, `--print`, `--bars` | [Export and overview](root.md#save-full-evidence-and-print-an-overview) |
| Root | `--depth`, Markdown and headline formats | [Output choices](root.md#choose-markdown-or-headline-output) |
| Root | `--help`, `--version`, `--schema`, `--guide` | [Discovery](root.md#discover-syntax-and-contracts) |
| Workspace | `--file`, `--root` | [Workspace file](workspace.md#use-a-workspace-file), [explicit roots](workspace.md#name-roots-directly) |
| Workspace | `--no-user-config`, `--no-color`, `--no-bars`, `--verbose`, `--categories`, `--sort`, `--format` | [Member policy and reports](workspace.md#use-a-workspace-file) |
| Workspace | `--purpose`, `--bars`, `--files`, `--output`, `--print`, `--depth`, `--help` | [Shares](workspace.md#select-purposes-and-show-shares), [exports](workspace.md#export-and-print-together), [Markdown](workspace.md#share-markdown-detail), [discovery](workspace.md#inspect-available-options) |
| Compare | `BEFORE`, `AFTER`, `--format`, `--no-color`, `--output` | [Known change](compare.md#capture-and-explain-a-known-change), [exports](compare.md#save-the-comparison) |
| Compare | `--member`, `--help` | [Member selection](compare.md#focus-on-a-workspace-member), [input contract](compare.md#respect-the-input-contract) |
| Explain | `PATH`, `--root`, `--no-user-config`, `--format`, `--no-color` | [Deciding rule](explain.md#trace-the-deciding-rule) |
| Explain | `--hidden`, `--respect-ignore-files`, `--exclude`, `--config` | [Selection overrides](explain.md#try-selection-overrides), [proposed policy](explain.md#compare-a-proposed-policy) |
| Explain | `--workspace`, `--help` | [Workspace and Markdown](explain.md#use-a-workspace-and-save-markdown), [syntax](explain.md#discover-explanation-syntax) |

## Verify and regenerate

After changing the executable, fixture or recorded guides, build once and replay:

```sh
cargo build --locked
uv run --no-project --python 3.11 \
  --with showboat==0.6.1 --with duckdb==1.5.6 \
  python scripts/check-recorded-guides.py
```

The same replay runs in the Linux documentation CI job on every pull request and push to `main`. The script checks option coverage, then verifies all four command guides and the analytics demo. Changed output or a failed command fails the check. Verification never rewrites the checked-in files. Unit/integration tests remain responsible for semantic correctness and edge cases; a replay match proves reproducibility, not that the output is correct.

For an intentional change, generate a candidate with updated output, inspect its diff, and replace the guide only after review:

```sh
uv run --no-project --python 3.11 --with showboat==0.6.1 \
  showboat verify docs/cli/root.md --output /tmp/locmetry-root-candidate.md
diff -u docs/cli/root.md /tmp/locmetry-root-candidate.md
```

A mismatch returns a nonzero status while producing the candidate; inspect the diagnostic to distinguish output drift from execution failure. Edit explanatory prose or commands when needed, then regenerate output by executing them. Do not hand-edit captured output. To recreate a document through Showboat's authoring commands, use `showboat extract docs/cli/root.md --filename /tmp/locmetry-root-recreated.md` and execute the emitted commands from the repository root. Showboat may refresh its document creation timestamp.

If a measurement change affects the sample counts, also refresh the README tables using the fixture's [maintenance instructions](../../examples/cli-workspace/README.md), update the tour's explanation and run `bash scripts/check-doc-examples.sh`. Analytics recipe checks and their optional tool versions are documented in the [DuckDB cookbook](../duckdb-cookbook.md#verification).
