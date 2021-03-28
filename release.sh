#!/usr/bin/env nix-shell
#!nix-shell --keep GITHUB_USER --keep GITHUB_TOKEN --pure -i bash -p cacert cargo cargo-release gh git jq yj gh

set -euxo pipefail

cargo release patch

tag="$(yj -tj < Cargo.toml | jq '.package.version' -rcM)"
gh release create "$tag" --target "$(git rev-parse HEAD)" -t "Release $tag" "$@"
