# Explain a File's Measurement

Use `locmetry explain PATH` when one file has an unexpected purpose, parser, or absence from a measurement. It applies the same configuration discovery, selection, counting and classification as an ordinary run. It traverses selected names for module inference but counts only the requested file. The [product contract](PRODUCT.md#path-explanations) defines the machine states; [measurement semantics](measurement.md) records counting limits.

## Start with One File

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
    "purpose": "unclassified",
    "category": "unclassified",
    "rule": "fallback",
    "counts": { "code": 1, "comments": 0, "blanks": 0, "lines": 1 }
  },
  "decision": {
    "stage": "fallback",
    "matched_patterns": []
  }
}
```

Use `--format table` for a compact labeled explanation or `--format markdown` for a shareable evidence table. A terminal defaults to table; redirected stdout defaults to JSON. Every format describes the same target. JSON has no separate headline or output-file mode: redirect stdout to save it. `locmetry --schema explanation` exposes the generated structure, state meanings and a complete example for agents.

## Choose the Correct Policy

| Task | Command |
| --- | --- |
| Explain a file from another working directory | `locmetry explain src/main.rs --root /path/to/repository` |
| Try a replacement project configuration | `locmetry explain src/main.rs --root /path/to/repository --config trial.toml` |
| Remove personal defaults from the experiment | `locmetry explain src/main.rs --no-user-config` |
| Check a workspace member | `locmetry explain api/src/main.rs --workspace repos.toml` |
| Check whether an additional exclusion applies | `locmetry explain vendor/lib.py --exclude 'vendor/**'` |

The target is relative to `--root`, which defaults to the working directory. An explicit `--config` is relative to the working directory and replaces discovery of the root's `.locmetryrc`. Personal defaults still apply unless disabled. All ordinary single-root scope flags are accepted. Use the same settings as the measurement you are investigating.

With `--workspace`, the target starts with the member's folder name; roots in the workspace file are relative to that file. Only the queried member's policy is loaded, while normal membership checks still reject duplicate names or nested roots. `--root`, `--config`, `--exclude`, `--hidden` and `--respect-ignore-files` cannot be combined with `--workspace`. Paths in the result are member-prefixed, but matching globs remain relative to that member's own root.

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
| `parse_failed` or `unread` | Counts are unavailable. Inspect the stable failure reason and human detail, or file access. Notebook support and limitations are in [measurement semantics](measurement.md#notebooks). |
| `symlink` | The path or an ancestor is a symlink, which measurement does not follow. `at` identifies the link; the command does not inspect its target or claim which other filters apply. |

Exit 0 means the explanation succeeded, including excluded, filtered and unrecognized outcomes. Exit 2 means the target was selected but its counts are unavailable. Exit 1 means the command could not establish an explanation, such as a missing path, directory query, invalid policy, inaccessible traversal or path escaping the root. A symlink ancestor ends inspection immediately, even if the remainder of the path would not exist at its destination.

This command is a current-filesystem diagnostic, not a historical explanation of a saved report. Ignore context and file contents can change between runs, including during the selection probes. An unrelated malformed notebook does not make this target partial, but traversal errors can still prevent module inference. The command does not inventory every ignored file or expose a new configuration self-test language.
