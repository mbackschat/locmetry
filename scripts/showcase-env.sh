# Source from the repository root in a showcase Bash example. It extends
# cli-demo-env.sh with links to the pinned Immich checkouts, named by tag
# because reports take the project name from the checkout folder.
source scripts/cli-demo-env.sh
showcase=$(bash "$repository/scripts/fetch-showcase.sh" --check)
overlay="$repository/examples/showcase/immich/overlay"
expected=$(cd "$overlay" && find . -type f | sed 's#^\./#?? #' | LC_ALL=C sort)
for checkout in "$showcase"/immich-*; do
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
    ln -s "$checkout" "$demo_directory/${checkout##*/}"
done
