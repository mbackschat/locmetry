# Saved Report Compatibility

[PRODUCT](PRODUCT.md#reporting-and-coverage) owns the current report contract. `locmetry --schema report` describes structural validity; comparison eligibility additionally requires recorded detail. [Contract types](../src/contract.rs) own serialization, deserialization and schema descriptions, using the same schema generator as configuration. [Validation](../src/contract/validation.rs) checks arithmetic and recorded identities internally; no public `validate` command is provided.

## Version 3

Locmetry 0.6.0 introduces measurement schema 3. Current measurements require `document_type: "measurement"`, `schema_version: "3"` and `detail: "full"` or `"headline"`. This identifies the document and its projection before loading nested data into SQL or dataframes. Full reports require their aggregate detail and policy evidence; headline reports must omit it. Optional `files` still means file detail was requested, not whether the measurement was empty. The schema and internal semantic validator enforce that projection agreement. The obsolete measurement `report` pointer is rejected.

Earlier measurement schemas are not read: a report with another `schema_version` is rejected as an invalid measurement report naming its version, and must be recaptured with the current executable. No migration can recover an earlier source state. Report formats may change before 1.0; retain producer identity and inspect the executable's schema. Schema 3 grows by optional fields: a newer producer's report adds them, a current reader treats their absence in older reports as not recorded, and an older executable rejects the unknown field, so read reports with an executable at least as new as their producer. `summary.largest_module_files` is the first such field after 0.7.3; the [release fixtures](../tests/fixtures/reports/README.md) lock that 0.7.3 reports stay valid and comparable. Comparison and explanation documents use the same version 3; until this change they carried schema version 2 with the same content. The [analytics research](research/report-analytics.md) records why the envelope is tool-neutral; the [DuckDB](duckdb-cookbook.md) and [dataframe](dataframe-cookbook.md) recipes consume it.

## Earlier schemas

Schema 1 (Locmetry 0.1–0.4) and schema 2 (0.5.x) used a vocabulary that 0.5 and 0.6 renamed, such as purpose `unclassified`, and no current executable reads them. Their field inventories remain in Git history up to commit `63250d8`.

## Validation boundaries

Counts are nonnegative `u64` values. Semantic validation uses checked addition, verifies `lines = code + comments + blanks`, reconciles summary dimensions, coverage, workspace members and available breakdown/file detail, and rejects duplicate identities in array rows. Language summary file counts measure participation and may overlap; they cannot exceed source physical files or fall below the host-file counts established by breakdown rows, globally or per member; embedded breakdown rows have zero files and remain additive. Missing aggregate keys mean zero only where the contract permits it. Explicit zero summary entries are equivalent to omission.

Unknown object fields and invalid enums are rejected at typed boundaries, including effective policy. Configuration snapshots share the configuration parser's types; the JSON report schema retains JSON null semantics, while TOML schemas remove null as a format-specific constraint. Backend mapping display names and enum aliases are both accepted, matching Tokei's parser. This tightens validation of malformed reports without changing valid current measurement output.

Duplicate raw JSON keys in dynamic maps retain the last value under the existing deserializer. Schema and semantic validation see that resulting map; they do not promise to detect overwritten keys. This is distinct from duplicate array-row identities, which are rejected.

## Comparison evidence

A structurally valid headline is not a full comparison input. The internal eligibility check requires `review`, `breakdown`, `metadata.kinds`, `metadata.modules`, `metadata.coverage.unrecognized`, and effective `config` for the root or every workspace member. Missing sections are named in the error; an absent module list cannot be replaced with invented empty evidence. File records are optional. Capture a full baseline with `--output FILE` or `--format json`; an existing headline cannot reconstruct missing historical detail, and rerunning measures the current tree.

The [net-change command](PRODUCT.md#net-change) reports missing historical failure evidence as a coverage finding, without treating that absence alone as an observed failure. Forward compatibility is not guaranteed: older strict schemas may reject newer optional fields and should be updated from the producing executable.

## Subfolder Selection

Reports include `metadata.scope.subfolder` when restricted, a normalized project-relative measured directory. Consumers must inspect this field before treating `summary` as whole-project totals. Absence means no subfolder restriction. Context modules may lie outside the restriction, but measured file/coverage/review paths must lie inside it; folder summaries can retain applicable project-relative ancestors. The reader rejects malformed restrictions and workspaces whose members mix project folders with independent roots; in a workspace of project folders, every member records its folder. Comparison refuses differing restrictions, including present versus absent. Older strict readers may reject this additive field; silently removing it would misrepresent measurement scope. [PRODUCT](PRODUCT.md#subfolder-measurement) owns the supported semantics.

A comparison across producer versions can emit `counter_changed` even when the release changed only output routing: the finding conservatively records a version difference, not proof of changed counting.

## Comparison Member Selection

Comparisons optionally record `member` when `compare --member NAME` selects a workspace root. Consumers must inspect this field: summary, detail and presence counts then cover only that member, while `before`/`after` metadata retain complete input provenance. Headline output retains the selector. Absence means the existing complete comparison. Older strict comparison readers may reject the new optional field; use the schema emitted by the producing executable and never strip the selector to bypass validation. [PRODUCT](PRODUCT.md#net-change) owns selection, paths, findings and partial-status semantics.
