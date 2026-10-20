# Explain a File's Measurement

Use `locmetry explain PATH` when one file has an unexpected purpose, parser, or absence from a measurement. It applies the same configuration discovery, selection, counting and classification as an ordinary run. It traverses selected names for module inference but counts only the requested file. The [product contract](../PRODUCT.md#path-explanations) defines the machine states; [measurement semantics](../measurement.md) records counting limits.

Run `locmetry explain --help` for accepted arguments and `locmetry --schema explanation` for exact result structure. For project-wide policy work, use `locmetry --guide configure`; for count/coverage interpretation, use `locmetry --guide reports`.

## Start with One File

In a fresh temporary working directory:

```sh
mkdir demo
printf 'print(1)\n' > demo/main.py
locmetry explain main.py --root demo --no-user-config --format json
```

The JSON includes the canonical root, effective scope, loaded configuration paths and this complete `result` object. The example deliberately uses no project or personal configuration, so the Python file reaches fallback. Its counts are known; its purpose still needs a project decision. The public test suite checks this documented object against the executable.

<!-- explanation-result -->
```json
{
  "state": "counted",
  "file": {
    "path": "main.py",
    "language": "Python",
    "kind": "source",
    "purpose": "other",
    "category": "unclaimed",
    "rule": "fallback",
    "counts": { "code": 1, "comments": 0, "blanks": 0, "lines": 1 }
  },
  "decision": {
    "stage": "fallback",
    "matched_patterns": []
  }
}
```

Use `--format table` for a compact labeled explanation or `--format markdown` for a shareable evidence table. A terminal defaults to table; redirected stdout defaults to JSON. Every format describes the same target. Unlike measurement and comparison, explanation has no headline or `--output` option: redirect stdout to save its complete JSON. See [Output Routing](../PRODUCT.md#output-routing). `locmetry --schema explanation` exposes the generated structure, state meanings and a complete example for agents.

## Explain one file

```sh
locmetry explain src/main.rs
```

For Markdown from another scan root:

```sh
locmetry explain src/main.rs --root ./service --format markdown
```

For a workspace member, prefix the path with its member name:

```sh
locmetry explain api/src/main.rs --workspace-file repos.toml
```

Discover the complete explanation contract:

```sh
locmetry --schema explanation
```

## Choose the Correct Policy

| Task | Command |
| --- | --- |
| Explain a file from another working directory | `locmetry explain src/main.rs --root /path/to/repository` |
| Try a replacement project configuration | `locmetry explain src/main.rs --root /path/to/repository --config trial.toml` |
| Remove personal defaults from the experiment | `locmetry explain src/main.rs --no-user-config` |
| Check a workspace member | `locmetry explain api/src/main.rs --workspace-file repos.toml` |
| Check whether an additional exclusion applies | `locmetry explain vendor/lib.py --exclude 'vendor/**'` |

The target must use the exact directory-entry spelling, including case and Unicode form, even on a filesystem that accepts aliases. An alias fails with exit 1 rather than being diagnosed as filtered. The target is relative to `--root`, which defaults to the working directory. An explicit `--config` is relative to the working directory and replaces discovery of the root's `.locmetryrc`. Personal defaults still apply unless disabled. All ordinary single-root scope flags are accepted. Use the same settings as the measurement you are investigating.

With `--workspace-file`, the target starts with the member's folder name; roots in the workspace file are relative to that file. Only the queried member's policy is loaded, while normal membership checks still reject duplicate names or nested roots. `--root`, `--config`, `--exclude`, `--hidden` and `--respect-ignore-files` cannot be combined with `--workspace-file`. Paths in the result are member-prefixed, but matching globs remain relative to that member's own root.

For a report made with `--subfolder`, keep the project root and use the file's project-relative path in `explain`; no subfolder option is accepted or needed. A successful explanation does not establish inclusion in an earlier restricted report. Subfolder selection errors may suggest changing ignore/hidden settings, but those overrides also change module context across the project. See [subfolder measurement](../PRODUCT.md#subfolder-measurement) and the [diagnostic recipes below](#common-issues-and-useful-queries).

## Interpret the Evidence

| Observation | Meaning and next step |
| --- | --- |
| `counted`, unexpected purpose | Inspect `file.rule`, `decision.stage`, matching patterns and the optional entry reason. First-match precedence is overrides → conventions → rules → module → fallback. Change the deciding policy; a later rule cannot beat an earlier stage. |
| `counted`, module evidence | The selected descriptor triggered a conventional source layout. Its module path and detector identify the inference. Confirm that layout fits the project; Locmetry does not read the build configuration's contents. |
| `counted`, unexpected language | `file.language` is the host parser; `file.language_mapping`, when present, identifies the fallback mapping. Mappings only apply when the backend does not recognize the path. Embedded-language detail remains in `file.languages`. |
| `excluded` | The ordinary walk recorded a matching scope exclusion at the file or a pruned ancestor. `at` identifies that path; `patterns` lists the matching globs. Loaded configuration files also have literal exclusions. |
| `not_selected`, `ignore_policy` | A probe that disables only ignore filtering selects the file. The walker does not expose the winning ignore filename or line; inspect the applicable ignore files. |
| `not_selected`, `hidden_policy` | A probe that enables only hidden paths selects the file. Check the effective `scope.hidden` setting. |
| `not_selected`, `reason_not_recorded` | Available evidence cannot establish a cause. Multiple filters may mask each other; this is not a claim that no rule applies. Inspect the recorded scope and vary one setting at a time. |
| `unrecognized` | The file was selected, but no parser or mapping recognized it. Add a mapping only when an existing language counter fits its syntax. There is no known-zero count. |
| `parse_failed` or `unread` | Counts are unavailable. Inspect the stable failure reason and human detail, or file access. Handlebars encoding and notebook support are described in [measurement semantics](../measurement.md#counting). |
| `symlink` | The path or an ancestor is a symlink, which measurement does not follow. `at` identifies the link; the command does not inspect its target or claim which other filters apply. |

[Command outcomes](../PRODUCT.md#command-outcomes) defines the statuses: successful explanations include symlinks, filtered and unrecognized paths; unavailable target counts are partial, and invalid input or traversal aborts the command. A symlink ancestor ends inspection immediately, even if the remainder of the path would not exist at its destination.

This command is a current-filesystem diagnostic, not a historical explanation of a saved report. Ignore context and file contents can change between runs, including during the selection probes. An unrelated malformed notebook does not make this target partial, but traversal errors abort the explanation with exit 1. The command does not inventory every ignored file or expose a new configuration self-test language.

## Common issues and useful queries

These diagnostic examples use the [README sample project](../../README.md#what-you-see), from its parent directory. Personal defaults are disabled to isolate the sample policy.

### A subdirectory scan unexpectedly reports production files as unclaimed

`locmetry acme-orders/web/src` treats `src` as an independent root. It does not discover the repository's `.locmetryrc` or the parent `web/package.json`. Supplying `--config` alone does not restore the project's path base or ancestor module evidence. Keep the project root and restrict the measurement explicitly:

```sh
locmetry acme-orders --no-user-config --subfolder web/src --format table --show-folders
```

This retains project-relative rules and module context while counting only the selected folder. `--show-folders` adds rows for its children. A target excluded, ignored or hidden by policy fails clearly instead of reporting an empty measurement. [Subfolder semantics](../PRODUCT.md#subfolder-measurement) explains context traversal, coverage, diagnostics and comparison compatibility. Ancestor configuration discovery is not automatic.

### A counted file has an unexpected purpose

An earlier classification stage may win over your rule. Explain the file using the same project root and policy as the measurement; inspect `result.file.rule` and `result.decision` before editing configuration:

```sh
locmetry explain web/src/cart.test.ts --root acme-orders --no-user-config --format json
```

The sample's test convention wins before module inference. When measuring a subfolder, use the same project-relative target and omit `--subfolder` from `explain`: the file already restricts counting. Entries listed as unused in a subtree may still decide files elsewhere. Only an unrestricted measurement supports the configuration-cleanup checks for `unused_entries` and `unused_exclusions`.

### A file is absent from the counts

Absence can mean exclusion, ignore/hidden filtering, an unrecognized format, or unavailable counts. Explain the path instead of assuming zero lines:

```sh
locmetry explain docs/order-flow.png --root acme-orders --no-user-config --format json
```

The sample image is unrecognized. The [evidence table](#interpret-the-evidence) covers other states and what their evidence supports.
