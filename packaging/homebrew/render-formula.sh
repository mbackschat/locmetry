#!/bin/sh
# Prints the tap formula for a published release, filled from its SHA256SUMS.
set -eu
version=${1:?usage: render-formula.sh VERSION}
sums=$(curl -fsSL "https://github.com/mbackschat/locmetry/releases/download/v$version/SHA256SUMS")

sum() {
    value=$(printf '%s\n' "$sums" | awk -v file="locmetry-v$version-$1.tar.gz" '$2 == file { print $1 }')
    [ -n "$value" ] || { echo "SHA256SUMS lacks $1" >&2; exit 1; }
    printf '%s' "$value"
}

darwin_arm=$(sum aarch64-apple-darwin)
linux_x86=$(sum x86_64-unknown-linux-musl)
linux_arm=$(sum aarch64-unknown-linux-musl)
sed -e "s/{{version}}/$version/g" \
    -e "s/{{sha256_aarch64_apple_darwin}}/$darwin_arm/" \
    -e "s/{{sha256_x86_64_unknown_linux_musl}}/$linux_x86/" \
    -e "s/{{sha256_aarch64_unknown_linux_musl}}/$linux_arm/" \
    "$(dirname "$0")/locmetry.rb"
