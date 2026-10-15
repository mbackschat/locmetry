# Repository Guidelines

## Project Structure & Module Organization

`locmetry` is a Rust CLI embedding Tokei. [src](src) contains configuration, backend integration, aggregation, output, and the CLI entry point. [tests](tests) contains synthetic integration tests and an opt-in private regression guard. [PRODUCT.md](PRODUCT.md) defines the product contract; [PLAN.md](PLAN.md) tracks remaining tasks, gaps, and acceptance criteria. Consult the plan before implementation and update it with verified progress. [PoC usage](poc/README.md) describes the implemented JSON/regex interface. [Research](docs/research/README.md) and [integration and packaging](docs/tokei-integration.md) explain design decisions. Edit this canonical file; preserve `AGENTS.md` as its relative symlink.

## Build, Test, and Development Commands

- `cargo build --release --locked`: build the standalone executable in `target/release/`.
- `cargo test --locked`: run public tests without reference checkouts.
- `cargo fmt --all --check`: verify Rust formatting.
- `cargo clippy --locked --all-targets -- -D warnings`: lint every target.
- `cargo run --locked -- --help`: inspect CLI usage.

Keep [Cargo.lock](Cargo.lock) with the application. Tokei is embedded with its default CLI feature disabled; do not add Python, Git, or Tokei executable requirements. Homebrew Rust dependencies are build-only. Use `git status --short` and `git diff --check` before submission.

## Coding Style & Naming Conventions

Use rustfmt, four-space indentation, `snake_case` functions/modules, and `PascalCase` types. Prefer enums and `match` for finite alternatives. Keep changes focused and avoid speculative adapter frameworks. Write Markdown with one paragraph per physical line and regular links to other documents.

## Testing Guidelines

Use red/green TDD. Name tests after observable behavior and guard precedence, scope boundaries, embedded-language deduplication, and folder reconciliation. No coverage threshold is set. Public tests must run without external executables. Private regression instructions are in [PoC usage](poc/README.md). Keep reference projects read-only; do not run their builds or tests merely to count files.

## Sensitive Analysis

Keep measured-project names, paths, profiles, counts, and reports exclusively in Git-ignored `docs/research/local/`. Use synthetic public fixtures. Never force-add local analysis files.

## Commit & Pull Request Guidelines

Commit autonomously, without asking first, whenever a coherent checkpoint is reached: a completed feature set or plan step whose tests, format, and lint checks pass. Group related edits so each commit is a reviewable unit; do not commit every small step. Pushing still requires an explicit request.

Use Conventional Commits, such as `feat(core): add purpose classification`, with a concise body when non-obvious rationale needs preservation. Pull requests should describe the problem, resulting behavior, and validation; link relevant issues and identify checks that could not run.
