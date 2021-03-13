let
  sources = import ./nix/sources.nix;
  nixpkgs-mozilla = import sources.nixpkgs-mozilla;

  pkgs = import sources.nixpkgs {
    overlays = [
      nixpkgs-mozilla
      (self: super:
        let
          channel = self.rustChannelOf {
            rustToolchain = ./rust-toolchain;
            sha256 = "19va5fnpbqkllw35rc19q2mixrx9p3m3q5dyi0881c8rcsja7rxc";
          };
          rust = channel.rust.override {
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
        in
        {
          rustc = rust;
          cargo = rust;
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
