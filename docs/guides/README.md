# Task Guides

Find the section that answers your task and read only that. This page is embedded verbatim as `locmetry --guide index`; `--section` takes a guide heading's GitHub anchor, or a schema's `overview`, top-level property or definition; repeat it to read several sections of one topic in one command. Ids ignore case, and an unknown one lists the valid ones.

| When you want to | Read |
| --- | --- |
| Measure first and review the evidence before configuring | `locmetry --guide configure --section start-with-evidence` |
| Decide a file's purpose, or how entries, globs and `except` match | `locmetry --guide configure --section decide-purpose-not-just-coverage`, `locmetry --schema config --section overview` |
| Write a rule, an override or a scope exclusion | `locmetry --schema config --section Entry --section Scope` |
| Give tests, benchmarks or scripts finer categories | `locmetry --guide configure --section refine-categories`, `locmetry --guide categories --section choose-a-name`, `locmetry --schema config --section Refinement` |
| Name a category, or one no reference name fits | `locmetry --guide categories --section choose-a-name --section project-specific-names` |
| Check a name against worked cases | `locmetry --guide categories --section worked-cases` |
| Know what the category reference decides | `locmetry --guide categories --section what-the-reference-decides` |
| Tell test support from verification tooling | `locmetry --guide configure --section test-support-or-verification-tooling`, `locmetry --schema config --section Purpose` |
| Write a reviewed policy | `locmetry --guide categories --section write-a-reviewed-policy`, `locmetry --guide configure --section validate-and-hand-back` |
| Count an unrecognized file | `locmetry --guide configure --section recognize-a-language` |
| Correct a misrecognized language | `locmetry --guide configure --section correct-a-misrecognized-language` |
| Follow a complete configuration example | `locmetry --guide configure --section worked-configuration` |
| Set personal defaults | `locmetry --schema user-config --section overview` |
| Diagnose one missing or surprising file | `locmetry --guide selection --section start-with-one-file --section explain-one-file --section interpret-the-evidence` |
| Choose which policy applies | `locmetry --guide selection --section choose-the-correct-policy` |
| Fix a subfolder scan, an unexpected purpose or an absent file | `locmetry --guide selection --section common-issues-and-useful-queries` |
| Choose report content and destination | `locmetry --guide reports --section choose-content-and-destination` |
| Interpret counts, Test/prod and unavailable counts | `locmetry --guide reports --section interpret-the-numbers --section unavailable-is-not-zero` |
| List unclaimed, unavailable or non-source files | `locmetry --guide reports --section common-issues-and-useful-queries` |
| Look inside large folders | `locmetry --guide reports --section look-inside-large-folders` |
| Measure several repositories together | `locmetry --guide reports --section measure-several-repositories-together`, `locmetry --schema workspace --section overview` |
| Query a saved report | `locmetry --guide reports --section query-a-saved-report`, `locmetry --schema report --section overview` |
| Gate measurements in continuous integration | `locmetry --guide reports --section continuous-integration` |
| Analyze saved measurements | `locmetry --guide reports --section analyze-saved-measurements` |
| Compare two measurements | `locmetry --guide compare --section a-small-example --section comparable-scope-and-evidence --section compare-saved-measurements` |
| Compare one workspace member | `locmetry --guide compare --section compare-saved-measurements --section inspect-one-workspace-member` |
| Review what a configuration change moved | `locmetry --guide compare --section inspect-configuration-transitions` |

Whole guides, for reading a workflow end to end:

- [Configure a project](configure.md): build an evidence-based measurement policy.
- [Explain selection](selection.md): diagnose missing files, classification and parser limits.
- [Read reports](reports.md): interpret counts, coverage and workspace results.
- [Compare measurements](compare.md): capture baselines and interpret net changes.
- [Choose category names](categories.md): reference category names, worked cases and conventions for reviewed policies.

See the [documentation index](../README.md) for contracts and contributor procedures.
