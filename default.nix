let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs {
    overlays = [
      (import sources.nixpkgs-mozilla)
      (import ./nix/overlays/rust.nix)
    ];
  };

  naersk = pkgs.callPackage sources.naersk { };
in
rec {
  turbocheck = naersk.buildPackage {
    root = ./.;
    buildInputs = with pkgs; [ openssl pkg-config ];
  };
  turbocheck-image = pkgs.dockerTools.buildLayeredImage {
    name = "turbocheck";
    config = {
      Entrypoint = [ "${turbocheck}/bin/turbocheck" ];
    };
  };
}
