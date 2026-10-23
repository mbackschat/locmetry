# axum Experiment (2026-10-23)

Main session, development build of Locmetry 0.9.0, `--no-user-config`, on a shallow clone of [tokio-rs/axum](https://github.com/tokio-rs/axum) at `86aa6153e47b`, removed afterwards. Purpose: test the [module credit proposal](../../proposals/archive/module-credit-by-coverage.md) in a Rust workspace. [axum.policy.toml](policies/axum.policy.toml) marks the example crates as `production:example` and the benchmarks as tests.

## Result

The Cargo detector finds 61 crates; most are examples under `examples/`, each with its own `Cargo.toml` and `src/`.

| | Cargo modules reported | Credited code lines |
| --- | ---: | ---: |
| Built-ins only | 61 | 27,364 |
| Policy, current credit | 61 | 22,569 |
| Policy, credit by layout coverage | 61 | 27,364 |

The `examples` rule only names a category and keeps `production`, yet it removes the example crates' credit today, while the module count still includes them. Credit by layout coverage keeps it without any change to the policy. The benchmark rule changes the purpose, so its file correctly leaves the credit in both models.
