# Strengthen Measurement Before Expanding Tool Coverage

Historical snapshot: statements and open findings below describe the pinned research baseline, not current development behavior. S1–S5 subsequently corrected notebook counting/coverage, unified contracts and added `compare` and `explain`; see the [implementation and review record](../plans/measurement-quality-and-net-change.md) for resolutions and the [product contract](../PRODUCT.md) for current guarantees. The original evidence is preserved below.

Research date: 2026-10-15. **The first findings to act on are two Locmetry correctness defects, followed by truthful documentation and stronger report contracts.** Broader tool research supports the product's focus, but not a feature-parity project or an accuracy ranking. Generated-file reasons, duplicate evidence and selection explanations are the most relevant optional ideas; history, cost estimates and additional renderers solve separate problems. This document records open findings and recommendations for discussion. No runtime fixes, product-scope changes, releases or accepted roadmap decisions are made here; net change remains the [existing proposal](../proposals/archive/net-change.md).

The [capability comparison](tool-capability-comparison.md) covers the major functionality families of scc 4.1.0, cloc 2.10, Tokei 15.0.0, gocloc 0.7.0 and pygount 3.2.0 against Locmetry 0.1.6. The [configuration/documentation audit](configuration-and-documentation.md) records precise interface gaps. External findings are documentation/source inspection, not executed peer tests. The notebook findings below were reproduced against a fresh build of Locmetry commit `bece839731d78092e997b5b8b959801324e64b26` on 2026-10-15.

## Notebooks break two existing measurement guarantees

**Open correctness finding 1: notebook source is counted twice.** A valid notebook with one Python line reports two code lines; an independent two-line notebook reports four. Tokei's `parse_jupyter` adds each cell to both the parent statistics and its language blob. Locmetry's `language_counts` adds parent `own()` and blob counts assuming they are disjoint. That representation assumption breaks the product's count-once invariant. The defect is in normalization of an upstream representation, not a general reason to discard embedded counting. ([Tokei cell aggregation](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/src/language/language_type.rs#L279), [Locmetry adapter](../../src/backend.rs), [reporting invariant](../PRODUCT.md#reporting-and-coverage))

**Open correctness finding 2: invalid notebooks become successful zero measurements.** Both malformed JSON and syntactically valid JSON with the wrong notebook shape produce zero-count file records. Tokei's Jupyter branch uses `unwrap_or_default()` on parsing failure; no I/O error reaches Locmetry's coverage logic. A scan of those two invalid files and the two valid fixtures exits 0 with all four selected files counted, no unread/unrecognized files, and `backend_inaccurate = false`. Missing parse evidence is therefore indistinguishable from measured zero. ([Tokei parse fallback](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/src/language/language_type.rs#L67), [coverage implementation](../../src/backend.rs), [missing-count contract](../PRODUCT.md#reporting-and-coverage))

The reproduction uses only temporary synthetic inputs. From the repository root, build with `cargo build --locked`, then run the following Python 3 probe; Python is a research convenience, not a Locmetry runtime dependency. Notebook counts here mean extracted code-cell lines, not physical lines of serialized JSON.

```python
import json
from pathlib import Path
import subprocess
import tempfile

binary = Path("target/debug/locmetry").resolve()
with tempfile.TemporaryDirectory(prefix="locmetry-notebook-probe-") as temp:
    root = Path(temp)
    for name, source in [("one", ["print(1)\n"]),
                         ("two", ["print(1)\n", "print(2)\n"])]:
        notebook = {
            "nbformat": 4, "nbformat_minor": 5,
            "metadata": {
                "kernelspec": {"display_name": "Python 3", "name": "python3",
                               "language": "python"},
                "language_info": {"name": "python", "file_extension": ".py"},
            },
            "cells": [{"id": name, "cell_type": "code", "metadata": {},
                       "execution_count": None, "outputs": [], "source": source}],
        }
        (root / f"{name}.ipynb").write_text(json.dumps(notebook))
    (root / "broken.ipynb").write_text("{ not valid json\n")
    (root / "wrong-shape.ipynb").write_text('{"hello":"not a notebook"}\n')
    run = subprocess.run(
        [str(binary), "--root", str(root), "--no-user-config", "--files",
         "--output", "-"], capture_output=True, text=True)
    report = json.loads(run.stdout)
    print("exit", run.returncode)
    print("backend_inaccurate", report["metadata"]["backend_inaccurate"])
    print("coverage", report["metadata"]["coverage"])
    print([(file["path"], file["counts"]) for file in report["files"]])
```

| Fixture | Independent expected outcome | Observed current-source outcome |
| --- | --- | --- |
| `one.ipynb` | One extracted Python code line. | Two code lines: one parent Jupyter contribution plus one Python contribution. |
| `two.ipynb` | Two extracted Python code lines. | Four code lines: two parent plus two Python. |
| `broken.ipynb` | Parsing unavailable; must not certify zero. | Zero code/comments/blanks/lines, recorded as counted. |
| `wrong-shape.ipynb` | Parsing unavailable; must not certify zero. | Zero code/comments/blanks/lines, recorded as counted. |

The fresh build completed successfully; its binary SHA-256 was `beb7048b3d0f8c2e89d9f81842bc7666b3df70747b7fc754886f31f73ffe23ee`. The combined run reported `selected_files = 4`, `counted_files = 4`, `unrecognized_files = 0`, `unread = []`, `backend_inaccurate = false`, and exit 0. The independent second instance for each mechanism supports a class-level finding. This was not a complete notebook audit: Markdown/raw cells, unsupported kernels and other container semantics still need examination before claiming comprehensive notebook correctness.

The recommendation is two separate coherent fixes: normalize overlapping notebook representations at the backend boundary, and make structured-document parse failure observable as unavailable measurement. Do not special-case the fixture filenames or merely adjust displayed totals. **Arithmetic reconciliation cannot catch a duplication repeated consistently through files and aggregates.** Guards need independent tiny-file count oracles and malformed-container cases, as well as reconciliation. Exact public error/exit treatment should be decided with the parse-failure fix; research has not silently widened the existing unread-file exit-2 contract.

## Shared contracts and accurate examples come next

| Priority | Recommended work | Acceptance question |
| --- | --- | --- |
| First | Correct notebook normalization and parse-failure reporting. | Do independent one/two-line and invalid/wrong-shape fixtures preserve count-once and unavailable-versus-zero semantics? |
| Next | Apply the [documented README/product corrections](configuration-and-documentation.md#documentation-deserves-the-same-consistency-checks-as-code), including safe CI recipes. | Can a reader understand recognized-file scope, local counter corrections, `--files`, actual platform linkage, module inference, and failure-preserving pipelines without reading source? |
| Before reports become comparison inputs | Follow the existing proposal's shared typed report/schema ownership; describe effective policy and nonnegative counts, and validate semantic constraints separately. | Can an agent discover valid structure and omission semantics from the executable, and are structurally invalid or semantically inconsistent inputs rejected? |
| Preserve throughout | One computed measurement/comparison result across JSON, Markdown and terminal renderers; explicit member/policy/coverage provenance. | Do all views retain the same counts and missing-evidence meaning, including zero-net entity replacement? |

This ordering protects net change's value. Subtracting two plausible but wrong measurements yields a plausible wrong delta; a schema cannot repair that. The proposal's union of folders, detected packages/modules and workspace roots remains appropriate, including arrivals/departures with zero aggregate change. Keep the [existing comparison guards](net-change-tools.md#carry-focused-guards-into-the-proposal) and distinguish descriptive net change from churn, source alignment and policy gates.

A further pre-existing parser weakness belongs in a separate follow-up: Locmetry's `.gitmodules` reader uses trimmed prefixes and splitting rather than full Git-config parsing, so quoted/escaped paths and inline-comment forms are not decoded. This was source-inspected, not reproduced. Recommend either explicit supported-syntax documentation or robust parsing after establishing fixtures; do not fold it into notebook work. ([Reader](../../src/backend.rs))

## Evidence can improve scope without silently changing totals

| Candidate | Recommendation and rationale | Boundary to decide before implementation |
| --- | --- | --- |
| Generated/minified evidence | Consider only when users need help identifying scope mistakes. Borrow pygount's explicit reason and scc's configurable marker/size heuristics; report suspicion before automatically excluding content. | Define inspected bytes/lines, thresholds, reasons, false positives, policy provenance and whether generated-but-maintained code remains owned. [pygount reasons](https://github.com/roskakori/pygount/blob/v3.2.0/pygount/analysis.py#L375), [scc heuristics](https://github.com/boyter/scc/blob/v4.1.0/processor/workers.go) |
| Exact duplicate evidence | Potentially useful as groups of equal-content paths while preserving physical totals. Avoid importing default deduplication. | Copies can belong to production and test or separate roots; unique-content metrics need their own scope and deterministic representative rules. This would extend current workspace exclusions. [cloc default deduplication](https://github.com/AlDanial/cloc/blob/v2.10/cloc#L1450), [pygount duplicate reference](https://github.com/roskakori/pygount/blob/v3.2.0/pygount/analysis.py#L385), [Locmetry workspace scope](../PRODUCT.md#per-repository-breakdown) |
| Selection/count explanation | Strongest optional usability candidate: complement current rule IDs and unused entries with answers for excluded/unrecognized paths and counter behavior. cloc's found/counted/ignored inventories and filter-stage explanation provide concrete precedents for debugging workflows. | Define which evidence can be retained without recording every pruned file or duplicating counting logic. Scope an explanation workflow from actual debugging tasks. [cloc diagnostics](https://github.com/AlDanial/cloc/blob/v2.10/cloc#L2310), [Locmetry findings](../../src/report.rs) |
| Binary/encoding/container policy | Audit and document first. A synthetic file `binary.rs` containing bytes `00 00 00 0a` counted as one Rust code line with no coverage warning in the initial existing-binary probe; source has no content binary filter. That binary's build commit was not verified, so this observation is weaker than the fresh notebook evidence. | A NUL heuristic can help but can also reject text encodings; define scope and unavailable states before changing selection defaults. No broad binary-accuracy claim follows. [Tokei parsing](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/src/language/language_type.rs), [Locmetry adapter](../../src/backend.rs) |
| Language/help discovery | A compact language listing and task-focused schema examples could reduce agent effort while retaining complete semantic schemas. | Validate the actual agent workflow first; do not create a second registry that drifts from parser types/constants. [Tokei language listing](https://github.com/XAMPPRocky/tokei/blob/v15.0.0/src/cli.rs), [Locmetry schema ownership](../../src/config/schema.rs) |

**Explicit processing states are the most transferable peer lesson.** Generated, duplicate, binary, unknown, unreadable and malformed are different facts; none should collapse silently into proven zero. Conversely, a heuristic finding is not automatically an error or a reason to exclude a maintained file. This is where richer evidence can improve Locmetry's existing purpose without turning it into a general quality scanner.

## Keep adjacent capabilities conditional on real consumers

| Topic for discussion | Recommendation | What would change that recommendation |
| --- | --- | --- |
| Git hotspots, coupling, authorship and timelines | Keep outside this proposal. scc already demonstrates a substantial separate Git-analysis product surface; saved-report net change needs neither repository history nor source checkout. | A concrete history-analysis use case that cannot be satisfied by interoperating with existing tools. [scc history](https://github.com/boyter/scc/blob/v4.1.0/processor/history.go) |
| Complexity, ULOC/DRYness, cost/effort models | Do not add for parity. Their heuristic/formula semantics differ from owned production/test LOC and would need independent contracts. | A stated decision requiring one of these metrics, with an agreed interpretation and validation corpus. [scc metrics](https://github.com/boyter/scc/blob/v4.1.0/processor/workers.go), [cost model](https://github.com/boyter/scc/blob/v4.1.0/processor/locomo.go) |
| MCP, HTML, CSV/SQL/OpenMetrics, streaming | Defer until a consumer needs an interface or format. Strengthen CLI/JSON/schema consistency first. | A client unable to invoke the CLI, a stakeholder-sharing workflow, an analytics integration, or measured memory pressure. scc's separate MCP policy is a caution against interface drift. [MCP](https://github.com/boyter/scc/blob/v4.1.0/mcp.go), [formats](https://github.com/boyter/scc/blob/v4.1.0/processor/formatters.go) |
| Archive/remote inputs, arbitrary selector commands, custom lexers, another backend | Keep outside the immediate discussion. These expand deployment, parsing and authority surfaces beyond the present embedded-counter goal. | A demonstrated unsupported repository/input requirement and a representative equal-scope correctness study. [cloc inputs](https://github.com/AlDanial/cloc/blob/v2.10/cloc#L1904), [pygount remote inputs](https://github.com/roskakori/pygount/blob/v3.2.0/pygount/git_storage.py), [integration boundary](../tokei-integration.md) |

The recommended discussion therefore starts with correctness and documentation/schema hardening, then considers selection explanations before optional content heuristics. The research supports a defensible focus: classified measurements whose scope and uncertainty can be inspected. It does not quantify return on investment, prove superiority, or justify replacing Tokei merely because another tool advertises more features.
