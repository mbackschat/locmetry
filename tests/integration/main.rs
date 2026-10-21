//! Every integration test in one binary. Each test file used to be its own
//! binary that linked the whole Tokei tree again, which dominated rebuilds
//! (23 s for 22 binaries after a one-file change, measured 2026-10-19).
//! Filter by module, for example `cargo test --locked --test integration compare::`.

mod common;

mod cli;
mod cli_findings;
mod cli_modules;
mod cli_options;
mod cli_tables;
mod cli_user_defaults;
mod compare;
mod compare_member;
mod compare_render;
mod config;
mod contracts;
mod explain;
mod folder_rows;
mod folders;
mod guides;
mod help;
mod inline_tests;
mod language_families;
mod measurement;
mod notebooks;
mod output_contract;
mod private_regression;
mod purposes;
mod queries;
mod readme;
mod report_totals;
mod schema;
mod subfolder;
mod submodules;
mod tables_only;
mod terminal_safety;
mod verbosity;
mod workspace;

/// Locks the single binary: Cargo builds every top-level `tests/*.rs` file as
/// its own binary, which would bring the per-binary link cost back.
#[test]
fn integration_tests_are_modules_of_this_binary() {
    let stray: Vec<_> = std::fs::read_dir(concat!(env!("CARGO_MANIFEST_DIR"), "/tests"))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .collect();
    assert!(
        stray.is_empty(),
        "declare these as modules in tests/integration/main.rs instead: {stray:?}"
    );
}
