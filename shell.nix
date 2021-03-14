let
  sources = import ./nix/sources.nix;
  pkgs = import sources.nixpkgs {
    overlays = [
      (import sources.nixpkgs-mozilla)
      (import ./nix/overlays/rust.nix)
      (self: super: {
        rustc = super.rustChannel.rust.override {
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
        cargo = self.rustc;
      })
    ];
  };
in
pkgs.mkShell {
  name = "turbocheck";
  buildInputs = with pkgs; [
    cargo-bloat
    cargo-edit
    cargo-udeps
    niv
    openssl
    pkg-config
    rustc
  ];
}
