let
  pkgs = import ./.;
in
pkgs.mkShell {
  name = "turbocheck";
  buildInputs = with pkgs; [
    rustToolchain
    # rustc
    # cargo
    # clippy-preview
    # rustfmt-preview
    # rust-analysis
    # rust-analyzer-preview
    # rust-std
    # rust-src
    cargo-bloat
    cargo-edit
    cargo-udeps
    niv
    openssl
    pkg-config
  ];
}
