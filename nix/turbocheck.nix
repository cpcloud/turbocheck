self: super: {
  turbocheck = self.naersk.buildPackage {
    root = ../.;
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
}
