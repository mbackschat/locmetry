#!/usr/bin/env bash
# Fetch the pinned Immich checkouts of the showcase, or with --check verify them
# without network access; both print the showcase directory on success. See
# examples/showcase/immich/README.md for the location and pinning contract.
set -euo pipefail

repository=$(cd "$(dirname "$0")/.." && pwd)
pins="$repository/examples/showcase/immich/pins.tsv"
upstream=https://github.com/immich-app/immich.git

mode=fetch
case ${1:-} in
    "") ;;
    --check) mode=check ;;
    *) echo "usage: $0 [--check]" >&2; exit 2 ;;
esac

# macOS backs up ~/.cache with Time Machine but excludes ~/Library/Caches.
if [[ -n ${LOCMETRY_SHOWCASE_DIR:-} ]]; then
    showcase=$LOCMETRY_SHOWCASE_DIR
elif [[ $(uname -s) == Darwin ]]; then
    showcase="$HOME/Library/Caches/locmetry/showcase"
else
    showcase="${XDG_CACHE_HOME:-$HOME/.cache}/locmetry/showcase"
fi

while IFS=$'\t' read -r role tag commit; do
    [[ $role == \#* || -z $role ]] && continue
    # Both releases are folders named immich, because reports take the project
    # name from the folder and comparison flags differing names.
    checkout="$showcase/$tag/immich"
    if [[ -d $checkout ]]; then
        actual=$(git -C "$checkout" rev-parse HEAD 2>/dev/null || echo none)
        if [[ $actual != "$commit" ]]; then
            echo "$checkout is at $actual, not the pinned $role commit $commit; remove it and run bash scripts/fetch-showcase.sh" >&2
            exit 1
        fi
    elif [[ $mode == check ]]; then
        echo "Missing the pinned $role checkout $checkout; run bash scripts/fetch-showcase.sh" >&2
        exit 1
    else
        mkdir -p "$showcase/$tag"
        partial="$checkout.partial"
        rm -rf "$partial"
        git -c advice.detachedHead=false clone --quiet --depth 1 --single-branch --branch "$tag" "$upstream" "$partial"
        actual=$(git -C "$partial" rev-parse HEAD)
        if [[ $actual != "$commit" ]]; then
            rm -rf "$partial"
            echo "Upstream tag $tag now points to $actual, not the pinned $commit" >&2
            exit 1
        fi
        mv "$partial" "$checkout"
        echo "Fetched $tag into $checkout" >&2
    fi
done < "$pins"

printf '%s\n' "$showcase"
