let
  sources = import ./nix/sources.nix;
in
import sources.nixpkgs {
  overlays = [
    # rust nightly overlay
    (import sources.fenix)

    # make naersk available as a nixpkgs package
    (import ./nix/naersk.nix { inherit sources; })

    # make the rust toolchain available
    (import ./nix/rust.nix)

    # finally, make our application binary and container image available
    (import ./nix/turbocheck.nix)
  ];
}
