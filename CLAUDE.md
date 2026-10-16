# Repository Guidelines

## Project Structure & Module Organization

`locmetry` is a Rust CLI embedding Tokei. [src](src) contains configuration, backend integration, build-module detection, aggregation, output, and the CLI entry point. [tests](tests) contains synthetic integration tests and an opt-in private regression guard. [PRODUCT.md](docs/PRODUCT.md) defines the product contract; [PLAN.md](docs/PLAN.md) tracks remaining tasks, gaps, and acceptance criteria. Consult the plan before implementation and update it with verified progress. [README](README.md) describes CLI usage; [measurement semantics](docs/measurement.md) documents selection details, limits, and the private regression. [Research](docs/research/README.md) and [integration and distribution](docs/tokei-integration.md) explain design decisions. [.github/workflows](.github/workflows) holds CI and the release build; [packaging/homebrew](packaging/homebrew) holds the formula template. Edit this canonical file; preserve `AGENTS.md` as its relative symlink.

## Build, Test, and Development Commands

- `cargo build --release --locked`: build the standalone executable in `target/release/`.
- `cargo test --locked`: run public tests without reference checkouts.
- `cargo fmt --all --check`: verify Rust formatting.
- `cargo clippy --locked --all-targets -- -D warnings`: lint every target.
- `cargo run --locked -- --help`: inspect CLI usage.

Keep [Cargo.lock](Cargo.lock) with the application. Tokei is embedded with its default CLI feature disabled; do not add Python, Git, or Tokei executable requirements. Homebrew Rust dependencies are build-only, and Linux release binaries are static musl builds. Use `git status --short` and `git diff --check` before submission.

## Coding Style & Naming Conventions

Use rustfmt, four-space indentation, `snake_case` functions/modules, and `PascalCase` types. Prefer enums and `match` for finite alternatives. Keep changes focused and avoid speculative adapter frameworks. Write Markdown with one paragraph per physical line and regular links to other documents.

Keep documentation current in every implementation checkpoint: update README workflows, the supported PRODUCT contract, measurement semantics, executable schema descriptions and relevant guides when behavior changes. Add a focused document when the material no longer fits, and link it from the appropriate index. Update [PLAN.md](docs/PLAN.md) and its active work plan with verified progress, remaining gaps and material rationale; preserve source provenance. Keep proposed behavior distinct from shipped guarantees. The [measurement quality and net change plan](docs/plans/measurement-quality-and-net-change.md) owns the current accepted sequence and stage review criteria.

Human presentation roles and palettes live in [src/style.rs](src/style.rs). Renderers select semantic roles and pad terminal cells before styling; keep escape sequences out of individual renderers and JSON. Configuration for custom themes remains outside the supported interface.

### Documentation ownership and checks

Update the owner of a changed guarantee and its user-facing entry points in the same commit. Link to the owner instead of copying detailed rules into another guide.

| Document | Owns | Check when behavior changes |
| --- | --- | --- |
| [README](README.md) | Tasks, CLI examples and navigation | Run affected examples; `bash scripts/check-doc-examples.sh` checks distinct policy failures, counted explanations, a real source delta, workspace Markdown values the sample-project troubleshooting queries and adaptive folder detail. |
| [PRODUCT](docs/PRODUCT.md) | Supported public contract | Reconcile flags, report fields and compatibility with executable schemas and tests. |
| [Measurement semantics](docs/measurement.md) | Counting, selection boundaries and limitations | Add independent regression oracles for changed measurement behavior. |
| [Troubleshooting](docs/troubleshooting.md) | Focused path explanations and evidence limits | `cargo test --locked --test explain` checks documented result, shared measurement parity and schema examples. |
| Executable schemas | Machine-readable structure and semantics | `cargo test --locked --test schema` checks emitted reports and documented TOML examples. |
| [Active work plan](docs/plans/measurement-quality-and-net-change.md) | Stage status, decisions and verification | Record only verified completion; distinguish planned from supported behavior. |
| [Research](docs/research/README.md) | Dated findings and provenance | Preserve the historical evidence; link its resolution from the active plan. |

CI checks executable examples and schema agreement; it cannot establish that every prose claim is current. Review affected prose at each checkpoint. The example script is a development check, not a dependency of the executable or public Cargo tests.

## Testing Guidelines

Use red/green TDD. Name tests after observable behavior and guard precedence, scope boundaries, embedded-language deduplication, and folder reconciliation. No coverage threshold is set. Public tests must run without external executables. Private regression instructions are in [measurement semantics](docs/measurement.md#private-regression). Keep reference projects read-only; do not run their builds or tests merely to count files.

## Sensitive Analysis

Keep measured-project names, paths, profiles, counts, and reports exclusively in Git-ignored `docs/research/local/`. Use synthetic public fixtures. Never force-add local analysis files. Exception, decided 2026-10-15 by the owner: a12-rulekit and bpmn-lean-experiment may be named in public documents; their paths, profiles, and counts stay local.

## Commit & Pull Request Guidelines

Commit autonomously, without asking first, whenever a coherent checkpoint is reached: a completed feature set or plan step whose tests, format, and lint checks pass. Group related edits so each commit is a reviewable unit; do not commit every small step. Pushing still requires an explicit request.

Use Conventional Commits, such as `feat(core): add purpose classification`, with a concise body when non-obvious rationale needs preservation. Pull requests should describe the problem, resulting behavior, and validation; link relevant issues and identify checks that could not run.
