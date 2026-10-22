#!/bin/sh
# Prints the bucket manifest for a published release, filled from its SHA256SUMS.
set -eu
version=${1:?usage: render-manifest.sh VERSION}
file="locmetry-v$version-x86_64-pc-windows-msvc.zip"
sums=$(curl -fsSL "https://github.com/mbackschat/locmetry/releases/download/v$version/SHA256SUMS")
hash=$(printf '%s\n' "$sums" | awk -v file="$file" '$2 == file { print $1 }')
[ -n "$hash" ] || { echo "SHA256SUMS lacks $file" >&2; exit 1; }
sed -e "s/{{version}}/$version/g" \
    -e "s/{{sha256_x86_64_pc_windows_msvc}}/$hash/" \
    "$(dirname "$0")/locmetry.json"
