# Repository Guidelines

## Project Structure & Module Organization

`locmetry` is a Rust CLI embedding Tokei. Edit this canonical file; preserve `AGENTS.md` as its relative symlink. Project-specific rules below refine the global defaults.

- Implementation: [src](src); synthetic integration tests and the opt-in private regression: [tests](tests).
- Current requirements: [PRODUCT](docs/PRODUCT.md); open work and links to deferred designs/history: [PLAN](docs/PLAN.md).
- CLI entry points: [README](README.md); counting and selection limits: [measurement semantics](docs/measurement.md).
- Design provenance: [research](docs/research/README.md); build/distribution decisions: [Tokei integration](docs/tokei-integration.md).
- CI and releases: [.github/workflows](.github/workflows); Homebrew template: [packaging/homebrew](packaging/homebrew).

Before implementation, scan PLAN for relevant open work, then read only the applicable contract, plan section and source. Search headings and filenames before opening large documents. Consult archived checkpoints only when their rationale or evidence is needed; they are not current work instructions.

## Build, Test, and Development Commands

- `cargo build --release --locked`: build the standalone executable in `target/release/`.
- `cargo test --locked`: run public tests without reference checkouts.
- `cargo fmt --all --check`: verify Rust formatting.
- `cargo clippy --locked --all-targets -- -D warnings`: lint every target.
- `cargo run --locked -- --help`: inspect CLI usage.
- `python3 scripts/check-file-sizes.py`: gate source file sizes with the built executable.

Keep [Cargo.lock](Cargo.lock) with the application. Tokei is embedded with its default CLI feature disabled; do not add Python, Git, or Tokei executable requirements. Homebrew Rust dependencies are build-only, and Linux release binaries are static musl builds. Use `git status --short` and `git diff --check` before submission.

## Coding Style & Naming Conventions

Use rustfmt, four-space indentation, `snake_case` functions/modules, and `PascalCase` types. Keep source files within the [file-size limits](docs/self-measurement.md#file-sizes) that CI enforces with `python3 scripts/check-file-sizes.py`: split a file instead of raising its limit, and lower or remove a ceiling in the same commit that shrinks its file. Prefer enums and `match` for finite alternatives. Keep changes focused and avoid speculative adapter frameworks. Write Markdown with one paragraph per physical line and regular links to other documents.

Update the owner of an affected public guarantee and any user-facing entry point made inaccurate by the change. Do not edit every documentation file for each checkpoint. Update PLAN or a detailed plan only when scope, status, a material decision, acceptance criteria or remaining work changes; keep routine small-fix verification in the commit handback. Keep current behavior in README and PRODUCT and decision/verification evidence in the existing archives. Add an entry under `Unreleased` in [CHANGELOG.md](CHANGELOG.md) in the same commit as each change users can notice, worded for users and marked **Breaking** when it breaks commands, reports or configuration; internal refactors, tests and tooling need none. The release command refuses an empty `Unreleased` section and turns it into the version's section, which becomes the GitHub release text. Change CLAUDE.md only when contributor instructions change. Keep proposed behavior distinct from supported guarantees. Breaking changes to commands, reports and configuration are acceptable before 1.0, decided 2026-10-21 by the owner; record each in the owning contract.

Keep PLAN focused on open work. When a plan is complete, transfer live follow-ups to PLAN or a focused deferred-design document, archive the completed record under `docs/plans/archive/`, and update tracked inbound/outbound links and fragments. Preserve historical text and provenance apart from link relocation and an archival status notice; do not maintain a second live copy. A history link in PLAN is sufficient; do not repeat completion narratives there.

Human presentation roles and palettes live in [src/style.rs](src/style.rs). Renderers select semantic roles, emphasis through `Theme::strong`, and pad terminal cells before styling; a palette alone maps roles to colors, so a new look (for example, light backgrounds) is a new palette, not renderer changes; keep escape sequences out of individual renderers and JSON. Configuration for custom themes remains outside the supported interface.

Follow the [proposal retirement procedure](docs/proposals/README.md#retirement-procedure) when an accepted design is verified, rejected or superseded. Move retired records to the archive, transfer live follow-ups to the plan, and preserve their evidence and links.

### Documentation ownership and checks

For help changes, follow the [CLI help design guidelines](docs/help-design.md): scope examples, make reference routing conditional, include verification and evaluate reading cost as well as correctness. Word notes, legends, warnings and errors by the [human output wording rules](docs/help-design.md#human-output-wording); `cargo test --locked --test integration wording::` enforces what it can detect. Before a release whose help, guides, flags, defaults or report fields changed since the last checked version, run an [agent regression check](docs/agent-regression-checks.md) at the level its findings justify and record it in the ledger. Keep experimental evidence distinct from general design judgments.

Follow the [documentation maintenance rules](docs/documentation-maintenance.md) for folder READMEs and authoritative status/date metadata. Update managed tables with `python3 scripts/doc-indexes.py update`; run `python3 scripts/doc-indexes.py check` after documentation changes. Keep status transitions, evidenced dates and regenerated indexes in the same commit. The checker and its regression suite run in CI; archived bodies remain frozen.

Keep changed guarantees and affected user-facing entry points consistent in the same commit. Link to the owner instead of copying detailed rules into another guide.

| Document | Owns | Check when behavior changes |
| --- | --- | --- |
| [README](README.md) | Task entry points, CLI examples and navigation | Check affected examples when their commands, values or behavior change; use `bash scripts/check-doc-examples.sh` for executable recipes and `cargo test --locked --test integration readme::` for the Common commands table, which mirrors the `--help` examples in [src/app/cli.rs](src/app/cli.rs). Regenerate the Immich excerpts with `python3 scripts/refresh-readme-showcase.py` after a pin move or a measurement or presentation change that affects them. |
| [PRODUCT](docs/PRODUCT.md) | Supported public contract | Reconcile flags, report fields and compatibility with executable schemas and tests. |
| [Measurement semantics](docs/measurement.md) | Counting, selection boundaries and limitations | Add independent regression oracles for changed measurement behavior. |
| [Configure](docs/guides/configure.md), [selection](docs/guides/selection.md), [reports](docs/guides/reports.md), [compare](docs/guides/compare.md) | Task workflows, examples and evidence interpretation | `cargo test --locked --test integration guides::` checks embedded sources and examples. Add the `explain::` or `queries::` filter when those behaviors change, and the documentation script when executable recipes are affected. |
| Executable schemas | Machine-readable structure and semantics | `cargo test --locked --test integration schema::` checks emitted reports and documented TOML examples. |
| [PLAN](docs/PLAN.md) and its linked live plans | Open scope, decisions and acceptance criteria | Update material changes only; record verified outcomes and retain deferred activation conditions. |
| [Research](docs/research/README.md) | Dated findings and provenance | Preserve the historical evidence; link its resolution from the active plan. |

Edit the regular Markdown files in `docs/guides/` directly. [src/guides.rs](src/guides.rs) embeds them verbatim with `include_str!`; rebuilding updates `--guide` output, with no generated documentation copy to maintain.

Before creating or editing recorded demonstrations, follow the [Showboat authoring guidelines](docs/showboat-authoring.md). Recorded command guides live in [docs/cli](docs/cli/README.md) and use the shared [synthetic workspace](examples/cli-workspace/README.md). Showcase recordings live in [docs/showcase](docs/showcase/README.md) and measure pinned Immich checkouts; fetch them once with `bash scripts/fetch-showcase.sh`, and change their policy only through [examples/showcase/immich](examples/showcase/immich/README.md). After CLI, fixture or recorded-guide changes, run the [pinned Showboat verification command](docs/cli/README.md#verify-and-regenerate); it also runs in Linux documentation CI. Preserve real captured output: edit commands/prose and regenerate with Showboat, never hand-edit output. Keep installed packages, environments and Python dependency/lock files out of Git.

CI checks executable examples and schema agreement; it cannot establish that every prose claim is current. Review affected prose. The example script is a development check, not a dependency of the executable or public Cargo tests.

## Testing Guidelines

Use red/green TDD. Name tests after observable behavior and guard precedence, scope boundaries, embedded-language deduplication, and folder reconciliation. No coverage threshold is set. Integration tests are modules of the single `tests/integration` binary, filtered as `cargo test --locked --test integration compare::`; a guard rejects new top-level `tests/*.rs` files because each would link its own binary again. Public tests must run without external executables. Private regression instructions are in [measurement semantics](docs/measurement.md#private-regression). Keep reference projects read-only; do not run their builds or tests merely to count files.

Choose verification from the changed behavior before implementation; combine rows when a change crosses boundaries. Full-suite runs are not required for every local checkpoint.

| Change | Required local verification |
| --- | --- |
| Isolated renderer/presentation change | Focused renderer/CLI regressions (for example, `cargo test --locked --test integration compare::`), formatting and Clippy; preserve styling/plain-text parity when affected. |
| Counting, selection, classification, schemas, dependencies or cross-cutting behavior | Full `cargo test --locked`, formatting and Clippy; add the private regression for measurement changes and affected executable documentation checks. |
| Embedded guide source or guide discovery | Guides tests plus affected query/explanation tests; formatting and Clippy for Rust changes. Run documentation recipes when their commands, expected results or behavior change. |
| Documentation-only instructions, plans, links or prose outside embedded guides | Check changed links/fragments, preserved history when moving records, and `git diff --check`; do not run unrelated Rust tests or builds. |
| Release preparation | Full public suite, formatting, Clippy, executable documentation examples and release build; follow the distribution checks for the release scope, and an [agent regression check](docs/agent-regression-checks.md) when the CLI surface changed since the last checked version. |

Run Cargo commands sharing a target directory sequentially to avoid build-lock contention. Reuse passing checks for unchanged relevant inputs; repeat or broaden them only after relevant changes, failures or unresolved concerns. CI retains the full platform matrix. In the handback, state the checks actually run and any material unverified boundary, without a routine list of unrelated skipped checks.

## Sensitive Analysis

Keep measured-project names, paths, profiles, counts, and reports exclusively in Git-ignored `docs/research/local/`. Use synthetic public fixtures. Never force-add local analysis files. Exception, decided 2026-10-15 by the owner: a12-rulekit and bpmn-lean-experiment may be named in public documents; their paths, profiles, and counts stay local. Exception, decided 2026-10-21 by the owner: the public open-source project Immich, measured at pinned release tags, is the public showcase; its name, Locmetry policies, counts and reports may appear in public documents. Local checkout paths stay out of them.

## Commit & Pull Request Guidelines

Commit autonomously, without asking first, whenever a coherent checkpoint passes the checks applicable under Testing Guidelines. Group related edits into reviewable units; do not commit every small step. For `F:` requests, leave smaller fixes uncommitted for grouping and propose a commit; commit only when the change warrants an independently reviewable checkpoint of its own. An explicit instruction not to commit always wins. Pushing still requires an explicit request.

Use Conventional Commits, such as `feat(core): add purpose classification`, with a concise body when non-obvious rationale needs preservation. Pull requests should describe the problem, resulting behavior, and validation; link relevant issues and identify checks that could not run.
