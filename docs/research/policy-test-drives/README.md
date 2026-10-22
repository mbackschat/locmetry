# Policy Test Drives

On 2026-10-22 and 2026-10-23, `.locmetryrc` policies were written for real projects to find out what Locmetry's configuration cannot express. Agents (Claude Code `sonnet` at `xhigh`) wrote most policies; the main session checked them and ran the experiments. This folder holds the records of the public GitHub projects; results of private projects stay in the Git-ignored local research folder, and the findings below mention them only without names or counts.

## Records

- [a12-kernel and its siblings](a12-kernel.md): a new policy for a Java kernel and a workspace with three sibling repositories, including an ANTLR fork with 172 C++ headers counted as C Header.
- [CIB seven](cibseven.md): a new policy for a Java workflow engine; unclaimed source from 68,912 to 5 lines.
- [Temporal](temporal.md): policies for the documentation, samples and TypeScript SDK repositories, and a workspace; a 121,960-line Git submodule excluded.
- [Loki](loki.md): experiment on a Go monorepo testing both configuration proposals outside the JVM and TypeScript sample.
- [axum](axum.md): experiment on a Rust workspace testing the module credit proposal.
- [Policies](policies): the created policy and workspace files, copied on 2026-10-23.

Immich, the public showcase, is documented in its [measurement recording](../../showcase/measure.md).

## Findings

**A recognized language cannot be corrected.** The counter assigns each extension to one language: ProGuard rules as Prolog (Immich), C++ headers as C Header and CMake templates as Autoconf (a12-kernel-antlr4), Go templates as Pan and goyacc grammars as Happy (Loki), and in a private project Go templates as Pan and INI files as Forge Config. Separately, Bash scripts appear as BASH or Shell depending on the file extension, in four projects. The [language corrections proposal](../../proposals/language-corrections.md) addresses the first; a table fold in [PLAN](../../PLAN.md) addresses the second.

**An entry that agrees with a build module takes its credit.** Every project lost module credit to entries that kept `production` and only named a category: Immich 4,102 lines, CIB seven 43,207, the Temporal SDK 7,866, Loki 134,430, axum 4,795, and two private projects. The module count also includes modules left without credit. Category refinements and category-only entries were drafted first and rejected after the Loki experiment; the [module credit proposal](../../proposals/module-credit-by-coverage.md) records why.

**Popular frameworks need configuration before their first useful measurement.** This came from a separate measurement of framework starters, recorded in [framework layouts](../framework-layouts.md) and addressed by the [framework layouts proposal](../../proposals/framework-layouts.md).

**Smaller candidates**, each seen in one to four projects, are tracked in PLAN once decided: a built-in convention for Go's `vendor/` (Loki: 3.16 million lines counted as production), Gradle `buildSrc` and Maven failsafe test conventions, reasons on scope exclusions, an `except` list for entries whose carved-out files must fall through to a later stage, and an unloaded `.locmetryrc` counted as project content.
