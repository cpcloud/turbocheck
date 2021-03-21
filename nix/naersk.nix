{ sources ? import ./sources.nix }:
self: super: {
  naersk = self.callPackage sources.naersk { };
}
