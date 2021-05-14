self: super: {
  inherit (self.fenix.latest)
    rustc
    cargo
    clippy-preview
    rustfmt-preview
    rust-analysis
    rust-analyzer-preview
    rust-std
    rust-src;

  rustToolchain = self.fenix.latest.withComponents [
    "rustc"
    "cargo"
    "clippy-preview"
    "rustfmt-preview"
    "rust-analysis"
    "rust-analyzer-preview"
    "rust-std"
    "rust-src"
  ];
}
