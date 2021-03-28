#!/usr/bin/env nix-shell
#!nix-shell --keep GITHUB_USER --keep GITHUB_TOKEN --pure -i bash -p cacert cargo cargo-release gh git jq yj gh

set -euxo pipefail

cargo release "${1:-patch}"

if [ "$#" -gt 1 ]; then
  shift 1
fi

tag="$(yj -tj < Cargo.toml | jq '.package.version' -rcM)"
gh release create "$tag" --target "$(git rev-parse HEAD)" -t "Release $tag" "$@"
