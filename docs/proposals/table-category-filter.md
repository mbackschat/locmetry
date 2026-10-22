# Table Category Filter

<!-- doc-status
{
  "status": "accepted",
  "date": "2026-10-22",
  "summary": "A --table-category option that narrows tables to purpose:category keys, with exact names and quoted wildcards.",
  "evidence": "[PLAN](../PLAN.md#table-category-filter)"
}
-->

Status: accepted by the owner on 2026-10-22; implementation follows test-first.

## Problem

`--table-purpose` and `--table-language` narrow tables by purpose or language, but nothing narrows them by category. Questions such as "how much tooling is there, and in which languages?" or "show tests beside tooling" need the full table and mental arithmetic. Giving `--show-categories` a value was rejected: the positional root makes `--show-categories production` ambiguous, and it would mix a show flag with a filter.

## Contract

```text
--table-category <CATEGORIES>
    Show only these categories' source in tables; comma-separated.
    Table format only.
```

| Form | Selects | Example |
| --- | --- | --- |
| `PURPOSE` | every category of that purpose | `test` |
| `PURPOSE:CATEGORY` | one category, as the category table prints its key | `production:verification-tooling` |
| `*` in either part | any run of characters, including none | `'production:*tooling*'`, `'*:generated'` |

- Category names are case-sensitive. A bare purpose, or a purpose part without `*`, must be `production`, `test` or `other`; anything else is an error naming the value. A category that matches nothing is not an error, as an unmatched `--table-language` is not.
- The filter applies to every table of a root, subfolder or workspace measurement: languages, folders, members and categories. JSON and Markdown keep the complete measurement; other formats warn that the option has no effect.
- With `--table-purpose` and `--table-language`, a row is shown only when every filter matches it.
- A `Not shown (categories): …` note names the categories left out and their code, largest first, as the purpose and language notes do.
- Test/prod is replaced by Comment %, as in a single-purpose view. Categories divide one purpose, so a category selection would compare an arbitrary slice of test code with an arbitrary slice of production code; a language selection divides both purposes alike and keeps the ratio.
- The category table is not implied: filters filter and show flags show.

```sh
locmetry --tables-only --show-categories --table-category 'production:*tooling*'
locmetry --tables-only --show-categories --table-category test,production:verification-tooling
```

Bare purposes and exact keys need no shell quoting. A wildcard does: zsh and fish refuse an unmatched `*`, and bash substitutes a matching file name.

## Scope

- Required: the option and its matching, the note, the ratio replacement, regressions for each form, the combination with other filters and workspaces, PRODUCT, the help-design inventory, help text and CHANGELOG.
- Optional: a help example, only within the help-size budget.
- Excluded: a value on `--show-categories`; comparison tables, which `--table-purpose` does not filter either; filtering JSON or Markdown; `?`, `**` and regular expressions.
