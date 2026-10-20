# Explain a File

*2026-10-20T12:56:38Z by Showboat 0.6.1*
<!-- showboat-id: 996f097b-9163-4c99-a692-9ccd0522780e -->

Run each Bash block from the Locmetry repository root. [Setup and replay instructions](README.md) describe the fresh temporary copy created by `source scripts/cli-demo-env.sh`, external tools and output projections. The [sample tour](../sample-project-tour.md) introduces the shared source tree.

## Trace the deciding rule

`PATH` is an existing root-relative file with exact spelling. `--root` supplies the scan root. The table keeps classification evidence; only its absolute root line is removed. `--no-color` disables styling.

```bash
source scripts/cli-demo-env.sh
locmetry explain service/scripts/check.py --root acme-orders --no-user-config --no-color \
  --format table \
  | sed '/^Root:/d'
```

```output
Path explanation
Path: service/scripts/check.py
Project config: acme-orders/.locmetryrc
Personal config: none
State: counted
Language: Python
Kind: source
Purpose: production
Category: tooling
Rule: tooling
Stage: rule
Counts: 2 code, 1 comments, 0 blanks, 3 lines
Matching patterns: service/scripts/**
```

## Inspect selection states

Full JSON has a tagged `result`. These files demonstrate test classification, project exclusion, hidden policy, ignore rules, unclaimed source and unrecognized content. Counted/unclaimed is different from unavailable counts. Neither excluded nor unrecognized means a measured zero.

```bash
source scripts/cli-demo-env.sh
for path in web/src/orders/Order.test.ts service/third_party/sdk.py service/.hidden.py \
  service/ignored.py service/experiments/probe.py web/docs/wireframe.asset; do
  locmetry explain "$path" --root acme-orders --no-user-config --format json \
    | jq '{path, result}'
done
```

```output
{
  "path": "web/src/orders/Order.test.ts",
  "result": {
    "state": "counted",
    "file": {
      "path": "web/src/orders/Order.test.ts",
      "language": "TypeScript",
      "kind": "source",
      "purpose": "test",
      "category": "test",
      "rule": "builtin:js-test-file",
      "counts": {
        "code": 3,
        "comments": 1,
        "blanks": 0,
        "lines": 4
      }
    },
    "decision": {
      "stage": "convention",
      "matched_patterns": [
        "**/*.{test,spec}.{js,jsx,mjs,cjs,ts,tsx,mts,cts}"
      ]
    }
  }
}
{
  "path": "service/third_party/sdk.py",
  "result": {
    "state": "excluded",
    "at": "service/third_party",
    "patterns": [
      "service/third_party/**"
    ]
  }
}
{
  "path": "service/.hidden.py",
  "result": {
    "state": "not_selected",
    "reason": "hidden_policy"
  }
}
{
  "path": "service/ignored.py",
  "result": {
    "state": "not_selected",
    "reason": "ignore_policy"
  }
}
{
  "path": "service/experiments/probe.py",
  "result": {
    "state": "counted",
    "file": {
      "path": "service/experiments/probe.py",
      "language": "Python",
      "kind": "source",
      "purpose": "other",
      "category": "unclaimed",
      "rule": "fallback",
      "counts": {
        "code": 1,
        "comments": 1,
        "blanks": 0,
        "lines": 2
      }
    },
    "decision": {
      "stage": "fallback",
      "matched_patterns": []
    }
  }
}
{
  "path": "web/docs/wireframe.asset",
  "result": {
    "state": "unrecognized"
  }
}
```

## Try selection overrides

`--hidden true` and `--respect-ignore-files false` override effective scope. `--exclude` adds a root-relative exclusion. The projections expose the resulting state and decision.

```bash
source scripts/cli-demo-env.sh
locmetry explain service/.hidden.py --root acme-orders --hidden true --no-user-config \
  --format json \
  | jq '{path, result}'
locmetry explain service/ignored.py --root acme-orders --respect-ignore-files false \
  --no-user-config --format json \
  | jq '{path, result}'
locmetry explain web/src/orders/Order.ts --root acme-orders --exclude 'web/src/**' \
  --no-user-config --format json \
  | jq '{path, result}'
```

```output
{
  "path": "service/.hidden.py",
  "result": {
    "state": "counted",
    "file": {
      "path": "service/.hidden.py",
      "language": "Python",
      "kind": "source",
      "purpose": "other",
      "category": "unclaimed",
      "rule": "fallback",
      "counts": {
        "code": 1,
        "comments": 0,
        "blanks": 0,
        "lines": 1
      }
    },
    "decision": {
      "stage": "fallback",
      "matched_patterns": []
    }
  }
}
{
  "path": "service/ignored.py",
  "result": {
    "state": "counted",
    "file": {
      "path": "service/ignored.py",
      "language": "Python",
      "kind": "source",
      "purpose": "other",
      "category": "unclaimed",
      "rule": "fallback",
      "counts": {
        "code": 1,
        "comments": 0,
        "blanks": 0,
        "lines": 1
      }
    },
    "decision": {
      "stage": "fallback",
      "matched_patterns": []
    }
  }
}
{
  "path": "web/src/orders/Order.ts",
  "result": {
    "state": "excluded",
    "at": "web/src",
    "patterns": [
      "web/src/**"
    ]
  }
}
```

## Compare a proposed policy

`--config` replaces discovered project policy. An explicit override claims the experiment as intentional Other/prototype. This changes classification, not source content. The experimental configuration exists only in this temporary copy.

```bash
source scripts/cli-demo-env.sh
cat > proposed.toml <<'TOML'
version = 1
[[overrides]]
id = "experiment"
match = ["service/experiments/**"]
purpose = "other"
category = "prototype"
TOML
locmetry explain service/experiments/probe.py --root acme-orders --config proposed.toml \
  --no-user-config --format json \
  | jq '{path, result}'
```

```output
{
  "path": "service/experiments/probe.py",
  "result": {
    "state": "counted",
    "file": {
      "path": "service/experiments/probe.py",
      "language": "Python",
      "kind": "source",
      "purpose": "other",
      "category": "prototype",
      "rule": "experiment",
      "counts": {
        "code": 1,
        "comments": 1,
        "blanks": 0,
        "lines": 2
      }
    },
    "decision": {
      "stage": "override",
      "matched_patterns": [
        "service/experiments/**"
      ]
    }
  }
}
```

## Use a workspace and save Markdown

`--workspace` takes member-prefixed paths and applies that member's policy. Single-root policy flags cannot accompany it. Explain supports table, JSON and Markdown; redirect stdout to save it because there is no `--output` or headline option. The Markdown excerpt omits its absolute root row.

```bash
source scripts/cli-demo-env.sh
locmetry explain web/src/orders/Order.test.ts --workspace workspace.toml --no-user-config \
  --format json \
  | jq '{member, path, result}'
locmetry explain service/scripts/check.py --root acme-orders --no-user-config --format markdown \
  > explanation.md
sed '/^| .*Root.* |/d' explanation.md
```

```output
{
  "member": "web",
  "path": "web/src/orders/Order.test.ts",
  "result": {
    "state": "counted",
    "file": {
      "path": "web/src/orders/Order.test.ts",
      "language": "TypeScript",
      "kind": "source",
      "purpose": "test",
      "category": "test",
      "rule": "builtin:js-test-file",
      "counts": {
        "code": 3,
        "comments": 1,
        "blanks": 0,
        "lines": 4
      }
    },
    "decision": {
      "stage": "convention",
      "matched_patterns": [
        "**/*.{test,spec}.{js,jsx,mjs,cjs,ts,tsx,mts,cts}"
      ]
    }
  }
}
# Path explanation

| Evidence | Value |
| --- | --- |
| **Path** | service/scripts/check.py |
| **Project config** | acme-orders/.locmetryrc |
| **Personal config** | none |
| **State** | counted |
| **Language** | Python |
| **Kind** | source |
| **Purpose** | production |
| **Category** | tooling |
| **Rule** | tooling |
| **Stage** | rule |
| **Counts** | 2 code, 1 comments, 0 blanks, 3 lines |
| **Matching patterns** | service/scripts/** |
```

## Discover explanation syntax

These successful states exit 0. Unread or parse-failed files exit 2, and invalid paths or configuration exit 1. The synthetic fixture avoids OS-dependent permission failures; dedicated integration tests cover failures.

```bash
source scripts/cli-demo-env.sh
locmetry explain --no-color --help | sed -n '1,20p'
locmetry --schema explanation | jq '{title, required}'
```

```output
Explain selection and classification of one existing root-relative file. JSON prints the complete
explanation; there is no separate headline

Usage: locmetry explain [OPTIONS] <PATH>

Arguments:
  <PATH>
          Root-relative file, or MEMBER/PATH with --workspace

Options:
      --no-color
          Disable terminal colors, bold and dim text, including help and diagnostics

      --root <ROOT>
          Scan root [default: the current directory]

      --config <CONFIG>
          TOML configuration [default: .locmetryrc in the scan root, else built-in settings]

      --exclude <GLOB>
{
  "title": "Explanation",
  "required": [
    "document_type",
    "schema_version",
    "root",
    "path",
    "scope",
    "result"
  ]
}
```
