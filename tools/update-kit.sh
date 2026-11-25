#!/bin/sh
# Copies the built themes of the be spex kit (github.com/bespex/kit, themes/dist) at a tag into
# dist/kit/: Maillet has no build step, so it takes the kit as files. Run again to move to a newer tag.
#
#   tools/update-kit.sh themes-v0.1.0
set -eu
tag="${1:?usage: tools/update-kit.sh <tag>, e.g. themes-v0.1.0}"
root="$(cd "$(dirname "$0")/.." && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
curl -fsSL "https://codeload.github.com/bespex/kit/tar.gz/refs/tags/$tag" | tar -xz -C "$tmp"
src="$(find "$tmp" -maxdepth 1 -mindepth 1 -type d)/themes/dist"
rm -rf "$root/dist/kit"
mkdir -p "$root/dist/kit"
cp "$src"/*.js "$src"/sx.css "$root/dist/kit/"
printf '%s\n' "$tag" > "$root/dist/kit/VERSION"
echo "dist/kit: $tag"
