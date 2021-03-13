let
  sources = import ./nix/sources.nix;

  rustChannelsOverlay = import "${sources.nixpkgs-mozilla}/rust-overlay.nix";
  rustChannelsSrcOverlay = import "${sources.nixpkgs-mozilla}/rust-src-overlay.nix";

  pkgs = import sources.nixpkgs {
    overlays = [
      rustChannelsOverlay
      rustChannelsSrcOverlay
      (self: super: {
        rustc = (self.rustChannelOf {
          rustToolchain = ./rust-toolchain;
          sha256 = "19va5fnpbqkllw35rc19q2mixrx9p3m3q5dyi0881c8rcsja7rxc";
        }).rust.override {
          extensions = [
            "clippy-preview"
            "rls-preview"
            "rustfmt-preview"
            "rust-analysis"
            "rust-analyzer-preview"
            "rust-std"
            "rust-src"
          ];
        };
      })
    ];
  };

  naersk = pkgs.callPackage sources.naersk { };
in
{
  inherit pkgs;
  turbocheck = naersk.buildPackage {
    root = ./.;
    buildInputs = with pkgs; [ openssl pkg-config ];
  };
}
