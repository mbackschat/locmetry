//! The `--help` text of options whose `-h` line is a one-line summary
//! (docs/help-design.md#what-belongs-where), and the long footers of each command.

/// The root `--help` examples and notes after the short footer. A macro
/// because `concat!` accepts only literals. The README "Common commands"
/// table mirrors these examples; tests/integration/readme.rs enforces it.
macro_rules! root_examples {
    () => {
        "\n\nExamples:
  Measure another project:
    locmetry --root ../service
  Inspect scope, classification and coverage notes:
    locmetry --verbose
  Measure a subfolder with project policy and module context:
    locmetry --subfolder web/src
  Explain a file's classification or missing counts:
    locmetry explain web/src/orders/Order.test.ts
  Save full JSON with per-file decisions outside the root:
    locmetry --files --output ../report.json
  Save full JSON and print the table from one scan, as for agents:
    locmetry --output ../report.json --print table
  Save a Markdown report:
    locmetry --format markdown --output ../report.md
  Measure repositories together, each with its own policy:
    locmetry workspace --root ../web --root ../api
  Show production source in the table, with its comment density:
    locmetry --format table --table-purpose production
  Inspect the configuration contract:
    locmetry --schema config
  Read full JSON on stdout:
    locmetry --root service --format json --files
  Measure selected source, including ignored files, except vendor:
    locmetry --root service --subfolder src --exclude 'src/vendor/**' --hidden true --respect-ignore-files false --format json
  Show adaptive folder detail and production categories:
    locmetry --root service --format table --folder-rows 30 --table-purpose production --show-categories --show-bars=comments --sort name
  Save a sorted Markdown tree to depth 3:
    locmetry --root service --format markdown --markdown-depth 3 --sort name --output report.md
  Measure an explicit workspace definition:
    locmetry workspace --workspace-file repos.toml --files --output workspace.json --print table
  Compare two saved full reports:
    locmetry compare before.json after.json --format table

  For a production-rule change covered by the example below, inspect existing
  configuration and measure current decisions, then adapt the paths and category
  and run the verification command. Reading a guide or schema first is optional
  for this route. For another need, read the section that answers it: test or
  script categories in --guide configure --section refine-categories, category
  names in --guide categories --section choose-a-name, the rest via --guide index.
  Verify the reported per-file decisions and unresolved review items before concluding.

  Classify maintained scripts in service/.locmetryrc (TOML):
    version = 1

    [[rules]]
    id = \"maintained-scripts\"
    match = [\"scripts/**\"]
    purpose = \"production\"
    category = \"tooling\"
  Rule patterns are root-relative; built-in test/build classification precedes rules.
  Verify the policy with per-file decisions and a human overview:
    locmetry --root service --files --output report.json --print table --verbose

Read results:
  Lines = Code + Comments + Blanks. Default tables describe source; JSON
  summary.source and summary.all_counted distinguish source from data/documentation.
  Test/prod = Test Code / Production Code, excluding Other; it is not test coverage.
  Files are classified whole; a legend names Rust #[cfg(test)] code in production.
  Other remains counted. Unclaimed source appears in review; a clear review list
  does not prove correct classification. Unavailable counts are not zero.
  Use --verbose for coverage, inferred build modules, submodule hints and unused
  policy. Full JSON has breakdown, review and metadata.coverage; --files adds files.

Choose scope and views:
  --subfolder retains the root's policy and module context; --root on that folder
  starts a different measurement. Configuration is not discovered in ancestors.
  Quote root-relative exclusion globs. --table-* options filter tables, not JSON or Markdown.
  Folder parent rows include descendants.

Broader policy:
  Project overrides precede test/build conventions; ordinary rules follow them.
  Use exclusions for scope, intentional Other for counted historical material,
  and language mappings for unrecognized extensions. Build modules infer conventional
  layouts from descriptor names without executing builds; submodule hints are not
  ownership decisions. --schema config gives precedence, conventions and settings.
  Personal defaults only add scope exclusions and language mappings. Inspect
  --schema user-config; use --no-user-config for reproducible runs without them.

Automation:
  --output FILE saves full JSON by default; --files adds per-file decisions and
  --print table displays that same scan. Save outside every measured root.
  Exit 0 means completed, not necessarily complete language recognition.
  Exit 2 writes a partial report with unavailable/inaccurate counts; retain that
  status. Exit 1 is a command failure, with no usable report promised. Diagnostics
  go to stderr. Preserve status before querying JSON or running a pipeline.

For routine capture-and-compare tasks, start with locmetry compare --help and execute
its recipes. Extra guide/schema reading is optional for that route; inspect summary
and findings before drawing conclusions.
For workflows and recipes beyond these examples, locmetry --guide index names the
guide section or schema definition to read."
    };
}

pub(super) const SHOW_BARS: &str = "Add charts to saved or printed tables, beside them when they fit, otherwise below. Chart code share by purpose or comment density; selecting one table purpose uses comment density even with =purpose. Terminals show fitting bars automatically; COLUMNS overrides the width. Table format only";

pub(super) const TABLE_LANGUAGE: &str = "Show only these source languages in tables; comma-separated, in any case. A variant such as TSX selects its family's row unless --split-languages. Table format only";

pub(super) const SUBFOLDER: &str = "Measure this root-relative folder and descendants, retaining project configuration and module context";

pub(super) const TABLE_CATEGORY: &str = "Show only these categories' source in tables; comma-separated PURPOSE for all its categories or PURPOSE:CATEGORY. '*' matches any text; quote it in the shell. Test/prod gives way to Comment % when only part of Production or Test is shown. Table format only.";

pub(super) const FOLDER_ROWS: &str = "Show the folder table and split the largest folders within an N-row budget; no top-level folder's code is merged away. Table format only";

pub(super) const SECTION: &str =
    "Print one section of the guide or schema; an unknown ID lists the valid ones";
pub(super) const GUIDE: &str = "Read a task guide. Use --help for syntax, --guide for workflows and caveats, and --schema for machine-readable contracts. Standalone query";

pub(super) const COMPARE_OUTPUT: &str =
    "Write only to FILE; - selects stdout. Defaults to full JSON; --format overrides it";

pub(super) const TABLES_ONLY: &str = "Print only the tables, without titles, legends, notes, findings or the closing location and date lines. Table format only";

pub(super) const SPLIT_LANGUAGES: &str = "Show language variants such as TSX, JSX and C++ Header as rows of their own. By default tables and Markdown count them in their family's row, such as TypeScript, and name each fold. JSON keeps every backend language";

pub(super) const WORKSPACE_ROOT: &str =
    "Member root; repeatable. Its folder name prefixes its paths in the report";

pub(super) const WORKSPACE_FILE: &str = "Read the member roots from a TOML workspace file instead; its roots are relative to the file (see --schema workspace)";

pub(super) const OUTPUT: &str = "Write only to FILE, outside the scan root; - selects stdout. Defaults to full JSON; --format overrides it. File extensions do not select a format. Add --print table to also print the overview from the same run";

pub(super) const PRINT: &str = "With --output FILE, also print the same measurement to stdout in FORMAT, such as table for an overview or headline for compact JSON";

pub(super) const FILES: &str =
    "Include per-file records in full JSON, saved or printed; headline omits them";

pub(super) const SHOW_CATEGORIES: &str =
    "Add a source purpose/category table to saved or printed table output";

pub(super) const TABLE_PURPOSE: &str =
    "Show only these purposes' source in tables; comma-separated. Table format only";

pub(super) const SORT: &str = "Row order of table and Markdown output; the Markdown folder table keeps its path hierarchy, and the review list stays largest first";

pub(super) const MARKDOWN_DEPTH: &str = "Folder-breakdown depth in Markdown; 0 leaves no data rows. Relative to --subfolder; workspace members occupy level 1. Other evidence and JSON retain deeper paths";

pub(super) const NO_COLOR: &str = "Disable terminal colors, bold and dim text, including help and diagnostics. A nonempty NO_COLOR does the same";

pub(super) const WORKSPACE_EXAMPLES: &str = "Examples:
  Measure roots directly, keeping each member's policy:
    locmetry workspace --root web --root api --files --output workspace.json --print table
  Measure a workspace file:
    locmetry workspace --workspace-file repos.toml --files --output workspace.json --print table
  Export member, category and language detail:
    locmetry workspace --workspace-file repos.toml --format markdown --markdown-depth 3 --sort name --output workspace.md
  Explain a member file:
    locmetry explain web/src/index.ts --workspace-file repos.toml --format json

repos.toml:
  version = 1
  roots = [\"web\", \"api\"]

Paths are relative to the workspace file; it is used only when --workspace-file names it.
Members need unique folder names and non-overlapping roots. Each scan uses
its own policy: each member's .locmetryrc and the personal defaults unless
--no-user-config is given. Set exclusions and overrides in member configuration;
workspace-wide --config/--exclude/--hidden/--respect-ignore-files are unsupported.
For one repository's components, a file can name project and folders instead of
roots: each folder is measured like --subfolder under the project's .locmetryrc.
Save outputs outside every member. JSON paths are member-prefixed; summary.members
holds per-member totals. Workspace Test/prod is sum(Test Code) / sum(Production Code),
not an average of member ratios. --verbose exposes member coverage and policy findings.

Save before.json and after.json using the same workspace command around a change,
then run locmetry compare before.json after.json --format table. Member names match
snapshots; an absent member means it left the measured workspace, not that its files
were deleted. Use compare --member NAME to inspect one member and its subfolder changes.
See locmetry compare --help for full comparison recipes and limitations.
Use locmetry --schema workspace for the file contract, locmetry --guide configure
for broader policy, and locmetry --guide reports for interpretation.";

pub(super) const COMPARE_EXAMPLES: &str =
    "For two full reports of the same mode and subfolder scope, execute the recipes below.
Inspect summary and findings before drawing conclusions. Additional guide/schema
reading is optional for this route; use the references below for uncovered semantics.

Examples:
  Capture a full baseline before changing files or policy:
    locmetry --root service --output before.json
  Capture the new state after the intended change:
    locmetry --root service --output after.json
  Inspect net change as a table:
    locmetry compare before.json after.json --format table
  Save the complete comparison:
    locmetry compare before.json after.json --output change.json
  Save a Markdown comparison:
    locmetry compare before.json after.json --format markdown --output change.md
  Capture a workspace baseline instead:
    locmetry workspace --workspace-file repos.toml --output before.json
  Capture the workspace after the intended change:
    locmetry workspace --workspace-file repos.toml --output after.json
  Inspect one workspace member and its immediate subfolders:
    locmetry compare before.json after.json --member api --format table

For --member, use two workspace reports and an exact member name present in either report.
The example assumes repos.toml includes api; unknown names and single-root inputs fail.
Check Member: NAME and Source total: grids and Test/prod cover only that member.
The folder table shows changed immediate subfolders; Markdown/JSON retain deeper detail.
JSON retains the member selector and member-prefixed paths. Input metadata, report-level
findings and partial status still cover the whole workspace inputs. Nested-folder
selection is unsupported. Nothing is rescanned.

Inputs must be full measurement JSON; headlines cannot be baselines. --files is
optional for aggregate comparison. Capture both inputs before comparing; rescanning
now cannot recover an earlier state. Compare two single roots or two workspaces,
with matching subfolder restrictions; use the same workspace form for after.json.
Measurement outputs must stay outside their roots. Comparison output must not
replace either input. --output is silent; use --format json for full JSON on stdout.
There is no --print option for compare.

Read summary.source/all_counted for counts; full JSON also has members, folders,
packages and findings. Human grids omit unchanged rows but totals include them.
Net change is after minus before, not Git additions/deletions, churn or per-file diff.
A departing workspace member does not prove source deletion. Recursive folder totals
include descendants. Inspect findings for changed policy, coverage and presence;
changed counts alone do not prove source-only growth. Exit 2 retains partial input
counts, exit 1 rejects invalid input, and exit 0 does not prove identical settings.
Use locmetry --schema comparison for fields or locmetry --guide compare for detail.";

pub(super) const EXPLAIN_EXAMPLES: &str = "Examples:
  Inspect a project-relative file:
    locmetry explain src/main.rs --root service
  Read a complete JSON explanation:
    locmetry explain src/main.rs --root service --format json
  Check several files, one JSON line each:
    locmetry explain src/main.rs tests/smoke.rs --root service --format json
  Diagnose a member file:
    locmetry explain web/src/index.ts --workspace-file repos.toml --format json
  Check an exclusion under a proposed scope:
    locmetry explain src/main.rs --root service --exclude 'src/**' --format json

PATH names an existing, exact-spelling root-relative file; workspace paths use
MEMBER/PATH while policy patterns remain relative to that member. Workspace mode
uses the member's own configuration and rejects single-root policy flags.
Several PATHs are explained in order; JSON then prints JSON Lines, one complete
explanation per line, and any target with unavailable counts makes the exit 2.
This inspects the current filesystem, not a saved report or missing/deleted file.
Redirect stdout to save JSON; explain has no --output option and no headline format.

Read result.state first: counted includes file counts and decision stage/patterns;
excluded names a scope exclusion; not_selected names an established ignore/hidden
cause or an unknown cause; unrecognized has no parser or counts. unread and
parse_failed mean unavailable counts (exit 2). symlink stops without following it.
Counted Other/unclaimed source is still counted. Zero is meaningful only with known
counts. Other successful states exit 0; invalid paths/configuration fail with exit 1.
Use locmetry --schema explanation for the tagged result, or locmetry --guide selection
for broader diagnosis. --schema and --guide are root-level standalone queries.";
