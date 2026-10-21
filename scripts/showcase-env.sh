# Source from the repository root in a showcase Bash example. It extends
# cli-demo-env.sh with the pinned Immich checkouts: `immich` is the showcase
# release and `<tag>/immich` each pinned release, all reporting project immich.
source scripts/cli-demo-env.sh
showcase=$(bash "$repository/scripts/fetch-showcase.sh" --check)
overlay="$repository/examples/showcase/immich/overlay"
expected=$(cd "$overlay" && find . -type f | sed 's#^\./#?? #' | LC_ALL=C sort)
showcase_tag=$(awk -F'\t' '$1 == "showcase" { print $2 }' "$repository/examples/showcase/immich/pins.tsv")
for checkout in "$showcase"/*/immich; do
    [[ -d $checkout/.git ]] || continue
    # The overlay is Locmetry's policy; anything else changed in the checkout
    # would make the recorded numbers describe a different tree.
    cp -R "$overlay/." "$checkout/"
    actual=$(git -C "$checkout" status --porcelain=v1 --untracked-files=all | LC_ALL=C sort)
    if [[ $actual != "$expected" ]]; then
        echo "$checkout differs from its pinned commit beyond the policy overlay:" >&2
        diff <(printf '%s\n' "$expected") <(printf '%s\n' "$actual") >&2 || true
        exit 1
    fi
    tag=$(basename "$(dirname "$checkout")")
    mkdir "$demo_directory/$tag"
    ln -s "$checkout" "$demo_directory/$tag/immich"
done
ln -s "$showcase_tag/immich" "$demo_directory/immich"
