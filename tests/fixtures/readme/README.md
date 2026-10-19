# Illustrative README report

[report.json](report.json) owns the invented counts for the two tables in the [project README](../../../README.md). Each row is one direct folder, language and purpose combination; parent totals, line totals and ratios are derived. The fixture contains source counts only, without embedded-language overlap. It is not a saved measurement or evidence about a real project.

[The README test](../../integration/readme.rs) feeds these counts into Locmetry's real table renderer. It distributes them into per-file report records in memory because that is the renderer's input; it never creates source files or a project directory. Increasing LOC changes numeric values, not generated content. The file count controls the number of in-memory records.

To resize or rebalance the example, edit the data rows, then refresh only the marked tables:

```sh
cargo test --locked --test integration readme::refresh_readme_tables -- --ignored
cargo test --locked --test integration readme::
```

The tests compare the README with renderer output and independently reconcile each displayed row against the fixture. They check file counts, line components, purposes, rounded test/production ratios and both grand totals. Folder rows must cover every contribution exactly once, including collapsed descendants and direct files, within the demonstrated 30-row view. Negative cases reject altered counts and duplicated folder rows. The surrounding classification examples describe rules; this fixture does not exercise counting or classification.
