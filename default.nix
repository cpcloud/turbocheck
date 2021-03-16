let
  sources = import ./nix/sources.nix;
in
import sources.nixpkgs {
  overlays = [
    (import sources.fenix)
    (self: super: {
      naersk = self.callPackage sources.naersk { };
    })
    (self: super: {
      inherit (self.rust-nightly.latest)
        rustc
        cargo
        clippy-preview
        rustfmt-preview
        rust-analysis
        rust-analyzer-preview
        rust-std
        rust-src;

      rustToolchain = self.rust-nightly.latest.withComponents [
        "rustc"
        "cargo"
        "clippy-preview"
        "rustfmt-preview"
        "rust-analysis"
        "rust-analyzer-preview"
        "rust-std"
        "rust-src"
      ];
    })
    (self: super: {
      turbocheck = self.naersk.buildPackage {
        root = ./.;
        buildInputs = [ self.openssl self.pkg-config ];
      };

      turbocheckImage = self.dockerTools.buildLayeredImage {
        name = "turbocheck";
        config = {
          Entrypoint = [ "${self.turbocheck}/bin/turbocheck" ];
          Env = [
            "SSL_CERT_FILE=${self.cacert}/etc/ssl/certs/ca-bundle.crt"
          ];
        };
      };
    })
  ];
}
