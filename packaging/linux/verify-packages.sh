#!/bin/sh
# Installs each .deb and .rpm in DIRECTORY into a clean Debian or Fedora
# container, measures a synthetic project and removes the package again.
set -eu
directory=$(cd "${1:?usage: verify-packages.sh DIRECTORY}" && pwd)

# Runs inside the container after installation. The helper must report the
# missing jq instead of failing obscurely, since packages only suggest it.
# Slim images skip /usr/share/doc and /usr/share/man on installation, so the
# notices and the man page are checked in the package's file list instead.
smoke='
mkdir -p /tmp/project
printf "print(1)\n" > /tmp/project/main.py
env -i /usr/bin/locmetry --root /tmp/project --no-user-config --format json | grep "\"Python\"" > /dev/null
$contents | grep usr/share/doc/locmetry/THIRD-PARTY-NOTICES.md > /dev/null
$contents | grep usr/share/man/man1/locmetry.1 > /dev/null
test -s /usr/share/bash-completion/completions/locmetry
test -s /usr/share/fish/vendor_completions.d/locmetry.fish
ls /usr/share/zsh/*/_locmetry > /dev/null
status=0
env -i PATH=/usr/bin:/bin /usr/bin/compare-locmetry-files.sh a.json b.json 2> /tmp/err || status=$?
test "$status" = 1
grep -q "package manager" /tmp/err
'

found=0
for package in "$directory"/*.deb "$directory"/*.rpm; do
    [ -e "$package" ] || continue
    found=$((found + 1))
    name=$(basename "$package")
    case "$name" in
        *amd64.deb | *x86_64.rpm) platform=linux/amd64 ;;
        *arm64.deb | *aarch64.rpm) platform=linux/arm64 ;;
        *) echo "unknown architecture: $name" >&2; exit 1 ;;
    esac
    case "$name" in
        *.deb) image=debian:13-slim install="apt-get install --yes --no-install-recommends" contents="dpkg-deb -c" remove="apt-get remove --yes locmetry" ;;
        *.rpm) image=fedora:43 install="rpm -i" contents="rpm -qlp" remove="rpm -e locmetry" ;;
    esac
    docker run --rm --platform "$platform" -v "$directory:/packages:ro" "$image" \
        sh -euc "contents='$contents /packages/$name'
$install /packages/$name
$smoke
$remove
test ! -e /usr/bin/locmetry"
    echo "verified $name"
done
[ "$found" -gt 0 ] || { echo "no .deb or .rpm in $directory" >&2; exit 1; }
