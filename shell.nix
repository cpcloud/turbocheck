let
  pkgs = (import ./.).pkgs;
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
