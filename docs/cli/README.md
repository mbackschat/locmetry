# Recorded CLI Guides

These guides are executable demonstrations made with [Showboat](https://github.com/simonw/showboat), following its command-and-captured-output workflow. They all use the checked-in [Acme Orders workspace](../../examples/cli-workspace/README.md); the [Immich showcase](../showcase/README.md) demonstrates the same commands on a real project. No application build or fixture dependency installation is needed.

- [Measure a project](root.md): root selection, classification, folder views, reporting, discovery and policy review.
- [Measure a workspace](workspace.md): member policy, combined reports and export choices.
- [Compare saved measurements](compare.md): a visible source change, output formats, member selection and membership changes.
- [Explain a file](explain.md): deciding rules, selection states, policy experiments and refinements.

Contributors and agents should read the [authoring guidelines](../showboat-authoring.md) before adding or changing a recording.

## Run an example

Build the current executable with `cargo build --locked`. From the repository root, run a guide's complete Bash block. Its first line sources [cli-demo-env.sh](../../scripts/cli-demo-env.sh), which copies the fixture into a new temporary directory, makes `target/debug/locmetry` available, fixes terminal width and isolates personal configuration. The copy is deleted on exit. Set `LOCMETRY_BIN` to an absolute executable path to test another build. Every block is independent; reports are outside the measured roots.

Commands and captured output are real. `jq` projections select meaningful JSON fields, and visible `sed` expressions omit variable location/time footers or select report sections. `demo_wrap` wraps displayed text at whitespace to 120 characters; JSON is pretty-printed with jq. The verifier enforces a 120-character limit on fenced commands and output. Complete exported files retain their original metadata. These projections are part of the recorded command, not edits to its output. Explicit formats avoid depending on whether stdout is a terminal. Styling suppression is demonstrated; the guides do not verify visual palettes.

Bash and jq are needed to execute the examples. Showboat is needed only for recording or replay. Use the pinned commands below to obtain this optional development tool through uv's external cache. Locmetry's Rust dependencies, runtime and releases do not include them. No virtual environment, installed package, Python dependency manifest or lockfile belongs in Git.

## Option coverage

Each row routes an option to its primary explanation. The workspace guide exercises all of its shared options too. The verification script checks that every long option in each command's current help occurs in that guide's executable blocks; this catches newly added options needing a demonstration. It does not claim to test every value or combination. Short aliases `-h`/`-V` retain their normal help/version meanings.

| Command | Options or arguments | Example |
| --- | --- | --- |
| Root | `--root`, positional directory, `--no-user-config`, `--format`, `--no-color`, `--no-bars` | [Whole tree](root.md#measure-the-whole-tree) |
| Root | `--folder-rows`, `--sort`, `--show-categories`, `--verbose` | [Folders and decisions](root.md#explore-folders-and-decisions) |
| Root | `--subfolder`, `--show-folders`, `--table-purpose`, `--show-bars=comments`, `--table-language` | [Production subtree](root.md#focus-on-production-in-a-subtree), [languages](root.md#focus-on-languages) |
| Root | `--split-languages` | [Language variants](root.md#focus-on-languages) |
| Root | `--tables-only` | [Tables only](root.md#print-only-the-tables) |
| Root | `--config`, `--hidden`, `--respect-ignore-files`, `--exclude` | [Selection overrides](root.md#override-selection) |
| Root | `--files`, `--output`, `--print`, `--show-bars` | [Export and overview](root.md#save-full-evidence-and-print-an-overview) |
| Root | `--markdown-depth`, Markdown and headline formats | [Output choices](root.md#choose-markdown-or-headline-output) |
| Root | `--help`, `--version`, `--schema`, `--guide` | [Discovery](root.md#discover-syntax-and-contracts) |
| Workspace | `--workspace-file`, `--root` | [Workspace file](workspace.md#use-a-workspace-file), [explicit roots](workspace.md#name-roots-directly) |
| Workspace | `--no-user-config`, `--no-color`, `--no-bars`, `--verbose`, `--show-categories`, `--sort`, `--format` | [Member policy and reports](workspace.md#use-a-workspace-file) |
| Workspace | `--table-purpose`, `--show-bars`, `--split-languages`, `--tables-only`, `--files`, `--output`, `--print`, `--markdown-depth`, `--help` | [Shares](workspace.md#select-purposes-and-show-shares), [exports](workspace.md#export-and-print-together), [Markdown](workspace.md#share-markdown-detail), [discovery](workspace.md#inspect-available-options) |
| Compare | `BEFORE`, `AFTER`, `--format`, `--no-color`, `--output` | [Known change](compare.md#capture-and-explain-a-known-change), [exports](compare.md#save-the-comparison) |
| Compare | `--tables-only` | [Change tables](compare.md#print-only-the-change-tables) |
| Compare | `--split-languages` | [Language variants](compare.md#keep-language-variants-apart) |
| Compare | `--member`, `--help` | [Member selection](compare.md#focus-on-a-workspace-member), [input contract](compare.md#respect-the-input-contract) |
| Explain | `PATH`, `--root`, `--no-user-config`, `--format`, `--no-color` | [Deciding rule](explain.md#trace-the-deciding-rule) |
| Explain | `--hidden`, `--respect-ignore-files`, `--exclude`, `--config` | [Selection overrides](explain.md#try-selection-overrides), [proposed policy](explain.md#compare-a-proposed-policy) |
| Explain | `--workspace-file`, `--help` | [Workspace and Markdown](explain.md#use-a-workspace-and-save-markdown), [syntax](explain.md#discover-explanation-syntax) |

## Verify and regenerate

After changing the executable, fixture or recorded guides, build once and replay. A completed `cargo test --locked` already provides the debug CLI; the standalone build below is needed only when that test build has not run for the current source. No release build is required:

```sh
cargo build --locked
uv run --no-project --python 3.11 --with showboat==0.6.1 \
  python scripts/check-recorded-guides.py
```

The same replay runs in the Linux documentation CI job on every pull request and push to `main`. The script checks option coverage, then verifies all four command guides, followed by the [Immich showcase recordings](../showcase/README.md), which need the pinned checkouts from `bash scripts/fetch-showcase.sh`. `--set reference` replays only these command guides; without the checkouts, the default `--set all` fails each showcase document with the fetch command. It reports each document's result and elapsed time, followed by the total. Changed output or a failed command fails the check; remaining documents still run so one CI attempt collects all failures. CI reuses the debug executable built by its preceding Cargo tests.

Each document is replayed once with a separate candidate destination. Default temporary files are removed when the run ends, on success or failure; failure diagnostics are printed in the terminal. Use `--artifacts-dir /absolute/path/outside/checkout` to retain logs, failed candidates and Git-state evidence from that execution. Successful candidates are discarded in either mode. Validation errors and timeouts may leave no complete candidate; inspect the diagnostic before using one. CI uploads the logs, summary and available failed candidates as `showboat-failure-evidence`, retained for seven days. No generated candidate is automatically accepted or committed.

The verifier compares Git state before and after replay and fails on changed tracked content, staged changes or newly created untracked paths. Existing local edits are allowed if replay leaves them unchanged; ignored scratch/build files are outside this check. With `--artifacts-dir`, a mutation failure retains `git-before.txt` and `git-after.txt` for inspection. The verifier does not revert files automatically. Verification itself never rewrites the checked-in guides. Unit/integration tests remain responsible for semantic correctness and edge cases; a replay match proves reproducibility, not that the output is correct.

For an intentional change, generate a candidate with updated output, inspect its diff, and replace the guide only after review:

```sh
uv run --no-project --python 3.11 --with showboat==0.6.1 \
  showboat verify docs/cli/root.md --output /tmp/locmetry-root-candidate.md
diff -u docs/cli/root.md /tmp/locmetry-root-candidate.md
```

A mismatch returns a nonzero status while producing the candidate; inspect the diagnostic to distinguish output drift from execution failure. Edit explanatory prose or commands when needed, then regenerate output by executing them. Do not hand-edit captured output. To recreate a document through Showboat's authoring commands, use `showboat extract docs/cli/root.md --filename /tmp/locmetry-root-recreated.md` and execute the emitted commands from the repository root. Showboat may refresh its document creation timestamp.

If a measurement change affects the fixture's counts, update the [fixture's](../../examples/cli-workspace/README.md) independently known totals and run `bash scripts/check-doc-examples.sh`. If it affects Immich, replay the [showcase recordings](../showcase/README.md) and refresh the README excerpts with `python3 scripts/refresh-readme-showcase.py`.
