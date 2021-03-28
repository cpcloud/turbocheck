#!/usr/bin/env nix-shell
#!nix-shell -i bash shell.nix

set -euxo pipefail

cargo release patch

TAG="$(yj -tj < Cargo.toml | jq '.package.version' -rcM)"
echo gh release create "$TAG" -t "Release $TAG" "$@"
