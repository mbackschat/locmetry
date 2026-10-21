# Authoring Showboat Guides

Use Showboat for executable user workflows: commands, actual captured output and interpretation. The [recorded CLI guides](cli/README.md) and the [Immich showcase recordings](showcase/README.md) are the maintained examples. This is a demonstration workflow, not a code-exploration walkthrough. [Simon Willison's introduction](https://simonwillison.net/2026/feb/10/showboat-and-rodney/) explains the approach; [Showboat's README](https://github.com/simonw/showboat) owns the tool interface.

## Scope and dependencies

One CLI guide per command is the current organization. Explain a useful task before showing its commands, then explain the evidence that matters. Cover every long option through the [option index](cli/README.md#option-coverage), without turning the guide into exhaustive permutations or copying the product contract. Link to the contract for detailed guarantees.

Showboat 0.6.1 is an optional development tool. Use the pinned uv commands in the [verification instructions](cli/README.md#verify-and-regenerate); CI uses the same versions. Keep packages and caches outside Git. Do not add Showboat or analytics tools to Cargo, release bundles or runtime requirements. The generated Markdown is readable without installing anything.

## Record actual execution

Create a document with `showboat init`, add explanations with `showboat note`, and capture commands with `showboat exec`. Preserve the generated timestamp/byline and `showboat-id` header. Keep native `bash` and `output` fences. Never invent or hand-edit captured output. Pass multiline command text through stdin or a structured subprocess argument to preserve quoting.

For example, from the repository root after building Locmetry:

```sh
showboat init /tmp/locmetry-example.md 'Measure the sample'
showboat note /tmp/locmetry-example.md 'Measure the shared synthetic source.'
showboat exec /tmp/locmetry-example.md bash <<'COMMAND'
source scripts/cli-demo-env.sh
locmetry --root acme-orders --no-user-config --format json \
  | jq '.summary.source'
COMMAND
showboat verify /tmp/locmetry-example.md
```

`exec` appends output even when a command fails. Use `showboat pop` to remove a failed last entry, then correct and rerun it. For expected errors, capture the status explicitly, assert the expected value, and show the diagnostic. Do not suppress failures with unconditional `|| true`. Do not fence commands that wait forever or require interaction: replay executes the fenced program.

## Isolate every example

Start each Bash block with `source scripts/cli-demo-env.sh`, run from the repository root. The [helper](../scripts/cli-demo-env.sh) supplies a fresh temporary copy of the [shared fixture](../examples/cli-workspace/README.md), the current executable, isolated personal configuration and a fixed display width. It cleans the copy on exit. Showboat starts a fresh shell for each block; never depend on variables or files left by another block.

Showcase recordings start with `source scripts/showcase-env.sh` instead. It verifies the pinned Immich checkouts without fetching, applies Locmetry's policy overlay, fails on any other change to a checkout and links the checkouts into the temporary directory; it never copies them. Commands measure them in place and write reports to the temporary directory.

Make edits visible in the recorded command, and keep exports beside the measured roots. Do not measure private projects, install fixture dependencies, or modify the checked-in fixture during replay. A comparison example should have a small change whose expected result can be explained independently.

Recordings made on macOS replay on Linux in CI, so recorded output must not depend on BSD or GNU formatting. Format such output explicitly: `uniq -c` pads its counts to different widths on the two systems, so pipe it through `awk '{ printf "%4d %s\n", $1, $2 }'`.

## Wrap commands and displayed output

Keep executable and output lines within 120 characters; aim for shorter command lines where natural. Use one logical argument group per continuation line, keeping an option beside its value. Break a pipeline with a trailing backslash and indent the next `|` by two spaces. Format long jq expressions across lines inside their quoted program. Use multiline SQL strings in Python rather than one long query line.

```sh
locmetry --root acme-orders --no-user-config \
  --files --output report.json --print table \
  --no-color --no-bars \
  | sed '/^Root:/d; /^Measured at:/d' \
  | demo_wrap
```

`demo_wrap` wraps long displayed text at whitespace using Python's standard library and a 120-character width. It runs inside the recorded command, so replay applies the same transformation. Use it for text, legends and diagnostics; keep JSON pretty-printed with jq. Prefer a narrower meaningful report section over folding wide Markdown table rows. Never change or trim the saved report itself to make its displayed excerpt fit.

The [verification script](../scripts/check-recorded-guides.py) rejects long fenced command/output lines and missing Showboat headers. Ordinary Markdown prose remains one paragraph per physical line. These conventions follow the owner's requested pattern from a12-rulekit's Showboat examples, adapted to Locmetry's timestamped reports.

## Keep changing metadata visible in the source command

Each measurement records the real time; temporary absolute paths vary by machine and invocation. Select stable JSON fields explicitly, or omit only the known volatile text lines with a visible filter. When a passage needs only the rows, `--tables-only` omits those lines together with legends and notes; keep the visible filter where the legends or notes are what the passage explains. For comparison tables, remove `Before measured at:` and `After measured at:` individually. Do not strip an arbitrary remainder of the report, substitute invented dates or replace all path-like strings indiscriminately.

Explain the projection in nearby prose. Exported reports retain their complete original metadata. The creation timestamp in Showboat's header is static provenance, not CLI output, and stays in the document. Use explicit output formats, `--no-color` and deterministic row ordering where needed. Both environment scripts isolate personal defaults; showcase recordings therefore omit `--no-user-config`, so readers do not mistake it for a required flag. Fixed `COLUMNS` affects report layout; it does not promise that every explanatory sentence wraps itself.

## Verify, regenerate and maintain

After changing the executable, fixture or recordings, use the debug executable from the current Cargo integration-test build, or build it once for standalone use, and run the [pinned replay command](cli/README.md#verify-and-regenerate). It verifies the command guides and the showcase recordings, checks option coverage and readability, and has a per-document timeout; `--set reference` or `--set showcase` selects one group. The Linux documentation CI job runs the same checks for pull requests and pushes to `main`, reusing the preceding test build. Each replay reports its duration; failures retain the log and any generated candidate from that same execution. The gate continues through the remaining documents and uploads failure evidence, rather than rerunning to obtain a different observation. Run `python3 scripts/test_recorded_guides.py` when changing the verifier.

Verification must fail on output drift or a change to Git state during replay. Pre-existing local edits are permitted when left unchanged; ignored build/scratch files are outside this guard. Keep `--artifacts-dir` outside the checkout. The verifier does not repair or revert a mutation automatically; inspect the retained before/after Git evidence. For intentional output changes, use `showboat verify --output` to produce a separate candidate, inspect the diff, then replace the original. For command/prose restructuring, edit those parts and regenerate captured output, or use `showboat extract --filename` to obtain an authoring sequence and rebuild it through Showboat. Replay again after the final change. Never auto-accept generated changes in CI.

Also run the affected semantic assertions. A passing replay proves the commands still produce the recorded output; it does not establish correctness by itself. Keep the fixture's independent count/classification checks, the README's showcase excerpts and documentation index checks current. When adding a new recorded guide, add it to its parent index and the verifier's inventory in the same change.
