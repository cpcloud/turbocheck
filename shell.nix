let
  pkgs = import ./.;
in
pkgs.mkShell {
  name = "turbocheck";
  buildInputs = with pkgs; [
    cargo-bloat
    cargo-edit
    cargo-release
    cargo-udeps
    gh
    jq
    niv
    openssl
    pkg-config
    rustToolchain
    yj
  ];
}
