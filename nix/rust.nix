self: super: {
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
}
