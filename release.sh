#!/usr/bin/env nix-shell
#!nix-shell --pure -i bash -p cacert cargo cargo-release gh git jq yj gh

set -euxo pipefail

cargo release patch

tag="$(yj -tj < Cargo.toml | jq '.package.version' -rcM)"
gh release create "$tag" -t "Release $tag" "$@"
