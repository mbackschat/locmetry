#!/usr/bin/env bash
# Record docs/assets/demo.gif from scripts/demo.tape: measure, explain and
# compare on the pinned Immich showcase. Needs VHS and gifsicle (brew install
# vhs gifsicle) and the checkouts from scripts/fetch-showcase.sh. The output is
# not byte-reproducible, so rerun it after changes to the commands or their
# output, not in CI.
set -euo pipefail
repository=$(cd "$(dirname "$0")/.." && pwd)
cd "$repository"
command -v vhs > /dev/null || { echo "VHS is missing; install it with: brew install vhs" >&2; exit 1; }
command -v gifsicle > /dev/null || { echo "gifsicle is missing; install it with: brew install gifsicle" >&2; exit 1; }
cargo build --release --locked
export LOCMETRY_BIN="$repository/target/release/locmetry"
source scripts/showcase-env.sh

# Reports print their canonical root, so the measured copy lives in a fixed,
# neutral folder that keeps local checkout paths and random names out of the
# GIF. mkdir without -p claims it, so the trap removes only a stage this run
# created. Its .git stays because ignore-file selection depends on the Git
# work tree.
stage=/tmp/locmetry-demo
mkdir "$stage" 2> /dev/null || {
    echo "$stage exists; remove it unless another recording is running" >&2
    exit 1
}
trap 'rm -rf "$demo_directory" "$stage"' EXIT
export LOCMETRY_DEMO_STAGE="$stage"
if [[ $(uname -s) == Darwin ]]; then
    cp -cR "$showcase_tag/immich/" "$stage/immich"
else
    cp -R --reflink=auto "$showcase_tag/immich/" "$stage/immich"
fi
locmetry --root "$baseline_tag/immich" --output "$stage/$baseline_tag.json"
locmetry --root "$stage/immich" --output "$stage/$showcase_tag.json"

# The tape types the pinned tags; fail before recording if they moved.
grep -q "$baseline_tag.json ../$showcase_tag.json" "$repository/scripts/demo.tape"
unset COLUMNS
vhs "$repository/scripts/demo.tape" --output "$demo_directory/demo.gif"
# With the tape's 12-point font, 40 colours brings 655 KB down to about 350 KB
# (measured 2026-10-23) and keeps dim zeros, rules and the rose of decreases
# distinct; 32 colours or --lossy merged decreases into the test magenta and
# left traces of earlier scenes, and 12 frames per second saved only 1%.
gifsicle -O3 --colors 40 "$demo_directory/demo.gif" -o "$repository/docs/assets/demo.gif"
