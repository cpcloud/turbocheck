let
  pkgs = import ./.;
in
pkgs.mkShell {
  name = "turbocheck";
  buildInputs = with pkgs; [
    rustToolchain
    cargo-bloat
    cargo-edit
    cargo-udeps
    niv
    openssl
    pkg-config
  ];
}
