# Release compatibility fixtures

`v0.7.3-root.json` and `v0.7.3-workspace.json` were generated on 2026-10-21 by the released 0.7.3 Homebrew executable with `--no-user-config --files`, on a synthetic npm package with a JavaScript source and test and a Cargo crate, measured alone and as a two-member workspace. Paths were normalized to `/synthetic` and timestamps to zero. They lock that the current reader accepts and compares schema-3 reports of the latest release, whose summary predates later optional fields.

[Report compatibility](../../../docs/report-compatibility.md) describes how schema 3 grows and what readers must accept. Reports before schema 3 need no fixture: `compare` refuses them by their version alone.
